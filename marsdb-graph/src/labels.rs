use marsdb_storage::{ReadableTable, Txn};

use crate::error::GraphError;
use crate::id::next_id;
use crate::write_ctx::WriteCtx;

/// Look up (or allocate) the u32 id interned for `label`, inside a write txn.
pub(crate) fn intern_label(ctx: &mut WriteCtx, label: &str) -> Result<u32, GraphError> {
    if let Some(existing) = ctx.label_to_id()?.get(label)?.map(|g| g.value()) {
        return Ok(existing);
    }
    let id = next_id(ctx, "next_label_id")? as u32;
    ctx.label_to_id()?.insert(label, id)?;
    ctx.id_to_label()?.insert(id, label)?;
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
/// `LABEL_TO_ID` is created lazily by `intern_label`'s own write path, so
/// against a database that has never created a node/edge the table doesn't
/// exist yet. A `ReadTransaction`'s `open_table` errors on a missing table
/// rather than treating it as empty (unlike a `WriteTransaction`'s, which
/// auto-creates); that error is treated as "not found" here, not a failure.
pub(crate) fn lookup_label_id(txn: Txn, label: &str) -> Result<Option<u32>, GraphError> {
    let l2i = match txn.open_table(marsdb_storage::tables::LABEL_TO_ID) {
        Ok(table) => table,
        Err(marsdb_storage::StorageError::Table(redb::TableError::TableDoesNotExist(_))) => {
            return Ok(None)
        }
        Err(e) => return Err(e.into()),
    };
    let found = l2i.get(label)?.map(|g| g.value());
    Ok(found)
}
