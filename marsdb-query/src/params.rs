use std::collections::HashMap;

use marsdb_graph::PropertyValue;

use crate::ast::{
    Expr, Literal, MergeClause, NodePattern, Pattern, QueryClause, QueryPart, ReturnExpr,
    ReturnTail, SetItem, Statement, Tail, UnwindClause, WithClause, WithExpr,
};
use crate::error::QueryError;

/// Resolves every `$name` placeholder in `stmt` to a concrete `Literal`
/// using `params`, in place. Called before execution so the executor never
/// sees `Literal::Param` — see the `unreachable!` in
/// `executor::literal_to_value`.
pub fn substitute_params(
    stmt: &mut Statement,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match stmt {
        Statement::Create(patterns) => {
            for pattern in patterns {
                substitute_pattern(pattern, params)?;
            }
        }
        // No `$param`-able position -- label/prop are identifiers, not
        // expressions.
        Statement::CreateIndex { .. } => {}
        Statement::Explain(inner) => substitute_params(inner, params)?,
        Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        } => {
            for clause in clauses {
                substitute_query_clause(clause, params)?;
            }
            if let Some(tail) = tail {
                substitute_tail(tail, params)?;
            }
            if let Some(items) = order_by {
                for (expr, _) in items {
                    substitute_return_expr(expr, params)?;
                }
            }
            if let Some(expr) = skip {
                substitute_return_expr(expr, params)?;
            }
            if let Some(expr) = limit {
                substitute_return_expr(expr, params)?;
            }
        }
        Statement::Union { parts, .. } => {
            for part in parts {
                substitute_params(part, params)?;
            }
        }
    }
    Ok(())
}

fn substitute_query_clause(
    clause: &mut QueryClause,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match clause {
        QueryClause::Match(part) => substitute_query_part(part, params),
        QueryClause::Unwind(u) => substitute_unwind_clause(u, params),
        QueryClause::Merge(m) => substitute_merge_clause(m, params),
        QueryClause::With(with) => substitute_with_clause(with, params),
        QueryClause::Set(items) => {
            for item in items {
                substitute_set_item(item, params)?;
            }
            Ok(())
        }
        QueryClause::Delete { items, detach: _ } => {
            for expr in items {
                substitute_return_expr(expr, params)?;
            }
            Ok(())
        }
        // No `$param`-able position -- `RemoveItem` is a bare prop/label
        // path, not a value expression.
        QueryClause::Remove(_) => Ok(()),
        QueryClause::Create(patterns) => {
            for pattern in patterns {
                substitute_pattern(pattern, params)?;
            }
            Ok(())
        }
    }
}

/// Shared by every `SetItem` list this file substitutes into (`SET`'s own
/// `QueryClause`/`Tail` forms, and `MERGE`'s `ON CREATE`/`ON MATCH SET`)
/// -- `Labels` has no `$param`-able position (a label name is always a
/// bare identifier), `Prop`/`MapAssign` both carry exactly one
/// `ReturnExpr` value to recurse into.
fn substitute_set_item(
    item: &mut SetItem,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match item {
        SetItem::Prop(_, value) | SetItem::MapAssign { value, .. } => {
            substitute_return_expr(value, params)
        }
        SetItem::Labels(..) => Ok(()),
    }
}

fn substitute_merge_clause(
    m: &mut MergeClause,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    substitute_pattern(&mut m.pattern, params)?;
    for item in m.on_create.iter_mut().chain(m.on_match.iter_mut()) {
        substitute_set_item(item, params)?;
    }
    if let Some(with) = &mut m.with {
        substitute_with_clause(with, params)?;
    }
    Ok(())
}

fn substitute_query_part(
    part: &mut QueryPart,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    substitute_pattern(&mut part.pattern, params)?;
    if let Some(expr) = &mut part.where_clause {
        substitute_expr(expr, params)?;
    }
    if let Some(with) = &mut part.with {
        substitute_with_clause(with, params)?;
    }
    Ok(())
}

fn substitute_unwind_clause(
    u: &mut UnwindClause,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    substitute_return_expr(&mut u.source.0, params)?;
    if let Some(expr) = &mut u.where_clause {
        substitute_with_expr(expr, params)?;
    }
    if let Some(with) = &mut u.with {
        substitute_with_clause(with, params)?;
    }
    Ok(())
}

