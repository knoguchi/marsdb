//! Scalar builtin functions (string/list/math/type coercions) plus
//! list indexing/slicing -- everything `call_builtin` dispatches to that
//! isn't temporal.

use super::*;

/// `list[index]` -- a negative index counts from the end (`-1` is the
/// last element). Out of bounds either way is `Null`, not an error --
/// matches real Cypher (`[1,2,3][10]` is `null`, not a failure), and is
/// the only sane behavior for an index that's itself a runtime expression
/// rather than a literal a human could sanity-check up front.
pub(crate) fn apply_index(list: &Value, index: &Value) -> Result<Value, QueryError> {
    if matches!(list, Value::Null) || matches!(index, Value::Null) {
        return Ok(Value::Null);
    }
    // `map[key]` -- real Cypher's dynamic map-field access (`map['name']`,
    // as opposed to `map.name`'s static form -- `lookup_prop`/`ReturnExpr
    // ::Prop` above). Unlike `.prop`, this can return a full nested
    // `Value` (a list/map field value), not just a scalar `PropertyValue`
    // -- `apply_index`'s return type already allows that, no narrowing
    // needed the way `map_value_as_property` has to for `.prop`.
    if let Value::Map(entries) = list {
        let Some(key) = as_arith_str(index) else {
            return Err(QueryError::Type(format!(
                "a map index must be a string, got {index:?}"
            )));
        };
        return Ok(entries.get(key).cloned().unwrap_or(Value::Null));
    }
    // `n['name']` -- dynamic property access on a node/relationship/
    // temporal value, same as `n.name`'s static form but with a computed
    // key (TCK's Graph7 `[1]`-`[3]`). Reuses `property_of_value` exactly
    // -- the only actual difference from `.prop` is where the key string
    // comes from.
    if matches!(list, Value::Node(_) | Value::Edge(_) | Value::Property(_)) {
        let Some(key) = as_arith_str(index) else {
            return Err(QueryError::Type(format!(
                "a property index must be a string, got {index:?}"
            )));
        };
        return property_of_value(list, key);
    }
    let Value::List(items) = list else {
        return Err(QueryError::Type(format!(
            "[] indexing needs a list or map, got {list:?}"
        )));
    };
    let Some(ArithNum::Int(i)) = as_arith_num(index) else {
        return Err(QueryError::Type(format!(
            "a list index must be an integer, got {index:?}"
        )));
    };
    let len = items.len() as i64;
    let i = if i < 0 { i + len } else { i };
    if i < 0 || i >= len {
        return Ok(Value::Null);
    }
    Ok(items[i as usize].clone())
}

/// `list[start..end]` -- same negative-counts-from-end rule as
/// `apply_index`, but bounds clamp to `[0, len]` instead of nulling out
/// (`[1,2,3][-5..5]` is the whole list, not `null`), and a start at or
/// past the (clamped) end yields `[]` rather than erroring
/// (`[1,2,3][3..1]` is `[]`) -- both match real Cypher, and both were
/// real TCK scenarios, not guessed behavior.
pub(crate) fn apply_slice(
    list: &Value,
    start: Option<&Value>,
    end: Option<&Value>,
) -> Result<Value, QueryError> {
    if matches!(list, Value::Null) {
        return Ok(Value::Null);
    }
    let Value::List(items) = list else {
        return Err(QueryError::Type(format!(
            "[..] slicing needs a list, got {list:?}"
        )));
    };
    let len = items.len() as i64;
    let clamp = |i: i64| -> i64 {
        let i = if i < 0 { i + len } else { i };
        i.clamp(0, len)
    };
    let bound_index = |v: Option<&Value>, default: i64| -> Result<Option<i64>, QueryError> {
        match v {
            None => Ok(Some(default)),
            Some(Value::Null) => Ok(None),
            Some(other) => match as_arith_num(other) {
                Some(ArithNum::Int(i)) => Ok(Some(clamp(i))),
                _ => Err(QueryError::Type(format!(
                    "a slice bound must be an integer, got {other:?}"
                ))),
            },
        }
    };
    // A null bound (as opposed to an *omitted* one, already handled by
    // `start`/`end` being `None` at the AST level) propagates -- same
    // null-handling convention as every other operator here.
    let (Some(start_idx), Some(end_idx)) = (bound_index(start, 0)?, bound_index(end, len)?) else {
        return Ok(Value::Null);
    };
    if start_idx >= end_idx {
        return Ok(Value::List(Vec::new()));
    }
    Ok(Value::List(
        items[start_idx as usize..end_idx as usize].to_vec(),
    ))
}

