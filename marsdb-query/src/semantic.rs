//! Statement-level name binding and structural type validation.
//!
//! This pass runs after parsing/parameter substitution and before a storage
//! transaction is opened. It deliberately validates only types knowable from
//! query structure (node, relationship, list, map, path, scalar); property
//! value types remain data-dependent runtime checks.

use std::collections::HashMap;

use crate::ast::{
    is_aggregate_name, ArithOp, Expr, Literal, MergeClause, NodePattern, Pattern, QueryClause,
    RemoveItem, ReturnExpr, ReturnItem, ReturnTail, SetItem, Statement, Tail, UnwindClause,
    WithClause, WithExpr,
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
        // Each part is independently scoped (no bindings shared across a
        // UNION boundary), so each just gets its own ordinary validation
        // pass -- the one UNION-specific check (every part's columns must
        // match) needs each part's real, evaluated `QueryResult.columns`,
        // which doesn't exist yet at this pre-execution stage, so it lives
        // in `executor::materialize_union` instead.
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
        } => {
            let mut scope = Scope::new();
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
                                .ok_or_else(|| {
                                    semantic("shortestPath() end node must have a variable")
                                })?;
                            require_kind(
                                &prior_scope,
                                start,
                                &Kind::Node,
                                "shortestPath endpoint",
                            )?;
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
                    QueryClause::Merge(clause) => bind_merge(clause, &mut scope)?,
                    QueryClause::With(with) => scope = project_with(with, &scope)?,
                    QueryClause::Set(items) => {
                        for item in items {
                            validate_set_item(item, &scope)?;
                        }
                    }
                    QueryClause::Delete { items, detach: _ } => {
                        for expr in items {
                            validate_delete_target(expr, &scope)?;
                        }
                    }
                    QueryClause::Remove(items) => {
                        for item in items {
                            validate_remove_item(item, &scope)?;
                        }
                    }
                    QueryClause::Create(patterns) => {
                        for pattern in patterns {
                            bind_create_pattern(pattern, &mut scope)?;
                        }
                    }
                }
            }

            let input_scope = scope.clone();
            let output_scope = validate_tail(tail, &mut scope)?;
            if let Some(order_by) = order_by {
                let mut order_scope = input_scope;
                order_scope.extend(output_scope);
                // Real Cypher: an aggregate in RETURN's ORDER BY is only
                // legal when RETURN itself is aggregating (the ORDER BY
                // then runs against the already-collapsed grouped rows,
                // same as its own WITH/RETURN items would) -- otherwise
                // it's a compile-time `InvalidAggregation` error (TCK's
                // ReturnOrderBy2 [14]), not a runtime one.
                let tail_items: Option<&[ReturnItem]> = match tail {
                    Some(Tail::Return(items, _)) => Some(items),
                    _ => None,
                };
                let tail_aggregates = tail_items.is_some_and(crate::executor::has_aggregate);
                for (expr, _) in order_by {
                    if tail_aggregates {
                        // An ORDER BY item that repeats a RETURN item's
                        // expression *or own alias* (`RETURN sum(x) AS s
                        // ORDER BY sum(x)` / `ORDER BY s`, TCK's
                        // WithOrderBy4 [11]/`ReturnOrderBy3`/
                        // `WithSkipLimit1 [2]`) refers to that
                        // already-aggregated item, not a fresh expression
                        // -- its kind is already known from
                        // `output_scope`, and re-running `infer_expr` on
                        // it would need pre-aggregation bindings (like
                        // `x`'s row) that no longer exist post-grouping.
                        // Unlike `validate_composed_expr`'s own *nested*-
                        // leaf check just below, this whole-expression
                        // match doesn't exclude aggregating items -- an
                        // aggregate's own alias referenced *directly* (not
                        // buried inside a larger expression) is exactly
                        // "reuse this item's already-finished value",
                        // which `materialize_aggregating_return_with_
                        // order`'s matching top-level lookup (executor.rs)
                        // handles the same way.
                        if tail_items
                            .unwrap()
                            .iter()
                            .enumerate()
                            .any(|(i, item)| crate::executor::item_matches_leaf(expr, i, item))
                        {
                            continue;
                        }
                        // Not a verbatim match -- may still be a *composed*
                        // expression (an aggregate combined with other
                        // values, or a plain non-aggregate expression
                        // referencing a pre-aggregation variable, TCK's
                        // ReturnOrderBy6) that `resolve_grouped_rows`/
                        // `rewrite_composed_item` (executor.rs) can
                        // evaluate the same way a composed RETURN item
                        // would -- validated the same way, by the same
                        // function, rather than `infer_expr` against a
                        // scope that structurally can't have pre-
                        // aggregation bindings in it anymore.
                        crate::executor::validate_order_by_composed_expr(
                            expr,
                            tail_items.unwrap(),
                        )?;
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
    }
}

fn bind_unwind(clause: &UnwindClause, scope: &mut Scope) -> Result<(), QueryError> {
    let source_kind = infer_expr(&clause.source.0, scope)?;
    let element_kind = match source_kind {
        Kind::List(element) => *element,
        // `Scalar` is deliberately not rejected here -- most function
        // calls (`infer_expr`'s own `Call` arm) type as `Scalar` even
        // when they in fact return a list at runtime (this codebase's
        // `Kind` system doesn't model every builtin's real return shape),
        // so treating it as "unknown, defer to the real runtime
        // Value::List check in eval_unwind" avoids rejecting legitimate
        // queries the semantic layer just can't see through.
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
    // A bare already-bound node with no relationship at all (`MATCH (a)
    // MERGE (a)`) does nothing real -- not searching for or creating
    // anything, just re-stating a var that already exists. A bound start
    // node used as a relationship endpoint (`MATCH (a) MERGE (a)-[:T]->
    // (b)`) stays legitimate -- only checked when there are no hops at
    // all. Checked here (compile time, TCK's Merge1 [15]), not only at
    // runtime -- a zero-row MATCH would otherwise skip this entirely
    // even though real Cypher's `VariableAlreadyBound` is a
    // structural/scope error, not a data-dependent one.
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
    // *create* any node its pattern names, so an already-bound node can't
    // also carry a new label/property predicate (TCK's Merge5 [22]). The
    // hopless-and-predicate-free case just above has its own, more
    // specific message; this covers every other node token, start and hop
    // ends alike.
    check_no_new_predicates_on_bound_node(&pattern.start, scope, "MERGE")?;
    for (_, node) in &pattern.hops {
        check_no_new_predicates_on_bound_node(node, scope, "MERGE")?;
    }
    // Unlike a node endpoint (which can legitimately reference an
    // already-bound node to search/create from), MERGE never reuses an
    // already-bound relationship as its own pattern token -- there's no
    // "search using this specific existing edge" mode (TCK's Merge5
    // [26]).
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
        // *create* this relationship on no-match, and a brand new edge
        // with no type (or more than one -- which one would it get?) is
        // meaningless (TCK's Merge5 [24]).
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
            bind_kind(scope, var, Kind::Edge, "relationship pattern")?;
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
        // Unlike MATCH (where an untyped/multi-typed hop just means "any
        // of these"), CREATE always makes exactly one new relationship,
        // and a brand new edge needs exactly one type -- real Cypher
        // requires a single explicit `:TYPE` here, never inferred,
        // defaulted, or a `|`-alternative list.
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

/// Mirrors `Executor::resolve_or_create_node`'s already-bound rejection
/// at compile time -- a node token naming a variable already in `scope`
/// either does nothing real (`is_bare`: no relationship, no new
/// labels/props -- `MATCH (a) CREATE (a)`) or would silently drop
/// user-written labels/props onto an existing node (any hop count --
/// `MATCH (a) CREATE (a {x: 1})`). Checked here, not only at runtime --
/// a zero-row MATCH would otherwise skip this entirely even though real
/// Cypher's `VariableAlreadyBound` is a structural/scope error, not a
/// data-dependent one (TCK's Create1 [13]/[14]).
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
/// MERGE (`bind_merge`, for each of its own node endpoints) -- both might
/// need to *create* a node the pattern names, so a variable already bound
/// to an *existing* node can't also carry a new label/property predicate
/// (would silently drop it on match, or ambiguously decide whether it
/// applies on create) (TCK's Create1 `[19]`/Merge5 `[22]`). Unlike
/// `check_create_node_not_already_bound`, this alone doesn't also cover
/// the "no relationship and no predicates at all" case -- CREATE and
/// MERGE phrase that differently (MERGE's own bare-node check lives in
/// `bind_merge`, keyed off `pattern.hops.is_empty()` the same way).
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
    // `WITH *` -- `input` already reflects this same clause's own new
    // bindings (`bind_match_pattern`/`bind_unwind`/`bind_merge` all
    // mutate `scope` before calling `apply_with`), so no union with
    // anything else is needed here, unlike `executor::
    // apply_with_or_carry`'s own `carried_vars`/`new_vars` split.
    let with_owned;
    let with: &WithClause = if with.star {
        let star_items = crate::executor::return_star_items(input.keys().cloned())?;
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
        // Unlike RETURN (where an unaliased expression just gets an
        // auto-generated column name, e.g. `RETURN 1+1`), every WITH
        // item that isn't a bare variable reference must have an
        // explicit `AS alias` -- real Cypher's `NoExpressionAlias`
        // error. A bare `Var` needs none since its own name already is
        // the alias (`WITH a` carries `a` forward as itself).
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
        // Real Cypher lets `WITH x AS y WHERE ...` see both the pre-WITH
        // binding (`x`) and the new alias (`y`) -- matches the merged-row
        // evaluation `executor::materialize_with` does at runtime for the
        // same reason (see its docs). Aggregation/DISTINCT both collapse
        // rows, so there's no single pre-WITH scope to fall back to there.
        if crate::executor::has_aggregate(&with.items) || with.distinct {
            validate_with_expr(expr, &projected)?;
        } else {
            let mut merged = input.clone();
            merged.extend(projected.iter().map(|(k, v)| (k.clone(), v.clone())));
            validate_with_expr(expr, &merged)?;
        }
    }
    if let Some(order_by) = &with.order_by {
        // Same `InvalidAggregation` rule as RETURN's own ORDER BY (see the
        // `Statement::Match` arm above) -- TCK's WithOrderBy2 [25].
        let with_aggregates = crate::executor::has_aggregate(&with.items);
        // Real Cypher lets a non-aggregating, non-`DISTINCT` `WITH`'s own
        // `ORDER BY` see both the pre-WITH scope and the new aliases, not
        // just the projected names (`WITH a.count AS count ORDER BY
        // a.count` -- `a` isn't projected but is still a valid sort key,
        // TCK's With4 [6]/WithSkipLimit3 [3]/Return4 [9,11]) -- same
        // merged-scope reasoning `where_clause` above already has, and
        // for the identical reason: aggregation/`DISTINCT` both collapse
        // many pre-WITH rows into one output row, so there's no single
        // pre-WITH scope left to fall back to there.
        let order_scope = if with_aggregates || with.distinct {
            projected.clone()
        } else {
            let mut merged = input.clone();
            merged.extend(projected.iter().map(|(k, v)| (k.clone(), v.clone())));
            merged
        };
        for (expr, _) in order_by {
            // Repeating a WITH item's expression *or own alias* verbatim
            // (see the matching comment on the RETURN side) -- WithOrderBy4
            // [11]/WithSkipLimit1 [2]. Applies to `DISTINCT` too, not just
            // aggregation: both collapse many pre-WITH rows into one
            // output row (that's exactly why `order_scope` above is
            // `projected`-only for either), so a `DISTINCT`-only `WITH`'s
            // `ORDER BY` needs the same shortcut to see its own item's
            // alias instead of failing to resolve a pre-WITH variable it
            // doesn't have access to (TCK's WithOrderBy2 [24] -- previously
            // this shortcut only fired `if with_aggregates`, a real gap: a
            // non-aggregating `DISTINCT` WITH's `order_scope` was *also*
            // narrowed to `projected`-only above, just without this
            // matching escape hatch).
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
                // Not a verbatim match -- may still be a *composed*
                // expression `resolve_grouped_rows`/`rewrite_composed_
                // item` (executor.rs) can evaluate the same way a
                // composed WITH item would, same reasoning as the
                // matching RETURN-side check above (TCK's WithOrderBy4
                // [16]-[18]). A `DISTINCT`-only (non-aggregating) WITH has
                // no such per-group evaluator to fall back to, so that
                // case still just falls through to `infer_expr` below,
                // which correctly fails on anything past its own
                // `projected`-only scope.
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

fn validate_tail(tail: &Option<Tail>, scope: &mut Scope) -> Result<Scope, QueryError> {
    let Some(tail) = tail else {
        return Ok(Scope::new());
    };
    match tail {
        Tail::Return(items, _) => project_return(items, scope),
        Tail::ReturnStar(_) => {
            let items = crate::executor::return_star_items(scope.keys().cloned())?;
            project_return(&items, scope)
        }
        Tail::Delete(exprs, ret) | Tail::DetachDelete(exprs, ret) => {
            for expr in exprs {
                validate_delete_target(expr, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Set(items, ret) => {
            for item in items {
                validate_set_item(item, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Remove(items, ret) => {
            for item in items {
                validate_remove_item(item, scope)?;
            }
            validate_return_tail(ret, scope)
        }
        Tail::Create(patterns, ret) => {
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
    // Some shapes can *never* evaluate to a node/relationship/path, by
    // construction, regardless of what any variable inside them turns out
    // to hold at runtime -- rejected immediately here rather than only once
    // a row actually reaches `delete_value` (which a `MATCH` matching zero
    // rows would skip entirely, real Cypher's own `InvalidArgumentType` is
    // independent of whether any data exists -- TCK's Delete5 `[9]`,
    // `DELETE 1 + 1`). `null` is the one literal exempt, since deleting it
    // is a documented no-op, not a type error.
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
    // `Scalar` is deliberately not rejected here, same reasoning as
    // `bind_unwind`'s: a map/list access (`nodes.key`, `friends[0]`) types
    // as `Scalar` in this codebase's `Kind` system even when it legitimately
    // holds a `Node`/`Edge`/`Path` at runtime (TCK's Delete5 `[3]`/`[5]`
    // scenarios are exactly this shape) -- only a confidently-wrong kind
    // (a real number/string/bool/map) is rejected here, everything else
    // defers to the runtime `QueryError::Type` in `delete_value`.
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
            Ok(())
        }
        Expr::GeneralIsNull(e) => {
            infer_expr(e, scope)?;
            Ok(())
        }
        Expr::GeneralBare(e) => {
            let kind = infer_expr(e, scope)?;
            require_boolean_predicate_kind(&kind, "WHERE predicate")
        }
        Expr::Pattern(pattern) => validate_pattern_predicate(pattern, scope),
        // Unlike `Pattern` above (existential-only, never introduces a
        // variable), `exists {}`'s pattern *can* introduce brand-new
        // node/relationship variables (TCK's ExistentialSubquery1 `[2]`'s
        // `m`), so it reuses `bind_match_pattern` against a scoped copy --
        // same reasoning as `PatternComprehension`'s own handling
        // (`infer_expr`, below) -- these bindings are local to the
        // `exists {}` block, they don't leak into the enclosing scope.
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
    }
}

/// `WHERE (n)-[r:REL]->(m)` etc (TCK's Pattern1) -- every named endpoint
/// must already be bound; unlike `bind_match_pattern` (a real MATCH's own
/// pattern, which introduces new variables), a pattern predicate never
/// does -- real Cypher's `UndefinedVariable` for anything it doesn't
/// recognize (TCK's Pattern1 [10] outline, `MATCH (n) WHERE (n)-[r]->(a)
/// RETURN n` with `a` never bound elsewhere). `require_kind`'s own
/// `lookup` already produces exactly that "references undefined
/// variable" error for an unbound name, so no separate check is needed.
/// An anonymous (var-less) token is always fine, same as any ordinary
/// MATCH pattern.
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
        WithExpr::Bare(e) => {
            let kind = infer_expr(e, scope)?;
            require_boolean_predicate_kind(&kind, "WHERE predicate")
        }
    }
}

fn infer_expr(expr: &ReturnExpr, scope: &Scope) -> Result<Kind, QueryError> {
    Ok(match expr {
        ReturnExpr::Var(var) => lookup(scope, var, "expression")?.clone(),
        ReturnExpr::Prop(access) => {
            require_property_owner(scope, &access.var)?;
            Kind::Scalar
        }
        // `<expr>.prop` where `<expr>` isn't a bare variable -- same
        // permissive stance as `Prop` above (the real node/relationship/
        // map/temporal-value-or-error check is a runtime one, see
        // `executor::property_of_value`); only checks that the base
        // expression itself is well-formed (e.g. no unbound variable
        // inside it).
        ReturnExpr::PropOf(base, _) => {
            infer_expr(base, scope)?;
            Kind::Scalar
        }
        ReturnExpr::Lit(_) | ReturnExpr::CountStar => Kind::Scalar,
        ReturnExpr::Call { name, args, .. } => {
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
                    | "datetime.realtime" => Kind::Scalar,
                    "length" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(kind, &Kind::Path, "length() argument")?;
                        }
                        Kind::Scalar
                    }
                    "nodes" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(kind, &Kind::Path, "nodes() argument")?;
                        }
                        Kind::List(Box::new(Kind::Node))
                    }
                    "relationships" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(kind, &Kind::Path, "relationships() argument")?;
                        }
                        Kind::List(Box::new(Kind::Edge))
                    }
                    // Unlike `keys`/`labels`/`id`/`size`/`exists` (each
                    // polymorphic over several kinds, so left to the
                    // runtime's own `QueryError::Type` below), `type()`
                    // only ever accepts a relationship -- checked here so
                    // `MATCH (r) RETURN type(r)` (`r` a *node*, from the
                    // pattern itself) is a compile-time error even when
                    // the `MATCH` matches zero rows, not only a runtime
                    // one a zero-row match would silently skip (TCK's
                    // Graph4 [7]).
                    "type" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(kind, &Kind::Edge, "type() argument")?;
                        }
                        Kind::Scalar
                    }
                    // Same compile-time-checkable-input-kind reasoning as
                    // `type()` just above -- both only ever accept a
                    // relationship, and return the node at its
                    // start/end.
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
                    // accept a node, relationship, or (for keys/
                    // properties/size) a map/list/string too, depending on
                    // the specific function -- narrower than what the
                    // runtime (`executor::call_builtin`'s own arms) already
                    // enforces with a clear `QueryError::Type`, so no
                    // additional structural check is added here beyond
                    // "the call itself is a recognized function."
                    // `keys`/`labels` each return a *list* of strings, not
                    // a scalar -- real Cypher needs this to be `Kind::
                    // List` so `[x IN labels(n) | ...]`'s own source-kind
                    // check (`list_element`) doesn't wrongly reject a
                    // perfectly good list comprehension source (TCK's
                    // List12 [6]).
                    "keys" | "labels" => Kind::List(Box::new(Kind::Scalar)),
                    "id" | "size" | "exists" => Kind::Scalar,
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
            // `+` alone also means real Cypher's list concatenation/
            // append/prepend (`[1,2] + [3]`, `[1,2] + 3`, `3 + [1,2]`) --
            // `-`/`*`/`/`/`%` have no defined meaning for a list, so
            // those still reject one outright via `require_scalarish`.
            if *op == ArithOp::Add && (matches!(lk, Kind::List(_)) || matches!(rk, Kind::List(_))) {
                Kind::List(Box::new(Kind::Scalar))
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
                // `map['key']` -- real Cypher's dynamic map-field access
                // (`apply_index`'s own runtime already fully supports
                // this, only this compile-time check was too narrow).
                // The result could be any value the map happens to hold
                // at that key -- `Kind::Scalar`, same imprecise fallback
                // `keys`/`labels`/etc already use elsewhere, not worth a
                // per-key type model.
                Kind::Map => Kind::Scalar,
                // `Scalar` is deliberately tolerated here too, not just
                // `Unknown` -- a `null`-valued base types as `Scalar` in
                // this imprecise `Kind` system (see `ReturnExpr::Lit`'s
                // own arm), and indexing into `null` is `null` at
                // runtime (`apply_index`'s own early check), not an
                // error. A genuinely wrong scalar (e.g. a bound integer)
                // still gets `apply_index`'s real `QueryError::Type` at
                // runtime -- same "defer to the runtime check" tolerance
                // every other `Kind::Scalar` case in this module already
                // gives.
                Kind::Unknown | Kind::Scalar => Kind::Unknown,
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
        // Real validation (undefined-variable checks etc) happens via
        // `validate_pattern_predicate` once `return_expr_to_expr` folds
        // this into `Expr::Pattern` -- reaching `infer_expr` at all means
        // it's in a position `Expr`-folding never runs (RETURN/WITH item,
        // function arg, ...), a real compile-time error (TCK's List6 [6]
        // "Fail for size() on pattern predicates" expects a SyntaxError
        // regardless of whether any row ever reaches evaluation -- found
        // via the TCK: the executor's own runtime rejection only fires
        // per-row, silently never triggering on an empty result set).
        ReturnExpr::PatternPredicate(_) => {
            return Err(QueryError::Semantic(
                "a pattern predicate (`(n)-->()` etc) can only be used inside WHERE".into(),
            ))
        }
        // Unlike `PatternPredicate` (existential-only, never introduces a
        // variable -- `validate_pattern_predicate`'s `require_kind`
        // checks, not `bind_kind`), a pattern comprehension is allowed to
        // introduce brand-new node/relationship variables (TCK's
        // Pattern2 `[4]`/`[5]`), so it reuses `bind_match_pattern` (same
        // "new var -> fresh binding, already-bound var -> compatibility
        // check" logic a real `MATCH` pattern gets) against a scoped
        // copy -- these bindings are local to the projection, they don't
        // leak into the enclosing RETURN/WITH scope.
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
        ReturnExpr::ExistsPattern { .. } => {
            return Err(QueryError::Semantic(
                "an exists {} subquery can only be used inside WHERE".into(),
            ))
        }
    })
}

fn list_element(kind: Kind, context: &str) -> Result<Kind, QueryError> {
    match kind {
        Kind::List(element) => Ok(*element),
        // `Scalar` is deliberately not rejected here, same reasoning as
        // `bind_unwind`'s own matching widening: a property access
        // (`n.numbers`) always types as `Kind::Scalar` in this codebase's
        // `Kind` system, even when it legitimately holds a `List` at
        // runtime now that list-valued properties are supported (TCK's
        // Set1 [5], `[i IN n.numbers | i / 2.0]`) -- only a confidently-
        // wrong kind (a real node/edge/map/path) is rejected here,
        // everything else defers to the real runtime `Value::List` check
        // in `eval_return_expr`'s own `ListComp`/`Quantifier` arms.
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
    lookup(scope, var, "property access")?;
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
/// path can *never* be a valid boolean predicate regardless of what data
/// the query runs against (`MATCH (n) WHERE (n) RETURN n`'s `(n)` is a
/// bare node reference, not a pattern predicate), so this is checked here
/// rather than left to `value_to_bool3`'s runtime error -- a zero-row
/// `MATCH` would otherwise never evaluate the predicate at all and the
/// query would wrongly "succeed" (TCK's Pattern1 `[11]`, `InvalidArgumentType`
/// expected "at compile time"). `Scalar`/`Unknown` both pass -- a `Scalar`
/// could still turn out to be a non-boolean scalar (a string/int
/// variable), which stays a real runtime `value_to_bool3` error, same
/// tolerance every other `Kind::Scalar` check in this module already
/// gives.
fn require_boolean_predicate_kind(kind: &Kind, context: &str) -> Result<(), QueryError> {
    match kind {
        Kind::Scalar | Kind::Unknown => Ok(()),
        other => Err(semantic(format!(
            "{context} requires a boolean, but found {}",
            kind_name(other)
        ))),
    }
}
