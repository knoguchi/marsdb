use marsdb_storage::{ReadableTable, Txn, WriteTransaction};

use crate::error::GraphError;
use crate::id::next_id;

/// Look up (or allocate) the u32 id interned for `label`, inside a write txn.
pub(crate) fn intern_label(write_txn: &WriteTransaction, label: &str) -> Result<u32, GraphError> {
    {
        let l2i = write_txn.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
        let existing = l2i.get(label)?.map(|g| g.value());
        if let Some(existing) = existing {
            return Ok(existing);
        }
    }
    let id = next_id(write_txn, "next_label_id")? as u32;
    {
        let mut l2i = write_txn.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
        l2i.insert(label, id)?;
    }
    {
        let mut i2l = write_txn.open_table(marsdb_storage::tables::ID_TO_LABEL)?;
        i2l.insert(id, label)?;
    }
    Ok(id)
}

/// Resolve a previously interned label id back to its string. Read-only —
/// takes `Txn` so it works against either a `WriteTransaction` (a write
/// statement's crash-safety boundary — see `GraphStore::begin_write`) or a
/// `ReadTransaction` (a read-only statement, run without contending for
/// redb's single-writer lock — see `GraphStore::begin_read`).
pub(crate) fn resolve_label(txn: Txn, label_id: u32) -> Result<String, GraphError> {
    let i2l = txn.open_table(marsdb_storage::tables::ID_TO_LABEL)?;
    let value = i2l.get(label_id)?.ok_or_else(|| {
        GraphError::CorruptData(format!("label id {label_id} has no interned string"))
    })?;
    Ok(value.value().to_string())
}

/// Look up the id for `label` without allocating one. Returns `None` if the
/// label has never been used, meaning no rows can reference it. Read-only —
/// see `resolve_label`'s doc comment for why this takes `Txn`.
///
/// `LABEL_TO_ID` is created lazily by `intern_label`'s own write path —
/// against a database that has never created a single node/edge, the
/// table doesn't exist yet, and a `ReadTransaction`'s `open_table` (unlike
/// a `WriteTransaction`'s, which auto-creates) errors on that rather than
/// treating it as empty. That specific error is exactly "not found" here,
/// not a real failure — found via a real test (`create_index` against a
/// property that was never interned hit the equivalent gap in
/// `lookup_prop_id`; this is the same latent issue in its label counterpart).
pub(crate) fn lookup_label_id(txn: Txn, label: &str) -> Result<Option<u32>, GraphError> {
    let l2i = match txn.open_table(marsdb_storage::tables::LABEL_TO_ID) {
        Ok(table) => table,
        Err(marsdb_storage::StorageError::Table(redb::TableError::TableDoesNotExist(_))) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let found = l2i.get(label)?.map(|g| g.value());
    Ok(found)
}