pub(crate) fn call_builtin(
    name: &str,
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    match name.to_ascii_lowercase().as_str() {
        "coalesce" => Ok(args
            .iter()
            .find(|v| !matches!(v, Value::Null))
            .cloned()
            .unwrap_or(Value::Null)),
        "tointeger" => match args.first() {
            Some(v) => to_integer(v),
            None => Ok(Value::Null),
        },
        "tostring" => match args.first() {
            Some(v) => to_string_value(v),
            None => Ok(Value::Null),
        },
        "date" => date_builtin(args, now),
        "date.transaction" | "date.statement" | "date.realtime" => Ok(now_or_null(args, || {
            Value::Property(PropertyValue::Date(now.epoch_day))
        })),
        "duration" => duration_builtin(args),
        "localtime" => local_time_builtin(args, now),
        "localtime.transaction" | "localtime.statement" | "localtime.realtime" => {
            Ok(now_or_null(args, || {
                Value::Property(PropertyValue::LocalTime(now.nanos_of_day))
            }))
        }
        "time" => time_builtin(args, now),
        "time.transaction" | "time.statement" | "time.realtime" => {
            // No-arg time() defaults to UTC offset (real Cypher's statement default timezone)
            Ok(now_or_null(args, || {
                Value::Property(PropertyValue::Time {
                    nanos_of_day: now.nanos_of_day,
                    offset_seconds: 0,
                })
            }))
        }
        "localdatetime" => local_date_time_builtin(args, now),
        "localdatetime.transaction" | "localdatetime.statement" | "localdatetime.realtime" => {
            Ok(now_or_null(args, || {
                Value::Property(PropertyValue::LocalDateTime {
                    epoch_seconds: now.epoch_seconds,
                    nanos: now.nanos,
                })
            }))
        }
        "datetime" => date_time_builtin(args, now),
        "datetime.transaction" | "datetime.statement" | "datetime.realtime" => {
            // No-arg datetime() defaults to UTC offset (real Cypher's statement default timezone)
            Ok(now_or_null(args, || {
                Value::Property(PropertyValue::DateTime {
                    epoch_seconds: now.epoch_seconds,
                    nanos: now.nanos,
                    zone: GraphTzId::Offset(0),
                })
            }))
        }
        "datetime.fromepoch" => {
            let seconds = require_int_arg(args.first(), "datetime.fromepoch")?;
            let nanos = require_int_arg(args.get(1), "datetime.fromepoch")?;
            Ok(Value::Property(PropertyValue::DateTime {
                epoch_seconds: seconds,
                nanos: nanos as i32,
                zone: GraphTzId::Offset(0),
            }))
        }
        "datetime.fromepochmillis" => {
            let millis = require_int_arg(args.first(), "datetime.fromepochmillis")?;
            Ok(Value::Property(PropertyValue::DateTime {
                epoch_seconds: millis.div_euclid(1000),
                nanos: (millis.rem_euclid(1000) * 1_000_000) as i32,
                zone: GraphTzId::Offset(0),
            }))
        }
        "duration.between" => {
            duration_between_builtin("duration.between", args, temporal::duration_between)
        }
        "duration.inmonths" => {
            duration_between_builtin("duration.inMonths", args, temporal::duration_in_months)
        }
        "duration.indays" => {
            duration_between_builtin("duration.inDays", args, temporal::duration_in_days)
        }
        "duration.inseconds" => {
            duration_between_builtin("duration.inSeconds", args, temporal::duration_in_seconds)
        }
        "date.truncate" => date_truncate_builtin(args),
        "localtime.truncate" => local_time_truncate_builtin(args),
        "time.truncate" => time_truncate_builtin(args),
        "localdatetime.truncate" => local_date_time_truncate_builtin(args),
        "datetime.truncate" => date_time_truncate_builtin(args),
        // The dominant real-world use of shortestPath() is measuring it
        // (degrees-of-separation queries), not returning/rendering the
        // raw path object — path elements alternate node/edge/.../node,
        // so edge count is (elements.len() - 1) / 2.
        "length" => Ok(match args.first() {
            Some(Value::Path(elems)) => {
                Value::Property(PropertyValue::Int(((elems.len().max(1) - 1) / 2) as i64))
            }
            Some(Value::Null) | None => Value::Null,
            Some(other) => {
                return Err(QueryError::Type(format!(
                    "length() expects a path, got {other:?}"
                )))
            }
        }),
        "keys" => keys_builtin(args.first()),
        "labels" => labels_builtin(args.first()),
        "type" => type_builtin(args.first()),
        "properties" => properties_builtin(args.first()),
        "id" => id_builtin(args.first()),
        "size" => size_builtin(args.first()),
        "nodes" => nodes_builtin(args.first()),
        "relationships" => relationships_builtin(args.first()),
        "head" => list_edge_builtin(args.first(), "head", |items| items.first().cloned()),
        "last" => list_edge_builtin(args.first(), "last", |items| items.last().cloned()),
        "tail" => match args.first() {
            Some(Value::List(items)) => Ok(Value::List(
                items.iter().skip(1).cloned().collect::<Vec<_>>(),
            )),
            Some(Value::Null) | None => Ok(Value::Null),
            Some(other) => Err(QueryError::Type(format!(
                "tail() expects a list, got {other:?}"
            ))),
        },
        "range" => range_builtin(args),
        "exists" => Ok(Value::Literal(Literal::Bool(!matches!(
            args.first(),
            None | Some(Value::Null)
        )))),
        "toupper" | "upper" => string_transform(args.first(), "toUpper", str::to_uppercase),
        "tolower" | "lower" => string_transform(args.first(), "toLower", str::to_lowercase),
        "trim" => string_transform(args.first(), "trim", |s| s.trim().to_string()),
        "ltrim" => string_transform(args.first(), "ltrim", |s| s.trim_start().to_string()),
        "rtrim" => string_transform(args.first(), "rtrim", |s| s.trim_end().to_string()),
        "reverse" => reverse_builtin(args.first()),
        "replace" => replace_builtin(args),
        "split" => split_builtin(args),
        "substring" => substring_builtin(args),
        "left" => left_right_builtin(args, true),
        "right" => left_right_builtin(args, false),
        "tofloat" => match args.first() {
            Some(v) => to_float(v),
            None => Ok(Value::Null),
        },
        "toboolean" => match args.first() {
            Some(v) => to_boolean(v),
            None => Ok(Value::Null),
        },
        "abs" => match args.first() {
            Some(Value::Property(PropertyValue::Int(i)))
            | Some(Value::Literal(Literal::Int(i))) => {
                Ok(Value::Property(PropertyValue::Int(i.abs())))
            }
            Some(Value::Null) | None => Ok(Value::Null),
            Some(other) => match value_as_f64(other) {
                Some(f) => Ok(Value::Property(PropertyValue::Float(f.abs()))),
                None => Err(QueryError::Type(format!(
                    "abs() expects a number, got {other:?}"
                ))),
            },
        },
        "ceil" => float_math_fn(args.first(), "ceil", f64::ceil),
        "floor" => float_math_fn(args.first(), "floor", f64::floor),
        "round" => float_math_fn(args.first(), "round", f64::round),
        "sqrt" => float_math_fn(args.first(), "sqrt", f64::sqrt),
        "sign" => match args.first() {
            Some(Value::Null) | None => Ok(Value::Null),
            Some(other) => match value_as_f64(other) {
                Some(f) => Ok(Value::Property(PropertyValue::Int(if f > 0.0 {
                    1
                } else if f < 0.0 {
                    -1
                } else {
                    0
                }))),
                None => Err(QueryError::Type(format!(
                    "sign() expects a number, got {other:?}"
                ))),
            },
        },
        "rand" => Ok(Value::Property(PropertyValue::Float(rand_f64()))),
        other => Err(QueryError::Semantic(format!("unknown function: {other}"))),
    }
}

