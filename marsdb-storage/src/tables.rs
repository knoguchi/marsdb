use redb::{MultimapTableDefinition, TableDefinition};

/// Metadata: "next_node_id" / "next_edge_id" / "schema_version" -> counter.
pub const META: TableDefinition<&str, u64> = TableDefinition::new("meta");

/// Label string -> interned u32 id.
pub const LABEL_TO_ID: TableDefinition<&str, u32> = TableDefinition::new("label_to_id");

/// Interned u32 id -> label string.
pub const ID_TO_LABEL: TableDefinition<u32, &str> = TableDefinition::new("id_to_label");

/// node_id -> postcard-encoded NodeRecord.
pub const NODES: TableDefinition<u64, &[u8]> = TableDefinition::new("nodes");

/// edge_id -> postcard-encoded EdgeRecord.
pub const EDGES: TableDefinition<u64, &[u8]> = TableDefinition::new("edges");

/// Outgoing adjacency as a composite-key plain table (v1.5 step 2):
/// `src_node_id(8B BE) ++ label_id(4B BE) ++ edge_id(8B BE)` -> dst
/// node_id. B-tree key order clusters one node's edges together and,
/// within a node, groups them by label — so a typed expansion
/// (`-[:KNOWS]->`) is a prefix `range()` over `node ++ label`, touching
/// only matching entries (`O(matching degree)`), and an untyped expansion
/// is the wider `node ++ *` prefix. The composite-key-on-sorted-KV shape
/// is the industry-standard pattern-B layout for exactly this. Replaces a
/// `node_id -> {20-byte AdjEntry}` multimap whose per-node entries were
/// ordered by edge id, forcing every typed expansion to decode and
/// label-check the node's entire entry set (`O(total degree)`).
pub const ADJ_OUT: TableDefinition<&[u8], u64> = TableDefinition::new("adj_out");

/// Incoming mirror of `ADJ_OUT`: `dst ++ label ++ edge` -> src node_id.
pub const ADJ_IN: TableDefinition<&[u8], u64> = TableDefinition::new("adj_in");

/// label_id -> set of node_ids carrying that label. Secondary index so a
/// label-filtered scan (`NodeByLabelScan`) can look up matching nodes
/// directly instead of scanning every row in `NODES`.
pub const NODE_LABEL_INDEX: MultimapTableDefinition<u32, u64> =
    MultimapTableDefinition::new("node_label_index");

/// Property-name string -> interned u32 id. Mirrors `LABEL_TO_ID` — property
/// names are interned globally (not per-label), since the same property
/// name (`name`, `id`, ...) is common across many labels and there's no
/// benefit to a separate namespace per label.
pub const PROP_TO_ID: TableDefinition<&str, u32> = TableDefinition::new("prop_to_id");

/// Interned u32 id -> property-name string.
pub const ID_TO_PROP: TableDefinition<u32, &str> = TableDefinition::new("id_to_prop");

/// `label_id(4 bytes BE) ++ property_id(4 bytes BE)` -> postcard-encoded
/// `IndexDef` (currently just a `unique` flag). Presence of a key here is
/// what "this (label, property) has a declared index" means — checked by
/// the planner before it can emit an `IndexSeek` and by every mutation path
/// before it needs to maintain `PROPERTY_INDEX`.
pub const INDEX_DEFS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("index_defs");

/// `label_id(4 bytes BE) ++ property_id(4 bytes BE) ++ encoded_value` ->
/// set of node_ids. One shared table for every declared property index
/// (not one table per index) — keeps schema/table lifecycle management
/// simple; the label_id+property_id prefix keeps each index's entries
/// contiguous under redb's own key ordering, which is what a future range
/// scan (`WHERE n.prop > x`) would need anyway. `encoded_value` uses an
/// order-preserving byte encoding (see `marsdb-graph::index_key`) so
/// lexicographic byte comparison matches the real value ordering within one
/// type — cross-type ordering isn't meaningful and isn't relied on.
pub const PROPERTY_INDEX: MultimapTableDefinition<&[u8], u64> =
    MultimapTableDefinition::new("property_index");
