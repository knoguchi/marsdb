use marsdb_storage::{ReadableTable, WriteTransaction};

use crate::error::GraphError;

/// Increment and return the counter stored at `key` in the `meta` table,
/// within the given write transaction. Allocation is only durable once the
/// caller commits `write_txn`, keeping id allocation inside the same
/// crash-safety boundary as the row(s) it's used for.
pub(crate) fn next_id(write_txn: &WriteTransaction, key: &str) -> Result<u64, GraphError> {
    let mut meta = write_txn.open_table(marsdb_storage::tables::META)?;
    let current = meta.get(key)?.map(|g| g.value()).unwrap_or(0);
    let next = current + 1;
    meta.insert(key, next)?;
    Ok(next)
}