/// `rand()` -- a fresh pseudo-random `f64` in `[0, 1)` on every call (no
/// memoization like `now()`/`date()`'s `NowSnapshot` -- real Cypher's
/// `rand()` is independently random each time it's evaluated, even
/// multiple times in the same query). No external RNG crate: combines an
/// atomic per-process counter with `RandomState`'s own already-randomized
/// per-construction seed (the same source `HashMap`'s DoS-resistant
/// default hasher draws from), good enough for a general-purpose
/// `rand()` without pulling in a dependency for one function.
pub(crate) fn rand_f64() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    let bits = hasher.finish();
    (bits >> 11) as f64 / (1u64 << 53) as f64
}

pub(crate) fn keys_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Node(n)) => Value::List(
            n.props
                .keys()
                .map(|k| Value::Property(PropertyValue::String(k.clone())))
                .collect(),
        ),
        Some(Value::Edge(e)) => Value::List(
            e.props
                .keys()
                .map(|k| Value::Property(PropertyValue::String(k.clone())))
                .collect(),
        ),
        Some(Value::Map(m)) => Value::List(
            m.keys()
                .map(|k| Value::Property(PropertyValue::String(k.clone())))
                .collect(),
        ),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "keys() expects a node, relationship, or map, got {other:?}"
            )))
        }
    })
}

