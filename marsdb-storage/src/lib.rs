//! Thin trait boundary over redb. `marsdb-graph` talks to this crate, never to
//! `redb` directly, so the underlying embedded KV engine can be swapped later
//! (see plan v2: hand-rolled storage engine) without touching graph/query code.

pub mod tables;

mod error;
pub use error::StorageError;

mod txn;
pub use txn::{MultimapTableHandle, TableHandle, Txn};

// Re-exported so callers can open transactions/tables without a direct redb
// dependency of their own.
pub use redb::{
    MultimapTableDefinition, ReadTransaction, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, TableDefinition, WriteTransaction,
};

use std::fs::OpenOptions;
use std::path::Path;

/// Version of the MarsDB-owned tables and record encodings. This is separate
/// from redb's own file-format version.
pub const CURRENT_FORMAT_VERSION: u64 = 1;
pub const OLDEST_SUPPORTED_FORMAT_VERSION: u64 = 1;

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
            let mut meta = write_txn.open_table(tables::META)?;
            let stored_version = meta.get("schema_version")?.map(|value| value.value());
            match stored_version {
                // Databases created before explicit versioning used the same
                // v1 table and postcard layouts, so adopting them is the v1
                // migration. Persist the marker atomically with table setup.
                None => {
                    meta.insert("schema_version", CURRENT_FORMAT_VERSION)?;
                }
                Some(found)
                    if !(OLDEST_SUPPORTED_FORMAT_VERSION..=CURRENT_FORMAT_VERSION)
                        .contains(&found) =>
                {
                    drop(meta);
                    write_txn.abort()?;
                    return Err(StorageError::UnsupportedFormat {
                        found,
                        oldest_supported: OLDEST_SUPPORTED_FORMAT_VERSION,
                        current: CURRENT_FORMAT_VERSION,
                    });
                }
                Some(_) => {}
            }
            drop(meta);
            write_txn.open_table(tables::LABEL_TO_ID)?;
            write_txn.open_table(tables::ID_TO_LABEL)?;
            write_txn.open_table(tables::NODES)?;
            write_txn.open_table(tables::EDGES)?;
            write_txn.open_multimap_table(tables::ADJ_OUT)?;
            write_txn.open_multimap_table(tables::ADJ_IN)?;
            write_txn.open_multimap_table(tables::NODE_LABEL_INDEX)?;
            write_txn.open_table(tables::PROP_TO_ID)?;
            write_txn.open_table(tables::ID_TO_PROP)?;
            write_txn.open_table(tables::INDEX_DEFS)?;
            write_txn.open_multimap_table(tables::PROPERTY_INDEX)?;
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

    /// Run redb's physical checksum/allocation integrity check. A `false`
    /// result means damage was found and repaired; an unrecoverable database
    /// is returned as an error.
    pub fn check_integrity(&mut self) -> Result<bool, StorageError> {
        Ok(self.db.check_integrity()?)
    }

    /// Write a transactionally consistent copy of every MarsDB table to a
    /// new database file. The destination is created exclusively so an
    /// existing file is never silently overwritten.
    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)?;

        let result = (|| {
            let source = self.db.begin_read()?;
            let destination = redb::Database::builder().create_file(file)?;
            let write = destination.begin_write()?;

            macro_rules! copy_table {
                ($definition:expr) => {{
                    let source_table = source.open_table($definition)?;
                    let mut destination_table = write.open_table($definition)?;
                    for entry in source_table.iter()? {
                        let (key, value) = entry?;
                        destination_table.insert(key.value(), value.value())?;
                    }
                }};
            }

            macro_rules! copy_multimap {
                ($definition:expr) => {{
                    let source_table = source.open_multimap_table($definition)?;
                    let mut destination_table = write.open_multimap_table($definition)?;
                    for entry in source_table.iter()? {
                        let (key, values) = entry?;
                        for value in values {
                            destination_table.insert(key.value(), value?.value())?;
                        }
                    }
                }};
            }

            copy_table!(tables::META);
            copy_table!(tables::LABEL_TO_ID);
            copy_table!(tables::ID_TO_LABEL);
            copy_table!(tables::NODES);
            copy_table!(tables::EDGES);
            copy_multimap!(tables::ADJ_OUT);
            copy_multimap!(tables::ADJ_IN);
            copy_multimap!(tables::NODE_LABEL_INDEX);
            copy_table!(tables::PROP_TO_ID);
            copy_table!(tables::ID_TO_PROP);
            copy_table!(tables::INDEX_DEFS);
            copy_multimap!(tables::PROPERTY_INDEX);

            write.commit()?;
            Ok::<(), StorageError>(())
        })();

        if result.is_err() {
            // This file was created exclusively above, so removing an
            // incomplete backup cannot affect pre-existing user data.
            let _ = std::fs::remove_file(path);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_records_current_format_version() {
        let engine = StorageEngine::open_memory().unwrap();
        let read = engine.begin_read().unwrap();
        let meta = read.open_table(tables::META).unwrap();
        assert_eq!(
            meta.get("schema_version").unwrap().unwrap().value(),
            CURRENT_FORMAT_VERSION
        );
    }

    #[test]
    fn database_from_newer_marsdb_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("future.redb");
        {
            let db = redb::Database::create(&path).unwrap();
            let write = db.begin_write().unwrap();
            {
                let mut meta = write.open_table(tables::META).unwrap();
                meta.insert("schema_version", CURRENT_FORMAT_VERSION + 1)
                    .unwrap();
            }
            write.commit().unwrap();
        }

        let err = match StorageEngine::open_file(&path) {
            Ok(_) => panic!("newer database format unexpectedly opened"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            StorageError::UnsupportedFormat {
                found,
                current: CURRENT_FORMAT_VERSION,
                ..
            } if found == CURRENT_FORMAT_VERSION + 1
        ));
    }

    #[test]
    fn backup_copies_all_tables_and_refuses_to_overwrite() {
        let source = StorageEngine::open_memory().unwrap();
        let write = source.begin_write().unwrap();
        {
            write
                .open_table(tables::META)
                .unwrap()
                .insert("next_node_id", 7)
                .unwrap();
            write
                .open_table(tables::LABEL_TO_ID)
                .unwrap()
                .insert("Person", 3)
                .unwrap();
            write
                .open_table(tables::ID_TO_LABEL)
                .unwrap()
                .insert(3, "Person")
                .unwrap();
            write
                .open_table(tables::NODES)
                .unwrap()
                .insert(6, &[1, 2, 3][..])
                .unwrap();
            write
                .open_multimap_table(tables::NODE_LABEL_INDEX)
                .unwrap()
                .insert(3, 6)
                .unwrap();
        }
        write.commit().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.redb");
        source.backup_to(&path).unwrap();

        let backup = StorageEngine::open_file(&path).unwrap();
        let read = backup.begin_read().unwrap();
        assert_eq!(
            read.open_table(tables::META)
                .unwrap()
                .get("next_node_id")
                .unwrap()
                .unwrap()
                .value(),
            7
        );
        assert_eq!(
            read.open_table(tables::NODES)
                .unwrap()
                .get(6)
                .unwrap()
                .unwrap()
                .value(),
            &[1, 2, 3]
        );
        assert_eq!(
            read.open_multimap_table(tables::NODE_LABEL_INDEX)
                .unwrap()
                .get(3)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .value(),
            6
        );

        assert!(matches!(source.backup_to(&path), Err(StorageError::Io(_))));
    }
}
