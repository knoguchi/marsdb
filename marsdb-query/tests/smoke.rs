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

#[test]
fn order_by_multi_key_against_aliases_not_raw_bindings() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Charlie', age: 30})");
    run(&store, "CREATE (b:Person {name: 'Alice', age: 30})");
    run(&store, "CREATE (c:Person {name: 'Bob', age: 25})");

    // Sort keys are aliases (personAge/personName), not raw pattern vars —
    // this is the shape every IS-query ORDER BY actually uses.
    let result = run(
        &store,
        "MATCH (n:Person) RETURN n.age AS personAge, n.name AS personName ORDER BY personAge DESC, personName ASC",
    );
    let names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[1] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["Alice".to_string(), "Charlie".to_string(), "Bob".to_string()]);
}

#[test]
fn order_by_then_limit_sorts_before_truncating() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS x ORDER BY x DESC LIMIT 2");
    let values: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    // Must be the top 2 by DESC order (4, 3), not an arbitrary 2 rows
    // taken before sorting.
    assert_eq!(values, vec![4, 3]);
}

#[test]
fn order_by_with_function_call_key() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {code: '20'})");
    run(&store, "CREATE (b:Person {code: '3'})");
    // toInteger(personId) as an ORDER BY key, matching IS3's shape.
    let result = run(
        &store,
        "MATCH (n:Person) RETURN n.code AS personId ORDER BY toInteger(personId) ASC",
    );
    let values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(values, vec!["3".to_string(), "20".to_string()]);
}

#[test]
fn undirected_pattern_matches_either_direction() {
    let store = GraphStore::open_memory().unwrap();
    // a->b (created via Right direction), so from b's perspective it's an
    // incoming edge — an undirected MATCH from b must still find a.
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");

    let from_a = run(&store, "MATCH (n:Person {name: 'Alice'})-[:KNOWS]-(friend) RETURN friend.name");
    assert_eq!(from_a.rows.len(), 1);

    let from_b = run(&store, "MATCH (n:Person {name: 'Bob'})-[:KNOWS]-(friend) RETURN friend.name");
    assert_eq!(from_b.rows.len(), 1);
    match &from_b.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Alice"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn undirected_pattern_dedupes_self_loop() {
    use std::collections::BTreeMap;

    // A true self-loop needs the same node bound as both src and dst, which
    // Cypher CREATE can't express in v1 (each pattern position always
    // creates a fresh node) — construct it directly via GraphStore instead.
    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", alice, alice, BTreeMap::new()).unwrap();

    let result = run(&store, "MATCH (n:Person)-[:KNOWS]-(friend) RETURN friend");
    assert_eq!(
        result.rows.len(),
        1,
        "a self-loop edge must be returned once via undirected dedup-by-edge_id, not twice"
    );
}

#[test]
fn create_rejects_undirected_pattern_at_execute_time() {
    // The grammar allows `-[...]−` anywhere a rel_pattern appears (CREATE
    // and MATCH share the same pattern rule), so this parses fine — the
    // rejection happens in execute_create, since CREATE always needs a
    // direction to know which node is src and which is dst.
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Person)-[:KNOWS]-(b:Person)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("direct"));
}

#[test]
fn hop_node_first_label_is_actually_filtered() {
    // Regression test: the planner used to unconditionally skip the FIRST
    // label when filtering a node reached via Expand/VarExpand (only
    // correct for the pattern's start node, where NodeByLabelScan already
    // handles the first label) -- so `(a)-[:R]->(b:Post)` would match ANY
    // labeled node at the far end of the hop, not just :Post ones.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Root {name: 'r'})-[:R]->(b:Post {name: 'post'})");
    run(&store, "CREATE (a2:Root {name: 'r2'})-[:R]->(c:Comment {name: 'comment'})");

    let result = run(&store, "MATCH (a:Root)-[:R]->(b:Post) RETURN b.name");
    assert_eq!(result.rows.len(), 1, "hop node's label filter must exclude the :Comment target");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "post"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn variable_length_pattern_walks_reply_chain_to_root() {
    use std::collections::BTreeMap;

    // Mirrors IS6's shape: MATCH (m:Message {id})-[:REPLY_OF*0..]->(p:Post) ...
    // Chain c2 -[:REPLY_OF]-> c1 -[:REPLY_OF]-> p. Cypher CREATE always
    // makes fresh nodes per pattern position (no MATCH+CREATE combo in
    // v1), so build this directly via GraphStore to get real shared node
    // identity across the chain.
    let store = GraphStore::open_memory().unwrap();
    let mut props_p = BTreeMap::new();
    props_p.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let p = store.create_node(&["Post", "Message"], props_p).unwrap();
    let mut props_c1 = BTreeMap::new();
    props_c1.insert("id".to_string(), marsdb_graph::PropertyValue::Int(2));
    let c1 = store.create_node(&["Comment", "Message"], props_c1).unwrap();
    let mut props_c2 = BTreeMap::new();
    props_c2.insert("id".to_string(), marsdb_graph::PropertyValue::Int(3));
    let c2 = store.create_node(&["Comment", "Message"], props_c2).unwrap();
    store.create_edge("REPLY_OF", c1, p, BTreeMap::new()).unwrap();
    store.create_edge("REPLY_OF", c2, c1, BTreeMap::new()).unwrap();

    let result = run(&store, "MATCH (m:Message {id: 3})-[:REPLY_OF*0..]->(p:Post) RETURN p.id");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 1),
        other => panic!("unexpected value {other:?}"),
    }

    // min_hops = 0 also includes the start node itself if it happens to
    // match the target label — not exercised by IS6 (a Comment never
    // has :Post too) but worth confirming: starting FROM the post with
    // *0.. must return the post itself at hop 0.
    let from_post = run(&store, "MATCH (m:Message {id: 1})-[:REPLY_OF*0..]->(p:Post) RETURN p.id");
    assert_eq!(from_post.rows.len(), 1);
}

#[test]
fn variable_length_bounded_range_respects_max_hops() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    for i in 0..5 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        ids.push(store.create_node(&["Item"], props).unwrap());
    }
    for i in 0..4 {
        store.create_edge("NEXT", ids[i], ids[i + 1], BTreeMap::new()).unwrap();
    }

    // From idx=0, *1..2 should reach idx=1 and idx=2 only (not 3, not 4;
    // not 0 itself since min_hops=1).
    let result = run(&store, "MATCH (n:Item {idx: 0})-[:NEXT*1..2]->(m:Item) RETURN m.idx");
    let mut reached: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    reached.sort();
    assert_eq!(reached, vec![1, 2]);
}

#[test]
fn variable_length_unbounded_depth_cap_errors_not_truncates() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut prev = {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(0));
        store.create_node(&["Item"], props).unwrap()
    };
    // 40 hops, past the 30-hop safety cap.
    for i in 1..40 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        let next = store.create_node(&["Item"], props).unwrap();
        store.create_edge("NEXT", prev, next, BTreeMap::new()).unwrap();
        prev = next;
    }

    let stmt = parse("MATCH (n:Item {idx: 0})-[:NEXT*0..]->(m:Item) RETURN m.idx").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("depth cap"), "expected a depth-cap error, got: {err}");
}

#[test]
fn create_rejects_variable_length_pattern() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Item)-[:NEXT*1..3]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}
