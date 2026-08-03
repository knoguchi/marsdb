use marsdb_graph::{Edge, Node, PropertyValue};

use crate::ast::Literal;

#[derive(Debug, Clone)]
pub enum Value {
    Node(Node),
    Edge(Edge),
    Property(PropertyValue),
    Literal(Literal),
    Null,
}