pub(crate) fn labels_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Node(n)) => Value::List(
            n.labels
                .iter()
                .map(|l| Value::Property(PropertyValue::String(l.clone())))
                .collect(),
        ),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "labels() expects a node, got {other:?}"
            )))
        }
    })
}

pub(crate) fn type_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Edge(e)) => Value::Property(PropertyValue::String(e.label.clone())),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "type() expects a relationship, got {other:?}"
            )))
        }
    })
}

pub(crate) fn properties_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Node(n)) => Value::Map(
            n.props
                .iter()
                .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
                .collect(),
        ),
        Some(Value::Edge(e)) => Value::Map(
            e.props
                .iter()
                .map(|(k, v)| (k.clone(), property_value_to_value(v.clone())))
                .collect(),
        ),
        Some(Value::Map(m)) => Value::Map(m.clone()),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "properties() expects a node, relationship, or map, got {other:?}"
            )))
        }
    })
}

pub(crate) fn id_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Node(n)) => Value::Property(PropertyValue::Int(n.id.0 as i64)),
        Some(Value::Edge(e)) => Value::Property(PropertyValue::Int(e.id.0 as i64)),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "id() expects a node or relationship, got {other:?}"
            )))
        }
    })
}

pub(crate) fn size_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::List(items)) => Value::Property(PropertyValue::Int(items.len() as i64)),
        Some(Value::Null) | None => Value::Null,
        Some(other) => match as_arith_str(other) {
            Some(s) => Value::Property(PropertyValue::Int(s.chars().count() as i64)),
            None => {
                return Err(QueryError::Type(format!(
                    "size() expects a list or string, got {other:?}"
                )))
            }
        },
    })
}

pub(crate) fn nodes_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Path(elems)) => Value::List(
            elems
                .iter()
                .filter_map(|e| match e {
                    PathElem::Node(n) => Some(Value::Node(n.clone())),
                    PathElem::Edge(_) => None,
                })
                .collect(),
        ),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "nodes() expects a path, got {other:?}"
            )))
        }
    })
}

pub(crate) fn relationships_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Path(elems)) => Value::List(
            elems
                .iter()
                .filter_map(|e| match e {
                    PathElem::Edge(e) => Some(Value::Edge(e.clone())),
                    PathElem::Node(_) => None,
                })
                .collect(),
        ),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "relationships() expects a path, got {other:?}"
            )))
        }
    })
}

