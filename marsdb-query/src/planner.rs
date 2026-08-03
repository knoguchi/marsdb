use std::collections::HashSet;

use crate::ast::{CompareOp, Expr, NodePattern, Pattern, PropAccess, RelDirection};
use crate::error::QueryError;
use crate::ir::{ExpandDirection, LogicalPlan};

struct VarNamer {
    next: usize,
}

impl VarNamer {
    fn new() -> Self {
        Self { next: 0 }
    }

    /// Anonymous nodes/rels (e.g. `(a)-->()`) still need a name to track
    /// their binding through the plan; synthesize one that can't collide
    /// with a user-written identifier.
    fn name(&mut self, given: &Option<String>) -> String {
        match given {
            Some(v) => v.clone(),
            None => {
                let n = format!("__anon{}", self.next);
                self.next += 1;
                n
            }
        }
    }
}

pub fn build_match_plan(
    pattern: &Pattern,
    where_clause: &Option<Expr>,
    carried_vars: &HashSet<String>,
) -> Result<LogicalPlan, QueryError> {
    let mut namer = VarNamer::new();
    let start_var = namer.name(&pattern.start.var);
    let mut plan = if carried_vars.contains(&start_var) {
        // Already bound by a prior QueryPart's WITH output — continue from
        // it instead of re-scanning, same Filter treatment as a hop node
        // (no preceding scan narrowed it, so check every listed label).
        wrap_labels_and_props(LogicalPlan::Seed { var: start_var.clone() }, &start_var, &pattern.start, 0)
    } else {
        scan_for(&start_var, &pattern.start)
    };
    let mut from_var = start_var.clone();
    // Real Cypher pattern matching is edge-isomorphic: no single MATCH
    // pattern may bind two hops to the *same* relationship instance, even
    // if their types/directions differ (a self-loop plus an undirected hop
    // back out is the case that surfaces this — without this check, the
    // hop back out can silently re-match the edge the previous hop just
    // came in on). Scoped to hops within *this* pattern only — a separate
    // MATCH clause, or a separate comma-separated pattern, may reuse the
    // same relationship freely.
    let mut prior_rel_vars: Vec<String> = Vec::new();
    // A node variable can repeat *within* one pattern too, not just across
    // a `WITH` boundary -- `MATCH (n)-[r]->(n)` (a self-relationship) reuses
    // `n` for both ends of the same pattern. Seeded with the start node so
    // a hop reusing its name is recognized as a repeat from the first hop
    // onward, same as a `carried_vars` repeat.
    let mut pattern_bound_vars: HashSet<String> = HashSet::new();
    pattern_bound_vars.insert(start_var);
    // Unlike a repeated *node* variable (a legal, meaningful constraint --
    // see `pattern_bound_vars` above), real Cypher rejects a relationship
    // variable written twice within one pattern outright, at compile time
    // (`MATCH (a)-[r]->()-[r]->(a)` — never "silently filter to the one
    // case where both hops happen to be the same edge"). Tracked
    // separately from `prior_rel_vars` below, which holds internal
    // synthesized names for the *different*, allowed edge-isomorphism
    // check ("two hops can't reuse the same relationship *instance*" even
    // when they're different variables or none at all).
    let mut pattern_rel_var_names: HashSet<String> = HashSet::new();
    for (rel, node) in &pattern.hops {
        // "Bound-node repetition": this hop's variable was already bound
        // before this hop -- either from a prior QueryPart (e.g. IS7's `p`,
        // bound by an earlier MATCH, reappearing as the endpoint of an
        // OPTIONAL MATCH pattern) or earlier in this same pattern (e.g. a
        // self-relationship `(n)-[r]->(n)`). Must synthesize a FRESH name
        // for the Expand to bind — reusing the original name here would let
        // Expand's `new_row.insert` overwrite the existing binding before
        // it can be compared against, defeating the whole check.
        let is_repeat = node.var.as_ref().is_some_and(|v| carried_vars.contains(v) || pattern_bound_vars.contains(v));
        let to_var = if is_repeat { namer.name(&None) } else { namer.name(&node.var) };
        if !is_repeat {
            if let Some(v) = &node.var {
                pattern_bound_vars.insert(v.clone());
            }
        }
        let direction = match rel.direction {
            RelDirection::Right => ExpandDirection::Out,
            RelDirection::Left => ExpandDirection::In,
            RelDirection::Either => ExpandDirection::Either,
        };
        // Same "bound-*-repetition" concern as `is_repeat` above, but for
        // the relationship variable: if it already names something from a
        // prior QueryPart (e.g. `WITH r1 AS r2 MATCH ()-[r2]->()`), this
        // hop must mean "verify *this exact* relationship again", not
        // "match any relationship and rebind r2 to it" -- the latter would
        // silently overwrite the carried binding with whatever the last
        // Expand candidate happened to be instead of filtering down to it.
        if let Some(v) = &rel.var {
            if pattern_rel_var_names.contains(v) {
                return Err(QueryError::Parse(format!(
                    "'{v}' is used for two different relationships in the same pattern — a relationship \
                     variable can't be reused within one MATCH pattern"
                )));
            }
            if !carried_vars.contains(v) {
                pattern_rel_var_names.insert(v.clone());
            }
        }
        let rel_is_repeat = rel.var.as_ref().is_some_and(|v| carried_vars.contains(v));
        // A fixed-hop relationship is always bound to an internal name, even
        // when the user didn't write one -- needed both to filter inline
        // properties (`-[:KNOWS {name: 'x'}]->`) and to enforce edge
        // isomorphism against earlier hops in this same pattern (below).
        // Never leaks: nothing outside this function's own Filters
        // reference a synthesized name, and downstream `RETURN`/`WHERE`
        // can't reference an identifier the user never wrote.
        let rel_filter_var = if rel.hop_range.is_none() {
            Some(if rel_is_repeat { namer.name(&None) } else { namer.name(&rel.var) })
        } else {
            rel.var.clone()
        };
        plan = match rel.hop_range {
            None => LogicalPlan::Expand {
                input: Box::new(plan),
                from_var: from_var.clone(),
                to_var: to_var.clone(),
                rel_var: rel_filter_var.clone(),
                rel_label: rel.rel_type.clone(),
                direction,
            },
            Some((min_hops, max_hops)) => {
                if rel.var.is_some() {
                    // Real Cypher binds a *list* of relationships for a
                    // variable-length pattern's rel_var; v1 doesn't support
                    // that value shape, so reject rather than silently bind
                    // just the last hop's edge (wrong, not just incomplete).
                    return Err(QueryError::Parse(
                        "binding a variable name to a variable-length relationship (e.g. \
                         [r:TYPE*1..3]) isn't supported — omit the variable name"
                            .into(),
                    ));
                }
                if !rel.props.is_empty() {
                    // Filtering each hop of a variable-length relationship by
                    // the same inline property map isn't supported -- reject
                    // rather than silently ignore the props (same reasoning
                    // as everywhere else in this planner: a correctness trap
                    // otherwise).
                    return Err(QueryError::Parse(
                        "inline properties on a variable-length relationship pattern (e.g. \
                         [:TYPE* {prop: 'x'}]) aren't supported"
                            .into(),
                    ));
                }
                LogicalPlan::VarExpand {
                    input: Box::new(plan),
                    from_var: from_var.clone(),
                    to_var: to_var.clone(),
                    rel_label: rel.rel_type.clone(),
                    direction,
                    min_hops,
                    max_hops,
                }
            }
        };
        // Hop nodes reach this point via Expand/VarExpand, which don't
        // pre-filter by label at all (unlike the start node's
        // NodeByLabelScan) — every listed label must be Filter-checked
        // here, not just the extras beyond the first.
        plan = wrap_labels_and_props(plan, &to_var, node, 0);
        if let Some(rel_var) = &rel_filter_var {
            for (key, lit) in &rel.props {
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: Expr::Compare(
                        PropAccess { var: rel_var.clone(), prop: key.clone() },
                        CompareOp::Eq,
                        lit.clone(),
                    ),
                };
            }
            for prior in &prior_rel_vars {
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: Expr::Not(Box::new(Expr::VarEq(rel_var.clone(), prior.clone()))),
                };
            }
            if rel_is_repeat {
                let original = rel.var.clone().expect("rel_is_repeat implies rel.var is Some");
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: Expr::VarEq(rel_var.clone(), original),
                };
            }
            // A variable-length hop (`VarExpand`) doesn't bind a single edge
            // to check future hops against -- its own internally-traversed
            // edges aren't tracked here, a pre-existing scope gap, not a
            // regression (it was never checked before this).
            if rel.hop_range.is_none() {
                prior_rel_vars.push(rel_var.clone());
            }
        }
        if is_repeat {
            let original = node.var.clone().expect("is_repeat implies node.var is Some");
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate: Expr::VarEq(to_var.clone(), original),
            };
        }
        from_var = to_var;
    }
    if let Some(expr) = where_clause {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: expr.clone(),
        };
    }
    Ok(plan)
}

