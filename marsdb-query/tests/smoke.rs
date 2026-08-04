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
fn to_integer_parses_a_float_formatted_string_by_truncating() {
    // Regression: `toInteger('1.7')` used to fail straight to null since
    // the string-parse path only ever tried an i64 parse.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [2, 2.9, '1.7'] AS things RETURN [n IN things | toInteger(n)] AS x");
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 2, 1]);
}

#[test]
fn to_integer_on_an_unparseable_string_is_null_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH ['2', '2.9', 'foo'] AS numbers RETURN [n IN numbers | toInteger(n)] AS x");
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(int(&items[0]), 2);
            assert_eq!(int(&items[1]), 2);
            assert!(matches!(items[2], Value::Null));
        }
        other => panic!("expected a List, got {other:?}"),
    }
}

#[test]
fn to_integer_on_a_list_errors_instead_of_silently_nulling() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN toInteger([1, 2])").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("tointeger"));
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

fn int(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        Value::Literal(marsdb_query::Literal::Int(i)) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

fn list_ints(v: &Value) -> Vec<i64> {
    match v {
        Value::List(items) => items.iter().map(int).collect(),
        other => panic!("expected List, got {other:?}"),
    }
}

fn bool_val(v: &Value) -> bool {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Bool(b)) => *b,
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

#[test]
fn standalone_with_no_preceding_match() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN list");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
}

#[test]
fn with_list_binds_as_a_real_list_not_null() {
    // Regression: `item_binding` used to route any non-Var WITH item
    // through `value_to_property_value`, which silently collapsed
    // Value::List/Node/Edge to PropertyValue::Null.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 1 + 1, 2 * 2] AS list RETURN list");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 4]);
}

#[test]
fn with_binds_a_node_as_a_real_node_not_null() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:X {n: 1})-[:R]->(b:Y {n: 2})");
    let result = run(
        &store,
        "MATCH (a:X)-[:R]->(b:Y) WITH CASE a.n WHEN 1 THEN a ELSE b END AS chosen RETURN chosen",
    );
    match &result.rows[0][0] {
        Value::Node(_) => {}
        other => panic!("expected a Node, got {other:?}"),
    }
}

#[test]
fn list_index_by_position() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3, 4, 5] AS list RETURN list[0], list[2]");
    assert_eq!(int(&result.rows[0][0]), 1);
    assert_eq!(int(&result.rows[0][1]), 3);
}

#[test]
fn list_index_negative_counts_from_end() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3, 4, 5] AS list RETURN list[-1]");
    assert_eq!(int(&result.rows[0][0]), 5);
}

#[test]
fn list_index_out_of_bounds_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN list[10], list[-10]");
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
}

#[test]
fn list_slice_basic_and_open_ended() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN list[1..3], list[..2], list[2..]",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 3]);
    assert_eq!(list_ints(&result.rows[0][1]), vec![1, 2]);
    assert_eq!(list_ints(&result.rows[0][2]), vec![3, 4, 5]);
}

#[test]
fn list_comprehension_filter_and_project() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN [x IN list WHERE x % 2 = 0 | x * 10] AS y",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![20, 40]);
}

#[test]
fn list_comprehension_project_only() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN [x IN list | x * 2] AS y");
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 4, 6]);
}

#[test]
fn list_comprehension_filter_only() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3, 4, 5] AS list RETURN [x IN list WHERE x > 2] AS y");
    assert_eq!(list_ints(&result.rows[0][0]), vec![3, 4, 5]);
}

#[test]
fn list_comprehension_bare_identity() {
    // No WHERE, no projection -- a legal no-op comprehension.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN [x IN list] AS y");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
}

#[test]
fn list_comprehension_over_collected_nodes_extracts_a_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Label1 {name: 'original'})");
    let result = run(
        &store,
        "MATCH (a:Label1) WITH collect(a) AS nodes RETURN [x IN nodes | x.name] AS oldNames",
    );
    match &result.rows[0][0] {
        Value::List(items) => match &items[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "original"),
            other => panic!("expected a String property, got {other:?}"),
        },
        other => panic!("expected a List, got {other:?}"),
    }
}

