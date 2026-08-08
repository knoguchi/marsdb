//! Smoke tests: count/collect/avg/sum/percentile and grouping semantics -- split from the original smoke.rs.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

/// An aggregating `WITH ... WHERE` must still only see the grouped/
/// aggregated names -- no pre-WITH fallback, since aggregation collapses
/// many rows into one and there's no single "the" pre-WITH row left.
#[test]
fn aggregating_with_where_does_not_fall_back_to_pre_with_scope() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:P {name: 'A'})-[:REL]->(), (a)-[:REL]->(), (b:P {name: 'B'})-[:REL]->()",
    );
    let result = run(
        &store,
        "MATCH (a)-->() WITH a, count(*) AS relCount WHERE relCount > 1 RETURN a.name",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "A"),
        other => panic!("unexpected value {other:?}"),
    }
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

#[test]
fn pattern_comprehension_can_be_aggregated_over() {
    // TCK expressions/pattern Pattern2 [6]: `count([p = (n)-[:HAS]->() | p])`
    // -- a pattern comprehension used directly as an aggregate argument.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A), (:A), (:A)");
    run(
        &store,
        "MATCH (a:A) WHERE NOT (a)--() WITH a LIMIT 1 CREATE (a)-[:HAS]->()",
    );
    let result = run(
        &store,
        "MATCH (n:A) RETURN count([p = (n)-[:HAS]->() | p]) AS c",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(n)) => assert_eq!(*n, 3),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn percentile_cont_and_disc_match_tck_aggregation6() {
    // TCK expressions/aggregation Aggregation6 [1]/[2]: percentileDisc/
    // percentileCont(n.price, p) over {10.0, 20.0, 30.0}.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({price: 10.0})");
    run(&store, "CREATE ({price: 20.0})");
    run(&store, "CREATE ({price: 30.0})");

    fn float_at(result: &marsdb_query::QueryResult) -> f64 {
        match &result.rows[0][0] {
            Value::Property(marsdb_graph::PropertyValue::Float(f)) => *f,
            other => panic!("unexpected value {other:?}"),
        }
    }

    for (p, expected) in [(0.0, 10.0), (0.5, 20.0), (1.0, 30.0)] {
        let result = run(
            &store,
            &format!("MATCH (n) RETURN percentileDisc(n.price, {p}) AS p"),
        );
        assert_eq!(float_at(&result), expected, "percentileDisc({p})");
        let result = run(
            &store,
            &format!("MATCH (n) RETURN percentileCont(n.price, {p}) AS p"),
        );
        assert_eq!(float_at(&result), expected, "percentileCont({p})");
    }
}

#[test]
fn percentile_cont_interpolates_between_ranks() {
    // percentileCont interpolates linearly between the two closest ranks
    // when the percentile doesn't land exactly on one; percentileDisc
    // never interpolates, it always returns one of the actual inputs.
    let store = GraphStore::open_memory().unwrap();
    for price in ["10.0", "20.0", "30.0", "40.0"] {
        run(&store, &format!("CREATE ({{price: {price}}})"));
    }
    let result = run(&store, "MATCH (n) RETURN percentileCont(n.price, 0.5) AS p");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => {
            assert!((*f - 25.0).abs() < 1e-9, "expected 25.0, got {f}")
        }
        other => panic!("unexpected value {other:?}"),
    }
    let result = run(&store, "MATCH (n) RETURN percentileDisc(n.price, 0.5) AS p");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => {
            assert!(
                *f == 20.0 || *f == 30.0,
                "expected an actual input, got {f}"
            )
        }
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn percentile_out_of_range_is_a_runtime_error() {
    // TCK Aggregation6 [3]/[4]: percentile must be in 0.0..=1.0.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({price: 10.0})");
    for bad in ["1000", "-1", "1.1"] {
        let stmt = parse(&format!("MATCH (n) RETURN percentileCont(n.price, {bad})")).unwrap();
        assert!(Executor::new(&store).execute(&stmt).is_err());
        let stmt = parse(&format!("MATCH (n) RETURN percentileDisc(n.price, {bad})")).unwrap();
        assert!(Executor::new(&store).execute(&stmt).is_err());
    }
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
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS x, count(*) AS c ORDER BY x DESC LIMIT 2",
    );
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

/// An aggregate in ORDER BY is only legal when the RETURN/WITH clause
/// itself is aggregating -- a compile-time error otherwise (real Cypher's
/// `InvalidAggregation`), not a runtime one. TCK's ReturnOrderBy2 [14] /
/// WithOrderBy2 [25].
#[test]
fn order_by_aggregate_without_aggregating_return_or_with_is_a_semantic_error() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n) RETURN n.idx ORDER BY max(n.idx)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregat"));

    let stmt = parse("MATCH (n) WITH n.idx AS x ORDER BY count(1) RETURN x").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregat"));
}

