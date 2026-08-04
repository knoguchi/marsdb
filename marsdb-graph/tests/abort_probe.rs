use std::collections::BTreeMap;

use marsdb_graph::GraphStore;

/// Deterministic probe (no process kill / timing involved): open a write
/// txn, insert many nodes via the same pattern the CREATE executor uses,
/// then explicitly abort instead of commit. If the abort is honored, a
/// reopened store must see zero nodes.
#[test]
fn many_inserts_then_explicit_abort_leaves_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("probe.db");

    {
        let store = GraphStore::open_file(&path).unwrap();
        let write_txn = store.begin_write().unwrap();
        for i in 0..3000 {
            let mut props = BTreeMap::new();
            props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
            GraphStore::create_node_in_txn(&write_txn, &["Item"], props).unwrap();
        }
        GraphStore::abort(write_txn).unwrap();
    }

    let store = GraphStore::open_file(&path).unwrap();
    let nodes = store.all_nodes(Some("Item")).unwrap();
    assert_eq!(
        nodes.len(),
        0,
        "aborted transaction leaked {} nodes",
        nodes.len()
    );
}