fn substitute_with_clause(
    with: &mut WithClause,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    for item in &mut with.items {
        substitute_return_expr(&mut item.expr, params)?;
    }
    if let Some(where_clause) = &mut with.where_clause {
        substitute_with_expr(where_clause, params)?;
    }
    if let Some(items) = &mut with.order_by {
        for (expr, _) in items {
            substitute_return_expr(expr, params)?;
        }
    }
    if let Some(expr) = &mut with.skip {
        substitute_return_expr(expr, params)?;
    }
    if let Some(expr) = &mut with.limit {
        substitute_return_expr(expr, params)?;
    }
    Ok(())
}

fn substitute_with_expr(
    expr: &mut WithExpr,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match expr {
        WithExpr::And(l, r) | WithExpr::Or(l, r) => {
            substitute_with_expr(l, params)?;
            substitute_with_expr(r, params)?;
        }
        WithExpr::Not(e) => substitute_with_expr(e, params)?,
        WithExpr::Compare(lhs, _, rhs) => {
            substitute_return_expr(lhs, params)?;
            substitute_return_expr(rhs, params)?;
        }
        WithExpr::IsNull(e) => substitute_return_expr(e, params)?,
        WithExpr::Bare(e) => substitute_return_expr(e, params)?,
    }
    Ok(())
}

fn substitute_pattern(
    pattern: &mut Pattern,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    substitute_node(&mut pattern.start, params)?;
    for (rel, node) in &mut pattern.hops {
        for (_, expr) in &mut rel.props {
            substitute_return_expr(expr, params)?;
        }
        substitute_node(node, params)?;
    }
    Ok(())
}

fn substitute_node(
    node: &mut NodePattern,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    for (_, expr) in &mut node.props {
        substitute_return_expr(expr, params)?;
    }
    Ok(())
}

fn substitute_expr(
    expr: &mut Expr,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match expr {
        Expr::And(l, r) | Expr::Or(l, r) => {
            substitute_expr(l, params)?;
            substitute_expr(r, params)?;
        }
        Expr::Not(e) => substitute_expr(e, params)?,
        Expr::Compare(_, _, lit) => substitute_literal(lit, params)?,
        Expr::PropCompare(_, _, _) => {}
        Expr::IsNull(_) => {}
        Expr::HasLabel(_, _) => {}
        Expr::VarEq(_, _) => {}
        Expr::GeneralCompare(lhs, _, rhs) => {
            substitute_return_expr(lhs, params)?;
            substitute_return_expr(rhs, params)?;
        }
        Expr::GeneralIsNull(e) => substitute_return_expr(e, params)?,
        Expr::GeneralBare(e) => substitute_return_expr(e, params)?,
        Expr::Pattern(pattern) => substitute_pattern(pattern, params)?,
        Expr::Exists {
            pattern,
            where_clause,
        } => {
            substitute_pattern(pattern, params)?;
            if let Some(w) = where_clause {
                substitute_expr(w, params)?;
            }
        }
    }
    Ok(())
}

fn substitute_tail(
    tail: &mut Tail,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match tail {
        Tail::Return(items, _) => {
            for item in items {
                substitute_return_expr(&mut item.expr, params)?;
            }
        }
        // No `$param`-able position -- a bare `*`, nothing to substitute.
        Tail::ReturnStar(_) => {}
        Tail::Delete(exprs, ret) | Tail::DetachDelete(exprs, ret) => {
            for expr in exprs {
                substitute_return_expr(expr, params)?;
            }
            substitute_return_tail(ret, params)?;
        }
        Tail::Remove(_, ret) => {
            substitute_return_tail(ret, params)?;
        }
        Tail::Set(items, ret) => {
            for item in items {
                substitute_set_item(item, params)?;
            }
            substitute_return_tail(ret, params)?;
        }
        Tail::Create(patterns, ret) => {
            for pattern in patterns {
                substitute_pattern(pattern, params)?;
            }
            substitute_return_tail(ret, params)?;
        }
    }
    Ok(())
}

/// Substitutes params in a mutating tail's optional trailing `RETURN`
/// (`MATCH (n) SET n.x = $x RETURN n` needs both the `SET`'s own `$x` *and*
/// nothing extra here since this RETURN has none — but `MATCH (n) DELETE n
/// RETURN $y` does).
fn substitute_return_tail(
    ret: &mut Option<ReturnTail>,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    if let Some(rt) = ret {
        for item in &mut rt.items {
            substitute_return_expr(&mut item.expr, params)?;
        }
    }
    Ok(())
}

