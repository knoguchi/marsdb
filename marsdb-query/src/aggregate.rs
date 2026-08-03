//! Per-group accumulators for `count`/`sum`/`avg`/`min`/`max`/`collect`.
//! `resolve_grouped_rows` (in `executor.rs`) owns folding rows into groups
//! and driving one `AggAcc` per aggregate return item per group; this
//! module only owns what happens to a single accumulator as values arrive.

use std::collections::HashSet;

use marsdb_graph::PropertyValue;

use crate::ast::Literal;
use crate::error::QueryError;
use crate::executor::comparable_ordering;
use crate::value::Value;

/// A hashable, `Eq` stand-in for `Value` — `Value`/`PropertyValue` don't
/// derive `Eq`/`Hash` themselves (`PropertyValue::Float(f64)` can't: IEEE
/// floats have no reflexive equality, so Rust's std deliberately excludes
/// `Eq`/`Hash` for `f64`). `FloatBits` hashes/compares by bit pattern
/// instead — the practical trade-off every DB doing this makes: ordinary
/// float grouping/dedup is completely unaffected (equal floats have equal
/// bits), the only visible difference is at the edges (`NaN` groups with
/// `NaN` here, unlike IEEE `NaN != NaN`; `+0.0`/`-0.0` are distinct here,
/// unlike IEEE `==`) — the same class of documented trade-off as the
/// label index and the linear-scan grouping this type replaces. `Node`/
/// `Edge` hash by id (graph identity), matching `value_eq`'s existing
/// convention. Used for both `resolve_grouped_rows`' grouping-key lookup
/// (`executor.rs`, via `binding_hash_key`) and `DISTINCT`'s "seen" set
/// below — same underlying problem, same fix.
#[derive(PartialEq, Eq, Hash)]
pub(crate) enum HashKey {
    Node(marsdb_graph::NodeId),
    Edge(marsdb_graph::EdgeId),
    Null,
    Bool(bool),
    Int(i64),
    FloatBits(u64),
    Str(String),
    List(Vec<HashKey>),
}

pub(crate) fn value_hash_key(v: &Value) -> HashKey {
    match v {
        Value::Null => HashKey::Null,
        Value::Node(n) => HashKey::Node(n.id),
        Value::Edge(e) => HashKey::Edge(e.id),
        Value::Property(pv) => property_value_hash_key(pv),
        Value::Literal(lit) => literal_hash_key(lit),
        Value::List(items) => HashKey::List(items.iter().map(value_hash_key).collect()),
    }
}

pub(crate) fn property_value_hash_key(pv: &PropertyValue) -> HashKey {
    match pv {
        PropertyValue::Null => HashKey::Null,
        PropertyValue::Bool(b) => HashKey::Bool(*b),
        PropertyValue::Int(i) => HashKey::Int(*i),
        PropertyValue::Float(f) => HashKey::FloatBits(f.to_bits()),
        PropertyValue::String(s) => HashKey::Str(s.clone()),
    }
}

fn literal_hash_key(lit: &Literal) -> HashKey {
    match lit {
        Literal::Null => HashKey::Null,
        Literal::Bool(b) => HashKey::Bool(*b),
        Literal::Int(i) => HashKey::Int(*i),
        Literal::Float(f) => HashKey::FloatBits(f.to_bits()),
        Literal::String(s) => HashKey::Str(s.clone()),
        Literal::Param(name) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
    }
}

/// Running accumulator for one `count`/`sum`/`avg`/`min`/`max`/`collect`
/// return item within one group. `count(*)` has no accumulator — it's
/// computed directly as the group's row count, independent of any per-row
/// argument (see `resolve_grouped_rows`), so it isn't represented here.
pub(crate) enum AggAcc {
    Count { distinct: Option<HashSet<HashKey>>, n: i64 },
    Sum { distinct: Option<HashSet<HashKey>>, total_int: i64, total_float: f64, saw_float: bool },
    Avg { distinct: Option<HashSet<HashKey>>, total: f64, n: i64 },
    Min { distinct: Option<HashSet<HashKey>>, best: Option<Value> },
    Max { distinct: Option<HashSet<HashKey>>, best: Option<Value> },
    Collect { distinct: Option<HashSet<HashKey>>, items: Vec<Value> },
}

enum Numeric {
    Int(i64),
    Float(f64),
}

fn numeric_value(v: &Value) -> Option<Numeric> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => Some(Numeric::Int(*i)),
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => Some(Numeric::Float(*f)),
        _ => None,
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Node(_) => "a node",
        Value::Edge(_) => "an edge",
        Value::List(_) => "a list",
        Value::Property(_) | Value::Literal(_) => "a scalar",
        Value::Null => "null",
    }
}

