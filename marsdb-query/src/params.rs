use std::collections::HashMap;

use marsdb_graph::PropertyValue;

use crate::ast::{
    CallClause, CallYield, Expr, Literal, MergeClause, NodePattern, Pattern, QueryClause,
    QueryPart, ReturnExpr, ReturnTail, SetItem, Statement, Tail, UnwindClause, WithClause,
    WithExpr,
};
use crate::error::QueryError;

/// One step descending one field/Vec-index toward a `$param` leaf,
/// mirroring the existing `substitute_*` functions' own recursion
/// structure below. Recorded while walking, so the same leaf can be
/// found again later without re-walking the whole tree.
///
/// Two variants cover every descent in this file: `Index` for stepping
/// into the Nth element of a `Vec`-typed child (a pattern in a list of
/// patterns, a hop in a pattern's chain, an item in a `RETURN` list,
/// ...), and `Field` for stepping into a named, non-`Vec` child (a
/// struct field like `where_clause`/`with`, an enum operand like the
/// left/right side of `And`/`Or`, or a name that disambiguates which of
/// several same-shaped `Vec`s an `Index` that follows belongs to --
/// e.g. `MergeClause`'s `on_create` vs `on_match`, each indexed
/// separately). A `Field` immediately followed by an `Index` reads as
/// "the Nth element of that named Vec".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathStep {
    /// Descend into a named, non-`Vec` child, or name the `Vec` that an
    /// immediately following `Index` indexes into.
    Field(&'static str),
    /// Descend into the Nth element of a `Vec`-typed child.
    Index(usize),
}

/// One resolved `$param` occurrence: which parameter, and where in the
/// `Statement` tree it landed.
#[derive(Debug, Clone)]
pub struct ParamSite {
    pub name: String,
    pub path: Vec<PathStep>,
}

/// Resolves every `$name` placeholder in `stmt` to a concrete `Literal`
/// using `params`, in place. Called before execution so the executor never
/// sees `Literal::Param` — see the `unreachable!` in
/// `executor::literal_to_value`.
pub fn substitute_params(
    stmt: &mut Statement,
    params: &HashMap<String, PropertyValue>,
) -> Result<(), QueryError> {
    substitute_params_tracked(stmt, params)?;
    Ok(())
}

/// Same as `substitute_params`, but also returns a `ParamSite` for
/// every `$name` occurrence resolved, recording the path to reach it.
pub fn substitute_params_tracked(
    stmt: &mut Statement,
    params: &HashMap<String, PropertyValue>,
) -> Result<Vec<ParamSite>, QueryError> {
    let mut sites = Vec::new();
    let mut path = Vec::new();
    substitute_params_inner(stmt, params, &mut path, &mut sites)?;
    Ok(sites)
}

