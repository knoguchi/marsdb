use std::collections::HashMap;

use marsdb_graph::PropertyValue;

use crate::ast::{Expr, Literal, NodePattern, Pattern, QueryPart, ReturnExpr, Statement, Tail, WithClause, WithExpr};
use crate::error::QueryError;

/// Resolves every `$name` placeholder in `stmt` to a concrete `Literal`
/// using `params`, in place. Called before execution so the executor never
/// sees `Literal::Param` — see the `unreachable!` in
/// `executor::literal_to_value`.
pub fn substitute_params(stmt: &mut Statement, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match stmt {
        Statement::Create(patterns) => {
            for pattern in patterns {
                substitute_pattern(pattern, params)?;
            }
        }
        Statement::Match {
            parts,
            tail,
            order_by,
            limit: _,
        } => {
            for part in parts {
                substitute_query_part(part, params)?;
            }
            substitute_tail(tail, params)?;
            if let Some(items) = order_by {
                for (expr, _) in items {
                    substitute_return_expr(expr, params)?;
                }
            }
        }
    }
    Ok(())
}

fn substitute_query_part(part: &mut QueryPart, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    substitute_pattern(&mut part.pattern, params)?;
    if let Some(expr) = &mut part.where_clause {
        substitute_expr(expr, params)?;
    }
    if let Some(with) = &mut part.with {
        substitute_with_clause(with, params)?;
    }
    Ok(())
}

fn substitute_with_clause(with: &mut WithClause, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
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
    Ok(())
}

fn substitute_with_expr(expr: &mut WithExpr, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match expr {
        WithExpr::And(l, r) | WithExpr::Or(l, r) => {
            substitute_with_expr(l, params)?;
            substitute_with_expr(r, params)?;
        }
        WithExpr::Not(e) => substitute_with_expr(e, params)?,
        WithExpr::Compare(lhs, _, lit) => {
            substitute_return_expr(lhs, params)?;
            substitute_literal(lit, params)?;
        }
    }
    Ok(())
}

fn substitute_pattern(pattern: &mut Pattern, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    substitute_node(&mut pattern.start, params)?;
    for (rel, node) in &mut pattern.hops {
        for (_, lit) in &mut rel.props {
            substitute_literal(lit, params)?;
        }
        substitute_node(node, params)?;
    }
    Ok(())
}

fn substitute_node(node: &mut NodePattern, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    for (_, lit) in &mut node.props {
        substitute_literal(lit, params)?;
    }
    Ok(())
}

fn substitute_expr(expr: &mut Expr, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match expr {
        Expr::And(l, r) | Expr::Or(l, r) => {
            substitute_expr(l, params)?;
            substitute_expr(r, params)?;
        }
        Expr::Not(e) => substitute_expr(e, params)?,
        Expr::Compare(_, _, lit) => substitute_literal(lit, params)?,
        Expr::HasLabel(_, _) => {}
        Expr::VarEq(_, _) => {}
    }
    Ok(())
}

fn substitute_tail(tail: &mut Tail, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match tail {
        Tail::Return(items) => {
            for item in items {
                substitute_return_expr(&mut item.expr, params)?;
            }
        }
        Tail::Delete(_) | Tail::DetachDelete(_) => {}
        Tail::Set(items) => {
            for (_, lit) in items {
                substitute_literal(lit, params)?;
            }
        }
        Tail::Create(patterns) => {
            for pattern in patterns {
                substitute_pattern(pattern, params)?;
            }
        }
    }
    Ok(())
}

fn substitute_return_expr(expr: &mut ReturnExpr, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match expr {
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::CountStar => {}
        ReturnExpr::Lit(lit) => substitute_literal(lit, params)?,
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
    }
    Ok(())
}

fn substitute_literal(lit: &mut Literal, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    if let Literal::Param(name) = lit {
        let value = params
            .get(name)
            .ok_or_else(|| QueryError::MissingParam(name.clone()))?;
        *lit = property_value_to_literal(value);
    }
    Ok(())
}

fn property_value_to_literal(pv: &PropertyValue) -> Literal {
    match pv {
        PropertyValue::Null => Literal::Null,
        PropertyValue::Bool(b) => Literal::Bool(*b),
        PropertyValue::Int(i) => Literal::Int(*i),
        PropertyValue::Float(f) => Literal::Float(*f),
        PropertyValue::String(s) => Literal::String(s.clone()),
    }
}
