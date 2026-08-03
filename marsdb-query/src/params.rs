use std::collections::HashMap;

use marsdb_graph::PropertyValue;

use crate::ast::{Expr, Literal, NodePattern, Pattern, ReturnExpr, Statement, Tail};
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
            pattern,
            where_clause,
            tail,
            limit: _,
        } => {
            substitute_pattern(pattern, params)?;
            if let Some(expr) = where_clause {
                substitute_expr(expr, params)?;
            }
            substitute_tail(tail, params)?;
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
    }
    Ok(())
}

fn substitute_return_expr(expr: &mut ReturnExpr, params: &HashMap<String, PropertyValue>) -> Result<(), QueryError> {
    match expr {
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) => {}
        ReturnExpr::Lit(lit) => substitute_literal(lit, params)?,
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
