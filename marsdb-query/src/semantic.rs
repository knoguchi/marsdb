//! Statement-level name binding and structural type validation.
//!
//! This pass runs after parsing/parameter substitution and before a storage
//! transaction is opened. It deliberately validates only types knowable from
//! query structure (node, relationship, list, map, path, scalar); property
//! value types remain data-dependent runtime checks.

use std::collections::HashMap;

use crate::ast::{
    is_aggregate_name, ArithOp, CallClause, CallYield, Expr, Literal, MergeClause, NodePattern,
    Pattern, QueryClause, RemoveItem, ReturnExpr, ReturnItem, ReturnTail, SetItem, Statement, Tail,
    UnwindClause, WithClause, WithExpr,
};
use crate::QueryError;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Node,
    Edge,
    Scalar,
    List(Box<Kind>),
    Map,
    Path,
    Unknown,
}

type Scope = HashMap<String, Kind>;

pub fn validate_statement(statement: &Statement) -> Result<(), QueryError> {
    match statement {
        // Session-level statements bind and reference nothing; whether one
        // is valid right now is session state, checked by `marsdb::Database`.
        Statement::Begin | Statement::Commit | Statement::Rollback => Ok(()),
        Statement::Create(patterns) => {
            let mut scope = Scope::new();
            for pattern in patterns {
                bind_create_pattern(pattern, &mut scope)?;
            }
            Ok(())
        }
        // No pattern/expression scoping to validate -- label/prop are
        // plain identifiers.
        Statement::CreateIndex { .. } => Ok(()),
        Statement::Explain(inner) => validate_statement(inner),
        // Each part is independently scoped (no bindings cross a UNION
        // boundary). The column-match check needs each part's evaluated
        // `QueryResult.columns`, not available here, so it lives in
        // `executor::materialize_union` instead.
        Statement::Union { parts, .. } => {
            for part in parts {
                validate_statement(part)?;
            }
            Ok(())
        }
        Statement::Match {
            clauses,
            tail,
            order_by,
            ..
        } => validate_match_clauses(clauses, tail, order_by, Scope::new(), true),
        // A standalone CALL is the whole statement, so it starts from an
        // empty scope -- nothing precedes it to shadow.
        Statement::StandaloneCall(call) => validate_call_clause(call, &mut Scope::new()),
    }
}

/// `CALL proc.name(args) [YIELD ...]`. No procedure registry exists at this
/// pass, so arity/existence/argument-type checks happen at execution time
/// instead (see `executor::ExecutionOptions::procedures`). This only
/// checks what's knowable from AST structure: an aggregate used as a CALL
/// argument, and a `YIELD` output name that shadows an outer variable or
/// repeats an earlier item's own output name in the same `YIELD`.
/// `CallYield::Star` only appears in the standalone-call grammar, which has
/// no bindings to shadow.
fn validate_call_clause(call: &CallClause, scope: &mut Scope) -> Result<(), QueryError> {
    if let Some(args) = &call.args {
        for arg in args {
            infer_expr(arg, scope)?;
            if crate::executor::contains_aggregate(arg) {
                return Err(semantic(
                    "an aggregate function can't be used as a CALL argument",
                ));
            }
        }
    }
    if let Some(CallYield::Items(items, where_expr)) = &call.yield_items {
        for (name, alias) in items {
            let out_name = alias.clone().unwrap_or_else(|| name.clone());
            if scope.contains_key(&out_name) {
                return Err(semantic(format!(
                    "'{out_name}' is already bound -- CALL's YIELD can't reuse an already-bound \
                     name, whether from an outer scope or another output in the same YIELD"
                )));
            }
            scope.insert(out_name, Kind::Unknown);
        }
        if let Some(w) = where_expr.as_deref() {
            validate_pattern_expr(w, scope)?;
        }
    }
    Ok(())
}

/// The body of `Statement::Match`'s own validation, factored out so
/// `Expr::ExistsSubquery` (a nested `exists { MATCH ... RETURN ... }`) can
/// reuse it correlated against the enclosing scope instead of a fresh one,
/// with `allow_mutation: false` -- an `exists {}` body may only contain
/// reading clauses, never an updating one.
fn validate_match_clauses(
    clauses: &[QueryClause],
    tail: &Option<Tail>,
    order_by: &Option<Vec<(ReturnExpr, crate::ast::SortDir)>>,
    mut scope: Scope,
    allow_mutation: bool,
) -> Result<(), QueryError> {
    let reject_mutation = |clause_name: &str| -> Result<(), QueryError> {
        if allow_mutation {
            Ok(())
        } else {
            Err(semantic(format!(
                "exists {{}} can't contain an updating clause ({clause_name}) -- only reading \
                 clauses (MATCH/UNWIND/WITH) are allowed inside it"
            )))
        }
    };
    for clause in clauses {
        match clause {
            QueryClause::Match(part) => {
                let prior_scope = scope.clone();
                bind_match_pattern(&part.pattern, &mut scope)?;
                if part.shortest_path {
                    let start = part.pattern.start.var.as_deref().ok_or_else(|| {
                        semantic("shortestPath() start node must have a variable")
                    })?;
                    let end = part
                        .pattern
                        .hops
                        .first()
                        .and_then(|(_, node)| node.var.as_deref())
                        .ok_or_else(|| semantic("shortestPath() end node must have a variable"))?;
                    require_kind(&prior_scope, start, &Kind::Node, "shortestPath endpoint")?;
                    require_kind(&prior_scope, end, &Kind::Node, "shortestPath endpoint")?;
                }
                if let Some(path_var) = &part.path_var {
                    bind_kind(&mut scope, path_var, Kind::Path, "path variable")?;
                }
                if let Some(expr) = &part.where_clause {
                    validate_pattern_expr(expr, &scope)?;
                }
                apply_with(&part.with, &mut scope)?;
            }
            QueryClause::Unwind(clause) => bind_unwind(clause, &mut scope)?,
            QueryClause::Merge(clause) => {
                reject_mutation("MERGE")?;
                bind_merge(clause, &mut scope)?
            }
            QueryClause::With(with) => scope = project_with(with, &scope)?,
            QueryClause::Set(items) => {
                reject_mutation("SET")?;
                for item in items {
                    validate_set_item(item, &scope)?;
                }
            }
            QueryClause::Delete { items, detach: _ } => {
                reject_mutation("DELETE")?;
                for expr in items {
                    validate_delete_target(expr, &scope)?;
                }
            }
            QueryClause::Remove(items) => {
                reject_mutation("REMOVE")?;
                for item in items {
                    validate_remove_item(item, &scope)?;
                }
            }
            QueryClause::Create(patterns) => {
                reject_mutation("CREATE")?;
                for pattern in patterns {
                    bind_create_pattern(pattern, &mut scope)?;
                }
            }
            // A procedure is opaque to MarsDB and might write, so it's
            // rejected inside `exists {}` too (same reasoning as
            // `executor::is_read_only`).
            QueryClause::Call(call) => {
                reject_mutation("CALL")?;
                validate_call_clause(call, &mut scope)?;
                apply_with(&call.with, &mut scope)?;
            }
        }
    }

    let input_scope = scope.clone();
    let output_scope = validate_tail(tail, &mut scope, allow_mutation)?;
    if let Some(order_by) = order_by {
        let mut order_scope = input_scope;
        order_scope.extend(output_scope);
        // An aggregate in RETURN's ORDER BY is only legal when RETURN
        // itself is aggregating -- the ORDER BY then runs against the
        // already-collapsed grouped rows.
        let tail_items: Option<&[ReturnItem]> = match tail {
            Some(Tail::Return(items, _)) => Some(items),
            _ => None,
        };
        let tail_aggregates = tail_items.is_some_and(crate::executor::has_aggregate);
        for (expr, _) in order_by {
            if tail_aggregates {
                // An ORDER BY item repeating a RETURN item's expression or
                // own alias (`RETURN sum(x) AS s ORDER BY sum(x)`/`ORDER BY
                // s`) refers to that already-aggregated item -- its kind is
                // known from `output_scope`, and pre-aggregation bindings
                // no longer exist to re-run `infer_expr` against.
                if tail_items
                    .unwrap()
                    .iter()
                    .enumerate()
                    .any(|(i, item)| crate::executor::item_matches_leaf(expr, i, item))
                {
                    continue;
                }
                // Not a verbatim match -- may still be a composed expression
                // (an aggregate combined with other values, or a plain
                // expression referencing a pre-aggregation variable) that
                // `resolve_grouped_rows`/`rewrite_composed_item` (executor.rs)
                // can evaluate the same way a composed RETURN item would.
                crate::executor::validate_order_by_composed_expr(expr, tail_items.unwrap())?;
                continue;
            }
            if crate::executor::contains_aggregate(expr) {
                return Err(semantic(
                    "ORDER BY cannot use an aggregate function unless RETURN itself \
                     is aggregating",
                ));
            }
            infer_expr(expr, &order_scope)?;
        }
    }
    Ok(())
}

