//! Built-in schema introspection (`CALL db.labels()` and friends) —
//! full-stack coverage through `Database::execute`, including YIELD
//! projection, the liveness rules (deleted instances drop out), and the
//! shared node-label/rel-type intern namespace being split correctly.

use marsdb::{Database, Literal, PropertyValue, Value};

fn seeded() -> Database {
    let db = Database::in_memory().unwrap();
    db.execute_batch(
        "CREATE INDEX ON :Person(name) UNIQUE; \
         CREATE INDEX ON :Movie(title); \
         CREATE (:Person {name: 'Alice', age: 40}); \
         CREATE (:Person {name: 'Bob'}); \
         CREATE (:Movie {title: 'Heat'}); \
         MATCH (a:Person {name: 'Alice'}), (b:Movie) CREATE (a)-[:WATCHED {stars: 5}]->(b)",
    )
    .unwrap();
    db
}

fn strings(result: &marsdb::QueryResult, col: usize) -> Vec<String> {
    result
        .rows
        .iter()
        .map(|row| match &row[col] {
            // Direct CALL output arrives as Literal; a value that went
            // through a YIELD binding and back out of RETURN arrives as
            // Property -- both are the same string.
            Value::Literal(Literal::String(s)) => s.clone(),
            Value::Property(PropertyValue::String(s)) => s.clone(),
            other => panic!("expected string, got {other:?}"),
        })
        .collect()
}

fn ints(result: &marsdb::QueryResult, col: usize) -> Vec<i64> {
    result
        .rows
        .iter()
        .map(|row| match &row[col] {
            Value::Literal(Literal::Int(n)) => *n,
            other => panic!("expected int, got {other:?}"),
        })
        .collect()
}

#[test]
fn labels_with_counts_sorted_by_name() {
    let result = seeded().execute("CALL db.labels()").unwrap();
    assert_eq!(result.columns, vec!["label", "count"]);
    assert_eq!(strings(&result, 0), vec!["Movie", "Person"]);
    assert_eq!(ints(&result, 1), vec![1, 2]);
}

#[test]
fn relationship_types_exclude_node_labels_and_vice_versa() {
    // WATCHED shares the intern table with Person/Movie -- each list
    // must contain only its own kind.
    let db = seeded();
    let types = db.execute("CALL db.relationshipTypes()").unwrap();
    assert_eq!(types.columns, vec!["relationshipType", "count"]);
    assert_eq!(strings(&types, 0), vec!["WATCHED"]);
    assert_eq!(ints(&types, 1), vec![1]);
    let labels = db.execute("CALL db.labels()").unwrap();
    assert!(!strings(&labels, 0).contains(&"WATCHED".to_string()));
}

#[test]
fn deleted_instances_drop_out() {
    let db = seeded();
    db.execute("MATCH ()-[r:WATCHED]->() DELETE r").unwrap();
    assert!(db
        .execute("CALL db.relationshipTypes()")
        .unwrap()
        .rows
        .is_empty());
    db.execute("MATCH (m:Movie) DELETE m").unwrap();
    assert_eq!(
        strings(&db.execute("CALL db.labels()").unwrap(), 0),
        vec!["Person"]
    );
}

#[test]
fn property_keys_span_nodes_and_edges() {
    let result = seeded().execute("CALL db.propertyKeys()").unwrap();
    assert_eq!(result.columns, vec!["propertyKey"]);
    assert_eq!(strings(&result, 0), vec!["age", "name", "stars", "title"]);
}

#[test]
fn indexes_report_label_property_unique() {
    let result = seeded().execute("CALL db.indexes()").unwrap();
    assert_eq!(result.columns, vec!["label", "property", "unique"]);
    let rows: Vec<(String, String, bool)> = result
        .rows
        .iter()
        .map(|row| {
            (
                strings(&result, 0)[0].clone(),
                match &row[1] {
                    Value::Literal(Literal::String(s)) => s.clone(),
                    other => panic!("{other:?}"),
                },
                matches!(&row[2], Value::Literal(Literal::Bool(true))),
            )
        })
        .collect();
    assert_eq!(result.rows.len(), 2);
    assert!(rows.iter().any(|(_, p, u)| p == "name" && *u));
    assert!(rows.iter().any(|(_, p, u)| p == "title" && !*u));
}

#[test]
fn yield_projection_and_where_work() {
    let db = seeded();
    let result = db
        .execute("CALL db.labels() YIELD label WHERE label = 'Movie' RETURN label")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(strings(&result, 0), vec!["Movie"]);
}

#[test]
fn wrong_arity_errors() {
    assert!(seeded().execute("CALL db.labels('x')").is_err());
}

#[test]
fn empty_database_yields_empty_lists() {
    let db = Database::in_memory().unwrap();
    assert!(db.execute("CALL db.labels()").unwrap().rows.is_empty());
    assert!(db
        .execute("CALL db.relationshipTypes()")
        .unwrap()
        .rows
        .is_empty());
    assert!(db
        .execute("CALL db.propertyKeys()")
        .unwrap()
        .rows
        .is_empty());
    assert!(db.execute("CALL db.indexes()").unwrap().rows.is_empty());
}