#[test]
fn list_comprehension_plain_list_with_bare_identifier_is_not_misparsed_as_a_comprehension() {
    // `x` alone in a list (no `IN` following) must fall through to the
    // ordinary comma-separated list_expr alternative, not be swallowed
    // partway through a failed list_comprehension attempt.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH 1 AS x, 2 AS y RETURN [x, y]");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2]);
}

#[test]
fn quantifier_all_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN all(x IN [1, 2, 3] WHERE x > 0) AS a, all(x IN [1, 2, 3] WHERE x > 1) AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_any_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN any(x IN [1, 2, 3] WHERE x > 2) AS a, any(x IN [1, 2, 3] WHERE x > 5) AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_none_on_empty_list_is_true() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN none(x IN [] WHERE x > 0) AS a");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn quantifier_single_counts_exact_matches() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN single(x IN [1, 2, 3] WHERE x = 2) AS a, single(x IN [1, 2, 2] WHERE x = 2) AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_over_collected_nodes_scopes_the_bound_variable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Label1 {name: 'a'})");
    let result = run(
        &store,
        "MATCH (a:Label1) WITH collect(a) AS nodes RETURN none(x IN nodes WHERE x.name = 'a') AS result",
    );
    assert!(!bool_val(&result.rows[0][0]));
}

#[test]
fn quantifier_three_valued_null_propagation() {
    // Regression: a first version collapsed a null predicate straight to
    // false, which happened to pass every non-null-list scenario but was
    // wrong on lists containing nulls -- a definite true/false among the
    // elements still decides the answer even with nulls present; only
    // "no definite answer, but at least one unknown" is null.
    let store = GraphStore::open_memory().unwrap();

    let all = run(&store, "RETURN all(x IN [null] WHERE x = 2) AS a, all(x IN [0, null] WHERE x = 2) AS b, all(x IN [2, null] WHERE x = 2) AS c");
    assert!(matches!(all.rows[0][0], Value::Null));
    assert!(!bool_val(&all.rows[0][1]));
    assert!(matches!(all.rows[0][2], Value::Null));

    let any = run(&store, "RETURN any(x IN [null] WHERE x = 2) AS a, any(x IN [2, null] WHERE x = 2) AS b");
    assert!(matches!(any.rows[0][0], Value::Null));
    assert!(bool_val(&any.rows[0][1]));

    let none = run(&store, "RETURN none(x IN [null] WHERE x = 2) AS a, none(x IN [2, null] WHERE x = 2) AS b");
    assert!(matches!(none.rows[0][0], Value::Null));
    assert!(!bool_val(&none.rows[0][1]));

    let single = run(
        &store,
        "RETURN single(x IN [2, null] WHERE x = 2) AS a, single(x IN [34, 0, null, 5, 900] WHERE x < 10) AS b",
    );
    assert!(matches!(single.rows[0][0], Value::Null));
    assert!(!bool_val(&single.rows[0][1]));
}

#[test]
fn quantifier_does_not_break_ordinary_function_calls() {
    // Regression: `ALL(...)` etc share `identifier ~ "("` with an ordinary
    // function_call -- an unrelated call like coalesce(...) must still
    // fall through to function_call, not get swallowed by a failed
    // quantifier_expr attempt.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN coalesce(null, 5) AS x");
    assert_eq!(int(&result.rows[0][0]), 5);
}

#[test]
fn map_literal_property_access() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH {existing: 42, notMissing: null} AS m RETURN m.missing, m.notMissing, m.existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);
}

