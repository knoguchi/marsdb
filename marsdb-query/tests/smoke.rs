use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, PathElem, Value};

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

/// `MATCH (n:Label) RETURN ... LIMIT k` (no WHERE, no hops, no ORDER BY) is
/// the one shape `execute_match` pushes the LIMIT all the way into the
/// storage scan for -- covers both edges that shape needs to get right:
/// `LIMIT 0` (valid Cypher, must return nothing, not error or return
/// everything) and a `LIMIT` past the available row count (must return
/// what's actually there, not pad or panic).
#[test]
fn limit_clause_pushed_into_scan_edge_cases() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    assert_eq!(run(&store, "MATCH (n:Item) RETURN n.idx LIMIT 0").rows.len(), 0);
    assert_eq!(run(&store, "MATCH (n:Item) RETURN n.idx LIMIT 100").rows.len(), 3);
    assert_eq!(run(&store, "MATCH (n) RETURN n LIMIT 2").rows.len(), 2);
}

#[test]
fn return_distinct_dedups_whole_row() {
    let store = GraphStore::open_memory().unwrap();
    for city in ["A", "B", "A", "A", "B"] {
        run(&store, &format!("CREATE (n:City {{name: '{city}'}})"));
    }
    let result = run(&store, "MATCH (n:City) RETURN DISTINCT n.name AS c ORDER BY c");
    let names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["A", "B"]);
}

/// `RETURN DISTINCT ... LIMIT k` (no ORDER BY) must apply LIMIT *after*
/// dedup, not before -- both the LIMIT push-down-into-scan shortcut and
/// the pre-tail truncate (`execute_match`'s `distinct_return` exclusion)
/// need to skip this shape, or fewer than `k` distinct rows could come
/// back even when more exist past what got scanned/truncated first.
#[test]
fn return_distinct_then_limit_counts_distinct_rows_not_raw_rows() {
    let store = GraphStore::open_memory().unwrap();
    // Two duplicate 'A's scanned first, then 'B' and 'C' -- a naive
    // truncate-before-dedup would only ever see the two 'A's and return
    // just one distinct row for LIMIT 2, not two.
    for city in ["A", "A", "B", "C"] {
        run(&store, &format!("CREATE (n:City {{name: '{city}'}})"));
    }
    let result = run(&store, "MATCH (n:City) RETURN DISTINCT n.name LIMIT 2");
    assert_eq!(result.rows.len(), 2);
}

/// A bare `RETURN <expr>` with no `MATCH`/`UNWIND`/`MERGE` at all is real
/// Cypher (`match_stmt`'s `clause*`, not `clause+`) -- needs no graph
/// access, just the one synthetic empty row `execute_match` already seeds
/// `current_rows` with by default.
#[test]
fn bare_return_needs_no_match_clause() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN 1 + 2 * 3 AS x");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 7),
        other => panic!("unexpected value {other:?}"),
    }
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

    // Sort keys are aliases (personAge/person_name), not raw pattern vars —
    // this is the shape every IS-query ORDER BY actually uses.
    let result = run(
        &store,
        "MATCH (n:Person) RETURN n.age AS personAge, n.name AS person_name ORDER BY personAge DESC, person_name ASC",
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

/// Exercises `top_k_by`'s aggregating path (`apply_order_by`) specifically
/// -- the non-aggregating case above goes through a different function
/// (`apply_order_by_with_scope`).
#[test]
fn order_by_then_limit_with_aggregation() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS x, count(*) AS c ORDER BY x DESC LIMIT 2");
    let values: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(values, vec![4, 3]);
}

/// `LIMIT 0` combined with `ORDER BY` is valid Cypher and must return
/// nothing, not error or return everything -- an edge case `top_k_by`'s
/// partial-select path needs to special-case explicitly (`select_nth_
/// unstable_by` panics on an out-of-bounds index, which `k - 1` would be
/// for `k == 0`).
#[test]
fn order_by_then_limit_zero_returns_nothing() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS x ORDER BY x LIMIT 0");
    assert_eq!(result.rows.len(), 0);
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

fn as_int(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        other => panic!("expected an int, got {other:?}"),
    }
}

fn as_float(v: &Value) -> f64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => *f,
        other => panic!("expected a float, got {other:?}"),
    }
}

#[test]
fn arithmetic_precedence_and_grouping() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {price: 10, qty: 3})");
    // * binds tighter than + without parens; parens override it.
    let a = run(&store, "MATCH (n:Item) RETURN 2 + 3 * n.qty AS x");
    let b = run(&store, "MATCH (n:Item) RETURN (2 + 3) * n.qty AS x");
    assert_eq!(as_int(&a.rows[0][0]), 11);
    assert_eq!(as_int(&b.rows[0][0]), 15);
    // Int/Int division truncates; a Float operand promotes the result.
    let div_int = run(&store, "MATCH (n:Item) RETURN n.price / n.qty AS x");
    assert_eq!(as_int(&div_int.rows[0][0]), 3);
    let div_float = run(&store, "MATCH (n:Item) RETURN n.price / 2.0 AS x");
    assert_eq!(as_float(&div_float.rows[0][0]), 5.0);
}