fn scan_for(var: &str, node: &NodePattern) -> LogicalPlan {
    // The first label (if any) narrows the scan; any additional labels
    // (`(n:Post:Message)`) become extra HasLabel filters — a node must
    // have ALL listed labels, matching Cypher's multi-label AND semantics.
    let base = match node.labels.first() {
        Some(label) => LogicalPlan::NodeByLabelScan {
            var: var.to_string(),
            label: label.clone(),
        },
        None => LogicalPlan::AllNodesScan { var: var.to_string() },
    };
    // Skip the first label — NodeByLabelScan above already selected for it.
    wrap_labels_and_props(base, var, node, 1)
}

/// Inline node-pattern properties (`(a:Person {name:'Alice'})`) and any
/// labels not already handled by a preceding scan (`skip` labels from the
/// front) compile to the same Filter machinery as a WHERE clause, just
/// synthesized from the pattern.
fn wrap_labels_and_props(plan: LogicalPlan, var: &str, node: &NodePattern, skip: usize) -> LogicalPlan {
    let mut plan = plan;
    for label in node.labels.iter().skip(skip) {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: Expr::HasLabel(var.to_string(), label.clone()),
        };
    }
    for (key, lit) in &node.props {
        let predicate = Expr::Compare(
            PropAccess {
                var: var.to_string(),
                prop: key.clone(),
            },
            CompareOp::Eq,
            lit.clone(),
        );
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate,
        };
    }
    plan
}

