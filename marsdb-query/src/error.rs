#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("parse error: {0}")]
    Parse(String),
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