/// An ORDER BY item that repeats a RETURN/WITH item's expression verbatim
/// refers to that already-aggregated item, not a fresh expression -- must
/// not be re-evaluated against pre-aggregation bindings (which no longer
/// exist post-grouping). Real, previously-broken TCK scenarios: WithOrderBy4
/// [11] (aliased, still matched by expression) and ReturnOrderBy3 (unaliased).
#[test]
fn order_by_repeats_an_aggregating_return_or_with_item_verbatim() {
    let store = GraphStore::open_memory().unwrap();
    for (num, num2) in [(1, 4), (5, 2), (9, 0), (3, 3), (7, 1)] {
        run(&store, &format!("CREATE (:A {{num: {num}, num2: {num2}}})"));
    }
    let result = run(
        &store,
        "MATCH (a:A) WITH a.num2 % 3 AS mod, sum(a.num + a.num2) AS sum \
         ORDER BY sum(a.num + a.num2) LIMIT 2 RETURN mod, sum",
    );
    let rows: Vec<(i64, i64)> = result
        .rows
        .iter()
        .map(|row| {
            let get = |v: &Value| match v {
                Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
                other => panic!("unexpected value {other:?}"),
            };
            (get(&row[0]), get(&row[1]))
        })
        .collect();
    assert_eq!(rows, vec![(2, 7), (1, 13)]);

    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE (:Person {division: 'Sweden'})");
    run(&store2, "CREATE (:Person {division: 'Sweden'})");
    run(&store2, "CREATE (:Person {division: 'England'})");
    run(&store2, "CREATE (:Person {division: 'Germany'})");
    let result2 = run(
        &store2,
        "MATCH (n:Person) RETURN n.division, count(*) ORDER BY count(*) DESC, n.division ASC",
    );
    let divisions: Vec<String> = result2
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(divisions, vec!["Sweden", "England", "Germany"]);
}

/// An ORDER BY aggregate that matches *no* RETURN/WITH item (not even by
/// expression identity) is still rejected, even though the tail itself
/// aggregates -- TCK's WithOrderBy4 [14].
#[test]
fn order_by_aggregate_not_matching_any_aggregating_item_is_an_error() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {num: 1, num2: 4})");
    let stmt = parse(
        "MATCH (a:A) WITH a.num2 % 3 AS mod, min(a.num + a.num2) AS min \
         ORDER BY sum(a.num + a.num2) LIMIT 2 RETURN mod, min",
    )
    .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregat"));
}

/// TCK ReturnOrderBy6 [2]/[3]: an aggregating RETURN's own ORDER BY key
/// can be *composed* (an aggregate combined with other values), as long
/// as every non-aggregate leaf is an explicit grouping key -- either the
/// item's own expression verbatim, or (unlike a plain composed RETURN
/// item) its output *alias*, and every aggregate call in it matches some
/// existing item (`age + count(you.age)` and `me.age + count(you.age)`
/// both use `count(you.age)`, item1's own expression verbatim).
#[test]
fn return_order_by_composed_aggregate_expression_sorts_by_the_computed_value() {
    // me.age=100 has 1 outgoing KNOWS (combined value 101); me.age=1 has
    // 3 (combined value 4) -- DESC order by the combined value puts
    // age=100 first, even though age alone would sort the other way.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:P {age: 100})-[:KNOWS]->(:F), \
                (b:P {age: 1})-[:KNOWS]->(:F), \
                (b)-[:KNOWS]->(:F), \
                (b)-[:KNOWS]->(:F)",
    );
    let result = run(
        &store,
        "MATCH (me:P)-[:KNOWS]->(you:F) RETURN me.age AS age, count(you) AS cnt \
         ORDER BY age + count(you) DESC",
    );
    let ages: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    assert_eq!(ages, vec![100, 1]);
}

