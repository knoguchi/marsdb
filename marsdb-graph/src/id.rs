use marsdb_storage::ReadableTable;

use crate::error::GraphError;
use crate::write_ctx::WriteCtx;

/// Increment and return the counter stored at `key` in the `meta` table,
/// within the given write transaction. Allocation is only durable once the
/// caller commits the underlying transaction, keeping id allocation inside
/// the same crash-safety boundary as the row(s) it's used for.
pub(crate) fn next_id(ctx: &mut WriteCtx, key: &str) -> Result<u64, GraphError> {
    let current = ctx.meta()?.get(key)?.map(|g| g.value()).unwrap_or(0);
    let next = current + 1;
    ctx.meta()?.insert(key, next)?;
    Ok(next)
}