/// A nested aggregate inside an arithmetic expression (`1 + count(x)`) is
/// a real, deliberate rejection, not a silent wrong answer -- `Arith`
/// existing at all made this reachable for the first time (previously
/// `count(x)` could only ever be a return item's entire expression, so
/// there was nothing to wrap it in), and it needs `has_aggregate` to
/// route the query to the grouping path at all before
/// `validate_return_items` gets a chance to reject it; a narrower check
/// (only detecting an aggregate as the item's *entire* top-level
/// expression) silently returned the wrong row count instead of erroring
/// -- see `has_aggregate`'s own doc comment for the real scenario this
/// caught.
#[test]
fn nested_aggregate_inside_arithmetic_is_rejected_not_silently_wrong() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = marsdb_query::parse("MATCH (n) RETURN 1 + count(n)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("entire expression"), "unexpected error: {err}");
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

#[test]
fn with_chaining_mirrors_is2_shape() {
    use std::collections::BTreeMap;

    // Mirrors IS2: MATCH ... WITH ... ORDER BY ... LIMIT ... MATCH
    // (comma-pattern) ... RETURN ... ORDER BY.
    //
    // m1 is its own Post, authored by Alice (REPLY_OF*0.. reaches itself).
    // m2 is a Comment authored by Alice, replying to p1 (a Post authored by
    // Bob) -- REPLY_OF*0.. must walk one hop to reach it.
    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    let mut bob_props = BTreeMap::new();
    bob_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Bob".into()));
    let bob = store.create_node(&["Person"], bob_props).unwrap();

    let mut m1_props = BTreeMap::new();
    m1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let m1 = store.create_node(&["Post"], m1_props).unwrap();
    store.create_edge("HAS_CREATOR", m1, alice, BTreeMap::new()).unwrap();

    let mut m2_props = BTreeMap::new();
    m2_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(2));
    let m2 = store.create_node(&["Comment"], m2_props).unwrap();
    store.create_edge("HAS_CREATOR", m2, alice, BTreeMap::new()).unwrap();

    let mut p1_props = BTreeMap::new();
    p1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(3));
    let p1 = store.create_node(&["Post"], p1_props).unwrap();
    store.create_edge("HAS_CREATOR", p1, bob, BTreeMap::new()).unwrap();
    store.create_edge("REPLY_OF", m2, p1, BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'})<-[:HAS_CREATOR]-(message) \
         WITH message, message.id AS message_id \
         ORDER BY message_id ASC \
         LIMIT 10 \
         MATCH (message)-[:REPLY_OF*0..]->(post:Post), (post)-[:HAS_CREATOR]->(person) \
         RETURN message_id, post.id AS post_id, person.name AS person_name \
         ORDER BY message_id ASC",
    );

    assert_eq!(result.columns, vec!["message_id", "post_id", "person_name"]);
    assert_eq!(result.rows.len(), 2, "both of Alice's messages must resolve to a post+author");

    let extract = |row: &Vec<Value>| -> (i64, i64, String) {
        let message_id = match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected message_id {other:?}"),
        };
        let post_id = match &row[1] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected post_id {other:?}"),
        };
        let person_name = match &row[2] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected person_name {other:?}"),
        };
        (message_id, post_id, person_name)
    };

    assert_eq!(extract(&result.rows[0]), (1, 1, "Alice".to_string()));
    assert_eq!(extract(&result.rows[1]), (2, 3, "Bob".to_string()));

    let _ = (m1, m2, p1); // silence unused warnings if any
}

#[test]
fn with_boundary_limit_restricts_what_flows_into_next_match() {
    use std::collections::BTreeMap;

    // WITH's own LIMIT must reduce which rows continue to the next MATCH,
    // not just presentation -- confirms apply_order_by_bindings + truncate
    // run before the second MATCH, not after.
    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    for i in 0..5 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        let n = store.create_node(&["Item"], props).unwrap();
        ids.push(n);
    }
    for &n in &ids {
        store.create_edge("SELF", n, n, BTreeMap::new()).unwrap();
    }

    let result = run(
        &store,
        "MATCH (n:Item) \
         WITH n, n.idx AS idx ORDER BY idx DESC LIMIT 2 \
         MATCH (n)-[:SELF]->(m) \
         RETURN idx",
    );
    let mut values: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec![3, 4], "only the top-2-by-idx rows from WITH should reach the second MATCH");
}

#[test]
fn multiple_match_without_with_is_rejected() {
    let err = parse("MATCH (a:Item) MATCH (b:Item) RETURN a").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("with"));
}

#[test]
fn two_with_boundaries_is_rejected() {
    // Grammar-valid (two match_parts, each with its own WITH) but rejected
    // at the AST level -- v1 only supports chaining past one WITH boundary.
    let err = parse("MATCH (a:Item) WITH a MATCH (b:Item) WITH b RETURN a").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("with"));
}

