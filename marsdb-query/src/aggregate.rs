//! Per-group accumulators for `count`/`sum`/`avg`/`min`/`max`/`collect`.
//! `resolve_grouped_rows` (in `executor.rs`) owns folding rows into groups
//! and driving one `AggAcc` per aggregate return item per group; this
//! module only owns what happens to a single accumulator as values arrive.

use std::collections::HashSet;

use marsdb_graph::PropertyValue;

use crate::ast::Literal;
use crate::error::QueryError;
use crate::executor::comparable_ordering;
use crate::value::{PathElem, Value};

/// A hashable, `Eq` stand-in for `Value` — `Value`/`PropertyValue` don't
/// derive `Eq`/`Hash` (IEEE floats have no reflexive equality, so Rust
/// excludes `f64`). `FloatBits` hashes/compares by bit pattern instead:
/// ordinary float grouping/dedup is unaffected (equal floats have equal
/// bits), but `NaN` groups with `NaN` here (unlike IEEE) and `+0.0`/`-0.0`
/// are distinct. `Node`/`Edge` hash by id, matching `value_eq`'s
/// convention. Used for both grouping-key lookup (`executor::
/// binding_hash_key`) and `DISTINCT`'s "seen" set.
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
    Date(i64),
    Duration(i64, i64, i64, i32),
    LocalTime(i64),
    // Keyed by UTC-equivalent instant-of-day, not raw wall-clock fields,
    // matching `Time`'s equality rule: two structurally-different `Time`s
    // at the same instant hash equal.
    TimeInstant(i64),
    LocalDateTime(i64, i32),
    // `offset_seconds` excluded, same instant-only equality as `TimeInstant`.
    DateTimeInstant(i64, i32),
}