/// TCK WithOrderBy4 [16]: a WITH's own composed ORDER BY key mixing a
/// constant/parameter with an aggregate that repeats a WITH item's
/// expression verbatim.
#[test]
fn with_order_by_composed_expression_mixing_constant_and_aggregate() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({age: 10})");
    run(&store, "CREATE ({age: 20})");
    let result = run(
        &store,
        "MATCH (person) WITH avg(person.age) AS avgAge \
         ORDER BY 1000 + avg(person.age) - 1000 RETURN avgAge",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => {
            assert!((*f - 15.0).abs() < 1e-9, "expected 15.0, got {f}")
        }
        other => panic!("unexpected value {other:?}"),
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

#[test]
fn integer_sum_overflow_returns_error_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("UNWIND [9223372036854775807, 1] AS x RETURN sum(x)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("sum() integer overflow"));
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
fn aggregate_composed_with_arithmetic_computes_per_group() {
    // TCK clauses/return Return6 [2]/[9]: an aggregate doesn't need to be
    // a return item's *entire* top-level expression -- `count(n) + 3`,
    // `count(*) * 10` etc are real Cypher, evaluated once per group with
    // the aggregate's finished value substituted in.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");

    fn int_at(result: &marsdb_query::QueryResult) -> i64 {
        match &result.rows[0][0] {
            Value::Property(marsdb_graph::PropertyValue::Int(n)) => *n,
            other => panic!("unexpected value {other:?}"),
        }
    }

    let stmt = marsdb_query::parse("MATCH (n) RETURN 1 + count(n)").unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int_at(&result), 2);

    let stmt = marsdb_query::parse("MATCH () RETURN count(*) * 10 AS c").unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int_at(&result), 10);
}

#[test]
fn nested_aggregate_inside_another_aggregates_argument_is_rejected() {
    // TCK Return6 [14]: `count(count(*))` -- an aggregate's own argument
    // can't itself contain another aggregate (NestedAggregation), even
    // though composing an aggregate with *non*-aggregate arithmetic
    // (the test above) is fine.
    let store = GraphStore::open_memory().unwrap();
    let stmt = marsdb_query::parse("RETURN count(count(*))").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("another aggregate"),
        "unexpected error: {err}"
    );
}

#[test]
fn aggregate_composed_expression_requires_leaf_to_be_an_explicit_grouping_key() {
    // TCK Return6 [20]/[21]: once any item aggregates, a non-aggregate
    // leaf used alongside it must itself be listed as its own item --
    // just being in scope isn't enough (AmbiguousAggregationExpression).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person)-[:X]->(:Person)");
    let stmt =
        marsdb_query::parse("MATCH (me:Person)--(you:Person) RETURN me.age + count(you.age)")
            .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("grouping key"),
        "unexpected error: {err}"
    );

    // Same leaf, but now also listed as its own item -- legal.
    let stmt = marsdb_query::parse(
        "MATCH (me:Person)--(you:Person) RETURN me.age, me.age + count(you.age)",
    )
    .unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_ok());
}

/// A later comma-separated group can reference a variable from an
/// even-earlier group, not just the immediately-preceding one -- the
/// executor's existing already-bound-variable handling resolves it once
/// both clauses run in order, no special grouping logic needed.
#[test]
fn comma_separated_match_pattern_references_an_earlier_group() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:KNOWS]->(:B)");
    run(&store, "CREATE (:A)");
    run(&store, "CREATE (:B)");

    let result = run(
        &store,
        "MATCH (a:A), (b:B), (a)-[r:KNOWS]->(b) RETURN type(r)",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "KNOWS"),
        other => panic!("unexpected value {other:?}"),
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
    assert_eq!(
        int_value(&result.rows[0][0]),
        2,
        "count(f) must exclude the null-padded row"
    );
    assert_eq!(
        int_value(&result.rows[0][1]),
        5,
        "count(*) counts every row, including null-padded ones"
    );
}