#[test]
fn optional_match_with_var_eq_mirrors_is7_shape() {
    use std::collections::BTreeMap;

    // Mirrors IS7: MATCH (m)<-[:REPLY_OF]-(c)-[:HAS_CREATOR]->(p)
    //              OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a)-[r:KNOWS]-(p)
    //              RETURN ... CASE r WHEN null THEN false ELSE true END
    //
    // p is bound by the first MATCH (comment author) then reappears as the
    // endpoint of the OPTIONAL pattern -- must mean "KNOWS THIS p", not
    // "KNOWS anyone" (Expr::VarEq). c1's author knows the original
    // message's author; c2's author doesn't -- so the two rows must get
    // different CASE results, not both true/both false.
    let store = GraphStore::open_memory().unwrap();

    let mut m_props = BTreeMap::new();
    m_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let m = store.create_node(&["Post", "Message"], m_props).unwrap();
    let original_author = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store.create_edge("HAS_CREATOR", m, original_author, BTreeMap::new()).unwrap();

    let mut p1_props = BTreeMap::new();
    p1_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("KnowsAuthor".into()));
    let p1 = store.create_node(&["Person"], p1_props).unwrap();
    let mut c1_props = BTreeMap::new();
    c1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(10));
    let c1 = store.create_node(&["Comment", "Message"], c1_props).unwrap();
    store.create_edge("REPLY_OF", c1, m, BTreeMap::new()).unwrap();
    store.create_edge("HAS_CREATOR", c1, p1, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", p1, original_author, BTreeMap::new()).unwrap();

    let mut p2_props = BTreeMap::new();
    p2_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("StrangerAuthor".into()));
    let p2 = store.create_node(&["Person"], p2_props).unwrap();
    let mut c2_props = BTreeMap::new();
    c2_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(20));
    let c2 = store.create_node(&["Comment", "Message"], c2_props).unwrap();
    store.create_edge("REPLY_OF", c2, m, BTreeMap::new()).unwrap();
    store.create_edge("HAS_CREATOR", c2, p2, BTreeMap::new()).unwrap();
    // No KNOWS edge between p2 and original_author.

    let result = run(
        &store,
        "MATCH (m:Message {id: 1})<-[:REPLY_OF]-(c:Comment)-[:HAS_CREATOR]->(p:Person) \
         OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a:Person)-[r:KNOWS]-(p) \
         RETURN c.id AS commentId, p.name AS replyAuthorName, \
                CASE r WHEN null THEN false ELSE true END AS knowsFlag \
         ORDER BY commentId ASC",
    );

    assert_eq!(result.rows.len(), 2);

    let extract = |row: &Vec<Value>| -> (i64, String, bool) {
        let comment_id = match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected commentId {other:?}"),
        };
        let name = match &row[1] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected replyAuthorName {other:?}"),
        };
        let knows = match &row[2] {
            Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
            other => panic!("unexpected knowsFlag {other:?}"),
        };
        (comment_id, name, knows)
    };

    assert_eq!(extract(&result.rows[0]), (10, "KnowsAuthor".to_string(), true));
    assert_eq!(extract(&result.rows[1]), (20, "StrangerAuthor".to_string(), false));

    let _ = (m, c1, c2, p1, p2, original_author);
}

#[test]
fn optional_match_null_pads_when_nothing_matches() {
    use std::collections::BTreeMap;

    // No KNOWS edge exists at all -- the OPTIONAL MATCH must still return
    // one row per outer row (not zero), with `a`/`r` null-padded.
    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (p:Person) \
         OPTIONAL MATCH (p)-[r:KNOWS]-(friend) \
         RETURN CASE r WHEN null THEN false ELSE true END AS knowsFlag",
    );
    assert_eq!(result.rows.len(), 1, "the outer MATCH row must survive even with zero optional matches");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Bool(b)) => assert!(!b),
        other => panic!("unexpected value {other:?}"),
    }
    let _ = alice;
}

#[test]
fn optional_match_without_with_shares_scope() {
    // MATCH ... OPTIONAL MATCH ... (no WITH between them) must be valid --
    // unlike two plain MATCHes, which require a WITH separator.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item {name: 'a'})");
    let result = run(&store, "MATCH (n:Item) OPTIONAL MATCH (n)-[:X]->(m) RETURN n.name");
    assert_eq!(result.rows.len(), 1);
}

fn int_value(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        other => panic!("expected an int, got {other:?}"),
    }
}

fn str_value(v: &Value) -> String {
    match v {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
        other => panic!("expected a string, got {other:?}"),
    }
}

#[test]
fn count_star_over_all_rows() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN count(*) AS c");
    assert_eq!(result.columns, vec!["c"]);
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 5);
}