/// Shared shape for `head()`/`last()` -- `[]` (an empty list) is `null`,
/// same as any other out-of-bounds list access in this codebase
/// (`apply_index`'s docs), not an error.
pub(crate) fn list_edge_builtin(
    arg: Option<&Value>,
    fn_name: &str,
    pick: impl Fn(&[Value]) -> Option<Value>,
) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::List(items)) => pick(items).unwrap_or(Value::Null),
        Some(Value::Null) | None => Value::Null,
        Some(other) => {
            return Err(QueryError::Type(format!(
                "{fn_name}() expects a list, got {other:?}"
            )))
        }
    })
}

/// `range(start, end[, step])` -- both bounds inclusive (real Cypher's own
/// convention, unlike Rust's exclusive-end ranges), `step` defaults to 1
/// and may be negative for a descending range. A zero step has no
/// sensible iteration direction -- a real error, not an infinite/empty
/// silent result.
pub(crate) fn range_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let int_arg = |v: &Value, which: &str| -> Result<i64, QueryError> {
        value_as_i64(v).ok_or_else(|| {
            QueryError::Type(format!("range()'s {which} must be an integer, got {v:?}"))
        })
    };
    let start = int_arg(
        args.first()
            .ok_or_else(|| QueryError::Semantic("range() requires at least 2 arguments".into()))?,
        "start",
    )?;
    let end = int_arg(
        args.get(1)
            .ok_or_else(|| QueryError::Semantic("range() requires at least 2 arguments".into()))?,
        "end",
    )?;
    let step = match args.get(2) {
        Some(v) => int_arg(v, "step")?,
        None => 1,
    };
    if step == 0 {
        return Err(QueryError::Type("range()'s step can't be 0".into()));
    }
    let mut out = Vec::new();
    let mut i = start;
    if step > 0 {
        while i <= end {
            out.push(Value::Property(PropertyValue::Int(i)));
            i += step;
        }
    } else {
        while i >= end {
            out.push(Value::Property(PropertyValue::Int(i)));
            i += step;
        }
    }
    Ok(Value::List(out))
}

pub(crate) fn string_transform(
    arg: Option<&Value>,
    fn_name: &str,
    f: impl FnOnce(&str) -> String,
) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Null) | None => Value::Null,
        Some(other) => match as_arith_str(other) {
            Some(s) => Value::Property(PropertyValue::String(f(s))),
            None => {
                return Err(QueryError::Type(format!(
                    "{fn_name}() expects a string, got {other:?}"
                )))
            }
        },
    })
}

pub(crate) fn reverse_builtin(arg: Option<&Value>) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Null) | None => Value::Null,
        Some(Value::List(items)) => Value::List(items.iter().rev().cloned().collect()),
        Some(other) => match as_arith_str(other) {
            Some(s) => Value::Property(PropertyValue::String(s.chars().rev().collect())),
            None => {
                return Err(QueryError::Type(format!(
                    "reverse() expects a string or list, got {other:?}"
                )))
            }
        },
    })
}

/// A closure can't express `str_arg`'s independent-lifetimes signature
/// (the returned `&str` borrows from `v`, not `which`) the way a real
/// `fn` item can -- see `replace_builtin`'s only caller of this.
pub(crate) fn replace_str_arg<'a>(v: &'a Value, which: &str) -> Result<&'a str, QueryError> {
    as_arith_str(v)
        .ok_or_else(|| QueryError::Type(format!("replace()'s {which} must be a string, got {v:?}")))
}