#[test]
fn group_by_implicit_via_with() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Alice".into()),
    );
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    let mut bob_props = BTreeMap::new();
    bob_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Bob".into()),
    );
    let bob = store.create_node(&["Person"], bob_props).unwrap();
    for _ in 0..2 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store
            .create_edge("OWNS", alice, item, BTreeMap::new())
            .unwrap();
    }
    let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store
        .create_edge("OWNS", bob, item, BTreeMap::new())
        .unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) WITH p.name AS name, count(i) AS c RETURN name, c ORDER BY name",
    );
    assert_eq!(result.rows.len(), 2);
    assert_eq!(
        (str_value(&result.rows[0][0]), int_value(&result.rows[0][1])),
        ("Alice".to_string(), 2)
    );
    assert_eq!(
        (str_value(&result.rows[1][0]), int_value(&result.rows[1][1])),
        ("Bob".to_string(), 1)
    );
}

#[test]
fn group_by_implicit_via_return_no_with() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Alice".into()),
    );
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    for _ in 0..3 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store
            .create_edge("OWNS", alice, item, BTreeMap::new())
            .unwrap();
    }

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN p.name AS name, count(i) AS c",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(
        (str_value(&result.rows[0][0]), int_value(&result.rows[0][1])),
        ("Alice".to_string(), 3)
    );
}

#[test]
fn count_distinct_dedupes() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    for cat in ["A", "A", "B"] {
        let mut props = BTreeMap::new();
        props.insert(
            "category".to_string(),
            marsdb_graph::PropertyValue::String(cat.into()),
        );
        let item = store.create_node(&["Item"], props).unwrap();
        store
            .create_edge("OWNS", alice, item, BTreeMap::new())
            .unwrap();
    }

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN count(DISTINCT i.category) AS c",
    );
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
    store
        .create_edge("OWNS", alice, item, BTreeMap::new())
        .unwrap();
    store
        .create_edge("OWNS", alice, item, BTreeMap::new())
        .unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) RETURN collect(DISTINCT i) AS items",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::List(items) => assert_eq!(
            items.len(),
            1,
            "the same node reached via 2 edges must collect once"
        ),
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
    assert_eq!(
        result.rows.len(),
        1,
        "a global aggregate over zero rows must still emit one row"
    );
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
    let result = run(
        &store,
        "MATCH (n:NoSuchLabel) RETURN n.type AS t, count(n) AS c",
    );
    assert_eq!(
        result.rows.len(),
        0,
        "a grouping key present means zero groups over zero rows, not one"
    );
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
    alice_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Alice".into()),
    );
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    for _ in 0..2 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store
            .create_edge("OWNS", alice, item, BTreeMap::new())
            .unwrap();
    }
    let mut co_props = BTreeMap::new();
    co_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Acme".into()),
    );
    let acme = store.create_node(&["Company"], co_props).unwrap();
    store
        .create_edge("WORKS_AT", alice, acme, BTreeMap::new())
        .unwrap();

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
    let result = run(
        &store,
        "MATCH (n:Item) RETURN sum(n.val) AS s, avg(n.val) AS a",
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        6,
        "sum of all-int inputs must stay Int"
    );
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
    let result = run(
        &store,
        "MATCH (n:Item) RETURN sum(n.val) AS s, avg(n.val) AS a",
    );
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
fn nested_aggregate_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN count(sum(n.idx)) AS c").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("aggregate"),
        "expected an aggregate-nesting error, got: {err}"
    );
}

#[test]
fn aggregate_not_top_level_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt =
        parse("MATCH (n:Item) RETURN CASE n.idx WHEN 1 THEN count(n) ELSE 0 END AS x").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("aggregate"),
        "expected a top-level-aggregate error, got: {err}"
    );
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
        store
            .create_edge("NEXT", prev, next, BTreeMap::new())
            .unwrap();
        prev = next;
    }

    let stmt = parse("MATCH (n:Item {idx: 0})-[:NEXT*0..]->(m:Item) RETURN count(m) AS c").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("depth cap"),
        "expected a depth-cap error, got: {err}"
    );
}

#[test]
fn with_where_filters_on_aggregate_result() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut alice_props = BTreeMap::new();
    alice_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Alice".into()),
    );
    let alice = store.create_node(&["Person"], alice_props).unwrap();
    let mut bob_props = BTreeMap::new();
    bob_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Bob".into()),
    );
    let bob = store.create_node(&["Person"], bob_props).unwrap();
    for _ in 0..3 {
        let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        store
            .create_edge("OWNS", alice, item, BTreeMap::new())
            .unwrap();
    }
    let item = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store
        .create_edge("OWNS", bob, item, BTreeMap::new())
        .unwrap();

    let result = run(
        &store,
        "MATCH (p:Person)-[:OWNS]->(i:Item) WITH p, count(i) AS c WHERE c > 1 RETURN p.name AS name, c",
    );
    assert_eq!(
        result.rows.len(),
        1,
        "only Alice's group (count 3) should survive c > 1"
    );
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(int_value(&result.rows[0][1]), 3);
}