#[test]
fn count_excludes_null_but_count_star_does_not() {
    use std::collections::BTreeMap;

    // p1/p2 each KNOWS one friend; p3 knows nobody -- its OPTIONAL MATCH
    // row null-pads `f`. count(f) must skip that row; count(*) must not.
    let store = GraphStore::open_memory().unwrap();
    let p1 = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let p2 = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let p3 = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let f1 = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let f2 = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", p1, f1, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", p2, f2, BTreeMap::new()).unwrap();
    let _ = p3;

    let result = run(
        &store,
        "MATCH (p:Person) OPTIONAL MATCH (p)-[:KNOWS]->(f:Person) RETURN count(f) AS cf, count(*) AS cs",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 2, "count(f) must exclude the null-padded row");
    assert_eq!(int_value(&result.rows[0][1]), 5, "count(*) counts every row, including null-padded ones");
}

#[test]
fn group_by_implicit_via_with() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    let mut bob_props = BTreeMap::new();
    bob_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Bob".into()));
    let bob = store.create_node(&["Person"], bob_props).unwrap();
    for _ in 0..2 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    }
    let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store.create_edge("OWNS", bob, item, BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) WITH p.name AS name, count(i) AS c RETURN name, c ORDER BY name",
    );
    assert_eq!(result.rows.len(), 2);
    assert_eq!((str_value(&result.rows[0][0]), int_value(&result.rows[0][1])), ("Alice".to_string(), 2));
    assert_eq!((str_value(&result.rows[1][0]), int_value(&result.rows[1][1])), ("Bob".to_string(), 1));
}

#[test]
fn group_by_implicit_via_return_no_with() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    for _ in 0..3 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    }

    let result = run(&store, "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN p.name AS name, count(i) AS c");
    assert_eq!(result.rows.len(), 1);
    assert_eq!((str_value(&result.rows[0][0]), int_value(&result.rows[0][1])), ("Alice".to_string(), 3));
}

#[test]
fn count_distinct_dedupes() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    for cat in ["A", "A", "B"] {
        let mut props = BTreeMap::new();
        props.insert("category".to_string(), marsdb_graph::PropertyValue::String(cat.into()));
        let item = store.create_node(&["Item"], props).unwrap();
        store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    }

    let result = run(&store, "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN count(DISTINCT i.category) AS c");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 2);
}

#[test]
fn collect_distinct_dedupes_nodes() {
    use std::collections::BTreeMap;

    // Two separate OWNS edges to the *same* item -- MATCH produces 2 rows
    // for it, but collect(DISTINCT i) must dedupe by node identity down to
    // one entry, not two.
    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();

    let result = run(&store, "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN collect(DISTINCT i) AS items");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::List(items) => assert_eq!(items.len(), 1, "the same node reached via 2 edges must collect once"),
        other => panic!("expected a list, got {other:?}"),
    }
}

#[test]
fn aggregate_over_empty_result_global() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "MATCH (n:NoSuchLabel) RETURN count(n) AS c, sum(n.x) AS s, avg(n.x) AS a, min(n.x) AS mn, max(n.x) AS mx, \
         collect(n.x) AS coll",
    );
    assert_eq!(result.rows.len(), 1, "a global aggregate over zero rows must still emit one row");
    let row = &result.rows[0];
    assert_eq!(int_value(&row[0]), 0);
    assert_eq!(int_value(&row[1]), 0);
    assert!(matches!(row[2], Value::Null));
    assert!(matches!(row[3], Value::Null));
    assert!(matches!(row[4], Value::Null));
    match &row[5] {
        Value::List(items) => assert!(items.is_empty()),
        other => panic!("expected an empty list, got {other:?}"),
    }
}

#[test]
fn aggregate_with_grouping_key_empty_result() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "MATCH (n:NoSuchLabel) RETURN n.type AS t, count(n) AS c");
    assert_eq!(result.rows.len(), 0, "a grouping key present means zero groups over zero rows, not one");
}

#[test]
fn collect_produces_list() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN collect(n.idx) AS idxs");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::List(items) => {
            let mut vals: Vec<i64> = items.iter().map(int_value).collect();
            vals.sort();
            assert_eq!(vals, vec![0, 1, 2]);
        }
        other => panic!("expected a list, got {other:?}"),
    }
}

#[test]
fn grouped_bare_var_stays_traversable_after_with() {
    use std::collections::BTreeMap;

    // The grouped `p` (a bare-var grouping key) must keep its graph
    // identity through the WITH boundary so the second MATCH can keep
    // traversing from it, not collapse to a value-only binding.
    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    for _ in 0..2 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    }
    let mut co_props = BTreeMap::new();
    co_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Acme".into()));
    let acme = store.create_node(&["Company"], co_props).unwrap();
    store.create_edge("WORKS_AT", alice, acme, BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(f:Item) \
         WITH p, count(f) AS c \
         MATCH (p)-[:WORKS_AT]->(co:Company) \
         RETURN p.name AS name, c, co.name AS company",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(int_value(&result.rows[0][1]), 2);
    assert_eq!(str_value(&result.rows[0][2]), "Acme");
}