fn substitute_params_inner(
    stmt: &mut Statement,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match stmt {
        // Bare keywords, nothing to substitute into.
        Statement::Begin | Statement::Commit | Statement::Rollback => {}
        Statement::Create(patterns) => {
            path.push(PathStep::Field("patterns"));
            for (i, pattern) in patterns.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_pattern_inner(pattern, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        // No `$param`-able position -- label/prop are identifiers, not
        // expressions.
        Statement::CreateIndex { .. } => {}
        Statement::Explain(inner) => {
            path.push(PathStep::Field("inner"));
            substitute_params_inner(inner, params, path, sites)?;
            path.pop();
        }
        Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        } => {
            path.push(PathStep::Field("clauses"));
            for (i, clause) in clauses.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_query_clause_inner(clause, params, path, sites)?;
                path.pop();
            }
            path.pop();
            if let Some(tail) = tail {
                path.push(PathStep::Field("tail"));
                substitute_tail_inner(tail, params, path, sites)?;
                path.pop();
            }
            if let Some(items) = order_by {
                path.push(PathStep::Field("order_by"));
                for (i, (expr, _)) in items.iter_mut().enumerate() {
                    path.push(PathStep::Index(i));
                    substitute_return_expr_inner(expr, params, path, sites)?;
                    path.pop();
                }
                path.pop();
            }
            if let Some(expr) = skip {
                path.push(PathStep::Field("skip"));
                substitute_return_expr_inner(expr, params, path, sites)?;
                path.pop();
            }
            if let Some(expr) = limit {
                path.push(PathStep::Field("limit"));
                substitute_return_expr_inner(expr, params, path, sites)?;
                path.pop();
            }
        }
        Statement::Union { parts, .. } => {
            path.push(PathStep::Field("parts"));
            for (i, part) in parts.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_params_inner(part, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        Statement::StandaloneCall(call) => {
            path.push(PathStep::Field("call"));
            substitute_call_clause_inner(call, params, path, sites)?;
            path.pop();
        }
    }
    Ok(())
}

/// `CallClause::args: None` (the implicit-argument form, `CALL proc` with
/// no parens) has nothing to substitute here — each declared input
/// resolves from a same-named `$param` at execution time instead, once
/// the procedure's signature is known.
fn substitute_call_clause_inner(
    call: &mut CallClause,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    if let Some(args) = &mut call.args {
        path.push(PathStep::Field("args"));
        for (i, arg) in args.iter_mut().enumerate() {
            path.push(PathStep::Index(i));
            substitute_return_expr_inner(arg, params, path, sites)?;
            path.pop();
        }
        path.pop();
    }
    if let Some(CallYield::Items(_, Some(where_expr))) = &mut call.yield_items {
        path.push(PathStep::Field("yield_where"));
        substitute_expr_inner(where_expr, params, path, sites)?;
        path.pop();
    }
    Ok(())
}

fn substitute_query_clause_inner(
    clause: &mut QueryClause,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match clause {
        QueryClause::Match(part) => substitute_query_part_inner(part, params, path, sites),
        QueryClause::Unwind(u) => substitute_unwind_clause_inner(u, params, path, sites),
        QueryClause::Merge(m) => substitute_merge_clause_inner(m, params, path, sites),
        QueryClause::With(with) => substitute_with_clause_inner(with, params, path, sites),
        QueryClause::Set(items) => {
            path.push(PathStep::Field("items"));
            for (i, item) in items.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_set_item_inner(item, params, path, sites)?;
                path.pop();
            }
            path.pop();
            Ok(())
        }
        QueryClause::Delete { items, detach: _ } => {
            path.push(PathStep::Field("items"));
            for (i, expr) in items.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(expr, params, path, sites)?;
                path.pop();
            }
            path.pop();
            Ok(())
        }
        // No `$param`-able position -- `RemoveItem` is a bare prop/label
        // path, not a value expression.
        QueryClause::Remove(_) => Ok(()),
        QueryClause::Create(patterns) => {
            path.push(PathStep::Field("patterns"));
            for (i, pattern) in patterns.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_pattern_inner(pattern, params, path, sites)?;
                path.pop();
            }
            path.pop();
            Ok(())
        }
        QueryClause::Call(call) => substitute_call_clause_inner(call, params, path, sites),
    }
}

/// Shared by every `SetItem` list this file substitutes into (`SET`'s own
/// `QueryClause`/`Tail` forms, and `MERGE`'s `ON CREATE`/`ON MATCH SET`).
/// `Labels` has no `$param`-able position; `Prop`/`MapAssign` each carry
/// one `ReturnExpr` value to recurse into.
fn substitute_set_item_inner(
    item: &mut SetItem,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match item {
        SetItem::Prop(_, value) | SetItem::MapAssign { value, .. } => {
            path.push(PathStep::Field("value"));
            let r = substitute_return_expr_inner(value, params, path, sites);
            path.pop();
            r
        }
        SetItem::Labels(..) => Ok(()),
    }
}

