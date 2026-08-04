use crate::model::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("storage error: {0}")]
    Storage(#[from] marsdb_storage::StorageError),
    #[error("table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("storage error: {0}")]
    RedbStorage(#[from] redb::StorageError),
    #[error("commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("encode error: {0}")]
    Encode(#[from] postcard::Error),
    #[error("node {0:?} has incident edges; use detach delete")]
    NodeHasEdges(NodeId),
    #[error("node {0:?} does not exist")]
    NodeNotFound(NodeId),
    #[error("corrupt database: {0}")]
    CorruptData(String),
    #[error("unique constraint violation: label {label:?} property {property:?} already has a node with this value")]
    UniqueConstraintViolation { label: String, property: String },
    #[error("no index on label {label:?} property {property:?}")]
    IndexNotFound { label: String, property: String },
}