#[test]
fn sum_avg_int_float_promotion() {
    let store = GraphStore::open_memory().unwrap();
    for v in [1, 2, 3] {
        run(&store, &format!("CREATE (n:Item {{val: {v}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN sum(n.val) AS s, avg(n.val) AS a");
    assert_eq!(int_value(&result.rows[0][0]), 6, "sum of all-int inputs must stay Int");
    match &result.rows[0][1] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => assert!((f - 2.0).abs() < 1e-9),
        other => panic!("avg must always be a float, got {other:?}"),
    }
}

#[test]
fn sum_avg_promotes_to_float_when_any_input_is_float() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {val: 1})");
    run(&store, "CREATE (n:Item {val: 2})");
    run(&store, "CREATE (n:Item {val: 1.5})");
    let result = run(&store, "MATCH (n:Item) RETURN sum(n.val) AS s, avg(n.val) AS a");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => assert!((f - 4.5).abs() < 1e-9),
        other => panic!("expected a float sum, got {other:?}"),
    }
    match &result.rows[0][1] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => assert!((f - 1.5).abs() < 1e-9),
        other => panic!("expected a float avg, got {other:?}"),
    }
}

#[test]
fn min_max_on_non_orderable_errors() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN min(n) AS m").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("comparable"), "expected a comparability error, got: {err}");
}

#[test]
fn nested_aggregate_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN count(sum(n.idx)) AS c").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregate"), "expected an aggregate-nesting error, got: {err}");
}

#[test]
fn aggregate_not_top_level_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN CASE n.idx WHEN 1 THEN count(n) ELSE 0 END AS x").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregate"), "expected a top-level-aggregate error, got: {err}");
}

#[test]
fn var_expand_depth_cap_error_survives_aggregation() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut prev = {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(0));
        store.create_node(&["Item"], props).unwrap()
    };
    for i in 1..40 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        let next = store.create_node(&["Item"], props).unwrap();
        store.create_edge("NEXT", prev, next, BTreeMap::new()).unwrap();
        prev = next;
    }

    let stmt = parse("MATCH (n:Item {idx: 0})-[:NEXT*0..]->(m:Item) RETURN count(m) AS c").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("depth cap"), "expected a depth-cap error, got: {err}");
}

#[test]
fn with_where_filters_on_aggregate_result() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Alice".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    let mut bob_props = BTreeMap::new();
    bob_props.insert("name".to_string(), marsdb_graph::PropertyValue::String("Bob".into()));
    let bob = store.create_node(&["Person"], bob_props).unwrap();
    for _ in 0..3 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store.create_edge("OWNS", alice, item, BTreeMap::new()).unwrap();
    }
    let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store.create_edge("OWNS", bob, item, BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) WITH p, count(i) AS c WHERE c > 1 RETURN p.name AS name, c",
    );
    assert_eq!(result.rows.len(), 1, "only Alice's group (count 3) should survive c > 1");
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(int_value(&result.rows[0][1]), 3);
}

#[test]
fn with_where_filters_without_aggregation() {
    let store = GraphStore::open_memory().unwrap();
    for i in [5, 15, 25] {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 RETURN y ORDER BY y");
    let vals: Vec<i64> = result.rows.iter().map(|r| int_value(&r[0])).collect();
    assert_eq!(vals, vec![15, 25]);
}

#[test]
fn with_where_and_or_not() {
    let store = GraphStore::open_memory().unwrap();
    for i in [5, 15, 25, 35] {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 AND NOT y > 30 RETURN y ORDER BY y");
    let vals: Vec<i64> = result.rows.iter().map(|r| int_value(&r[0])).collect();
    assert_eq!(vals, vec![15, 25]);
}

#[test]
fn ldbc_ic_shaped_grouping_having_orderby_limit_collect_checkpoint() {
    use std::collections::BTreeMap;

    // Not literal IC1 text/fixtures (out of scope, same as the IS1-7 plan's
    // deferral of IC fixtures) -- a hand-crafted shape combining grouping,
    // WITH...WHERE, ORDER BY, LIMIT, and collect() together in one query,
    // since no single mechanic test above exercises that combination.
    let store = GraphStore::open_memory().unwrap();
    let names = ["Alice", "Bob", "Carol", "Dave"];
    let mut people = Vec::new();
    for name in names {
        let mut props = BTreeMap::new();
        props.insert("name".to_string(), marsdb_graph::PropertyValue::String(name.into()));
        people.push(store.create_node(&["Person"], props).unwrap());
    }
    // Alice: 3 posts, Bob: 2 posts, Carol: 1 post, Dave: 0 posts.
    let post_counts = [3, 2, 1, 0];
    for (person, &n) in people.iter().zip(&post_counts) {
        for i in 0..n {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(i));
            let post = store.create_node(&["Post"], props).unwrap();
            store.create_edge("HAS_CREATOR", post, *person, BTreeMap::new()).unwrap();
        }
    }

    let result = run(
        &store,
        "MATCH (p:Person)<-[:HAS_CREATOR]-(post:Post) \
         WITH p, count(post) AS postCount, collect(post.id) AS postIds \
         WHERE postCount > 0 \
         RETURN p.name AS name, postCount, postIds \
         ORDER BY postCount DESC \
         LIMIT 2",
    );
    assert_eq!(result.rows.len(), 2, "Dave (0 posts) filtered by WHERE, then LIMIT 2 of the remaining 3");
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(int_value(&result.rows[0][1]), 3);
    match &result.rows[0][2] {
        Value::List(items) => assert_eq!(items.len(), 3),
        other => panic!("expected a 3-item list, got {other:?}"),
    }
    assert_eq!(str_value(&result.rows[1][0]), "Bob");
    assert_eq!(int_value(&result.rows[1][1]), 2);
}

#[test]
fn match_create_connects_two_already_existing_nodes() {
    // Standalone CREATE can never do this -- every node token it sees is
    // always fresh. WITH-chaining is what lets two independently matched
    // *existing* nodes both stay bound in the same row for the CREATE tail.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    run(&store, "CREATE (b:Person {name: 'Bob'})");

    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) CREATE (a)-[:KNOWS]->(b)",
    );

    // No new Person nodes were created -- still exactly 2.
    let people = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(people.rows.len(), 2);

    let result = run(&store, "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Bob");
}

