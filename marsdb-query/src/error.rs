/// Typed error taxonomy, in three tiers that reflect *when* the problem
/// was knowable:
///
/// - [`Syntax`](QueryError::Syntax): the query text itself is malformed —
///   never reached planning/execution at all (`parser.rs`, including
///   `pest`'s own grammar failures).
/// - [`Semantic`](QueryError::Semantic): the query text parsed fine but
///   describes something structurally invalid — knowable from the query
///   alone, no data/parameters needed (an unsupported pattern shape, an
///   aggregate nested somewhere it can't be, `EXPLAIN EXPLAIN`, ...).
/// - [`Type`](QueryError::Type): only knowable once a real value (from
///   stored data or a `$parameter`) is in hand and turns out to be the
///   wrong shape (arithmetic on a non-number, indexing a non-list, a
///   `date({...})` field of the wrong type, ...).
///
/// Callers that only need "did it work" (most of this codebase) keep
/// using `Display`/`?` as before — this tiering exists for callers that
/// want to react differently to "you wrote something illegal" vs "the
/// data didn't match what the query assumed" (an application surfacing
/// user-facing messages, or `ExecutionOutcome`'s telemetry categories).
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("syntax error: {0}")]
    Syntax(String),
    #[error("semantic error: {0}")]
    Semantic(String),
    #[error("type error: {0}")]
    Type(String),
    #[error("graph error: {0}")]
    Graph(#[from] marsdb_graph::GraphError),
    #[error("unbound variable: {0}")]
    UnboundVariable(String),
    #[error("missing value for parameter: ${0}")]
    MissingParam(String),
    #[error("query cancelled")]
    Cancelled,
    #[error("query timed out")]
    Timeout,
    #[error("query resource limit exceeded: {0}")]
    ResourceLimit(String),
}
