use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub props: BTreeMap<String, PropertyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub label: String,
    pub src: NodeId,
    pub dst: NodeId,
    pub props: BTreeMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Out,
    In,
}

/// A traversal-hop candidate read directly from an adjacency multimap entry,
/// without touching the `edges`/`nodes` tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjEntry {
    pub edge_id: EdgeId,
    pub other: NodeId,
    pub label_id: u32,
}

impl AdjEntry {
    pub(crate) fn encode(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0..8].copy_from_slice(&self.edge_id.0.to_be_bytes());
        buf[8..16].copy_from_slice(&self.other.0.to_be_bytes());
        buf[16..20].copy_from_slice(&self.label_id.to_be_bytes());
        buf
    }

    pub(crate) fn decode(bytes: &[u8]) -> Self {
        let edge_id = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let other = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let label_id = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        AdjEntry {
            edge_id: EdgeId(edge_id),
            other: NodeId(other),
            label_id,
        }
    }
}