#[test]
fn with_where_filters_without_aggregation() {
    let store = GraphStore::open_memory().unwrap();
    for i in [5, 15, 25] {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 RETURN y ORDER BY y",
    );
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
        props.insert(
            "name".to_string(),
            marsdb_graph::PropertyValue::String(name.into()),
        );
        people.push(store.create_node(&["Person"], props).unwrap());
    }
    // Alice: 3 posts, Bob: 2 posts, Carol: 1 post, Dave: 0 posts.
    let post_counts = [3, 2, 1, 0];
    for (person, &n) in people.iter().zip(&post_counts) {
        for i in 0..n {
            let mut props = BTreeMap::new();
            props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(i));
            let post = store.create_node(&["Post"], props).unwrap();
            store
                .create_edge("HAS_CREATOR", post, *person, BTreeMap::new())
                .unwrap();
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
    assert_eq!(
        result.rows.len(),
        2,
        "Dave (0 posts) filtered by WHERE, then LIMIT 2 of the remaining 3"
    );
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
fn unwind_collected_nodes_restores_graph_identity() {
    // The whole point of value_to_binding_restore: a node that went into
    // collect() as a Value::Node must come back out of UNWIND as a real
    // Binding::Node, not just a display value -- provable by traversing
    // further (m.name) after the UNWIND, which only works with real graph
    // identity, not a frozen snapshot value.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(c:Person {name: 'Carol'})",
    );
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'})-[:KNOWS]->(f:Person) WITH collect(f) AS friends \
         UNWIND friends AS m RETURN m.name AS name ORDER BY name",
    );
    let names: Vec<String> = result.rows.iter().map(|r| str_value(&r[0])).collect();
    assert_eq!(names, vec!["Bob".to_string(), "Carol".to_string()]);
}

/// The node cache's reset (clear + enable/disable based on
/// `is_read_only`) must happen on *every* statement-execution entry
/// point, not just `execute`/`execute_with_options` -- `Executor` has a
/// second, separate entry point (`execute_in_write_transaction`, used by
/// an explicit multi-statement `Transaction` or a group-commit loop with
/// an already-open `WriteTransaction`), and `node_cache` is a field on
/// `Executor` shared by both, not private to either. A read via
/// `execute` leaves the cache populated and enabled; a write via
/// `execute_in_write_transaction` on the *same* `Executor` right after
/// must not inherit that state.
/// The Expand->Expand->count(*) fast path must produce byte-identical
/// results to the generic pipeline. Same collaborative-filtering query
/// run twice: once in the exact fast-path shape, once with a `WHERE`
/// predicate the recognizer doesn't accept (forcing the generic path) —
/// the two must agree on rows AND order. The fixture deliberately
/// includes the two semantic traps: a duplicate parallel edge (rec == m
/// must be counted when reached via a *different* edge) and the
/// single-edge user (whose only path back is the same edge, excluded by
/// edge isomorphism).
#[test]
fn fast_expand_count_matches_generic_path() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (m:Movie {title: 'Target'}), (x:Movie {title: 'X'}), (y:Movie {title: 'Y'}), \
         (u1:User {name: 'u1'}), (u2:User {name: 'u2'}), (u3:User {name: 'u3'})",
    );
    // u1 rates Target and X; u2 rates Target, X, Y; u3 rates ONLY Target
    // (isomorphism: contributes nothing). u2 also rates Target TWICE
    // (parallel edge): the second edge makes Target itself reachable as a
    // recommendation via a different edge — must be counted, not
    // special-cased away.
    for stmt in [
        "MATCH (u:User {name:'u1'}), (m:Movie {title:'Target'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u1'}), (m:Movie {title:'X'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'Target'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'Target'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'X'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'Y'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u3'}), (m:Movie {title:'Target'}) CREATE (u)-[:RATED]->(m)",
    ] {
        run(&store, stmt);
    }

    let fast = run(
        &store,
        "MATCH (m:Movie {title: 'Target'})<-[:RATED]-(u:User)-[:RATED]->(rec:Movie) \
         WITH rec, count(*) AS c ORDER BY c DESC LIMIT 5 RETURN rec.title, c",
    );
    // Same query with a recognizer-defeating (but semantically inert)
    // WHERE — routes through the generic pipeline.
    let generic = run(
        &store,
        "MATCH (m:Movie {title: 'Target'})<-[:RATED]-(u:User)-[:RATED]->(rec:Movie) \
         WHERE rec.title <> '\u{0}never' \
         WITH rec, count(*) AS c ORDER BY c DESC LIMIT 5 RETURN rec.title, c",
    );
    assert_eq!(
        format!("{:?}", fast.rows),
        format!("{:?}", generic.rows),
        "fast path diverged from the generic pipeline"
    );
    // And pin the absolute expectation so both being wrong together can't
    // slip through. X = 3 (once via u1, twice via u2's two Target edges);
    // Target = 2 (u2's parallel edges cross-count each other: e1->e2 and
    // e2->e1, both surviving the r1 != r2 isomorphism check); Y = 2 (via
    // each of u2's Target edges). u3 contributes nothing (its only path
    // back is its own edge, excluded by isomorphism).
    let flat: Vec<(String, i64)> = fast
        .rows
        .iter()
        .map(|r| {
            let title = match &r[0] {
                Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
                other => panic!("unexpected {other:?}"),
            };
            let c = match &r[1] {
                Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
                other => panic!("unexpected {other:?}"),
            };
            (title, c)
        })
        .collect();
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0], ("X".to_string(), 3));
    assert_eq!(flat[1].1, 2);
    assert_eq!(flat[2].1, 2);
}

