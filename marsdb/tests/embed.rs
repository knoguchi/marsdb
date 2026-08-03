//! Proves the crate is embeddable independent of the CLI subprocess.

use marsdb::Database;

#[test]
fn in_memory_roundtrip() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
        .unwrap();
    let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn file_backed_persists_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");

    {
        let db = Database::open(&path).unwrap();
        db.execute("CREATE (a:Person {name: 'Alice'})").unwrap();
    }

    let db = Database::open(&path).unwrap();
    let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn two_in_memory_databases_do_not_share_data() {
    let a = Database::in_memory().unwrap();
    let b = Database::in_memory().unwrap();
    a.execute("CREATE (n:Person {name: 'Alice'})").unwrap();
    let result = b.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn execute_with_params_substitutes_dollar_placeholders() {
    use std::collections::HashMap;

    let db = Database::in_memory().unwrap();
    db.execute("CREATE (n:Person {personId: 42, name: 'Alice'})").unwrap();

    let mut params = HashMap::new();
    params.insert("personId".to_string(), marsdb::PropertyValue::Int(42));
    let result = db
        .execute_with_params("MATCH (n:Person {personId: $personId}) RETURN n.name", &params)
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    // Missing param -> clean error, not a panic.
    let err = db
        .execute_with_params("MATCH (n:Person {personId: $missing}) RETURN n.name", &HashMap::new())
        .unwrap_err();
    assert!(err.to_string().contains("missing"));
}
