//! Structural comparison between a TCK expected-result table cell and a
//! real `marsdb_query::Value` -- both sides convert into the same
//! [`TckValue`] shape first, so comparison is structural (label sets and
//! property maps order-independent) rather than string equality.

use std::collections::{BTreeMap, BTreeSet};

use marsdb::{PathElem, PropertyValue, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum TckScalar {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TckValue {
    Node {
        labels: BTreeSet<String>,
        /// `TckValue`, not `TckScalar` -- a node/rel property can now be a
        /// list (`PropertyValue::List`, TCK's WithOrderBy1 `(:B {list: [1,
        /// 2]})`-shaped expected cells), not just a scalar.
        props: BTreeMap<String, TckValue>,
    },
    Rel {
        rel_type: String,
        props: BTreeMap<String, TckValue>,
    },
    List(Vec<TckValue>),
    Scalar(TckScalar),
    Null,
    /// `<(:A)-[:T]->(:B)>`-shaped expected cells (TCK's own path-literal
    /// syntax) and real `Value::Path` results, both converted to this
    /// same alternating-element shape. A dedicated `PathTckElem::Edge`
    /// (not the bare `Rel` variant above) carries traversal direction,
    /// which matters for equality (`<(:A)-[:T]->(:B)>` !=
    /// `<(:A)<-[:T]-(:B)>`, same two nodes, opposite direction); a bare
    /// `[:KNOWS {...}]`-shaped value has no path context and no such
    /// concept.
    Path(Vec<PathTckElem>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathTckElem {
    Node {
        labels: BTreeSet<String>,
        props: BTreeMap<String, TckValue>,
    },
    Edge {
        rel_type: String,
        props: BTreeMap<String, TckValue>,
        /// `true` iff this edge's stored direction (`src` -> `dst`)
        /// matches the order it's walked in the path (`-[...]->`);
        /// `false` means the path walks it backward (`<-[...]-`).
        forward: bool,
    },
}

/// Equality respecting `list_order_matters` -- when `false` (the TCK's
/// "ignoring element order for lists" result form), list elements compare
/// as a multiset instead of position-by-position. Everything else
/// (node/rel prop maps, label sets) is already order-independent by
/// construction (`BTreeMap`/`BTreeSet`).
pub fn tck_eq(a: &TckValue, b: &TckValue, list_order_matters: bool) -> bool {
    match (a, b) {
        // `f64::NAN != f64::NAN` under IEEE 754, which the derived
        // `PartialEq` every other arm's `a == b` fallback relies on would
        // inherit -- TCK's "sort distinct types" scenarios just need "is
        // this classified as NaN", so special-cased here instead.
        (TckValue::Scalar(TckScalar::Float(x)), TckValue::Scalar(TckScalar::Float(y)))
            if x.is_nan() && y.is_nan() =>
        {
            true
        }
        (TckValue::List(xs), TckValue::List(ys)) => {
            if xs.len() != ys.len() {
                return false;
            }
            if list_order_matters {
                xs.iter()
                    .zip(ys)
                    .all(|(x, y)| tck_eq(x, y, list_order_matters))
            } else {
                let mut remaining: Vec<&TckValue> = ys.iter().collect();
                for x in xs {
                    let Some(pos) = remaining
                        .iter()
                        .position(|y| tck_eq(x, y, list_order_matters))
                    else {
                        return false;
                    };
                    remaining.remove(pos);
                }
                true
            }
        }
        _ => a == b,
    }
}

pub fn value_to_tck(v: &Value) -> TckValue {
    match v {
        Value::Null => TckValue::Null,
        Value::Node(n) => TckValue::Node {
            labels: n.labels.iter().cloned().collect(),
            props: n
                .props
                .iter()
                .map(|(k, v)| (k.clone(), property_to_tck(v)))
                .collect(),
        },
        Value::Edge(e) => TckValue::Rel {
            rel_type: e.label.clone(),
            props: e
                .props
                .iter()
                .map(|(k, v)| (k.clone(), property_to_tck(v)))
                .collect(),
        },
        Value::Property(p) => property_to_tck(p),
        Value::List(items) => TckValue::List(items.iter().map(value_to_tck).collect()),
        // A bare map literal used as a returned value compares the same
        // way node props already do (see `parse_map_value`). Recurses
        // through `value_to_tck` directly (not the narrower
        // `property_to_tck`) since a map's values are already full
        // `Value`s, not raw stored `PropertyValue`s.
        Value::Map(m) => TckValue::Node {
            labels: BTreeSet::new(),
            props: m
                .iter()
                .map(|(k, v)| (k.clone(), value_to_tck(v)))
                .collect(),
        },
        Value::Literal(lit) => literal_to_tck(lit),
        Value::Path(elems) => path_to_tck(elems),
    }
}

/// Walks a real `Value::Path`'s alternating `Node`/`Edge` elements into
/// the same `PathTckElem` shape `parse_path` builds from a TCK expected
/// cell's `<...>` syntax. `forward` is derived from comparing each edge's
/// stored `src` against the *preceding* node's id in the walk -- the
/// path itself carries no separate direction flag, direction is implicit
/// in which node came first.
fn path_to_tck(elems: &[PathElem]) -> TckValue {
    let mut out = Vec::with_capacity(elems.len());
    let mut prev_node_id = None;
    for elem in elems {
        match elem {
            PathElem::Node(n) => {
                prev_node_id = Some(n.id);
                out.push(PathTckElem::Node {
                    labels: n.labels.iter().cloned().collect(),
                    props: n
                        .props
                        .iter()
                        .map(|(k, v)| (k.clone(), property_to_tck(v)))
                        .collect(),
                });
            }
            PathElem::Edge(e) => {
                out.push(PathTckElem::Edge {
                    rel_type: e.label.clone(),
                    props: e
                        .props
                        .iter()
                        .map(|(k, v)| (k.clone(), property_to_tck(v)))
                        .collect(),
                    forward: prev_node_id == Some(e.src),
                });
            }
        }
    }
    TckValue::Path(out)
}

/// A raw stored `PropertyValue` (a node/edge property, or a top-level
/// `Value::Property` RETURN result) converted to the same `TckValue`
/// shape everything else compares through. `Null` maps to
/// `TckValue::Null` (mirrors `parse_props`'s convention on the
/// expected-cell side); `List` recurses per-element, since a node/edge
/// property can hold one (`PropertyValue::List`), not just a scalar.
fn property_to_tck(p: &PropertyValue) -> TckValue {
    match p {
        PropertyValue::Null => TckValue::Null,
        PropertyValue::List(items) => TckValue::List(items.iter().map(property_to_tck).collect()),
        // Same reasoning as `value_to_tck`'s own `Value::Map` arm -- a
        // map compares the same way node props already do. Only ever
        // reached for a `$parameter` echoed back into a result (e.g.
        // `RETURN $mapParam`), never a real stored node/edge property
        // (`PropertyValue::Map`'s own doc comment).
        PropertyValue::Map(m) => TckValue::Node {
            labels: BTreeSet::new(),
            props: m
                .iter()
                .map(|(k, v)| (k.clone(), property_to_tck(v)))
                .collect(),
        },
        other => TckValue::Scalar(property_to_scalar(other)),
    }
}

/// The scalar-only cases of `property_to_tck` -- split out so
/// `property_to_tck` can wrap the result in `TckValue::Scalar` once,
/// rather than every arm repeating it. Never called directly with
/// `Null`/`List`/`Map` (see `property_to_tck`'s own dispatch).
fn property_to_scalar(p: &PropertyValue) -> TckScalar {
    match p {
        PropertyValue::Null | PropertyValue::List(_) | PropertyValue::Map(_) => {
            unreachable!("property_to_tck dispatches Null/List/Map before reaching here")
        }
        PropertyValue::Bool(b) => TckScalar::Bool(*b),
        PropertyValue::Int(i) => TckScalar::Int(*i),
        PropertyValue::Float(f) => TckScalar::Float(*f),
        PropertyValue::String(s) => TckScalar::Str(s.clone()),
        // The TCK's expected-result cells write a date/duration as its
        // ISO-8601 string (`'1984-10-11'`, `'P12Y5M...'`), never a
        // distinct literal syntax -- comparing as `Str` (not adding a
        // `TckScalar::Date`/`Duration` variant) is what makes that
        // comparison line up, matching `format_property`'s equivalent
        // choice in `marsdb-cli`.
        PropertyValue::Date(d) => TckScalar::Str(marsdb::temporal::format_date(*d)),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => TckScalar::Str(marsdb::temporal::format_duration(
            *months, *days, *seconds, *nanos,
        )),
        PropertyValue::LocalTime(nanos_of_day) => {
            TckScalar::Str(marsdb::temporal::format_local_time(*nanos_of_day))
        }
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => TckScalar::Str(marsdb::temporal::format_time(
            *nanos_of_day,
            *offset_seconds,
        )),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => TckScalar::Str(marsdb::temporal::format_local_date_time(
            *epoch_seconds,
            *nanos,
        )),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => TckScalar::Str(marsdb::temporal::format_date_time(
            *epoch_seconds,
            *nanos,
            &to_temporal_tz(zone),
        )),
    }
}

/// `marsdb::TzId` <-> `marsdb::temporal::TzId` -- two independent,
/// same-shaped types (`temporal.rs` has no dependency on `marsdb_graph`),
/// converted at this formatting boundary.
fn to_temporal_tz(zone: &marsdb::TzId) -> marsdb::temporal::TzId {
    match zone {
        marsdb::TzId::Offset(o) => marsdb::temporal::TzId::Offset(*o),
        marsdb::TzId::Named(name) => marsdb::temporal::TzId::Named(name.clone()),
    }
}

fn literal_to_tck(lit: &marsdb::Literal) -> TckValue {
    use marsdb::Literal;
    match lit {
        Literal::Null => TckValue::Null,
        Literal::Bool(b) => TckValue::Scalar(TckScalar::Bool(*b)),
        Literal::Int(i) => TckValue::Scalar(TckScalar::Int(*i)),
        Literal::Float(f) => TckValue::Scalar(TckScalar::Float(*f)),
        Literal::String(s) => TckValue::Scalar(TckScalar::Str(s.clone())),
        Literal::Param(name) => TckValue::Scalar(TckScalar::Str(format!("${name}"))),
    }
}

/// Parses a TCK result-table cell's literal text (`(:B {name: 'b'})`,
/// `[:ACTED_IN {roles: ['x']}]`, `[1, 2, 3]`, `'x'`, `42`, `null`, ...)
/// into the same [`TckValue`] shape `value_to_tck` produces, so both
/// sides of a comparison are structurally comparable. A hand-rolled
/// recursive-descent parser, not a grammar dependency -- the TCK's own
/// literal syntax is small and Cypher-like, close to (but not reused
/// from) `marsdb-query`'s own literal grammar.
pub fn parse_cell(text: &str) -> Result<TckValue, String> {
    let mut p = CellParser {
        chars: text.trim().chars().collect(),
        pos: 0,
    };
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!(
            "trailing input after value: {:?}",
            p.chars[p.pos..].iter().collect::<String>()
        ));
    }
    Ok(v)
}

struct CellParser {
    chars: Vec<char>,
    pos: usize,
}

impl CellParser {
    fn skip_ws(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "expected {c:?} at position {}, got {:?}",
                self.pos,
                self.peek()
            ))
        }
    }

    fn parse_value(&mut self) -> Result<TckValue, String> {
        self.skip_ws();
        match self.peek() {
            Some('(') => self.parse_node(),
            Some('[') => self.parse_list_or_rel(),
            Some('{') => self.parse_map_value(),
            Some('<') => self.parse_path(),
            Some('\'') => Ok(TckValue::Scalar(TckScalar::Str(self.parse_string()?))),
            // `NaN`/`Infinity`/`-Infinity` are keywords in TCK's expected-
            // cell syntax, not parseable as an ordinary signed number (no
            // digits) -- tried before the general digit/`-` branch below
            // since they also start with `-`.
            Some(c) if c == '-' || c.is_ascii_digit() => {
                if self.chars[self.pos..].starts_with(&['N', 'a', 'N'])
                    || self.chars[self.pos..].starts_with(&['I', 'n', 'f', 'i', 'n', 'i', 't', 'y'])
                    || self.chars[self.pos..]
                        .starts_with(&['-', 'I', 'n', 'f', 'i', 'n', 'i', 't', 'y'])
                {
                    self.parse_keyword()
                } else {
                    self.parse_number()
                }
            }
            _ => self.parse_keyword(),
        }
    }

    /// `<node ((-[...]-> | <-[...]-) node)*>` -- TCK's own path-literal
    /// syntax (`<(:A)-[:T]->(:B)>`), matching the same `PathTckElem` shape
    /// `path_to_tck` builds from a real `Value::Path`.
    fn parse_path(&mut self) -> Result<TckValue, String> {
        self.expect('<')?;
        self.skip_ws();
        let mut elems = vec![self.parse_path_node()?];
        self.skip_ws();
        while self.peek() != Some('>') {
            elems.push(self.parse_path_edge()?);
            elems.push(self.parse_path_node()?);
            self.skip_ws();
        }
        self.expect('>')?;
        Ok(TckValue::Path(elems))
    }

    fn parse_path_node(&mut self) -> Result<PathTckElem, String> {
        let TckValue::Node { labels, props } = self.parse_node()? else {
            unreachable!("parse_node always returns TckValue::Node");
        };
        Ok(PathTckElem::Node { labels, props })
    }

    /// Either `-[:TYPE {props}]->` (forward) or `<-[:TYPE {props}]-`
    /// (backward) -- the two arrowhead shapes a path's edges appear in.
    /// An undirected `-[...]-` never appears in a matched path's written
    /// form, since a concrete walk always has a real traversed direction.
    fn parse_path_edge(&mut self) -> Result<PathTckElem, String> {
        self.skip_ws();
        let backward = self.peek() == Some('<');
        if backward {
            self.pos += 1;
        }
        self.expect('-')?;
        self.expect('[')?;
        self.expect(':')?;
        let rel_type = self.parse_identifier();
        self.skip_ws();
        let props = if self.peek() == Some('{') {
            self.parse_props()?
        } else {
            BTreeMap::new()
        };
        self.expect(']')?;
        self.expect('-')?;
        if !backward {
            self.expect('>')?;
        }
        Ok(PathTckElem::Edge {
            rel_type,
            props,
            forward: !backward,
        })
    }

    fn parse_keyword(&mut self) -> Result<TckValue, String> {
        for (kw, val) in [
            ("null", TckValue::Null),
            ("true", TckValue::Scalar(TckScalar::Bool(true))),
            ("false", TckValue::Scalar(TckScalar::Bool(false))),
            ("NaN", TckValue::Scalar(TckScalar::Float(f64::NAN))),
            (
                "-Infinity",
                TckValue::Scalar(TckScalar::Float(f64::NEG_INFINITY)),
            ),
            (
                "Infinity",
                TckValue::Scalar(TckScalar::Float(f64::INFINITY)),
            ),
        ] {
            if self.chars[self.pos..].starts_with(&kw.chars().collect::<Vec<_>>()[..]) {
                self.pos += kw.len();
                return Ok(val);
            }
        }
        Err(format!(
            "unrecognized value at position {}: {:?}",
            self.pos,
            self.chars.get(self.pos..)
        ))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect('\'')?;
        let mut s = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".to_string()),
                Some('\'') => {
                    self.pos += 1;
                    break;
                }
                Some('\\') => {
                    self.pos += 1;
                    let escaped = self.peek().ok_or("dangling escape in string")?;
                    // Matches MarsDB's own Cypher string-literal escapes
                    // (\' \" \\ \n \r \t \b \f) -- without decoding, an
                    // actual newline/tab in a query result wouldn't
                    // structurally equal the two-character `\n` sequence
                    // in the expected cell.
                    let decoded = match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'b' => '\u{8}',
                        'f' => '\u{c}',
                        other => other,
                    };
                    s.push(decoded);
                    self.pos += 1;
                }
                Some(c) => {
                    s.push(c);
                    self.pos += 1;
                }
            }
        }
        Ok(s)
    }

    fn parse_number(&mut self) -> Result<TckValue, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' && !is_float {
                is_float = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        // Scientific notation (`1e308`, `1.23456789e308`, `1e-305`) --
        // always a float even if the mantissa had no `.`. `e`/`E`
        // followed by an optional sign and at least one digit; back out
        // to just the mantissa if that shape isn't present.
        if matches!(self.peek(), Some('e' | 'E')) {
            let exp_start = self.pos;
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            let digits_start = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == digits_start {
                self.pos = exp_start;
            } else {
                is_float = true;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            text.parse::<f64>()
                .map(|f| TckValue::Scalar(TckScalar::Float(f)))
                .map_err(|e| e.to_string())
        } else {
            text.parse::<i64>()
                .map(|i| TckValue::Scalar(TckScalar::Int(i)))
                .map_err(|e| e.to_string())
        }
    }

    fn parse_identifier(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_alphanumeric() || c == '_') {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn parse_node(&mut self) -> Result<TckValue, String> {
        self.expect('(')?;
        let mut labels = BTreeSet::new();
        self.skip_ws();
        while self.peek() == Some(':') {
            self.pos += 1;
            labels.insert(self.parse_identifier());
            self.skip_ws();
        }
        let props = if self.peek() == Some('{') {
            self.parse_props()?
        } else {
            BTreeMap::new()
        };
        self.expect(')')?;
        Ok(TckValue::Node { labels, props })
    }

    /// `[:TYPE]`, `[:TYPE {props}]`, or a plain list `[v, v, ...]` --
    /// distinguished by whether the first non-whitespace char after `[`
    /// is `:` (a relationship) or not (a list).
    fn parse_list_or_rel(&mut self) -> Result<TckValue, String> {
        self.expect('[')?;
        self.skip_ws();
        if self.peek() == Some(':') {
            self.pos += 1;
            let rel_type = self.parse_identifier();
            self.skip_ws();
            // A type-union rel literal (`[:A|:B]`) can appear as an
            // expected value shape in a couple of scenarios -- only the
            // first type is kept.
            while self.peek() == Some('|') {
                self.pos += 1;
                self.skip_ws();
                if self.peek() == Some(':') {
                    self.pos += 1;
                }
                self.parse_identifier();
                self.skip_ws();
            }
            let props = if self.peek() == Some('{') {
                self.parse_props()?
            } else {
                BTreeMap::new()
            };
            self.expect(']')?;
            Ok(TckValue::Rel { rel_type, props })
        } else {
            let mut items = Vec::new();
            self.skip_ws();
            if self.peek() != Some(']') {
                loop {
                    items.push(self.parse_value()?);
                    self.skip_ws();
                    match self.peek() {
                        Some(',') => {
                            self.pos += 1;
                            continue;
                        }
                        _ => break,
                    }
                }
            }
            self.expect(']')?;
            Ok(TckValue::List(items))
        }
    }

    fn parse_map_value(&mut self) -> Result<TckValue, String> {
        // A bare map literal used as a returned value (not node/rel
        // props) -- represented the same way node props are; there's no
        // separate "Map" TckValue variant since nothing needs to tell
        // them apart.
        let props = self.parse_props()?;
        Ok(TckValue::Node {
            labels: BTreeSet::new(),
            props,
        })
    }

    /// `TckValue`, not `TckScalar` -- a node/rel prop (or a bare map
    /// entry, `parse_map_value` reuses this too) can be any value,
    /// including a `[...]` list (TCK's WithOrderBy1 `(:B {list: [1,
    /// 2]})`) or `null`.
    fn parse_props(&mut self) -> Result<BTreeMap<String, TckValue>, String> {
        self.expect('{')?;
        let mut props = BTreeMap::new();
        self.skip_ws();
        if self.peek() != Some('}') {
            loop {
                self.skip_ws();
                let key = self.parse_identifier();
                self.expect(':')?;
                let value = self.parse_value()?;
                props.insert(key, value);
                self.skip_ws();
                match self.peek() {
                    Some(',') => {
                        self.pos += 1;
                        continue;
                    }
                    _ => break,
                }
            }
        }
        self.expect('}')?;
        Ok(props)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_node() {
        assert_eq!(
            parse_cell("()").unwrap(),
            TckValue::Node {
                labels: BTreeSet::new(),
                props: BTreeMap::new()
            }
        );
    }

    #[test]
    fn parses_labeled_node_with_props() {
        let v = parse_cell("(:A:B {name: 'b', age: 30})").unwrap();
        let TckValue::Node { labels, props } = v else {
            panic!("expected a node")
        };
        assert_eq!(labels, BTreeSet::from(["A".to_string(), "B".to_string()]));
        assert_eq!(
            props.get("name"),
            Some(&TckValue::Scalar(TckScalar::Str("b".to_string())))
        );
        assert_eq!(
            props.get("age"),
            Some(&TckValue::Scalar(TckScalar::Int(30)))
        );
    }

    #[test]
    fn parses_relationship() {
        let v = parse_cell("[:KNOWS {since: 2020}]").unwrap();
        let TckValue::Rel { rel_type, props } = v else {
            panic!("expected a rel")
        };
        assert_eq!(rel_type, "KNOWS");
        assert_eq!(
            props.get("since"),
            Some(&TckValue::Scalar(TckScalar::Int(2020)))
        );
    }

    #[test]
    fn parses_nested_list() {
        let v = parse_cell("['a', 'b', 'c']").unwrap();
        let TckValue::List(items) = v else {
            panic!("expected a list")
        };
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], TckValue::Scalar(TckScalar::Str("a".to_string())));
    }

    #[test]
    fn parses_null_and_bools() {
        assert_eq!(parse_cell("null").unwrap(), TckValue::Null);
        assert_eq!(
            parse_cell("true").unwrap(),
            TckValue::Scalar(TckScalar::Bool(true))
        );
        assert_eq!(
            parse_cell("false").unwrap(),
            TckValue::Scalar(TckScalar::Bool(false))
        );
    }

    #[test]
    fn parses_negative_float() {
        assert_eq!(
            parse_cell("-3.125").unwrap(),
            TckValue::Scalar(TckScalar::Float(-3.125))
        );
    }

    #[test]
    fn parses_scientific_notation_floats() {
        assert_eq!(
            parse_cell("1e308").unwrap(),
            TckValue::Scalar(TckScalar::Float(1e308))
        );
        assert_eq!(
            parse_cell("1.23456789e308").unwrap(),
            TckValue::Scalar(TckScalar::Float(1.23456789e308))
        );
        assert_eq!(
            parse_cell("1e-305").unwrap(),
            TckValue::Scalar(TckScalar::Float(1e-305))
        );
        assert_eq!(
            parse_cell("-1e-305").unwrap(),
            TckValue::Scalar(TckScalar::Float(-1e-305))
        );
    }

    #[test]
    fn parses_nan_and_infinity_keywords() {
        let TckValue::Scalar(TckScalar::Float(f)) = parse_cell("NaN").unwrap() else {
            panic!("expected a float");
        };
        assert!(f.is_nan());
        assert_eq!(
            parse_cell("Infinity").unwrap(),
            TckValue::Scalar(TckScalar::Float(f64::INFINITY))
        );
        assert_eq!(
            parse_cell("-Infinity").unwrap(),
            TckValue::Scalar(TckScalar::Float(f64::NEG_INFINITY))
        );
    }

    #[test]
    fn nan_compares_equal_to_nan_via_tck_eq() {
        let a = TckValue::Scalar(TckScalar::Float(f64::NAN));
        let b = TckValue::Scalar(TckScalar::Float(f64::NAN));
        assert!(tck_eq(&a, &b, true));
    }

    #[test]
    fn parses_zero_length_path() {
        assert_eq!(
            parse_cell("<()>").unwrap(),
            TckValue::Path(vec![PathTckElem::Node {
                labels: BTreeSet::new(),
                props: BTreeMap::new(),
            }])
        );
    }

    #[test]
    fn parses_forward_path() {
        let v = parse_cell("<(:A)-[:T]->(:B)>").unwrap();
        let TckValue::Path(elems) = v else {
            panic!("expected a path")
        };
        assert_eq!(elems.len(), 3);
        assert!(matches!(
            &elems[1],
            PathTckElem::Edge { rel_type, forward: true, .. } if rel_type == "T"
        ));
    }

    #[test]
    fn parses_backward_path() {
        let v = parse_cell("<(:B)<-[:T]-(:A)>").unwrap();
        let TckValue::Path(elems) = v else {
            panic!("expected a path")
        };
        assert!(matches!(
            &elems[1],
            PathTckElem::Edge { rel_type, forward: false, .. } if rel_type == "T"
        ));
    }

    #[test]
    fn same_nodes_different_edge_direction_are_not_equal() {
        let forward = parse_cell("<(:A)-[:T]->(:B)>").unwrap();
        let backward = parse_cell("<(:A)<-[:T]-(:B)>").unwrap();
        assert!(!tck_eq(&forward, &backward, true));
    }

    #[test]
    fn parses_multi_hop_path_with_props() {
        let v = parse_cell(
            "<(:A {name: 'A'})-[:KNOWS {num: 1}]->(:B {name: 'B'})-[:KNOWS {num: 2}]->(:C {name: 'C'})>",
        )
        .unwrap();
        let TckValue::Path(elems) = v else {
            panic!("expected a path")
        };
        assert_eq!(elems.len(), 5);
    }

    #[test]
    fn list_order_insensitive_comparison() {
        let a = TckValue::List(vec![
            TckValue::Scalar(TckScalar::Int(1)),
            TckValue::Scalar(TckScalar::Int(2)),
        ]);
        let b = TckValue::List(vec![
            TckValue::Scalar(TckScalar::Int(2)),
            TckValue::Scalar(TckScalar::Int(1)),
        ]);
        assert!(!tck_eq(&a, &b, true));
        assert!(tck_eq(&a, &b, false));
    }
}
