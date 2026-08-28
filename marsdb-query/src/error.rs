/// Typed error taxonomy, in three tiers reflecting *when* the problem was
/// knowable: [`Syntax`](QueryError::Syntax) (malformed query text),
/// [`Semantic`](QueryError::Semantic) (parses fine, structurally invalid
/// independent of data), and [`Type`](QueryError::Type) (only knowable
/// once a stored or parameter value turns out the wrong shape). Most
/// callers just use `Display`/`?`; the tiering is for callers that need
/// to distinguish "illegal query" from "data didn't match" (e.g.
/// user-facing messages or telemetry categories).
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