#[test]
fn map_literal_with_expression_values() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN {a: 1, b: 1 + 1} AS m");
    match &result.rows[0][0] {
        Value::Map(m) => {
            assert_eq!(int(m.get("a").unwrap()), 1);
            assert_eq!(int(m.get("b").unwrap()), 2);
        }
        other => panic!("expected a Map, got {other:?}"),
    }
}

#[test]
fn map_literal_property_access_on_null_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH null AS m RETURN m.missing");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn boolean_expr_and_or_xor_not() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN true AND false AS a, true OR false AS b, true XOR true AS c, NOT true AS d",
    );
    assert!(!bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(!bool_val(&result.rows[0][2]));
    assert!(!bool_val(&result.rows[0][3]));
}

#[test]
fn boolean_expr_comparison_as_return_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN 1 = 1 AS a, 1 < 2 AS b, 2 > 3 AS c, 'ab' STARTS WITH 'a' AS d");
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(!bool_val(&result.rows[0][2]));
    assert!(bool_val(&result.rows[0][3]));
}

#[test]
fn boolean_expr_precedence_and_binds_tighter_than_or() {
    let store = GraphStore::open_memory().unwrap();
    // AND binds tighter than OR: false AND false = false, then true OR
    // false = true -- if OR bound tighter this would instead need to
    // evaluate (true OR false) AND false = false.
    let result = run(&store, "RETURN true OR false AND false AS x");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn boolean_expr_not_binds_looser_than_comparison() {
    let store = GraphStore::open_memory().unwrap();
    // NOT (1 = 2), not (NOT 1) = 2 -- comparison binds tighter.
    let result = run(&store, "RETURN NOT 1 = 2 AS x");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn boolean_expr_three_valued_null_propagation() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN null AND false AS a, null AND true AS b, null OR true AS c, null OR false AS d",
    );
    assert!(!bool_val(&result.rows[0][0])); // false wins over unknown
    assert!(matches!(result.rows[0][1], Value::Null));
    assert!(bool_val(&result.rows[0][2])); // true wins over unknown
    assert!(matches!(result.rows[0][3], Value::Null));
}

#[test]
fn boolean_expr_non_bool_operand_errors() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AND true").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("boolean"));
}

#[test]
fn with_where_comparison_followed_by_order_by_still_parses() {
    // Regression: widening `return_expr` to include an optional trailing
    // comparison broke `with_comparison`'s own separate `return_expr ~
    // compare_op ~ literal` shape (used by WITH's own WHERE) -- the
    // return_expr operand greedily swallowed the whole `y > 10` itself,
    // leaving nothing for with_comparison's own trailing compare_op to
    // match. Fixed by narrowing with_comparison's LHS to add_expr.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item {idx: 20})");
    run(&store, "CREATE (:Item {idx: 5})");
    let result = run(&store, "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 RETURN y ORDER BY y");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 20);
}

#[test]
fn return_expr_or_immediately_before_order_by_does_not_swallow_order() {
    // Regression: a bare `^"OR"` keyword has no word-boundary check, so
    // it happily matched the first two letters of `ORDER`, mis-parsing
    // `RETURN x OR y ORDER BY z` as `RETURN (x OR y OR DER) BY z` --
    // caught because `y ORDER BY y` (nothing between the boolean
    // expression and the ORDER BY clause) is exactly what a `RETURN`
    // item's own trailing structure looks like.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item {idx: 2})");
    run(&store, "CREATE (:Item {idx: 1})");
    let result = run(&store, "MATCH (n:Item) RETURN n.idx = 1 OR n.idx = 2 AS x, n.idx ORDER BY n.idx DESC");
    assert_eq!(result.rows.len(), 2);
    assert_eq!(int(&result.rows[0][1]), 2);
    assert_eq!(int(&result.rows[1][1]), 1);
}

#[test]
fn list_equality_is_structural_not_null() {
    // Regression: compare_values used to reduce List/Map operands
    // through value_to_property_value, which collapses both to
    // PropertyValue::Null -- every list/map `=`/`<>` comparison silently
    // became `null` regardless of actual content.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN [1, 2] = [1, 2] AS a, [1, 2] = [1, 3] AS b, [null] = [1] AS c");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
}

