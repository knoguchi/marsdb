use std::collections::BTreeMap;

use marsdb_graph::{Edge, Node, PropertyValue};

use crate::ast::Literal;

/// One element of a `Value::Path` — a path is `node, edge, node, edge,
/// ..., node`, alternating, as a single `Vec` rather than parallel
/// node/edge vecs, avoiding an unenforced length-relationship invariant.
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
    /// A list literal, `collect()` result, or a list-valued property read
    /// back from storage (`PropertyValue::List` always converts to this,
    /// see `executor::property_value_to_value`), so list operations
    /// (indexing, `size()`, `IN`, `UNWIND`, ...) match one variant.
    List(Vec<Value>),
    /// A named path (`MATCH p = (a)-->(b) RETURN p`) or a `shortestPath()`
    /// result — see `Binding::Path`'s docs (executor.rs).
    Path(Vec<PathElem>),
    /// A map literal (`{a: 1, b: 2}`) — query-layer-only, never persisted
    /// as a `PropertyValue`; a `CREATE {...}` prop map stores each value
    /// as its own scalar `PropertyValue` (`Executor::eval_props_to_values`).
    /// Also used as `date(...)`/`duration(...)` constructor arguments.
    /// `BTreeMap` for deterministic key order in display/comparison.
    Map(BTreeMap<String, Value>),
    Null,
}
