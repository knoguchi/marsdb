use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Database(redb::DatabaseError),
    Transaction(Box<redb::TransactionError>),
    Table(redb::TableError),
    Storage(redb::StorageError),
    Commit(redb::CommitError),
    UnsupportedFormat {
        found: u64,
        oldest_supported: u64,
        current: u64,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "I/O error: {e}"),
            StorageError::Database(e) => write!(f, "database error: {e}"),
            StorageError::Transaction(e) => write!(f, "transaction error: {e}"),
            StorageError::Table(e) => write!(f, "table error: {e}"),
            StorageError::Storage(e) => write!(f, "storage error: {e}"),
            StorageError::Commit(e) => write!(f, "commit error: {e}"),
            StorageError::UnsupportedFormat {
                found,
                oldest_supported,
                current,
            } => write!(
                f,
                "unsupported database format version {found}; this build supports {oldest_supported}..={current}"
            ),
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl std::error::Error for StorageError {}

impl From<redb::DatabaseError> for StorageError {
    fn from(e: redb::DatabaseError) -> Self {
        StorageError::Database(e)
    }
}

impl From<redb::TransactionError> for StorageError {
    fn from(e: redb::TransactionError) -> Self {
        StorageError::Transaction(Box::new(e))
    }
}

impl From<redb::TableError> for StorageError {
    fn from(e: redb::TableError) -> Self {
        StorageError::Table(e)
    }
}

impl From<redb::StorageError> for StorageError {
    fn from(e: redb::StorageError) -> Self {
        StorageError::Storage(e)
    }
}

impl From<redb::CommitError> for StorageError {
    fn from(e: redb::CommitError) -> Self {
        StorageError::Commit(e)
    }
}
