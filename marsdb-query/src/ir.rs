//! Language-agnostic logical plan for the read path (MATCH traversal).
//!
//! Deliberately close to Neo4j's Cypher runtime operator shape, and maps
//! ~1:1 onto TinkerPop Gremlin traversal steps (`g.V().hasLabel(X)` ->
//! NodeByLabelScan+Filter, `.out('REL')` -> Expand) so a future Gremlin
//! frontend can compile into this same IR without redesigning the
//! executor.
//!
//! CREATE has no traversal/filtering semantics (it only ever produces new
//! rows), so it's executed directly from the AST rather than through this
//! IR — see `executor::execute_create`. `LIMIT`/`ORDER BY` aren't IR nodes
//! either: `execute_match` bounds consumption of a non-blocking plan stream
//! for LIMIT, while ORDER BY explicitly materializes before sorting.

use marsdb_graph::PropertyValue;

use crate::ast::{Expr, ReturnExpr};

/// Traversal direction for `Expand`. Distinct from `marsdb_graph::Direction`
/// (which only has `Out`/`In`) because `Either` (undirected patterns,
/// `-[r:TYPE]-`) has no single-call storage-level meaning — the executor
/// handles it by calling `neighbors_in_txn` twice (`Out` then `In`) and
/// deduping by `edge_id`, with no `GraphStore`/storage change needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandDirection {
    Out,
    In,
    Either,
}

#[derive(Debug, Clone)]
pub enum LogicalPlan {
    AllNodesScan {
        var: String,
    },
    NodeByLabelScan {
        var: String,
        label: String,
    },
    /// A direct lookup against a declared property index — `label`'s nodes
    /// with `prop == value`, without walking `NODE_LABEL_INDEX` or reading
    /// every candidate node's record to check the property. Never produced
    /// by `build_match_plan` directly (which has no storage access, so no
    /// way to know which indexes exist) — `planner::apply_index_seeks`
    /// rewrites a `Filter(Compare(var.prop = literal))` over a
    /// `NodeByLabelScan` into this, post-build, once a real `Txn` is
    /// available to check `GraphStore::index_def_in_txn`.
    IndexSeek {
        var: String,
        label: String,
        prop: String,
        value: PropertyValue,
    },
    /// "Start from the rows already bound coming into this statement" —
    /// used when a `QueryPart`'s pattern start-variable was already bound
    /// by a prior part's `WITH` output, instead of a fresh
    /// `AllNodesScan`/`NodeByLabelScan`. A leaf like the scans above, but
    /// reads externally-supplied rows (the executor's `seed` parameter)
    /// rather than the graph.
    Seed {
        var: String,
    },
    Expand {
        input: Box<LogicalPlan>,
        from_var: String,
        to_var: String,
        rel_var: Option<String>,
        /// Empty means untyped (any relationship matches); more than one
        /// means "any of these" (`[:A|B]`).
        rel_labels: Vec<String>,
        direction: ExpandDirection,
    },
    /// `[:TYPE*min..max]` — BFS from each input row's bound node instead of
    /// a single fixed hop.
    VarExpand {
        input: Box<LogicalPlan>,
        from_var: String,
        to_var: String,
        rel_labels: Vec<String>,
        direction: ExpandDirection,
        min_hops: u32,
        max_hops: Option<u32>,
        /// Internal rel-vars bound by *earlier* fixed hops of this same
        /// pattern (`build_match_plan`'s `prior_rel_vars` at the point this
        /// hop was built) — real Cypher's edge-isomorphism rule (no
        /// relationship repeated within one MATCH pattern) applies across
        /// the whole pattern, not just within a variable-length hop's own
        /// BFS. Seeds the BFS's excluded-edges set so it can't walk back
        /// over an edge an earlier hop in the same pattern already used
        /// (found via TCK's Match5 `[27]`: an earlier fixed hop's edge got
        /// silently re-walked by a later `<-[:LIKES*3]->`, producing extra
        /// spurious rows). Later hops checking a variable-length hop's own
        /// *internally* traversed edges is a separate, still-open gap (a
        /// var-length hop binds no single edge to check against) — not
        /// needed by that scenario, since nothing follows its `VarExpand`.
        exclude_edge_vars: Vec<String>,
        /// Set to a fresh internal name (`executor::name_pattern_for_path`)
        /// iff this hop's `RelPattern::capture_path_segment` was set --
        /// asks `executor::expand_variable_row` to also deposit its own
        /// internally-traversed edge/node sequence into each output row
        /// under that name, for `executor::assemble_path` to read back
        /// (named-path capture over a variable-length hop, TCK's
        /// Quantifier1-4 `[8]`/`[9]`).
        path_segment_var: Option<String>,
        /// Set to the user's own `rel.var` when they wrote one on a
        /// variable-length hop (`MATCH (a)-[r:TYPE*1..3]->(b)`, TCK's
        /// Match4 `[1]`/`[6]`) -- real Cypher binds a *list* of the
        /// traversed relationships, not a single edge the way a fixed
        /// hop's own `rel_var` would. Mutually exclusive with
        /// `path_segment_var` in practice (one is the user's own real
        /// variable, the other `name_pattern_for_path`'s internal
        /// bookkeeping for a *different*, anonymous position) --
        /// `executor::expand_variable_row` deposits a `Binding::List` of
        /// fully-materialized `Value::Edge`s under this name instead of
        /// `path_segment_var`'s cheaper id-only `Binding::Path` segment.
        rel_list_var: Option<String>,
        /// `[:TYPE* {year: 1988}]` -- filters *every* hop of the
        /// traversal by this same inline property map, not just the
        /// final one (TCK's Match4 `[5]`: only a path where every hop's
        /// own edge matches survives, so a 2-hop path can't "average out"
        /// -- one non-matching hop excludes the whole path from that
        /// point on). Evaluated once per `expand_variable_row` call
        /// (constant across the whole BFS, not per-candidate), then
        /// checked against each candidate edge's own stored properties
        /// during expansion.
        rel_props: Vec<(String, ReturnExpr)>,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
}
