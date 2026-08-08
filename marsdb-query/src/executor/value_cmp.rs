//! Value comparison, ordering, equality, dedup, and projected-expression
//! evaluation -- the pure-value half of the executor (no graph access).

use super::*;

/// Three-valued: `None` is Cypher's "unknown", not `false` -- any
/// comparison touching a null (a missing property, or a literal `null` on
/// either side) is unknown, always, regardless of operator -- including
/// `Eq` (`x = null` is unknown, never true, same as real Cypher; it is
/// *not* how `x`'s own missing-ness is tested -- there's no `IS NULL`
/// operator yet). Callers combine this with `and3`/`or3`/`Option::map`
/// (for `NOT`) rather than unwrapping early, so unknown propagates
/// correctly through `AND`/`OR`/`NOT` instead of collapsing to `false`.
pub(crate) fn compare(prop: &Option<PropertyValue>, op: CompareOp, lit: &Literal) -> Option<bool> {
    let Some(prop) = prop else { return None };
    if matches!(prop, PropertyValue::Null) || matches!(lit, Literal::Null) {
        return None;
    }
    compare_property_pair(prop, op, &literal_to_value(lit))
}

/// Same null-handling as `compare()`, but both sides are a looked-up
/// property (`Expr::PropCompare` -- `a.id = b.id`) instead of one side
/// being a fixed `Literal`.
pub(crate) fn compare_property_pair_opt(
    a: &Option<PropertyValue>,
    op: CompareOp,
    b: &Option<PropertyValue>,
) -> Option<bool> {
    let (Some(a), Some(b)) = (a, b) else {
        return None;
    };
    if matches!(a, PropertyValue::Null) || matches!(b, PropertyValue::Null) {
        return None;
    }
    compare_property_pair(a, op, b)
}

/// The actual per-type comparison rules, shared by `compare()`
/// (`PropertyValue` vs a `Literal`, reduced to a `PropertyValue` via
/// `literal_to_value`) and `compare_values` (two arbitrary `Value`s,
/// each reduced to a `PropertyValue` via `value_to_property_value`) --
/// both callers have already handled the "either side is null" case
/// before reaching here. Returns `Option<bool>`, not `bool` -- a
/// type-mismatched pair (`1 < 'a'`) isn't a uniform "false" the way an
/// earlier version of this function had it: real Cypher's `=`/`<>` on
/// mismatched types is a definite `false`/`true` (never equal, so
/// "not equal" is true), but ordering (`<`/`<=`/`>`/`>=`) on mismatched
/// types is `null` (no defined ordering exists to be definite about) --
/// confirmed against real TCK scenarios (`'1.0' < 1.0` is `null`, not
/// `false`; `NaN <> 'a'` is `true`, not `false`), not assumed.
pub(crate) fn compare_property_pair(
    a: &PropertyValue,
    op: CompareOp,
    b: &PropertyValue,
) -> Option<bool> {
    match (a, b) {
        (PropertyValue::Int(a), PropertyValue::Int(b)) => Some(cmp_ord(op, *a, *b)),
        (PropertyValue::Int(a), PropertyValue::Float(b)) => Some(cmp_f64(op, *a as f64, *b)),
        (PropertyValue::Float(a), PropertyValue::Float(b)) => Some(cmp_f64(op, *a, *b)),
        (PropertyValue::Float(a), PropertyValue::Int(b)) => Some(cmp_f64(op, *a, *b as f64)),
        (PropertyValue::String(a), PropertyValue::String(b)) => Some(match op {
            CompareOp::StartsWith => a.starts_with(b.as_str()),
            CompareOp::EndsWith => a.ends_with(b.as_str()),
            CompareOp::Contains => a.contains(b.as_str()),
            _ => cmp_ord(op, a.as_str(), b.as_str()),
        }),
        // Real Cypher defines boolean ordering (`false < true`), same as
        // Rust's own `bool: PartialOrd` -- confirmed via a real TCK
        // scenario (`Quantifier7 :: [3]`) that specifically compares two
        // boolean expressions with `<=`.
        (PropertyValue::Bool(a), PropertyValue::Bool(b)) => Some(cmp_ord(op, *a, *b)),
        // `Date` had no arm here at all before -- fell through to the
        // generic mismatch fallback below, which always answers
        // `Eq -> false`/`Ne -> true` regardless of the actual values, so
        // `WHERE a.date = b.date` on two genuinely-equal stored dates
        // incorrectly evaluated to `false`. A real, pre-existing gap,
        // fixed here rather than left alongside the new temporal types.
        (PropertyValue::Date(a), PropertyValue::Date(b)) => Some(cmp_ord(op, *a, *b)),
        (PropertyValue::LocalTime(a), PropertyValue::LocalTime(b)) => Some(cmp_ord(op, *a, *b)),
        // Compares the UTC-equivalent instant-of-day, not the raw
        // wall-clock fields -- see `PropertyValue::Time`'s doc comment.
        (
            PropertyValue::Time {
                nanos_of_day: na,
                offset_seconds: oa,
            },
            PropertyValue::Time {
                nanos_of_day: nb,
                offset_seconds: ob,
            },
        ) => Some(cmp_ord(
            op,
            na - *oa as i64 * 1_000_000_000,
            nb - *ob as i64 * 1_000_000_000,
        )),
        (
            PropertyValue::LocalDateTime {
                epoch_seconds: sa,
                nanos: na,
            },
            PropertyValue::LocalDateTime {
                epoch_seconds: sb,
                nanos: nb,
            },
        ) => Some(cmp_ord(op, (*sa, *na), (*sb, *nb))),
        // Instant-only, `offset_seconds` ignored -- see
        // `PropertyValue::DateTime`'s doc comment.
        (
            PropertyValue::DateTime {
                epoch_seconds: sa,
                nanos: na,
                ..
            },
            PropertyValue::DateTime {
                epoch_seconds: sb,
                nanos: nb,
                ..
            },
        ) => Some(cmp_ord(op, (*sa, *na), (*sb, *nb))),
        // `Duration` has no defined *ordering* (see its own doc comment)
        // but `=`/`<>` are still real, component-wise comparisons (the
        // same bug `Date` had above -- the generic mismatch fallback's
        // unconditional `Eq -> false` would otherwise make two
        // genuinely-equal durations compare unequal).
        (PropertyValue::Duration { .. }, PropertyValue::Duration { .. }) => match op {
            CompareOp::Eq => Some(a == b),
            CompareOp::Ne => Some(a != b),
            _ => None,
        },
        _ => match op {
            CompareOp::Eq => Some(false),
            CompareOp::Ne => Some(true),
            // A string predicate on a non-null, non-string operand has no
            // defined answer (undefined, not "definitely false") -- same
            // "type mismatch -> null" stance as ordering, confirmed via a
            // real TCK scenario (`'abc' STARTS WITH true` must be `null`,
            // not `false`, so `(x STARTS WITH true) <> (x STARTS WITH
            // true)` correctly stays `null` rather than folding to a
            // spurious `false`/`true`).
            CompareOp::StartsWith
            | CompareOp::EndsWith
            | CompareOp::Contains
            | CompareOp::Lt
            | CompareOp::Le
            | CompareOp::Gt
            | CompareOp::Ge => None,
        },
    }
}

