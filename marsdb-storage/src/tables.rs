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

/// src node_id -> set of encoded AdjEntry bytes (outgoing edges).
pub const ADJ_OUT: MultimapTableDefinition<u64, &[u8]> = MultimapTableDefinition::new("adj_out");

/// dst node_id -> set of encoded AdjEntry bytes (incoming edges).
pub const ADJ_IN: MultimapTableDefinition<u64, &[u8]> = MultimapTableDefinition::new("adj_in");