/// 1-hop variant of the fast path (matrix_review_counts' shape): single
/// Expand + count(*) grouped by the expanded node. Same
/// fast-vs-generic-equivalence discipline as the 2-hop test.
#[test]
fn fast_single_expand_count_matches_generic_path() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Movie {title: 'A'}), (b:Movie {title: 'B'}), \
         (u1:User {name: 'u1'}), (u2:User {name: 'u2'})",
    );
    for stmt in [
        "MATCH (u:User {name:'u1'}), (m:Movie {title:'A'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'A'}) CREATE (u)-[:RATED]->(m)",
        "MATCH (u:User {name:'u2'}), (m:Movie {title:'B'}) CREATE (u)-[:RATED]->(m)",
    ] {
        run(&store, stmt);
    }
    let fast = run(
        &store,
        "MATCH (m:Movie)<-[:RATED]-(u:User) WITH m, count(*) AS reviews \
         ORDER BY reviews DESC LIMIT 5 RETURN m.title, reviews",
    );
    let generic = run(
        &store,
        "MATCH (m:Movie)<-[:RATED]-(u:User) WHERE u.name <> '\u{0}never' \
         WITH m, count(*) AS reviews ORDER BY reviews DESC LIMIT 5 RETURN m.title, reviews",
    );
    assert_eq!(format!("{:?}", fast.rows), format!("{:?}", generic.rows));
    assert_eq!(fast.rows.len(), 2);
    match (&fast.rows[0][1], &fast.rows[1][1]) {
        (
            Value::Property(marsdb_graph::PropertyValue::Int(a)),
            Value::Property(marsdb_graph::PropertyValue::Int(b)),
        ) => {
            assert_eq!((*a, *b), (2, 1));
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// collect(mid.prop) variant (inception_genre_similarity's shape):
/// 2-hop expansion grouped by the far node, collecting the MIDDLE node's
/// property alongside count(*). Pins collect's null-skipping (one genre
/// deliberately has no name) and in-group order equivalence.
#[test]
fn fast_expand_collect_matches_generic_path() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (m:Movie {title: 'Seed'}), (r1:Movie {title: 'R1'}), (r2:Movie {title: 'R2'}), \
         (g1:Genre {name: 'Action'}), (g2:Genre {name: 'Drama'}), (g3:Genre)",
    );
    for stmt in [
        "MATCH (m:Movie {title:'Seed'}), (g:Genre {name:'Action'}) CREATE (m)-[:IN_GENRE]->(g)",
        "MATCH (m:Movie {title:'Seed'}), (g:Genre {name:'Drama'}) CREATE (m)-[:IN_GENRE]->(g)",
        // The nameless genre: reachable, collected as null -> skipped.
        "MATCH (m:Movie {title:'Seed'}), (g:Genre) WHERE g.name IS NULL CREATE (m)-[:IN_GENRE]->(g)",
        "MATCH (r:Movie {title:'R1'}), (g:Genre {name:'Action'}) CREATE (r)-[:IN_GENRE]->(g)",
        "MATCH (r:Movie {title:'R1'}), (g:Genre {name:'Drama'}) CREATE (r)-[:IN_GENRE]->(g)",
        "MATCH (r:Movie {title:'R2'}), (g:Genre {name:'Drama'}) CREATE (r)-[:IN_GENRE]->(g)",
        "MATCH (r:Movie {title:'R2'}), (g:Genre) WHERE g.name IS NULL CREATE (r)-[:IN_GENRE]->(g)",
    ] {
        run(&store, stmt);
    }
    let q_fast = "MATCH (m:Movie)-[:IN_GENRE]->(g:Genre)<-[:IN_GENRE]-(rec:Movie) \
         WHERE m.title = 'Seed' \
         WITH rec, collect(g.name) AS genres, count(*) AS commonGenres \
         RETURN rec.title, genres, commonGenres ORDER BY commonGenres DESC";
    // Recognizer-defeating variant: an extra inert predicate on rec.
    let q_generic = "MATCH (m:Movie)-[:IN_GENRE]->(g:Genre)<-[:IN_GENRE]-(rec:Movie) \
         WHERE m.title = 'Seed' AND rec.title <> '\u{0}never' \
         WITH rec, collect(g.name) AS genres, count(*) AS commonGenres \
         RETURN rec.title, genres, commonGenres ORDER BY commonGenres DESC";
    let fast = run(&store, q_fast);
    let generic = run(&store, q_generic);
    assert_eq!(
        format!("{:?}", fast.rows),
        format!("{:?}", generic.rows),
        "collect fast path diverged from the generic pipeline"
    );
    // R1 shares Action+Drama (2 paths, 2 names); R2 shares Drama + the
    // nameless genre (2 paths, but only 1 collected name -- null skipped).
    assert_eq!(fast.rows.len(), 2);
}