/// A `ReturnExpr` boolean operand -- `Null` is "unknown" (`None`), a real
/// bool passes through, anything else is a genuine type error (real
/// Cypher: `1 AND true` doesn't silently coerce).
pub(crate) fn value_to_bool3(v: &Value) -> Result<Option<bool>, QueryError> {
    match v {
        Value::Null => Ok(None),
        Value::Literal(Literal::Bool(b)) | Value::Property(PropertyValue::Bool(b)) => Ok(Some(*b)),
        other => Err(QueryError::Type(format!(
            "expected a boolean, got {other:?}"
        ))),
    }
}

pub(crate) fn bool3_to_value(b: Option<bool>) -> Value {
    match b {
        Some(b) => Value::Literal(Literal::Bool(b)),
        None => Value::Null,
    }
}

/// `None`/`None` (both unknown) combines to unknown, matching Cypher's
/// `AND` truth table -- `false` wins over `unknown` (`false AND unknown =
/// false`), but `true AND unknown = unknown`, not `true`.
pub(crate) fn and3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

/// Mirrors `and3` for `OR` -- `true` wins over `unknown`.
pub(crate) fn or3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

/// `XOR` has no "one side already decides it" shortcut the way `AND`/`OR`
/// do -- either operand being unknown makes the whole result unknown,
/// since flipping the unknown side could flip the answer either way.
pub(crate) fn xor3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a != b),
        _ => None,
    }
}

pub(crate) fn cmp_f64(op: CompareOp, a: f64, b: f64) -> bool {
    match op {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
        // Only meaningful for String/String, handled separately in
        // `compare()` before reaching here -- a numeric operand with one
        // of these ops is a type mismatch, same as any other.
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => false,
    }
}

pub(crate) fn cmp_ord<T: PartialOrd>(op: CompareOp, a: T, b: T) -> bool {
    match op {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => false,
    }
}

/// Value equality for CASE's WHEN-comparison (and, elsewhere, DISTINCT
/// dedup within an aggregate). Null == Null -> true here deliberately,
/// unlike `compare()`'s three-valued `WHERE`-filter semantics -- CASE and
/// DISTINCT need a definite yes/no ("is this the same value as a value
/// already collected", "does this WHEN branch match") rather than
/// "unknown", so plain equality is the correct, separate choice here, not
/// an oversight. `Node`/`Edge` compare by id (graph identity), not
/// full-struct contents — cheaper, and the correct semantics regardless
/// (two bindings are "the same node" iff the same node, not iff their
/// label/prop snapshots happen to match).
pub(crate) fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Null, _) | (_, Value::Null) => false,
        (Value::Property(pa), Value::Property(pb)) => property_value_eq(pa, pb),
        (Value::Literal(la), Value::Literal(lb)) => la == lb,
        (Value::Property(pa), Value::Literal(lb)) => *pa == literal_to_value(lb),
        (Value::Literal(la), Value::Property(pb)) => literal_to_value(la) == *pb,
        (Value::Node(na), Value::Node(nb)) => na.id == nb.id,
        (Value::Edge(ea), Value::Edge(eb)) => ea.id == eb.id,
        (Value::List(la), Value::List(lb)) => {
            la.len() == lb.len() && la.iter().zip(lb).all(|(x, y)| value_eq(x, y))
        }
        // Two paths are equal iff they visit the same nodes/relationships
        // in the same order (real Cypher's own path-equality rule) --
        // element-wise identity, same `.id` comparison `Value::Node`/
        // `Value::Edge` above already use. Previously fell through to the
        // catch-all `_ => false` (any two paths were unconditionally
        // unequal, even two bindings of the identical path) -- unreachable
        // until two independently-MATCHed paths could be compared via `=`
        // in one statement (TCK's Comparison1 [14]).
        (Value::Path(pa), Value::Path(pb)) => {
            pa.len() == pb.len()
                && pa.iter().zip(pb).all(|(x, y)| match (x, y) {
                    (PathElem::Node(na), PathElem::Node(nb)) => na.id == nb.id,
                    (PathElem::Edge(ea), PathElem::Edge(eb)) => ea.id == eb.id,
                    _ => false,
                })
        }
        _ => false,
    }
}

/// `PropertyValue`'s derived `PartialEq` is structural (every field must
/// match), which is wrong for `Time`/`DateTime`: two values at the same
/// instant but different offsets must compare equal (see their own doc
/// comments -- same rule `compare_property_pair`/`compare_non_null`/
/// `comparable_ordering` already apply for `<`/`>`/ORDER BY/min/max).
/// Everything else keeps plain structural equality.
pub(crate) fn property_value_eq(a: &PropertyValue, b: &PropertyValue) -> bool {
    match (a, b) {
        (
            PropertyValue::Time {
                nanos_of_day: na,
                offset_seconds: oa,
            },
            PropertyValue::Time {
                nanos_of_day: nb,
                offset_seconds: ob,
            },
        ) => na - *oa as i64 * 1_000_000_000 == nb - *ob as i64 * 1_000_000_000,
        (
            PropertyValue::DateTime {
                epoch_seconds: sa,
                nanos: na,
                ..
            },
            PropertyValue::DateTime {
                epoch_seconds: sb,
                nanos: nb,
                ..
            },
        ) => sa == sb && na == nb,
        _ => a == b,
    }
}

