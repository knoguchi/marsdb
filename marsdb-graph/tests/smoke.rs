use std::collections::BTreeMap;

use marsdb_graph::{Direction, GraphStore, PropertyValue};

#[test]
fn create_and_traverse_in_memory() {
    let store = GraphStore::open_memory().unwrap();

    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();

    let mut bob_props = BTreeMap::new();
    bob_props.insert("name".to_string(), PropertyValue::String("Bob".into()));
    let bob = store.create_node(&["Person"], bob_props).unwrap();

    store
        .create_edge("KNOWS", alice, bob, BTreeMap::new())
        .unwrap();

    let node = store.get_node(alice).unwrap().unwrap();
    assert_eq!(node.labels, vec!["Person".to_string()]);
    assert_eq!(
        node.props.get("name"),
        Some(&PropertyValue::String("Alice".into()))
    );

    let out = store.neighbors(alice, Direction::Out, None).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].other, bob);

    let filtered = store
        .neighbors(alice, Direction::Out, Some("KNOWS"))
        .unwrap();
    assert_eq!(filtered.len(), 1);

    let missing_label = store
        .neighbors(alice, Direction::Out, Some("FOLLOWS"))
        .unwrap();
    assert_eq!(missing_label.len(), 0);

    let in_edges = store.neighbors(bob, Direction::In, None).unwrap();
    assert_eq!(in_edges.len(), 1);
    assert_eq!(in_edges[0].other, alice);

    let all = store.all_nodes(Some("Person")).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn create_edge_rejects_missing_endpoints() {
    let store = GraphStore::open_memory().unwrap();
    let live = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let missing = marsdb_graph::NodeId(u64::MAX);

    let err = store
        .create_edge("KNOWS", live, missing, BTreeMap::new())
        .unwrap_err();
    assert!(matches!(err, marsdb_graph::GraphError::NodeNotFound(id) if id == missing));
    assert!(store
        .neighbors(live, Direction::Out, None)
        .unwrap()
        .is_empty());
}

#[test]
fn delete_node_detach() {
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", a, b, BTreeMap::new()).unwrap();

    let err = store.delete_node(a, false).unwrap_err();
    assert!(matches!(err, marsdb_graph::GraphError::NodeHasEdges(_)));

    assert!(store.delete_node(a, true).unwrap());
    assert!(store.get_node(a).unwrap().is_none());
    assert_eq!(store.neighbors(b, Direction::In, None).unwrap().len(), 0);
}

/// Regression test for the label index (`NODE_LABEL_INDEX`): a deleted
/// multi-label node must disappear from every label bucket it was in, not
/// just the `nodes` table, and a label-filtered scan must never return a
/// stale/dangling node id.
#[test]
fn label_index_tracks_multi_label_create_and_delete() {
    let store = GraphStore::open_memory().unwrap();
    let post = store
        .create_node(&["Post", "Message"], BTreeMap::new())
        .unwrap();
    let comment = store
        .create_node(&["Comment", "Message"], BTreeMap::new())
        .unwrap();
    let _person = store.create_node(&["Person"], BTreeMap::new()).unwrap();

    let messages = store.all_nodes(Some("Message")).unwrap();
    assert_eq!(messages.len(), 2);
    assert!(messages.iter().any(|n| n.id == post));
    assert!(messages.iter().any(|n| n.id == comment));

    assert_eq!(store.all_nodes(Some("Post")).unwrap().len(), 1);
    assert_eq!(store.all_nodes(Some("Person")).unwrap().len(), 1);
    assert_eq!(store.all_nodes(Some("NoSuchLabel")).unwrap().len(), 0);

    assert!(store.delete_node(post, false).unwrap());

    // `post` carried both `Post` and `Message` — deleting it must clear both
    // buckets, not just the one a naive single-label implementation would
    // remember.
    let messages_after = store.all_nodes(Some("Message")).unwrap();
    assert_eq!(messages_after.len(), 1);
    assert_eq!(messages_after[0].id, comment);
    assert_eq!(store.all_nodes(Some("Post")).unwrap().len(), 0);
    assert_eq!(store.all_nodes(Some("Person")).unwrap().len(), 1);
}

/// `PropertyValue::Date`/`Duration` round-trip through real postcard
/// encode/decode (`store.create_node`/`get_node` go through `encode.rs`'s
/// `encode`/`decode`, not an in-memory shortcut) unchanged -- the whole
/// point of giving them first-class variants instead of reusing `Int`/
/// `String` (see `PropertyValue`'s doc comment) is that this survives the
/// storage boundary exactly, not just within one query's execution.
#[test]
fn date_and_duration_properties_round_trip_through_storage() {
    let store = GraphStore::open_memory().unwrap();
    let mut props = BTreeMap::new();
    // 1984-10-11, as an epoch-day count -- see temporal.rs's
    // `epoch_day_from_ymd` for how the query layer derives this same
    // value from calendar year/month/day.
    props.insert("date".to_string(), PropertyValue::Date(5397));
    props.insert(
        "duration".to_string(),
        PropertyValue::Duration {
            months: 149,
            days: 14,
            seconds: 58390,
            nanos: 2,
        },
    );
    let id = store.create_node(&["Val"], props).unwrap();

    let node = store.get_node(id).unwrap().unwrap();
    assert_eq!(node.props.get("date"), Some(&PropertyValue::Date(5397)));
    assert_eq!(
        node.props.get("duration"),
        Some(&PropertyValue::Duration {
            months: 149,
            days: 14,
            seconds: 58390,
            nanos: 2
        })
    );
}

/// `all_nodes(None)` -- the unfiltered full-scan path (as opposed to
/// every other `all_nodes` call in this crate's test suite, which always
/// passes a label filter and so only exercises the `NODE_LABEL_INDEX`
/// path).
#[test]
fn all_nodes_with_no_label_filter_scans_every_node() {
    let store = GraphStore::open_memory().unwrap();
    store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store.create_node(&["Dog"], BTreeMap::new()).unwrap();
    store.create_node(&[], BTreeMap::new()).unwrap();

    let all = store.all_nodes(None).unwrap();
    assert_eq!(all.len(), 3);
}

/// `set_edge_prop` -- the standalone (own-transaction) convenience
/// wrapper, distinct from `set_edge_prop_in_txn` (used internally by the
/// Cypher executor within its own already-open write transaction).
#[test]
fn set_edge_prop_updates_a_relationship_directly() {
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let edge = store.create_edge("KNOWS", a, b, BTreeMap::new()).unwrap();

    let updated = store
        .set_edge_prop(edge, "since", PropertyValue::Int(2020))
        .unwrap();
    assert!(updated);

    // A nonexistent edge id reports `false`, not an error.
    let missing = store
        .set_edge_prop(
            marsdb_graph::EdgeId(999_999),
            "since",
            PropertyValue::Int(1),
        )
        .unwrap();
    assert!(!missing);
}