pub(crate) fn replace_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if args.iter().any(|v| matches!(v, Value::Null)) {
        return Ok(Value::Null);
    }
    let original = replace_str_arg(
        args.first()
            .ok_or_else(|| QueryError::Semantic("replace() requires 3 arguments".into()))?,
        "original",
    )?;
    let search = replace_str_arg(
        args.get(1)
            .ok_or_else(|| QueryError::Semantic("replace() requires 3 arguments".into()))?,
        "search",
    )?;
    let replacement = replace_str_arg(
        args.get(2)
            .ok_or_else(|| QueryError::Semantic("replace() requires 3 arguments".into()))?,
        "replacement",
    )?;
    Ok(Value::Property(PropertyValue::String(
        original.replace(search, replacement),
    )))
}

pub(crate) fn split_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if args.iter().any(|v| matches!(v, Value::Null)) {
        return Ok(Value::Null);
    }
    let s = args
        .first()
        .and_then(as_arith_str)
        .ok_or_else(|| QueryError::Type("split()'s first argument must be a string".into()))?;
    let delim = args
        .get(1)
        .and_then(as_arith_str)
        .ok_or_else(|| QueryError::Type("split()'s second argument must be a string".into()))?;
    let parts = if delim.is_empty() {
        s.split("").filter(|p| !p.is_empty()).collect::<Vec<_>>()
    } else {
        s.split(delim).collect::<Vec<_>>()
    };
    Ok(Value::List(
        parts
            .into_iter()
            .map(|p| Value::Property(PropertyValue::String(p.to_string())))
            .collect(),
    ))
}

/// `substring(s, start[, length])` -- 0-indexed, both `start` and
/// `length` clamp to the string's bounds rather than erroring (matches
/// real Cypher: an out-of-range `substring` call is well-defined, not a
/// failure). Indexes by Unicode scalar (`char`), not byte offset, so a
/// multi-byte character never gets split.
pub(crate) fn substring_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if matches!(args.first(), Some(Value::Null)) {
        return Ok(Value::Null);
    }
    let s = args
        .first()
        .and_then(as_arith_str)
        .ok_or_else(|| QueryError::Type("substring()'s first argument must be a string".into()))?;
    let chars: Vec<char> = s.chars().collect();
    let start = args
        .get(1)
        .and_then(value_as_i64)
        .ok_or_else(|| QueryError::Type("substring()'s start must be an integer".into()))?
        .max(0) as usize;
    let start = start.min(chars.len());
    let end = match args.get(2) {
        Some(v) => {
            let len = value_as_i64(v)
                .ok_or_else(|| QueryError::Type("substring()'s length must be an integer".into()))?
                .max(0) as usize;
            (start + len).min(chars.len())
        }
        None => chars.len(),
    };
    Ok(Value::Property(PropertyValue::String(
        chars[start..end].iter().collect(),
    )))
}

/// `left(s, n)`/`right(s, n)` -- the first/last `n` characters, clamped
/// to the string's length rather than erroring on an over-long `n`.
pub(crate) fn left_right_builtin(args: &[Value], from_left: bool) -> Result<Value, QueryError> {
    if matches!(args.first(), Some(Value::Null)) {
        return Ok(Value::Null);
    }
    let fn_name = if from_left { "left" } else { "right" };
    let s = args.first().and_then(as_arith_str).ok_or_else(|| {
        QueryError::Type(format!("{fn_name}()'s first argument must be a string"))
    })?;
    let n = args
        .get(1)
        .and_then(value_as_i64)
        .ok_or_else(|| {
            QueryError::Type(format!("{fn_name}()'s second argument must be an integer"))
        })?
        .max(0) as usize;
    let chars: Vec<char> = s.chars().collect();
    let n = n.min(chars.len());
    let slice = if from_left {
        &chars[..n]
    } else {
        &chars[chars.len() - n..]
    };
    Ok(Value::Property(PropertyValue::String(
        slice.iter().collect(),
    )))
}

pub(crate) fn float_math_fn(
    arg: Option<&Value>,
    fn_name: &str,
    f: impl FnOnce(f64) -> f64,
) -> Result<Value, QueryError> {
    Ok(match arg {
        Some(Value::Null) | None => Value::Null,
        Some(other) => match value_as_f64(other) {
            Some(x) => Value::Property(PropertyValue::Float(f(x))),
            None => {
                return Err(QueryError::Type(format!(
                    "{fn_name}() expects a number, got {other:?}"
                )))
            }
        },
    })
}

