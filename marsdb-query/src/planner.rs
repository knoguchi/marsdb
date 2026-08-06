use std::collections::HashSet;

use marsdb_graph::{GraphStore, Txn};

use crate::ast::{
    CompareOp, Expr, Literal, NodePattern, Pattern, PropAccess, RelDirection, ReturnExpr,
};
use crate::error::QueryError;
use crate::executor::literal_to_value;
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
        wrap_labels_and_props(
            LogicalPlan::Seed {
                var: start_var.clone(),
            },
            &start_var,
            &pattern.start,
            0,
        )?
    } else {
        scan_for(&start_var, &pattern.start)?
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
    // Complementary to `prior_rel_vars` above -- edges an *earlier
    // variable-length* hop of this same pattern traversed can't be named
    // by a single id the way a fixed hop's own `rel_var` can (each row's
    // own BFS can use a different set of edges), so this tracks each such
    // hop's own `exclude_edge_var` name instead (see `LogicalPlan::
    // VarExpand::exclude_edge_sets`'s own docs).
    let mut prior_edge_sets: Vec<String> = Vec::new();
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
        let is_repeat = node
            .var
            .as_ref()
            .is_some_and(|v| carried_vars.contains(v) || pattern_bound_vars.contains(v));
        let to_var = if is_repeat {
            namer.name(&None)
        } else {
            namer.name(&node.var)
        };
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
                return Err(QueryError::Semantic(format!(
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
            Some(if rel_is_repeat {
                namer.name(&None)
            } else {
                namer.name(&rel.var)
            })
        } else {
            // Neither `rel.var` shape a variable-length hop can have here
            // -- `name_pattern_for_path`'s own internal path-segment
            // binding (`capture_path_segment`), or the user's own real
            // relationship-*list* variable (`rel_list_var` below) -- names
            // a single `Binding::Edge` the way an ordinary fixed hop's
            // `rel_filter_var` would, so this must stay `None` either way
            // (the props/edge-isomorphism `Filter`s just below both
            // assume `rel_var` names a `Binding::Edge`).
            None
        };
        plan = match rel.hop_range {
            None => LogicalPlan::Expand {
                input: Box::new(plan),
                from_var: from_var.clone(),
                to_var: to_var.clone(),
                rel_var: rel_filter_var.clone(),
                rel_labels: rel.rel_types.clone(),
                direction,
            },
            // `MATCH (a)-[rs*]->(b)` where `rs` is *already* bound (e.g.
            // `WITH [r1, r2] AS rs`) -- see `LogicalPlan::MatchRelList`'s
            // own docs for why this is a distinct, deterministic
            // "verify the chain" plan node rather than `VarExpand`'s
            // fresh BFS (TCK's Match4 `[8]`, Match9 `[6]`/`[7]`).
            Some((min_hops, max_hops)) if rel_is_repeat && !rel.capture_path_segment => {
                LogicalPlan::MatchRelList {
                    input: Box::new(plan),
                    from_var: from_var.clone(),
                    to_var: to_var.clone(),
                    rel_list_var: rel
                        .var
                        .clone()
                        .expect("rel_is_repeat implies rel.var is Some"),
                    rel_labels: rel.rel_types.clone(),
                    direction,
                    min_hops,
                    max_hops,
                }
            }
            Some((min_hops, max_hops)) => {
                // Always synthesized, regardless of whether this hop's
                // own path/list capture was requested -- see
                // `exclude_edge_var`'s own docs on `LogicalPlan::
                // VarExpand`.
                let exclude_edge_var = namer.name(&None);
                let plan = LogicalPlan::VarExpand {
                    input: Box::new(plan),
                    from_var: from_var.clone(),
                    to_var: to_var.clone(),
                    rel_labels: rel.rel_types.clone(),
                    direction,
                    min_hops,
                    max_hops,
                    exclude_edge_vars: prior_rel_vars.clone(),
                    exclude_edge_sets: prior_edge_sets.clone(),
                    exclude_edge_var: exclude_edge_var.clone(),
                    path_segment_var: rel.capture_path_segment.then(|| {
                        rel.var
                            .clone()
                            .expect("name_pattern_for_path always sets rel.var alongside capture_path_segment")
                    }),
                    // The user's own `[r:TYPE*1..3]` -- a real Cypher
                    // relationship-list binding (TCK's Match4 `[1]`/`[6]`,
                    // Match9 `[9]`). Not `rel.var` itself when
                    // `capture_path_segment` is set -- that field holds
                    // this hop's own internal path-segment bookkeeping
                    // name in that case instead (see `RelPattern::
                    // rel_list_var`'s own docs) -- but the two aren't
                    // mutually exclusive: a hop can have both a named-path
                    // capture *and* its own real rel-list variable at once.
                    rel_list_var: if rel.capture_path_segment {
                        rel.rel_list_var.clone()
                    } else {
                        rel.var.clone()
                    },
                    rel_props: rel.props.clone(),
                };
                // Propagate forward -- a *later* hop (fixed, via a new
                // `Expr::EdgeNotInSet` `Filter` below, or another
                // `VarExpand`, via its own `exclude_edge_sets`) must
                // exclude whatever this row's traversal happened to use
                // (TCK's Match4 `[7]`, `mars-pbp`).
                prior_edge_sets.push(exclude_edge_var);
                plan
            }
        };
        // Hop nodes reach this point via Expand/VarExpand, which don't
        // pre-filter by label at all (unlike the start node's
        // NodeByLabelScan) — every listed label must be Filter-checked
        // here, not just the extras beyond the first.
        plan = wrap_labels_and_props(plan, &to_var, node, 0)?;
        if let Some(rel_var) = &rel_filter_var {
            for (key, expr) in &rel.props {
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: pattern_prop_predicate(rel_var, key, expr),
                };
            }
            for prior in &prior_rel_vars {
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: Expr::Not(Box::new(Expr::VarEq(rel_var.clone(), prior.clone()))),
                };
            }
            // Complementary direction: this fixed hop's own edge must not
            // be one an *earlier variable-length* hop of this same pattern
            // already traversed (TCK's Match4 `[7]`, `mars-pbp`).
            for prior_set in &prior_edge_sets {
                plan = LogicalPlan::Filter {
                    input: Box::new(plan),
                    predicate: Expr::EdgeNotInSet {
                        edge_var: rel_var.clone(),
                        edge_set_var: prior_set.clone(),
                    },
                };
            }
            if rel_is_repeat {
                let original = rel
                    .var
                    .clone()
                    .expect("rel_is_repeat implies rel.var is Some");
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
            let original = node
                .var
                .clone()
                .expect("is_repeat implies node.var is Some");
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

fn scan_for(var: &str, node: &NodePattern) -> Result<LogicalPlan, QueryError> {
    // The first label (if any) narrows the scan; any additional labels
    // (`(n:Post:Message)`) become extra HasLabel filters — a node must
    // have ALL listed labels, matching Cypher's multi-label AND semantics.
    let base = match node.labels.first() {
        Some(label) => LogicalPlan::NodeByLabelScan {
            var: var.to_string(),
            label: label.clone(),
        },
        None => LogicalPlan::AllNodesScan {
            var: var.to_string(),
        },
    };
    // Skip the first label — NodeByLabelScan above already selected for it.
    wrap_labels_and_props(base, var, node, 1)
}

/// Inline node-pattern properties (`(a:Person {name:'Alice'})`) and any
/// labels not already handled by a preceding scan (`skip` labels from the
/// front) compile to the same Filter machinery as a WHERE clause, just
/// synthesized from the pattern -- see `pattern_prop_predicate`'s own
/// docs for the literal-vs-computed split.
fn wrap_labels_and_props(
    plan: LogicalPlan,
    var: &str,
    node: &NodePattern,
    skip: usize,
) -> Result<LogicalPlan, QueryError> {
    let mut plan = plan;
    for label in node.labels.iter().skip(skip) {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: Expr::HasLabel(var.to_string(), label.clone()),
        };
    }
    for (key, expr) in &node.props {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: pattern_prop_predicate(var, key, expr),
        };
    }
    Ok(plan)
}

/// Splits `expr` into a flat list of top-level `AND`-conjuncts, appended
/// to `out` — `And(l, r)` decomposes both sides recursively; anything
/// else (a single `Compare`, `Or`, `Not`, ...) is one conjunct as-is.
/// Used by `apply_index_seeks` to find every equality candidate in a
/// `WHERE a = 1 AND b = 2`-shaped predicate, not just a bare single
/// comparison.
fn push_conjuncts(expr: Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::And(l, r) => {
            push_conjuncts(*l, out);
            push_conjuncts(*r, out);
        }
        other => out.push(other),
    }
}