fn bind_unwind(clause: &UnwindClause, scope: &mut Scope) -> Result<(), QueryError> {
    let source_kind = infer_expr(&clause.source.0, scope)?;
    let element_kind = match source_kind {
        Kind::List(element) => *element,
        // `Scalar` isn't rejected: most function calls type as `Scalar`
        // even when they return a list at runtime (the `Kind` system
        // doesn't model every builtin's return shape), so this defers to
        // the real `Value::List` check in `eval_unwind`.
        Kind::Unknown | Kind::Scalar => Kind::Unknown,
        other => {
            return Err(semantic(format!(
                "UNWIND source is {}, not a list",
                kind_name(&other)
            )))
        }
    };
    scope.insert(clause.var.clone(), element_kind);
    if let Some(expr) = &clause.where_clause {
        validate_with_expr(expr, scope)?;
    }
    apply_with(&clause.with, scope)
}

fn bind_merge(clause: &MergeClause, scope: &mut Scope) -> Result<(), QueryError> {
    let pattern = &clause.pattern;
    // A bare already-bound node with no relationship (`MATCH (a) MERGE (a)`)
    // does nothing real -- just re-states a var that already exists. A bound
    // start node used as a relationship endpoint stays legitimate, so this
    // only fires when there are no hops. Checked at compile time rather than
    // only at runtime, since a zero-row MATCH would otherwise skip it.
    if pattern.hops.is_empty() {
        if let Some(var) = &pattern.start.var {
            if pattern.start.labels.is_empty()
                && !pattern.start.has_explicit_props
                && scope.contains_key(var)
            {
                return Err(semantic(format!(
                    "'{var}' is already bound — MERGE ({var}) with no relationship and no \
                     labels/properties doesn't search for or create anything"
                )));
            }
        }
    }
    // Same reasoning as CREATE's own node check -- MERGE might need to
    // create any node its pattern names, so an already-bound node can't
    // also carry a new label/property predicate. Covers every node token
    // besides the hopless-and-predicate-free case handled above.
    check_no_new_predicates_on_bound_node(&pattern.start, scope, "MERGE")?;
    for (_, node) in &pattern.hops {
        check_no_new_predicates_on_bound_node(node, scope, "MERGE")?;
    }
    // Unlike a node endpoint, MERGE never reuses an already-bound
    // relationship as its own pattern token -- there's no "search using
    // this specific existing edge" mode.
    for (rel, _) in &pattern.hops {
        if let Some(var) = &rel.var {
            if scope.contains_key(var) {
                return Err(semantic(format!(
                    "'{var}' is already bound — MERGE can't reuse an existing relationship \
                     variable as its own pattern token"
                )));
            }
        }
        // Same reasoning as CREATE's own check -- MERGE might need to
        // create this relationship on no-match, and a brand new edge with
        // no type or more than one type is meaningless.
        if rel.rel_types.len() != 1 {
            return Err(semantic(
                "MERGE requires exactly one explicit relationship type (e.g. -[:KNOWS]->) -- an \
                 untyped or multi-typed relationship pattern can't be created if the MERGE \
                 doesn't find a match",
            ));
        }
    }
    bind_match_pattern(pattern, scope)?;
    if let Some(path_var) = &clause.path_var {
        bind_kind(scope, path_var, Kind::Path, "path variable")?;
    }
    for item in clause.on_create.iter().chain(&clause.on_match) {
        validate_set_item(item, scope)?;
    }
    apply_with(&clause.with, scope)
}

fn bind_match_pattern(pattern: &Pattern, scope: &mut Scope) -> Result<(), QueryError> {
    if let Some(var) = &pattern.start.var {
        bind_kind(scope, var, Kind::Node, "node pattern")?;
    }
    for (rel, node) in &pattern.hops {
        if let Some(var) = &rel.var {
            // A variable-length hop's `rel.var` binds a list of
            // relationships (`[r:TYPE*1..3]`), not a single edge.
            let kind = if rel.hop_range.is_some() {
                Kind::List(Box::new(Kind::Edge))
            } else {
                Kind::Edge
            };
            bind_kind(scope, var, kind, "relationship pattern")?;
        }
        if let Some(var) = &node.var {
            bind_kind(scope, var, Kind::Node, "node pattern")?;
        }
    }
    Ok(())
}

fn bind_create_pattern(pattern: &Pattern, scope: &mut Scope) -> Result<(), QueryError> {
    validate_props(&pattern.start.props, scope)?;
    check_create_node_not_already_bound(&pattern.start, scope, pattern.hops.is_empty())?;
    if let Some(var) = &pattern.start.var {
        bind_kind(scope, var, Kind::Node, "CREATE node")?;
    }
    for (rel, node) in &pattern.hops {
        validate_props(&node.props, scope)?;
        check_create_node_not_already_bound(node, scope, false)?;
        if let Some(var) = &node.var {
            bind_kind(scope, var, Kind::Node, "CREATE node")?;
        }
        // Unlike MATCH (where an untyped/multi-typed hop means "any of
        // these"), CREATE always makes exactly one new relationship, which
        // needs exactly one explicit `:TYPE` -- never inferred, defaulted,
        // or a `|`-alternative list.
        if rel.rel_types.len() != 1 {
            return Err(semantic(
                "CREATE requires exactly one explicit relationship type (e.g. -[:KNOWS]->) -- \
                 unlike MATCH, an untyped or multi-typed relationship pattern can't be created",
            ));
        }
        validate_props(&rel.props, scope)?;
        if let Some(var) = &rel.var {
            bind_kind(scope, var, Kind::Edge, "CREATE relationship")?;
        }
    }
    Ok(())
}