pub(crate) fn to_float(v: &Value) -> Result<Value, QueryError> {
    Ok(match v {
        Value::Property(PropertyValue::Int(i)) => Value::Property(PropertyValue::Float(*i as f64)),
        Value::Property(PropertyValue::Float(f)) => Value::Property(PropertyValue::Float(*f)),
        Value::Literal(Literal::Int(i)) => Value::Property(PropertyValue::Float(*i as f64)),
        Value::Literal(Literal::Float(f)) => Value::Property(PropertyValue::Float(*f)),
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => {
            match s.trim().parse::<f64>() {
                Ok(f) => Value::Property(PropertyValue::Float(f)),
                Err(_) => Value::Null,
            }
        }
        Value::Property(PropertyValue::Null) | Value::Literal(Literal::Null) | Value::Null => {
            Value::Null
        }
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        // `Bool` is a real, deliberate type error, not `null` -- unlike
        // an unparseable *string*, which real Cypher does treat as
        // `null` (a string always at least plausibly *could* be numeric
        // text), a boolean never could be (TCK's TypeConversion3 [6]).
        other => {
            return Err(QueryError::Type(format!(
                "toFloat() cannot convert {other:?} to a float"
            )))
        }
    })
}

pub(crate) fn to_boolean(v: &Value) -> Result<Value, QueryError> {
    Ok(match v {
        Value::Property(PropertyValue::Bool(b)) | Value::Literal(Literal::Bool(b)) => {
            Value::Literal(Literal::Bool(*b))
        }
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => {
            match s.trim().to_ascii_lowercase().as_str() {
                "true" => Value::Literal(Literal::Bool(true)),
                "false" => Value::Literal(Literal::Bool(false)),
                _ => Value::Null,
            }
        }
        Value::Property(PropertyValue::Null) | Value::Literal(Literal::Null) | Value::Null => {
            Value::Null
        }
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        other => {
            return Err(QueryError::Type(format!(
                "toBoolean() cannot convert {other:?} to a boolean"
            )))
        }
    })
}

/// A quantifier's own per-element truthiness check when it has no `WHERE`
/// at all (`ANY(x IN list)`, not `ANY(x IN list WHERE ...)`) -- three-
/// valued, same as a real `WHERE` predicate: `null` propagates as
/// "unknown" (`None`), a literal bool passes through, anything else
/// (non-bool, non-null) is definitely-false, same convention `CASE`'s
/// subject-less `WHEN` branch already uses for a non-bool test value.
pub(crate) fn item_truthy(v: &Value) -> Option<bool> {
    match v {
        Value::Null => None,
        Value::Literal(Literal::Bool(b)) | Value::Property(PropertyValue::Bool(b)) => Some(*b),
        _ => Some(false),
    }
}

/// Real Cypher quantifiers use three-valued logic, not a simple count --
/// a single definite `true`/`false` among the elements can already decide
/// the answer even in the presence of other `null` elements, and only
/// "no definite answer, but at least one unknown" actually yields `null`.
/// Confirmed against the real TCK scenarios (Quantifier1-4, scenario 10,
/// "... on lists containing nulls") rather than assumed -- a first version
/// of this collapsed `null` predicates to `false`, which silently passed
/// every non-null-list scenario but produced 19 real wrong answers on
/// exactly these null-list cases.
pub(crate) fn eval_quantifier(kind: QuantifierKind, preds: &[Option<bool>]) -> Option<bool> {
    let true_count = preds.iter().filter(|p| **p == Some(true)).count();
    let any_false = preds.contains(&Some(false));
    let any_null = preds.iter().any(|p| p.is_none());
    match kind {
        QuantifierKind::Any => {
            if true_count > 0 {
                Some(true)
            } else if any_null {
                None
            } else {
                Some(false)
            }
        }
        QuantifierKind::None => {
            if true_count > 0 {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true)
            }
        }
        QuantifierKind::All => {
            if any_false {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true)
            }
        }
        QuantifierKind::Single => {
            if true_count >= 2 {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true_count == 1)
            }
        }
    }
}

