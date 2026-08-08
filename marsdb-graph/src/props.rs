use marsdb_storage::{ReadableTable, Txn};

use crate::error::GraphError;
use crate::id::next_id;
use crate::write_ctx::WriteCtx;

/// Look up (or allocate) the u32 id interned for `prop`, inside a write
/// txn. Mirrors `labels::intern_label` — property names are interned
/// globally, not per-label, since the same name is common across labels.
pub(crate) fn intern_prop(ctx: &mut WriteCtx, prop: &str) -> Result<u32, GraphError> {
    if let Some(existing) = ctx.prop_to_id()?.get(prop)?.map(|g| g.value()) {
        return Ok(existing);
    }
    let id = next_id(ctx, "next_prop_id")? as u32;
    ctx.prop_to_id()?.insert(prop, id)?;
    ctx.id_to_prop()?.insert(id, prop)?;
    Ok(id)
}

/// A prop-id -> name resolver over one pre-opened `ID_TO_PROP` handle —
/// for record decode (`encode::decode_node`/`decode_edge`), which resolves
/// one id per property per record. Opening the table once here instead of
/// per resolution matters because table opens were themselves a measured
/// hot cost (mars-3va, 23.67% of a bulk load) — a resolver that re-opened
/// per prop would reintroduce exactly that.
///
/// Only for read paths that hold no `WriteCtx` — inside a `WriteCtx`-based
/// write method, `ID_TO_PROP` may already be open on the same transaction
/// and a second handle is redb's `TableAlreadyOpen` error; those paths use
/// `index::resolve_prop_ctx` instead.
pub(crate) fn prop_resolver(
    txn: Txn<'_>,
) -> Result<impl FnMut(u32) -> Result<String, GraphError> + '_, GraphError> {
    let i2p = txn.open_table(marsdb_storage::tables::ID_TO_PROP)?;
    Ok(move |prop_id: u32| {
        i2p.get(prop_id)?
            .map(|g| g.value().to_string())
            .ok_or_else(|| {
                GraphError::CorruptData(format!("prop id {prop_id} has no interned string"))
            })
    })
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
        Err(marsdb_storage::StorageError::Table(redb::TableError::TableDoesNotExist(_))) => {
            return Ok(None)
        }
        Err(e) => return Err(e.into()),
    };
    let found = p2i.get(prop)?.map(|g| g.value());
    Ok(found)
}
