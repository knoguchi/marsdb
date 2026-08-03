//! Thin trait boundary over redb. `marsdb-graph` talks to this crate, never to
//! `redb` directly, so the underlying embedded KV engine can be swapped later
//! (see plan v2: hand-rolled storage engine) without touching graph/query code.

pub mod tables;

mod error;
pub use error::StorageError;

// Re-exported so callers can open transactions/tables without a direct redb
// dependency of their own.
pub use redb::{
    MultimapTableDefinition, ReadTransaction, ReadableMultimapTable, ReadableTable,
    TableDefinition, WriteTransaction,
};

use std::path::Path;

pub struct StorageEngine {
    db: redb::Database,
}

impl StorageEngine {
    /// Open (creating if absent) a single-file, on-disk database.
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let db = redb::Database::create(path)?;
        Self::from_db(db)
    }

    /// Open a purely in-memory database. Nothing is written to disk and all
    /// data is lost when the `StorageEngine` is dropped.
    pub fn open_memory() -> Result<Self, StorageError> {
        let backend = redb::backends::InMemoryBackend::new();
        let db = redb::Database::builder().create_with_backend(backend)?;
        Self::from_db(db)
    }

    /// redb only creates a table on its first write-mode open; a table
    /// nobody has ever written to doesn't exist yet, and reading from it
    /// errors instead of returning empty. Eagerly open (and thus create)
    /// every table up front so read paths never have to special-case "brand
    /// new, still-empty database" as an error.
    fn from_db(db: redb::Database) -> Result<Self, StorageError> {
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(tables::META)?;
            write_txn.open_table(tables::LABEL_TO_ID)?;
            write_txn.open_table(tables::ID_TO_LABEL)?;
            write_txn.open_table(tables::NODES)?;
            write_txn.open_table(tables::EDGES)?;
            write_txn.open_multimap_table(tables::ADJ_OUT)?;
            write_txn.open_multimap_table(tables::ADJ_IN)?;
            write_txn.open_multimap_table(tables::NODE_LABEL_INDEX)?;
        }
        write_txn.commit()?;
        Ok(Self { db })
    }

    pub fn begin_write(&self) -> Result<WriteTransaction, StorageError> {
        Ok(self.db.begin_write()?)
    }

    pub fn begin_read(&self) -> Result<ReadTransaction, StorageError> {
        Ok(self.db.begin_read()?)
    }
}