#[test]
fn list_ordering_is_lexicographic() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN [1, 0] >= [1] AS a, [1, null] >= [1] AS b, [1, 2] >= [1, null] AS c");
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
}

#[test]
fn boolean_ordering_false_less_than_true() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN false <= true AS x, false > true AS y");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn type_mismatch_comparison_semantics_differ_by_operator() {
    // Regression: a single blanket "type mismatch -> false" was wrong for
    // three different operator families -- confirmed against real TCK
    // scenarios: `=` on mismatched types is false, `<>` is true (never
    // equal, so "not equal" holds), ordering is null (no defined order),
    // and STARTS WITH/ENDS WITH/CONTAINS on a non-string operand is also
    // null, not false.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN (1 = 'a') AS a, (1 <> 'a') AS b, ('1.0' < 1.0) AS c, ('abc' STARTS WITH true) AS d");
    assert!(!bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
    assert!(matches!(result.rows[0][3], Value::Null));
}

#[test]
fn list_comprehension_bare_where_now_parses() {
    // Regression: previously `filter_expr`'s WHERE reused WithExpr, which
    // only ever wrapped a single Compare -- a bare boolean value (`WHERE
    // x`/`WHERE true`) failed to parse. Now that boolean logic is a real
    // ReturnExpr, this works.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [true, false, true] AS list RETURN [x IN list WHERE x] AS y");
    match &result.rows[0][0] {
        Value::List(items) => assert_eq!(items.len(), 2),
        other => panic!("expected a List, got {other:?}"),
    }
}

#[test]
fn quantifier_bare_where_now_parses() {
    // Just the parsing gap this task fixes -- none() on an empty list is
    // vacuously true regardless of the WHERE condition (real Cypher
    // semantics, already covered by quantifier_none_on_empty_list_is_true).
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN none(x IN [] WHERE true) AS a, none(x IN [] WHERE false) AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
}

#[test]
fn list_slice_out_of_range_bounds_clamp_instead_of_null() {
    // Regression guard: unlike single-element indexing, out-of-range slice
    // bounds clamp to [0, len] rather than producing null.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN list[-100..100], list[5..10]");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
    assert_eq!(list_ints(&result.rows[0][1]), Vec::<i64>::new());
}

// --- `<mutating-clause> RETURN ...` (SET/DELETE/DETACH DELETE/REMOVE/
// MATCH...CREATE followed directly by a RETURN in the same statement) ---

#[test]
fn set_then_return_sees_the_just_set_value() {
    // The RETURN must see the *updated* property, not the pre-SET one --
    // materialize_set applies the mutation before materialize_return runs,
    // same real-Cypher shape as TCK's Set2.feature scenario [1].
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {property1: 'orig'})");
    let result = run(&store, "MATCH (n:A) SET n.property1 = 'updated' RETURN n.property1");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "updated");
}

