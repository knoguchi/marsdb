//! Statement-level name binding and structural type validation.
//!
//! This pass runs after parsing/parameter substitution and before a storage
//! transaction is opened. It deliberately validates only types knowable from
//! query structure (node, relationship, list, map, path, scalar); property
//! value types remain data-dependent runtime checks.

use std::collections::HashMap;

use crate::ast::{
    is_aggregate_name, Expr, Literal, MergeClause, NodePattern, Pattern, QueryClause, RemoveItem,
    ReturnExpr, ReturnItem, ReturnTail, SetItem, Statement, Tail, UnwindClause, WithClause,
    WithExpr,
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
                        // expression verbatim (`RETURN sum(x) AS s ORDER
                        // BY sum(x)`) refers to that already-aggregated
                        // item, not a fresh expression -- its kind is
                        // already known from `output_scope`, and
                        // re-running `infer_expr` on it would need
                        // pre-aggregation bindings (like `x`'s row) that
                        // no longer exist post-grouping (TCK's
                        // WithOrderBy4 [11]/`ReturnOrderBy3`). Only an
                        // aggregate that *doesn't* match any RETURN item
                        // is rejected here.
                        if tail_items.unwrap().iter().any(|item| item.expr == *expr) {
                            continue;
                        }
                        if crate::executor::contains_aggregate(expr) {
                            return Err(semantic(
                                "ORDER BY aggregate does not match any RETURN item",
                            ));
                        }
                    } else if crate::executor::contains_aggregate(expr) {
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
                && pattern.start.props.is_empty()
                && scope.contains_key(var)
            {
                return Err(semantic(format!(
                    "'{var}' is already bound — MERGE ({var}) with no relationship and no \
                     labels/properties doesn't search for or create anything"
                )));
            }
        }
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
    }
    bind_match_pattern(pattern, scope)?;
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
        // Unlike MATCH (where an untyped hop just means "any relationship"),
        // CREATE always makes exactly one new relationship, and a brand
        // new edge with no type is meaningless -- real Cypher requires an
        // explicit `:TYPE` here, it's never inferred/defaulted.
        if rel.rel_type.is_none() {
            return Err(semantic(
                "CREATE requires an explicit relationship type (e.g. -[:KNOWS]->) -- unlike MATCH, \
                 an untyped relationship pattern can't be created",
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
    if is_bare && node.labels.is_empty() && node.props.is_empty() {
        return Err(semantic(format!(
            "'{var}' is already bound — CREATE ({var}) with no relationship and no new \
             labels/properties doesn't create or connect anything"
        )));
    }
    if !node.labels.is_empty() || !node.props.is_empty() {
        return Err(semantic(format!(
            "'{var}' is already bound — CREATE can't add labels/properties to an existing node"
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
        // same reason (see its docs). Aggregation collapses rows, so
        // there's no single pre-WITH scope to fall back to there.
        if crate::executor::has_aggregate(&with.items) {
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
        for (expr, _) in order_by {
            if with_aggregates {
                // Repeating a WITH item's expression verbatim (see the
                // matching comment on the RETURN side) -- WithOrderBy4
                // [11].
                if with.items.iter().any(|item| item.expr == *expr) {
                    continue;
                }
                if crate::executor::contains_aggregate(expr) {
                    return Err(semantic("ORDER BY aggregate does not match any WITH item"));
                }
            } else if crate::executor::contains_aggregate(expr) {
                return Err(semantic(
                    "ORDER BY cannot use an aggregate function unless WITH itself is \
                     aggregating",
                ));
            }
            infer_expr(expr, &projected)?;
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
        Tail::Delete(exprs, ret) | Tail::DetachDelete(exprs, ret) => {
            for expr in exprs {
                // Some shapes can *never* evaluate to a node/relationship/
                // path, by construction, regardless of what any variable
                // inside them turns out to hold at runtime -- rejected
                // immediately here rather than only once a row actually
                // reaches `delete_value` (which a `MATCH` matching zero
                // rows would skip entirely, real Cypher's own
                // `InvalidArgumentType` is independent of whether any data
                // exists -- TCK's Delete5 `[9]`, `DELETE 1 + 1`). `null` is
                // the one literal exempt, since deleting it is a
                // documented no-op, not a type error.
                if !matches!(expr, ReturnExpr::Lit(Literal::Null))
                    && matches!(
                        expr,
                        ReturnExpr::Lit(_)
                            | ReturnExpr::CountStar
                            | ReturnExpr::Arith(..)
                            | ReturnExpr::And(..)
                            | ReturnExpr::Or(..)
                            | ReturnExpr::Xor(..)
                            | ReturnExpr::Not(..)
                            | ReturnExpr::Compare(..)
                            | ReturnExpr::IsNull(..)
                            | ReturnExpr::MapLit(..)
                            | ReturnExpr::ListLit(..)
                    )
                {
                    return Err(semantic(
                        "DELETE target must evaluate to a node, relationship, or path -- a \
                         literal/arithmetic/boolean/map/list expression never can",
                    ));
                }
                let kind = infer_expr(expr, scope)?;
                // `Scalar` is deliberately not rejected here, same
                // reasoning as `bind_unwind`'s: a map/list access
                // (`nodes.key`, `friends[0]`) types as `Scalar` in this
                // codebase's `Kind` system even when it legitimately holds
                // a `Node`/`Edge`/`Path` at runtime (TCK's Delete5 `[3]`/
                // `[5]` scenarios are exactly this shape) -- only a
                // confidently-wrong kind (a real number/string/bool/map)
                // is rejected here, everything else defers to the runtime
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

fn validate_set_item(item: &SetItem, scope: &Scope) -> Result<(), QueryError> {
    match item {
        SetItem::Prop(access, value) => {
            require_graph(scope, &access.var, "SET property target")?;
            infer_expr(value, scope)?;
            Ok(())
        }
        SetItem::Labels(var, _) => require_kind(scope, var, &Kind::Node, "SET label target"),
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
    }
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
    }
}

fn infer_expr(expr: &ReturnExpr, scope: &Scope) -> Result<Kind, QueryError> {
    Ok(match expr {
        ReturnExpr::Var(var) => lookup(scope, var, "expression")?.clone(),
        ReturnExpr::Prop(access) => {
            require_property_owner(scope, &access.var)?;
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
                    | "datetime.truncate" => Kind::Scalar,
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
                    // `keys`/`labels`/`properties`/`id`/`size`/`exists`
                    // accept a node, relationship, or (for keys/
                    // properties/size) a map/list/string too, depending on
                    // the specific function -- narrower than what the
                    // runtime (`executor::call_builtin`'s own arms) already
                    // enforces with a clear `QueryError::Type`, so no
                    // additional structural check is added here beyond
                    // "the call itself is a recognized function."
                    "keys" | "labels" | "id" | "size" | "exists" => Kind::Scalar,
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
                    | "round" | "sqrt" | "sign" => Kind::Scalar,
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
        ReturnExpr::Arith(left, _, right) => {
            require_scalarish(&infer_expr(left, scope)?, "arithmetic operand")?;
            require_scalarish(&infer_expr(right, scope)?, "arithmetic operand")?;
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
                Kind::Unknown => Kind::Unknown,
                other => {
                    return Err(semantic(format!(
                        "index base is {}, not a list",
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
        ReturnExpr::HasLabel(var, _) => {
            require_graph(scope, var, "(n:Label) target")?;
            Kind::Scalar
        }
    })
}

fn list_element(kind: Kind, context: &str) -> Result<Kind, QueryError> {
    match kind {
        Kind::List(element) => Ok(*element),
        Kind::Unknown => Ok(Kind::Unknown),
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