/// Mirrors `Executor::resolve_or_create_node`'s already-bound rejection at
/// compile time -- a node token naming a variable already in `scope`
/// either does nothing real (`is_bare`: `MATCH (a) CREATE (a)`) or would
/// silently drop user-written labels/props onto an existing node
/// (`MATCH (a) CREATE (a {x: 1})`). Checked here rather than only at
/// runtime, since a zero-row MATCH would otherwise skip it.
fn check_create_node_not_already_bound(
    node: &NodePattern,
    scope: &Scope,
    is_bare: bool,
) -> Result<(), QueryError> {
    let Some(var) = &node.var else {
        return Ok(());
    };
    if !scope.contains_key(var) {
        return Ok(());
    }
    if is_bare && node.labels.is_empty() && !node.has_explicit_props {
        return Err(semantic(format!(
            "'{var}' is already bound — CREATE ({var}) with no relationship and no new \
             labels/properties doesn't create or connect anything"
        )));
    }
    check_no_new_predicates_on_bound_node(node, scope, "CREATE")
}

/// Shared by CREATE (via `check_create_node_not_already_bound` above) and
/// MERGE (`bind_merge`, for each node endpoint) -- both might need to
/// create a node the pattern names, so a variable already bound to an
/// existing node can't also carry a new label/property predicate (would
/// silently drop it on match, or ambiguously decide whether it applies on
/// create). Doesn't cover the "no relationship and no predicates" case --
/// CREATE and MERGE each check that separately.
fn check_no_new_predicates_on_bound_node(
    node: &NodePattern,
    scope: &Scope,
    verb: &str,
) -> Result<(), QueryError> {
    let Some(var) = &node.var else {
        return Ok(());
    };
    if !scope.contains_key(var) {
        return Ok(());
    }
    if !node.labels.is_empty() || node.has_explicit_props {
        return Err(semantic(format!(
            "'{var}' is already bound — {verb} can't add labels/properties to an existing node"
        )));
    }
    Ok(())
}

fn validate_props(props: &[(String, ReturnExpr)], scope: &Scope) -> Result<(), QueryError> {
    for (_, expr) in props {
        infer_expr(expr, scope)?;
    }
    Ok(())
}

fn apply_with(with: &Option<WithClause>, scope: &mut Scope) -> Result<(), QueryError> {
    if let Some(with) = with {
        *scope = project_with(with, scope)?;
    }
    Ok(())
}

fn project_with(with: &WithClause, input: &Scope) -> Result<Scope, QueryError> {
    // `WITH *`: `input` already reflects this clause's own new bindings
    // (`bind_match_pattern`/`bind_unwind`/`bind_merge` mutate `scope`
    // before calling `apply_with`), so no union with anything else is
    // needed here.
    let with_owned;
    let with: &WithClause = if with.star {
        let star_items = crate::executor::with_star_items(input.keys().cloned());
        let mut owned = with.clone();
        let mut items = star_items;
        items.extend(owned.items);
        owned.items = items;
        with_owned = owned;
        &with_owned
    } else {
        with
    };
    crate::executor::validate_return_items(&with.items)?;
    let mut projected = Scope::new();
    for (index, item) in with.items.iter().enumerate() {
        // Unlike RETURN (which auto-names an unaliased expression), every
        // WITH item that isn't a bare variable reference needs an
        // explicit `AS alias`. A bare `Var` needs none since `WITH a`
        // already carries `a` forward as itself.
        if item.alias.is_none() && !matches!(item.expr, ReturnExpr::Var(_)) {
            return Err(semantic(
                "WITH requires an alias (AS ...) for every item except a bare variable reference",
            ));
        }
        let kind = infer_expr(&item.expr, input)?;
        let name = item_output_name(index, item);
        if projected.insert(name.clone(), kind).is_some() {
            return Err(semantic(format!(
                "WITH projects duplicate variable '{name}'"
            )));
        }
    }
    if let Some(expr) = &with.where_clause {
        // `WITH x AS y WHERE ...` sees both the pre-WITH binding (`x`) and
        // the new alias (`y`) -- matches `executor::materialize_with`'s
        // merged-row evaluation. Only aggregation collapses rows here, not
        // `DISTINCT`: WHERE runs before DISTINCT's dedup, so every row it
        // sees still has its own unambiguous pre-WITH binding.
        if crate::executor::has_aggregate(&with.items) {
            validate_with_expr(expr, &projected)?;
        } else {
            let mut merged = input.clone();
            merged.extend(projected.iter().map(|(k, v)| (k.clone(), v.clone())));
            validate_with_expr(expr, &merged)?;
        }
    }
    if let Some(order_by) = &with.order_by {
        // Same InvalidAggregation rule as RETURN's own ORDER BY above.
        let with_aggregates = crate::executor::has_aggregate(&with.items);
        // A non-aggregating, non-DISTINCT WITH's ORDER BY sees both the
        // pre-WITH scope and the new aliases, not just the projected names
        // (`WITH a.count AS count ORDER BY a.count` -- `a` isn't projected
        // but is still a valid sort key). Aggregation/DISTINCT both
        // collapse many pre-WITH rows into one output row, so there's no
        // single pre-WITH scope left to fall back to there.
        let order_scope = if with_aggregates || with.distinct {
            projected.clone()
        } else {
            let mut merged = input.clone();
            merged.extend(projected.iter().map(|(k, v)| (k.clone(), v.clone())));
            merged
        };
        for (expr, _) in order_by {
            // Repeating a WITH item's expression or own alias verbatim
            // (see the matching RETURN-side check) applies to DISTINCT
            // too, not just aggregation: both collapse many pre-WITH rows
            // into one output row, narrowing `order_scope` to `projected`
            // only, so a DISTINCT-only WITH's ORDER BY needs the same
            // escape hatch to see its own item's alias.
            if (with_aggregates || with.distinct)
                && with
                    .items
                    .iter()
                    .enumerate()
                    .any(|(i, item)| crate::executor::item_matches_leaf(expr, i, item))
            {
                continue;
            }
            if with_aggregates {
                // Not a verbatim match -- may still be a composed
                // expression `resolve_grouped_rows`/`rewrite_composed_item`
                // (executor.rs) can evaluate the same way a composed WITH
                // item would. A DISTINCT-only (non-aggregating) WITH has no
                // such per-group evaluator, so that case falls through to
                // `infer_expr` below against its `projected`-only scope.
                crate::executor::validate_order_by_composed_expr(expr, &with.items)?;
                continue;
            }
            if crate::executor::contains_aggregate(expr) {
                return Err(semantic(
                    "ORDER BY cannot use an aggregate function unless WITH itself is \
                     aggregating",
                ));
            }
            infer_expr(expr, &order_scope)?;
        }
    }
    Ok(projected)
}

