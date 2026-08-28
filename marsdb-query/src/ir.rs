//! Language-agnostic logical plan for the read path (MATCH traversal).
//!
//! Shaped to map ~1:1 onto TinkerPop Gremlin traversal steps
//! (`g.V().hasLabel(X)` -> NodeByLabelScan+Filter, `.out('REL')` ->
//! Expand), so a future Gremlin frontend could compile into this same IR.
//!
//! CREATE has no traversal/filtering semantics, so it executes directly
//! from the AST rather than through this IR (`executor::execute_create`).
//! `LIMIT`/`ORDER BY` aren't IR nodes either: `execute_match` bounds
//! consumption of a non-blocking plan stream for LIMIT, while ORDER BY
//! materializes before sorting.

use marsdb_graph::PropertyValue;

use crate::ast::{Expr, ReturnExpr};

/// Traversal direction for `Expand`. Distinct from `marsdb_graph::Direction`
/// (`Out`/`In` only) because `Either` (undirected patterns, `-[r:TYPE]-`)
/// has no single-call storage meaning — the executor handles it by calling
/// `neighbors_in_txn` twice and deduping by `edge_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandDirection {
    Out,
    In,
    Either,
}

/// How an `IndexSeek`'s lookup value is obtained. `Fixed` is known before
/// the first row (a literal, or a `$param` resolved upstream) and reused
/// across every seed row. `RowExpr` depends on the current seed row
/// (`UNWIND row ... MATCH (n {prop: row.field})`) and is re-evaluated for
/// each one; see `Executor::stream_index_seek`.
#[derive(Debug, Clone, PartialEq)]
pub enum IndexSeekValue {
    Fixed(PropertyValue),
    RowExpr(ReturnExpr),
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
    /// every candidate node's record. Never produced by `build_match_plan`
    /// directly (no storage access, so no way to know which indexes
    /// exist) — `planner::apply_index_seeks` rewrites a matching `Filter`
    /// over a `NodeByLabelScan` into this once a real `Txn` is available.
    IndexSeek {
        var: String,
        label: String,
        prop: String,
        value: IndexSeekValue,
    },
    /// Full `EDGES`-table sweep binding an entire single-hop pattern at
    /// once — chosen by the planner (cost-gated on the O(1) edge count)
    /// when anchoring at either endpoint would walk more adjacency
    /// entries than one sequential sweep costs. Type membership, the
    /// pushed-down `rel_predicate` (conjuncts on `rel_var` only,
    /// evaluated from the swept record's in-hand bytes), and endpoint
    /// label checks all happen in-scan; anything else stays in a
    /// residual `Filter` above.
    EdgeTypeScan {
        src_var: String,
        rel_var: String,
        dst_var: String,
        /// Empty = any relationship type.
        rel_types: Vec<String>,
        src_label: Option<String>,
        dst_label: Option<String>,
        rel_predicate: Option<Expr>,
    },
    /// Bounded scan over one indexed `(label, prop)`'s order-preserving
    /// key space — `WHERE n.year > 2000 [AND n.year < 2010]` with an
    /// index on `(Label, year)`. Each bound is `(value, inclusive)`. The
    /// storage lookup returns a superset for numeric bounds (lossy
    /// conversions widened outward), so the planner keeps the originating
    /// conjuncts as a residual `Filter` above this node.
    IndexRangeSeek {
        var: String,
        label: String,
        prop: String,
        lo: Option<(PropertyValue, bool)>,
        hi: Option<(PropertyValue, bool)>,
    },
    /// Start from rows already bound coming into this statement, used
    /// when a `QueryPart`'s pattern start-variable was already bound by a
    /// prior part's `WITH` output. A leaf like the scans above, but reads
    /// externally-supplied rows (the executor's `seed` parameter) rather
    /// than the graph.
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
        /// pattern (`build_match_plan`'s `prior_rel_vars`). Cypher's
        /// edge-isomorphism rule (no relationship repeated within one
        /// MATCH pattern) applies across the whole pattern, so this seeds
        /// the BFS's excluded-edges set. See `exclude_edge_sets` below for
        /// the complementary case of an earlier variable-length hop.
        exclude_edge_vars: Vec<String>,
        /// Same purpose as `exclude_edge_vars`, but for edges an earlier
        /// *variable-length* hop of this pattern traversed
        /// (`build_match_plan`'s `prior_edge_sets`, each entry naming
        /// another `VarExpand`'s `exclude_edge_var` below). A single edge
        /// id can't represent "whichever edges that hop used for this
        /// row," so this is a list of `Binding::Path`-valued names, each
        /// row's set unioned into the BFS's excluded-edges seed.
        exclude_edge_sets: Vec<String>,
        /// This hop's own internal, always-synthesized name that
        /// `executor::expand_variable_row` deposits a `Binding::Path` of
        /// its per-row traversed edges under — the producer half of
        /// `exclude_edge_sets`, letting a later hop exclude whatever this
        /// row's traversal used. Populated unconditionally since the
        /// segment is already built for the BFS's own isomorphism
        /// tracking regardless of whether this hop requested it.
        exclude_edge_var: String,
        /// Set to a fresh internal name (`executor::name_pattern_for_path`)
        /// iff this hop's `RelPattern::capture_path_segment` was set,
        /// asking `executor::expand_variable_row` to deposit its
        /// internally-traversed edge/node sequence into each output row
        /// under that name for `executor::assemble_path` to read back.
        path_segment_var: Option<String>,
        /// Set to the user's own `rel.var` when written on a
        /// variable-length hop (`MATCH (a)-[r:TYPE*1..3]->(b)`) — Cypher
        /// binds a list of traversed relationships here, not a single
        /// edge. Mutually exclusive with `path_segment_var` in practice;
        /// `executor::expand_variable_row` deposits a `Binding::List` of
        /// fully-materialized `Value::Edge`s under this name instead of
        /// `path_segment_var`'s cheaper id-only `Binding::Path` segment.
        rel_list_var: Option<String>,
        /// `[:TYPE* {year: 1988}]` — filters every hop of the traversal
        /// by this same inline property map, not just the final one: one
        /// non-matching hop excludes the whole path from that point on.
        /// Evaluated once per `expand_variable_row` call, then checked
        /// against each candidate edge during expansion.
        rel_props: Vec<(String, ReturnExpr)>,
    },
    /// `MATCH (first)-[rs*]->(second)` where `rs` was already bound
    /// earlier (e.g. `WITH [r1, r2] AS rs`). Cypher's semantics here are
    /// "verify this exact, already-fixed relationship sequence forms a
    /// connected path from `from_var`," not `VarExpand`'s fresh BFS
    /// search, which would overwrite `rs`'s carried binding. Deterministic:
    /// `executor::match_bound_rel_list_row` walks the edges in order, each
    /// edge's direction-appropriate endpoint must equal the current node,
    /// and binds `to_var` to wherever the walk ends up (0 output rows if
    /// the list doesn't chain, is out of hop range, or a label mismatches).
    MatchRelList {
        input: Box<LogicalPlan>,
        from_var: String,
        to_var: String,
        rel_list_var: String,
        rel_labels: Vec<String>,
        direction: ExpandDirection,
        min_hops: u32,
        max_hops: Option<u32>,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
}