/// `push_conjuncts`'s inverse — folds a conjunct list back into one `And`
/// tree (`None` for an empty list, meaning "no predicate left to
/// enforce" — the whole thing became one equality that's now satisfied
/// by an `IndexSeek` instead).
fn rebuild_and(mut exprs: Vec<Expr>) -> Option<Expr> {
    let first = exprs.pop()?;
    Some(
        exprs
            .into_iter()
            .fold(first, |acc, e| Expr::And(Box::new(e), Box::new(acc))),
    )
}

/// A `MATCH`/`MERGE` pattern's inline `{key: value}` -- a plain literal
/// compiles to the narrow `Expr::Compare(PropAccess, Eq, Literal)` shape
/// (the only one `apply_index_seeks` recognizes, so this is what keeps a
/// literal pattern property index-seek-eligible), anything else (a bound
/// variable, a function call, ...) compiles to `Expr::GeneralCompare`
/// instead -- a generic post-scan filter, never index-seek-eligible, but
/// evaluated per-row against the row's own bindings via
/// `Executor::eval_expr` (real Cypher fully supports this, e.g. `WITH 42
/// AS var MERGE (c:N {var: var})`, TCK's Merge1 [8] -- an earlier version
/// of this codebase rejected it outright at plan-build time, which was
/// wrong, not a real Cypher restriction).
fn pattern_prop_predicate(var: &str, key: &str, expr: &ReturnExpr) -> Expr {
    let access = PropAccess {
        var: var.to_string(),
        prop: key.to_string(),
    };
    match expr {
        ReturnExpr::Lit(lit) => Expr::Compare(access, CompareOp::Eq, lit.clone()),
        other => Expr::GeneralCompare(ReturnExpr::Prop(access), CompareOp::Eq, other.clone()),
    }
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
        // A named-path-captured hop's own real rel-list variable, if it
        // had one (`p = (a)-[r*]->(b)`) -- `rel.var` itself holds this
        // hop's internal path-segment bookkeeping name in that case
        // instead, see `RelPattern::rel_list_var`'s own docs.
        if let Some(v) = &rel.rel_list_var {
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

/// Post-processing pass over an already-built plan: fuses a
/// `Filter(Compare(var.prop = literal))` sitting directly over a
/// `NodeByLabelScan{var, label}` into a single `IndexSeek`, if a real
/// index happens to be declared on `(label, prop)` — checked against
/// `txn`, since `build_match_plan` itself has no storage access and can't
/// know. Deliberately narrow for this first pass: only the *exact* shape
/// a `MATCH (n:Label {prop: literal})` pattern property compiles to (see
/// `wrap_labels_and_props`) is recognized — a `WHERE`-clause equality
/// predicate reaching the same shape through a different path, or an
/// index candidate buried under an `Expand`, is `rule-based pushdown`'s
/// job (a separate, later change), not this fusion's.
pub fn apply_index_seeks(plan: LogicalPlan, txn: Txn) -> Result<LogicalPlan, QueryError> {
    Ok(match plan {
        LogicalPlan::Filter { .. } => {
            // Peel every directly-nested `Filter` down to whatever
            // non-`Filter` node sits underneath, flattening each level's
            // predicate into one flat conjunct list. Both an inline
            // pattern property (`{prop: literal}`) and a `WHERE` clause
            // compile to the identical `Filter{predicate: Expr::Compare}`
            // shape (see `wrap_labels_and_props`/`build_match_plan`), so
            // an equality on either one -- or on either side of a `WHERE
            // a = 1 AND b = 2`, which is one `Filter` with an `And`
            // predicate, not two nested `Filter`s -- is an equally valid
            // index-seek candidate. Only a *directly* nested chain is
            // peeled; a conjunct sitting past an `Expand`/`VarExpand`
            // belongs to a different node's scan, not this one's.
            let mut node = plan;
            let mut candidates = Vec::new();
            let base = loop {
                match node {
                    LogicalPlan::Filter { input, predicate } => {
                        push_conjuncts(predicate, &mut candidates);
                        node = *input;
                    }
                    other => break other,
                }
            };
            let base = apply_index_seeks(base, txn)?;
            if let LogicalPlan::NodeByLabelScan { var, label } = &base {
                // Among every `var.prop = literal` equality conjunct that
                // *has* a declared index, pick the one with the smallest
                // cheap cardinality estimate (`index_match_count_in_txn`,
                // O(1) via redb's per-key entry count) -- not just the
                // first syntactically. Two candidate indexes rarely narrow
                // equally well (e.g. `WHERE country = 'US' AND email =
                // 'x@y.com'` -- `email` is far more selective), and an
                // `IndexSeek` reading fewer entries is strictly cheaper, so
                // this is a real cost comparison, not a guess. Ties (equal
                // counts, including the common "both empty/unbacked" case)
                // keep the first-encountered candidate for determinism.
                let mut chosen: Option<(usize, u64)> = None;
                for (i, c) in candidates.iter().enumerate() {
                    let Expr::Compare(pa, CompareOp::Eq, lit) = c else {
                        continue;
                    };
                    if pa.var != *var || matches!(lit, Literal::Param(_)) {
                        continue;
                    }
                    if GraphStore::index_def_in_txn(txn, label, &pa.prop)?.is_some() {
                        let value = literal_to_value(lit);
                        let count =
                            GraphStore::index_match_count_in_txn(txn, label, &pa.prop, &value)?;
                        if chosen.is_none_or(|(_, best)| count < best) {
                            chosen = Some((i, count));
                        }
                    }
                }
                if let Some((i, _)) = chosen {
                    let Expr::Compare(pa, _, lit) = candidates.remove(i) else {
                        unreachable!("chosen index always points at a Compare, checked above")
                    };
                    let seek = LogicalPlan::IndexSeek {
                        var: var.clone(),
                        label: label.clone(),
                        prop: pa.prop,
                        value: literal_to_value(&lit),
                    };
                    return Ok(match rebuild_and(candidates) {
                        Some(predicate) => LogicalPlan::Filter {
                            input: Box::new(seek),
                            predicate,
                        },
                        None => seek,
                    });
                }
            }
            match rebuild_and(candidates) {
                Some(predicate) => LogicalPlan::Filter {
                    input: Box::new(base),
                    predicate,
                },
                None => base,
            }
        }
        LogicalPlan::Expand {
            input,
            from_var,
            to_var,
            rel_var,
            rel_labels,
            direction,
        } => LogicalPlan::Expand {
            input: Box::new(apply_index_seeks(*input, txn)?),
            from_var,
            to_var,
            rel_var,
            rel_labels,
            direction,
        },
        LogicalPlan::VarExpand {
            input,
            from_var,
            to_var,
            rel_labels,
            direction,
            min_hops,
            max_hops,
            exclude_edge_vars,
            exclude_edge_sets,
            exclude_edge_var,
            path_segment_var,
            rel_list_var,
            rel_props,
        } => LogicalPlan::VarExpand {
            input: Box::new(apply_index_seeks(*input, txn)?),
            from_var,
            to_var,
            rel_labels,
            direction,
            min_hops,
            max_hops,
            exclude_edge_vars,
            exclude_edge_sets,
            exclude_edge_var,
            path_segment_var,
            rel_list_var,
            rel_props,
        },
        LogicalPlan::MatchRelList {
            input,
            from_var,
            to_var,
            rel_list_var,
            rel_labels,
            direction,
            min_hops,
            max_hops,
        } => LogicalPlan::MatchRelList {
            input: Box::new(apply_index_seeks(*input, txn)?),
            from_var,
            to_var,
            rel_list_var,
            rel_labels,
            direction,
            min_hops,
            max_hops,
        },
        leaf @ (LogicalPlan::AllNodesScan { .. }
        | LogicalPlan::NodeByLabelScan { .. }
        | LogicalPlan::Seed { .. }
        | LogicalPlan::IndexSeek { .. }) => leaf,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use marsdb_graph::{GraphStore, PropertyValue, Txn};

    use super::*;
    use crate::ast::{QueryClause, Statement};

    fn pattern_from(cypher: &str) -> crate::ast::Pattern {
        part_from(cypher).pattern
    }

    fn part_from(cypher: &str) -> crate::ast::QueryPart {
        let Statement::Match { clauses, .. } = crate::antlr_visitor::parse_antlr(cypher).unwrap()
        else {
            panic!("expected a Match statement");
        };
        let QueryClause::Match(part) = clauses.into_iter().next().unwrap() else {
            panic!("expected a Match clause");
        };
        part
    }

    #[test]
    fn fuses_node_pattern_property_into_index_seek_when_an_index_exists() {
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Person", "email", false).unwrap();
        let pattern = pattern_from("MATCH (n:Person {email: 'alice@x.com'}) RETURN n");

        let write = store.begin_write().unwrap();
        let plan = build_match_plan(&pattern, &None, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::IndexSeek {
                var,
                label,
                prop,
                value,
            } => {
                assert_eq!(var, "n");
                assert_eq!(label, "Person");
                assert_eq!(prop, "email");
                assert_eq!(value, PropertyValue::String("alice@x.com".to_string()));
            }
            other => panic!("expected an IndexSeek, got {other:?}"),
        }
    }

    #[test]
    fn falls_back_to_filter_over_scan_when_no_index_exists() {
        let store = GraphStore::open_memory().unwrap();
        let pattern = pattern_from("MATCH (n:Person {email: 'alice@x.com'}) RETURN n");

        let write = store.begin_write().unwrap();
        let plan = build_match_plan(&pattern, &None, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::Filter { input, .. } => {
                assert!(matches!(*input, LogicalPlan::NodeByLabelScan { .. }));
            }
            other => panic!("expected a Filter over a scan, got {other:?}"),
        }
    }

    #[test]
    fn fuses_a_where_clause_equality_into_index_seek() {
        // Unlike an inline pattern property, a WHERE-clause equality
        // compiles to a *separate* outer Filter wrapping the scan --
        // apply_index_seeks must still find it.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Person", "email", false).unwrap();
        let part = part_from("MATCH (n:Person) WHERE n.email = 'alice@x.com' RETURN n");

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::IndexSeek {
                var,
                label,
                prop,
                value,
            } => {
                assert_eq!(var, "n");
                assert_eq!(label, "Person");
                assert_eq!(prop, "email");
                assert_eq!(value, PropertyValue::String("alice@x.com".to_string()));
            }
            other => panic!("expected an IndexSeek, got {other:?}"),
        }
    }

    #[test]
    fn seeks_one_equality_and_keeps_the_other_conjunct_as_a_residual_filter() {
        // `WHERE email = 'x' AND age > 35` -- only `email` has an index,
        // so the seek must fire for it while `age > 35` survives as a
        // Filter wrapping the seek, not get silently dropped.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Person", "email", false).unwrap();
        let part =
            part_from("MATCH (n:Person) WHERE n.email = 'alice@x.com' AND n.age > 35 RETURN n");

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::Filter { input, predicate } => {
                assert!(
                    matches!(*input, LogicalPlan::IndexSeek { .. }),
                    "expected the seek underneath"
                );
                match predicate {
                    Expr::Compare(pa, CompareOp::Gt, Literal::Int(35)) => {
                        assert_eq!(pa.prop, "age")
                    }
                    other => panic!("expected the residual age > 35 predicate, got {other:?}"),
                }
            }
            other => panic!("expected a residual Filter over an IndexSeek, got {other:?}"),
        }
    }

    #[test]
    fn picks_the_more_selective_index_when_multiple_equality_candidates_are_indexed() {
        // `country = 'US'` matches most of the graph, `email = '...'`
        // matches exactly one node -- both have declared indexes, so the
        // cardinality-based choice must seek on `email`, not just take
        // whichever conjunct appears first in the WHERE clause.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Person", "country", false).unwrap();
        store.create_index("Person", "email", false).unwrap();
        for i in 0..20 {
            let mut props = BTreeMap::new();
            props.insert(
                "country".to_string(),
                PropertyValue::String("US".to_string()),
            );
            props.insert(
                "email".to_string(),
                PropertyValue::String(format!("user{i}@x.com")),
            );
            store.create_node(&["Person"], props).unwrap();
        }
        let part = part_from(
            "MATCH (n:Person) WHERE n.country = 'US' AND n.email = 'user7@x.com' RETURN n",
        );

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::Filter { input, predicate } => {
                match *input {
                    LogicalPlan::IndexSeek { prop, value, .. } => {
                        assert_eq!(prop, "email");
                        assert_eq!(value, PropertyValue::String("user7@x.com".to_string()));
                    }
                    other => panic!("expected the seek underneath, got {other:?}"),
                }
                match predicate {
                    Expr::Compare(pa, CompareOp::Eq, Literal::String(s)) => {
                        assert_eq!(pa.prop, "country");
                        assert_eq!(s, "US");
                    }
                    other => panic!("expected the residual country predicate, got {other:?}"),
                }
            }
            other => panic!("expected a residual Filter over an IndexSeek, got {other:?}"),
        }
    }
}