fn validate_tail(
    tail: &Option<Tail>,
    scope: &mut Scope,
    allow_mutation: bool,
) -> Result<Scope, QueryError> {
    let Some(tail) = tail else {
        return Ok(Scope::new());
    };
    let reject_mutation = |clause_name: &str| -> Result<(), QueryError> {
        if allow_mutation {
            Ok(())
        } else {
            Err(semantic(format!(
                "exists {{}} can't contain an updating clause ({clause_name}) -- only reading \
                 clauses (MATCH/UNWIND/WITH) are allowed inside it"
            )))
        }
    };
    match tail {
        Tail::Return(items, _) => project_return(items, scope),
        Tail::ReturnStar(_) => {
            let items = crate::executor::return_star_items(scope.keys().cloned())?;
            project_return(&items, scope)
        }
        Tail::Delete(exprs, ret) | Tail::DetachDelete(exprs, ret) => {
            reject_mutation("DELETE")?;
            for expr in exprs {
                validate_delete_target(expr, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Set(items, ret) => {
            reject_mutation("SET")?;
            for item in items {
                validate_set_item(item, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Remove(items, ret) => {
            reject_mutation("REMOVE")?;
            for item in items {
                validate_remove_item(item, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Create(patterns, ret) => {
            reject_mutation("CREATE")?;
            for pattern in patterns {
                bind_create_pattern(pattern, scope)?;
            }
            validate_return_tail(ret, scope)
        }
    }
}

fn validate_return_tail(ret: &Option<ReturnTail>, scope: &Scope) -> Result<Scope, QueryError> {
    match ret {
        Some(ret) => project_return(&ret.items, scope),
        None => Ok(Scope::new()),
    }
}

fn project_return(items: &[ReturnItem], scope: &Scope) -> Result<Scope, QueryError> {
    crate::executor::validate_return_items(items)?;
    let mut projected = Scope::new();
    for (index, item) in items.iter().enumerate() {
        let name = item_output_name(index, item);
        let kind = infer_expr(&item.expr, scope)?;
        // Only a real name collision -- an explicit alias reused, or a
        // bare variable/property-access name repeated -- is a genuine
        // conflict. An *unaliased* function call/`count(*)` falls back
        // to a generic placeholder name (`"date(...)"`,`"count(*)"`,
        // not argument-aware -- see `default_output_name`), so two
        // different unaliased calls to the same function legitimately
        // collide there without being a real duplicate (real Cypher
        // auto-names each by its full source text instead, which
        // MarsDB's AST-only naming can't reproduce) -- skip the check
        // for that specific case rather than reject valid queries.
        let name_is_real =
            item.alias.is_some() || matches!(item.expr, ReturnExpr::Var(_) | ReturnExpr::Prop(_));
        let existing = projected.insert(name.clone(), kind);
        if name_is_real && existing.is_some() {
            return Err(semantic(format!(
                "RETURN projects duplicate column name '{name}'"
            )));
        }
    }
    Ok(projected)
}

/// Shared by `Tail::Delete`/`Tail::DetachDelete` and `QueryClause::Delete`
/// (the `DELETE ... WITH ...` mid-statement form) -- same target-kind rules
/// either way.
fn validate_delete_target(expr: &ReturnExpr, scope: &Scope) -> Result<(), QueryError> {
    // Some shapes can never evaluate to a node/relationship/path,
    // regardless of runtime data, so they're rejected here rather than
    // only once a row reaches `delete_value` (a zero-row MATCH would
    // otherwise skip the check). `null` is exempt: deleting it is a
    // documented no-op, not a type error.
    if !matches!(expr, ReturnExpr::Lit(Literal::Null))
        && matches!(
            expr,
            ReturnExpr::Lit(_)
                | ReturnExpr::CountStar
                | ReturnExpr::Arith(..)
                | ReturnExpr::Neg(..)
                | ReturnExpr::And(..)
                | ReturnExpr::Or(..)
                | ReturnExpr::Xor(..)
                | ReturnExpr::Not(..)
                | ReturnExpr::Compare(..)
                | ReturnExpr::IsNull(..)
                | ReturnExpr::In(..)
                | ReturnExpr::MapLit(..)
                | ReturnExpr::ListLit(..)
                | ReturnExpr::HasLabel(..)
        )
    {
        return Err(semantic(
            "DELETE target must evaluate to a node, relationship, or path -- a \
             literal/arithmetic/boolean/map/list expression never can",
        ));
    }
    let kind = infer_expr(expr, scope)?;
    // `Scalar` isn't rejected, same reasoning as `bind_unwind`'s: a
    // map/list access (`nodes.key`, `friends[0]`) types as `Scalar` even
    // when it holds a Node/Edge/Path at runtime. Only a confidently-wrong
    // kind is rejected here; everything else defers to the runtime
    // `QueryError::Type` in `delete_value`.
    if !matches!(
        kind,
        Kind::Node | Kind::Edge | Kind::Path | Kind::Unknown | Kind::Scalar
    ) {
        return Err(semantic(format!(
            "DELETE target is {}, not a node, relationship, or path",
            kind_name(&kind)
        )));
    }
    Ok(())
}

fn validate_set_item(item: &SetItem, scope: &Scope) -> Result<(), QueryError> {
    match item {
        SetItem::Prop(access, value) => {
            require_graph(scope, &access.var, "SET property target")?;
            infer_expr(value, scope)?;
            Ok(())
        }
        SetItem::Labels(var, _) => require_kind(scope, var, &Kind::Node, "SET label target"),
        SetItem::MapAssign { var, value, .. } => {
            require_graph(scope, var, "SET map-assignment target")?;
            infer_expr(value, scope)?;
            Ok(())
        }
    }
}

fn validate_remove_item(item: &RemoveItem, scope: &Scope) -> Result<(), QueryError> {
    match item {
        RemoveItem::Prop(access) => require_graph(scope, &access.var, "REMOVE property target"),
        RemoveItem::Labels(var, _) => require_kind(scope, var, &Kind::Node, "REMOVE label target"),
    }
}

/// An aggregate function is never legal inside a pattern-level `WHERE`:
/// aggregates only make sense as a RETURN/WITH item's top-level
/// expression, evaluated once after every row is matched and filtered,
/// while a WHERE predicate runs per-row before any such collapsing
/// exists. `infer_expr` itself stays permissive since it's shared with
/// RETURN/WITH items, where an aggregate is legal -- this is the
/// WHERE-specific half of the check.
fn reject_aggregate_in_where(expr: &ReturnExpr) -> Result<(), QueryError> {
    if crate::executor::contains_aggregate(expr) {
        return Err(semantic(
            "an aggregate function can't be used inside a WHERE clause",
        ));
    }
    Ok(())
}

fn validate_pattern_expr(expr: &Expr, scope: &Scope) -> Result<(), QueryError> {
    match expr {
        Expr::And(left, right) | Expr::Or(left, right) => {
            validate_pattern_expr(left, scope)?;
            validate_pattern_expr(right, scope)
        }
        Expr::Not(inner) => validate_pattern_expr(inner, scope),
        Expr::Compare(access, _, _) | Expr::IsNull(access) => {
            require_property_owner(scope, &access.var)
        }
        Expr::PropCompare(left, _, right) => {
            require_property_owner(scope, &left.var)?;
            require_property_owner(scope, &right.var)
        }
        Expr::HasLabel(var, _) => require_kind(scope, var, &Kind::Node, "label predicate"),
        Expr::VarEq(left, right) => {
            require_graph(scope, left, "identity predicate")?;
            require_graph(scope, right, "identity predicate")
        }
        Expr::GeneralCompare(left, _, right) => {
            infer_expr(left, scope)?;
            infer_expr(right, scope)?;
            reject_aggregate_in_where(left)?;
            reject_aggregate_in_where(right)
        }
        Expr::GeneralIsNull(e) => {
            infer_expr(e, scope)?;
            reject_aggregate_in_where(e)
        }
        Expr::GeneralBare(e) => {
            let kind = infer_expr(e, scope)?;
            require_boolean_predicate_kind(&kind, "WHERE predicate")?;
            reject_aggregate_in_where(e)
        }
        Expr::Pattern(pattern) => validate_pattern_predicate(pattern, scope),
        // Unlike `Pattern` above (existential-only, never introduces a
        // variable), `exists {}`'s pattern can introduce new
        // node/relationship variables, so it reuses `bind_match_pattern`
        // against a scoped copy -- these bindings are local to the
        // `exists {}` block and don't leak into the enclosing scope.
        Expr::Exists {
            pattern,
            where_clause,
        } => {
            let mut inner_scope = scope.clone();
            bind_match_pattern(pattern, &mut inner_scope)?;
            if let Some(w) = where_clause.as_deref() {
                validate_pattern_expr(w, &inner_scope)?;
            }
            Ok(())
        }
        // `exists { MATCH ... RETURN ... }` -- correlated against the
        // enclosing scope, same reasoning as `Exists` above, reusing
        // `validate_match_clauses` with `allow_mutation: false`. Only
        // `Statement::Match` is a valid shape here (never a bare CREATE
        // or UNION).
        Expr::ExistsSubquery(stmt) => {
            let Statement::Match {
                clauses,
                tail,
                order_by,
                ..
            } = stmt.as_ref()
            else {
                return Err(semantic(
                    "exists {} subquery must be a MATCH ... RETURN ... statement",
                ));
            };
            validate_match_clauses(clauses, tail, order_by, scope.clone(), false)
        }
        // Never reaches here: synthesized by the planner (`build_match_
        // plan`) after this pass already validated the original parsed
        // AST. No surface syntax constructs this directly.
        Expr::EdgeNotInSet { .. } => {
            unreachable!("Expr::EdgeNotInSet is only ever synthesized by the planner")
        }
    }
}

/// `WHERE (n)-[r:REL]->(m)` etc -- every named endpoint must already be
/// bound; unlike `bind_match_pattern` (a real MATCH pattern, which
/// introduces new variables), a pattern predicate never does. `require_kind`'s
/// `lookup` already produces an "undefined variable" error for an unbound
/// name. An anonymous (var-less) token is always fine.
fn validate_pattern_predicate(pattern: &Pattern, scope: &Scope) -> Result<(), QueryError> {
    if let Some(var) = &pattern.start.var {
        require_kind(scope, var, &Kind::Node, "pattern predicate node")?;
    }
    validate_props(&pattern.start.props, scope)?;
    for (rel, node) in &pattern.hops {
        if let Some(var) = &rel.var {
            require_kind(scope, var, &Kind::Edge, "pattern predicate relationship")?;
        }
        validate_props(&rel.props, scope)?;
        if let Some(var) = &node.var {
            require_kind(scope, var, &Kind::Node, "pattern predicate node")?;
        }
        validate_props(&node.props, scope)?;
    }
    Ok(())
}

fn validate_with_expr(expr: &WithExpr, scope: &Scope) -> Result<(), QueryError> {
    match expr {
        WithExpr::And(left, right) | WithExpr::Or(left, right) => {
            validate_with_expr(left, scope)?;
            validate_with_expr(right, scope)
        }
        WithExpr::Not(inner) => validate_with_expr(inner, scope),
        WithExpr::Compare(left, _, right) => {
            infer_expr(left, scope)?;
            infer_expr(right, scope)?;
            Ok(())
        }
        WithExpr::IsNull(e) => {
            infer_expr(e, scope)?;
            Ok(())
        }
        // `WithExpr` has no `Expr::Pattern`-equivalent folding, so a bare
        // pattern predicate reaches here as `ReturnExpr::PatternPredicate`
        // inside `Bare` -- validated the same way ordinary MATCH's own
        // WHERE validates one, not `infer_expr`'s generic (rejecting)
        // handling. Matches `executor::eval_with_expr`'s special case.
        WithExpr::Bare(ReturnExpr::PatternPredicate(pattern)) => {
            validate_pattern_predicate(pattern, scope)
        }
        WithExpr::Bare(e) => {
            let kind = infer_expr(e, scope)?;
            require_boolean_predicate_kind(&kind, "WHERE predicate")
        }
    }
}

/// `(min, max)` argument count for a built-in function name, case-
/// insensitively matched. `max: None` means unbounded (`coalesce` only).
/// `None` for an unrecognized name -- `infer_expr`'s "unknown function"
/// error covers that case; this only narrows an already-known function.
///
/// Checked once at compile time, before any per-argument work, since
/// argument count is knowable from the call's AST shape alone.
fn function_arity(name: &str) -> Option<(usize, Option<usize>)> {
    Some(match name.to_ascii_lowercase().as_str() {
        "count" | "sum" | "avg" | "min" | "max" | "collect" => (1, Some(1)),
        "percentilecont" | "percentiledisc" => (2, Some(2)),
        "coalesce" => (1, None),
        "tointeger" | "tostring" | "tofloat" | "toboolean" => (1, Some(1)),
        "date" | "localtime" | "time" | "localdatetime" | "datetime" => (0, Some(1)),
        // 0 args ordinarily, but also accepts exactly 1: a `null` argument
        // propagates `null` rather than erroring (`now_or_null`
        // implements this). `rand()` has no such exception -- always 0 args.
        "date.transaction"
        | "date.statement"
        | "date.realtime"
        | "localtime.transaction"
        | "localtime.statement"
        | "localtime.realtime"
        | "time.transaction"
        | "time.statement"
        | "time.realtime"
        | "localdatetime.transaction"
        | "localdatetime.statement"
        | "localdatetime.realtime"
        | "datetime.transaction"
        | "datetime.statement"
        | "datetime.realtime" => (0, Some(1)),
        "rand" => (0, Some(0)),
        "duration" => (1, Some(1)),
        "datetime.fromepoch" => (2, Some(2)),
        "datetime.fromepochmillis" => (1, Some(1)),
        "duration.between" | "duration.inmonths" | "duration.indays" | "duration.inseconds" => {
            (2, Some(2))
        }
        "date.truncate"
        | "localtime.truncate"
        | "time.truncate"
        | "localdatetime.truncate"
        | "datetime.truncate" => (2, Some(3)),
        "length" | "nodes" | "relationships" | "type" | "startnode" | "endnode" | "keys"
        | "labels" | "properties" | "id" | "size" | "exists" | "head" | "last" | "tail"
        | "toupper" | "upper" | "tolower" | "lower" | "trim" | "ltrim" | "rtrim" | "reverse"
        | "abs" | "ceil" | "floor" | "round" | "sqrt" | "sign" => (1, Some(1)),
        "range" => (2, Some(3)),
        "split" | "left" | "right" => (2, Some(2)),
        "substring" => (2, Some(3)),
        "replace" => (3, Some(3)),
        _ => return None,
    })
}

fn check_arity(name: &str, arg_count: usize) -> Result<(), QueryError> {
    let Some((min, max)) = function_arity(name) else {
        return Ok(());
    };
    let ok = arg_count >= min && max.is_none_or(|max| arg_count <= max);
    if ok {
        return Ok(());
    }
    let arg_word = |n: usize| if n == 1 { "argument" } else { "arguments" };
    let expected = match max {
        Some(max) if max == min => format!("exactly {min} {}", arg_word(min)),
        Some(max) => format!("{min} to {max} arguments"),
        None => format!("at least {min} {}", arg_word(min)),
    };
    Err(semantic(format!(
        "{name}() expects {expected}, got {arg_count}"
    )))
}

fn infer_expr(expr: &ReturnExpr, scope: &Scope) -> Result<Kind, QueryError> {
    Ok(match expr {
        ReturnExpr::Var(var) => lookup(scope, var, "expression")?.clone(),
        ReturnExpr::Prop(access) => {
            require_property_owner(scope, &access.var)?;
            Kind::Scalar
        }
        // `<expr>.prop` where `<expr>` isn't a bare variable -- the real
        // type check is a runtime one (`executor::property_of_value`);
        // this only checks the base expression is well-formed.
        ReturnExpr::PropOf(base, _) => {
            infer_expr(base, scope)?;
            Kind::Scalar
        }
        // `null` types as `Unknown`, not `Scalar`: it's compatible with
        // any type, and every check here already treats `Unknown` as
        // compatible with everything, avoiding per-site "Scalar tolerated
        // too" exceptions. A non-null scalar still types as `Scalar`.
        ReturnExpr::Lit(Literal::Null) => Kind::Unknown,
        ReturnExpr::Lit(_) | ReturnExpr::CountStar => Kind::Scalar,
        ReturnExpr::Call { name, args, .. } => {
            check_arity(name, args.len())?;
            let arg_kinds = args
                .iter()
                .map(|arg| infer_expr(arg, scope))
                .collect::<Result<Vec<_>, _>>()?;
            if is_aggregate_name(name) {
                if name.eq_ignore_ascii_case("collect") {
                    Kind::List(Box::new(
                        arg_kinds.first().cloned().unwrap_or(Kind::Unknown),
                    ))
                } else {
                    Kind::Scalar
                }
            } else {
                match name.to_ascii_lowercase().as_str() {
                    "coalesce" => unify_many(&arg_kinds),
                    "tointeger"
                    | "tostring"
                    | "tofloat"
                    | "toboolean"
                    | "date"
                    | "duration"
                    | "localtime"
                    | "time"
                    | "localdatetime"
                    | "datetime"
                    | "duration.between"
                    | "duration.inmonths"
                    | "duration.indays"
                    | "duration.inseconds"
                    | "date.truncate"
                    | "localtime.truncate"
                    | "time.truncate"
                    | "localdatetime.truncate"
                    | "datetime.truncate"
                    | "date.transaction"
                    | "date.statement"
                    | "date.realtime"
                    | "localtime.transaction"
                    | "localtime.statement"
                    | "localtime.realtime"
                    | "time.transaction"
                    | "time.statement"
                    | "time.realtime"
                    | "localdatetime.transaction"
                    | "localdatetime.statement"
                    | "localdatetime.realtime"
                    | "datetime.transaction"
                    | "datetime.statement"
                    | "datetime.realtime"
                    | "datetime.fromepoch"
                    | "datetime.fromepochmillis" => Kind::Scalar,
                    "length" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_path_or_null(kind, "length() argument")?;
                        }
                        Kind::Scalar
                    }
                    "nodes" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_path_or_null(kind, "nodes() argument")?;
                        }
                        Kind::List(Box::new(Kind::Node))
                    }
                    "relationships" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_path_or_null(kind, "relationships() argument")?;
                        }
                        Kind::List(Box::new(Kind::Edge))
                    }
                    // Unlike `keys`/`labels`/`id`/`size`/`exists` (each
                    // polymorphic over several kinds, left to the runtime's
                    // `QueryError::Type`), `type()` only accepts a
                    // relationship -- checked here so it's a compile-time
                    // error even when the MATCH matches zero rows.
                    "type" => {
                        // `Scalar` tolerated too: a `null`-valued argument
                        // types as `Scalar`, and `type(null)` is `null` at
                        // runtime, not an error.
                        if let Some(kind) = arg_kinds.first() {
                            if !matches!(kind, Kind::Edge | Kind::Scalar | Kind::Unknown) {
                                return Err(semantic(format!(
                                    "type() argument requires a relationship, but found {}",
                                    kind_name(kind)
                                )));
                            }
                        }
                        Kind::Scalar
                    }
                    // Same reasoning as `type()` above -- both only accept
                    // a relationship, and return the node at its start/end.
                    "startnode" | "endnode" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(
                                kind,
                                &Kind::Edge,
                                "startNode()/endNode() argument",
                            )?;
                        }
                        Kind::Node
                    }
                    // `keys`/`labels`/`properties`/`id`/`size`/`exists`
                    // accept several kinds depending on the function --
                    // left to `executor::call_builtin`'s own runtime
                    // `QueryError::Type`, no additional check needed here.
                    // `keys`/`labels` return a list of strings, not a
                    // scalar: `Kind::List` so `[x IN labels(n) | ...]`'s
                    // source-kind check (`list_element`) doesn't wrongly
                    // reject a valid list comprehension source.
                    "keys" | "labels" => Kind::List(Box::new(Kind::Scalar)),
                    // Unlike `id`/`exists`, `size()` never accepts a
                    // `Path` -- knowable here without running a row, so
                    // checked at compile time rather than left to a
                    // zero-row MATCH silently skipping it.
                    "size" => {
                        if let Some(Kind::Path) = arg_kinds.first() {
                            return Err(semantic(
                                "size() doesn't accept a path -- use length() instead",
                            ));
                        }
                        Kind::Scalar
                    }
                    "id" | "exists" => Kind::Scalar,
                    "properties" => Kind::Map,
                    "head" | "last" => match arg_kinds.first() {
                        Some(Kind::List(inner)) => (**inner).clone(),
                        _ => Kind::Unknown,
                    },
                    "tail" => match arg_kinds.first() {
                        Some(kind @ Kind::List(_)) => kind.clone(),
                        _ => Kind::Unknown,
                    },
                    "range" | "split" => Kind::List(Box::new(Kind::Scalar)),
                    "toupper" | "upper" | "tolower" | "lower" | "trim" | "ltrim" | "rtrim"
                    | "replace" | "substring" | "left" | "right" | "abs" | "ceil" | "floor"
                    | "round" | "sqrt" | "sign" | "rand" => Kind::Scalar,
                    // Polymorphic over string/list -- the input's own kind
                    // (if known) is the output's kind too.
                    "reverse" => arg_kinds.first().cloned().unwrap_or(Kind::Unknown),
                    other => return Err(semantic(format!("unknown function '{other}'"))),
                }
            }
        }
        ReturnExpr::Case { test, whens, else_ } => {
            if let Some(test) = test {
                infer_expr(test, scope)?;
            }
            let mut result_kinds = Vec::new();
            for (when, then) in whens {
                infer_expr(when, scope)?;
                result_kinds.push(infer_expr(then, scope)?);
            }
            if let Some(else_) = else_ {
                result_kinds.push(infer_expr(else_, scope)?);
            }
            unify_many(&result_kinds)
        }
        ReturnExpr::Arith(left, op, right) => {
            let lk = infer_expr(left, scope)?;
            let rk = infer_expr(right, scope)?;
            // `+` also means list concatenation/append/prepend (`[1,2] +
            // [3]`, `[1,2] + 3`, `3 + [1,2]`); `-`/`*`/`/`/`%` have no
            // list meaning, so those reject one via `require_scalarish`.
            // The element kind unifies whichever side is a list with the
            // other operand's kind rather than hardcoding `Scalar`, which
            // would wrongly forget a concatenated node/relationship list's
            // real element kind (`[a] + collect(n) + [b]` must type as
            // `List(Node)`, not `List(Scalar)`).
            if *op == ArithOp::Add && (matches!(lk, Kind::List(_)) || matches!(rk, Kind::List(_))) {
                let elem = |k: Kind| match k {
                    Kind::List(inner) => *inner,
                    other => other,
                };
                Kind::List(Box::new(unify_many(&[elem(lk), elem(rk)])))
            } else {
                require_scalarish(&lk, "arithmetic operand")?;
                require_scalarish(&rk, "arithmetic operand")?;
                Kind::Scalar
            }
        }
        ReturnExpr::Neg(e) => {
            let k = infer_expr(e, scope)?;
            require_scalarish(&k, "unary minus operand")?;
            Kind::Scalar
        }
        ReturnExpr::ListLit(items) => {
            let kinds = items
                .iter()
                .map(|item| infer_expr(item, scope))
                .collect::<Result<Vec<_>, _>>()?;
            Kind::List(Box::new(unify_many(&kinds)))
        }
        ReturnExpr::Index(base, index) => {
            require_scalarish(&infer_expr(index, scope)?, "list index")?;
            match infer_expr(base, scope)? {
                Kind::List(element) => *element,
                // `map['key']` -- dynamic map-field access. The result
                // could be any value the map holds at that key, so
                // `Kind::Scalar`, the same imprecise fallback `keys`/
                // `labels`/etc use elsewhere.
                Kind::Map => Kind::Scalar,
                // `Scalar` tolerated too: a `null`-valued base types as
                // `Scalar`, and indexing into `null` is `null` at runtime,
                // not an error. A genuinely wrong scalar still gets
                // `apply_index`'s real `QueryError::Type` at runtime.
                Kind::Unknown | Kind::Scalar => Kind::Unknown,
                // `n['name']` -- dynamic property access on a node/
                // relationship, same as `n.name`'s static form;
                // `apply_index` supports this via `property_of_value`.
                Kind::Node | Kind::Edge => Kind::Scalar,
                other => {
                    return Err(semantic(format!(
                        "index base is {}, not a list or map",
                        kind_name(&other)
                    )))
                }
            }
        }
        ReturnExpr::Slice(base, start, end) => {
            if let Some(start) = start {
                require_scalarish(&infer_expr(start, scope)?, "slice bound")?;
            }
            if let Some(end) = end {
                require_scalarish(&infer_expr(end, scope)?, "slice bound")?;
            }
            match infer_expr(base, scope)? {
                list @ Kind::List(_) => list,
                Kind::Unknown => Kind::List(Box::new(Kind::Unknown)),
                other => {
                    return Err(semantic(format!(
                        "slice base is {}, not a list",
                        kind_name(&other)
                    )))
                }
            }
        }
        ReturnExpr::ListComp {
            var,
            source,
            where_clause,
            project,
        } => {
            let element = list_element(infer_expr(source, scope)?, "list comprehension source")?;
            let mut local = scope.clone();
            local.insert(var.clone(), element.clone());
            if let Some(where_clause) = where_clause {
                require_scalarish(&infer_expr(where_clause, &local)?, "list filter")?;
            }
            let projected = match project {
                Some(project) => infer_expr(project, &local)?,
                None => element,
            };
            Kind::List(Box::new(projected))
        }
        ReturnExpr::Quantifier {
            var,
            source,
            where_clause,
            ..
        } => {
            let element = list_element(infer_expr(source, scope)?, "quantifier source")?;
            let mut local = scope.clone();
            local.insert(var.clone(), element);
            if let Some(where_clause) = where_clause {
                require_scalarish(&infer_expr(where_clause, &local)?, "quantifier predicate")?;
            }
            Kind::Scalar
        }
        ReturnExpr::MapLit(entries) => {
            for (_, value) in entries {
                infer_expr(value, scope)?;
            }
            Kind::Map
        }
        ReturnExpr::And(left, right)
        | ReturnExpr::Or(left, right)
        | ReturnExpr::Xor(left, right) => {
            require_scalarish(&infer_expr(left, scope)?, "boolean operand")?;
            require_scalarish(&infer_expr(right, scope)?, "boolean operand")?;
            Kind::Scalar
        }
        ReturnExpr::Not(inner) => {
            require_scalarish(&infer_expr(inner, scope)?, "boolean operand")?;
            Kind::Scalar
        }
        ReturnExpr::Compare(left, _, right) => {
            infer_expr(left, scope)?;
            infer_expr(right, scope)?;
            Kind::Scalar
        }
        ReturnExpr::IsNull(inner) => {
            infer_expr(inner, scope)?;
            Kind::Scalar
        }
        ReturnExpr::In(needle, haystack) => {
            infer_expr(needle, scope)?;
            infer_expr(haystack, scope)?;
            Kind::Scalar
        }
        ReturnExpr::HasLabel(var, _) => {
            require_graph(scope, var, "(n:Label) target")?;
            Kind::Scalar
        }
        // Real validation happens via `validate_pattern_predicate` once
        // `return_expr_to_expr` folds this into `Expr::Pattern`. Reaching
        // `infer_expr` at all means it's in a position that folding never
        // runs (RETURN/WITH item, function arg, ...), a compile-time
        // error rather than a runtime rejection that a zero-row MATCH
        // could silently skip.
        ReturnExpr::PatternPredicate(_) => {
            return Err(QueryError::Semantic(
                "a pattern predicate (`(n)-->()` etc) can only be used inside WHERE".into(),
            ))
        }
        // Unlike `PatternPredicate` (existential-only, never introduces a
        // variable), a pattern comprehension can introduce new
        // node/relationship variables, so it reuses `bind_match_pattern`
        // against a scoped copy -- these bindings are local to the
        // projection and don't leak into the enclosing RETURN/WITH scope.
        ReturnExpr::PatternComprehension {
            path_var,
            pattern,
            where_clause,
            projection,
        } => {
            if path_var.is_some() {
                crate::parse_helpers::validate_named_path_pattern(pattern)?;
            }
            let mut inner_scope = scope.clone();
            bind_match_pattern(pattern, &mut inner_scope)?;
            if let Some(path_var) = path_var {
                bind_kind(&mut inner_scope, path_var, Kind::Path, "path variable")?;
            }
            if let Some(where_expr) = where_clause {
                validate_pattern_expr(where_expr, &inner_scope)?;
            }
            Kind::List(Box::new(infer_expr(projection, &inner_scope)?))
        }
        ReturnExpr::ExistsPattern { .. } | ReturnExpr::ExistsSubquery(_) => {
            return Err(QueryError::Semantic(
                "an exists {} subquery can only be used inside WHERE".into(),
            ))
        }
    })
}

fn list_element(kind: Kind, context: &str) -> Result<Kind, QueryError> {
    match kind {
        Kind::List(element) => Ok(*element),
        // `Scalar` isn't rejected, same reasoning as `bind_unwind`'s
        // widening: a property access (`n.numbers`) types as `Kind::
        // Scalar` even when it holds a `List` at runtime (list-valued
        // properties are supported). Only a confidently-wrong kind is
        // rejected here; everything else defers to the runtime `Value::
        // List` check in `eval_return_expr`'s `ListComp`/`Quantifier` arms.
        Kind::Unknown | Kind::Scalar => Ok(Kind::Unknown),
        other => Err(semantic(format!(
            "{context} is {}, not a list",
            kind_name(&other)
        ))),
    }
}

fn bind_kind(
    scope: &mut Scope,
    var: &str,
    expected: Kind,
    context: &str,
) -> Result<(), QueryError> {
    match scope.get(var) {
        Some(actual) => require_compatible_kind(actual, &expected, context),
        None => {
            scope.insert(var.to_string(), expected);
            Ok(())
        }
    }
}

fn require_kind(
    scope: &Scope,
    var: &str,
    expected: &Kind,
    context: &str,
) -> Result<(), QueryError> {
    let actual = lookup(scope, var, context)?;
    require_compatible_kind(actual, expected, context)
}

fn require_compatible_kind(
    actual: &Kind,
    expected: &Kind,
    context: &str,
) -> Result<(), QueryError> {
    if actual == expected || matches!(actual, Kind::Unknown) {
        return Ok(());
    }
    Err(semantic(format!(
        "{context} requires {}, but found {}",
        kind_name(expected),
        kind_name(actual)
    )))
}

/// `length()`/`nodes()`/`relationships()`'s shared argument check --
/// `Kind::Path`, or `Scalar` (a `null` argument types as `Scalar` and all
/// three return `null` at runtime for it, not an error), or `Unknown`.
fn require_path_or_null(actual: &Kind, context: &str) -> Result<(), QueryError> {
    if matches!(actual, Kind::Path | Kind::Scalar | Kind::Unknown) {
        return Ok(());
    }
    Err(semantic(format!(
        "{context} requires {}, but found {}",
        kind_name(&Kind::Path),
        kind_name(actual)
    )))
}

fn require_graph(scope: &Scope, var: &str, context: &str) -> Result<(), QueryError> {
    let actual = lookup(scope, var, context)?;
    if matches!(actual, Kind::Node | Kind::Edge | Kind::Unknown) {
        Ok(())
    } else {
        Err(semantic(format!(
            "{context} '{var}' is {}, not a node or relationship",
            kind_name(actual)
        )))
    }
}

fn require_property_owner(scope: &Scope, var: &str) -> Result<(), QueryError> {
    // Scalars deliberately remain valid: Date/Duration expose component
    // fields, and null/other scalars yield null for a missing component in
    // the current runtime semantics. The binder resolves the name here;
    // the exact property/component remains data-dependent.
    let kind = lookup(scope, var, "property access")?;
    // `Path` is the one kind that's never valid here, knowable without
    // running a row, so checked at compile time rather than left to a
    // zero-row MATCH silently skipping it.
    if matches!(kind, Kind::Path) {
        return Err(semantic(format!(
            "'{var}' is a path — property access requires a node, relationship, or map"
        )));
    }
    Ok(())
}

fn require_scalarish(kind: &Kind, context: &str) -> Result<(), QueryError> {
    if matches!(kind, Kind::Scalar | Kind::Unknown) {
        Ok(())
    } else {
        Err(semantic(format!(
            "{context} cannot use {}",
            kind_name(kind)
        )))
    }
}

fn lookup<'a>(scope: &'a Scope, var: &str, context: &str) -> Result<&'a Kind, QueryError> {
    scope
        .get(var)
        .ok_or_else(|| semantic(format!("{context} references undefined variable '{var}'")))
}