fn substitute_merge_clause_inner(
    m: &mut MergeClause,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("pattern"));
    substitute_pattern_inner(&mut m.pattern, params, path, sites)?;
    path.pop();
    // Split from the original `on_create.iter_mut().chain(on_match.iter_mut())`
    // into two separate loops so each item's path names which of the two
    // Vecs it came from -- same iteration order (all of `on_create` then
    // all of `on_match`), so substitution order (and thus which error
    // surfaces first on a missing param) is unchanged.
    path.push(PathStep::Field("on_create"));
    for (i, item) in m.on_create.iter_mut().enumerate() {
        path.push(PathStep::Index(i));
        substitute_set_item_inner(item, params, path, sites)?;
        path.pop();
    }
    path.pop();
    path.push(PathStep::Field("on_match"));
    for (i, item) in m.on_match.iter_mut().enumerate() {
        path.push(PathStep::Index(i));
        substitute_set_item_inner(item, params, path, sites)?;
        path.pop();
    }
    path.pop();
    if let Some(with) = &mut m.with {
        path.push(PathStep::Field("with"));
        substitute_with_clause_inner(with, params, path, sites)?;
        path.pop();
    }
    Ok(())
}

fn substitute_query_part_inner(
    part: &mut QueryPart,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("pattern"));
    substitute_pattern_inner(&mut part.pattern, params, path, sites)?;
    path.pop();
    if let Some(expr) = &mut part.where_clause {
        path.push(PathStep::Field("where_clause"));
        substitute_expr_inner(expr, params, path, sites)?;
        path.pop();
    }
    if let Some(with) = &mut part.with {
        path.push(PathStep::Field("with"));
        substitute_with_clause_inner(with, params, path, sites)?;
        path.pop();
    }
    Ok(())
}

fn substitute_unwind_clause_inner(
    u: &mut UnwindClause,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("source"));
    substitute_return_expr_inner(&mut u.source.0, params, path, sites)?;
    path.pop();
    if let Some(expr) = &mut u.where_clause {
        path.push(PathStep::Field("where_clause"));
        substitute_with_expr_inner(expr, params, path, sites)?;
        path.pop();
    }
    if let Some(with) = &mut u.with {
        path.push(PathStep::Field("with"));
        substitute_with_clause_inner(with, params, path, sites)?;
        path.pop();
    }
    Ok(())
}

fn substitute_with_clause_inner(
    with: &mut WithClause,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("items"));
    for (i, item) in with.items.iter_mut().enumerate() {
        path.push(PathStep::Index(i));
        substitute_return_expr_inner(&mut item.expr, params, path, sites)?;
        path.pop();
    }
    path.pop();
    if let Some(where_clause) = &mut with.where_clause {
        path.push(PathStep::Field("where_clause"));
        substitute_with_expr_inner(where_clause, params, path, sites)?;
        path.pop();
    }
    if let Some(items) = &mut with.order_by {
        path.push(PathStep::Field("order_by"));
        for (i, (expr, _)) in items.iter_mut().enumerate() {
            path.push(PathStep::Index(i));
            substitute_return_expr_inner(expr, params, path, sites)?;
            path.pop();
        }
        path.pop();
    }
    if let Some(expr) = &mut with.skip {
        path.push(PathStep::Field("skip"));
        substitute_return_expr_inner(expr, params, path, sites)?;
        path.pop();
    }
    if let Some(expr) = &mut with.limit {
        path.push(PathStep::Field("limit"));
        substitute_return_expr_inner(expr, params, path, sites)?;
        path.pop();
    }
    Ok(())
}

