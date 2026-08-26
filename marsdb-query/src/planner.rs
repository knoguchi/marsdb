use std::collections::HashSet;

use marsdb_graph::{GraphStore, PropertyValue, Txn};

use crate::ast::{
    CompareOp, Expr, Literal, NodePattern, Pattern, PropAccess, RelDirection, ReturnExpr,
};
use crate::error::QueryError;
use crate::executor::literal_to_value;
use crate::ir::{ExpandDirection, IndexSeekValue, LogicalPlan};

struct VarNamer {
    next: usize,
}

impl VarNamer {
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

/// Relationship-uniqueness state shared by the comma-separated parts of
/// one `MATCH` clause (see `QueryPart::continues_clause`). Real Cypher's
/// edge-isomorphism rule spans the whole clause pattern: `MATCH
/// (p)-[:T]->(a), (p)-[:T]->(b)` must not bind both hops to the same
/// relationship instance, exactly as if the parts were hops of one chain.
/// Carries forward, part to part: the synthesized-name counter (so a
/// later part's `__anonN` names can't collide with — and silently
/// overwrite — an earlier part's still-in-row bindings), every earlier
/// part's bound relationship vars (fixed hops) and traversed-edge-set
/// vars (variable-length hops), and the user-written relationship
/// variable names (a name may not repeat across parts of one clause,
/// same compile-time error as within one pattern). A separate `MATCH`
/// clause starts a fresh scope — reuse across clauses stays legal.
#[derive(Default)]
pub struct MatchClauseScope {
    namer_next: usize,
    prior_rel_vars: Vec<String>,
    prior_edge_sets: Vec<String>,
    rel_var_names: HashSet<String>,
}

pub fn build_match_plan(
    pattern: &Pattern,
    where_clause: &Option<Expr>,
    carried_vars: &HashSet<String>,
) -> Result<LogicalPlan, QueryError> {
    // Standalone pattern (a whole clause of its own, or a context like a
    // pattern predicate / MERGE that never has comma-separated parts):
    // fresh, discarded scope.
    build_match_plan_scoped(
        pattern,
        where_clause,
        carried_vars,
        &mut MatchClauseScope::default(),
    )
}

pub fn build_match_plan_scoped(
    pattern: &Pattern,
    where_clause: &Option<Expr>,
    carried_vars: &HashSet<String>,
    scope: &mut MatchClauseScope,
) -> Result<LogicalPlan, QueryError> {
    let mut namer = VarNamer {
        next: scope.namer_next,
    };
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
    // Push WHERE-clause conjuncts that depend *only* on the start node down
    // to wrap its scan directly, rather than leaving every conjunct in the
    // one big Filter this function otherwise wraps around the *whole*
    // pattern (every hop's Expand included) at the very end. Without this,
    // a multi-hop pattern's `WHERE start.prop = <literal>` sits above every
    // Expand, so `apply_index_seeks` (which only looks at what's
    // *immediately* under a Filter) never reaches the NodeByLabelScan it
    // should rewrite — real difference between `MATCH (a {prop: 'x'})-->()`
    // (inline property, already index-seek-eligible before this fix) and
    // the equivalent `MATCH (a)-->() WHERE a.prop = 'x'`.
    let mut where_conjuncts = Vec::new();
    if let Some(expr) = where_clause {
        push_conjuncts(expr.clone(), &mut where_conjuncts);
    }
    let mut start_only = Vec::new();
    where_conjuncts.retain(|c| {
        if conjunct_sole_var(c) == Some(start_var.as_str()) {
            start_only.push(c.clone());
            false
        } else {
            true
        }
    });
    if let Some(predicate) = rebuild_and(start_only) {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate,
        };
    }
    let mut from_var = start_var.clone();
    // Real Cypher pattern matching is edge-isomorphic: no single MATCH
    // clause pattern may bind two hops to the *same* relationship
    // instance, even if their types/directions differ (a self-loop plus
    // an undirected hop back out is the case that surfaces this — without
    // this check, the hop back out can silently re-match the edge the
    // previous hop just came in on). Scoped to the whole MATCH clause:
    // hops within this pattern, plus every hop of an earlier
    // comma-separated part of the same clause (seeded from `scope`, see
    // `MatchClauseScope`). Only a separate MATCH *clause* may reuse the
    // same relationship freely.
    let mut prior_rel_vars: Vec<String> = std::mem::take(&mut scope.prior_rel_vars);
    // Complementary to `prior_rel_vars` above -- edges an *earlier
    // variable-length* hop of this same pattern traversed can't be named
    // by a single id the way a fixed hop's own `rel_var` can (each row's
    // own BFS can use a different set of edges), so this tracks each such
    // hop's own `exclude_edge_var` name instead (see `LogicalPlan::
    // VarExpand::exclude_edge_sets`'s own docs).
    let mut prior_edge_sets: Vec<String> = std::mem::take(&mut scope.prior_edge_sets);
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
    let mut pattern_rel_var_names: HashSet<String> = std::mem::take(&mut scope.rel_var_names);
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
                // (TCK's Match4 `[7]`).
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
            // already traversed (TCK's Match4 `[7]`).
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
    if let Some(predicate) = rebuild_and(where_conjuncts) {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate,
        };
    }
    // Hand the accumulated uniqueness state back for the clause's next
    // comma-separated part (a no-op for the standalone-`build_match_plan`
    // wrapper, whose scope is discarded).
    scope.namer_next = namer.next;
    scope.prior_rel_vars = prior_rel_vars;
    scope.prior_edge_sets = prior_edge_sets;
    scope.rel_var_names = pattern_rel_var_names;
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