#[test]
fn set_label_then_return_labels() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:X)");
    let result = run(&store, "MATCH (n:X) SET n:Foo RETURN n");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Node(node) => {
            let mut labels = node.labels.clone();
            labels.sort();
            assert_eq!(labels, vec!["Foo".to_string(), "X".to_string()]);
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn delete_then_return_computed_value_not_the_deleted_var() {
    // Real TCK DELETE+RETURN scenarios (Delete1/Delete4/Delete6) never
    // RETURN the deleted variable's live properties -- they return a
    // computed value (a literal, count(*), or a WITH-projected scalar
    // captured before the delete). Exact shape and expected count (2, not
    // 1 -- the undirected pattern matches both directions) from Delete4's
    // scenario [1]: "Undirected expand followed by delete and count".
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A)-[:R]->(b:B)");
    let result = run(&store, "MATCH (a)-[r]-(b) DELETE r, a, b RETURN count(*) AS c");
    assert_eq!(int_value(&result.rows[0][0]), 2);
    // The delete itself really happened -- nothing left to match.
    let remaining = run(&store, "MATCH (n) RETURN n");
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn delete_then_return_the_deleted_var_itself_errors_not_panics() {
    // Not a shape any real TCK scenario directly tests with a bare `RETURN
    // n`, but real TCK scenarios *do* test the property-access cousin of
    // this shape (`MATCH (n) DELETE n RETURN n.num` must raise
    // `DeletedEntityAccess`, TCK's Return2 [15]/[17]) -- and `materialize_
    // delete` runs the physical delete before evaluating the trailing
    // RETURN to get that right. `binding_to_value`/`lookup_prop` (via
    // `deleted_entity_access`) must turn "the bound id's record is gone"
    // into a proper `QueryError`, not a panic — this is the regression
    // guard for that path specifically (accessing the whole node, not just
    // one of its properties).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p: 1})");
    let stmt = parse("MATCH (n:A) DELETE n RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("no longer exists"), "expected a deleted-entity error, got: {err}");
    // A failed statement rolls back its whole write transaction (see
    // `Executor::execute`'s abort-on-error path) -- the delete itself must
    // NOT have taken effect, same as any other error mid-statement.
    let remaining = run(&store, "MATCH (n:A) RETURN n");
    assert_eq!(remaining.rows.len(), 1, "a failed statement must roll back, not partially apply its delete");
}

#[test]
fn delete_then_return_a_property_of_the_deleted_var_errors() {
    // TCK Return2 scenarios [15]/[17]: accessing a property of a just-
    // deleted node/relationship must raise DeletedEntityAccess, not
    // silently succeed with the pre-delete value.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n {num: 0})");
    let stmt = parse("MATCH (n) DELETE n RETURN n.num").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("no longer exists"), "expected a deleted-entity error, got: {err}");

    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE ()-[:T {num: 0}]->()");
    let stmt2 = parse("MATCH ()-[r]->() DELETE r RETURN r.num").unwrap();
    let err2 = Executor::new(&store2).execute(&stmt2).unwrap_err();
    assert!(err2.to_string().to_lowercase().contains("no longer exists"), "expected a deleted-entity error, got: {err2}");
}

#[test]
fn detach_delete_then_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})");
    let result = run(&store, "MATCH (n:Person {name: 'Alice'}) DETACH DELETE n RETURN 42 AS num");
    // `42` is a bare literal, not a node/edge property -- eval_return_expr
    // yields Value::Literal here, not Value::Property.
    assert!(matches!(&result.rows[0][0], Value::Literal(marsdb_query::Literal::Int(42))));
    let remaining = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(remaining.rows.len(), 1);
    assert_eq!(str_value(&remaining.rows[0][0]), "Bob");
}

#[test]
fn optional_match_delete_null_return_null() {
    // TCK Delete1 scenario [5]: "Ignore null when deleting node" -- an
    // OPTIONAL MATCH that finds nothing pads with a null binding, DELETE on
    // null is a documented no-op, and the trailing RETURN of that same
    // (null) variable must round-trip as null, not error.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Real)");
    let result = run(&store, "OPTIONAL MATCH (a:DoesNotExist) DELETE a RETURN a");
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn remove_then_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p1: 1, p2: 2})");
    let result = run(&store, "MATCH (n:A) REMOVE n.p1 RETURN n");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Node(node) => {
            assert!(!node.props.contains_key("p1"));
            assert_eq!(int_value(&Value::Property(node.props.get("p2").unwrap().clone())), 2);
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn match_create_then_return_sees_the_newly_created_binding() {
    // The trailing RETURN must see `i`, the node CREATE just made in this
    // same statement -- materialize_create threads each row's updated
    // bindings forward for exactly this.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    let result = run(&store, "MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:OWNS]->(i:Item {name: 'Widget'}) RETURN i.name");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Widget");
}

#[test]
fn set_then_return_distinct_dedups() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A)-[:R]->(x:X {tag: 'same'})");
    run(&store, "CREATE (b:A)-[:R]->(y:X {tag: 'same'})");
    let result = run(&store, "MATCH (a:A)-[:R]->(x:X) SET a.touched = true RETURN DISTINCT x.tag");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "same");
}