fn unify_many(kinds: &[Kind]) -> Kind {
    let Some(first) = kinds.first() else {
        return Kind::Unknown;
    };
    if kinds.iter().all(|kind| kind == first) {
        first.clone()
    } else {
        Kind::Unknown
    }
}

fn item_output_name(index: usize, item: &ReturnItem) -> String {
    item.alias
        .clone()
        .unwrap_or_else(|| default_output_name(&item.expr, index))
}

fn default_output_name(expr: &ReturnExpr, index: usize) -> String {
    match expr {
        ReturnExpr::Var(var) => var.clone(),
        ReturnExpr::Prop(access) => format!("{}.{}", access.var, access.prop),
        ReturnExpr::Call { name, .. } => format!("{name}(...)"),
        ReturnExpr::CountStar => "count(*)".to_string(),
        ReturnExpr::Case { .. } => format!("case{index}"),
        _ => format!("col{index}"),
    }
}

fn kind_name(kind: &Kind) -> &'static str {
    match kind {
        Kind::Node => "a node",
        Kind::Edge => "a relationship",
        Kind::Scalar => "a scalar",
        Kind::List(_) => "a list",
        Kind::Map => "a map",
        Kind::Path => "a path",
        Kind::Unknown => "a dynamically typed value",
    }
}

fn semantic(message: impl Into<String>) -> QueryError {
    QueryError::Semantic(message.into())
}

/// `WHERE (n)` / `WHERE (n)-->()`-shaped bare-expression predicates
/// (`Expr::GeneralBare`/`WithExpr::Bare`) -- a node/relationship/list/map/
/// path can never be a valid boolean predicate regardless of runtime data
/// (`MATCH (n) WHERE (n) RETURN n`'s `(n)` is a bare node reference, not
/// a pattern predicate), so this is checked here rather than left to
/// `value_to_bool3`'s runtime error, which a zero-row MATCH would
/// otherwise never trigger. `Scalar`/`Unknown` both pass -- a `Scalar`
/// could still turn out non-boolean at runtime, a real `value_to_bool3`
/// error.
fn require_boolean_predicate_kind(kind: &Kind, context: &str) -> Result<(), QueryError> {
    match kind {
        Kind::Scalar | Kind::Unknown => Ok(()),
        other => Err(semantic(format!(
            "{context} requires a boolean, but found {}",
            kind_name(other)
        ))),
    }
}