pub(crate) fn to_integer(v: &Value) -> Result<Value, QueryError> {
    // A float-formatted string ('1.7', '2.9') isn't an i64, but real
    // Cypher's toInteger() still accepts it -- parse as a float and
    // truncate, same as the Float arm below, rather than failing straight
    // to null the way a bare `i64::parse` would (found via a real TCK
    // scenario: `toInteger('1.7')` must be `1`, not `null`).
    let as_str_parse = |s: &str| match s.trim().parse::<i64>() {
        Ok(i) => Value::Property(PropertyValue::Int(i)),
        Err(_) => match s.trim().parse::<f64>() {
            Ok(f) => Value::Property(PropertyValue::Int(f as i64)),
            Err(_) => Value::Null,
        },
    };
    Ok(match v {
        Value::Property(PropertyValue::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Property(PropertyValue::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Property(PropertyValue::String(s)) => as_str_parse(s),
        Value::Literal(Literal::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Literal(Literal::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Literal(Literal::String(s)) => as_str_parse(s),
        Value::Property(PropertyValue::Bool(_) | PropertyValue::Null)
        | Value::Literal(Literal::Bool(_) | Literal::Null)
        | Value::Null => Value::Null,
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        // A node/edge/list/map/path has no numeric conversion at all -- a
        // real error (found via a real TCK scenario expecting exactly
        // this), not a silent null the way an out-of-range/unparseable
        // scalar is.
        Value::Property(
            PropertyValue::Date(_)
            | PropertyValue::Duration { .. }
            | PropertyValue::LocalTime(_)
            | PropertyValue::Time { .. }
            | PropertyValue::LocalDateTime { .. }
            | PropertyValue::DateTime { .. }
            | PropertyValue::List(_)
            | PropertyValue::Map(_),
        )
        | Value::Node(_)
        | Value::Edge(_)
        | Value::List(_)
        | Value::Map(_)
        | Value::Path(_) => {
            return Err(QueryError::Type(format!(
                "toInteger() cannot convert {v:?} to an integer"
            )))
        }
    })
}

/// `toString(...)` — Int/Float/Bool render the same as their `Display`
/// impl already does elsewhere (`marsdb-cli`'s `format_property`/
/// `format_literal`); `Date`/`Duration` go through `temporal::format_*`.
/// Null propagates, while graph, collection, map, and path values are a
/// runtime type error rather than silently becoming null (TypeConversion4
/// scenario [10]).
pub(crate) fn to_string_value(v: &Value) -> Result<Value, QueryError> {
    let s = match v {
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => s.clone(),
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => i.to_string(),
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => {
            f.to_string()
        }
        Value::Property(PropertyValue::Bool(b)) | Value::Literal(Literal::Bool(b)) => b.to_string(),
        Value::Property(PropertyValue::Date(d)) => temporal::format_date(*d),
        Value::Property(PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => temporal::format_duration(*months, *days, *seconds, *nanos),
        Value::Property(PropertyValue::LocalTime(nanos_of_day)) => {
            temporal::format_local_time(*nanos_of_day)
        }
        Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => temporal::format_time(*nanos_of_day, *offset_seconds),
        Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => temporal::format_local_date_time(*epoch_seconds, *nanos),
        Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => temporal::format_date_time(*epoch_seconds, *nanos, &tz_from_graph(zone)),
        Value::Property(PropertyValue::Null) | Value::Literal(Literal::Null) | Value::Null => {
            return Ok(Value::Null);
        }
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        Value::Property(PropertyValue::List(_) | PropertyValue::Map(_))
        | Value::Node(_)
        | Value::Edge(_)
        | Value::List(_)
        | Value::Map(_)
        | Value::Path(_) => {
            return Err(QueryError::Type(format!(
                "toString() cannot convert {v:?} to a string"
            )))
        }
    };
    Ok(Value::Property(PropertyValue::String(s)))
}
