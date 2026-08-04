//! Structural comparison between a TCK expected-result table cell and a
//! real `marsdb_query::Value` -- both sides convert into the same
//! [`TckValue`] shape first, so comparison is structural (label sets and
//! property maps order-independent) rather than string equality, which
//! would be fragile against harmless formatting differences.

use std::collections::{BTreeMap, BTreeSet};

use marsdb::{PropertyValue, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum TckScalar {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TckValue {
    Node { labels: BTreeSet<String>, props: BTreeMap<String, TckScalar> },
    Rel { rel_type: String, props: BTreeMap<String, TckScalar> },
    List(Vec<TckValue>),
    Scalar(TckScalar),
    Null,
}

/// Equality respecting `list_order_matters` -- when `false` (the TCK's
/// "ignoring element order for lists" result form), list elements compare
/// as a multiset instead of position-by-position. Everything else
/// (node/rel prop maps, label sets) is already order-independent by
/// construction (`BTreeMap`/`BTreeSet`).
pub fn tck_eq(a: &TckValue, b: &TckValue, list_order_matters: bool) -> bool {
    match (a, b) {
        (TckValue::List(xs), TckValue::List(ys)) => {
            if xs.len() != ys.len() {
                return false;
            }
            if list_order_matters {
                xs.iter().zip(ys).all(|(x, y)| tck_eq(x, y, list_order_matters))
            } else {
                let mut remaining: Vec<&TckValue> = ys.iter().collect();
                for x in xs {
                    let Some(pos) = remaining.iter().position(|y| tck_eq(x, y, list_order_matters)) else {
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
            props: n.props.iter().map(|(k, v)| (k.clone(), property_to_scalar(v))).collect(),
        },
        Value::Edge(e) => TckValue::Rel {
            rel_type: e.label.clone(),
            props: e.props.iter().map(|(k, v)| (k.clone(), property_to_scalar(v))).collect(),
        },
        Value::Property(p) => match p {
            PropertyValue::Null => TckValue::Null,
            other => TckValue::Scalar(property_to_scalar(other)),
        },
        Value::List(items) => TckValue::List(items.iter().map(value_to_tck).collect()),
        Value::Literal(lit) => literal_to_tck(lit),
        Value::Path(_) => TckValue::Scalar(TckScalar::Str("<path -- not TCK-comparable in v1>".to_string())),
        // A bare map value returned directly (not as a node/rel's props)
        // -- same shape `parse_cell`'s `parse_map_value` produces for the
        // TCK's expected-result side, so the two sides compare correctly
        // structurally. Only scalar-valued entries are TCK-comparable
        // this way (matching `parse_props`' own restriction below); a
        // nested map/list value falls back to a `null` placeholder rather
        // than failing outright, since MarsDB's own map-typed RETURN
        // values are already a niche case (see `Value::Map`'s docs) not
        // otherwise TCK-relevant.
        Value::Map(entries) => TckValue::Node {
            labels: BTreeSet::new(),
            props: entries
                .iter()
                .map(|(k, v)| {
                    let scalar = match value_to_tck(v) {
                        TckValue::Scalar(s) => s,
                        _ => TckScalar::Str("null".to_string()),
                    };
                    (k.clone(), scalar)
                })
                .collect(),
        },
    }
}

fn property_to_scalar(p: &PropertyValue) -> TckScalar {
    match p {
        PropertyValue::Null => TckScalar::Str("null".to_string()), // unreachable in practice, see callers
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
        PropertyValue::Duration { months, days, seconds, nanos } => {
            TckScalar::Str(marsdb::temporal::format_duration(*months, *days, *seconds, *nanos))
        }
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
    let mut p = CellParser { chars: text.trim().chars().collect(), pos: 0 };
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(format!("trailing input after value: {:?}", &p.chars[p.pos..].iter().collect::<String>()));
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
            Err(format!("expected {c:?} at position {}, got {:?}", self.pos, self.peek()))
        }
    }

    fn parse_value(&mut self) -> Result<TckValue, String> {
        self.skip_ws();
        match self.peek() {
            Some('(') => self.parse_node(),
            Some('[') => self.parse_list_or_rel(),
            Some('{') => self.parse_map_value(),
            Some('\'') => Ok(TckValue::Scalar(TckScalar::Str(self.parse_string()?))),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ => self.parse_keyword(),
        }
    }

    fn parse_keyword(&mut self) -> Result<TckValue, String> {
        for (kw, val) in [
            ("null", TckValue::Null),
            ("true", TckValue::Scalar(TckScalar::Bool(true))),
            ("false", TckValue::Scalar(TckScalar::Bool(false))),
        ] {
            if self.chars[self.pos..].starts_with(&kw.chars().collect::<Vec<_>>()[..]) {
                self.pos += kw.len();
                return Ok(val);
            }
        }
        Err(format!("unrecognized value at position {}: {:?}", self.pos, self.chars.get(self.pos..)))
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
                    // Real escape decoding, matching MarsDB's own Cypher
                    // string-literal parser (\' \" \\ \n \r \t \b \f) --
                    // TCK expected-result cells use the same escapes (e.g.
                    // `'\nFoo\n'`), and a real query result's actual
                    // newline/tab won't structurally equal the two-
                    // character sequence backslash+n otherwise.
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
        let text: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            text.parse::<f64>().map(|f| TckValue::Scalar(TckScalar::Float(f))).map_err(|e| e.to_string())
        } else {
            text.parse::<i64>().map(|i| TckValue::Scalar(TckScalar::Int(i))).map_err(|e| e.to_string())
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
        let props = if self.peek() == Some('{') { self.parse_props()? } else { BTreeMap::new() };
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
            // first type is kept, adequate for v1 comparison purposes.
            while self.peek() == Some('|') {
                self.pos += 1;
                self.skip_ws();
                if self.peek() == Some(':') {
                    self.pos += 1;
                }
                self.parse_identifier();
                self.skip_ws();
            }
            let props = if self.peek() == Some('{') { self.parse_props()? } else { BTreeMap::new() };
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
        // them apart for v1 comparison purposes.
        let props = self.parse_props()?;
        Ok(TckValue::Node { labels: BTreeSet::new(), props })
    }

    fn parse_props(&mut self) -> Result<BTreeMap<String, TckScalar>, String> {
        self.expect('{')?;
        let mut props = BTreeMap::new();
        self.skip_ws();
        if self.peek() != Some('}') {
            loop {
                self.skip_ws();
                let key = self.parse_identifier();
                self.expect(':')?;
                let value = self.parse_value()?;
                let scalar = match value {
                    TckValue::Scalar(s) => s,
                    TckValue::Null => TckScalar::Str("null".to_string()),
                    other => return Err(format!("unsupported non-scalar prop value: {other:?}")),
                };
                props.insert(key, scalar);
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
        assert_eq!(parse_cell("()").unwrap(), TckValue::Node { labels: BTreeSet::new(), props: BTreeMap::new() });
    }

    #[test]
    fn parses_labeled_node_with_props() {
        let v = parse_cell("(:A:B {name: 'b', age: 30})").unwrap();
        let TckValue::Node { labels, props } = v else { panic!("expected a node") };
        assert_eq!(labels, BTreeSet::from(["A".to_string(), "B".to_string()]));
        assert_eq!(props.get("name"), Some(&TckScalar::Str("b".to_string())));
        assert_eq!(props.get("age"), Some(&TckScalar::Int(30)));
    }

    #[test]
    fn parses_relationship() {
        let v = parse_cell("[:KNOWS {since: 2020}]").unwrap();
        let TckValue::Rel { rel_type, props } = v else { panic!("expected a rel") };
        assert_eq!(rel_type, "KNOWS");
        assert_eq!(props.get("since"), Some(&TckScalar::Int(2020)));
    }

    #[test]
    fn parses_nested_list() {
        let v = parse_cell("['a', 'b', 'c']").unwrap();
        let TckValue::List(items) = v else { panic!("expected a list") };
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], TckValue::Scalar(TckScalar::Str("a".to_string())));
    }

    #[test]
    fn parses_null_and_bools() {
        assert_eq!(parse_cell("null").unwrap(), TckValue::Null);
        assert_eq!(parse_cell("true").unwrap(), TckValue::Scalar(TckScalar::Bool(true)));
        assert_eq!(parse_cell("false").unwrap(), TckValue::Scalar(TckScalar::Bool(false)));
    }

    #[test]
    fn parses_negative_float() {
        assert_eq!(parse_cell("-3.14").unwrap(), TckValue::Scalar(TckScalar::Float(-3.14)));
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
