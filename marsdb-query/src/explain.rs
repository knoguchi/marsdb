//! `EXPLAIN <statement>` — describes the plan a statement would run
//! (scan vs seek, pushdown applied) without executing any of it.
//!
//! Full `LogicalPlan` detail (the actual payoff: seeing whether an
//! `IndexSeek` fired, and which residual `Filter` survives) is only
//! produced for `QueryClause::Match` parts — the only clause kind that
//! ever compiles to a `LogicalPlan` at all (see `ir.rs`'s own doc comment:
//! `UNWIND`/`WITH` are row-vector operations with no traversal/filter
//! shape, `MERGE`'s match-half plan depends on each row's own bindings and
//! isn't practical to show without executing, and `CREATE` has no
//! traversal semantics whatsoever). Those other clause kinds still get a
//! one-line label so the overall statement shape is visible, and
//! `carried_vars` is still threaded through them correctly (via the exact
//! same pure name-computation `execute_match` itself uses) so that a
//! `MATCH` clause *after* one of them sees the right `Seed` vs scan
//! choice.

use std::collections::HashSet;

use marsdb_graph::{PropertyValue, Txn};

use crate::ast::{
    CompareOp, Expr, Literal, MergeClause, QueryClause, QueryPart, RemoveItem, ReturnExpr,
    ReturnItem, SetItem, Statement, Tail, UnwindClause, WithClause,
};
use crate::error::QueryError;
use crate::executor::with_item_output_name;
use crate::ir::{ExpandDirection, LogicalPlan};
use crate::planner::{apply_index_seeks, build_match_plan, pattern_all_vars};

pub fn explain_statement(stmt: &Statement, txn: Txn) -> Result<Vec<String>, QueryError> {
    match stmt {
        Statement::Create(_) => Ok(vec![
            "(no query plan -- CREATE has no traversal/filter plan, it only constructs rows)"
                .to_string(),
        ]),
        Statement::CreateIndex {
            label,
            prop,
            unique,
        } => Ok(vec![format!(
            "(no query plan -- CREATE INDEX ON :{label}({prop}){} declares an index, it doesn't scan/match anything)",
            if *unique { " UNIQUE" } else { "" }
        )]),
        Statement::Explain(_) => Err(QueryError::Semantic(
            "EXPLAIN EXPLAIN isn't supported".to_string(),
        )),
        Statement::Match { clauses, tail, .. } => {
            let mut out = Vec::new();
            let mut carried_vars: HashSet<String> = HashSet::new();
            for clause in clauses {
                explain_clause(clause, txn, &mut carried_vars, &mut out)?;
            }
            if let Some(tail) = tail {
                out.push(explain_tail(tail));
            }
            Ok(out)
        }
        Statement::Union { parts, all } => {
            let mut out = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    out.push(if *all { "UNION ALL".to_string() } else { "UNION".to_string() });
                }
                out.extend(explain_statement(part, txn)?);
            }
            Ok(out)
        }
    }
}

