use marsdb_storage::{ReadableTable, WriteTransaction};

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

/// Resolve a previously interned label id back to its string.
///
/// Takes `&WriteTransaction` (not `&ReadTransaction`) even for read-only
/// callers: v1 drives an entire statement through one `WriteTransaction` —
/// see `GraphStore::begin_write` — so every table access in this crate goes
/// through that one transaction type, read or write.
pub(crate) fn resolve_label(write_txn: &WriteTransaction, label_id: u32) -> Result<String, GraphError> {
    let i2l = write_txn.open_table(marsdb_storage::tables::ID_TO_LABEL)?;
    let value = i2l
        .get(label_id)?
        .expect("label id present in nodes/edges table must be interned");
    Ok(value.value().to_string())
}

/// Look up the id for `label` without allocating one. Returns `None` if the
/// label has never been used, meaning no rows can reference it.
pub(crate) fn lookup_label_id(write_txn: &WriteTransaction, label: &str) -> Result<Option<u32>, GraphError> {
    let l2i = write_txn.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
    let found = l2i.get(label)?.map(|g| g.value());
    Ok(found)
}
