//! Language-agnostic logical plan for the read path (MATCH traversal).
//!
//! Deliberately close to Neo4j's Cypher runtime operator shape, and maps
//! ~1:1 onto TinkerPop Gremlin traversal steps (`g.V().hasLabel(X)` ->
//! NodeByLabelScan+Filter, `.out('REL')` -> Expand, `.limit(n)` -> Limit) so
//! a future Gremlin frontend can compile into this same IR without
//! redesigning the executor.
//!
//! CREATE has no traversal/filtering semantics (it only ever produces new
//! rows), so it's executed directly from the AST rather than through this
//! IR — see `executor::execute_create`.

use crate::ast::Expr;

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
    Expand {
        input: Box<LogicalPlan>,
        from_var: String,
        to_var: String,
        rel_var: Option<String>,
        rel_label: Option<String>,
        direction: ExpandDirection,
    },
    /// `[:TYPE*min..max]` — BFS from each input row's bound node instead of
    /// a single fixed hop. `rel_var` isn't meaningful here (a variable-
    /// length pattern binds a *list* of relationships in real Cypher; v1
    /// doesn't support that, so a `rel_var` on a variable-length pattern is
    /// rejected by the planner rather than silently bound to just the last
    /// hop's edge).
    VarExpand {
        input: Box<LogicalPlan>,
        from_var: String,
        to_var: String,
        rel_label: Option<String>,
        direction: ExpandDirection,
        min_hops: u32,
        max_hops: Option<u32>,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    Limit {
        input: Box<LogicalPlan>,
        count: i64,
    },
}
