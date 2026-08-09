//! REL_TYPE_COUNTS maintenance: the planner-statistic table must track
//! every way an edge is born or dies, and rebuild itself once for files
//! written before the table existed.

use std::collections::BTreeMap;

use marsdb_graph::{GraphStore, Txn};

fn counts(store: &GraphStore, rel_type: &str) -> u64 {
    let read = store.begin_read().unwrap();
    GraphStore::rel_type_count_in_txn(Txn::Read(&read), rel_type).unwrap()
}

#[test]
fn create_and_delete_maintain_per_type_counts() {
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["N"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["N"], BTreeMap::new()).unwrap();

    let knows = store.create_edge("KNOWS", a, b, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", b, a, BTreeMap::new()).unwrap();
    store.create_edge("LIKES", a, b, BTreeMap::new()).unwrap();
    assert_eq!(counts(&store, "KNOWS"), 2);
    assert_eq!(counts(&store, "LIKES"), 1);
    assert_eq!(counts(&store, "NEVER_USED"), 0);

    store.delete_edge(knows).unwrap();
    assert_eq!(counts(&store, "KNOWS"), 1);
    // Deleting an already-gone edge is a no-op, not a double decrement.
    store.delete_edge(knows).unwrap();
    assert_eq!(counts(&store, "KNOWS"), 1);
}

#[test]
fn detach_deleting_a_node_decrements_its_incident_edges() {
    let store = GraphStore::open_memory().unwrap();
    let hub = store.create_node(&["N"], BTreeMap::new()).unwrap();
    for _ in 0..3 {
        let other = store.create_node(&["N"], BTreeMap::new()).unwrap();
        store
            .create_edge("KNOWS", hub, other, BTreeMap::new())
            .unwrap();
        store
            .create_edge("LIKES", other, hub, BTreeMap::new())
            .unwrap();
    }
    assert_eq!(counts(&store, "KNOWS"), 3);
    assert_eq!(counts(&store, "LIKES"), 3);

    store.delete_node(hub, true).unwrap();
    assert_eq!(counts(&store, "KNOWS"), 0);
    assert_eq!(counts(&store, "LIKES"), 0);
}

#[test]
fn opening_a_file_without_the_counts_table_backfills_it_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    {
        let store = GraphStore::open_file(&path).unwrap();
        let a = store.create_node(&["N"], BTreeMap::new()).unwrap();
        let b = store.create_node(&["N"], BTreeMap::new()).unwrap();
        store.create_edge("KNOWS", a, b, BTreeMap::new()).unwrap();
        store.create_edge("KNOWS", b, a, BTreeMap::new()).unwrap();
        store.create_edge("LIKES", a, b, BTreeMap::new()).unwrap();
    }
    // Simulate a file written by a build that predates the table: strip
    // it with raw redb, exactly the state an old build leaves behind.
    {
        let db = redb::Database::create(&path).unwrap();
        let write = db.begin_write().unwrap();
        assert!(write
            .delete_table(redb::TableDefinition::<u32, u64>::new("rel_type_counts"))
            .unwrap());
        write.commit().unwrap();
    }
    let store = GraphStore::open_file(&path).unwrap();
    assert_eq!(counts(&store, "KNOWS"), 2);
    assert_eq!(counts(&store, "LIKES"), 1);
}