fn substitute_with_expr_inner(
    expr: &mut WithExpr,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match expr {
        WithExpr::And(l, r) | WithExpr::Or(l, r) => {
            path.push(PathStep::Field("left"));
            substitute_with_expr_inner(l, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("right"));
            substitute_with_expr_inner(r, params, path, sites)?;
            path.pop();
        }
        WithExpr::Not(e) => {
            path.push(PathStep::Field("inner"));
            substitute_with_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        WithExpr::Compare(lhs, _, rhs) => {
            path.push(PathStep::Field("lhs"));
            substitute_return_expr_inner(lhs, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("rhs"));
            substitute_return_expr_inner(rhs, params, path, sites)?;
            path.pop();
        }
        WithExpr::IsNull(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        WithExpr::Bare(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
    }
    Ok(())
}

fn substitute_pattern_inner(
    pattern: &mut Pattern,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("start"));
    substitute_node_inner(&mut pattern.start, params, path, sites)?;
    path.pop();
    path.push(PathStep::Field("hops"));
    for (i, (rel, node)) in pattern.hops.iter_mut().enumerate() {
        path.push(PathStep::Index(i));
        path.push(PathStep::Field("rel_props"));
        for (j, (_, expr)) in rel.props.iter_mut().enumerate() {
            path.push(PathStep::Index(j));
            substitute_return_expr_inner(expr, params, path, sites)?;
            path.pop();
        }
        path.pop();
        path.push(PathStep::Field("node"));
        substitute_node_inner(node, params, path, sites)?;
        path.pop();
        path.pop();
    }
    path.pop();
    Ok(())
}

fn substitute_node_inner(
    node: &mut NodePattern,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    path.push(PathStep::Field("props"));
    for (i, (_, expr)) in node.props.iter_mut().enumerate() {
        path.push(PathStep::Index(i));
        substitute_return_expr_inner(expr, params, path, sites)?;
        path.pop();
    }
    path.pop();
    Ok(())
}

fn substitute_expr_inner(
    expr: &mut Expr,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match expr {
        Expr::And(l, r) | Expr::Or(l, r) => {
            path.push(PathStep::Field("left"));
            substitute_expr_inner(l, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("right"));
            substitute_expr_inner(r, params, path, sites)?;
            path.pop();
        }
        Expr::Not(e) => {
            path.push(PathStep::Field("inner"));
            substitute_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        Expr::Compare(_, _, lit) => {
            path.push(PathStep::Field("lit"));
            substitute_literal_inner(lit, params, path, sites)?;
            path.pop();
        }
        Expr::PropCompare(_, _, _) => {}
        Expr::IsNull(_) => {}
        Expr::HasLabel(_, _) => {}
        Expr::VarEq(_, _) => {}
        // Just variable names, like `VarEq` above; also planner-synthesized
        // only, never present in the AST this pass runs against.
        Expr::EdgeNotInSet { .. } => {}
        Expr::GeneralCompare(lhs, _, rhs) => {
            path.push(PathStep::Field("lhs"));
            substitute_return_expr_inner(lhs, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("rhs"));
            substitute_return_expr_inner(rhs, params, path, sites)?;
            path.pop();
        }
        Expr::GeneralIsNull(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        Expr::GeneralBare(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        Expr::Pattern(pattern) => {
            path.push(PathStep::Field("pattern"));
            substitute_pattern_inner(pattern, params, path, sites)?;
            path.pop();
        }
        Expr::Exists {
            pattern,
            where_clause,
        } => {
            path.push(PathStep::Field("pattern"));
            substitute_pattern_inner(pattern, params, path, sites)?;
            path.pop();
            if let Some(w) = where_clause {
                path.push(PathStep::Field("where_clause"));
                substitute_expr_inner(w, params, path, sites)?;
                path.pop();
            }
        }
        Expr::ExistsSubquery(stmt) => {
            path.push(PathStep::Field("subquery"));
            substitute_params_inner(stmt, params, path, sites)?;
            path.pop();
        }
    }
    Ok(())
}

fn substitute_tail_inner(
    tail: &mut Tail,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match tail {
        Tail::Return(items, _) => {
            path.push(PathStep::Field("items"));
            for (i, item) in items.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(&mut item.expr, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        // No `$param`-able position -- a bare `*`, nothing to substitute.
        Tail::ReturnStar(_) => {}
        Tail::Delete(exprs, ret) | Tail::DetachDelete(exprs, ret) => {
            path.push(PathStep::Field("exprs"));
            for (i, expr) in exprs.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(expr, params, path, sites)?;
                path.pop();
            }
            path.pop();
            path.push(PathStep::Field("ret"));
            substitute_return_tail_inner(ret, params, path, sites)?;
            path.pop();
        }
        Tail::Remove(_, ret) => {
            path.push(PathStep::Field("ret"));
            substitute_return_tail_inner(ret, params, path, sites)?;
            path.pop();
        }
        Tail::Set(items, ret) => {
            path.push(PathStep::Field("items"));
            for (i, item) in items.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_set_item_inner(item, params, path, sites)?;
                path.pop();
            }
            path.pop();
            path.push(PathStep::Field("ret"));
            substitute_return_tail_inner(ret, params, path, sites)?;
            path.pop();
        }
        Tail::Create(patterns, ret) => {
            path.push(PathStep::Field("patterns"));
            for (i, pattern) in patterns.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_pattern_inner(pattern, params, path, sites)?;
                path.pop();
            }
            path.pop();
            path.push(PathStep::Field("ret"));
            substitute_return_tail_inner(ret, params, path, sites)?;
            path.pop();
        }
    }
    Ok(())
}

/// Substitutes params in a mutating tail's optional trailing `RETURN`
/// (`MATCH (n) SET n.x = $x RETURN n` needs both the `SET`'s own `$x` *and*
/// nothing extra here since this RETURN has none — but `MATCH (n) DELETE n
/// RETURN $y` does).
fn substitute_return_tail_inner(
    ret: &mut Option<ReturnTail>,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    if let Some(rt) = ret {
        path.push(PathStep::Field("items"));
        for (i, item) in rt.items.iter_mut().enumerate() {
            path.push(PathStep::Index(i));
            substitute_return_expr_inner(&mut item.expr, params, path, sites)?;
            path.pop();
        }
        path.pop();
    }
    Ok(())
}

fn substitute_return_expr_inner(
    expr: &mut ReturnExpr,
    params: &HashMap<String, PropertyValue>,
    path: &mut Vec<PathStep>,
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    match expr {
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::CountStar => {}
        ReturnExpr::PatternPredicate(pattern) => {
            path.push(PathStep::Field("pattern"));
            substitute_pattern_inner(pattern, params, path, sites)?;
            path.pop();
        }
        // No `Literal::List` (no list-literal syntax in Cypher), so a
        // list-valued `$param` replaces the whole node with a
        // `ReturnExpr::ListLit` instead, recursively — everything
        // downstream already handles `ListLit` like any other list.
        ReturnExpr::Lit(Literal::Param(name)) => {
            let value = params
                .get(name)
                .ok_or_else(|| QueryError::MissingParam(name.clone()))?
                .clone();
            sites.push(ParamSite {
                name: name.clone(),
                path: path.clone(),
            });
            *expr = property_value_to_return_expr(name, &value)?;
        }
        ReturnExpr::Lit(_) => {}
        ReturnExpr::Call { args, .. } => {
            path.push(PathStep::Field("args"));
            for (i, arg) in args.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(arg, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        ReturnExpr::Case { test, whens, else_ } => {
            if let Some(t) = test {
                path.push(PathStep::Field("test"));
                substitute_return_expr_inner(t, params, path, sites)?;
                path.pop();
            }
            path.push(PathStep::Field("whens"));
            for (i, (when, then)) in whens.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                path.push(PathStep::Field("when"));
                substitute_return_expr_inner(when, params, path, sites)?;
                path.pop();
                path.push(PathStep::Field("then"));
                substitute_return_expr_inner(then, params, path, sites)?;
                path.pop();
                path.pop();
            }
            path.pop();
            if let Some(e) = else_ {
                path.push(PathStep::Field("else"));
                substitute_return_expr_inner(e, params, path, sites)?;
                path.pop();
            }
        }
        ReturnExpr::Arith(l, _, r) => {
            path.push(PathStep::Field("left"));
            substitute_return_expr_inner(l, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("right"));
            substitute_return_expr_inner(r, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::Neg(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::ListLit(items) => {
            path.push(PathStep::Field("items"));
            for (i, item) in items.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(item, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        ReturnExpr::Index(base, index) => {
            path.push(PathStep::Field("base"));
            substitute_return_expr_inner(base, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("index"));
            substitute_return_expr_inner(index, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::PropOf(base, _) => {
            path.push(PathStep::Field("base"));
            substitute_return_expr_inner(base, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::Slice(base, start, end) => {
            path.push(PathStep::Field("base"));
            substitute_return_expr_inner(base, params, path, sites)?;
            path.pop();
            if let Some(s) = start {
                path.push(PathStep::Field("start"));
                substitute_return_expr_inner(s, params, path, sites)?;
                path.pop();
            }
            if let Some(e) = end {
                path.push(PathStep::Field("end"));
                substitute_return_expr_inner(e, params, path, sites)?;
                path.pop();
            }
        }
        ReturnExpr::ListComp {
            source,
            where_clause,
            project,
            ..
        } => {
            path.push(PathStep::Field("source"));
            substitute_return_expr_inner(source, params, path, sites)?;
            path.pop();
            if let Some(w) = where_clause {
                path.push(PathStep::Field("where_clause"));
                substitute_return_expr_inner(w, params, path, sites)?;
                path.pop();
            }
            if let Some(p) = project {
                path.push(PathStep::Field("project"));
                substitute_return_expr_inner(p, params, path, sites)?;
                path.pop();
            }
        }
        ReturnExpr::Quantifier {
            source,
            where_clause,
            ..
        } => {
            path.push(PathStep::Field("source"));
            substitute_return_expr_inner(source, params, path, sites)?;
            path.pop();
            if let Some(w) = where_clause {
                path.push(PathStep::Field("where_clause"));
                substitute_return_expr_inner(w, params, path, sites)?;
                path.pop();
            }
        }
        ReturnExpr::MapLit(entries) => {
            path.push(PathStep::Field("entries"));
            for (i, (_, v)) in entries.iter_mut().enumerate() {
                path.push(PathStep::Index(i));
                substitute_return_expr_inner(v, params, path, sites)?;
                path.pop();
            }
            path.pop();
        }
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            path.push(PathStep::Field("left"));
            substitute_return_expr_inner(l, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("right"));
            substitute_return_expr_inner(r, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::Not(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::Compare(l, _, r) => {
            path.push(PathStep::Field("left"));
            substitute_return_expr_inner(l, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("right"));
            substitute_return_expr_inner(r, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::IsNull(e) => {
            path.push(PathStep::Field("inner"));
            substitute_return_expr_inner(e, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::In(needle, haystack) => {
            path.push(PathStep::Field("needle"));
            substitute_return_expr_inner(needle, params, path, sites)?;
            path.pop();
            path.push(PathStep::Field("haystack"));
            substitute_return_expr_inner(haystack, params, path, sites)?;
            path.pop();
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
            path.push(PathStep::Field("pattern"));
            substitute_pattern_inner(pattern, params, path, sites)?;
            path.pop();
            if let Some(w) = where_clause {
                path.push(PathStep::Field("where_clause"));
                substitute_expr_inner(w, params, path, sites)?;
                path.pop();
            }
            path.push(PathStep::Field("projection"));
            substitute_return_expr_inner(projection, params, path, sites)?;
            path.pop();
        }
        ReturnExpr::ExistsPattern {
            pattern,
            where_clause,
        } => {
            path.push(PathStep::Field("pattern"));
            substitute_pattern_inner(pattern, params, path, sites)?;
            path.pop();
            if let Some(w) = where_clause {
                path.push(PathStep::Field("where_clause"));
                substitute_expr_inner(w, params, path, sites)?;
                path.pop();
            }
        }
        ReturnExpr::ExistsSubquery(stmt) => {
            path.push(PathStep::Field("subquery"));
            substitute_params_inner(stmt, params, path, sites)?;
            path.pop();
        }
    }
    Ok(())
}

fn substitute_literal_inner(
    lit: &mut Literal,
    params: &HashMap<String, PropertyValue>,
    path: &[PathStep],
    sites: &mut Vec<ParamSite>,
) -> Result<(), QueryError> {
    if let Literal::Param(name) = lit {
        let value = params
            .get(name)
            .ok_or_else(|| QueryError::MissingParam(name.clone()))?;
        sites.push(ParamSite {
            name: name.clone(),
            path: path.to_vec(),
        });
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
        // `Literal` has no temporal variant (Cypher builds dates/durations
        // via `date(...)`/`duration(...)`, never literal syntax), so a
        // temporal `$param` has nowhere to substitute in this function,
        // which only produces a bare `Literal` for the one spot that
        // structurally requires one (`Expr::Compare`'s RHS, `n.prop = $x`).
        // In ordinary expression position it substitutes into a
        // `ReturnExpr::Call` instead — see `property_value_to_return_expr`.
        PropertyValue::Date(_)
        | PropertyValue::Duration { .. }
        | PropertyValue::LocalTime(_)
        | PropertyValue::Time { .. }
        | PropertyValue::LocalDateTime { .. }
        | PropertyValue::DateTime { .. } => {
            return Err(QueryError::Type(format!(
                "${name}: passing a temporal value as a query parameter isn't supported in a \
                 pattern-level property comparison (only in ordinary expression position)"
            )))
        }
        // Same gap as the temporal variants: no `Literal::List`. Reaching
        // here means a list-valued param was used where a bare `Literal`
        // is structurally required (`Expr::Compare`'s RHS), which has no
        // meaningful list-valued fallback either.
        PropertyValue::List(_) => {
            return Err(QueryError::Type(format!(
                "${name}: a list-valued query parameter can't be used here (only in ordinary \
                 expression position, not a pattern-level property comparison)"
            )))
        }
        // Same gap as `List` above: no `Literal::Map`.
        PropertyValue::Map(_) => {
            return Err(QueryError::Type(format!(
                "${name}: a map-valued query parameter can't be used here (only in ordinary \
                 expression position, not a pattern-level property comparison)"
            )))
        }
    })
}

/// Converts a parameter's stored `PropertyValue` into the `ReturnExpr`
/// that should replace a `ReturnExpr::Lit(Literal::Param(name))` node: a
/// bare `Literal` for a scalar (`property_value_to_literal`), or a
/// recursive `ReturnExpr::ListLit`/`MapLit` for a list/map value.
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
        // `property_value_to_literal`'s temporal arm can't represent these
        // (no temporal literal syntax in Cypher). Ordinary expression
        // position allows a `ReturnExpr::Call`, and every temporal
        // constructor accepts its own formatted string back, so a
        // temporal-valued param becomes a call to the matching
        // constructor over its formatted string instead of erroring.
        PropertyValue::Date(d) => temporal_call("date", crate::temporal::format_date(*d)),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => temporal_call(
            "duration",
            crate::temporal::format_duration(*months, *days, *seconds, *nanos),
        ),
        PropertyValue::LocalTime(nanos_of_day) => temporal_call(
            "localtime",
            crate::temporal::format_local_time(*nanos_of_day),
        ),
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => temporal_call(
            "time",
            crate::temporal::format_time(*nanos_of_day, *offset_seconds),
        ),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => temporal_call(
            "localdatetime",
            crate::temporal::format_local_date_time(*epoch_seconds, *nanos),
        ),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => temporal_call(
            "datetime",
            crate::temporal::format_date_time(
                *epoch_seconds,
                *nanos,
                &crate::executor::tz_from_graph(zone),
            ),
        ),
        other => ReturnExpr::Lit(property_value_to_literal(name, other)?),
    })
}

fn temporal_call(name: &str, formatted: String) -> ReturnExpr {
    ReturnExpr::Call {
        name: name.to_string(),
        args: vec![ReturnExpr::Lit(Literal::String(formatted))],
        distinct: false,
    }
}
