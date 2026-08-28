//! Built-in schema-introspection procedures (`CALL db.labels()`, ...).
//!
//! These live in the executor, not behind `ProcedureProvider`, because
//! they need the statement's own transaction; a provider's `call` only
//! gets values, so embedder procedures can't reach into storage.
//! `Executor::call_procedure` dispatches built-ins first, then the
//! embedder's provider — `db.*` names are reserved and not shadowable.
//!
//! `db.labels`/`db.relationshipTypes` also yield a `count` column beyond
//! the conventional single column, since the backing O(1) statistics
//! (`NODE_LABEL_INDEX`, `REL_TYPE_COUNTS`) make it free.

use marsdb_graph::{GraphStore, Txn};

use crate::ast::Literal;
use crate::error::QueryError;
use crate::procedure::ProcedureSignature;
use crate::value::Value;

/// Signature lookup for built-in names; `None` = not a built-in (fall
/// through to the embedder's provider).
pub(crate) fn signature(name: &str) -> Option<ProcedureSignature> {
    let (inputs, outputs): (Vec<&str>, Vec<&str>) = match name {
        "db.labels" => (vec![], vec!["label", "count"]),
        "db.relationshipTypes" => (vec![], vec!["relationshipType", "count"]),
        "db.propertyKeys" => (vec![], vec!["propertyKey"]),
        "db.indexes" => (vec![], vec!["label", "property", "unique"]),
        _ => return None,
    };
    Some(ProcedureSignature {
        inputs: inputs.iter().map(|s| s.to_string()).collect(),
        input_types: vec![],
        outputs: outputs.iter().map(|s| s.to_string()).collect(),
    })
}

/// Produce a built-in's rows. Only called with a name `signature`
/// accepted; arity was already checked against the (empty) input list.
pub(crate) fn call(txn: Txn, name: &str) -> Result<Vec<Vec<Value>>, QueryError> {
    let string = |s: String| Value::Literal(Literal::String(s));
    let int = |n: u64| Value::Literal(Literal::Int(n as i64));
    match name {
        "db.labels" => Ok(GraphStore::list_node_labels_in_txn(txn)?
            .into_iter()
            .map(|(label, count)| vec![string(label), int(count)])
            .collect()),
        "db.relationshipTypes" => Ok(GraphStore::list_rel_types_in_txn(txn)?
            .into_iter()
            .map(|(rel_type, count)| vec![string(rel_type), int(count)])
            .collect()),
        "db.propertyKeys" => Ok(GraphStore::list_property_keys_in_txn(txn)?
            .into_iter()
            .map(|key| vec![string(key)])
            .collect()),
        "db.indexes" => Ok(GraphStore::list_indexes_in_txn(txn)?
            .into_iter()
            .map(|(label, property, unique)| {
                vec![
                    string(label),
                    string(property),
                    Value::Literal(Literal::Bool(unique)),
                ]
            })
            .collect()),
        _ => unreachable!("call() only invoked for names signature() accepted"),
    }
}
