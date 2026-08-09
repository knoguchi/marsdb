use crate::value::Value;

#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    /// What the statement changed — the answer to "how many did my
    /// DELETE delete", which was previously unanswerable at any layer
    /// (every write statement reported only its RETURN rows, usually
    /// none). All-zero for read-only statements.
    pub stats: QueryStats,
}

/// Per-statement write counters, following the widely-used summary-
/// counter conventions: `properties_set` counts removals too (removing
/// a property is setting it away, and `SET n.p = null` is literally the
/// same operation as `REMOVE n.p` here), while label changes are
/// tracked as their own pair.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryStats {
    pub nodes_created: u64,
    pub nodes_deleted: u64,
    pub relationships_created: u64,
    pub relationships_deleted: u64,
    pub properties_set: u64,
    pub labels_added: u64,
    pub labels_removed: u64,
}

impl QueryStats {
    /// True when the statement changed nothing — lets output layers
    /// (the CLI, the C ABI's JSON) skip stats noise for pure reads.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}