pub(crate) fn value_hash_key(v: &Value) -> Result<HashKey, QueryError> {
    Ok(match v {
        Value::Null => HashKey::Null,
        Value::Node(n) => HashKey::Node(n.id),
        Value::Edge(e) => HashKey::Edge(e.id),
        Value::Property(pv) => property_value_hash_key(pv),
        Value::Literal(lit) => literal_hash_key(lit),
        Value::List(items) => HashKey::List(
            items
                .iter()
                .map(value_hash_key)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        // Identity is the exact node/edge sequence, matching
        // executor::binding_hash_key's `Binding::Path` arm.
        Value::Path(elems) => HashKey::List(
            elems
                .iter()
                .map(|e| match e {
                    PathElem::Node(n) => HashKey::Node(n.id),
                    PathElem::Edge(edge) => HashKey::Edge(edge.id),
                })
                .collect(),
        ),
        // `BTreeMap` already iterates in sorted key order, so this is a
        // deterministic, canonical key regardless of the map literal's
        // own written order. Each entry becomes its own 2-element
        // `HashKey::List` (key, value), wrapped in one outer list.
        Value::Map(m) => HashKey::List(
            m.iter()
                .map(|(k, v)| -> Result<HashKey, QueryError> {
                    Ok(HashKey::List(vec![
                        HashKey::Str(k.clone()),
                        value_hash_key(v)?,
                    ]))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    })
}

pub(crate) fn property_value_hash_key(pv: &PropertyValue) -> HashKey {
    match pv {
        PropertyValue::Null => HashKey::Null,
        PropertyValue::Bool(b) => HashKey::Bool(*b),
        PropertyValue::Int(i) => HashKey::Int(*i),
        PropertyValue::Float(f) => HashKey::FloatBits(f.to_bits()),
        PropertyValue::String(s) => HashKey::Str(s.clone()),
        PropertyValue::Date(d) => HashKey::Date(*d),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => HashKey::Duration(*months, *days, *seconds, *nanos),
        PropertyValue::LocalTime(nanos_of_day) => HashKey::LocalTime(*nanos_of_day),
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => HashKey::TimeInstant(nanos_of_day - *offset_seconds as i64 * 1_000_000_000),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => HashKey::LocalDateTime(*epoch_seconds, *nanos),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            ..
        } => HashKey::DateTimeInstant(*epoch_seconds, *nanos),
        PropertyValue::List(items) => {
            HashKey::List(items.iter().map(property_value_hash_key).collect())
        }
        // Same encoding as `value_hash_key`'s own `Value::Map` arm above --
        // each entry becomes its own 2-element `HashKey::List` (key,
        // value), all wrapped in one outer list.
        PropertyValue::Map(m) => HashKey::List(
            m.iter()
                .map(|(k, v)| {
                    HashKey::List(vec![HashKey::Str(k.clone()), property_value_hash_key(v)])
                })
                .collect(),
        ),
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
    Count {
        distinct: Option<HashSet<HashKey>>,
        n: i64,
    },
    Sum {
        distinct: Option<HashSet<HashKey>>,
        total_int: i64,
        total_float: f64,
        saw_float: bool,
    },
    Avg {
        distinct: Option<HashSet<HashKey>>,
        total: f64,
        n: i64,
    },
    Min {
        distinct: Option<HashSet<HashKey>>,
        best: Option<Value>,
    },
    Max {
        distinct: Option<HashSet<HashKey>>,
        best: Option<Value>,
    },
    Collect {
        distinct: Option<HashSet<HashKey>>,
        items: Vec<Value>,
    },
    /// Always emits a `Float` (real Cypher's documented behavior --
    /// interpolating between two ranks can't stay an `Int` even when every
    /// input was), unlike `PercentileDisc` below.
    PercentileCont {
        distinct: Option<HashSet<HashKey>>,
        values: Vec<f64>,
        percentile: Option<f64>,
    },
    /// Keeps each folded value's original `Value` (not just its numeric
    /// magnitude) -- `percentileDisc` always returns one of its actual
    /// inputs verbatim (an `Int` input stays an `Int`), unlike
    /// `PercentileCont`'s interpolation.
    PercentileDisc {
        distinct: Option<HashSet<HashKey>>,
        values: Vec<Value>,
        percentile: Option<f64>,
    },
}

enum Numeric {
    Int(i64),
    Float(f64),
}

fn numeric_value(v: &Value) -> Option<Numeric> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => {
            Some(Numeric::Int(*i))
        }
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => {
            Some(Numeric::Float(*f))
        }
        _ => None,
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Node(_) => "a node",
        Value::Edge(_) => "an edge",
        Value::List(_) => "a list",
        Value::Map(_) => "a map",
        Value::Path(_) => "a path",
        Value::Property(_) | Value::Literal(_) => "a scalar",
        Value::Null => "null",
    }
}

/// True iff `v` hasn't been seen before in this accumulator's DISTINCT set
/// (and records it if so) — `HashKey` is what makes this an O(1) average
/// hash-set insert instead of a linear rescan-and-compare per value.
fn dedup_seen(distinct: &mut Option<HashSet<HashKey>>, v: &Value) -> Result<bool, QueryError> {
    Ok(match distinct {
        None => true,
        Some(seen) => seen.insert(value_hash_key(v)?),
    })
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
            "percentilecont" => AggAcc::PercentileCont {
                distinct: d(),
                values: Vec::new(),
                percentile: None,
            },
            "percentiledisc" => AggAcc::PercentileDisc {
                distinct: d(),
                values: Vec::new(),
                percentile: None,
            },
            other => unreachable!(
                "AggAcc::identity called with non-aggregate name {other:?} — is_aggregate_name should have rejected this earlier"
            ),
        }
    }

    /// Folds one row's already-evaluated argument value into the
    /// accumulator. Callers must skip calling this for `Value::Null`
    /// (standard Cypher null-skipping) — `fold` never sees `Value::Null`.
    /// This is what makes `count(x)` exclude an `OPTIONAL MATCH`
    /// non-match while `count(*)`, computed separately, includes it.
    pub(crate) fn fold(&mut self, v: &Value) -> Result<(), QueryError> {
        debug_assert!(
            !matches!(v, Value::Null),
            "callers must skip Value::Null before calling fold"
        );
        match self {
            AggAcc::Count { distinct, n } => {
                if dedup_seen(distinct, v)? {
                    *n = n
                        .checked_add(1)
                        .ok_or_else(|| QueryError::Type("count() overflow".into()))?;
                }
            }
            AggAcc::Sum {
                distinct,
                total_int,
                total_float,
                saw_float,
            } => {
                if !dedup_seen(distinct, v)? {
                    return Ok(());
                }
                match numeric_value(v) {
                    Some(Numeric::Int(i)) => {
                        *total_int = total_int
                            .checked_add(i)
                            .ok_or_else(|| QueryError::Type("sum() integer overflow".into()))?;
                    }
                    Some(Numeric::Float(f)) => {
                        *saw_float = true;
                        *total_float += f;
                    }
                    None => {
                        return Err(QueryError::Type(format!(
                            "sum() requires a numeric argument, got {}",
                            value_type_name(v)
                        )))
                    }
                }
            }
            AggAcc::Avg { distinct, total, n } => {
                if !dedup_seen(distinct, v)? {
                    return Ok(());
                }
                let f = match numeric_value(v) {
                    Some(Numeric::Int(i)) => i as f64,
                    Some(Numeric::Float(f)) => f,
                    None => {
                        return Err(QueryError::Type(format!(
                            "avg() requires a numeric argument, got {}",
                            value_type_name(v)
                        )))
                    }
                };
                *total += f;
                *n = n
                    .checked_add(1)
                    .ok_or_else(|| QueryError::Type("avg() count overflow".into()))?;
            }
            AggAcc::Min { distinct, best } => {
                if !dedup_seen(distinct, v)? {
                    return Ok(());
                }
                if comparable_ordering(v, v).is_none() {
                    return Err(QueryError::Type(format!(
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
                if !dedup_seen(distinct, v)? {
                    return Ok(());
                }
                if comparable_ordering(v, v).is_none() {
                    return Err(QueryError::Type(format!(
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
                if dedup_seen(distinct, v)? {
                    items.push(v.clone());
                }
            }
            AggAcc::PercentileCont { .. } | AggAcc::PercentileDisc { .. } => unreachable!(
                "percentileCont()/percentileDisc() take two arguments -- callers must use \
                 fold_percentile, not fold"
            ),
        }
        Ok(())
    }

    /// `count()`-only fold of a node/relationship reference by graph
    /// identity, bypassing `fold`'s materialized-`Value` interface:
    /// `count` never inspects the record, so an entity deleted earlier in
    /// the same statement (`... DELETE p RETURN count(p)`) still counts,
    /// while every record-touching path (`count(p.prop)`, `RETURN p`
    /// itself) keeps erroring via `deleted_entity_access`. `key` is the
    /// same `HashKey::Node`/`Edge` `value_hash_key` derives from a
    /// materialized entity, so `count(DISTINCT p)` dedups identically on
    /// both paths.
    pub(crate) fn fold_count_entity(&mut self, key: HashKey) -> Result<(), QueryError> {
        match self {
            AggAcc::Count { distinct, n } => {
                let fresh = match distinct {
                    None => true,
                    Some(seen) => seen.insert(key),
                };
                if fresh {
                    *n = n
                        .checked_add(1)
                        .ok_or_else(|| QueryError::Type("count() overflow".into()))?;
                }
                Ok(())
            }
            _ => unreachable!(
                "fold_count_entity is only ever called on a count() accumulator — \
                 resolve_grouped_rows checks the aggregate name first"
            ),
        }
    }

    /// Folds one row's (value, percentile) pair for `percentileCont()`/
    /// `percentileDisc()` — the only two-argument aggregates, so they
    /// can't share `fold`'s single-`Value` interface. Same null-skipping
    /// convention as `fold`. The percentile is validated (numeric,
    /// `0.0..=1.0`) on every call rather than just the first, since
    /// nothing here assumes it's constant across the group.
    pub(crate) fn fold_percentile(
        &mut self,
        value: &Value,
        percentile: &Value,
    ) -> Result<(), QueryError> {
        debug_assert!(
            !matches!(value, Value::Null),
            "callers must skip Value::Null before calling fold_percentile"
        );
        let p = match numeric_value(percentile) {
            Some(Numeric::Int(i)) => i as f64,
            Some(Numeric::Float(f)) => f,
            None => {
                return Err(QueryError::Type(format!(
                "percentileCont()/percentileDisc()'s percentile argument must be numeric, got {}",
                value_type_name(percentile)
            )))
            }
        };
        if !(0.0..=1.0).contains(&p) {
            return Err(QueryError::Type(format!(
                "percentileCont()/percentileDisc()'s percentile argument must be between 0.0 and \
                 1.0, got {p}"
            )));
        }
        match self {
            AggAcc::PercentileCont {
                distinct,
                values,
                percentile,
            } => {
                *percentile = Some(p);
                if !dedup_seen(distinct, value)? {
                    return Ok(());
                }
                match numeric_value(value) {
                    Some(Numeric::Int(i)) => values.push(i as f64),
                    Some(Numeric::Float(f)) => values.push(f),
                    None => {
                        return Err(QueryError::Type(format!(
                            "percentileCont() requires a numeric argument, got {}",
                            value_type_name(value)
                        )))
                    }
                }
            }
            AggAcc::PercentileDisc {
                distinct,
                values,
                percentile,
            } => {
                *percentile = Some(p);
                if numeric_value(value).is_none() {
                    return Err(QueryError::Type(format!(
                        "percentileDisc() requires a numeric argument, got {}",
                        value_type_name(value)
                    )));
                }
                if dedup_seen(distinct, value)? {
                    values.push(value.clone());
                }
            }
            _ => unreachable!("fold_percentile called on a non-percentile accumulator"),
        }
        Ok(())
    }

    /// Finishes the accumulator into its group's output `Value`, including
    /// the zero-contributing-rows case: `count` -> 0, `sum` -> 0,
    /// `avg`/`min`/`max` -> `Null`, `collect` -> `[]`.
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
            AggAcc::PercentileCont {
                mut values,
                percentile,
                ..
            } => {
                if values.is_empty() {
                    return Value::Null;
                }
                // No contributing row means no percentile either -- but
                // `values` can't be non-empty without one, since
                // `fold_percentile` always sets it alongside pushing a
                // value.
                let p = percentile.expect("non-empty values implies a captured percentile");
                values.sort_by(f64::total_cmp);
                let n = values.len();
                let rank = p * (n as f64 - 1.0);
                let lower = rank.floor() as usize;
                let upper = rank.ceil() as usize;
                let result = if lower == upper {
                    values[lower]
                } else {
                    let weight = rank - lower as f64;
                    values[lower] + (values[upper] - values[lower]) * weight
                };
                Value::Property(PropertyValue::Float(result))
            }
            AggAcc::PercentileDisc {
                values, percentile, ..
            } => {
                if values.is_empty() {
                    return Value::Null;
                }
                let p = percentile.expect("non-empty values implies a captured percentile");
                let mut ranked: Vec<(f64, Value)> = values
                    .into_iter()
                    .map(|v| {
                        let key = match numeric_value(&v) {
                            Some(Numeric::Int(i)) => i as f64,
                            Some(Numeric::Float(f)) => f,
                            // `fold_percentile` already rejected any
                            // non-numeric value before it could reach here.
                            None => unreachable!(
                                "percentileDisc() accumulator holds a non-numeric value"
                            ),
                        };
                        (key, v)
                    })
                    .collect();
                ranked.sort_by(|a, b| a.0.total_cmp(&b.0));
                let n = ranked.len();
                let idx = ((p * n as f64).ceil() as isize - 1).max(0) as usize;
                let idx = idx.min(n - 1);
                ranked.into_iter().nth(idx).expect("idx < n").1
            }
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
        assert!(matches!(
            acc.finish(),
            Value::Property(PropertyValue::Int(3))
        ));
    }

    #[test]
    fn integer_sum_overflow_is_an_error() {
        let mut acc = AggAcc::identity("sum", false);
        acc.fold(&int(i64::MAX)).unwrap();
        let err = acc.fold(&int(1)).unwrap_err();
        assert!(err.to_string().contains("sum() integer overflow"));
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
        assert!(matches!(
            AggAcc::identity("count", false).finish(),
            Value::Property(PropertyValue::Int(0))
        ));
        assert!(matches!(
            AggAcc::identity("sum", false).finish(),
            Value::Property(PropertyValue::Int(0))
        ));
        assert!(matches!(
            AggAcc::identity("avg", false).finish(),
            Value::Null
        ));
        assert!(matches!(
            AggAcc::identity("min", false).finish(),
            Value::Null
        ));
        assert!(matches!(
            AggAcc::identity("max", false).finish(),
            Value::Null
        ));
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
        assert!(matches!(
            acc.finish(),
            Value::Property(PropertyValue::Int(2))
        ));
    }

    #[test]
    fn min_max_on_non_orderable_errors() {
        // `Map` has no defined order (see `comparable_ordering`), so it's
        // a convenient non-orderable stand-in without constructing a real
        // Node/Edge.
        let node = Value::Map(std::collections::BTreeMap::new());
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
        assert!(matches!(
            min_acc.finish(),
            Value::Property(PropertyValue::Int(1))
        ));
        assert!(matches!(
            max_acc.finish(),
            Value::Property(PropertyValue::Int(9))
        ));
    }
}