/// All variable names (node + relationship) a pattern binds, regardless of
/// whether they're a fresh binding or a bound-node repetition. Used by the
/// executor to grow `carried_vars` across `QueryPart`s that aren't
/// separated by a `WITH` — real Cypher shares one binding scope across
/// `MATCH`/`OPTIONAL MATCH` clauses that aren't WITH-separated.
pub fn pattern_all_vars(pattern: &Pattern) -> HashSet<String> {
    let mut vars = HashSet::new();
    if let Some(v) = &pattern.start.var {
        vars.insert(v.clone());
    }
    for (rel, node) in &pattern.hops {
        if let Some(v) = &rel.var {
            vars.insert(v.clone());
        }
        if let Some(v) = &node.var {
            vars.insert(v.clone());
        }
    }
    vars
}

/// Variables this pattern introduces newly — excludes anything already in
/// `carried_vars` (those are Seed/`VarEq` repetitions, not fresh
/// bindings). Used by `OPTIONAL MATCH` null-padding to know exactly which
/// keys need `Null` when the whole pattern fails to match for an outer
/// row — a repeated variable keeps whatever it already was, only genuinely
/// new ones need padding.
pub fn pattern_new_vars(pattern: &Pattern, carried_vars: &HashSet<String>) -> HashSet<String> {
    pattern_all_vars(pattern)
        .into_iter()
        .filter(|v| !carried_vars.contains(v))
        .collect()
}
