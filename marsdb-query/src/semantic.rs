//! Statement-level name binding and structural type validation.
//!
//! This pass runs after parsing/parameter substitution and before a storage
//! transaction is opened. It deliberately validates only types knowable from
//! query structure (node, relationship, list, map, path, scalar); property
//! value types remain data-dependent runtime checks.

use std::collections::HashMap;

use crate::ast::{
    is_aggregate_name, Expr, MergeClause, Pattern, QueryClause, RemoveItem, ReturnExpr, ReturnItem,
    ReturnTail, SetItem, Statement, Tail, UnwindClause, UnwindSource, WithClause, WithExpr,
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
                for (expr, _) in order_by {
                    infer_expr(expr, &order_scope)?;
                }
            }
            Ok(())
        }
    }
}

fn bind_unwind(clause: &UnwindClause, scope: &mut Scope) -> Result<(), QueryError> {
    let element_kind = match &clause.source {
        UnwindSource::Var(name) => match lookup(scope, name, "UNWIND source")? {
            Kind::List(element) => (**element).clone(),
            Kind::Unknown => Kind::Unknown,
            other => {
                return Err(semantic(format!(
                    "UNWIND source '{name}' is {}, not a list",
                    kind_name(other)
                )))
            }
        },
        UnwindSource::List(_) => Kind::Scalar,
    };
    scope.insert(clause.var.clone(), element_kind);
    if let Some(expr) = &clause.where_clause {
        validate_with_expr(expr, scope)?;
    }
    apply_with(&clause.with, scope)
}

fn bind_merge(clause: &MergeClause, scope: &mut Scope) -> Result<(), QueryError> {
    bind_match_pattern(&clause.pattern, scope)?;
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
    if let Some(var) = &pattern.start.var {
        bind_kind(scope, var, Kind::Node, "CREATE node")?;
    }
    for (rel, node) in &pattern.hops {
        validate_props(&node.props, scope)?;
        if let Some(var) = &node.var {
            bind_kind(scope, var, Kind::Node, "CREATE node")?;
        }
        validate_props(&rel.props, scope)?;
        if let Some(var) = &rel.var {
            bind_kind(scope, var, Kind::Edge, "CREATE relationship")?;
        }
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
        let kind = infer_expr(&item.expr, input)?;
        let name = item_output_name(index, item);
        if projected.insert(name.clone(), kind).is_some() {
            return Err(semantic(format!(
                "WITH projects duplicate variable '{name}'"
            )));
        }
    }
    if let Some(expr) = &with.where_clause {
        validate_with_expr(expr, &projected)?;
    }
    if let Some(order_by) = &with.order_by {
        for (expr, _) in order_by {
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
        Tail::Delete(vars, ret) | Tail::DetachDelete(vars, ret) => {
            for var in vars {
                require_graph(scope, var, "DELETE target")?;
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
        projected.insert(
            item_output_name(index, item),
            infer_expr(&item.expr, scope)?,
        );
    }
    Ok(projected)
}

fn validate_set_item(item: &SetItem, scope: &Scope) -> Result<(), QueryError> {
    match item {
        SetItem::Prop(access, _) => require_graph(scope, &access.var, "SET property target"),
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
        Expr::HasLabel(var, _) => require_kind(scope, var, &Kind::Node, "label predicate"),
        Expr::VarEq(left, right) => {
            require_graph(scope, left, "identity predicate")?;
            require_graph(scope, right, "identity predicate")
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
        WithExpr::Compare(left, _, _) => {
            infer_expr(left, scope)?;
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
                    "tointeger" | "tostring" | "date" | "duration" => Kind::Scalar,
                    "length" => {
                        if let Some(kind) = arg_kinds.first() {
                            require_compatible_kind(kind, &Kind::Path, "length() argument")?;
                        }
                        Kind::Scalar
                    }
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
    QueryError::Parse(format!("semantic error: {}", message.into()))
}
