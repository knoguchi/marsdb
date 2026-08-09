//! `QueryResult::stats` — per-statement write counters, the answer to
//! "how many did my DELETE delete".

use marsdb::Database;

fn stats(db: &Database, cypher: &str) -> marsdb::QueryStats {
    db.execute(cypher).unwrap().stats
}

#[test]
fn create_counts_nodes_and_relationships() {
    let db = Database::in_memory().unwrap();
    let s = stats(&db, "CREATE (a:P {x: 1})-[:R {y: 2}]->(b:P)");
    assert_eq!(s.nodes_created, 2);
    assert_eq!(s.relationships_created, 1);
    // Inline CREATE props are part of the records being born, not
    // SET-style mutations of existing records.
    assert_eq!(s.properties_set, 0);
    assert_eq!(s.nodes_deleted, 0);
}

#[test]
fn reads_report_all_zero() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:P)").unwrap();
    let result = db.execute("MATCH (n:P) RETURN n").unwrap();
    assert!(result.stats.is_empty());
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn bulk_delete_counts_every_edge() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (a:U), (b:M)").unwrap();
    for _ in 0..5 {
        db.execute("MATCH (a:U), (b:M) CREATE (a)-[:RATED]->(b)")
            .unwrap();
    }
    let s = stats(&db, "MATCH ()-[r:RATED]->() DELETE r");
    assert_eq!(s.relationships_deleted, 5);
    assert_eq!(s.nodes_deleted, 0);
}

#[test]
fn detach_delete_counts_node_and_incident_edges() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (hub:H)").unwrap();
    for _ in 0..3 {
        db.execute("MATCH (hub:H) CREATE (hub)-[:R]->(:Leaf)")
            .unwrap();
    }
    let s = stats(&db, "MATCH (hub:H) DETACH DELETE hub");
    assert_eq!(s.nodes_deleted, 1);
    assert_eq!(s.relationships_deleted, 3);
}

#[test]
fn set_and_remove_count_properties_and_labels() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:P {a: 1})").unwrap();
    assert_eq!(stats(&db, "MATCH (n:P) SET n.b = 2").properties_set, 1);
    // Removing (or nulling) counts as a property set, per the usual
    // summary-counter convention.
    assert_eq!(stats(&db, "MATCH (n:P) SET n.a = null").properties_set, 1);
    assert_eq!(stats(&db, "MATCH (n:P) REMOVE n.b").properties_set, 1);
    assert_eq!(stats(&db, "MATCH (n:P) SET n:Extra").labels_added, 1);
    assert_eq!(stats(&db, "MATCH (n:P) REMOVE n:Extra").labels_removed, 1);
}

#[test]
fn set_counts_once_per_matched_row() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:P), (:P), (:P)").unwrap();
    assert_eq!(
        stats(&db, "MATCH (n:P) SET n.seen = true").properties_set,
        3
    );
}

#[test]
fn merge_counts_only_what_it_creates() {
    let db = Database::in_memory().unwrap();
    let s = stats(&db, "MERGE (n:P {k: 1})");
    assert_eq!(s.nodes_created, 1);
    // Second time: matched, nothing created.
    let s = stats(&db, "MERGE (n:P {k: 1})");
    assert!(s.is_empty());
    // ON CREATE / ON MATCH SET flow into properties_set.
    let s = stats(&db, "MERGE (n:P {k: 2}) ON CREATE SET n.fresh = true");
    assert_eq!(s.nodes_created, 1);
    assert_eq!(s.properties_set, 1);
    let s = stats(&db, "MERGE (n:P {k: 2}) ON MATCH SET n.seen = true");
    assert_eq!(s.nodes_created, 0);
    assert_eq!(s.properties_set, 1);
}

#[test]
fn stats_flow_through_session_transactions() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    let s = db.execute("CREATE (:P), (:P)").unwrap().stats;
    assert_eq!(s.nodes_created, 2);
    db.execute("COMMIT").unwrap();
}