#[test]
fn set_then_return_with_param_substitution() {
    // Regression guard for `params::substitute_tail`/`substitute_return_tail`
    // -- a `$param` inside the trailing RETURN of a mutating tail must be
    // resolved just like one inside the mutating clause itself.
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p: 1})");
    let mut params = HashMap::new();
    params.insert("newp".to_string(), marsdb_graph::PropertyValue::Int(99));
    let mut stmt = marsdb_query::parse("MATCH (n:A) SET n.p = 2 RETURN $newp AS x").unwrap();
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert!(matches!(&result.rows[0][0], Value::Literal(marsdb_query::Literal::Int(99))));
    let after = run(&store, "MATCH (n:A) RETURN n.p");
    assert_eq!(int_value(&after.rows[0][0]), 2);
}

#[test]
fn set_property_to_null_removes_it_not_stores_a_null_value() {
    // Regression guard for a real bug found while adding SET...RETURN:
    // `SET n.prop = null` must *remove* the property (real Cypher, TCK's
    // Set2 "Set a Property to Null" scenarios), not store a literal
    // `PropertyValue::Null` under that key -- the two are observably
    // different (a stored null still shows up when a node's props are
    // enumerated). This bug pre-dated SET...RETURN but was unreachable
    // until a RETURN could follow SET to observe it.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {property1: 45, property2: 46})");
    let result = run(&store, "MATCH (n:A) SET n.property1 = null RETURN n");
    match &result.rows[0][0] {
        Value::Node(node) => {
            assert!(!node.props.contains_key("property1"), "property1 must be gone, not null: {:?}", node.props);
            assert_eq!(int_value(&Value::Property(node.props.get("property2").unwrap().clone())), 46);
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn set_and_remove_on_a_null_binding_are_silent_no_ops() {
    // Regression guard for a second real bug found the same way: an
    // OPTIONAL MATCH miss pads its variable with a null binding, and
    // SET/REMOVE (property *and* label forms) on that null must be silent
    // no-ops -- same documented behavior DELETE already had -- not an
    // "isn't a node" error. TCK's Set1/Set3/Remove1/Remove2 "Ignore null
    // when setting/removing property/label" scenarios.
    let store = GraphStore::open_memory().unwrap();
    let prop_set = run(&store, "OPTIONAL MATCH (a:DoesNotExist) SET a.num = 42 RETURN a");
    assert!(matches!(prop_set.rows[0][0], Value::Null));
    let label_set = run(&store, "OPTIONAL MATCH (a:DoesNotExist) SET a:L RETURN a");
    assert!(matches!(label_set.rows[0][0], Value::Null));
    let prop_remove = run(&store, "OPTIONAL MATCH (a:DoesNotExist) REMOVE a.num RETURN a");
    assert!(matches!(prop_remove.rows[0][0], Value::Null));
    let label_remove = run(&store, "OPTIONAL MATCH (a:DoesNotExist) REMOVE a:L RETURN a");
    assert!(matches!(label_remove.rows[0][0], Value::Null));
}

#[test]
fn mutating_tail_with_no_return_is_still_terminal() {
    // Regression guard: the grammar change (`return_clause?` after a
    // mutating clause) must not force a RETURN -- the pre-existing
    // terminal-mutation shape (no trailing RETURN at all) still has to
    // keep working exactly as before.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p: 1})");
    run(&store, "MATCH (n:A) SET n.p = 2");
    let result = run(&store, "MATCH (n:A) RETURN n.p");
    assert_eq!(int_value(&result.rows[0][0]), 2);
}