#[test]
fn match_create_adds_new_node_to_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");

    run(&store, "MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:OWNS]->(i:Item {name: 'Widget'})");

    let result = run(&store, "MATCH (a:Person)-[:OWNS]->(i:Item) RETURN a.name, i.name");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Widget");
}

#[test]
fn match_create_runs_once_per_matched_row() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (p:Person {{idx: {i}}})"));
    }

    run(&store, "MATCH (p:Person) CREATE (p)-[:HAS_LOG]->(l:Log)");

    let result = run(&store, "MATCH (p:Person)-[:HAS_LOG]->(l:Log) RETURN count(*)");
    assert_eq!(int_value(&result.rows[0][0]), 3, "one new Log node per matched Person row");
}

#[test]
fn match_create_rejects_relabeling_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    let stmt = parse("MATCH (a:Person {name: 'Alice'}) CREATE (a:Employee)-[:X]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("already bound"), "expected an already-bound error, got: {err}");
}

#[test]
fn match_create_rejects_variable_length_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)-[:NEXT*1..3]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}

#[test]
fn match_create_rejects_undirected_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)-[:X]-(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("undirected"));
}

#[test]
fn with_chaining_disjoint_second_match_cross_joins_carried_var() {
    // `b`'s pattern doesn't chain from `a` at all -- before the scan/seed
    // cross-join fix, this silently dropped `a` instead of producing the
    // 2x2 cross join real Cypher semantics require here.
    let store = GraphStore::open_memory().unwrap();
    for name in ["Alice", "Bob"] {
        run(&store, &format!("CREATE (:Left {{name: '{name}'}})"));
    }
    for name in ["X", "Y"] {
        run(&store, &format!("CREATE (:Right {{name: '{name}'}})"));
    }

    let result = run(
        &store,
        "MATCH (a:Left) WITH a MATCH (b:Right) RETURN a.name AS leftName, b.name AS rightName \
         ORDER BY leftName, rightName",
    );
    assert_eq!(result.rows.len(), 4, "2 Left x 2 Right must cross-join to 4 rows, not drop `a`");
    let pairs: Vec<(String, String)> =
        result.rows.iter().map(|r| (str_value(&r[0]), str_value(&r[1]))).collect();
    assert_eq!(
        pairs,
        vec![
            ("Alice".to_string(), "X".to_string()),
            ("Alice".to_string(), "Y".to_string()),
            ("Bob".to_string(), "X".to_string()),
            ("Bob".to_string(), "Y".to_string()),
        ]
    );
}

#[test]
fn optional_match_disjoint_pattern_does_not_panic() {
    // The OPTIONAL pattern doesn't chain from the outer `a` either -- same
    // root cause as the cross-join test above, but through
    // eval_optional_part's __seed_idx tagging instead of a plain MATCH.
    // Before the fix, scan() silently dropped that tag and
    // eval_optional_part's `unreachable!` fired.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Left {name: 'Alice'})");
    run(&store, "CREATE (:Right {name: 'X'})");

    let result = run(&store, "MATCH (a:Left) OPTIONAL MATCH (c:Right) RETURN a.name, c.name");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "X");
}

#[test]
fn string_literal_escaped_quote_round_trips() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, r"CREATE (:Person {name: 'O\'Brien'})");
    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "O'Brien");
}

#[test]
fn string_literal_escaped_backslash_and_common_escapes() {
    // Cypher source (raw string below is literal, no extra doubling
    // needed): `\\` is one escaped backslash, `\t`/`\n` are tab/newline.
    let store = GraphStore::open_memory().unwrap();
    run(&store, r"CREATE (:Path {p: 'C:\\Users\\x', tab: 'a\tb', nl: 'a\nb'})");
    let result = run(&store, "MATCH (n:Path) RETURN n.p, n.tab, n.nl");
    assert_eq!(str_value(&result.rows[0][0]), r"C:\Users\x");
    assert_eq!(str_value(&result.rows[0][1]), "a\tb");
    assert_eq!(str_value(&result.rows[0][2]), "a\nb");
}

