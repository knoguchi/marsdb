use std::collections::BTreeMap;

use marsdb_graph::{Edge, Node, PropertyValue};

use crate::ast::Literal;

/// One element of a `Value::Path` — a path is `node, edge, node, edge,
/// ..., node`, alternating, as a single `Vec`, not two parallel node/edge
/// vecs (which would create an unenforced `nodes.len() == edges.len() +
/// 1` invariant across every place a path gets built or read).
#[derive(Debug, Clone)]
pub enum PathElem {
    Node(Node),
    Edge(Edge),
}

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
    /// A named path (`MATCH p = (a)-->(b) RETURN p`) or a `shortestPath()`
    /// result — see `Binding::Path`'s docs (executor.rs) for how this
    /// gets assembled during MATCH evaluation.
    Path(Vec<PathElem>),
    /// A map literal (`{a: 1, b: 2}`) — like `List`, a query-layer-only
    /// concept, never persisted as a `PropertyValue` (nothing in the
    /// grammar can construct a map literal to store as a node/edge
    /// property; node/edge props are still each individually a scalar
    /// `PropertyValue`). `BTreeMap`, not `HashMap` — canonical key order
    /// makes display/comparison deterministic without a separate sort
    /// step.
    Map(BTreeMap<String, Value>),
    Null,
}