/// True iff `v` hasn't been seen before in this accumulator's DISTINCT set
/// (and records it if so) — `HashKey` is what makes this an O(1) average
/// hash-set insert instead of a linear rescan-and-compare per value.
fn dedup_seen(distinct: &mut Option<HashSet<HashKey>>, v: &Value) -> bool {
    match distinct {
        None => true,
        Some(seen) => seen.insert(value_hash_key(v)),
    }
}

impl AggAcc {
    /// `name` must satisfy `is_aggregate_name` — callers (`resolve_grouped_rows`)
    /// only ever construct one for a return item already classified as an
    /// aggregate call by `has_aggregate`/`validate_return_items`.
    pub(crate) fn identity(name: &str, distinct: bool) -> Self {
        let d = || if distinct { Some(HashSet::new()) } else { None };
        match name.to_ascii_lowercase().as_str() {
            "count" => AggAcc::Count { distinct: d(), n: 0 },
            "sum" => AggAcc::Sum {
                distinct: d(),
                total_int: 0,
                total_float: 0.0,
                saw_float: false,
            },
            "avg" => AggAcc::Avg { distinct: d(), total: 0.0, n: 0 },
            "min" => AggAcc::Min { distinct: d(), best: None },
            "max" => AggAcc::Max { distinct: d(), best: None },
            "collect" => AggAcc::Collect { distinct: d(), items: Vec::new() },
            other => unreachable!(
                "AggAcc::identity called with non-aggregate name {other:?} — is_aggregate_name should have rejected this earlier"
            ),
        }
    }

    /// Folds one row's already-evaluated argument value into the
    /// accumulator. Callers must skip calling this entirely for
    /// `Value::Null` — standard Cypher null-skipping, and what makes
    /// `count(x)` exclude an `OPTIONAL MATCH` non-match while `count(*)`
    /// (computed separately, not through this accumulator at all)
    /// includes it. `fold` never sees `Value::Null`.
    pub(crate) fn fold(&mut self, v: &Value) -> Result<(), QueryError> {
        debug_assert!(!matches!(v, Value::Null), "callers must skip Value::Null before calling fold");
        match self {
            AggAcc::Count { distinct, n } => {
                if dedup_seen(distinct, v) {
                    *n += 1;
                }
            }
            AggAcc::Sum {
                distinct,
                total_int,
                total_float,
                saw_float,
            } => {
                if !dedup_seen(distinct, v) {
                    return Ok(());
                }
                match numeric_value(v) {
                    Some(Numeric::Int(i)) => *total_int += i,
                    Some(Numeric::Float(f)) => {
                        *saw_float = true;
                        *total_float += f;
                    }
                    None => {
                        return Err(QueryError::Parse(format!(
                            "sum() requires a numeric argument, got {}",
                            value_type_name(v)
                        )))
                    }
                }
            }
            AggAcc::Avg { distinct, total, n } => {
                if !dedup_seen(distinct, v) {
                    return Ok(());
                }
                let f = match numeric_value(v) {
                    Some(Numeric::Int(i)) => i as f64,
                    Some(Numeric::Float(f)) => f,
                    None => {
                        return Err(QueryError::Parse(format!(
                            "avg() requires a numeric argument, got {}",
                            value_type_name(v)
                        )))
                    }
                };
                *total += f;
                *n += 1;
            }
            AggAcc::Min { distinct, best } => {
                if !dedup_seen(distinct, v) {
                    return Ok(());
                }
                if comparable_ordering(v, v).is_none() {
                    return Err(QueryError::Parse(format!(
                        "min() requires a comparable scalar argument, got {}",
                        value_type_name(v)
                    )));
                }
                let replace = match best {
                    None => true,
                    Some(cur) => comparable_ordering(v, cur) == Some(std::cmp::Ordering::Less),
                };
                if replace {
                    *best = Some(v.clone());
                }
            }
            AggAcc::Max { distinct, best } => {
                if !dedup_seen(distinct, v) {
                    return Ok(());
                }
                if comparable_ordering(v, v).is_none() {
                    return Err(QueryError::Parse(format!(
                        "max() requires a comparable scalar argument, got {}",
                        value_type_name(v)
                    )));
                }
                let replace = match best {
                    None => true,
                    Some(cur) => comparable_ordering(v, cur) == Some(std::cmp::Ordering::Greater),
                };
                if replace {
                    *best = Some(v.clone());
                }
            }
            AggAcc::Collect { distinct, items } => {
                if dedup_seen(distinct, v) {
                    items.push(v.clone());
                }
            }
        }
        Ok(())
    }

