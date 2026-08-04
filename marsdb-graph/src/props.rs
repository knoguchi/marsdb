use marsdb_storage::{ReadableTable, Txn, WriteTransaction};

use crate::error::GraphError;
use crate::id::next_id;

/// Look up (or allocate) the u32 id interned for `prop`, inside a write
/// txn. Mirrors `labels::intern_label` — property names are interned
/// globally, not per-label, since the same name is common across labels.
pub(crate) fn intern_prop(write_txn: &WriteTransaction, prop: &str) -> Result<u32, GraphError> {
    {
        let p2i = write_txn.open_table(marsdb_storage::tables::PROP_TO_ID)?;
        let existing = p2i.get(prop)?.map(|g| g.value());
        if let Some(existing) = existing {
            return Ok(existing);
        }
    }
    let id = next_id(write_txn, "next_prop_id")? as u32;
    {
        let mut p2i = write_txn.open_table(marsdb_storage::tables::PROP_TO_ID)?;
        p2i.insert(prop, id)?;
    }
    {
        let mut i2p = write_txn.open_table(marsdb_storage::tables::ID_TO_PROP)?;
        i2p.insert(id, prop)?;
    }
    Ok(id)
}

/// Resolve a previously interned property id back to its string. Mirrors
/// `labels::resolve_label`.
pub(crate) fn resolve_prop(txn: Txn, prop_id: u32) -> Result<String, GraphError> {
    let i2p = txn.open_table(marsdb_storage::tables::ID_TO_PROP)?;
    let value = i2p
        .get(prop_id)?
        .ok_or_else(|| GraphError::CorruptData(format!("prop id {prop_id} has no interned string")))?;
    Ok(value.value().to_string())
}

/// Look up the id for `prop` without allocating one. `None` means `prop`
/// has never been interned, so it can't be indexed/declared yet.
///
/// `PROP_TO_ID` is created lazily by `intern_prop`'s own write path —
/// against a freshly-opened database that has never interned *any*
/// property (no index ever declared), the table itself doesn't exist yet.
/// A `ReadTransaction`'s `open_table` (unlike a `WriteTransaction`'s,
/// which auto-creates) errors on a missing table rather than treating it
/// as empty, so that specific error is exactly equivalent to "not found"
/// here, not a real failure.
pub(crate) fn lookup_prop_id(txn: Txn, prop: &str) -> Result<Option<u32>, GraphError> {
    let p2i = match txn.open_table(marsdb_storage::tables::PROP_TO_ID) {
        Ok(table) => table,
        Err(marsdb_storage::StorageError::Table(redb::TableError::TableDoesNotExist(_))) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let found = p2i.get(prop)?.map(|g| g.value());
    Ok(found)
}
