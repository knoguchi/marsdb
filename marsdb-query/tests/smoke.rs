use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

fn run(store: &GraphStore, cypher: &str) -> marsdb_query::QueryResult {
    let stmt = parse(cypher).unwrap_or_else(|e| panic!("parse failed for {cypher:?}: {e}"));
    Executor::new(store)
        .execute(&stmt)
        .unwrap_or_else(|e| panic!("execute failed for {cypher:?}: {e}"))
}

#[test]
fn create_match_return() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice', age: 30})-[:KNOWS]->(b:Person {name: 'Bob', age: 25})",
    );

    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(result.columns, vec!["n.name"]);
    let mut names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    names.sort();
    assert_eq!(names, vec!["Alice", "Bob"]);
}

#[test]
fn traversal_with_label_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:BLOCKS]->(c:Person {name: 'Carol'})");

    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person) RETURN b.name",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Bob"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn where_clause_filters() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice', age: 30})");
    run(&store, "CREATE (b:Person {name: 'Bob', age: 17})");

    let result = run(&store, "MATCH (n:Person) WHERE n.age >= 18 RETURN n.name");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Alice"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn limit_clause() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx LIMIT 2");
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn detach_delete() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");
    run(&store, "MATCH (n:Person {name: 'Alice'}) DETACH DELETE n");
    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn set_updates_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice', age: 30})");
    run(&store, "MATCH (n:Person {name: 'Alice'}) SET n.age = 31");
    let result = run(&store, "MATCH (n:Person {name: 'Alice'}) RETURN n.age");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 31),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn multi_label_create_and_match() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (p:Post:Message {id: 1})");
    run(&store, "CREATE (c:Comment:Message {id: 2})");
    run(&store, "CREATE (p2:Post {id: 3})"); // single-label, must NOT match :Message

    let as_message = run(&store, "MATCH (m:Message) RETURN m.id");
    let mut ids: Vec<i64> = as_message
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);

    let as_post = run(&store, "MATCH (p:Post) RETURN p.id");
    let mut post_ids: Vec<i64> = as_post
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    post_ids.sort();
    assert_eq!(post_ids, vec![1, 3]);

    // multi-label pattern match: AND semantics
    let both = run(&store, "MATCH (n:Post:Message) RETURN n.id");
    assert_eq!(both.rows.len(), 1);
    match &both.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 1),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn coalesce_returns_first_non_null() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Post {content: 'hello', imageFile: 'ignored.png'})");
    run(&store, "CREATE (b:Post {imageFile: 'pic.png'})"); // no content prop

    let result = run(&store, "MATCH (n:Post) RETURN coalesce(n.content, n.imageFile) AS x");
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["hello".to_string(), "pic.png".to_string()]);
}

#[test]
fn to_integer_parses_string_and_passes_through_int() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {code: '42'})");
    let result = run(&store, "MATCH (n:Person) RETURN toInteger(n.code) AS x");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 42),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn case_when_then_else() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {age: 30})");
    run(&store, "CREATE (b:Person {age: 17})");
    let result = run(&store, "MATCH (n:Person) RETURN CASE n.age WHEN 30 THEN 'thirty' ELSE 'other' END AS x");
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["other".to_string(), "thirty".to_string()]);
}

#[test]
fn case_null_equals_null_is_true_not_standard_three_valued_logic() {
    // Documents the deliberate convention CASE relies on for IS7: a missing
    // property compared against `null` in a WHEN arm matches.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})"); // no `age` prop
    let result = run(&store, "MATCH (n:Person) RETURN CASE n.age WHEN null THEN 'yes' ELSE 'no' END AS x");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "yes"),
        other => panic!("unexpected value {other:?}"),
    }
}
