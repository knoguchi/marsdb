use marsdb_graph::{Edge, Node, PropertyValue};

use crate::ast::Literal;

#[derive(Debug, Clone)]
pub enum Value {
    Node(Node),
    Edge(Edge),
    Property(PropertyValue),
    Literal(Literal),
    /// A `collect()` result — a query-layer-only concept, not persisted as
    /// a `PropertyValue` (nothing in the grammar can construct a list
    /// literal to store as a node/edge property).
    List(Vec<Value>),
    Null,
}