    /// Finishes the accumulator into its group's output `Value`. Called
    /// once a group's rows are fully folded — includes the zero-contributing-
    /// rows case (e.g. every row in the group had a null argument, or the
    /// group itself is the single synthesized empty-result group — see
    /// `resolve_grouped_rows`): `count` -> 0, `sum` -> 0, `avg`/`min`/`max`
    /// -> `Null`, `collect` -> `[]`, matching real Cypher's documented
    /// empty-aggregate behavior.
    pub(crate) fn finish(self) -> Value {
        match self {
            AggAcc::Count { n, .. } => Value::Property(PropertyValue::Int(n)),
            AggAcc::Sum {
                total_int,
                total_float,
                saw_float,
                ..
            } => {
                if saw_float {
                    Value::Property(PropertyValue::Float(total_int as f64 + total_float))
                } else {
                    Value::Property(PropertyValue::Int(total_int))
                }
            }
            AggAcc::Avg { total, n, .. } => {
                if n == 0 {
                    Value::Null
                } else {
                    Value::Property(PropertyValue::Float(total / n as f64))
                }
            }
            AggAcc::Min { best, .. } | AggAcc::Max { best, .. } => best.unwrap_or(Value::Null),
            AggAcc::Collect { items, .. } => Value::List(items),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(i: i64) -> Value {
        Value::Property(PropertyValue::Int(i))
    }
    fn float(f: f64) -> Value {
        Value::Property(PropertyValue::Float(f))
    }

    #[test]
    fn sum_promotes_to_float_when_any_input_is_float() {
        let mut acc = AggAcc::identity("sum", false);
        acc.fold(&int(1)).unwrap();
        acc.fold(&int(2)).unwrap();
        acc.fold(&float(1.5)).unwrap();
        match acc.finish() {
            Value::Property(PropertyValue::Float(f)) => assert!((f - 4.5).abs() < 1e-9),
            other => panic!("expected float sum, got {other:?}"),
        }
    }

    #[test]
    fn sum_stays_int_when_all_inputs_are_int() {
        let mut acc = AggAcc::identity("sum", false);
        acc.fold(&int(1)).unwrap();
        acc.fold(&int(2)).unwrap();
        assert!(matches!(acc.finish(), Value::Property(PropertyValue::Int(3))));
    }

    #[test]
    fn avg_always_returns_float() {
        let mut acc = AggAcc::identity("avg", false);
        acc.fold(&int(2)).unwrap();
        acc.fold(&int(4)).unwrap();
        match acc.finish() {
            Value::Property(PropertyValue::Float(f)) => assert!((f - 3.0).abs() < 1e-9),
            other => panic!("expected float avg, got {other:?}"),
        }
    }

    #[test]
    fn empty_contribution_results_match_cypher_conventions() {
        assert!(matches!(AggAcc::identity("count", false).finish(), Value::Property(PropertyValue::Int(0))));
        assert!(matches!(AggAcc::identity("sum", false).finish(), Value::Property(PropertyValue::Int(0))));
        assert!(matches!(AggAcc::identity("avg", false).finish(), Value::Null));
        assert!(matches!(AggAcc::identity("min", false).finish(), Value::Null));
        assert!(matches!(AggAcc::identity("max", false).finish(), Value::Null));
        match AggAcc::identity("collect", false).finish() {
            Value::List(items) => assert!(items.is_empty()),
            other => panic!("expected empty list, got {other:?}"),
        }
    }

    #[test]
    fn count_distinct_dedupes() {
        let mut acc = AggAcc::identity("count", true);
        acc.fold(&int(1)).unwrap();
        acc.fold(&int(1)).unwrap();
        acc.fold(&int(2)).unwrap();
        assert!(matches!(acc.finish(), Value::Property(PropertyValue::Int(2))));
    }

    #[test]
    fn min_max_on_non_orderable_errors() {
        let node = Value::List(vec![]); // non-orderable stand-in, avoids constructing a real Node/Edge in a unit test
        assert!(AggAcc::identity("min", false).fold(&node).is_err());
        assert!(AggAcc::identity("max", false).fold(&node).is_err());
    }

    #[test]
    fn min_max_track_extremes() {
        let mut min_acc = AggAcc::identity("min", false);
        let mut max_acc = AggAcc::identity("max", false);
        for v in [int(5), int(1), int(9), int(3)] {
            min_acc.fold(&v).unwrap();
            max_acc.fold(&v).unwrap();
        }
        assert!(matches!(min_acc.finish(), Value::Property(PropertyValue::Int(1))));
        assert!(matches!(max_acc.finish(), Value::Property(PropertyValue::Int(9))));
    }
}