fn substitute_return_expr(
    expr: &mut ReturnExpr,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    match expr {
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::CountStar => {}
        ReturnExpr::PatternPredicate(pattern) => substitute_pattern(pattern, params)?,
        // A list-valued `$param` can't substitute into a bare `Literal`
        // (no `Literal::List` -- there's no list *literal* syntax in
        // Cypher for one to mean, see `cypher.pest`'s docs) the way a
        // scalar one does, so this replaces the *whole* `ReturnExpr::Lit`
        // node with a `ReturnExpr::ListLit` instead, recursively (a param
        // list can itself contain nested lists) -- everything downstream
        // (indexing, `IN`, iteration, ...) already handles `ListLit` like
        // any other list-valued expression, so nothing else needs to know
        // this value originated from a parameter rather than `[1, 2, 3]`
        // literal syntax.
        ReturnExpr::Lit(Literal::Param(name)) => {
            let value = params
                .get(name)
                .ok_or_else(|| QueryError::MissingParam(name.clone()))?
                .clone();
            *expr = property_value_to_return_expr(name, &value)?;
        }
        ReturnExpr::Lit(_) => {}
        ReturnExpr::Call { args, .. } => {
            for arg in args {
                substitute_return_expr(arg, params)?;
            }
        }
        ReturnExpr::Case { test, whens, else_ } => {
            if let Some(t) = test {
                substitute_return_expr(t, params)?;
            }
            for (when, then) in whens {
                substitute_return_expr(when, params)?;
                substitute_return_expr(then, params)?;
            }
            if let Some(e) = else_ {
                substitute_return_expr(e, params)?;
            }
        }
        ReturnExpr::Arith(l, _, r) => {
            substitute_return_expr(l, params)?;
            substitute_return_expr(r, params)?;
        }
        ReturnExpr::Neg(e) => substitute_return_expr(e, params)?,
        ReturnExpr::ListLit(items) => {
            for item in items {
                substitute_return_expr(item, params)?;
            }
        }
        ReturnExpr::Index(base, index) => {
            substitute_return_expr(base, params)?;
            substitute_return_expr(index, params)?;
        }
        ReturnExpr::PropOf(base, _) => substitute_return_expr(base, params)?,
        ReturnExpr::Slice(base, start, end) => {
            substitute_return_expr(base, params)?;
            if let Some(s) = start {
                substitute_return_expr(s, params)?;
            }
            if let Some(e) = end {
                substitute_return_expr(e, params)?;
            }
        }
        ReturnExpr::ListComp {
            source,
            where_clause,
            project,
            ..
        } => {
            substitute_return_expr(source, params)?;
            if let Some(w) = where_clause {
                substitute_return_expr(w, params)?;
            }
            if let Some(p) = project {
                substitute_return_expr(p, params)?;
            }
        }
        ReturnExpr::Quantifier {
            source,
            where_clause,
            ..
        } => {
            substitute_return_expr(source, params)?;
            if let Some(w) = where_clause {
                substitute_return_expr(w, params)?;
            }
        }
        ReturnExpr::MapLit(entries) => {
            for (_, v) in entries {
                substitute_return_expr(v, params)?;
            }
        }
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            substitute_return_expr(l, params)?;
            substitute_return_expr(r, params)?;
        }
        ReturnExpr::Not(e) => substitute_return_expr(e, params)?,
        ReturnExpr::Compare(l, _, r) => {
            substitute_return_expr(l, params)?;
            substitute_return_expr(r, params)?;
        }
        ReturnExpr::IsNull(e) => substitute_return_expr(e, params)?,
        ReturnExpr::In(needle, haystack) => {
            substitute_return_expr(needle, params)?;
            substitute_return_expr(haystack, params)?;
        }
        // No `$param`-able position -- var/labels are identifiers, not
        // expressions.
        ReturnExpr::HasLabel(..) => {}
        ReturnExpr::PatternComprehension {
            pattern,
            where_clause,
            projection,
            ..
        } => {
            substitute_pattern(pattern, params)?;
            if let Some(w) = where_clause {
                substitute_expr(w, params)?;
            }
            substitute_return_expr(projection, params)?;
        }
        ReturnExpr::ExistsPattern {
            pattern,
            where_clause,
        } => {
            substitute_pattern(pattern, params)?;
            if let Some(w) = where_clause {
                substitute_expr(w, params)?;
            }
        }
    }
    Ok(())
}