/// The single variable this conjunct exclusively depends on, if pushing it
/// down to wrap that variable's own scan directly (rather than leaving it
/// in the Filter that wraps the *whole* pattern, at the very end of
/// `build_match_plan`) is provably safe. Deliberately narrow: only the
/// simple leaf shapes already known to reference exactly the variable(s)
/// named in them — a conjunct this doesn't recognize (`And`/`Or`/`VarEq`,
/// a `PropCompare`/`GeneralCompare` naming two *different* variables,
/// pattern predicates, ...) returns `None`, leaving it exactly where it
/// already was rather than guessing.
fn conjunct_sole_var(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Compare(pa, _, _) | Expr::IsNull(pa) => Some(&pa.var),
        Expr::HasLabel(var, _) => Some(var),
        Expr::PropCompare(l, _, r) if l.var == r.var => Some(&l.var),
        Expr::GeneralCompare(ReturnExpr::Prop(pa), _, other)
            if !return_expr_references_var(other, &pa.var) =>
        {
            Some(&pa.var)
        }
        Expr::GeneralIsNull(ReturnExpr::Prop(pa)) => Some(&pa.var),
        _ => None,
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
/// Conservative "could this expression's value depend on `var`" check —
/// used by `apply_index_seeks` to confirm a `GeneralCompare` conjunct's
/// non-scanned side is safe to evaluate once per seed row (not once per
/// candidate node `var` could bind to). Recurses through the two shapes
/// real bulk-load data actually produces (`row.field`, and `row.a.b` —
/// `PropOf`'s nested-base case, e.g. APOC's own exported `row.start.movieId`
/// shape); anything else (a function call, arithmetic, `CASE`, ...) is
/// treated as "might reference it," not walked further, so the caller
/// just declines to promote rather than risking a wrong answer.
fn return_expr_references_var(expr: &ReturnExpr, var: &str) -> bool {
    match expr {
        ReturnExpr::Var(v) => v == var,
        ReturnExpr::Prop(pa) => pa.var == var,
        ReturnExpr::PropOf(base, _) => return_expr_references_var(base, var),
        ReturnExpr::Lit(_) | ReturnExpr::CountStar => false,
        // A function call references `var` iff any argument does -- so
        // `date('2020-01-10')` (the shape a `$param`-substituted temporal
        // equality takes, mars-9ez) is promotable while `date(n.born)`
        // correctly isn't. `rand()` is the one argument-free call whose
        // *value* still can't be hoisted from per-candidate to
        // per-seed-row evaluation (a fresh number each call is the whole
        // point of it), so it's treated as referencing everything; a
        // rand() nested deeper inside an argument hits this same arm
        // through the recursion. The temporal now-functions (`date()`,
        // `timestamp()`, ...) are NOT excluded: they're pinned to one
        // per-statement `NowSnapshot`, so per-seed-row evaluation returns
        // the identical value per-candidate evaluation would.
        ReturnExpr::Call { name, args, .. } => {
            name.eq_ignore_ascii_case("rand")
                || args.iter().any(|a| return_expr_references_var(a, var))
        }
        _ => true,
    }
}

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

/// Start-point selection: decide whether the pattern's traversal should
/// begin from its *last* endpoint instead of its first, and if so return
/// the reversed pattern (each hop's direction flipped, node order
/// reversed) for `build_match_plan` to compile as usual. `MATCH
/// (a:Common)-->(b:Rare {id: 1}) ...` written from the `Common` side
/// otherwise scans every `Common` node and expands, when starting from
/// the one indexed `Rare` node and expanding backwards touches only the
/// matching rows — the plan is direction-symmetric (`ADJ_IN` mirrors
/// `ADJ_OUT`), so which endpoint seeds the traversal is a pure cost
/// choice with identical results.
///
/// The decision is a two-sided cost estimate, all O(1) statistics
/// (`label_count_in_txn`/`node_count_in_txn`/`rel_type_count_in_txn`):
///
/// ```text
/// cost(anchor A, other B) = scan_rows_A · (1 + filtered_A)
///                         + E_A         · (1 + filtered_B)
/// E_A = E · out_rows_A / label_rows_A
/// ```
///
/// where `scan_rows` is what the anchor's leaf physically visits (0 for
/// a bound Seed, the index-match count when an indexed literal-equality
/// candidate turns the scan into a seek — the same candidates
/// `apply_index_seeks` would fuse — else the full label count),
/// `out_rows` is what the leaf *emits* after its pushed filters (equal
/// to `scan_rows` for a seek; an *unindexed* literal equality keeps the
/// scan full but credits the emitted rows with a default 1/10
/// selectivity — see `UNINDEXED_EQ_SELECTIVITY_DIVISOR`; other
/// uncredited filters stay at selectivity 1, their true selectivity
/// genuinely unknowable here), `E` is the live edge count of the
/// anchor-adjacent hop's relationship type(s), and `filtered` marks
/// pushable predicate work priced per scanned row (a `CONTAINS`, a
/// range, an unindexed or `$param` equality). The terms are the
/// traversal's real work items: visit `scan_rows_A` leaf rows; evaluate
/// the anchor's own filter once per visited row (`scan_rows_A ·
/// filtered_A` — pushed below the Expand by `build_match_plan`, this is
/// what anchoring *at* the filtered side buys); walk `E_A` edges, the
/// type's total prorated by the fraction of the label the leaf emits
/// (uniform-degree assumption); evaluate the *other* endpoint's
/// stranded filter once per walked edge (`E_A · filtered_B`). Row
/// scans, filter evaluations, and edge walks are weighted equally —
/// measured on the recommendations dataset at ~0.66µs and ~0.65µs per
/// item for the filter-eval and edge-walk halves, same order.
///
/// Splitting `scan_rows` from `out_rows` is what lets an unindexed
/// equality price correctly on *both* sides of the trade: the emitted-
/// row credit stops a huge equality-filtered label from dragging the
/// whole edge population into its estimate (`(m:Post {id: 100})-->
/// (p:Person)` must anchor at Post even though Person's label is
/// smaller — issue #208, measured 6x there and 70x on a variant whose
/// far endpoint was unlabeled), while the still-full scan term keeps a
/// low-selectivity equality against a tiny far endpoint reversing
/// exactly as before.
///
/// With no filters anywhere the model degenerates to the old plain row
/// comparison (both sides carry the same full `E`). The benchmark query
/// that motivated the `filtered` term (`MATCH (m:Movie)<-[:RATED]-
/// (u:User) WHERE m.title CONTAINS '...'`, 671 users vs 9,125 movies,
/// 100k RATED edges, 3 title matches) prices as 118k written vs 200k
/// reversed — written order, the measured-9x-faster answer. The shape
/// the interim always-decline rule got wrong — a huge filtered label
/// against a tiny far endpoint with few edges — now reverses, because a
/// small `E` caps how much the stranded filter can cost.
///
/// Reversal fires only when the far endpoint prices strictly cheaper —
/// ties keep written order, both for determinism and because reversal
/// is never free to reason about.
///
/// Deliberately conservative, same stance as every other planner pass:
/// only all-fixed-hop patterns are considered. A variable-length hop's
/// own relationship-list binding (`[r*1..3]`) and named-path capture
/// both expose traversal *order* to the user, which reversal would flip;
/// rather than distinguishing the observable cases, any `hop_range` in
/// the pattern disqualifies it. Callers additionally skip named-path
/// (`p = ...`) and `shortestPath` clauses for the same reason.
pub fn plan_reversed_pattern(
    pattern: &Pattern,
    where_clause: &Option<Expr>,
    carried_vars: &HashSet<String>,
    txn: Txn,
) -> Result<Option<Pattern>, QueryError> {
    if pattern.hops.is_empty() {
        return Ok(None);
    }
    if pattern.hops.iter().any(|(rel, _)| rel.hop_range.is_some()) {
        return Ok(None);
    }
    let mut conjuncts = Vec::new();
    if let Some(expr) = where_clause {
        push_conjuncts(expr.clone(), &mut conjuncts);
    }
    let (first_rel, _) = &pattern.hops[0];
    let (last_rel, end) = pattern.hops.last().expect("hops checked non-empty");
    let start_cost = endpoint_start_cost(&pattern.start, &conjuncts, carried_vars, txn)?;
    let end_cost = endpoint_start_cost(end, &conjuncts, carried_vars, txn)?;
    // Each side's expand term uses its own adjacent hop's edge count —
    // for a single-hop pattern they're the same hop; for multi-hop, the
    // hop the anchor would walk first (later hops are unmodeled, the
    // same first-hop-dominates approximation the rest of this pass
    // already makes).
    let start_edges = rel_types_edge_count(txn, &first_rel.rel_types)?;
    let end_edges = rel_types_edge_count(txn, &last_rel.rel_types)?;
    if anchor_cost(&end_cost, &start_cost, end_edges)
        < anchor_cost(&start_cost, &end_cost, start_edges)
    {
        Ok(Some(reverse_pattern(pattern)))
    } else {
        Ok(None)
    }
}

/// Third start strategy alongside written/reversed order: bind the
/// whole single-hop pattern from one sequential `EDGES`-table sweep
/// (`LogicalPlan::EdgeTypeScan`) instead of scanning an endpoint and
/// expanding adjacency. Wins when a relationship-property predicate
/// exists (evaluable from the swept record's own bytes -- the whole
/// point: no per-edge storage get) and the O(1) total edge count is
/// smaller than the best anchored estimate. Measured basis: a warm
/// sequential sweep of 166k edge records incl. per-record predicate
/// decode costs ~5-6ms, vs ~110ms for the same edges through
/// per-edge adjacency gets (the recommendations bulk-delete shape).
///
/// Eligibility is deliberately narrow (same conservatism as reversal):
/// exactly one fixed hop, a written direction (`Either`'s dedup
/// semantics excluded), all three variables named and fresh (no
/// carried vars, no self-reference), no inline endpoint props, at most
/// one label per endpoint, and at least one scan-evaluable conjunct on
/// the relationship variable (`Compare` vs a non-param literal,
/// `IS NULL`, `IS NOT NULL` -- exactly the shapes the executor
/// evaluates from raw bytes with `value_cmp::compare` semantics).
/// Everything else stays in the residual `Filter` the returned plan is
/// wrapped in.
pub fn plan_edge_scan(
    pattern: &Pattern,
    where_clause: &Option<Expr>,
    carried_vars: &HashSet<String>,
    txn: Txn,
) -> Result<Option<LogicalPlan>, QueryError> {
    if pattern.hops.len() != 1 {
        return Ok(None);
    }
    let (rel, end) = &pattern.hops[0];
    if rel.hop_range.is_some() {
        return Ok(None);
    }
    let (Some(start_var), Some(rel_var), Some(end_var)) = (
        pattern.start.var.as_ref(),
        rel.var.as_ref(),
        end.var.as_ref(),
    ) else {
        return Ok(None);
    };
    if start_var == end_var
        || rel_var == start_var
        || rel_var == end_var
        || carried_vars.contains(start_var)
        || carried_vars.contains(rel_var)
        || carried_vars.contains(end_var)
    {
        return Ok(None);
    }
    if !pattern.start.props.is_empty() || !end.props.is_empty() {
        return Ok(None);
    }
    if pattern.start.labels.len() > 1 || end.labels.len() > 1 {
        return Ok(None);
    }
    let (src_var, dst_var, src_label, dst_label) = match rel.direction {
        RelDirection::Right => (
            start_var,
            end_var,
            pattern.start.labels.first(),
            end.labels.first(),
        ),
        RelDirection::Left => (
            end_var,
            start_var,
            end.labels.first(),
            pattern.start.labels.first(),
        ),
        RelDirection::Either => return Ok(None),
    };

    let mut conjuncts = Vec::new();
    if let Some(expr) = where_clause {
        push_conjuncts(expr.clone(), &mut conjuncts);
    }
    let (scan_preds, residual): (Vec<Expr>, Vec<Expr>) = conjuncts
        .clone()
        .into_iter()
        .partition(|c| edge_scan_evaluable(c, rel_var));
    if scan_preds.is_empty() {
        return Ok(None);
    }

    // Cost gate. The two sides' units are NOT equal work: a sweep unit
    // is one sequential record visit with the predicate read off bytes
    // already in hand (measured ~35ns/record: 166k records in ~5.8ms,
    // warm, release), while an anchored unit is a random adjacency walk
    // whose predicate needs a per-edge storage get (measured ~1.1us:
    // the same shape's ~110ms match phase over 100k edges) -- roughly
    // 30x apart on the reference dataset. Gated at a conservative 8x
    // (understating the sweep's advantage several-fold) so the sweep
    // only fires where the measured gap can't plausibly invert;
    // strictly-less keeps ties on the tried-and-true path.
    const SWEEP_UNIT_ADVANTAGE: u64 = 8;
    let e_total = GraphStore::edge_count_in_txn(txn)?;
    let start_cost = endpoint_start_cost(&pattern.start, &conjuncts, carried_vars, txn)?;
    let end_cost = endpoint_start_cost(end, &conjuncts, carried_vars, txn)?;
    let edges = rel_types_edge_count(txn, &rel.rel_types)?;
    let anchored =
        anchor_cost(&start_cost, &end_cost, edges).min(anchor_cost(&end_cost, &start_cost, edges));
    if e_total >= anchored.saturating_mul(SWEEP_UNIT_ADVANTAGE) {
        return Ok(None);
    }

    let leaf = LogicalPlan::EdgeTypeScan {
        src_var: src_var.clone(),
        rel_var: rel_var.clone(),
        dst_var: dst_var.clone(),
        rel_types: rel.rel_types.clone(),
        src_label: src_label.cloned(),
        dst_label: dst_label.cloned(),
        rel_predicate: rebuild_and(scan_preds),
    };
    Ok(Some(match rebuild_and(residual) {
        Some(predicate) => LogicalPlan::Filter {
            input: Box::new(leaf),
            predicate,
        },
        None => leaf,
    }))
}

/// Which conjuncts the `EdgeTypeScan` stream can decide from the raw
/// record: definite-answer shapes only (`Not` is admitted solely over
/// `IS NULL` -- negating a three-valued `Compare` whose unknown
/// collapses to false would flip unknowns to true, which Cypher
/// forbids).
fn edge_scan_evaluable(conjunct: &Expr, rel_var: &str) -> bool {
    if conjunct_sole_var(conjunct) != Some(rel_var) {
        return false;
    }
    match conjunct {
        Expr::Compare(_, _, lit) => !matches!(lit, Literal::Param(_)),
        Expr::IsNull(_) => true,
        Expr::Not(inner) => matches!(inner.as_ref(), Expr::IsNull(_)),
        _ => false,
    }
}

/// Live edge count for one hop's relationship-type list: sum of the
/// per-type counts, or the whole-table edge count for an untyped hop
/// (`-->`). All O(1) reads.
fn rel_types_edge_count(txn: Txn, rel_types: &[String]) -> Result<u64, QueryError> {
    if rel_types.is_empty() {
        return Ok(GraphStore::edge_count_in_txn(txn)?);
    }
    let mut total: u64 = 0;
    for rel_type in rel_types {
        total = total.saturating_add(GraphStore::rel_type_count_in_txn(txn, rel_type)?);
    }
    Ok(total)
}

/// One side of `plan_reversed_pattern`'s comparison — see its docs for
/// the model. Saturating u64 throughout: the counts are real table
/// sizes, but `rows · 2 + E · 2` on a pathological database shouldn't
/// wrap into a nonsense comparison.
fn anchor_cost(anchor: &EndpointCost, other: &EndpointCost, edges: u64) -> u64 {
    // Physical rows visited (a filter never shrinks an unindexed scan),
    // plus one filter evaluation per visited row when there is one.
    let scan_and_filter = anchor
        .scan_rows
        .saturating_mul(1 + u64::from(anchor.filtered));
    // Prorate the type's edges by how much the anchor's own narrowing
    // (index seek, unindexed-equality selectivity, or a Seed's zero
    // rows) shrank what the leaf emits into the Expand.
    let walked = if anchor.label_rows == 0 {
        0
    } else {
        u64::try_from(
            u128::from(edges) * u128::from(anchor.out_rows) / u128::from(anchor.label_rows),
        )
        .unwrap_or(u64::MAX)
    };
    let expand_and_stranded = walked.saturating_mul(1 + u64::from(other.filtered));
    scan_and_filter.saturating_add(expand_and_stranded)
}

/// Default selectivity divisor for a literal-equality predicate with no
/// index behind it: the endpoint is assumed to emit `1/10` of its
/// scanned rows (System R's classic default for equality without
/// statistics). Deliberately coarse — the point isn't accuracy, it's
/// that an equality must price *better* than no filter at all: before
/// this credit existed, an unindexed `{id: 100}` left `out_rows` at the
/// full label count while `filtered` doubled the scan term, so the
/// filter made its endpoint price strictly worse and start-point
/// selection inverted (measured at 6-70x on the LDBC-style workload —
/// see the fix's regression tests below and issue #208).
const UNINDEXED_EQ_SELECTIVITY_DIVISOR: u64 = 10;

/// What `endpoint_start_cost` knows about starting a traversal at one
/// endpoint — the inputs to `anchor_cost`, see `plan_reversed_pattern`
/// for the model.
struct EndpointCost {
    /// Rows the leaf scan physically visits: 0 for a bound `Seed`, the
    /// index-match count when an indexed literal-equality candidate
    /// narrows the scan to a seek, else the full label count — an
    /// *unindexed* filter never shrinks this, since the scan still
    /// touches every label row to evaluate it.
    scan_rows: u64,
    /// Rows the leaf emits into the Expand above it, after every pushed
    /// filter: equal to `scan_rows` for a seek (the index count is
    /// exact), narrowed by `UNINDEXED_EQ_SELECTIVITY_DIVISOR` for a
    /// literal equality with no index. `out_rows / label_rows` is the
    /// fraction of the label the traversal actually walks edges for,
    /// which is what prorates the edge-walk estimate.
    out_rows: u64,
    /// The unnarrowed size of the endpoint's scan domain — its label
    /// count, or the whole node table when unlabeled.
    label_rows: u64,
    /// The endpoint has pushable filtering work priced per scanned row:
    /// a non-equality conjunct (`CONTAINS`, a range, ...), an equality
    /// with no index behind it, or a `$param` equality whose value is
    /// unknown at plan time. Anchoring here evaluates it once per
    /// scanned row, below the Expand; anchoring at the other side
    /// strands it above, once per walked edge.
    filtered: bool,
}

/// Cost facts for starting the pattern's traversal at `node` — see
/// `plan_reversed_pattern` and `EndpointCost`. `scan_rows` uses the
/// same candidates `apply_index_seeks` would fuse into an `IndexSeek`
/// once this endpoint actually is the start; a pushable literal
/// equality that *isn't* such a candidate can't narrow the scan but
/// still narrows `out_rows` (default equality selectivity) and marks
/// the endpoint `filtered`; every other pushable predicate only marks
/// `filtered`. Pushability here mirrors `build_match_plan`'s own
/// start-only test (`conjunct_sole_var`), so `filtered` is only set for
/// predicates that genuinely would wrap this endpoint's scan. A
/// `HasLabel` conjunct is skipped outright: labels are what `rows`
/// already counts, and an extra-label test has no per-label count to
/// credit it with, so treating it as a stranded filter would suppress
/// reversal for multi-label endpoints on no evidence.
fn endpoint_start_cost(
    node: &NodePattern,
    conjuncts: &[Expr],
    carried_vars: &HashSet<String>,
    txn: Txn,
) -> Result<EndpointCost, QueryError> {
    if node.var.as_ref().is_some_and(|v| carried_vars.contains(v)) {
        return Ok(EndpointCost {
            scan_rows: 0,
            out_rows: 0,
            label_rows: 0,
            filtered: false,
        });
    }
    let label = node.labels.first();
    let label_rows = match label {
        Some(label) => GraphStore::label_count_in_txn(txn, label)?,
        None => GraphStore::node_count_in_txn(txn)?,
    };
    let mut scan_rows = label_rows;
    let mut out_rows = label_rows;
    let mut filtered = false;
    let consider = |scan_rows: &mut u64,
                    out_rows: &mut u64,
                    filtered: &mut bool,
                    prop: &str,
                    lit: &Literal|
     -> Result<(), QueryError> {
        if !matches!(lit, Literal::Param(_)) {
            if let Some(label) = label {
                if GraphStore::index_def_in_txn(txn, label, prop)?.is_some() {
                    let count = GraphStore::index_match_count_in_txn(
                        txn,
                        label,
                        prop,
                        &literal_to_value(lit),
                    )?;
                    // A seek both shrinks the physical scan and gives an
                    // exact emitted-row count.
                    *scan_rows = (*scan_rows).min(count);
                    *out_rows = (*out_rows).min(count);
                    return Ok(());
                }
            }
            // Unindexed literal equality: the scan still visits every
            // label row (priced via `filtered`), but what it *emits*
            // gets the default equality-selectivity credit — see
            // `UNINDEXED_EQ_SELECTIVITY_DIVISOR`.
            *out_rows = (*out_rows).min((label_rows / UNINDEXED_EQ_SELECTIVITY_DIVISOR).max(1));
        }
        *filtered = true;
        Ok(())
    };
    for (key, expr) in &node.props {
        if let ReturnExpr::Lit(lit) = expr {
            consider(&mut scan_rows, &mut out_rows, &mut filtered, key, lit)?;
        } else {
            filtered = true;
        }
    }
    if let Some(var) = &node.var {
        for c in conjuncts {
            if conjunct_sole_var(c) != Some(var.as_str()) {
                continue;
            }
            match c {
                Expr::Compare(pa, CompareOp::Eq, lit) => {
                    consider(&mut scan_rows, &mut out_rows, &mut filtered, &pa.prop, lit)?
                }
                Expr::HasLabel(..) => {}
                _ => filtered = true,
            }
        }
    }
    Ok(EndpointCost {
        scan_rows,
        out_rows,
        label_rows,
        filtered,
    })
}

/// The same pattern walked from its other end: node order reversed, each
/// hop's direction flipped (`Either` stays). Only called for all-fixed-
/// hop patterns (see `plan_reversed_pattern`'s guards), so none of the
/// variable-length-only `RelPattern` fields need adjusting.
fn reverse_pattern(pattern: &Pattern) -> Pattern {
    let nodes: Vec<&NodePattern> = std::iter::once(&pattern.start)
        .chain(pattern.hops.iter().map(|(_, node)| node))
        .collect();
    let start = (*nodes.last().expect("nodes is never empty")).clone();
    let hops = pattern
        .hops
        .iter()
        .enumerate()
        .rev()
        .map(|(i, (rel, _))| {
            let mut rel = rel.clone();
            rel.direction = match rel.direction {
                RelDirection::Right => RelDirection::Left,
                RelDirection::Left => RelDirection::Right,
                RelDirection::Either => RelDirection::Either,
            };
            // `nodes[i]` is the node on the near side of hop `i` in the
            // written pattern — the far side once the hop is walked
            // backwards.
            (rel, nodes[i].clone())
        })
        .collect();
    Pattern { start, hops }
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
                        value: IndexSeekValue::Fixed(literal_to_value(&lit)),
                    };
                    return Ok(match rebuild_and(candidates) {
                        Some(predicate) => LogicalPlan::Filter {
                            input: Box::new(seek),
                            predicate,
                        },
                        None => seek,
                    });
                }
                // No literal-valued conjunct had a declared index. A
                // row-dependent one still might (`UNWIND rows AS row MATCH
                // (n:Label {prop: row.field})` -- exactly the shape a bulk
                // import's relationship-creation pass uses, one indexed
                // lookup per incoming row instead of a full label scan
                // repeated per row). No cardinality to rank these by (the
                // value isn't known until execution), so just take the
                // first match with a declared index rather than the
                // most-selective one the literal branch above picks.
                for (i, c) in candidates.iter().enumerate() {
                    let Expr::GeneralCompare(ReturnExpr::Prop(pa), CompareOp::Eq, other) = c else {
                        continue;
                    };
                    if pa.var != *var {
                        continue;
                    }
                    // Guards against a self-referential `n.a = n.b` (or
                    // `n.a = n.b.c`, ...) ever reaching here
                    // (pattern_prop_predicate never produces that shape
                    // today, but nothing else stops a future caller from
                    // trying) -- `other` must be evaluable from the row
                    // *without* the very node this scan is trying to find.
                    if return_expr_references_var(other, var) {
                        continue;
                    }
                    if GraphStore::index_def_in_txn(txn, label, &pa.prop)?.is_some() {
                        let Expr::GeneralCompare(ReturnExpr::Prop(pa), _, value_expr) =
                            candidates.remove(i)
                        else {
                            unreachable!("just matched this index above, shape can't have changed")
                        };
                        let seek = LogicalPlan::IndexSeek {
                            var: var.clone(),
                            label: label.clone(),
                            prop: pa.prop,
                            value: IndexSeekValue::RowExpr(value_expr),
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

                // No equality seek fired -- try a RANGE seek: `var.prop`
                // inequality conjuncts (`>`, `>=`, `<`, `<=`) against
                // literals, on a prop with a declared index. Bounds on
                // the same prop combine into one bounded scan (`year >
                // 2000 AND year < 2010`). Unlike the equality fusion,
                // the originating conjuncts are KEPT in the residual
                // filter: the storage scan is a deliberate superset for
                // numeric bounds (int/float type regions, widened lossy
                // conversions -- see `lookup_range`), so the filter
                // stays the source of truth and the seek only shrinks
                // the candidate set. First indexed prop encountered
                // wins (no O(1) range-cardinality statistic exists to
                // rank candidates by).
                let mut range_prop: Option<String> = None;
                let mut lo: Option<(PropertyValue, bool)> = None;
                let mut hi: Option<(PropertyValue, bool)> = None;
                for c in &candidates {
                    let Expr::Compare(pa, op, lit) = c else {
                        continue;
                    };
                    if pa.var != *var || matches!(lit, Literal::Param(_)) {
                        continue;
                    }
                    let (is_lo, inclusive) = match op {
                        CompareOp::Gt => (true, false),
                        CompareOp::Ge => (true, true),
                        CompareOp::Lt => (false, false),
                        CompareOp::Le => (false, true),
                        _ => continue,
                    };
                    if range_prop.as_deref().is_some_and(|p| p != pa.prop) {
                        continue;
                    }
                    if range_prop.is_none() {
                        if GraphStore::index_def_in_txn(txn, label, &pa.prop)?.is_none() {
                            continue;
                        }
                        range_prop = Some(pa.prop.clone());
                    }
                    let value = literal_to_value(lit);
                    // Two bounds on the same side keep the first -- the
                    // residual filter enforces the tighter one anyway.
                    if is_lo && lo.is_none() {
                        lo = Some((value, inclusive));
                    } else if !is_lo && hi.is_none() {
                        hi = Some((value, inclusive));
                    }
                }
                if let Some(prop) = range_prop {
                    let seek = LogicalPlan::IndexRangeSeek {
                        var: var.clone(),
                        label: label.clone(),
                        prop,
                        lo,
                        hi,
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
        | LogicalPlan::IndexSeek { .. }
        | LogicalPlan::IndexRangeSeek { .. }
        | LogicalPlan::EdgeTypeScan { .. }) => leaf,
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
                assert_eq!(
                    value,
                    IndexSeekValue::Fixed(PropertyValue::String("alice@x.com".to_string()))
                );
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
                assert_eq!(
                    value,
                    IndexSeekValue::Fixed(PropertyValue::String("alice@x.com".to_string()))
                );
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

    /// Issue #208's IS5 shape: a large label carrying an *unindexed*
    /// literal equality versus a smaller unfiltered far label. The
    /// equality collapses what Big emits, so Big must stay the anchor
    /// even though Small's label is 10x smaller — before out_rows
    /// existed, the equality only doubled Big's scan term and the
    /// planner anchored at Small, walking every :E edge (measured 6x
    /// slower at LDBC-style SF 0.1 scale, 33.9ms vs 5.6ms).
    #[test]
    fn unindexed_equality_keeps_its_own_large_label_as_anchor() {
        let store = GraphStore::open_memory().unwrap();
        // Big=100 {id}, Small=10, 150 :E edges Big->Small — the measured
        // 10,000/1,000/15,000 ratios scaled by 100.
        let mut big_ids = Vec::new();
        for i in 0..100 {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), PropertyValue::Int(i));
            big_ids.push(store.create_node(&["Big"], props).unwrap());
        }
        let mut small_ids = Vec::new();
        for _ in 0..10 {
            small_ids.push(store.create_node(&["Small"], BTreeMap::new()).unwrap());
        }
        for (i, big) in big_ids.iter().enumerate() {
            store
                .create_edge("E", *big, small_ids[i % 10], BTreeMap::new())
                .unwrap();
        }
        for big in big_ids.iter().take(50) {
            store
                .create_edge("E", *big, small_ids[0], BTreeMap::new())
                .unwrap();
        }

        let write = store.begin_write().unwrap();
        // Written from the equality side: no reversal.
        let pattern = pattern_from("MATCH (m:Big {id: 42})-[:E]->(p:Small) RETURN p");
        assert!(
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .is_none(),
            "the unindexed-equality endpoint must stay the anchor"
        );
        // Written from the far side: reversal lands on the equality.
        let pattern = pattern_from("MATCH (p:Small)<-[:E]-(m:Big {id: 42}) RETURN p");
        let reversed =
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .expect("expected reversal toward the unindexed equality on m");
        assert_eq!(reversed.start.var.as_deref(), Some("m"));
    }

    /// Issue #208's IC2 shape: an equality-filtered start versus an
    /// *unlabeled* far endpoint, with the start's adjacent hop the
    /// larger edge population. The old model saw only the start's
    /// doubled scan term plus its full adjacent-edge count and anchored
    /// at the unlabeled end — an AllNodesScan (measured 70x slower,
    /// 650ms vs ~9ms). The emitted-rows credit prorates the start's
    /// edge walk down to its post-equality fraction.
    #[test]
    fn equality_start_beats_an_unlabeled_far_endpoint() {
        let store = GraphStore::open_memory().unwrap();
        let mut small_ids = Vec::new();
        for i in 0..10 {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), PropertyValue::Int(i));
            small_ids.push(store.create_node(&["Small"], props).unwrap());
        }
        let mut big_ids = Vec::new();
        for _ in 0..100 {
            big_ids.push(store.create_node(&["Big"], BTreeMap::new()).unwrap());
        }
        // 500 :E1 edges adjacent to Small (the start's first hop), 150
        // :E2 edges adjacent to the unlabeled end.
        for i in 0..500 {
            store
                .create_edge("E1", small_ids[i % 10], big_ids[i % 100], BTreeMap::new())
                .unwrap();
        }
        for i in 0..150 {
            store
                .create_edge(
                    "E2",
                    big_ids[i % 100],
                    big_ids[(i * 7) % 100],
                    BTreeMap::new(),
                )
                .unwrap();
        }

        let write = store.begin_write().unwrap();
        let pattern = pattern_from("MATCH (p:Small {id: 1})-[:E1]->(x:Big)<-[:E2]-(m) RETURN m");
        assert!(
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .is_none(),
            "the equality-filtered start must beat the unlabeled far endpoint"
        );
    }

    fn seed_people(store: &GraphStore, common: usize, rare: usize) -> Vec<marsdb_graph::NodeId> {
        let mut common_ids = Vec::with_capacity(common);
        for i in 0..common {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), PropertyValue::Int(i as i64));
            common_ids.push(store.create_node(&["Common"], props).unwrap());
        }
        for i in 0..rare {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), PropertyValue::Int(i as i64));
            store.create_node(&["Rare"], props).unwrap();
        }
        common_ids
    }

    #[test]
    fn reverses_when_the_far_endpoint_label_is_smaller() {
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 1);
        let pattern = pattern_from("MATCH (a:Common)-[:R]->(b:Rare) RETURN a");

        let write = store.begin_write().unwrap();
        let reversed =
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .expect("expected reversal toward the 1-node Rare label");

        assert_eq!(reversed.start.var.as_deref(), Some("b"));
        assert_eq!(reversed.start.labels, vec!["Rare"]);
        let (rel, node) = &reversed.hops[0];
        // The written `->` walked backwards is `<-`.
        assert_eq!(rel.direction, RelDirection::Left);
        assert_eq!(rel.rel_types, vec!["R"]);
        assert_eq!(node.var.as_deref(), Some("a"));
    }

    #[test]
    fn keeps_written_order_when_the_start_is_already_cheapest_or_tied() {
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 1, 20);
        let write = store.begin_write().unwrap();

        let cheaper_start = pattern_from("MATCH (a:Common)-[:R]->(b:Rare) RETURN a");
        assert!(plan_reversed_pattern(
            &cheaper_start,
            &None,
            &Default::default(),
            Txn::Write(&write)
        )
        .unwrap()
        .is_none());

        // Tie (same label both ends) keeps written order for determinism.
        let tied = pattern_from("MATCH (a:Rare)-[:R]->(b:Rare) RETURN a");
        assert!(
            plan_reversed_pattern(&tied, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn reverses_toward_an_indexed_where_equality_on_the_far_endpoint() {
        // Both labels are the same size; only the WHERE equality on `b`
        // (backed by an index) distinguishes them -- the conjunct-based
        // half of endpoint_start_cost.
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 20);
        store.create_index("Rare", "id", false).unwrap();
        let part = part_from("MATCH (a:Rare)-[:R]->(b:Rare) WHERE b.id = 7 RETURN a");

        let write = store.begin_write().unwrap();
        let reversed = plan_reversed_pattern(
            &part.pattern,
            &part.where_clause,
            &Default::default(),
            Txn::Write(&write),
        )
        .unwrap()
        .expect("expected reversal toward the indexed b.id = 7");
        assert_eq!(reversed.start.var.as_deref(), Some("b"));
    }

    #[test]
    fn keeps_written_order_when_reversal_would_strand_a_start_filter() {
        // `WHERE a.id > 5` is pushable to a's scan but has no index and
        // isn't an equality, so a's row estimate gets no credit for it
        // (`EndpointCost::filtered`). With a real edge population (40 R
        // edges, all incident to the single Rare node -- the shape of
        // the recommendations benchmark's `m.title CONTAINS 'Matrix'`
        // regression, where every RATED edge touches the small label),
        // reversing toward the smaller Rare label strands that filter
        // above an Expand that walks all 40 edges: 1 + 2*40 = 81
        // reversed vs 2*20 + 40 = 80 written -- written order stands.
        let store = GraphStore::open_memory().unwrap();
        let common = seed_people(&store, 20, 0);
        let rare = store.create_node(&["Rare"], BTreeMap::new()).unwrap();
        for id in &common {
            for _ in 0..2 {
                store.create_edge("R", *id, rare, BTreeMap::new()).unwrap();
            }
        }
        let part = part_from("MATCH (a:Common)-[:R]->(b:Rare) WHERE a.id > 5 RETURN a");

        let write = store.begin_write().unwrap();
        assert!(plan_reversed_pattern(
            &part.pattern,
            &part.where_clause,
            &Default::default(),
            Txn::Write(&write)
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn reverses_a_filtered_start_when_the_edge_population_is_small() {
        // The shape the interim always-decline rule got wrong: a big
        // filtered label against a tiny far endpoint with few edges.
        // Expanding from the one Rare node walks 2 edges total no matter
        // how unselective a's filter is -- 1 + 2*2 = 5 reversed vs
        // 2*200 + 2 = 402 written. A small live-edge count caps what
        // the stranded filter can cost, so reversal wins.
        let store = GraphStore::open_memory().unwrap();
        let common = seed_people(&store, 200, 0);
        let rare = store.create_node(&["Rare"], BTreeMap::new()).unwrap();
        for id in common.iter().take(2) {
            store.create_edge("R", *id, rare, BTreeMap::new()).unwrap();
        }
        let part = part_from("MATCH (a:Common)-[:R]->(b:Rare) WHERE a.id > 5 RETURN a");

        let write = store.begin_write().unwrap();
        let reversed = plan_reversed_pattern(
            &part.pattern,
            &part.where_clause,
            &Default::default(),
            Txn::Write(&write),
        )
        .unwrap()
        .expect("expected reversal toward the tiny edge population");
        assert_eq!(reversed.start.var.as_deref(), Some("b"));
    }

    #[test]
    fn still_reverses_a_filtered_start_toward_an_indexed_far_equality() {
        // Same stranded-filter start as above, but the far endpoint's
        // advantage is a *measured* indexed-equality count
        // (`EndpointCost::seek_backed`), not merely a smaller label --
        // expanding from the one matching `b` beats scanning 20 `a`s no
        // matter how selective a's filter turns out to be.
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 20);
        store.create_index("Rare", "id", false).unwrap();
        let part =
            part_from("MATCH (a:Common)-[:R]->(b:Rare) WHERE a.id > 5 AND b.id = 7 RETURN a");

        let write = store.begin_write().unwrap();
        let reversed = plan_reversed_pattern(
            &part.pattern,
            &part.where_clause,
            &Default::default(),
            Txn::Write(&write),
        )
        .unwrap()
        .expect("expected reversal toward the indexed b.id = 7");
        assert_eq!(reversed.start.var.as_deref(), Some("b"));
    }

    #[test]
    fn reverses_toward_a_carried_far_endpoint() {
        // `WITH p MATCH (a:Common)-->(p)` -- `p` is already bound, so
        // starting there is a Seed (cost 0) instead of scanning Common.
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 1);
        let pattern = pattern_from("MATCH (a:Common)-[:R]->(p) RETURN a");
        let carried: HashSet<String> = ["p".to_string()].into();

        let write = store.begin_write().unwrap();
        let reversed = plan_reversed_pattern(&pattern, &None, &carried, Txn::Write(&write))
            .unwrap()
            .expect("expected reversal toward the carried p");
        assert_eq!(reversed.start.var.as_deref(), Some("p"));
    }

    #[test]
    fn never_reverses_a_pattern_containing_a_variable_length_hop() {
        // `[r*1..2]` binds a relationship *list* in pattern order --
        // user-visible, so reversal is disqualified outright.
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 1);
        let pattern = pattern_from("MATCH (a:Common)-[:R*1..2]->(b:Rare) RETURN a");

        let write = store.begin_write().unwrap();
        assert!(
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn multi_hop_reversal_flips_every_hop_and_keeps_inner_nodes_in_order() {
        let store = GraphStore::open_memory().unwrap();
        seed_people(&store, 20, 1);
        let pattern = pattern_from("MATCH (a:Common)-[:X]->(m)<-[:Y]-(b:Rare) RETURN a");

        let write = store.begin_write().unwrap();
        let reversed =
            plan_reversed_pattern(&pattern, &None, &Default::default(), Txn::Write(&write))
                .unwrap()
                .expect("expected reversal toward Rare");

        assert_eq!(reversed.start.var.as_deref(), Some("b"));
        assert_eq!(reversed.hops.len(), 2);
        // Written `<-[:Y]-` from b's side becomes `-[:Y]->` into m...
        assert_eq!(reversed.hops[0].0.rel_types, vec!["Y"]);
        assert_eq!(reversed.hops[0].0.direction, RelDirection::Right);
        assert_eq!(reversed.hops[0].1.var.as_deref(), Some("m"));
        // ...and the written `-[:X]->` becomes `<-[:X]-` into a.
        assert_eq!(reversed.hops[1].0.rel_types, vec!["X"]);
        assert_eq!(reversed.hops[1].0.direction, RelDirection::Left);
        assert_eq!(reversed.hops[1].1.var.as_deref(), Some("a"));
    }

    #[test]
    fn fuses_a_literal_arg_call_equality_into_a_row_expr_index_seek() {
        // `n.joined = date('2020-01-10')` -- the shape a `$param`-
        // substituted temporal equality takes (mars-9ez). The call's
        // arguments are all var-free, so it's evaluable once per seed row
        // and must promote to an IndexSeek with a RowExpr value, not stay
        // a per-candidate Filter over the label scan.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Event", "joined", false).unwrap();
        let part = part_from("MATCH (n:Event) WHERE n.joined = date('2020-01-10') RETURN n");

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::IndexSeek {
                prop,
                value: IndexSeekValue::RowExpr(ReturnExpr::Call { name, .. }),
                ..
            } => {
                assert_eq!(prop, "joined");
                assert_eq!(name, "date");
            }
            other => panic!("expected a RowExpr IndexSeek on the call, got {other:?}"),
        }
    }

    #[test]
    fn does_not_promote_a_call_whose_argument_references_the_scan_var() {
        // `date(n.born)` needs `n` itself to evaluate -- promoting it
        // would evaluate against a row that doesn't have `n` yet.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Event", "joined", false).unwrap();
        let part = part_from("MATCH (n:Event) WHERE n.joined = date(n.born) RETURN n");

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::Filter { input, .. } => {
                assert!(matches!(*input, LogicalPlan::NodeByLabelScan { .. }));
            }
            other => panic!("expected a Filter over the scan, got {other:?}"),
        }
    }

    #[test]
    fn does_not_promote_a_rand_call() {
        // rand() has no arguments but must still evaluate per candidate
        // row, not once per seed row -- hoisting it into an IndexSeek
        // value would change which rows match.
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Event", "score", false).unwrap();
        let part = part_from("MATCH (n:Event) WHERE n.score = rand() RETURN n");

        let write = store.begin_write().unwrap();
        let plan =
            build_match_plan(&part.pattern, &part.where_clause, &Default::default()).unwrap();
        let plan = apply_index_seeks(plan, Txn::Write(&write)).unwrap();

        match plan {
            LogicalPlan::Filter { input, .. } => {
                assert!(matches!(*input, LogicalPlan::NodeByLabelScan { .. }));
            }
            other => panic!("expected a Filter over the scan, got {other:?}"),
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
                        assert_eq!(
                            value,
                            IndexSeekValue::Fixed(PropertyValue::String("user7@x.com".to_string()))
                        );
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
