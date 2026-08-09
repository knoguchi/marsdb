//! Thin trait boundary over redb. `marsdb-graph` talks to this crate, never to
//! `redb` directly, so the underlying embedded KV engine could be swapped
//! without touching graph/query code. (A hand-rolled replacement engine was
//! prototyped and abandoned -- redb stays; the boundary remains because it
//! costs nothing and keeps the dependency surface honest.)

pub mod tables;

mod error;
pub use error::StorageError;

mod txn;
pub use txn::{MultimapTableHandle, TableHandle, Txn};

// Re-exported so callers can open transactions/tables without a direct redb
// dependency of their own.
pub use redb::{
    MultimapTableDefinition, ReadTransaction, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, ReadableTableMetadata, TableDefinition, WriteTransaction,
};

use std::fs::OpenOptions;
use std::path::Path;

/// Version of the MarsDB-owned tables and record encodings. This is separate
/// from redb's own file-format version.
// v2 (2026-08): directory record format — interned u32 prop-id keys with
// per-property offsets replace the v1 whole-blob postcard map (see
// marsdb-graph/src/encode.rs). v1 files are rejected cleanly; the
// documented path is export from a v1 build, reimport here.
pub const CURRENT_FORMAT_VERSION: u64 = 2;
pub const OLDEST_SUPPORTED_FORMAT_VERSION: u64 = 2;

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
        // Distinguishes "brand-new file" (no tables at all -- `from_db`
        // commits table setup and the version marker atomically, so a
        // crash can't produce a half-initialized state) from a
        // pre-versioning v1-era file (has data tables, but no
        // `schema_version` key). The latter used to be silently adopted
        // and stamped with the current version -- correct when the marker
        // was introduced (the layouts were identical then), but wrong
        // ever since format 2 changed the record encoding: stamping a
        // real v1 file as 2 makes its records decode as garbage later
        // instead of failing cleanly at open.
        let is_fresh = write_txn.list_tables()?.next().is_none()
            && write_txn.list_multimap_tables()?.next().is_none();
        {
            let mut meta = write_txn.open_table(tables::META)?;
            let stored_version = meta.get("schema_version")?.map(|value| value.value());
            match stored_version {
                None if is_fresh => {
                    meta.insert("schema_version", CURRENT_FORMAT_VERSION)?;
                }
                // An existing database with no version marker predates
                // explicit versioning -- format 1 by construction (the
                // marker shipped before format 2 existed, so every
                // format-2 file has one).
                None => {
                    drop(meta);
                    write_txn.abort()?;
                    return Err(StorageError::UnsupportedFormat {
                        found: 1,
                        oldest_supported: OLDEST_SUPPORTED_FORMAT_VERSION,
                        current: CURRENT_FORMAT_VERSION,
                    });
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
            write_txn.open_table(tables::ADJ_OUT)?;
            write_txn.open_table(tables::ADJ_IN)?;
            write_txn.open_table(tables::REL_TYPE_COUNTS)?;
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
            copy_table!(tables::ADJ_OUT);
            copy_table!(tables::ADJ_IN);
            copy_table!(tables::REL_TYPE_COUNTS);
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

    /// A pre-versioning v1-era file (data tables present, no
    /// `schema_version` marker) must be rejected as format 1, not
    /// silently stamped as the current version -- its records are in the
    /// old whole-blob encoding and would decode as garbage.
    #[test]
    fn unversioned_v1_era_database_is_rejected_not_adopted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1-era.redb");
        {
            let db = redb::Database::create(&path).unwrap();
            let write = db.begin_write().unwrap();
            {
                // A v1-era file always has data tables; META may exist
                // too (it held the id counters) -- just no
                // schema_version key.
                write
                    .open_table(tables::NODES)
                    .unwrap()
                    .insert(1, &[0u8][..])
                    .unwrap();
                write
                    .open_table(tables::META)
                    .unwrap()
                    .insert("next_node_id", 1)
                    .unwrap();
            }
            write.commit().unwrap();
        }

        let err = match StorageEngine::open_file(&path) {
            Ok(_) => panic!("unversioned v1-era database unexpectedly opened"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            StorageError::UnsupportedFormat { found: 1, .. }
        ));
        // Rejection must not have stamped a version into the file.
        let db = redb::Database::create(&path).unwrap();
        let read = db.begin_read().unwrap();
        assert!(read
            .open_table(tables::META)
            .unwrap()
            .get("schema_version")
            .unwrap()
            .is_none());
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

    /// Guards against the exact bug this pair of methods once had recurring
    /// the next time a table is added to `tables.rs`: rather than hardcoding
    /// the current table list a second time, this asks redb itself what
    /// tables exist on each side and compares, so a table added to
    /// `from_db` but forgotten in `backup_to` (or vice versa) fails here
    /// instead of silently losing data on the next real backup.
    #[test]
    fn backup_copies_every_table_that_exists_in_the_source() {
        use redb::{MultimapTableHandle as _, TableHandle as _};
        use std::collections::BTreeSet;

        fn table_names(read: &ReadTransaction) -> BTreeSet<String> {
            let mut names: BTreeSet<String> = read
                .list_tables()
                .unwrap()
                .map(|t| t.name().to_string())
                .collect();
            names.extend(
                read.list_multimap_tables()
                    .unwrap()
                    .map(|t| t.name().to_string()),
            );
            names
        }

        let source = StorageEngine::open_memory().unwrap();
        // Touch every table explicitly, not just the ones `from_db` happens
        // to eagerly create -- this must catch a missing `copy_table!` even
        // if a future table is lazily created instead.
        let write = source.begin_write().unwrap();
        write
            .open_table(tables::PROP_TO_ID)
            .unwrap()
            .insert("email", 1)
            .unwrap();
        write
            .open_table(tables::ID_TO_PROP)
            .unwrap()
            .insert(1, "email")
            .unwrap();
        write
            .open_table(tables::INDEX_DEFS)
            .unwrap()
            .insert(&[0u8, 0, 0, 0, 0, 0, 0, 1][..], &[0u8][..])
            .unwrap();
        write
            .open_multimap_table(tables::PROPERTY_INDEX)
            .unwrap()
            .insert(&[0u8, 0, 0, 0, 0, 0, 0, 1][..], 6)
            .unwrap();
        write.commit().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("completeness.redb");
        source.backup_to(&path).unwrap();
        let backup = StorageEngine::open_file(&path).unwrap();

        let source_tables = table_names(&source.begin_read().unwrap());
        let backup_tables = table_names(&backup.begin_read().unwrap());
        assert_eq!(source_tables, backup_tables);
    }
}