#[test]
fn string_literal_unrecognized_escape_errors() {
    let err = parse(r"MATCH (n {x: 'a\qb'}) RETURN n").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("escape"));
}

#[test]
fn string_literal_without_backslash_unaffected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
}

#[test]
fn unwind_inline_list_fans_out_one_row_per_element() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "UNWIND [10, 20, 30] AS x RETURN x");
    let values: Vec<i64> = result.rows.iter().map(|r| int_value(&r[0])).collect();
    assert_eq!(values, vec![10, 20, 30]);
}

#[test]
fn unwind_cross_joins_against_existing_rows() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Bob'})");
    let result = run(&store, "MATCH (p:Person) UNWIND [1, 2] AS n RETURN p.name AS name, n ORDER BY name, n");
    let pairs: Vec<(String, i64)> = result.rows.iter().map(|r| (str_value(&r[0]), int_value(&r[1]))).collect();
    assert_eq!(
        pairs,
        vec![
            ("Alice".to_string(), 1),
            ("Alice".to_string(), 2),
            ("Bob".to_string(), 1),
            ("Bob".to_string(), 2),
        ]
    );
}

#[test]
fn unwind_collected_nodes_restores_graph_identity() {
    // The whole point of value_to_binding_restore: a node that went into
    // collect() as a Value::Node must come back out of UNWIND as a real
    // Binding::Node, not just a display value -- provable by traversing
    // further (m.name) after the UNWIND, which only works with real graph
    // identity, not a frozen snapshot value.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(c:Person {name: 'Carol'})");
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'})-[:KNOWS]->(f:Person) WITH collect(f) AS friends \
         UNWIND friends AS m RETURN m.name AS name ORDER BY name",
    );
    let names: Vec<String> = result.rows.iter().map(|r| str_value(&r[0])).collect();
    assert_eq!(names, vec!["Bob".to_string(), "Carol".to_string()]);
}

#[test]
fn unwind_own_where_filters_without_needing_a_second_with() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "UNWIND [1, 2, 3, 4, 5] AS x WHERE x > 2 RETURN x ORDER BY x");
    let values: Vec<i64> = result.rows.iter().map(|r| int_value(&r[0])).collect();
    assert_eq!(values, vec![3, 4, 5]);
}

#[test]
fn unwind_non_list_var_errors_clearly() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    // `p` is bound to a node, not a list -- UNWIND needs a real list
    // (e.g. from collect()), not any bound variable.
    let stmt = parse("MATCH (p:Person) UNWIND p AS x RETURN x").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("list"));
}

#[test]
fn merge_single_node_creates_then_reuses() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "MERGE (n:Person {name: 'Alice'})");
    run(&store, "MERGE (n:Person {name: 'Alice'})");
    let result = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(int_value(&result.rows[0][0]), 1, "second MERGE must reuse, not create a duplicate");
}

#[test]
fn merge_one_hop_both_endpoints_bound_reuses_existing_edge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})");
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) MERGE (a)-[:KNOWS]->(b)",
    );
    let result = run(&store, "MATCH (:Person)-[r:KNOWS]->(:Person) RETURN count(*)");
    assert_eq!(int_value(&result.rows[0][0]), 1, "must reuse the existing edge, not create a 2nd one");
}

#[test]
fn merge_one_hop_both_endpoints_bound_creates_missing_edge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Bob'})");
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) MERGE (a)-[:KNOWS]->(b)",
    );
    let result = run(&store, "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)");
    assert_eq!(int_value(&result.rows[0][0]), 1);
    // No new nodes -- both endpoints already existed, only the edge is new.
    let nodes = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(int_value(&nodes.rows[0][0]), 2);
}

#[test]
fn merge_one_fresh_endpoint_does_not_reuse_an_unconnected_matching_node() {
    // The wrong-answer scenario the plan review flagged: an independent
    // per-token scan would find this pre-existing, unconnected Bob and
    // wrongly reuse it. The correct (composite) search must come up empty
    // and create a brand-new, properly-connected Bob instead.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Bob'})"); // unconnected to Alice
    run(&store, "MATCH (a:Person {name: 'Alice'}) MERGE (a)-[:KNOWS]->(b:Person {name: 'Bob'})");

    let bobs = run(&store, "MATCH (n:Person {name: 'Bob'}) RETURN count(*)");
    assert_eq!(int_value(&bobs.rows[0][0]), 2, "must create a 2nd Bob, not reuse the unconnected one");
    let connected =
        run(&store, "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)");
    assert_eq!(int_value(&connected.rows[0][0]), 1);
}

#[test]
fn merge_standalone_both_endpoints_fresh() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");
    run(&store, "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");
    let nodes = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(int_value(&nodes.rows[0][0]), 2, "2nd MERGE must reuse both nodes and the edge, not duplicate");
    let edges = run(&store, "MATCH (:Person)-[:KNOWS]->(:Person) RETURN count(*)");
    assert_eq!(int_value(&edges.rows[0][0]), 1);
}