fn explain_clause(
    clause: &QueryClause,
    txn: Txn,
    carried_vars: &mut HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), QueryError> {
    match clause {
        QueryClause::Match(part) => explain_match_part(part, txn, carried_vars, out),
        QueryClause::Unwind(u) => {
            explain_unwind(u, carried_vars, out);
            Ok(())
        }
        QueryClause::Merge(m) => {
            explain_merge(m, carried_vars, out);
            Ok(())
        }
        QueryClause::With(with) => {
            explain_with_projection(with, HashSet::new(), carried_vars, out);
            Ok(())
        }
        QueryClause::Set(items) => {
            out.push(format!(
                "SET {}",
                items
                    .iter()
                    .map(|item| match item {
                        SetItem::Prop(pa, value) => format!("{}.{} = {value:?}", pa.var, pa.prop),
                        SetItem::Labels(var, labels) => format!("{var}:{}", labels.join(":")),
                        SetItem::MapAssign { var, value, merge } => {
                            format!("{var} {}= {value:?}", if *merge { "+" } else { "" })
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            Ok(())
        }
        QueryClause::Delete { items, detach } => {
            out.push(format!(
                "{}DELETE {}",
                if *detach { "DETACH " } else { "" },
                items
                    .iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            Ok(())
        }
        QueryClause::Remove(items) => {
            out.push(format!(
                "REMOVE {}",
                items
                    .iter()
                    .map(|item| match item {
                        RemoveItem::Prop(pa) => format!("{}.{}", pa.var, pa.prop),
                        RemoveItem::Labels(var, labels) => format!("{var}:{}", labels.join(":")),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            Ok(())
        }
        QueryClause::Create(patterns) => {
            out.push(format!(
                "CREATE ({} pattern{})",
                patterns.len(),
                if patterns.len() == 1 { "" } else { "s" }
            ));
            Ok(())
        }
    }
}

fn explain_match_part(
    part: &QueryPart,
    txn: Txn,
    carried_vars: &mut HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), QueryError> {
    if part.shortest_path {
        out.push(format!(
            "{}ShortestPath (BFS, not compiled to a LogicalPlan)",
            if part.optional { "OPTIONAL " } else { "" }
        ));
    } else {
        let plan = apply_index_seeks(
            build_match_plan(&part.pattern, &part.where_clause, carried_vars)?,
            txn,
        )?;
        let header = match (&part.path_var, part.optional) {
            (Some(p), true) => format!("OPTIONAL MATCH p = {p}"),
            (Some(p), false) => format!("MATCH p = {p}"),
            (None, true) => "OPTIONAL MATCH".to_string(),
            (None, false) => "MATCH".to_string(),
        };
        out.push(header);
        format_plan(&plan, 1, out);
    }
    let mut new_vars = pattern_all_vars(&part.pattern);
    if let Some(path_var) = &part.path_var {
        new_vars.insert(path_var.clone());
    }
    apply_with_to_carried_vars(&part.with, new_vars, carried_vars, out);
    Ok(())
}

fn explain_unwind(u: &UnwindClause, carried_vars: &mut HashSet<String>, out: &mut Vec<String>) {
    out.push(format!("UNWIND ... AS {}", u.var));
    apply_with_to_carried_vars(&u.with, HashSet::from([u.var.clone()]), carried_vars, out);
}

fn explain_merge(m: &MergeClause, carried_vars: &mut HashSet<String>, out: &mut Vec<String>) {
    out.push(
        "MERGE (match-or-create; per-row plan depends on each row's own bindings, not shown)"
            .to_string(),
    );
    apply_with_to_carried_vars(&m.with, pattern_all_vars(&m.pattern), carried_vars, out);
}

fn explain_with_projection(
    with: &WithClause,
    new_vars: HashSet<String>,
    carried_vars: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    // `WITH *` -- same `carried_vars ∪ new_vars` reasoning as
    // `executor::apply_with_or_carry`'s own star handling (this
    // function's `carried_vars` hasn't absorbed `new_vars` yet either).
    let with_owned;
    let with: &WithClause = if with.star {
        let mut owned = with.clone();
        let mut items: Vec<_> = carried_vars
            .union(&new_vars)
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|name| ReturnItem {
                expr: ReturnExpr::Var(name),
                alias: None,
            })
            .collect();
        items.extend(owned.items);
        owned.items = items;
        with_owned = owned;
        &with_owned
    } else {
        with
    };
    out.push(format!("WITH {}", with_columns(with)));
    *carried_vars = with
        .items
        .iter()
        .enumerate()
        .map(with_item_output_name)
        .collect();
}

/// Shared "does this clause end in a `WITH`" carried-vars update every
/// clause kind needs: no `WITH` extends `carried_vars` with whatever the
/// clause itself just bound (real Cypher shares one binding scope across
/// WITH-unseparated clauses); a `WITH` replaces `carried_vars` entirely
/// with its own projected output names — mirrors `Executor::
/// apply_with_or_carry`'s carried-vars bookkeeping exactly, without
/// needing any actual bound rows to compute it.
fn apply_with_to_carried_vars(
    with: &Option<WithClause>,
    new_vars: HashSet<String>,
    carried_vars: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    match with {
        None => carried_vars.extend(new_vars),
        Some(with) => explain_with_projection(with, new_vars, carried_vars, out),
    }
}

fn with_columns(with: &WithClause) -> String {
    with.items
        .iter()
        .enumerate()
        .map(with_item_output_name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn explain_tail(tail: &Tail) -> String {
    match tail {
        Tail::Return(items, distinct) => format!(
            "RETURN{} {}",
            if *distinct { " DISTINCT" } else { "" },
            items
                .iter()
                .enumerate()
                .map(with_item_output_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tail::ReturnStar(distinct) => {
            format!("RETURN{} *", if *distinct { " DISTINCT" } else { "" })
        }
        // Targets are a general `ReturnExpr` now (DELETE's target can be
        // any expression, not just a bare variable) -- same `Debug`
        // fallback SET's RHS formatting already uses, for the same reason
        // (no dedicated formatter for the full expression grammar here).
        Tail::Delete(targets, _) => format!(
            "DELETE {}",
            targets
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tail::DetachDelete(targets, _) => format!(
            "DETACH DELETE {}",
            targets
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tail::Set(items, _) => format!(
            "SET {}",
            items
                .iter()
                .map(|item| match item {
                    // `value` is a general `ReturnExpr` now (SET's RHS can
                    // be any expression, not just a literal) -- no
                    // dedicated formatter for the full expression grammar
                    // exists here, so this falls back to `Debug` rather
                    // than under-representing it as if it were always a
                    // literal.
                    SetItem::Prop(pa, value) => format!("{}.{} = {value:?}", pa.var, pa.prop),
                    SetItem::Labels(var, labels) => format!("{var}:{}", labels.join(":")),
                    SetItem::MapAssign { var, value, merge } => {
                        format!("{var} {}= {value:?}", if *merge { "+" } else { "" })
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tail::Remove(items, _) => format!(
            "REMOVE {}",
            items
                .iter()
                .map(|item| match item {
                    RemoveItem::Prop(pa) => format!("{}.{}", pa.var, pa.prop),
                    RemoveItem::Labels(var, labels) => format!("{var}:{}", labels.join(":")),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Tail::Create(patterns, _) => format!(
            "CREATE ({} pattern{})",
            patterns.len(),
            if patterns.len() == 1 { "" } else { "s" }
        ),
    }
}

/// Renders `plan` as an indented operator tree, root (outermost operator,
/// e.g. the final `Filter`) first, its `input` beneath and indented one
/// level further — same top-down reading convention as most databases'
/// own `EXPLAIN` output, even though evaluation order is actually
/// bottom-up (the leaf scan/seek runs first).
fn format_plan(plan: &LogicalPlan, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    match plan {
        LogicalPlan::AllNodesScan { var } => out.push(format!("{pad}AllNodesScan({var})")),
        LogicalPlan::NodeByLabelScan { var, label } => {
            out.push(format!("{pad}NodeByLabelScan({var}:{label})"))
        }
        LogicalPlan::IndexSeek {
            var,
            label,
            prop,
            value,
        } => out.push(format!(
            "{pad}IndexSeek({var}:{label} {{{prop}: {}}})",
            format_property_value(value)
        )),
        LogicalPlan::Seed { var } => out.push(format!("{pad}Seed({var})")),
        LogicalPlan::Expand {
            input,
            from_var,
            to_var,
            rel_var,
            rel_labels,
            direction,
        } => {
            out.push(format!(
                "{pad}Expand({from_var}){}({to_var})",
                rel_arrow(*direction, rel_var, rel_labels)
            ));
            format_plan(input, depth + 1, out);
        }
        LogicalPlan::VarExpand {
            input,
            from_var,
            to_var,
            rel_labels,
            direction,
            min_hops,
            max_hops,
        } => {
            let hops = match max_hops {
                Some(max) => format!("*{min_hops}..{max}"),
                None => format!("*{min_hops}.."),
            };
            out.push(format!(
                "{pad}VarExpand({from_var}){}({to_var})",
                rel_arrow_var(*direction, rel_labels, &hops)
            ));
            format_plan(input, depth + 1, out);
        }
        LogicalPlan::Filter { input, predicate } => {
            out.push(format!("{pad}Filter {}", format_expr(predicate)));
            format_plan(input, depth + 1, out);
        }
    }
}

fn rel_arrow(
    direction: ExpandDirection,
    rel_var: &Option<String>,
    rel_labels: &[String],
) -> String {
    let inner = rel_label_text(rel_var, rel_labels);
    match direction {
        ExpandDirection::Out => format!("-[{inner}]->"),
        ExpandDirection::In => format!("<-[{inner}]-"),
        ExpandDirection::Either => format!("-[{inner}]-"),
    }
}

fn rel_arrow_var(direction: ExpandDirection, rel_labels: &[String], hops: &str) -> String {
    let inner = format!("{}{hops}", rel_labels_text(rel_labels));
    match direction {
        ExpandDirection::Out => format!("-[{inner}]->"),
        ExpandDirection::In => format!("<-[{inner}]-"),
        ExpandDirection::Either => format!("-[{inner}]-"),
    }
}

/// `[:A|B]` -- joined with `|`, empty means untyped (no `:` at all).
fn rel_labels_text(rel_labels: &[String]) -> String {
    if rel_labels.is_empty() {
        String::new()
    } else {
        format!(":{}", rel_labels.join("|"))
    }
}

fn rel_label_text(rel_var: &Option<String>, rel_labels: &[String]) -> String {
    // `rel_var` here is `LogicalPlan::Expand`'s `rel_var` -- always some
    // name, even for a pattern the user wrote with none at all
    // (`build_match_plan` synthesizes an internal `__anonN` so it can
    // still enforce edge-isomorphism/inline-property filters). Showing
    // that synthesized name to an EXPLAIN reader would look like a real
    // binding that leaked out; only a real user-written var is worth
    // surfacing here.
    let user_var = rel_var.as_deref().filter(|v| !v.starts_with("__anon"));
    let labels = rel_labels_text(rel_labels);
    match user_var {
        Some(v) if labels.is_empty() => v.to_string(),
        Some(v) => format!("{v}{labels}"),
        None => labels,
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::And(l, r) => format!("({} AND {})", format_expr(l), format_expr(r)),
        Expr::Or(l, r) => format!("({} OR {})", format_expr(l), format_expr(r)),
        Expr::Not(e) => format!("NOT {}", format_expr(e)),
        Expr::Compare(pa, op, lit) => format!(
            "{}.{} {} {}",
            pa.var,
            pa.prop,
            format_compare_op(*op),
            format_literal(lit)
        ),
        Expr::PropCompare(l, op, r) => format!(
            "{}.{} {} {}.{}",
            l.var,
            l.prop,
            format_compare_op(*op),
            r.var,
            r.prop
        ),
        Expr::IsNull(pa) => format!("{}.{} IS NULL", pa.var, pa.prop),
        Expr::HasLabel(var, label) => format!("{var}:{label}"),
        Expr::VarEq(a, b) => format!("{a} = {b}"),
        // Operands are a general `ReturnExpr` (function calls, arithmetic,
        // ...) -- same `Debug` fallback DELETE/SET already use above, for
        // the same reason (no dedicated formatter for the full expression
        // grammar here).
        Expr::GeneralCompare(l, op, r) => {
            format!("{l:?} {} {r:?}", format_compare_op(*op))
        }
        Expr::GeneralIsNull(e) => format!("{e:?} IS NULL"),
        Expr::GeneralBare(e) => format!("{e:?}"),
        Expr::Pattern(pattern) => format!("{pattern:?}"),
    }
}

fn format_compare_op(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "=",
        CompareOp::Ne => "<>",
        CompareOp::Lt => "<",
        CompareOp::Le => "<=",
        CompareOp::Gt => ">",
        CompareOp::Ge => ">=",
        CompareOp::StartsWith => "STARTS WITH",
        CompareOp::EndsWith => "ENDS WITH",
        CompareOp::Contains => "CONTAINS",
    }
}

fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(i) => i.to_string(),
        Literal::Float(f) => f.to_string(),
        Literal::String(s) => format!("'{s}'"),
        Literal::Bool(b) => b.to_string(),
        Literal::Null => "null".to_string(),
        Literal::Param(p) => format!("${p}"),
    }
}

fn format_property_value(v: &PropertyValue) -> String {
    match v {
        PropertyValue::Null => "null".to_string(),
        PropertyValue::Bool(b) => b.to_string(),
        PropertyValue::Int(i) => i.to_string(),
        PropertyValue::Float(f) => f.to_string(),
        PropertyValue::String(s) => format!("'{s}'"),
        PropertyValue::Date(_)
        | PropertyValue::Duration { .. }
        | PropertyValue::LocalTime(_)
        | PropertyValue::Time { .. }
        | PropertyValue::LocalDateTime { .. }
        | PropertyValue::DateTime { .. }
        | PropertyValue::List(_) => format!("{v:?}"),
    }
}