#[test]
fn list_index_negative_counts_from_end() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3, 4, 5] AS list RETURN list[-1]");
    assert_eq!(int(&result.rows[0][0]), 5);
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
fn quantifier_single_counts_exact_matches() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN single(x IN [1, 2, 3] WHERE x = 2) AS a, single(x IN [1, 2, 2] WHERE x = 2) AS b",
    );
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

/// `count(rand())` -- an aggregate's argument must be deterministic per
/// row for grouping to have well-defined semantics, which `rand()`
/// fundamentally breaks. Real Cypher rejects this at compile time (TCK's
/// Return6 [15]).
#[test]
fn rand_inside_an_aggregate_argument_is_rejected() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN count(rand())").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("rand() as an aggregate argument must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("non-deterministic"), "unexpected error: {msg}");

    // rand() elsewhere (not inside an aggregate call) is completely fine.
    let result = run(&store, "RETURN rand() < 2.0 AS x");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn limit_on_a_non_unique_index_seek_stops_at_the_right_count() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {city: 'Tokyo', name: 'a'})");
    run(&store, "CREATE (:Person {city: 'Tokyo', name: 'b'})");
    run(&store, "CREATE (:Person {city: 'Tokyo', name: 'c'})");
    run(&store, "CREATE (:Person {city: 'Osaka', name: 'd'})");
    run(&store, "CREATE INDEX ON :Person(city)");

    let result = run(
        &store,
        "MATCH (n:Person {city: 'Tokyo'}) RETURN n.name LIMIT 2",
    );
    assert_eq!(result.rows.len(), 2);
}

/// `MATCH (a) WHERE count(a) > 10` -- an aggregate function is never
/// legal inside a pattern-level WHERE, a real compile-time error (not
/// something a zero-row MATCH could silently skip checking). TCK's
/// MatchWhere1 [15].
#[test]
fn aggregate_in_pattern_where_is_a_compile_time_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (a) WHERE count(a) > 10 RETURN a").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("aggregate"));
}
