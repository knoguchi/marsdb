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
fn delete_node_detach() {
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store
        .create_edge("KNOWS", a, b, BTreeMap::new())
        .unwrap();

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
    let post = store.create_node(&["Post", "Message"], BTreeMap::new()).unwrap();
    let comment = store.create_node(&["Comment", "Message"], BTreeMap::new()).unwrap();
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