fn substitute_literal(
    lit: &mut Literal,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    if let Literal::Param(name) = lit {
        let value = params
            .get(name)
            .ok_or_else(|| QueryError::MissingParam(name.clone()))?;
        *lit = property_value_to_literal(name, value)?;
    }
    Ok(())
}

fn property_value_to_literal(name: &str, pv: &PropertyValue) -> Result<Literal, QueryError> {
    Ok(match pv {
        PropertyValue::Null => Literal::Null,
        PropertyValue::Bool(b) => Literal::Bool(*b),
        PropertyValue::Int(i) => Literal::Int(*i),
        PropertyValue::Float(f) => Literal::Float(*f),
        PropertyValue::String(s) => Literal::String(s.clone()),
        // `Literal` has no temporal variant (there's no temporal *literal*
        // syntax in Cypher -- see cypher.pest's docs: a date/duration is
        // always built via `date(...)`/`duration(...)`), so a Date/
        // Duration bound in from Rust as a `$param` has nowhere to
        // substitute to. Erroring here (not silently dropping to Null) is
        // the same "a real gap should say so, not produce a plausible-
        // looking wrong answer" stance `apply_arith` already documents.
        PropertyValue::Date(_)
        | PropertyValue::Duration { .. }
        | PropertyValue::LocalTime(_)
        | PropertyValue::Time { .. }
        | PropertyValue::LocalDateTime { .. }
        | PropertyValue::DateTime { .. } => {
            return Err(QueryError::Type(format!(
                "${name}: passing a temporal value as a query parameter isn't supported yet"
            )))
        }
        // Same "no literal syntax to substitute to" gap as the temporal
        // variants above -- there's no `Literal::List`. A list-valued
        // `$param` used in ordinary expression position (`RETURN $x`,
        // `WHERE y IN $x`, ...) substitutes into a `ReturnExpr::ListLit`
        // instead (`property_value_to_return_expr`, used by
        // `substitute_return_expr`'s own `Lit` arm, not this function) --
        // reaching here at all means a list-valued param was used
        // somewhere a bare `Literal` is structurally required (only
        // pattern-level `Expr::Compare`'s RHS, `n.prop = $x`), which
        // doesn't have an equivalent list-valued shape to fall back to
        // either (comparing a scalar property against a list isn't
        // meaningful).
        PropertyValue::List(_) => {
            return Err(QueryError::Type(format!(
                "${name}: a list-valued query parameter can't be used here (only in ordinary \
                 expression position, not a pattern-level property comparison)"
            )))
        }
        // Same "no literal syntax to substitute to" gap as `List` above --
        // there's no `Literal::Map` either. A map-valued `$param` used in
        // ordinary expression position substitutes into a `ReturnExpr::
        // MapLit` instead (`property_value_to_return_expr`, not this
        // function).
        PropertyValue::Map(_) => {
            return Err(QueryError::Type(format!(
                "${name}: a map-valued query parameter can't be used here (only in ordinary \
                 expression position, not a pattern-level property comparison)"
            )))
        }
    })
}

/// Converts a parameter's stored `PropertyValue` into the `ReturnExpr`
/// that should replace a `ReturnExpr::Lit(Literal::Param(name))` node --
/// a bare `Literal` for a scalar value (delegates to
/// `property_value_to_literal`), a `ReturnExpr::ListLit` for a list value
/// (recursively -- a param list can itself contain nested lists/maps), or
/// a `ReturnExpr::MapLit` for a map value (same recursion, TCK's Map2/
/// Map3, Unwind1 `[6]`/`[14]`).
fn property_value_to_return_expr(name: &str, pv: &PropertyValue) -> Result<ReturnExpr, QueryError> {
    Ok(match pv {
        PropertyValue::List(items) => ReturnExpr::ListLit(
            items
                .iter()
                .map(|item| property_value_to_return_expr(name, item))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        PropertyValue::Map(entries) => ReturnExpr::MapLit(
            entries
                .iter()
                .map(|(key, value)| Ok((key.clone(), property_value_to_return_expr(name, value)?)))
                .collect::<Result<Vec<_>, QueryError>>()?,
        ),
        other => ReturnExpr::Lit(property_value_to_literal(name, other)?),
    })
}