#[test]
fn merge_on_create_and_on_match_fire_on_the_right_rows() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2");
    let after_create = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_create.rows[0][0]), 1);

    run(&store, "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2");
    let after_match = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_match.rows[0][0]), 2);
}

#[test]
fn merge_unconstrained_node_pattern_errors() {
    let err = parse("MERGE (n) RETURN n").and_then(|stmt| {
        let store = GraphStore::open_memory().unwrap();
        Executor::new(&store).execute(&stmt)
    });
    let err = err.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("ambiguous"));
}

#[test]
fn merge_two_hop_pattern_errors_at_parse_time() {
    let err = parse("MERGE (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("one relationship hop"));
}

#[test]
fn merge_can_be_followed_by_return() {
    // The whole reason MERGE is a QueryClause, not a Tail -- unlike
    // MATCH...CREATE, MATCH...MERGE...RETURN must work.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "MERGE (n:Person {name: 'Alice'}) RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
}

fn path_elems(v: &Value) -> &[PathElem] {
    match v {
        Value::Path(elems) => elems,
        other => panic!("expected a path, got {other:?}"),
    }
}

fn node_name(elem: &PathElem) -> &str {
    match elem {
        PathElem::Node(n) => match n.props.get("name") {
            Some(marsdb_graph::PropertyValue::String(s)) => s.as_str(),
            other => panic!("expected node to have a string 'name' prop, got {other:?}"),
        },
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn named_path_capture_returns_alternating_node_edge_elements() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})-[:LIKES]->(:Person {name: 'Carol'})",
    );
    let result = run(
        &store,
        "MATCH p = (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person)-[:LIKES]->(c:Person) RETURN p",
    );
    let elems = path_elems(&result.rows[0][0]);
    assert_eq!(elems.len(), 5, "node,edge,node,edge,node");
    assert_eq!(node_name(&elems[0]), "Alice");
    assert!(matches!(&elems[1], PathElem::Edge(e) if e.label == "KNOWS"));
    assert_eq!(node_name(&elems[2]), "Bob");
    assert!(matches!(&elems[3], PathElem::Edge(e) if e.label == "LIKES"));
    assert_eq!(node_name(&elems[4]), "Carol");
}

#[test]
fn named_path_capture_with_anonymous_relationships() {
    // Neither hop's relationship is named -- name_pattern_for_path must
    // still track them internally (synthesized names), and they must not
    // leak into the output row (only `p` should be bound/returned here).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})");
    let result = run(&store, "MATCH p = (a:Person {name: 'Alice'})-[]->(b:Person) RETURN p");
    let elems = path_elems(&result.rows[0][0]);
    assert_eq!(elems.len(), 3);
    assert_eq!(node_name(&elems[0]), "Alice");
    assert_eq!(node_name(&elems[2]), "Bob");
}

#[test]
fn shortest_path_finds_the_actual_shortest_not_just_a_path() {
    let store = GraphStore::open_memory().unwrap();
    // Direct 1-hop route.
    run(&store, "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Dave'})");
    // Longer 3-hop route between the *same* two people -- MATCH...CREATE,
    // not a chained plain CREATE, so the trailing (:Person{name:'Dave'})
    // token reuses the existing Dave instead of silently creating a 2nd
    // one (a chained CREATE never reuses an unbound token, even one that
    // matches an existing node by props -- exactly the gap MATCH...CREATE
    // exists to work around).
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (d:Person {name: 'Dave'}) \
         CREATE (a)-[:KNOWS]->(:Person {name: 'Bob'})-[:KNOWS]->(:Person {name: 'Carol'})-[:KNOWS]->(d)",
    );
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) OPTIONAL MATCH (d:Person {name: 'Dave'}) \
         OPTIONAL MATCH p = shortestPath((a)-[:KNOWS*]-(d)) RETURN length(p)",
    );
    assert_eq!(result.rows.len(), 1, "only one Dave -- must not have duplicated it");
    assert_eq!(int_value(&result.rows[0][0]), 1, "must pick the 1-hop route, not the 3-hop one");
}

#[test]
fn shortest_path_returns_null_when_unreachable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Zed'})");
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) OPTIONAL MATCH (z:Person {name: 'Zed'}) \
         OPTIONAL MATCH p = shortestPath((a)-[:KNOWS*]-(z)) RETURN p",
    );
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn shortest_path_requires_both_endpoints_already_bound() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    let stmt = parse("MATCH p = shortestPath((a:Person {name: 'Alice'})-[:KNOWS*]-(z:Person)) RETURN p").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("shortestpath"));
}

#[test]
fn named_path_over_variable_length_pattern_errors_at_parse_time() {
    let err = parse("MATCH p = (a)-[:KNOWS*1..3]->(b) RETURN p").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}

#[test]
fn shortest_path_requires_a_variable_length_hop() {
    let err = parse("MATCH p = shortestPath((a)-[:KNOWS]->(b)) RETURN p").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}