/// Sorts `rows` (already-projected `RETURN`/`WITH` output, `columns`
/// aligned by index) by `order_by`, which evaluates against the projected
/// column names — never the raw pattern `BindingRow` — since every ORDER BY
/// key in practice is a RETURN/WITH alias, not a bare pattern variable.
pub(crate) fn apply_order_by(
    rows: Vec<Vec<Value>>,
    columns: &[String],
    order_by: &[(ReturnExpr, SortDir)],
    items: Option<&[ReturnItem]>,
    skip: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Vec<Value>>, QueryError> {
    // An ORDER BY expression that repeats a returned expression verbatim
    // (`RETURN n.name, count(*) AS foo ORDER BY n.name`) names a real
    // output column by its default name -- match it directly by position
    // rather than re-evaluating the expression, which would need bindings
    // (e.g. `n`) that only the pre-aggregation rows had and are gone by
    // this post-projection point. That name-based match only works for an
    // *unaliased* item (its column name literally is its default name) --
    // an aliased item repeated verbatim (`RETURN sum(x) AS s ORDER BY
    // sum(x)`, TCK's WithOrderBy4 [11]) needs a structural match against
    // the item's own expression instead, falling back to position in
    // `items` (1:1 with `columns`, one column per return item).
    let order_by_col: Vec<Option<usize>> = order_by
        .iter()
        .map(|(expr, _)| {
            columns
                .iter()
                .position(|c| *c == default_column_name(expr, 0))
                .or_else(|| {
                    items.and_then(|items| items.iter().position(|item| item.expr == *expr))
                })
        })
        .collect();
    let mut keyed: Vec<(Vec<Value>, Vec<Value>)> = Vec::with_capacity(rows.len());
    for row in rows {
        let row_map: HashMap<String, Value> =
            columns.iter().cloned().zip(row.iter().cloned()).collect();
        let keys = order_by
            .iter()
            .zip(&order_by_col)
            .map(|((expr, _), col)| match col {
                Some(i) => Ok(row[*i].clone()),
                None => eval_projected_expr(expr, &row_map),
            })
            .collect::<Result<Vec<_>, _>>()?;
        keyed.push((keys, row));
    }
    Ok(top_k_by(keyed, order_by, skip, limit)
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Same expression shape as `eval_return_expr`, but resolves `Var`/`Prop`
/// against already-projected output columns instead of the graph-bound
/// `BindingRow` — no `WriteTransaction`/`GraphStore` access needed, since a
/// projected `Value::Node`/`Value::Edge` already carries its full record
/// (including props) from when it was first materialized.
pub(crate) fn eval_projected_expr(
    expr: &ReturnExpr,
    row: &HashMap<String, Value>,
) -> Result<Value, QueryError> {
    match expr {
        ReturnExpr::Var(name) => row
            .get(name)
            .cloned()
            .ok_or_else(|| QueryError::UnboundVariable(name.clone())),
        ReturnExpr::Prop(pa) => {
            let base = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            match base {
                Value::Map(m) => Ok(m.get(&pa.prop).cloned().unwrap_or(Value::Null)),
                Value::Node(n) => Ok(match n.props.get(&pa.prop).cloned() {
                    Some(PropertyValue::Null) | None => Value::Null,
                    Some(v) => property_value_to_value(v),
                }),
                Value::Edge(e) => Ok(match e.props.get(&pa.prop).cloned() {
                    Some(PropertyValue::Null) | None => Value::Null,
                    Some(v) => property_value_to_value(v),
                }),
                // `d.year`/`d.months`/etc component access on a `Date`/
                // `Duration` in projected/ORDER BY position -- mirrors
                // `lookup_prop_value`'s equivalent `Binding::Value(pv)`
                // handling for the pre-projection path.
                Value::Property(pv) => Ok(match temporal_component(pv, &pa.prop) {
                    Some(component) => Value::Property(component),
                    None => Value::Null,
                }),
                _ => Ok(Value::Null),
            }
        }
        ReturnExpr::PropOf(base, prop) => {
            let v = eval_projected_expr(base, row)?;
            property_of_value(&v, prop)
        }
        ReturnExpr::Lit(lit) => Ok(match lit {
            Literal::Null => Value::Null,
            other => Value::Literal(other.clone()),
        }),
        ReturnExpr::Call { name, args, .. } => {
            // Same internal-consistency stance as `eval_return_expr`'s
            // `Call` arm: by the time ORDER BY runs, aggregation has
            // already resolved into ordinary named output columns
            // (referenced here via `Var`), so a raw aggregate `Call`
            // reaching this point means it wasn't top-level as
            // `validate_return_items` requires.
            if is_aggregate_name(name) {
                return Err(QueryError::Semantic(format!(
                    "aggregate function '{name}' can only be used as a return item's top-level expression"
                )));
            }
            let arg_values = args
                .iter()
                .map(|a| eval_projected_expr(a, row))
                .collect::<Result<Vec<_>, _>>()?;
            // No `Executor` (and so no cached `now_snapshot()`) reachable
            // from this post-projection/ORDER BY path -- a fresh capture
            // here is a real, narrow inconsistency (a no-arg `date()`/
            // etc re-evaluated from *inside* an ORDER BY expression could
            // in principle read a different instant than the same call
            // during `RETURN`'s own projection), but reaching this
            // specific shape at all is a rare, arguably degenerate query.
            call_builtin(name, &arg_values, temporal::capture_now())
        }
        ReturnExpr::CountStar => Err(QueryError::Semantic(
            "count(*) can only be used as a return item's top-level expression".into(),
        )),
        ReturnExpr::Case { test, whens, else_ } => {
            let test_value = match test {
                Some(t) => Some(eval_projected_expr(t, row)?),
                None => None,
            };
            for (when, then) in whens {
                let when_value = eval_projected_expr(when, row)?;
                let matched = match &test_value {
                    Some(tv) => value_eq(tv, &when_value),
                    None => matches!(when_value, Value::Literal(Literal::Bool(true))),
                };
                if matched {
                    return eval_projected_expr(then, row);
                }
            }
            match else_ {
                Some(e) => eval_projected_expr(e, row),
                None => Ok(Value::Null),
            }
        }
        ReturnExpr::Arith(l, op, r) => {
            let lv = eval_projected_expr(l, row)?;
            let rv = eval_projected_expr(r, row)?;
            apply_arith(*op, &lv, &rv)
        }
        ReturnExpr::Neg(e) => {
            let v = eval_projected_expr(e, row)?;
            apply_neg(&v)
        }
        ReturnExpr::ListLit(items) => Ok(Value::List(
            items
                .iter()
                .map(|item| eval_projected_expr(item, row))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        ReturnExpr::Index(base, index) => {
            let base_v = eval_projected_expr(base, row)?;
            let index_v = eval_projected_expr(index, row)?;
            apply_index(&base_v, &index_v)
        }
        ReturnExpr::Slice(base, start, end) => {
            let base_v = eval_projected_expr(base, row)?;
            let start_v = start
                .as_deref()
                .map(|s| eval_projected_expr(s, row))
                .transpose()?;
            let end_v = end
                .as_deref()
                .map(|e| eval_projected_expr(e, row))
                .transpose()?;
            apply_slice(&base_v, start_v.as_ref(), end_v.as_ref())
        }
        ReturnExpr::ListComp {
            var,
            source,
            where_clause,
            project,
        } => {
            let source_v = eval_projected_expr(source, row)?;
            let items = match source_v {
                Value::List(items) => items,
                Value::Null => return Ok(Value::Null),
                other => {
                    return Err(QueryError::Type(format!(
                        "list comprehension source must be a list, got {other:?}"
                    )))
                }
            };
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                let mut scoped_row = row.clone();
                scoped_row.insert(var.clone(), item.clone());
                let keep = match where_clause {
                    Some(w) => value_to_bool3(&eval_projected_expr(w, &scoped_row)?)? == Some(true),
                    None => true,
                };
                if !keep {
                    continue;
                }
                result.push(match project {
                    Some(p) => eval_projected_expr(p, &scoped_row)?,
                    None => item,
                });
            }
            Ok(Value::List(result))
        }
        ReturnExpr::Quantifier {
            kind,
            var,
            source,
            where_clause,
        } => {
            let source_v = eval_projected_expr(source, row)?;
            let items = match source_v {
                Value::List(items) => items,
                Value::Null => return Ok(Value::Null),
                other => {
                    return Err(QueryError::Type(format!(
                        "quantifier source must be a list, got {other:?}"
                    )))
                }
            };
            let mut preds = Vec::with_capacity(items.len());
            for item in &items {
                let mut scoped_row = row.clone();
                scoped_row.insert(var.clone(), item.clone());
                preds.push(match where_clause {
                    Some(w) => value_to_bool3(&eval_projected_expr(w, &scoped_row)?)?,
                    None => item_truthy(item),
                });
            }
            Ok(match eval_quantifier(*kind, &preds) {
                Some(b) => Value::Literal(Literal::Bool(b)),
                None => Value::Null,
            })
        }
        ReturnExpr::MapLit(entries) => {
            let mut map = BTreeMap::new();
            for (k, v) in entries {
                map.insert(k.clone(), eval_projected_expr(v, row)?);
            }
            Ok(Value::Map(map))
        }
        ReturnExpr::And(l, r) => Ok(bool3_to_value(and3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Or(l, r) => Ok(bool3_to_value(or3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Xor(l, r) => Ok(bool3_to_value(xor3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Not(e) => Ok(bool3_to_value(
            value_to_bool3(&eval_projected_expr(e, row)?)?.map(|b| !b),
        )),
        ReturnExpr::Compare(l, op, r) => {
            let lv = eval_projected_expr(l, row)?;
            let rv = eval_projected_expr(r, row)?;
            Ok(bool3_to_value(compare_values(&lv, *op, &rv)))
        }
        ReturnExpr::IsNull(e) => Ok(Value::Literal(Literal::Bool(matches!(
            eval_projected_expr(e, row)?,
            Value::Null
        )))),
        ReturnExpr::In(needle, haystack) => {
            let nv = eval_projected_expr(needle, row)?;
            let hv = eval_projected_expr(haystack, row)?;
            Ok(bool3_to_value(list_membership_ternary(&nv, &hv)?))
        }
        ReturnExpr::HasLabel(var, labels) => {
            let binding = row
                .get(var)
                .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
            match binding {
                Value::Node(n) => Ok(Value::Literal(Literal::Bool(
                    labels.iter().all(|l| n.labels.contains(l)),
                ))),
                Value::Null => Ok(Value::Null),
                other => Err(QueryError::Type(format!(
                    "'{var}' isn't a node — (n:Label) needs a node binding, got {other:?}"
                ))),
            }
        }
        ReturnExpr::PatternPredicate(_) => Err(QueryError::Semantic(
            "a pattern predicate (`(n)-->()` etc) can only be used inside WHERE".into(),
        )),
        // No `Txn`/`ExecutionGuard` reachable from this post-projection
        // path (same "no `Executor`" limitation as the `Call` arm above)
        // -- a pattern comprehension needs a real graph traversal to
        // re-evaluate, which this function structurally can't do. Only
        // reachable for an ORDER BY key that references a pattern
        // comprehension *without* repeating a RETURN/WITH item verbatim
        // (the verbatim case matches by column position before ever
        // reaching here -- see `apply_order_by`'s `order_by_col`) --
        // not exercised by any current TCK scenario.
        ReturnExpr::PatternComprehension { .. } => Err(QueryError::Semantic(
            "a pattern comprehension can only be used in RETURN/WITH position, or as an ORDER BY \
             key that repeats one of their items verbatim"
                .into(),
        )),
        ReturnExpr::ExistsPattern { .. } | ReturnExpr::ExistsSubquery(_) => Err(
            QueryError::Semantic("an exists {} subquery can only be used inside WHERE".into()),
        ),
    }
}

/// `RETURN DISTINCT`'s result-set-level dedup -- structural equality of
/// the whole row (same `HashKey` machinery `DISTINCT` inside an aggregate
/// call and `resolve_grouped_rows`' grouping already use, not `value_eq`'s
/// definite-equality-only comparison, since a `HashSet` needs `Hash` too).
/// Keeps the first occurrence of each distinct row, preserving order --
/// what every other DB's `DISTINCT` does, and what a human reading the
/// query would expect.
pub(crate) fn dedup_rows(rows: Vec<Vec<Value>>) -> Result<Vec<Vec<Value>>, QueryError> {
    let mut seen: HashSet<Vec<HashKey>> = HashSet::with_capacity(rows.len());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key = row
            .iter()
            .map(value_hash_key)
            .collect::<Result<Vec<_>, _>>()?;
        if seen.insert(key) {
            out.push(row);
        }
    }
    Ok(out)
}

/// `WITH DISTINCT`'s result-set-level dedup -- same first-occurrence-wins
/// structural equality as `dedup_rows` (`RETURN DISTINCT`), but keyed at
/// the `Binding` level via `binding_hash_key` (node/edge identity, not
/// re-fetched contents) since a `WITH`-projected row can still carry a
/// real `Binding::Node`/`Edge` a later clause keeps traversing from,
/// unlike `RETURN`'s already-fully-evaluated `Value` rows.
pub(crate) fn dedup_binding_rows(
    items: &[ReturnItem],
    rows: Vec<BindingRow>,
) -> Result<Vec<BindingRow>, QueryError> {
    let names: Vec<String> = items
        .iter()
        .enumerate()
        .map(with_item_output_name)
        .collect();
    let mut seen: HashSet<Vec<HashKey>> = HashSet::with_capacity(rows.len());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key = names
            .iter()
            .map(|name| {
                binding_hash_key(row.get(name).unwrap_or_else(|| {
                    panic!("DISTINCT row missing its own projected column '{name}'")
                }))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if seen.insert(key) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Sorts `keyed` (each entry paired with its precomputed per-column sort
/// keys) by `order_by`'s directions, keeping only the first `limit` items
/// when one is given and smaller than the row count. When it is, uses
/// `select_nth_unstable_by` to partition around the k-th smallest element
/// (O(n) average) and sorts only that k-sized prefix (O(k log k)), instead
/// of a full O(n log n) sort of every row just to immediately discard all
/// but the first few -- the "ORDER BY + LIMIT -> TOP-K" rewrite real query
/// engines apply. Shared by all three ORDER BY sites (`WITH`'s own,
/// non-aggregating `RETURN`'s, and aggregating `RETURN`'s), which otherwise
/// each build the identical `keyed`-then-sort shape around a different row
/// type.
/// Selects the top `skip + limit` elements by `order_by` (the
/// `select_nth_unstable_by` partial-selection optimization still applies
/// to that combined bound, not just `limit` alone), sorts just that
/// prefix, then drops the first `skip` of it — real Cypher's own
/// "SKIP applies after ORDER BY, LIMIT applies after SKIP" rule.
pub(crate) fn top_k_by<T>(
    mut keyed: Vec<(Vec<Value>, T)>,
    order_by: &[(ReturnExpr, SortDir)],
    skip: Option<i64>,
    limit: Option<i64>,
) -> Vec<(Vec<Value>, T)> {
    let cmp = |a: &(Vec<Value>, T), b: &(Vec<Value>, T)| -> std::cmp::Ordering {
        for (i, (_, dir)) in order_by.iter().enumerate() {
            let ord = compare_with_dir(&a.0[i], &b.0[i], *dir);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    };
    let skip_n = skip.unwrap_or(0).max(0) as usize;
    match limit {
        Some(n) => {
            let k = skip_n + n.max(0) as usize;
            if k == 0 {
                keyed.clear();
            } else if k < keyed.len() {
                keyed.select_nth_unstable_by(k - 1, cmp);
                keyed.truncate(k);
                keyed.sort_by(cmp);
            } else {
                keyed.sort_by(cmp);
            }
        }
        None => keyed.sort_by(cmp),
    }
    if skip_n > 0 {
        keyed.drain(0..skip_n.min(keyed.len()));
    }
    keyed
}

/// `Null` is just the highest-ranked type in `type_rank`'s total order
/// (see its docs), not a special case here -- confirmed via TCK's
/// `ReturnOrderBy1 [12]`/`WithOrderBy1 [22]` ("sort distinct types...
/// descending"), which expect `null` to sort *first* under `DESC`, not
/// last. An earlier version of this function hardcoded nulls-last
/// regardless of direction (citing Neo4j's docs); that's wrong per the
/// TCK's own evidence -- `DESC` is a real reversal of the whole order,
/// `null` included, not just of the non-null comparisons.
pub(crate) fn compare_with_dir(a: &Value, b: &Value, dir: SortDir) -> std::cmp::Ordering {
    let ord = compare_non_null(a, b);
    if dir == SortDir::Desc {
        ord.reverse()
    } else {
        ord
    }
}

/// Real Cypher regards `NaN` as larger than every other number (confirmed
/// via TCK's `ReturnOrderBy1 [11]`/`[12]`: `NaN` sorts directly below
/// `null`, above every finite float, both ASC and DESC) -- plain
/// `f64::partial_cmp` returns `None` for any comparison involving `NaN`,
/// which `.unwrap_or(Ordering::Equal)` used to paper over by treating
/// `NaN` as *equal* to every number. That's not just cosmetically wrong:
/// a stable sort over a comparator that calls two genuinely-different
/// values "equal" preserves their original relative order instead of
/// actually ordering them, and `DESC`'s blanket `.reverse()` of an
/// "equal" result is still "equal" -- so `1.5`/`NaN` kept the same
/// relative order under both ASC and DESC, when DESC should have
/// swapped them.
pub(crate) fn cmp_f64_nan_greatest(x: f64, y: f64) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (x.is_nan(), y.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
    }
}

pub(crate) fn compare_non_null(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    // Real Cypher orders two lists lexicographically (element-by-element,
    // shorter-is-less on a common prefix), a genuinely different rule
    // from any single scalar comparison -- delegate to its own recursive
    // comparator before reaching the scalar-only match below, which would
    // otherwise silently treat every pair of lists as "equal" (found via
    // TCK's ReturnOrderBy1 `[10]`/WithOrderBy1 `[10]`: `ORDER BY <list
    // column>` produced no reordering at all, ASC and DESC alike -- a
    // stable sort over an always-`Equal` comparator is a no-op).
    if let (Value::List(_), Value::List(_)) = (a, b) {
        return list_cmp_asc(a, b);
    }
    let pa = value_to_comparable(a);
    let pb = value_to_comparable(b);
    match (pa, pb) {
        (Some(PropertyValue::Int(x)), Some(PropertyValue::Int(y))) => x.cmp(&y),
        (Some(PropertyValue::Int(x)), Some(PropertyValue::Float(y))) => {
            cmp_f64_nan_greatest(x as f64, y)
        }
        (Some(PropertyValue::Float(x)), Some(PropertyValue::Int(y))) => {
            cmp_f64_nan_greatest(x, y as f64)
        }
        (Some(PropertyValue::Float(x)), Some(PropertyValue::Float(y))) => {
            cmp_f64_nan_greatest(x, y)
        }
        (Some(PropertyValue::String(x)), Some(PropertyValue::String(y))) => x.cmp(&y),
        (Some(PropertyValue::Bool(x)), Some(PropertyValue::Bool(y))) => x.cmp(&y),
        (Some(PropertyValue::Date(x)), Some(PropertyValue::Date(y))) => x.cmp(&y),
        (Some(PropertyValue::LocalTime(x)), Some(PropertyValue::LocalTime(y))) => x.cmp(&y),
        (
            Some(PropertyValue::Time {
                nanos_of_day: x,
                offset_seconds: ox,
            }),
            Some(PropertyValue::Time {
                nanos_of_day: y,
                offset_seconds: oy,
            }),
        ) => (x - ox as i64 * 1_000_000_000).cmp(&(y - oy as i64 * 1_000_000_000)),
        (
            Some(PropertyValue::LocalDateTime {
                epoch_seconds: xs,
                nanos: xn,
            }),
            Some(PropertyValue::LocalDateTime {
                epoch_seconds: ys,
                nanos: yn,
            }),
        ) => (xs, xn).cmp(&(ys, yn)),
        (
            Some(PropertyValue::DateTime {
                epoch_seconds: xs,
                nanos: xn,
                ..
            }),
            Some(PropertyValue::DateTime {
                epoch_seconds: ys,
                nanos: yn,
                ..
            }),
        ) => (xs, xn).cmp(&(ys, yn)),
        // Cross-type scalars (e.g. a String vs a Number) fall through to
        // `type_rank`'s real Cypher orderability rank rather than this
        // arm's own `Equal` fallback -- see `list_cmp_asc`, the only
        // caller that can actually produce a cross-type pair here (a
        // top-level ORDER BY key is already one uniform column in
        // practice, but a list's *elements* legitimately mix types, e.g.
        // `['a', 1]`).
        _ => match (type_rank(a), type_rank(b)) {
            (Some(ra), Some(rb)) if ra != rb => ra.cmp(&rb),
            _ => Ordering::Equal,
        },
    }
}

/// Real Cypher's cross-type "orderability" rank (distinct from
/// `WHERE`'s three-valued comparison semantics) -- only covers the types
/// that can actually reach here with no same-type match already handling
/// them (see `compare_non_null`'s cross-type fallback and `list_cmp_asc`).
/// Order confirmed against a real TCK scenario (`ReturnOrderBy1`/
/// `WithOrderBy1`'s "sort distinct types" scenarios, only reachable once
/// `marsdb-tck`'s own harness could parse a path-shaped expected cell --
/// previously these scenarios could never even run): `Map < Node <
/// Relationship < List < Path < String < Boolean < Number`, `Null` always
/// last regardless (`compare_with_dir`'s own separate check). This is
/// also a fix, not just an addition -- `Bool`/`String` were previously
/// ranked in the wrong relative order (`Bool` before `String`; real
/// Cypher has `String` before `Bool`), and `List` sorting before every
/// scalar (confirmed separately, `max()`/`min()` over `[1, 'a', null,
/// [1, 2], 0.2, 'b']` picks `1` for max and `[1, 2]` for min) still
/// holds with `Map`/`Node`/`Relationship` now ranking below it too.
/// Temporal types (`Date`.../`Duration`) have no TCK evidence placing
/// them anywhere in this cross-type order -- kept after `Number` in
/// their pre-existing relative order among themselves, arbitrarily but
/// harmlessly (nothing tests a temporal-vs-Map-shaped ORDER BY column).
/// `Null` ranks highest of all -- also TCK-confirmed
/// (`ReturnOrderBy1 [11]`'s own expected order ends with `null` last),
/// and, critically, ranking it here rather than special-casing it in
/// `compare_with_dir` is what makes `DESC` correctly put `null` *first*
/// (`ReturnOrderBy1 [12]`/`WithOrderBy1 [22]`) -- a hardcoded
/// "nulls always last" rule would get the ascending case right and the
/// descending case wrong, since real Cypher's `DESC` is a genuine
/// reversal of the total order, not just of the non-null comparisons.
pub(crate) fn type_rank(v: &Value) -> Option<u8> {
    match v {
        Value::Map(_) => Some(0),
        Value::Node(_) => Some(1),
        Value::Edge(_) => Some(2),
        Value::List(_) => Some(3),
        Value::Path(_) => Some(4),
        Value::Literal(Literal::String(_)) | Value::Property(PropertyValue::String(_)) => Some(5),
        Value::Literal(Literal::Bool(_)) | Value::Property(PropertyValue::Bool(_)) => Some(6),
        Value::Literal(Literal::Int(_))
        | Value::Property(PropertyValue::Int(_))
        | Value::Literal(Literal::Float(_))
        | Value::Property(PropertyValue::Float(_)) => Some(7),
        Value::Property(PropertyValue::Date(_)) => Some(8),
        Value::Property(PropertyValue::LocalTime(_)) => Some(9),
        Value::Property(PropertyValue::Time { .. }) => Some(10),
        Value::Property(PropertyValue::LocalDateTime { .. }) => Some(11),
        Value::Property(PropertyValue::DateTime { .. }) => Some(12),
        Value::Null | Value::Literal(Literal::Null) | Value::Property(PropertyValue::Null) => {
            Some(13)
        }
        _ => None,
    }
}

/// Ascending, element-by-element list comparison for ORDER BY, mirroring
/// `compare_with_dir`'s "null sorts last" rule recursively at every
/// position (deliberately *not* `value_partial_cmp`'s WHERE-filter
/// three-valued semantics, where a null anywhere makes the whole
/// comparison undecided instead of a definite presentation order) — a
/// shorter list that's a prefix of a longer one sorts first, same
/// convention `value_partial_cmp` already uses. `compare_with_dir`
/// reverses the *overall* result for `DESC`, not each element
/// individually — verified element-by-element against TCK's
/// ReturnOrderBy1 `[10]` ("ORDER BY DESC should order lists in the
/// expected order").
pub(crate) fn list_cmp_asc(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let a_null = matches!(a, Value::Null);
    let b_null = matches!(b, Value::Null);
    match (a_null, b_null) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }
    if let (Value::List(xs), Value::List(ys)) = (a, b) {
        for (x, y) in xs.iter().zip(ys) {
            match list_cmp_asc(x, y) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        return xs.len().cmp(&ys.len());
    }
    compare_non_null(a, b)
}

pub(crate) fn value_to_comparable(v: &Value) -> Option<PropertyValue> {
    match v {
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(lit) => Some(literal_to_value(lit)),
        _ => None,
    }
}

/// Ordering for `min`/`max` aggregate folding — `None` for values with no
/// natural order (`Node`/`Edge`/`Map`/`Path`, or a `Null`, which
/// `AggAcc::fold` never passes here anyway since null contributions are
/// skipped before folding). The caller turns `None` into a clear error
/// rather than an arbitrary "always equal" fallback — unlike ORDER BY's
/// `compare_non_null`, which tolerates that for presentation ordering
/// (see its docs), silently treating two nodes as "equal" inside an
/// aggregate would be a wrong-answer failure mode, not just an
/// unhelpful sort order.
///
/// `List` *is* comparable here (real Cypher's `max()`/`min()` handle a
/// list argument, ordered element-by-element the same way ORDER BY
/// does — reuses `list_cmp_asc`), and so is a genuine cross-type pair
/// (`max()` over `[1, 'a', [1, 2]]`-shaped input), via the same
/// `type_rank` fallback `compare_non_null` uses.
pub(crate) fn comparable_ordering(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    if let (Value::List(_), Value::List(_)) = (a, b) {
        return Some(list_cmp_asc(a, b));
    }
    let (pa, pb) = match (value_to_comparable(a), value_to_comparable(b)) {
        (Some(pa), Some(pb)) => (pa, pb),
        _ => {
            return match (type_rank(a), type_rank(b)) {
                // Different rank -- a real cross-type comparison (e.g. a
                // `List` vs a `String` inside a `max()` fold), safe to
                // order by rank.
                (Some(ra), Some(rb)) if ra != rb => Some(ra.cmp(&rb)),
                // Same rank only ever means both are `Map`/`Node`/`Edge`/
                // `Path` here (every type with a real per-value order
                // already matched via `value_to_comparable`'s `Some` case
                // above, `List` is handled separately at the top) --
                // those have no defined per-value order at all. Real for
                // ORDER BY's own use of `type_rank` (`compare_non_null`,
                // which tolerates "equal" for presentation purposes), but
                // silently treating two different `Map`s (or `Node`s,
                // ...) as "equal" here would be a wrong-answer failure
                // mode for an aggregate, not just an unhelpful sort
                // order -- `None` instead (see this function's own docs).
                _ => None,
            };
        }
    };
    Some(match (pa, pb) {
        (PropertyValue::Int(x), PropertyValue::Int(y)) => x.cmp(&y),
        (PropertyValue::Int(x), PropertyValue::Float(y)) => cmp_f64_nan_greatest(x as f64, y),
        (PropertyValue::Float(x), PropertyValue::Int(y)) => cmp_f64_nan_greatest(x, y as f64),
        (PropertyValue::Float(x), PropertyValue::Float(y)) => cmp_f64_nan_greatest(x, y),
        (PropertyValue::String(x), PropertyValue::String(y)) => x.cmp(&y),
        (PropertyValue::Bool(x), PropertyValue::Bool(y)) => x.cmp(&y),
        // `Duration` deliberately has no arm here (falls through to
        // `None` below) -- no defined ordering, only equality (see
        // `compare_values`'s docs on why months/days/seconds aren't
        // fungible enough to order against each other).
        (PropertyValue::Date(x), PropertyValue::Date(y)) => x.cmp(&y),
        (PropertyValue::LocalTime(x), PropertyValue::LocalTime(y)) => x.cmp(&y),
        (
            PropertyValue::Time {
                nanos_of_day: x,
                offset_seconds: ox,
            },
            PropertyValue::Time {
                nanos_of_day: y,
                offset_seconds: oy,
            },
        ) => (x - ox as i64 * 1_000_000_000).cmp(&(y - oy as i64 * 1_000_000_000)),
        (
            PropertyValue::LocalDateTime {
                epoch_seconds: xs,
                nanos: xn,
            },
            PropertyValue::LocalDateTime {
                epoch_seconds: ys,
                nanos: yn,
            },
        ) => (xs, xn).cmp(&(ys, yn)),
        (
            PropertyValue::DateTime {
                epoch_seconds: xs,
                nanos: xn,
                ..
            },
            PropertyValue::DateTime {
                epoch_seconds: ys,
                nanos: yn,
                ..
            },
        ) => (xs, xn).cmp(&(ys, yn)),
        _ => return None,
    })
}

/// General `lhs op rhs` for `ReturnExpr::Compare` -- unlike `compare()`
/// (a `PropertyValue`-vs-`Literal` comparison for pattern-level `WHERE`,
/// where the RHS is always a literal), both sides here are already-
/// evaluated `Value`s, since either can be a *computed* result (e.g. two
/// `date(...)` calls) with no `Literal` able to stand in for it.
/// Three-valued like `compare()`: `None` (Cypher's "unknown") for a null
/// operand, an operator with no meaning for the operands' types (e.g. `<`
/// between two `Duration`s), or a type mismatch.
pub(crate) fn compare_values(a: &Value, op: CompareOp, b: &Value) -> Option<bool> {
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return None;
    }
    match op {
        CompareOp::Eq => value_equal_ternary(a, b),
        CompareOp::Ne => value_equal_ternary(a, b).map(|eq| !eq),
        CompareOp::Lt => ordered_compare(a, b, |o| o == std::cmp::Ordering::Less),
        CompareOp::Le => ordered_compare(a, b, |o| o != std::cmp::Ordering::Greater),
        CompareOp::Gt => ordered_compare(a, b, |o| o == std::cmp::Ordering::Greater),
        CompareOp::Ge => ordered_compare(a, b, |o| o != std::cmp::Ordering::Less),
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => {
            let (Some(s), Some(p)) = (as_arith_str(a), as_arith_str(b)) else {
                return None;
            };
            Some(match op {
                CompareOp::StartsWith => s.starts_with(p),
                CompareOp::EndsWith => s.ends_with(p),
                CompareOp::Contains => s.contains(p),
                _ => unreachable!("only StartsWith/EndsWith/Contains reach this arm"),
            })
        }
    }
}

/// `<`/`<=`/`>`/`>=` -- numeric operands are special-cased (not folded
/// into `value_partial_cmp` below) specifically so `NaN` compares as a
/// definite `false` on every operator, matching real Cypher (`0.0/0.0 >
/// 1` is `false`, not `null`) -- verified against Comparison2's
/// "Comparing NaN" scenario, which is what exposed `comparable_ordering`'s
/// `unwrap_or(Equal)` silently making `NaN >= x`/`NaN <= x` both `true`.
/// Every other type (`List`, `Date`, `String`, `Bool`, ...) has no NaN-like
/// "exists but is unorderable" value, so `None` there really does mean
/// Cypher's ordinary "unknown" (a null operand, a null found while
/// lexicographically comparing two lists, or a genuine type mismatch),
/// not something to special-case to `false`.
pub(crate) fn ordered_compare(
    a: &Value,
    b: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Option<bool> {
    if let (Some(x), Some(y)) = (value_as_f64(a), value_as_f64(b)) {
        return Some(x.partial_cmp(&y).map(pred).unwrap_or(false));
    }
    value_partial_cmp(a, b).map(pred)
}

/// `<`/`<=`/`>`/`>=` between two `List`s -- real Cypher orders lists
/// lexicographically: the first position where the two lists differ
/// decides the result; if every position up to the shorter list's length
/// is equal, the shorter list is "less". A `null` found at a
/// not-yet-decided position makes the *whole* comparison unknown (`None`)
/// -- lexicographic order can't skip past an undecided position to look
/// for a later one that happens to differ, since whether that later
/// position is even reached depends on what the undecided one turns out
/// to be. Verified element-by-element against every row of Comparison2's
/// "Comparing lists" scenario (`[1, 2] >= [1, null]` is `null`, not
/// `false`, even though `2 >= null` alone would also be `null` -- the
/// point is *why*: position 0 is equal, so position 1 is where the
/// answer would come from, and it's undecided). Delegates to
/// `comparable_ordering` for every non-list, non-numeric pair (`Date`,
/// `String`, `Bool`, ...), which has no list case to get wrong.
pub(crate) fn value_partial_cmp(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return None;
    }
    if let (Value::List(xs), Value::List(ys)) = (a, b) {
        for (x, y) in xs.iter().zip(ys) {
            match value_partial_cmp(x, y) {
                Some(Ordering::Equal) => continue,
                other => return other,
            }
        }
        return Some(xs.len().cmp(&ys.len()));
    }
    // Real Cypher's `<`/`<=`/`>`/`>=` (unlike ORDER BY/`min`/`max`, which
    // need a *total* order across every type for presentation purposes --
    // see `comparable_ordering`'s own docs) is only ever defined within a
    // single comparable type. A genuine cross-type pair (a list against a
    // string, a node against a number, ...) must be `null`, not
    // `comparable_ordering`'s type-rank fallback -- that fallback exists
    // purely for `list_cmp_asc`/`min`/`max`'s total-order needs and must
    // not leak into a real WHERE-predicate comparison. Verified against
    // Comparison2's own "Comparing across types yields null, except
    // numbers" scenario (`[] < 1`/`[] < ''`/`[] < true` were all wrongly
    // `true` before this check, since `[]` alone -- not both sides --
    // isn't `Value`-to-`PropertyValue` representable, falling through to
    // the type-rank fallback).
    if value_to_comparable(a).is_none() || value_to_comparable(b).is_none() {
        return None;
    }
    comparable_ordering(a, b)
}

/// `=`/`<>`'s equality -- three-valued (`None` is Cypher's "unknown"),
/// recursing into `List`/`Map` element-by-element so a `null` *inside* a
/// list/map only makes the overall result unknown when it actually
/// matters, not automatically `false`/`true`: a length/key-set mismatch
/// is `false` outright (definite, regardless of any null present --
/// `{k: null} = {}` is `false`, not `null`, since the key sets alone
/// already prove inequality), a definite element mismatch anywhere makes
/// the whole comparison `false` (short-circuits, `false` outranks
/// `unknown` the same way `and3`/`or3` already rank them), and only once
/// every element is confirmed equal or unknown (never definitely
/// unequal) does an unknown element propagate to an unknown overall
/// result. Verified against every row of List3's and Comparison1's
/// list/map equality scenarios. Scalars fall back to numeric-cross-type-
/// aware equality (`1 = 1.0` is `true`, unlike `value_eq`'s plain
/// `PropertyValue` equality, which doesn't promote `Int`/`Float` against
/// each other) or plain `value_eq` for everything else (`Date`,
/// `Duration`'s component equality, `Node`/`Edge` identity, ...).
pub(crate) fn value_equal_ternary(a: &Value, b: &Value) -> Option<bool> {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => None,
        (Value::List(xs), Value::List(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            fold_ternary_eq(xs.iter().zip(ys).map(|(x, y)| value_equal_ternary(x, y)))
        }
        (Value::Map(x), Value::Map(y)) => {
            if !x.keys().eq(y.keys()) {
                return Some(false);
            }
            fold_ternary_eq(x.iter().map(|(k, xv)| value_equal_ternary(xv, &y[k])))
        }
        _ => Some(values_equal_numeric_aware(a, b)),
    }
}

/// `needle IN haystack` -- three-valued like `=`, since it's built from
/// `=` per element: a definite match wins outright even past a later
/// `null` element (short-circuits, matching `and3`/`or3`'s "false/true
/// outranks unknown" convention), no match with at least one `null`
/// element compared along the way is "unknown" (not `false` -- that
/// element *might* have matched), no match and no `null` anywhere is a
/// definite `false`. An empty list is always a definite `false`
/// regardless of `needle`'s own nullness (nothing to compare against, no
/// unknown comparisons ever happened) -- verified against Comparison5's
/// exact empty-list scenarios. `haystack` being `Null` itself (not an
/// empty list) is "unknown", matching `=`'s own null-operand rule;
/// anything else on the right isn't a list at all, a real type error.
pub(crate) fn list_membership_ternary(
    needle: &Value,
    haystack: &Value,
) -> Result<Option<bool>, QueryError> {
    match haystack {
        Value::Null => Ok(None),
        Value::List(items) => {
            let mut saw_unknown = false;
            for item in items {
                match value_equal_ternary(needle, item) {
                    Some(true) => return Ok(Some(true)),
                    Some(false) => {}
                    None => saw_unknown = true,
                }
            }
            Ok(if saw_unknown { None } else { Some(false) })
        }
        other => Err(QueryError::Type(format!(
            "IN requires a list on the right-hand side, got {other:?}"
        ))),
    }
}

/// Combines a sequence of per-element three-valued equality results into
/// one overall result: any definite `Some(false)` wins outright
/// (short-circuits), otherwise `Some(true)` only if every element was a
/// definite `Some(true)`, else `None` (at least one element's equality
/// was itself unknown, and nothing else disproved the match).
pub(crate) fn fold_ternary_eq(mut results: impl Iterator<Item = Option<bool>>) -> Option<bool> {
    let mut saw_unknown = false;
    for r in results.by_ref() {
        match r {
            Some(false) => return Some(false),
            Some(true) => {}
            None => saw_unknown = true,
        }
    }
    if saw_unknown {
        None
    } else {
        Some(true)
    }
}

/// `=`/`<>`'s scalar leaf case: numeric cross-type promotion (`1 = 1.0`
/// is `true` in real Cypher, matching `compare()`'s existing `Int`-vs-
/// `Float` handling) that `value_eq`'s plain `PropertyValue` equality
/// doesn't give (`PropertyValue::Int(1) != PropertyValue::Float(1.0)`,
/// different enum variants) -- falls back to `value_eq` for every non-
/// numeric pair (`Date`, `Duration`'s component equality, `String`,
/// `Bool`, `Node`/`Edge` identity, ...), which is already correct for
/// those.
pub(crate) fn values_equal_numeric_aware(a: &Value, b: &Value) -> bool {
    match (as_arith_num(a), as_arith_num(b)) {
        (Some(ArithNum::Int(x)), Some(ArithNum::Int(y))) => x == y,
        (Some(ArithNum::Int(x)), Some(ArithNum::Float(y)))
        | (Some(ArithNum::Float(y)), Some(ArithNum::Int(x))) => x as f64 == y,
        (Some(ArithNum::Float(x)), Some(ArithNum::Float(y))) => x == y,
        _ => value_eq(a, b),
    }
}
