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

/// Real Cypher's two comment forms (`//` line, `/* */` block) -- a real
/// grammar gap found via the openCypher TCK's own fixture text, which
/// pervasively annotates `CREATE` blocks this way.
#[test]
fn line_and_block_comments_are_ignored() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:A {num: 1}), //first node\n(:A {num: 2}) // second node",
    );
    let result = run(
        &store,
        "/* leading */ MATCH (a:A) // trailing\nRETURN a.num /* mid-expr */ + 0 ORDER BY a.num",
    );
    let nums: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(nums, vec![1, 2]);
}

/// Real Cypher allows chaining multiple `CREATE` clauses in one
/// statement, each seeing bindings from every earlier one -- a repeated
/// `CREATE` keyword is just another pattern separator, same as `,`. A
/// real gap found via the openCypher TCK's own fixture convention
/// (`CREATE (a {..}), (b {..})\nCREATE (a)-[:T]->(b)`, common enough to
/// cause real, previously-misdiagnosed wrong-result bugs when a test
/// harness naively ran each line as an independent statement instead,
/// losing the shared `a`/`b` bindings and creating disconnected nodes).
#[test]
fn chained_create_clauses_share_scope() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a {name: 'a'}), (b {name: 'b'})\nCREATE (a)-[:T]->(b)",
    );
    let result = run(&store, "MATCH p = ({name: 'a'})-->({name: 'b'}) RETURN p");
    assert_eq!(result.rows.len(), 1);

    // Three-deep chaining, not just two.
    let store2 = GraphStore::open_memory().unwrap();
    run(
        &store2,
        "CREATE (a {n: 1})\nCREATE (b {n: 2})\nCREATE (a)-[:T]->(b)",
    );
    let result2 = run(&store2, "MATCH ({n: 1})-->({n: 2}) RETURN 1");
    assert_eq!(result2.rows.len(), 1);
}

#[test]
fn traversal_with_label_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:BLOCKS]->(c:Person {name: 'Carol'})",
    );

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
fn pattern_predicate_outside_where_is_a_compile_time_error() {
    // TCK's List6 [6] "Fail for size() on pattern predicates": must
    // reject at compile time, regardless of whether any row ever reaches
    // evaluation -- an empty MATCH (zero rows) must still error, not
    // silently succeed with no output.
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (a), (b), (c) RETURN size((a)-->())").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_err());
}

#[test]
fn exists_simple_subquery_with_and_without_inline_where() {
    // TCK expressions/existentialSubqueries ExistentialSubquery1 [1]/[2]:
    // `exists { <pattern> }` / `exists { <pattern> WHERE ... }` -- the
    // "simple" form, same existential search as a bare pattern predicate
    // (`WHERE (n)-->()`) but able to introduce a new variable (`m`) and
    // carry its own inline `WHERE`.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A {prop: 1})-[:R]->(:B {prop: 1}), (a)-[:R]->(:C {prop: 2})",
    );

    let result = run(&store, "MATCH (n) WHERE exists { (n)-->() } RETURN n.prop");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);

    let result = run(
        &store,
        "MATCH (n) WHERE exists { (n)-->(m) WHERE n.prop = m.prop } RETURN n.prop",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);

    // A relationship type/inline WHERE that never matches -- no rows.
    let result = run(&store, "MATCH (n) WHERE exists { (n)-[:NA]->() } RETURN n");
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn exists_full_subquery_form_is_not_supported_yet() {
    // TCK ExistentialSubquery2 [1]: `exists { MATCH ... RETURN ... }` --
    // a full nested query, not the simple pattern-with-where form.
    // Deliberately out of scope (would need running an arbitrary
    // correlated nested Statement) -- a clear compile-time rejection,
    // not a panic or a silently wrong answer.
    let stmt = parse("MATCH (n) WHERE exists { MATCH (n)-->() RETURN true } RETURN n");
    assert!(stmt.is_err());
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
    assert_eq!(
        run(&store, "MATCH (n:Item) RETURN n.idx LIMIT 0")
            .rows
            .len(),
        0
    );
    assert_eq!(
        run(&store, "MATCH (n:Item) RETURN n.idx LIMIT 100")
            .rows
            .len(),
        3
    );
    assert_eq!(run(&store, "MATCH (n) RETURN n LIMIT 2").rows.len(), 2);
}

/// `WHERE a:A` -- a user-typed label predicate directly in pattern-level
/// `WHERE`, not just the planner-synthesized form multi-label node
/// patterns already used internally.
/// Real Cypher's `WITH x AS y WHERE ...` sees *both* the pre-WITH
/// binding (`x`) and the new alias (`y`) -- the WHERE isn't scoped to
/// only the projected/aliased row the way a later clause is.
#[test]
fn with_where_sees_both_pre_and_post_with_bindings() {
    let store = GraphStore::open_memory().unwrap();
    for name in ["A", "B", "C"] {
        run(&store, &format!("CREATE (:N {{name2: '{name}'}})"));
    }

    let result = run(
        &store,
        "MATCH (a) WITH a.name2 AS name WHERE a.name2 = 'B' RETURN name",
    );
    assert_eq!(result.rows.len(), 1);

    let result = run(
        &store,
        "MATCH (a) WITH a.name2 AS name WHERE name = 'B' RETURN name",
    );
    assert_eq!(result.rows.len(), 1);

    let result = run(
        &store,
        "MATCH (a) WITH a.name2 AS name WHERE name = 'B' OR a.name2 = 'C' RETURN name",
    );
    assert_eq!(result.rows.len(), 2);
}

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

/// `WITH ... WHERE x IS NULL` on a whole bound var (not just a
/// property), the common "OPTIONAL MATCH missed" check.
#[test]
fn with_where_is_null_checks_a_whole_bound_variable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'x'})");
    let result = run(
        &store,
        "MATCH (a:A) OPTIONAL MATCH (a)-[r:REL]->(x) WITH r WHERE r IS NULL RETURN count(*) AS c",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 1),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `WITH ... WHERE x IS NOT NULL` -- the `Not(IsNull(..))` desugaring.
#[test]
fn with_where_is_not_null() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'x'})");
    let result = run(
        &store,
        "MATCH (a:A) WITH a WHERE a IS NOT NULL RETURN a.name",
    );
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn where_label_predicate_filters_by_label() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A:B {id: 1})");
    run(&store, "CREATE (:A {id: 2})");
    run(&store, "CREATE (:B {id: 3})");

    let result = run(&store, "MATCH (n) WHERE n:A RETURN n.id");
    assert_eq!(result.rows.len(), 2);

    let result = run(&store, "MATCH (n) WHERE n:A:B RETURN n.id");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 1),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `WHERE a.prop op b.prop` -- a property compared against another
/// property, not a constant (`Expr::PropCompare`, never eligible for
/// the planner's index-seek fusion).
#[test]
fn where_prop_compare_filters_by_another_variables_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {val: 5})-[:E]->(:Y {val: 10})");
    run(&store, "CREATE (:X {val: 20})-[:E]->(:Y {val: 1})");

    let result = run(
        &store,
        "MATCH (x:X)-[:E]->(y:Y) WHERE x.val < y.val RETURN x.val, y.val",
    );
    assert_eq!(result.rows.len(), 1);
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Property(marsdb_graph::PropertyValue::Int(x)),
            Value::Property(marsdb_graph::PropertyValue::Int(y)),
        ) => {
            assert_eq!(*x, 5);
            assert_eq!(*y, 10);
        }
        other => panic!("unexpected value {other:?}"),
    }
}

/// `WHERE a = b` / `WHERE a <> b` -- node/relationship identity
/// comparison (`Expr::VarEq`/`Not(VarEq)`), distinct from comparing two
/// of their properties.
#[test]
fn where_var_compare_checks_node_identity() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {id: 1})-[:E]->(:N {id: 2})");

    let result = run(&store, "MATCH (a)-[:E]->(b) WHERE a = a RETURN a.id");
    assert_eq!(result.rows.len(), 1);

    let result = run(&store, "MATCH (a)-[:E]->(b) WHERE a <> b RETURN a.id, b.id");
    assert_eq!(result.rows.len(), 1);
}

/// Only `=`/`<>` are meaningful for node/relationship identity -- `<`
/// etc. have no defined ordering between two nodes, must be a real
/// error, not a silent `false`/panic.
#[test]
fn where_var_compare_with_ordering_operator_is_a_syntax_error() {
    let err = marsdb_query::parse("MATCH (a)-[:E]->(b) WHERE a < b RETURN a")
        .expect_err("ordering a node identity comparison must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("only = and <>"), "unexpected error: {msg}");
}

/// MATCH's own bare `WHERE` widened to the same general-expression power
/// `WITH`'s `WHERE` already had (`Expr::GeneralCompare`) -- a function
/// call operand, not just `prop_access op (prop_access | literal)`.
#[test]
fn where_general_comparison_allows_a_function_call_operand() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {id: '1'})");
    run(&store, "CREATE (:N {id: '2'})");

    let result = run(&store, "MATCH (n) WHERE toInteger(n.id) = 1 RETURN n.id");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "1"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Same widening, but the comparison's LHS is an edge-typed builtin
/// (`type(r)`), a shape the old `comparison = prop_access ~ ...` rule
/// couldn't parse at all.
#[test]
fn where_general_comparison_allows_type_of_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)-[:KNOWS]->(:N)");
    run(&store, "CREATE (:N)-[:LIKES]->(:N)");

    let result = run(
        &store,
        "MATCH ()-[r]->() WHERE type(r) = 'KNOWS' RETURN type(r)",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "KNOWS"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `WHERE n.val + 0 IS NULL` -- an arithmetic operand, not just
/// `prop_access IS NULL` (`Expr::GeneralIsNull`, mirrors
/// `WithExpr::IsNull`); a missing property propagates `Null` through `+`,
/// same null-propagation arithmetic already has everywhere else.
#[test]
fn where_general_is_null_checks_an_arithmetic_expression() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {name: 'a', val: 5})");
    run(&store, "CREATE (:N {name: 'b'})");

    let result = run(&store, "MATCH (n) WHERE n.val + 0 IS NULL RETURN n.name");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "b"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// MATCH's own bare `WHERE` accepts a boolean-valued expression directly
/// as the predicate, no comparison operator at all (`Expr::GeneralBare`)
/// -- `WHERE n.flag`, `WHERE NOT n.flag`, distinct from `WHERE n.flag =
/// true`.
#[test]
fn where_general_bare_accepts_a_boolean_property_directly() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {name: 'a', flag: true})");
    run(&store, "CREATE (:N {name: 'b', flag: false})");

    let result = run(&store, "MATCH (n) WHERE n.flag RETURN n.name");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "a"),
        other => panic!("unexpected value {other:?}"),
    }

    let result = run(&store, "MATCH (n) WHERE NOT n.flag RETURN n.name");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "b"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Same widening for `WITH ... WHERE` (`WithExpr::Bare`) -- a bare
/// quantifier expression combined via `OR`, no comparison operator
/// wrapping either side. Exact shape the TCK's `Quantifier11 [3]`
/// scenario needs (`WHERE single(...) OR all(...)`).
#[test]
fn with_where_bare_accepts_quantifier_expressions_combined_with_or() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(
        &store,
        "WITH [1, 2, 2] AS list \
         WHERE single(x IN list WHERE x = 1) OR all(x IN list WHERE x = 2) \
         RETURN list",
    );
    assert_eq!(result.rows.len(), 1);

    let result = run(
        &store,
        "WITH [2, 2, 2] AS list \
         WHERE single(x IN list WHERE x = 1) OR all(x IN list WHERE x = 1) \
         RETURN list",
    );
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn skip_alone_drops_the_first_n_rows_after_order_by() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS v ORDER BY v SKIP 2");
    let vals: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(vals, vec![2, 3, 4]);
}

/// Real Cypher always applies SKIP before LIMIT, regardless of clause
/// order in the query text -- `SKIP 1 LIMIT 2` over `[0,1,2,3,4]` must
/// yield `[1,2]`, not the first two rows then skip one of those.
#[test]
fn skip_and_limit_together_skip_applies_before_limit() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS v ORDER BY v SKIP 1 LIMIT 2",
    );
    let vals: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(vals, vec![1, 2]);
}

/// SKIP with no ORDER BY at all -- covers the pre-truncate path
/// (`execute_match`'s non-ORDER-BY, non-DISTINCT branch), not the
/// `top_k_by`/ORDER BY path the other SKIP tests exercise.
#[test]
fn skip_without_order_by_drops_rows_from_the_scan_order() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS v SKIP 3");
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn skip_zero_is_a_no_op() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(&store, "MATCH (n:Item) RETURN n.idx AS v ORDER BY v SKIP 0");
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn skip_past_the_end_of_the_result_set_returns_empty_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS v ORDER BY v SKIP 100",
    );
    assert_eq!(result.rows.len(), 0);
}

/// SKIP is negative -- must be a real error, not silently clamped to 0 or
/// treated as unbounded. SKIP/LIMIT accept any expression (`SKIP $n`,
/// `SKIP toInteger(rand()*9)` -- TCK's `ReturnSkipLimit1 [2]`/`[3]`), so
/// this is only checkable once at execution time, not at parse time.
#[test]
fn skip_negative_is_a_syntax_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AS x SKIP -1").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("negative SKIP must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("SKIP"), "unexpected error: {msg}");
}

/// SKIP/LIMIT accept `$param` and arbitrary constant expressions (not
/// just a literal integer) -- TCK's `ReturnSkipLimit1 [2]`/`[3]`,
/// `ReturnSkipLimit2 [6]`.
#[test]
fn skip_limit_accept_params_and_constant_expressions() {
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let mut stmt =
        parse("MATCH (n:Item) RETURN n.idx AS idx ORDER BY idx SKIP $s LIMIT $l").unwrap();
    let mut params = HashMap::new();
    params.insert("s".to_string(), marsdb_graph::PropertyValue::Int(2));
    params.insert("l".to_string(), marsdb_graph::PropertyValue::Int(2));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(result.rows.len(), 2);

    let stmt =
        parse("MATCH (n:Item) RETURN n.idx AS idx ORDER BY idx LIMIT toInteger(2.9)").unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(result.rows.len(), 2);
}

/// SKIP on a WITH clause -- separate code path from RETURN's own SKIP
/// (`apply_with_or_carry`), needs its own coverage.
#[test]
fn skip_on_with_clause_paginates_before_the_next_match() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) WITH n.idx AS v ORDER BY v SKIP 1 LIMIT 2 RETURN v",
    );
    let vals: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(vals, vec![1, 2]);
}

#[test]
fn return_distinct_dedups_whole_row() {
    let store = GraphStore::open_memory().unwrap();
    for city in ["A", "B", "A", "A", "B"] {
        run(&store, &format!("CREATE (n:City {{name: '{city}'}})"));
    }
    let result = run(
        &store,
        "MATCH (n:City) RETURN DISTINCT n.name AS c ORDER BY c",
    );
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
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
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
    run(
        &store,
        "CREATE (a:Post {content: 'hello', imageFile: 'ignored.png'})",
    );
    run(&store, "CREATE (b:Post {imageFile: 'pic.png'})"); // no content prop

    let result = run(
        &store,
        "MATCH (n:Post) RETURN coalesce(n.content, n.imageFile) AS x",
    );
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
    let result = run(
        &store,
        "WITH [2, 2.9, '1.7'] AS things RETURN [n IN things | toInteger(n)] AS x",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 2, 1]);
}

#[test]
fn to_integer_on_an_unparseable_string_is_null_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH ['2', '2.9', 'foo'] AS numbers RETURN [n IN numbers | toInteger(n)] AS x",
    );
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
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE n.age WHEN 30 THEN 'thirty' ELSE 'other' END AS x",
    );
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
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE n.age WHEN null THEN 'yes' ELSE 'no' END AS x",
    );
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "yes"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Real Cypher's "searched CASE" form -- no subject expression, each
/// `WHEN` carries its own full boolean condition (`CASE WHEN cond THEN
/// ... END`), distinct from the "simple CASE" form both other `case_*`
/// tests exercise (`CASE x WHEN v THEN ... END`). A bare `WHEN` right
/// after `CASE` used to get swallowed as a bare-identifier subject
/// expression (pest's `?` doesn't backtrack across a later parse
/// failure), rejecting every searched-CASE query.
#[test]
fn case_searched_form_has_no_subject_expression() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {age: 30})");
    run(&store, "CREATE (b:Person {age: 17})");
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE WHEN n.age >= 18 THEN 'adult' ELSE 'minor' END AS x",
    );
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["adult".to_string(), "minor".to_string()]);
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
    assert_eq!(
        names,
        vec![
            "Alice".to_string(),
            "Charlie".to_string(),
            "Bob".to_string()
        ]
    );
}

#[test]
fn order_by_sorts_distinct_types_in_defined_cross_type_order() {
    // TCK clauses/return-orderby ReturnOrderBy1 [11]: distinct types have a
    // total order for ORDER BY, Map < Node < Relationship < List < Path <
    // String < Boolean < Number < Null.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)-[:REL]->()");
    let result = run(
        &store,
        "MATCH p = (n:N)-[r:REL]->() \
         UNWIND [n, r, p, 1.5, ['list'], 'text', null, false, 0.0 / 0.0, {a: 'map'}] AS types \
         RETURN types ORDER BY types",
    );
    let kinds: Vec<&str> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Map(_) => "map",
            Value::Node(_) => "node",
            Value::Edge(_) => "edge",
            Value::List(_) => "list",
            Value::Path(_) => "path",
            Value::Literal(marsdb_query::Literal::String(_))
            | Value::Property(marsdb_graph::PropertyValue::String(_)) => "string",
            Value::Literal(marsdb_query::Literal::Bool(_))
            | Value::Property(marsdb_graph::PropertyValue::Bool(_)) => "bool",
            Value::Literal(marsdb_query::Literal::Float(f))
            | Value::Property(marsdb_graph::PropertyValue::Float(f))
                if f.is_nan() =>
            {
                "nan"
            }
            Value::Literal(marsdb_query::Literal::Float(_))
            | Value::Literal(marsdb_query::Literal::Int(_))
            | Value::Property(marsdb_graph::PropertyValue::Float(_))
            | Value::Property(marsdb_graph::PropertyValue::Int(_)) => "number",
            Value::Literal(marsdb_query::Literal::Null)
            | Value::Property(marsdb_graph::PropertyValue::Null)
            | Value::Null => "null",
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["map", "node", "edge", "list", "path", "string", "bool", "number", "nan", "null"]
    );
}

#[test]
fn order_by_desc_reverses_the_whole_cross_type_order_null_and_nan_included() {
    // TCK clauses/return-orderby ReturnOrderBy1 [12]: DESC is a genuine
    // reversal of the whole total order, not just of non-null/non-NaN
    // comparisons -- `null` sorts *first* under DESC (not last
    // regardless of direction), and `NaN` (the largest number) sorts
    // right after it, ahead of every finite float.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [1.5, null, 0.0 / 0.0, false, 'text'] AS types \
         RETURN types ORDER BY types DESC",
    );
    let kinds: Vec<&str> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::Null)
            | Value::Property(marsdb_graph::PropertyValue::Null)
            | Value::Null => "null",
            Value::Literal(marsdb_query::Literal::Float(f))
            | Value::Property(marsdb_graph::PropertyValue::Float(f))
                if f.is_nan() =>
            {
                "nan"
            }
            Value::Literal(marsdb_query::Literal::Float(_))
            | Value::Literal(marsdb_query::Literal::Int(_))
            | Value::Property(marsdb_graph::PropertyValue::Float(_))
            | Value::Property(marsdb_graph::PropertyValue::Int(_)) => "number",
            Value::Literal(marsdb_query::Literal::Bool(_))
            | Value::Property(marsdb_graph::PropertyValue::Bool(_)) => "bool",
            Value::Literal(marsdb_query::Literal::String(_))
            | Value::Property(marsdb_graph::PropertyValue::String(_)) => "string",
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(kinds, vec!["null", "nan", "number", "bool", "string"]);
}

#[test]
fn pattern_comprehension_introduces_new_node_and_rel_vars() {
    // TCK expressions/pattern Pattern2 [4]/[5]: a pattern comprehension
    // can introduce brand-new node/relationship variables, unlike a
    // pattern predicate.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a {name: 'a'}), (b {name: 'val'}), (c)");
    run(
        &store,
        "MATCH (a), (b) WHERE a.name = 'a' AND b.name = 'val' CREATE (a)-[:T]->(b)",
    );
    run(
        &store,
        "MATCH (b {name: 'val'}), (c) WHERE c.name IS NULL CREATE (b)-[:T]->(c)",
    );

    let result = run(&store, "MATCH (n) RETURN [(n)-[:T]->(b) | b.name] AS list");
    let mut lists: Vec<Vec<Option<String>>> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::List(items) => items
                .iter()
                .map(|v| match v {
                    Value::Property(marsdb_graph::PropertyValue::String(s)) => Some(s.clone()),
                    Value::Null => None,
                    other => panic!("unexpected value {other:?}"),
                })
                .collect(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    lists.sort();
    assert_eq!(
        lists,
        vec![vec![], vec![None], vec![Some("val".to_string())]]
    );
}

#[test]
fn pattern_comprehension_with_named_path_and_where() {
    // TCK expressions/pattern Pattern2 [2]/[3]: named-path capture
    // (`p = ...`) plus a label predicate on the pattern's own end node.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A), (b:B), (c:C)");
    run(&store, "MATCH (a:A), (b:B) CREATE (a)-[:T]->(b)");
    run(&store, "MATCH (a:A), (c:C) CREATE (a)-[:T]->(c)");

    let result = run(&store, "MATCH (n:A) RETURN [p = (n)-->(:B) | p] AS list");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Path(_)));
        }
        other => panic!("unexpected value {other:?}"),
    }
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

#[test]
fn variable_length_hop_cannot_reuse_an_earlier_fixed_hops_edge() {
    // TCK clauses/match Match5 [27]: real Cypher's edge-isomorphism rule
    // (no relationship repeated within one MATCH pattern) applies across a
    // whole pattern, not just within one variable-length hop's own BFS --
    // a var-length hop mustn't walk back over an edge an earlier fixed hop
    // in the *same* pattern already used.
    //
    // A -[:R]-> B <-[:R]- C (both edges point into B). `(a:A)-[:R]->(b)`
    // must use the A-B edge; a subsequent `<-[:R*1]->` from `b` can then
    // only reach `C` via the B-C edge -- walking back to `A` would replay
    // the already-used A-B edge, and (with only one real path to `C`)
    // reusing it is the *only* way to get a second row.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'A'})-[:R]->(:B {name: 'B'})");
    run(&store, "MATCH (b:B) CREATE (:C {name: 'C'})-[:R]->(b)");
    let result = run(&store, "MATCH (a:A)-[:R]->(b)<-[:R*1]->(c) RETURN c.name");
    let names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["C".to_string()]);
}

#[test]
fn order_by_then_limit_sorts_before_truncating() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..5 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS x ORDER BY x DESC LIMIT 2",
    );
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
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS x ORDER BY x LIMIT 0",
    );
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

#[test]
fn integer_arithmetic_overflow_returns_errors_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    for cypher in [
        "RETURN 9223372036854775807 + 1",
        "RETURN -9223372036854775808 - 1",
        "RETURN 9223372036854775807 * 2",
        "RETURN -9223372036854775808 / -1",
        "RETURN -9223372036854775808 % -1",
    ] {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(err.to_string().contains("overflow"), "{cypher}: {err}");
    }
}

#[test]
fn integer_sum_overflow_returns_error_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("UNWIND [9223372036854775807, 1] AS x RETURN sum(x)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("sum() integer overflow"));
}

#[test]
fn execution_options_enforce_rows_expansions_cancellation_and_timeout() {
    use std::time::Duration;

    use marsdb_query::{CancellationToken, ExecutionOptions, QueryError};

    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Item {id: 1})-[:NEXT]->(:Item {id: 2})-[:NEXT]->(:Item {id: 3})",
    );
    let executor = Executor::new(&store);

    let scan = parse("MATCH (n:Item) RETURN n").unwrap();
    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                max_intermediate_rows: Some(2),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                max_intermediate_rows: Some(10),
                max_result_rows: Some(2),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let expand = parse("MATCH (n:Item)-[:NEXT]->(m:Item) RETURN m").unwrap();
    let err = executor
        .execute_with_options(
            &expand,
            &ExecutionOptions {
                max_relationship_expansions: Some(1),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let token = CancellationToken::new();
    token.cancel();
    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                cancellation_token: Some(token),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::Cancelled));

    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                timeout: Some(Duration::ZERO),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::Timeout));
}

#[test]
fn streaming_limit_stops_expansion_but_blocking_order_by_consumes_input() {
    use std::collections::BTreeMap;

    use marsdb_graph::PropertyValue;
    use marsdb_query::{ExecutionOptions, QueryError};

    let store = GraphStore::open_memory().unwrap();
    let hub = store.create_node(&["Hub"], BTreeMap::new()).unwrap();
    for id in 0..50 {
        let leaf = store
            .create_node(
                &["Leaf"],
                BTreeMap::from([("id".to_string(), PropertyValue::Int(id))]),
            )
            .unwrap();
        store.create_edge("R", hub, leaf, BTreeMap::new()).unwrap();
    }
    let executor = Executor::new(&store);

    // The pull pipeline requests just one expanded row, so the second
    // relationship is never touched.
    let limited = parse("MATCH (:Hub)-[:R]->(b) RETURN b LIMIT 1").unwrap();
    let result = executor
        .execute_with_options(
            &limited,
            &ExecutionOptions {
                max_relationship_expansions: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    // ORDER BY is a blocking operator and must inspect all 50 candidates
    // before it can know which row belongs first.
    let ordered = parse("MATCH (:Hub)-[:R]->(b) RETURN b.id ORDER BY b.id LIMIT 1").unwrap();
    let error = executor
        .execute_with_options(
            &ordered,
            &ExecutionOptions {
                max_relationship_expansions: Some(1),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(error, QueryError::ResourceLimit(_)));

    // LIMIT 0 never polls the expansion operator at all.
    let zero = parse("MATCH (:Hub)-[:R]->(b) RETURN b LIMIT 0").unwrap();
    let result = executor
        .execute_with_options(
            &zero,
            &ExecutionOptions {
                max_relationship_expansions: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(result.rows.is_empty());
}

#[test]
fn semantic_validation_rejects_invalid_names_and_structural_types() {
    let store = GraphStore::open_memory().unwrap();
    for (cypher, expected) in [
        ("RETURN missing", "undefined variable 'missing'"),
        (
            "WITH 1 AS x MATCH (x)-[:R]->(n) RETURN n",
            "node pattern requires a node",
        ),
        (
            "MATCH ()-[r:R]->() SET r:Label",
            "SET label target requires a node",
        ),
        (
            "MATCH (n) WITH n AS kept RETURN n",
            "undefined variable 'n'",
        ),
        (
            "MATCH (n) WITH n AS x, n AS x RETURN x",
            "duplicate variable 'x'",
        ),
    ] {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().contains("semantic error") && err.to_string().contains(expected),
            "{cypher}: expected {expected:?}, got {err}"
        );
    }
}

#[test]
fn match_create_binds_created_relationship_for_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Root {id: 1})");
    let result = run(
        &store,
        "MATCH (a:Root {id: 1}) CREATE (a)-[r:LINK]->(b:Leaf) RETURN r",
    );
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(&result.rows[0][0], Value::Edge(edge) if edge.label == "LINK"));
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

#[test]
fn undirected_pattern_matches_either_direction() {
    let store = GraphStore::open_memory().unwrap();
    // a->b (created via Right direction), so from b's perspective it's an
    // incoming edge — an undirected MATCH from b must still find a.
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );

    let from_a = run(
        &store,
        "MATCH (n:Person {name: 'Alice'})-[:KNOWS]-(friend) RETURN friend.name",
    );
    assert_eq!(from_a.rows.len(), 1);

    let from_b = run(
        &store,
        "MATCH (n:Person {name: 'Bob'})-[:KNOWS]-(friend) RETURN friend.name",
    );
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
    store
        .create_edge("KNOWS", alice, alice, BTreeMap::new())
        .unwrap();

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
    run(
        &store,
        "CREATE (a:Root {name: 'r'})-[:R]->(b:Post {name: 'post'})",
    );
    run(
        &store,
        "CREATE (a2:Root {name: 'r2'})-[:R]->(c:Comment {name: 'comment'})",
    );

    let result = run(&store, "MATCH (a:Root)-[:R]->(b:Post) RETURN b.name");
    assert_eq!(
        result.rows.len(),
        1,
        "hop node's label filter must exclude the :Comment target"
    );
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
    let c1 = store
        .create_node(&["Comment", "Message"], props_c1)
        .unwrap();
    let mut props_c2 = BTreeMap::new();
    props_c2.insert("id".to_string(), marsdb_graph::PropertyValue::Int(3));
    let c2 = store
        .create_node(&["Comment", "Message"], props_c2)
        .unwrap();
    store
        .create_edge("REPLY_OF", c1, p, BTreeMap::new())
        .unwrap();
    store
        .create_edge("REPLY_OF", c2, c1, BTreeMap::new())
        .unwrap();

    let result = run(
        &store,
        "MATCH (m:Message {id: 3})-[:REPLY_OF*0..]->(p:Post) RETURN p.id",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 1),
        other => panic!("unexpected value {other:?}"),
    }

    // min_hops = 0 also includes the start node itself if it happens to
    // match the target label — not exercised by IS6 (a Comment never
    // has :Post too) but worth confirming: starting FROM the post with
    // *0.. must return the post itself at hop 0.
    let from_post = run(
        &store,
        "MATCH (m:Message {id: 1})-[:REPLY_OF*0..]->(p:Post) RETURN p.id",
    );
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
        store
            .create_edge("NEXT", ids[i], ids[i + 1], BTreeMap::new())
            .unwrap();
    }

    // From idx=0, *1..2 should reach idx=1 and idx=2 only (not 3, not 4;
    // not 0 itself since min_hops=1).
    let result = run(
        &store,
        "MATCH (n:Item {idx: 0})-[:NEXT*1..2]->(m:Item) RETURN m.idx",
    );
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

/// A bare `[*]` (no explicit bounds) defaults to `min_hops = 1`, not 0 --
/// the destination is never the start node itself, same as `[*1..]`
/// already correctly behaved. A real bug found via the TCK: `parse_
/// rel_range` defaulted the *omitted* min (both the fully bare `*` case
/// and the `*..M` case with an empty min before `..`) to 0, incorrectly
/// including the zero-hop "reached myself" row. TCK's Match4 [2].
#[test]
fn variable_length_bare_star_defaults_to_min_hops_one() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    for i in 0..4 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        ids.push(store.create_node(&["Item"], props).unwrap());
    }
    for i in 0..3 {
        store
            .create_edge("NEXT", ids[i], ids[i + 1], BTreeMap::new())
            .unwrap();
    }
    for query in [
        "MATCH (n:Item {idx: 0})-[:NEXT*]->(m:Item) RETURN m.idx",
        "MATCH (n:Item {idx: 0})-[:NEXT*..3]->(m:Item) RETURN m.idx",
    ] {
        let result = run(&store, query);
        let mut reached: Vec<i64> = result
            .rows
            .iter()
            .map(|row| match &row[0] {
                Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
                other => panic!("unexpected value {other:?}"),
            })
            .collect();
        reached.sort();
        assert_eq!(reached, vec![1, 2, 3], "query: {query}");
    }

    // `*0..` (explicit zero lower bound) still legitimately includes the
    // start node itself -- only the *omitted*-min cases default to 1.
    let result = run(
        &store,
        "MATCH (n:Item {idx: 0})-[:NEXT*0..2]->(m:Item) RETURN m.idx",
    );
    let mut reached: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    reached.sort();
    assert_eq!(reached, vec![0, 1, 2]);
}

#[test]
fn variable_length_preserves_distinct_paths_to_same_node() {
    use std::collections::BTreeMap;

    // Diamond: a -> b -> d and a -> c -> d are distinct paths and must
    // therefore produce two rows even though they share the same endpoint.
    let store = GraphStore::open_memory().unwrap();
    let mut nodes = Vec::new();
    for name in ["a", "b", "c", "d"] {
        let mut props = BTreeMap::new();
        props.insert(
            "name".to_string(),
            marsdb_graph::PropertyValue::String(name.to_string()),
        );
        nodes.push(store.create_node(&["Item"], props).unwrap());
    }
    for (src, dst) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        store
            .create_edge("NEXT", nodes[src], nodes[dst], BTreeMap::new())
            .unwrap();
    }

    let result = run(
        &store,
        "MATCH (a:Item {name: 'a'})-[:NEXT*2]->(d:Item {name: 'd'}) RETURN d.name",
    );
    assert_eq!(result.rows.len(), 2, "both distinct paths must survive");
}

#[test]
fn variable_length_does_not_reuse_relationship_in_same_path() {
    use std::collections::BTreeMap;

    // An undirected traversal may cross the same edge in either direction,
    // but relationship uniqueness forbids using it to walk a -> b -> a.
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store.create_edge("LINK", a, b, BTreeMap::new()).unwrap();

    let result = run(&store, "MATCH (a:Item)-[:LINK*2]-(b:Item) RETURN b");
    assert!(result.rows.is_empty());
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
        store
            .create_edge("NEXT", prev, next, BTreeMap::new())
            .unwrap();
        prev = next;
    }

    let stmt = parse("MATCH (n:Item {idx: 0})-[:NEXT*0..]->(m:Item) RETURN m.idx").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("depth cap"),
        "expected a depth-cap error, got: {err}"
    );
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

    let mut m1_props = BTreeMap::new();
    m1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let m1 = store.create_node(&["Post"], m1_props).unwrap();
    store
        .create_edge("HAS_CREATOR", m1, alice, BTreeMap::new())
        .unwrap();

    let mut m2_props = BTreeMap::new();
    m2_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(2));
    let m2 = store.create_node(&["Comment"], m2_props).unwrap();
    store
        .create_edge("HAS_CREATOR", m2, alice, BTreeMap::new())
        .unwrap();

    let mut p1_props = BTreeMap::new();
    p1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(3));
    let p1 = store.create_node(&["Post"], p1_props).unwrap();
    store
        .create_edge("HAS_CREATOR", p1, bob, BTreeMap::new())
        .unwrap();
    store
        .create_edge("REPLY_OF", m2, p1, BTreeMap::new())
        .unwrap();

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
    assert_eq!(
        result.rows.len(),
        2,
        "both of Alice's messages must resolve to a post+author"
    );

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
    assert_eq!(
        values,
        vec![3, 4],
        "only the top-2-by-idx rows from WITH should reach the second MATCH"
    );
}

/// Real Cypher allows chained plain `MATCH` clauses with no `WITH`
/// between them (an implicit join on any shared variable, e.g. TCK's
/// Match5 `[1]`: `MATCH (a:A) MATCH (a)-[:LIKES*]->(c) RETURN c.name`) --
/// an earlier version of this parser wrongly required `WITH` there,
/// based on a mistaken assumption about real Cypher's own rule (only
/// OPTIONAL MATCH/UNWIND were exempted, when in fact plain MATCH needs no
/// exemption at all). The executor's `carried_vars` threading already
/// handled this correctly regardless -- the parser-level check was the
/// only thing blocking it.
#[test]
fn multiple_match_without_with_is_allowed() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'a'})-[:LIKES]->(:B {name: 'b'})");
    run(&store, "CREATE (:A {name: 'a2'})");

    let result = run(
        &store,
        "MATCH (a:A) MATCH (a)-[:LIKES]->(b) RETURN a.name, b.name",
    );
    assert_eq!(result.rows.len(), 1);
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Property(marsdb_graph::PropertyValue::String(a)),
            Value::Property(marsdb_graph::PropertyValue::String(b)),
        ) => {
            assert_eq!(a, "a");
            assert_eq!(b, "b");
        }
        other => panic!("unexpected value {other:?}"),
    }
}

/// Two paths are equal iff they visit the same nodes/relationships in the
/// same order -- `value_eq` had no `Value::Path` arm at all before this,
/// so any two paths were unconditionally unequal via `=` (fell through to
/// the catch-all `_ => false`). TCK's Comparison1 [14]: a self-loop
/// traversed forward vs backward is the same path (same single node,
/// same single relationship) either way.
#[test]
fn path_equality_compares_nodes_and_relationships_not_always_false() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A)-[:LOOP]->(n)");

    let result = run(
        &store,
        "MATCH p1 = (:A)-->() MATCH p2 = (:A)<--() RETURN p1 = p2",
    );
    assert!(bool_val(&result.rows[0][0]));

    run(&store, "CREATE (:B)-[:X]->(:C)");
    let result = run(
        &store,
        "MATCH p1 = (:A)-->() MATCH p2 = (:B)-->(:C) RETURN p1 = p2",
    );
    assert!(!bool_val(&result.rows[0][0]));
}

/// `MATCH (a:A), (b:B)` -- a genuine disjoint cross join, not a
/// continuation (`b` doesn't continue from `a`'s pattern). Real Cypher's
/// own implicit-join shape (TCK's Merge6/Merge7), previously rejected
/// outright since `group_into_linear_patterns` used to require every
/// comma-separated pattern to continue the previous one.
#[test]
fn comma_separated_match_patterns_cross_join() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A), (:A), (:B)");

    let result = run(&store, "MATCH (a:A), (b:B) RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 2);
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

/// A comma-separated cross join followed directly by `MERGE` -- the
/// disjoint groups become separate `QueryClause::Match` entries, and
/// `MERGE` still sees both `a`/`b` bound by the time it runs (TCK's
/// Merge6 [1]).
#[test]
fn comma_separated_match_cross_join_feeds_a_following_merge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A), (:B)");

    run(
        &store,
        "MATCH (a:A), (b:B) MERGE (a)-[:KNOWS]->(b) ON CREATE SET b.created = 1",
    );
    let result = run(&store, "MATCH (b:B) RETURN b.created");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// MERGE might need to *create* its relationship on no-match, and a
/// brand new edge with no type is meaningless -- same requirement CREATE
/// already had, but `bind_merge` never checked it (TCK's Merge5 [24]).
#[test]
fn merge_requires_an_explicit_relationship_type() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (a), (b) MERGE (a)-[NO_COLON]->(b)").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("an untyped MERGE relationship pattern must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("explicit relationship type"),
        "unexpected error: {msg}"
    );
}

#[test]
fn chaining_past_multiple_with_boundaries_works() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:A {num: 1, num2: 4}), (:A {num: 5, num2: 2}), (:A {num: 9, num2: 0})",
    );
    // Two chained WITH boundaries (a real, previously-rejected shape --
    // TCK's WithOrderBy4, chained `WITH x AS y WITH y % 3 AS y ...`).
    let result = run(
        &store,
        "MATCH (a:A) WITH a.num AS x WITH x % 3 AS x ORDER BY x * -1 LIMIT 3 RETURN x",
    );
    let values: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(values, vec![2, 1, 0]);

    // Three chained WITH boundaries, mixing an aggregating one in the
    // middle -- grouping and non-grouping WITH clauses both carry
    // through correctly across a chain, not just a single boundary.
    let result = run(
        &store,
        "MATCH (a:A) WITH a AS a, a.num + a.num2 AS sum \
         WITH a.num2 % 3 AS mod, min(sum) AS min \
         WITH mod AS mod, min AS min ORDER BY min LIMIT 2 RETURN mod, min",
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
    assert_eq!(rows, vec![(1, 5), (2, 7)]);
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
    store
        .create_edge("HAS_CREATOR", m, original_author, BTreeMap::new())
        .unwrap();

    let mut p1_props = BTreeMap::new();
    p1_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("KnowsAuthor".into()),
    );
    let p1 = store.create_node(&["Person"], p1_props).unwrap();
    let mut c1_props = BTreeMap::new();
    c1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(10));
    let c1 = store
        .create_node(&["Comment", "Message"], c1_props)
        .unwrap();
    store
        .create_edge("REPLY_OF", c1, m, BTreeMap::new())
        .unwrap();
    store
        .create_edge("HAS_CREATOR", c1, p1, BTreeMap::new())
        .unwrap();
    store
        .create_edge("KNOWS", p1, original_author, BTreeMap::new())
        .unwrap();

    let mut p2_props = BTreeMap::new();
    p2_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("StrangerAuthor".into()),
    );
    let p2 = store.create_node(&["Person"], p2_props).unwrap();
    let mut c2_props = BTreeMap::new();
    c2_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(20));
    let c2 = store
        .create_node(&["Comment", "Message"], c2_props)
        .unwrap();
    store
        .create_edge("REPLY_OF", c2, m, BTreeMap::new())
        .unwrap();
    store
        .create_edge("HAS_CREATOR", c2, p2, BTreeMap::new())
        .unwrap();
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

    assert_eq!(
        extract(&result.rows[0]),
        (10, "KnowsAuthor".to_string(), true)
    );
    assert_eq!(
        extract(&result.rows[1]),
        (20, "StrangerAuthor".to_string(), false)
    );

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
    assert_eq!(
        result.rows.len(),
        1,
        "the outer MATCH row must survive even with zero optional matches"
    );
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
    let result = run(
        &store,
        "MATCH (n:Item) OPTIONAL MATCH (n)-[:X]->(m) RETURN n.name",
    );
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

fn float_value(v: &Value) -> f64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => *f,
        other => panic!("expected a float, got {other:?}"),
    }
}

fn bool_value(v: &Value) -> bool {
    match v {
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected a bool, got {other:?}"),
    }
}

fn list_str_values(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items.iter().map(str_value).collect(),
        other => panic!("expected a list, got {other:?}"),
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
fn min_max_on_non_orderable_errors() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN min(n) AS m").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("comparable"),
        "expected a comparability error, got: {err}"
    );
}

/// `max()`/`min()` over a list argument -- real Cypher orders a list
/// element-by-element (reusing the same `list_cmp_asc` ORDER BY
/// already uses), found as a real bug: `comparable_ordering` had no
/// `List` arm at all, so any list argument unconditionally errored
/// "requires a comparable scalar argument". Aggregation2 [9]/[10].
#[test]
fn max_min_over_list_values() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [[1], [2], [2, 1]] AS x RETURN max(x), min(x)",
    );
    match (&result.rows[0][0], &result.rows[0][1]) {
        (Value::List(max), Value::List(min)) => {
            assert_eq!(max.len(), 2); // [2, 1]
            assert_eq!(min.len(), 1); // [1]
        }
        other => panic!("expected two lists, got {other:?}"),
    }
}

/// `max()`/`min()` over genuinely mixed types (numbers, strings, a
/// list) -- real Cypher's cross-type orderability ranks `List` *below*
/// every scalar (sorts first), the opposite of an earlier, unverified
/// version of `type_rank` that put it last. Aggregation2 [11]/[12].
#[test]
fn max_min_over_mixed_types_including_a_list() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [1, 'a', null, [1, 2], 0.2, 'b'] AS x RETURN max(x), min(x)",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 1),
        other => panic!("expected max to be the int 1, got {other:?}"),
    }
    match &result.rows[0][1] {
        Value::List(items) => assert_eq!(items.len(), 2), // [1, 2]
        other => panic!("expected min to be the list [1, 2], got {other:?}"),
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
fn with_where_and_or_not() {
    let store = GraphStore::open_memory().unwrap();
    for i in [5, 15, 25, 35] {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 AND NOT y > 30 RETURN y ORDER BY y",
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

    let result = run(
        &store,
        "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Bob");
}

#[test]
fn match_create_adds_new_node_to_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");

    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:OWNS]->(i:Item {name: 'Widget'})",
    );

    let result = run(
        &store,
        "MATCH (a:Person)-[:OWNS]->(i:Item) RETURN a.name, i.name",
    );
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

    let result = run(
        &store,
        "MATCH (p:Person)-[:HAS_LOG]->(l:Log) RETURN count(*)",
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        3,
        "one new Log node per matched Person row"
    );
}

#[test]
fn match_create_rejects_relabeling_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    let stmt =
        parse("MATCH (a:Person {name: 'Alice'}) CREATE (a:Employee)-[:X]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MATCH (a) CREATE (a)` -- a bare already-bound node token with no
/// relationship at all doesn't create anything (that var already
/// exists) and doesn't connect anything either, so real Cypher rejects
/// it outright rather than silently no-op'ing.
#[test]
fn match_create_rejects_bare_already_bound_node_with_no_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `VariableAlreadyBound` is a compile-time/structural error, not a
/// data-dependent one -- it must fire even when the preceding `MATCH`
/// produces *zero* rows, since the check is about variable scope, not
/// runtime data. A real bug found and fixed: the check used to live only
/// in the per-row runtime path (`materialize_create`/`merge_one_row`),
/// which a zero-row MATCH skips entirely, silently no-op'ing instead of
/// erroring. Now duplicated at compile time in `semantic.rs`. TCK's
/// Create1 [13]/[14], Merge1 [15], Merge5 [26] ("any graph" -- MarsDB's
/// own harness exercises the empty-graph case).
#[test]
fn already_bound_rejection_fires_even_on_a_zero_row_match() {
    let store = GraphStore::open_memory().unwrap();
    for q in [
        "MATCH (a) CREATE (a)",
        "MATCH (a) CREATE (a {name: 'foo'}) RETURN a",
        "MATCH (a) MERGE (a)",
        "MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)",
    ] {
        let stmt = parse(q).unwrap();
        let result = Executor::new(&store).execute(&stmt);
        let err = match result {
            Ok(r) => panic!(
                "expected an already-bound error for {q:?}, got Ok: {:?}",
                r.rows
            ),
            Err(e) => e,
        };
        assert!(
            err.to_string().to_lowercase().contains("already bound"),
            "expected an already-bound error for {q:?}, got: {err}"
        );
    }
}

/// TCK Create1 [19]: `(n {})` -- an *explicit but empty* inline map --
/// still counts as "imposing a new predicate" on an already-bound node,
/// same as a non-empty one. A real bug found via the TCK: `NodePattern`'s
/// `props` alone can't tell `(n {})` apart from `(n)` (both give an empty
/// `Vec`), so the already-bound check silently passed until
/// `has_explicit_props` was added specifically to preserve that
/// syntactic distinction.
#[test]
fn create_rejects_empty_explicit_prop_map_on_already_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (n:Foo) CREATE (n {})-[:OWNS]->(:Dog)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// TCK Merge5 [22]: MERGE's node endpoints share CREATE's "might need to
/// create this node" reasoning -- an already-bound variable can't carry a
/// new label/property predicate there either, even when it's a different
/// endpoint of the same relationship than the one that's actually bound.
#[test]
fn merge_rejects_new_label_predicate_on_already_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Foo) MERGE (a)-[r:KNOWS]->(a:Bar)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
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
    assert_eq!(
        result.rows.len(),
        4,
        "2 Left x 2 Right must cross-join to 4 rows, not drop `a`"
    );
    let pairs: Vec<(String, String)> = result
        .rows
        .iter()
        .map(|r| (str_value(&r[0]), str_value(&r[1])))
        .collect();
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

    let result = run(
        &store,
        "MATCH (a:Left) OPTIONAL MATCH (c:Right) RETURN a.name, c.name",
    );
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
    run(
        &store,
        r"CREATE (:Path {p: 'C:\\Users\\x', tab: 'a\tb', nl: 'a\nb'})",
    );
    let result = run(&store, "MATCH (n:Path) RETURN n.p, n.tab, n.nl");
    assert_eq!(str_value(&result.rows[0][0]), r"C:\Users\x");
    assert_eq!(str_value(&result.rows[0][1]), "a\tb");
    assert_eq!(str_value(&result.rows[0][2]), "a\nb");
}

#[test]
fn string_literal_unrecognized_escape_errors() {
    // `\q` isn't one of openCypher.bnf's own closed set of valid escape
    // sequences (backslash/quote/tab/etc/\uXXXX) -- a real syntax error,
    // not just a specific message's wording, which legitimately differs
    // by implementation (a lenient-lex-then-semantic-check parser can
    // give a precise "unrecognized escape" message; a parser whose lexer
    // itself only matches real escape sequences, arguably closer to spec,
    // fails to tokenize the string at all instead -- both are correct
    // rejections of the same invalid input).
    let err = parse(r"MATCH (n {x: 'a\qb'}) RETURN n").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("syntax error"));
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
    let result = run(
        &store,
        "MATCH (p:Person) UNWIND [1, 2] AS n RETURN p.name AS name, n ORDER BY name, n",
    );
    let pairs: Vec<(String, i64)> = result
        .rows
        .iter()
        .map(|r| (str_value(&r[0]), int_value(&r[1])))
        .collect();
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

#[test]
fn unwind_then_with_where_filters() {
    // `UNWIND ... WHERE ...` (WHERE directly on UNWIND, no WITH needed
    // in between) isn't real openCypher -- UNWIND has no WHERE of its own
    // per openCypher.bnf, only a standalone WITH does. `UNWIND ... WITH
    // ... WHERE ...` is the real, spec-correct equivalent.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [1, 2, 3, 4, 5] AS x WITH x WHERE x > 2 RETURN x ORDER BY x",
    );
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
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "second MERGE must reuse, not create a duplicate"
    );
}

#[test]
fn merge_one_hop_both_endpoints_bound_reuses_existing_edge() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
    );
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) MERGE (a)-[:KNOWS]->(b)",
    );
    let result = run(
        &store,
        "MATCH (:Person)-[r:KNOWS]->(:Person) RETURN count(*)",
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "must reuse the existing edge, not create a 2nd one"
    );
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
    let result = run(
        &store,
        "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)",
    );
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
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) MERGE (a)-[:KNOWS]->(b:Person {name: 'Bob'})",
    );

    let bobs = run(&store, "MATCH (n:Person {name: 'Bob'}) RETURN count(*)");
    assert_eq!(
        int_value(&bobs.rows[0][0]),
        2,
        "must create a 2nd Bob, not reuse the unconnected one"
    );
    let connected = run(
        &store,
        "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)",
    );
    assert_eq!(int_value(&connected.rows[0][0]), 1);
}

#[test]
fn merge_standalone_both_endpoints_fresh() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(
        &store,
        "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    let nodes = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(
        int_value(&nodes.rows[0][0]),
        2,
        "2nd MERGE must reuse both nodes and the edge, not duplicate"
    );
    let edges = run(
        &store,
        "MATCH (:Person)-[:KNOWS]->(:Person) RETURN count(*)",
    );
    assert_eq!(int_value(&edges.rows[0][0]), 1);
}

#[test]
fn merge_on_create_and_on_match_fire_on_the_right_rows() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2",
    );
    let after_create = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_create.rows[0][0]), 1);

    run(
        &store,
        "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2",
    );
    let after_match = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_match.rows[0][0]), 2);
}

/// `MERGE (a)` -- a completely unconstrained, unbound node pattern (no
/// label/property) is real, valid Cypher (TCK's Merge1 [1]): searches
/// for/creates any node with no constraints at all. An earlier version of
/// this codebase treated this as an "ambiguous shape" mistake to reject,
/// which real Cypher's own TCK disproves.
#[test]
fn merge_unconstrained_node_pattern_creates_then_matches() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "MERGE (a) RETURN count(*) AS n");
    assert_eq!(int(&result.rows[0][0]), 1);
    assert_eq!(
        int(&run(&store, "MATCH (n) RETURN count(*) AS c").rows[0][0]),
        1
    );

    // A second MERGE (a) must match the existing node, not create a
    // second one.
    run(&store, "MERGE (a)");
    assert_eq!(
        int(&run(&store, "MATCH (n) RETURN count(*) AS c").rows[0][0]),
        1
    );
}

/// `parse_many`'s `queries` grammar used to have `~ ";"? ~ EOI` at the
/// end -- with a genuinely-trailing `;`, `(";" ~ statement)*` greedily
/// consumed it as one more separator, needing a `statement` after it;
/// since `match_stmt` can match zero-width, an empty string satisfied
/// that, producing a spurious extra empty statement that then failed its
/// own "needs a tail" validation. Caught via the TCK's binary-tree named-
/// graph fixture, a real multi-statement file that ends with `;`.
#[test]
fn parse_many_tolerates_a_trailing_semicolon() {
    assert_eq!(marsdb_query::parse_many("CREATE (a);").unwrap().len(), 1);
    assert_eq!(marsdb_query::parse_many("CREATE (a)").unwrap().len(), 1);
    assert_eq!(
        marsdb_query::parse_many("CREATE (a); CREATE (b);")
            .unwrap()
            .len(),
        2
    );
    // A semicolon inside a string literal must not get mis-split either.
    assert_eq!(
        marsdb_query::parse_many("RETURN ';' AS x;").unwrap().len(),
        1
    );
}

/// `map['key']` -- real Cypher's dynamic map-field access, previously
/// rejected at compile time even though `apply_index`'s runtime already
/// fully supported it. `null['key']` must still be `null`, not an error
/// (a `null`-valued base types as `Kind::Scalar` in this codebase's
/// imprecise `Kind` system, deliberately tolerated here the same way
/// every other `Kind::Scalar` case is).
#[test]
fn map_index_access() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH {a: 1, b: 2} AS m RETURN m['a'] AS value");
    assert_eq!(int(&result.rows[0][0]), 1);

    let result = run(
        &store,
        "WITH null AS expr, 'x' AS idx RETURN expr[idx] AS value",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
}

/// A `MATCH`/`MERGE` pattern's inline `{key: value}` can be any
/// expression, not just a literal -- a bound variable (TCK's Merge1 [8])
/// compiles to `Expr::GeneralCompare` (a generic post-scan filter,
/// evaluated per-row) instead of the index-seek-eligible
/// `Expr::Compare(_, _, Literal)` shape a real literal still gets.
#[test]
fn pattern_property_accepts_a_bound_variable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({var: 42}), ({var: 'not42'})");

    let result = run(&store, "WITH 42 AS x MATCH (n {var: x}) RETURN n.var");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 42);
}

/// A non-aggregating, non-`DISTINCT` `WITH`'s own `ORDER BY` sees both
/// the pre-WITH scope and the new aliases, not just the projected names
/// -- `WITH a.count AS count ORDER BY a.count` (`a` isn't projected but
/// is still a valid sort key), matching `WHERE`'s own already-
/// implemented merged-scope rule (TCK's With4 [6]/WithSkipLimit3 [3]).
#[test]
fn with_order_by_sees_pre_with_scope() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "UNWIND range(0, 15) AS i CREATE ({count: i})");

    let result = run(
        &store,
        "MATCH (a) WITH a.count AS count ORDER BY a.count SKIP 10 LIMIT 10 RETURN count",
    );
    let vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    assert_eq!(vals, vec![10, 11, 12, 13, 14, 15]);
}

/// TCK's WithOrderBy2 [24]: a `DISTINCT` (non-aggregating) `WITH`'s own
/// `ORDER BY` needs the same "verbatim item match" shortcut aggregation
/// already had -- a real bug found via the TCK, `DISTINCT`'s own
/// `order_scope` is narrowed to the projected names only (same reasoning
/// as aggregation: both collapse many pre-WITH rows into one output row,
/// so there's no single pre-WITH row to resolve `a.name` against), but
/// the shortcut that lets `ORDER BY a.name` still resolve via its own
/// `name` alias only checked `with_aggregates`, not `with.distinct` --
/// `a.name` (a pre-WITH property access) failed to resolve at all.
#[test]
fn with_distinct_order_by_sees_its_own_item_verbatim() {
    let store = GraphStore::open_memory().unwrap();
    for name in ["A", "A", "B", "C", "C"] {
        run(&store, &format!("CREATE ({{name: '{name}'}})"));
    }
    let result = run(
        &store,
        "MATCH (a) WITH DISTINCT a.name AS name ORDER BY a.name ASC LIMIT 1 RETURN *",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "A"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `RETURN *` -- every currently-bound variable, alphabetically. Can't
/// resolve to a concrete `Tail::Return` at parse time (no scope exists
/// yet); resolved independently in both `semantic.rs` (compile-time
/// validation) and `executor.rs` (`carried_vars`, at actual execution).
#[test]
fn return_star_returns_every_bound_variable_alphabetically() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:REL]->(:X)");
    run(&store, "CREATE (:B)");

    let result = run(
        &store,
        "MATCH (n:A) WITH n LIMIT 1 MATCH (m:B), (n)-->(x:X) RETURN *",
    );
    assert_eq!(result.columns, vec!["m", "n", "x"]);
    assert_eq!(result.rows.len(), 1);

    // Nothing bound at all -- a real compile-time error, not an empty
    // projection.
    let stmt = parse("MATCH () RETURN *").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("at least one variable"));
}

/// `$1` -- real Cypher's legacy positional-parameter form (a plain
/// non-negative-integer name), not just a `$name` identifier.
#[test]
fn numeric_named_parameters() {
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    let mut stmt = parse("RETURN $1 AS x").unwrap();
    let mut params = HashMap::new();
    params.insert("1".to_string(), marsdb_graph::PropertyValue::Int(42));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int(&result.rows[0][0]), 42);
}

/// A list-valued (including nested-list) `$param` -- `Literal` has no
/// list variant (no list-literal *syntax* to substitute one into), so
/// `substitute_params` rewrites the whole `ReturnExpr::Lit(Literal::
/// Param(_))` node into a `ReturnExpr::ListLit` instead, recursively.
/// TCK's List1 [3]/[5], Null3 [4].
#[test]
fn list_valued_parameters_substitute_into_a_list_literal_expression() {
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();

    let mut stmt = parse("RETURN $coll[1] AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::String("a".into()),
            marsdb_graph::PropertyValue::String("b".into()),
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "b"),
        other => panic!("unexpected value {other:?}"),
    }

    // Three-valued `IN`: a `null` element present, no definite match ->
    // "unknown" (null), not `false`.
    let mut stmt = parse("RETURN 2 IN $coll AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::Int(1),
            marsdb_graph::PropertyValue::Null,
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert!(matches!(&result.rows[0][0], Value::Null));

    // Nested list -- a param list can itself contain lists.
    let mut stmt = parse("RETURN $coll[1][0] AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::Int(1),
            marsdb_graph::PropertyValue::List(vec![marsdb_graph::PropertyValue::Int(2)]),
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int(&result.rows[0][0]), 2);
}

/// A bare (unparenthesized) `var:Label` used directly as a `WITH ...
/// WHERE` predicate (`WHERE i.var > 'te' AND i:TextNode`) -- distinct
/// from `label_check_expr`'s own `(n:Foo)` parenthesized general-
/// expression form. Pattern-level `WHERE` already had this; `WITH`'s own
/// `WHERE` didn't.
#[test]
fn with_where_bare_label_predicate() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:TextNode {var: 'text'})");
    run(&store, "CREATE (:IntNode {var: 0})");

    let result = run(&store, "MATCH (i) WITH i WHERE i:TextNode RETURN i.var");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "text"),
        other => panic!("unexpected value {other:?}"),
    }

    let result = run(
        &store,
        "MATCH (i) WITH i WHERE i.var > 'te' AND i:TextNode RETURN i.var",
    );
    assert_eq!(result.rows.len(), 1);
}

/// `SET ... WITH ...` -- continues the query past the mutation instead
/// of only ever allowing one trailing `RETURN` right after it (TCK's
/// Set6 [5]). `QueryClause::Set` doesn't change any row's bindings, only
/// the underlying graph -- confirmed here by the SET's own side effect
/// (all 5 nodes incremented) being independent of the later WHERE filter
/// only letting 3 of them through to the final RETURN.
#[test]
fn set_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) SET n.num = n.num + 1 WITH n WHERE n.num % 2 = 0 RETURN n.num AS num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4, 6]);

    // The SET's own side effect applies to every matched row, not just
    // the ones that survive the later WHERE filter.
    let all = run(&store, "MATCH (n:N) RETURN n.num");
    let mut all_vals: Vec<i64> = all.rows.iter().map(|row| int(&row[0])).collect();
    all_vals.sort();
    assert_eq!(all_vals, vec![2, 3, 4, 5, 6]);

    // The pre-existing `SET ... RETURN` (no WITH) shape must still work
    // unaffected -- this is the grammar's positive-lookahead safety net
    // (`set_as_clause` only ever fires when a real WITH is definitely
    // next), not just a coincidence.
    let result = run(&store, "MATCH (n:N {num: 2}) SET n.num = 100 RETURN n.num");
    assert_eq!(int(&result.rows[0][0]), 100);

    // A bare terminal SET with nothing after must still work too.
    run(&store, "MATCH (n:N {num: 3}) SET n.num = 200");
    let result = run(&store, "MATCH (n:N {num: 200}) RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `DELETE ... WITH ...` -- same continuation as `SET ... WITH ...` above
/// (TCK's Delete6 [5]/[6]/[7]), applied to `QueryClause::Delete`. The
/// deleted node's own `num` was already carried into a `WITH`-projected
/// scalar before the DELETE, so the later WHERE filter/aggregation never
/// touches the now-gone node itself.
#[test]
fn delete_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) WITH n, n.num AS num DELETE n WITH num WHERE num % 2 = 0 RETURN num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4]);

    // Every matched node was actually deleted, not just filtered out of
    // the result set.
    let left = run(&store, "MATCH (n:N) RETURN count(n) AS c");
    assert_eq!(int(&left.rows[0][0]), 0);

    // The pre-existing `DELETE ... RETURN` (no WITH) shape must still work.
    run(&store, "CREATE (:M {num: 1})");
    let result = run(&store, "MATCH (n:M) DELETE n RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `REMOVE ... WITH ...` -- same continuation, applied to
/// `QueryClause::Remove` (TCK's Remove3).
#[test]
fn remove_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}, tag: 'x'}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) REMOVE n.tag WITH n WHERE n.num % 2 = 0 RETURN n.num AS num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4]);

    // The REMOVE's own side effect applies to every matched row, not just
    // the ones that survive the later WHERE filter.
    let tagged = run(
        &store,
        "MATCH (n:N) WHERE n.tag IS NOT NULL RETURN count(n) AS c",
    );
    assert_eq!(int(&tagged.rows[0][0]), 0);

    // The pre-existing `REMOVE ... RETURN` (no WITH) shape must still work.
    let result = run(&store, "MATCH (n:N {num: 2}) REMOVE n:N RETURN labels(n)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
}

fn node_labels(v: &Value) -> Vec<String> {
    let Value::Node(node) = v else {
        panic!("expected a node, got {v:?}");
    };
    node.labels.clone()
}

/// `WHERE (n)-[:REL]->()` used as a boolean predicate (TCK's Pattern1) --
/// existential: true iff a real match exists, without binding a fresh
/// row per match (unlike an ordinary `MATCH`). Covers the existential/
/// negated/conjunction shapes and the two-already-bound-endpoints shape
/// (`(n)-->(m)`, both `n`/`m` from an outer `MATCH (n), (m)`); the
/// compile-time rejection of a pattern predicate introducing a brand
/// new, never-bound variable is `pattern_predicate_introducing_new_variable_is_rejected` below.
#[test]
fn pattern_predicate_in_where() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A)-[:REL1]->(b:B), (b)-[:REL2]->(a), (a)-[:REL3]->(:C), (a)-[:REL1]->(:D)",
    );

    let result = run(&store, "MATCH (n) WHERE (n)-[]->() RETURN n");
    let mut labels: Vec<Vec<String>> = result.rows.iter().map(|r| node_labels(&r[0])).collect();
    labels.sort();
    assert_eq!(labels, vec![vec!["A".to_string()], vec!["B".to_string()]]);

    let result = run(&store, "MATCH (n) WHERE (n)-[:REL1*]->() RETURN n");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(node_labels(&result.rows[0][0]), vec!["A".to_string()]);

    let result = run(&store, "MATCH (n), (m) WHERE (n)-[]->(m) RETURN n, m");
    assert_eq!(result.rows.len(), 4);

    let result = run(&store, "MATCH (n) WHERE NOT (n)-[:REL2]-() RETURN n");
    let mut labels: Vec<Vec<String>> = result.rows.iter().map(|r| node_labels(&r[0])).collect();
    labels.sort();
    assert_eq!(labels, vec![vec!["C".to_string()], vec!["D".to_string()]]);

    let result = run(
        &store,
        "MATCH (n) WHERE (n)-[:REL1]-() AND (n)-[:REL3]-() RETURN n",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(node_labels(&result.rows[0][0]), vec!["A".to_string()]);
}

/// TCK's Pattern1 [10] outline -- a pattern predicate's endpoint that was
/// never bound anywhere else (`a` here) is a compile-time error, not a
/// pattern that silently matches anything.
#[test]
fn pattern_predicate_introducing_new_variable_is_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)");
    let stmt = parse("MATCH (n) WHERE (n)-[r]->(a) RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string()
            .to_lowercase()
            .contains("undefined variable"),
        "expected an undefined-variable error, got: {err}"
    );
}

/// `a:Label` as a general boolean expression, usable anywhere a
/// `return_expr` is (RETURN/WITH items, not just pattern-level WHERE) --
/// TCK's Graph5 "Node and edge label expressions". Reuses the existing
/// `ReturnExpr::HasLabel` (previously only reachable via the
/// parenthesized `(n:Foo)` general-expression form) through a new bare
/// grammar alternative.
#[test]
fn bare_label_predicate_as_return_expr() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:A:B:C), (:A:B), (:A:C), (:B:C), (:A), (:B), (:C), ()",
    );

    let result = run(&store, "MATCH (a) RETURN a:B AS result");
    let mut vals: Vec<bool> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
            other => panic!("expected a bool, got {other:?}"),
        })
        .collect();
    vals.sort();
    assert_eq!(
        vals,
        vec![false, false, false, false, true, true, true, true]
    );

    // A DELETE target must never be a label predicate (TCK's Delete1
    // [8]) -- a boolean can never be a node/relationship/path, and this
    // must be rejected at compile time regardless of whether any row
    // actually matches (an empty MATCH would otherwise let it through).
    let stmt = parse("MATCH (n) DELETE n:Person").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("delete target"),
        "expected a DELETE-target error, got: {err}"
    );
}

/// A standalone (no preceding `MATCH`) `CREATE ... RETURN ...`/`EXPLAIN
/// CREATE ... RETURN ...` -- TCK's Graph3 "Node labels" (e.g. `CREATE
/// (node) RETURN labels(node)`) and many similarly-shaped Create1
/// scenarios. `create_stmt`'s own top-level `statement` alternative used
/// to greedily match just the `CREATE (...)` half and leave `RETURN ...`
/// unconsumed (a real parse failure) since a successful `|` alternative
/// is never revisited just because something later fails to parse --
/// `create_stmt_only`'s `!(return_clause | with_clause)` lookahead is
/// what now lets this correctly fall through to `match_stmt`'s own
/// `mutating_tail` instead, which already fully supports it.
#[test]
fn standalone_create_followed_by_return() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());

    let result = run(
        &store,
        "CREATE (node:Foo:Bar {name: 'Mattias'}) RETURN labels(node)",
    );
    let mut labels = list_str_values(&result.rows[0][0]);
    labels.sort();
    assert_eq!(labels, vec!["Bar".to_string(), "Foo".to_string()]);

    // The plain, nothing-after shape (Statement::Create) must still work
    // unaffected.
    run(&store, "CREATE (:X), (:Y)");
    let result = run(&store, "MATCH (n) RETURN count(n) AS c");
    assert_eq!(int(&result.rows[0][0]), 4);

    // EXPLAIN has the identical trap (its own `create_stmt`-in-ordered-
    // choice list) -- must not error for either shape.
    let stmt = parse("EXPLAIN CREATE (:Q)").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_ok());
    let stmt = parse("EXPLAIN CREATE (n:Q) RETURN n").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_ok());
}

/// `^` (exponentiation) and general unary minus -- TCK's Precedence2
/// "On numeric values" plus Return2 [1]. `^` always produces a `Float`
/// (even for two `Int`s), binds tighter than `*`/`/`/`%`/`+`/`-` but
/// looser than unary minus, and is LEFT-associative (`4^(3*2)^3` is
/// `(4^6)^3`, confirmed against the real TCK fixture -- general math
/// convention's right-associativity would have been wrong here).
#[test]
fn exponentiation_and_unary_minus_precedence() {
    let store = GraphStore::open_memory().unwrap();

    // `^` binds tighter than `*`.
    let result = run(&store, "RETURN 4 ^ 3 * 2 ^ 3 AS a, 4 ^ (3 * 2) ^ 3 AS c");
    assert!((as_float(&result.rows[0][0]) - 512.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - 68719476736.0).abs() < 1e-3);

    // `^` binds tighter than `+`.
    let result = run(&store, "RETURN 4 ^ 3 + 2 ^ 3 AS a, 4 ^ (3 + 2) ^ 3 AS c");
    assert!((as_float(&result.rows[0][0]) - 72.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - 1073741824.0).abs() < 1e-3);

    // Unary minus binds tighter than `^`: `-3^2` is `(-3)^2`, not `-(3^2)`.
    let result = run(&store, "RETURN -3 ^ 2 AS a, -(3 ^ 2) AS c");
    assert!((as_float(&result.rows[0][0]) - 9.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - (-9.0)).abs() < 1e-9);

    // A negative numeric literal is unaffected -- still a plain `Literal`,
    // not a `Neg`-wrapped computed value (preserves the planner's
    // index-seek fusion for `MATCH (n {x: -5})`-shaped patterns).
    let result = run(&store, "RETURN -3 AS x");
    assert_eq!(int(&result.rows[0][0]), -3);

    // General unary minus on a non-literal (a bound variable) -- this is
    // the actually-new grammar shape (`-3` alone always worked). Chained
    // unary minus (`--n`) isn't real openCypher -- per
    // openCypher.bnf's `<arithmetic unary> ::= [<sign>] <postfix
    // expression>`, the sign is a single optional, not repeatable.
    let result = run(&store, "WITH 3 AS n RETURN -n AS x");
    assert_eq!(int(&result.rows[0][0]), -3);
}

/// `WITH *` -- every currently-bound variable carries forward unchanged
/// (TCK's Match8/Match4/Create3/TypeConversion2-4), mirroring `RETURN
/// *`'s own already-supported star expansion. Covers both a plain `WITH
/// *` and one combined with an extra item, plus the "includes this same
/// clause's own new bindings, not just what was carried in before it"
/// case (`MERGE`'s own target must be visible after `WITH *`, matching
/// TCK's Match8 [2]).
#[test]
fn with_star_carries_every_bound_variable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X)");

    let result = run(
        &store,
        "MATCH (a) MERGE (b:Y) WITH * OPTIONAL MATCH (a)--(b) RETURN count(*) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 1);

    // Chained `WITH x WITH *` -- the star sees the prior WITH's own
    // output, not the original pre-WITH scope. Two nodes exist by now
    // (:X from setup, :Y from the MERGE above), so this matches both.
    let result = run(&store, "MATCH (a) WITH a WITH * RETURN a");
    assert_eq!(result.rows.len(), 2);

    // `WITH *, expr AS extra` -- star-expanded names plus a real
    // additional item in the same clause.
    let result = run(
        &store,
        "WITH 1 AS a, 2 AS b WITH *, a + b AS c RETURN a, b, c",
    );
    assert_eq!(int(&result.rows[0][0]), 1);
    assert_eq!(int(&result.rows[0][1]), 2);
    assert_eq!(int(&result.rows[0][2]), 3);

    // Nothing bound yet -- unlike `RETURN *` (real Cypher's own
    // `NoVariablesInScope`), `WITH *` tolerates this: it's a legal, if
    // useless, "carry forward nothing" no-op (TCK's Create3 [2]/[3]; see
    // `with_star_tolerates_an_empty_scope`).
    let stmt = parse("WITH * RETURN 1").unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `[:A|B]`/`[:A|:B]` -- a relationship pattern matches if the edge's
/// type is any of the listed alternatives (TCK's Match2 [6]/Match3 [8],
/// Pattern1 [13]'s undirected pattern-predicate form). CREATE/MERGE
/// still require exactly one explicit type -- a brand new edge can't be
/// ambiguous about which type it gets.
#[test]
fn multi_type_relationship_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a {name: 'A'}), (b {name: 'B'}), (c {name: 'C'}), \
         (a)-[:KNOWS]->(b), (a)-[:HATES]->(c), (a)-[:WONDERS]->(c)",
    );

    let result = run(&store, "MATCH (n)-[r:KNOWS|HATES]->(x) RETURN r");
    assert_eq!(result.rows.len(), 2);

    // `:T|:T` -- the colon before subsequent alternatives is optional,
    // both forms mean the same thing.
    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE (a)-[:T]->(b)");
    let result = run(&store2, "MATCH (a)-[:T|:T]->(b) RETURN a, b");
    assert_eq!(result.rows.len(), 1);

    // Untyped (`[]`/`[r]`) is unaffected -- still matches any type.
    let store3 = GraphStore::open_memory().unwrap();
    run(&store3, "CREATE (a)-[:X]->(b)");
    let result = run(&store3, "MATCH (a)-[r]->(b) RETURN r");
    assert_eq!(result.rows.len(), 1);

    // CREATE/MERGE reject a multi-type target -- which type would the
    // new edge get?
    let store4 = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a)-[:A|B]->(b)").unwrap();
    assert!(Executor::new(&store4).execute(&stmt).is_err());
    let stmt = parse("MATCH (a), (b) MERGE (a)-[:A|B]->(b)").unwrap();
    assert!(Executor::new(&store4).execute(&stmt).is_err());
}

/// `CREATE ... WITH ...` -- continues the query past the mutation
/// instead of only ever being the very last (optionally RETURN-followed)
/// thing in a statement (TCK's Create3/Match4/Match5/Match6 fixtures,
/// e.g. `CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)`).
/// Unlike `SET`/`DELETE`/`REMOVE`-as-clause, CREATE genuinely changes
/// row bindings (each pattern's own vars) -- confirmed here by the
/// second CREATE seeing the first CREATE's own binding (`a`), and by a
/// `WITH *` in between correctly carrying it forward.
#[test]
fn create_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(
        &store,
        "CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)",
    );
    assert!(result.rows.is_empty());

    let nodes = run(&store, "MATCH (n) RETURN count(n) AS c");
    assert_eq!(int(&nodes.rows[0][0]), 2);
    let rels = run(&store, "MATCH ()-[r:T]->() RETURN count(r) AS c");
    assert_eq!(int(&rels.rows[0][0]), 1);

    // CREATE followed by WITH *, UNWIND, and another CREATE -- the
    // second CREATE's new nodes must each see the first CREATE's own
    // binding across the UNWIND fan-out.
    let store2 = GraphStore::open_memory().unwrap();
    let result = run(
        &store2,
        "CREATE (a {var: 'start'})
         WITH *
         UNWIND range(1, 5) AS i
         CREATE (n {var: i})-[:T]->(a)
         RETURN count(n) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 5);

    // The pre-existing standalone `CREATE ... RETURN ...`/bare-CREATE
    // shapes (PR #106) must still work unaffected.
    let result = run(&store2, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
    run(&store2, "CREATE (:X), (:Y)");
    let result = run(&store2, "MATCH (n:X) RETURN count(n) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `MATCH (a) MERGE (a)` -- a bare already-bound node with no
/// relationship at all doesn't search for or create anything, so real
/// Cypher rejects it, the same rule `materialize_create` already
/// applies to standalone `CREATE (a)`.
#[test]
fn merge_rejects_bare_already_bound_node_with_no_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item)");
    let stmt = parse("MATCH (a:Item) MERGE (a)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)` -- reusing an already-bound
/// relationship as MERGE's own pattern token is always an error; unlike
/// a node endpoint, MERGE has no "search using this specific existing
/// edge" mode.
#[test]
fn merge_rejects_reusing_an_already_bound_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:R]->(:B)");
    let stmt = parse("MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MERGE ({num: null})` -- a null-valued property in MERGE's own
/// pattern can never be searched-or-created consistently (null never
/// matches on search, but storing null is the same as not storing the
/// property at all).
#[test]
fn merge_rejects_a_null_valued_pattern_property() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MERGE ({num: null})").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("null"),
        "expected a null-property error, got: {err}"
    );
}

/// `WITH a, count(*)` -- every WITH item except a bare variable
/// reference needs an explicit `AS alias`, real Cypher's
/// `NoExpressionAlias` error. RETURN has no such requirement (an
/// unaliased expression there just gets an auto-generated column
/// name).
#[test]
fn with_requires_an_alias_for_non_bare_variable_items() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X)");
    let stmt = parse("MATCH (a) WITH a, count(*) RETURN a").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("alias"),
        "expected a missing-alias error, got: {err}"
    );

    // Bare variables need no alias, still works.
    let result = run(&store, "MATCH (a) WITH a RETURN a");
    assert_eq!(result.rows.len(), 1);
    // An explicitly aliased aggregate is fine too.
    let result = run(&store, "MATCH (a) WITH count(*) AS c RETURN c");
    assert_eq!(result.rows.len(), 1);
}

/// `RETURN 1 AS a, 2 AS a` -- reusing the same explicit alias for two
/// columns is a real error (`ColumnNameConflict`). An unaliased
/// expression repeated (`RETURN date(x), date(y)`) is *not* a
/// conflict, even though both currently fall back to the same generic
/// placeholder column name (`"date(...)"`, not argument-aware) --
/// only a genuinely meaningful name (an alias, or a bare variable/
/// property-access name) can actually collide.
#[test]
fn return_rejects_duplicate_explicit_column_names() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AS a, 2 AS a").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("duplicate"),
        "expected a duplicate-column error, got: {err}"
    );

    let result = run(&store, "RETURN date('2015-07-21'), date('2015-07-22')");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns, vec!["date(...)", "date(...)"]);
}

#[test]
fn merge_two_hop_pattern_errors_at_parse_time() {
    let err = parse("MERGE (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)").unwrap_err();
    assert!(err
        .to_string()
        .to_lowercase()
        .contains("one relationship hop"));
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
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
    );
    let result = run(
        &store,
        "MATCH p = (a:Person {name: 'Alice'})-[]->(b:Person) RETURN p",
    );
    let elems = path_elems(&result.rows[0][0]);
    assert_eq!(elems.len(), 3);
    assert_eq!(node_name(&elems[0]), "Alice");
    assert_eq!(node_name(&elems[2]), "Bob");
}

#[test]
fn shortest_path_finds_the_actual_shortest_not_just_a_path() {
    let store = GraphStore::open_memory().unwrap();
    // Direct 1-hop route.
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Dave'})",
    );
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
    assert_eq!(
        result.rows.len(),
        1,
        "only one Dave -- must not have duplicated it"
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "must pick the 1-hop route, not the 3-hop one"
    );
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
    let stmt =
        parse("MATCH p = shortestPath((a:Person {name: 'Alice'})-[:KNOWS*]-(z:Person)) RETURN p")
            .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("shortestpath"));
}

/// Named-path capture over a *single* variable-length hop -- TCK's
/// Quantifier1-4 [8]/[9], ReturnOrderBy2 [12], Pattern2 [9]. Assembles the
/// path from `expand_variable_row`'s own internally-traversed edge/node
/// sequence (deposited under that hop's own synthesized `rel.var`, see
/// `assemble_path`'s docs), not a plain fixed-hop token.
#[test]
fn named_path_over_a_single_variable_length_hop() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:N {n: 1})-[:KNOWS]->(b:N {n: 2})-[:KNOWS]->(c:N {n: 3})",
    );
    let result = run(
        &store,
        "MATCH (a:N {n: 1}) MATCH p = (a)-[:KNOWS*1..2]->(b) RETURN p, length(p) AS len",
    );
    assert_eq!(result.rows.len(), 2);
    for row in &result.rows {
        match &row[0] {
            Value::Path(elems) => {
                // A path of length L has 2L+1 elements (Node, Edge, Node,
                // ..., Node) -- `nodes.len() == edges.len() + 1`.
                let len = int(&row[1]) as usize;
                assert_eq!(elems.len(), 2 * len + 1);
                assert!(matches!(elems[0], PathElem::Node(_)));
                assert!(matches!(elems[elems.len() - 1], PathElem::Node(_)));
            }
            other => panic!("expected a Path, got {other:?}"),
        }
    }
}

/// Mixing a variable-length hop with another hop in the same named path
/// still isn't supported -- see `validate_named_path_pattern`'s own docs
/// (a pre-existing edge-isomorphism gap one level down makes this unsafe
/// in general, even though path *assembly* itself handles it fine).
#[test]
fn named_path_mixing_variable_length_and_fixed_hops_errors() {
    let err = parse("MATCH p = (a)-[:KNOWS*0..1]->(b)-[:FRIEND*0..1]->(c) RETURN p").unwrap_err();
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
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN list[0], list[2]",
    );
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
    let result = run(
        &store,
        "WITH [1, 2, 3] AS list RETURN [x IN list | x * 2] AS y",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 4, 6]);
}

#[test]
fn list_comprehension_filter_only() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN [x IN list WHERE x > 2] AS y",
    );
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
    let result = run(
        &store,
        "RETURN all(x IN [1, 2, 3] WHERE x > 0) AS a, all(x IN [1, 2, 3] WHERE x > 1) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_any_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN any(x IN [1, 2, 3] WHERE x > 2) AS a, any(x IN [1, 2, 3] WHERE x > 5) AS b",
    );
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

    let any = run(
        &store,
        "RETURN any(x IN [null] WHERE x = 2) AS a, any(x IN [2, null] WHERE x = 2) AS b",
    );
    assert!(matches!(any.rows[0][0], Value::Null));
    assert!(bool_val(&any.rows[0][1]));

    let none = run(
        &store,
        "RETURN none(x IN [null] WHERE x = 2) AS a, none(x IN [2, null] WHERE x = 2) AS b",
    );
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
    let result = run(
        &store,
        "RETURN 1 = 1 AS a, 1 < 2 AS b, 2 > 3 AS c, 'ab' STARTS WITH 'a' AS d",
    );
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
    let result = run(
        &store,
        "MATCH (n:Item) WITH n.idx AS y WHERE y > 10 RETURN y ORDER BY y",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 20);
}

/// Real Cypher accepts `ASCENDING`/`DESCENDING` as full-word spellings of
/// `ASC`/`DESC` -- the grammar's `sort_dir` rule tried `^"ASC"` before
/// `^"ASCENDING"`, so it matched just the `ASC` prefix and left `ENDING`
/// dangling as a syntax error (longest alternative must come first in a
/// pest `|` alternation).
#[test]
fn order_by_accepts_ascending_and_descending_spellings() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "UNWIND [3, 1, 2] AS x CREATE (:Item {idx: x})");

    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx ORDER BY n.idx ASCENDING",
    );
    let vals: Vec<i64> = result.rows.iter().map(|r| int(&r[0])).collect();
    assert_eq!(vals, vec![1, 2, 3]);

    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx ORDER BY n.idx DESCENDING",
    );
    let vals: Vec<i64> = result.rows.iter().map(|r| int(&r[0])).collect();
    assert_eq!(vals, vec![3, 2, 1]);
}

/// `WITH DISTINCT x` -- dedups the projected rows, same as `RETURN
/// DISTINCT`, applied before `ORDER BY`/`LIMIT` see the (now-deduped) rows.
#[test]
fn with_distinct_dedups_projected_rows_before_order_by_and_limit() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(
        &store,
        "UNWIND [0, 2, 1, 2, 0, 1] AS x WITH DISTINCT x ORDER BY x ASC LIMIT 1 RETURN x",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 0);

    let result = run(
        &store,
        "UNWIND [0, 2, 1, 2, 0, 1] AS x WITH DISTINCT x ORDER BY x DESC LIMIT 1 RETURN x",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 2);
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
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx = 1 OR n.idx = 2 AS x, n.idx ORDER BY n.idx DESC",
    );
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
    let result = run(
        &store,
        "RETURN [1, 2] = [1, 2] AS a, [1, 2] = [1, 3] AS b, [null] = [1] AS c",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
}

#[test]
fn list_ordering_is_lexicographic() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN [1, 0] >= [1] AS a, [1, null] >= [1] AS b, [1, 2] >= [1, null] AS c",
    );
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
    let result = run(
        &store,
        "RETURN (1 = 'a') AS a, (1 <> 'a') AS b, ('1.0' < 1.0) AS c, ('abc' STARTS WITH true) AS d",
    );
    assert!(!bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
    assert!(matches!(result.rows[0][3], Value::Null));
}

#[test]
fn is_null_pattern_where_finds_missing_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person)");
    let result = run(&store, "MATCH (n:Person) WHERE n.name IS NULL RETURN n");
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn is_not_null_pattern_where_excludes_missing_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person)");
    let result = run(
        &store,
        "MATCH (n:Person) WHERE n.name IS NOT NULL RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn is_null_return_expr() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN null IS NULL AS a, 1 IS NULL AS b, 1 IS NOT NULL AS c",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
    assert!(bool_val(&result.rows[0][2]));
}

#[test]
fn chained_comparisons_fold_into_and() {
    // `1 < x < 10` means `(1 < x) AND (x < 10)`, not `(1 < x) < 10`.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH 5 AS x RETURN 1 < x < 10 AS a, 1 < x < 3 AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

/// `IS [NOT] NULL` binds *tighter* than a surrounding comparison -- `false
/// = true IS NULL` is `false = (true IS NULL)`, not `(false = true) IS
/// NULL`. Real Cypher's own precedence rule (TCK's Precedence1 [8]/[23]).
#[test]
fn is_null_binds_tighter_than_comparison() {
    let store = GraphStore::open_memory().unwrap();
    // true IS NULL == false, so false = false == true.
    let result = run(&store, "RETURN false = true IS NULL AS a");
    assert!(bool_val(&result.rows[0][0]));

    // Both sides describe the same precedence via different groupings --
    // must agree regardless of operator or null-ness.
    let result = run(
        &store,
        "WITH 1 AS a, null AS b \
         RETURN (a = b IS NULL) = (a = (b IS NULL)) AS eq",
    );
    assert!(bool_val(&result.rows[0][0]));
}

/// `x IN list` -- real Cypher's list membership test, previously
/// unsupported as a general expression (only existed inside a list
/// comprehension/quantifier's own `filter_expr`). Three-valued like `=`,
/// and binds *tighter* than a surrounding comparison, same precedence
/// tier as `IS NULL` (TCK's Precedence3 [6]: `[1,2] = [3,4] IN
/// [[3,4],false]` is `[1,2] = ([3,4] IN [[3,4],false])`).
#[test]
fn in_operator_list_membership_and_precedence() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "RETURN 3 IN [1, 2, 3] AS r");
    assert!(bool_val(&result.rows[0][0]));

    let result = run(&store, "RETURN 3 IN [1, 2, 3][0..2] AS r");
    assert!(!bool_val(&result.rows[0][0]));

    // Binds tighter than `=`: `[3,4] IN [...]` is `true`, so the whole
    // thing is `[1,2] = true`, which is `false` (never `[1,2] = [3,4]`
    // first, which would make the IN operand a bool, nonsensical).
    let result = run(&store, "RETURN [1, 2] = [3, 4] IN [[3, 4], false] AS a");
    assert!(!bool_val(&result.rows[0][0]));

    // null propagation: an empty list is always definite false regardless
    // of the needle's nullness; a null element only makes the result
    // unknown when no earlier element definitely matched.
    assert!(matches!(
        run(&store, "RETURN null IN [1, 2] AS r").rows[0][0],
        Value::Null
    ));
    assert!(!bool_val(&run(&store, "RETURN 1 IN [] AS r").rows[0][0]));
    assert!(matches!(
        run(&store, "RETURN null IN [] AS r").rows[0][0],
        Value::Literal(marsdb_query::Literal::Bool(false))
    ));
    assert!(bool_val(
        &run(&store, "RETURN 1 IN [null, 1] AS r").rows[0][0]
    ));
    assert!(matches!(
        run(&store, "RETURN 1 IN [null, 2] AS r").rows[0][0],
        Value::Null
    ));
}

/// `MATCH ... WHERE n.name IN [x IN labels(b) | toLower(x)]` -- `IN` used
/// directly as a bare WHERE predicate (no comparison operator), needing
/// `general_bare_expr`'s widening from raw `add_expr` to
/// `null_predicate_expr`. Also caught a separate, pre-existing bug this
/// surfaced: `labels()`/`keys()` were both mis-typed as `Kind::Scalar` in
/// semantic inference (they return a *list* of strings in real Cypher),
/// wrongly rejecting `[x IN labels(n) | ...]`'s otherwise-valid list
/// comprehension source.
#[test]
fn in_as_bare_where_predicate_with_labels_list_comprehension() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A {name: 'c'})-[:T]->(:B), (a)-[:T]->(:C)",
    );

    let result = run(
        &store,
        "MATCH (n)-->(b) WHERE n.name IN [x IN labels(b) | toLower(x)] RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "c"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `CREATE (n {prop: null})` never actually stores `prop` at all in real
/// Cypher -- the same "setting to null removes/never-creates the
/// property" rule `SET n.prop = null` already had, but `CREATE`'s own
/// inline `{...}` map never applied it. Observable via `keys()` (TCK's
/// Graph8 [8]): a stored `PropertyValue::Null` still shows up as a key,
/// where a real missing property wouldn't.
#[test]
fn create_inline_null_property_is_never_stored() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({exists: 42, missing: null})");

    let result = run(
        &store,
        "MATCH (n) RETURN 'exists' IN keys(n) AS a, 'missing' IN keys(n) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

/// `rand()` -- a fresh pseudo-random float in `[0, 1)` on every call, no
/// memoization (unlike `now()`/`date()`'s per-query `NowSnapshot`).
#[test]
fn rand_returns_a_fresh_value_in_zero_one_each_call() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "UNWIND range(0, 9) AS i RETURN rand() AS r");
    let vals: Vec<f64> = result.rows.iter().map(|row| as_float(&row[0])).collect();
    for v in &vals {
        assert!((0.0..1.0).contains(v), "rand() out of range: {v}");
    }
    // Vanishingly unlikely all 10 calls collide if it's actually random.
    let distinct: std::collections::HashSet<u64> = vals.iter().map(|v| v.to_bits()).collect();
    assert!(
        distinct.len() > 1,
        "rand() returned the same value every call"
    );
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

/// `+` is also real Cypher's list concatenation/append/prepend operator
/// (`[1,2] + [3]` concatenates, `[1,2] + 3`/`3 + [1,2]` appends/prepends
/// the scalar) -- `apply_arith`/`ReturnExpr::Arith`'s semantic check both
/// only ever handled numbers/strings before this, unconditionally
/// rejecting any list operand.
#[test]
fn plus_concatenates_and_appends_lists() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "RETURN [1, 10, 100] + [4, 5] AS foo");
    match &result.rows[0][0] {
        Value::List(items) => {
            let ints: Vec<i64> = items.iter().map(int).collect();
            assert_eq!(ints, vec![1, 10, 100, 4, 5]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    let result = run(&store, "RETURN [false, true] + false AS foo");
    match &result.rows[0][0] {
        Value::List(items) => assert_eq!(items.len(), 3),
        other => panic!("expected a List, got {other:?}"),
    }

    let result = run(&store, "RETURN 0 + [1, 2] AS foo");
    match &result.rows[0][0] {
        Value::List(items) => {
            let ints: Vec<i64> = items.iter().map(int).collect();
            assert_eq!(ints, vec![0, 1, 2]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    // Non-`+` operators must still reject a list at compile time.
    let stmt = parse("RETURN [1, 2] - 1").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("subtracting from a list must be rejected");
    assert!(format!("{err}").contains("cannot use a list"));
}

/// Real Cypher's integer literal grammar has hex (`0x...`) and octal
/// (`0o...`) forms beyond plain decimal, on both a positive and negative
/// literal. Also exercises `i64::MIN`'s magnitude (`2^63`), which doesn't
/// fit in a *positive* `i64` at all -- only `-0x8000000000000000` (the
/// negated form) is representable, needing the two's-complement special
/// case `parse_int_literal` has.
#[test]
fn int_literal_accepts_hex_and_octal_forms() {
    let store = GraphStore::open_memory().unwrap();
    let cases: &[(&str, i64)] = &[
        ("0x1", 1),
        ("0x7FFFFFFFFFFFFFFF", i64::MAX),
        ("-0x1", -1),
        ("-0x8000000000000000", i64::MIN),
        ("0x1a2b3", 0x1a2b3),
        ("0x1A2B3", 0x1a2b3),
        ("0o1", 1),
        ("0o777777777777777777777", i64::MAX),
        ("-0o1", -1),
        ("-0o1000000000000000000000", i64::MIN),
    ];
    for (text, expected) in cases {
        let result = run(&store, &format!("RETURN {text} AS x"));
        assert_eq!(int(&result.rows[0][0]), *expected, "for {text}");
    }
    // A plain decimal literal must stay unaffected.
    assert_eq!(int(&run(&store, "RETURN 42 AS x").rows[0][0]), 42);
}

/// Real Cypher accepts either quote style for a string literal, not just
/// `'...'` -- and `\uXXXX` (exactly 4 hex digits, a BMP code point) as a
/// string escape, previously unrecognized.
#[test]
fn double_quoted_strings_and_unicode_escapes() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN \"\" AS a, \"hello\" AS b");
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Literal(marsdb_query::Literal::String(a)),
            Value::Literal(marsdb_query::Literal::String(b)),
        ) => {
            assert_eq!(a, "");
            assert_eq!(b, "hello");
        }
        other => panic!("unexpected value {other:?}"),
    }

    let result = run(&store, "RETURN '\\u01FF' AS a");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "\u{1FF}"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Real Cypher's float literal grammar has three shapes beyond plain
/// `digits.digits`: a leading-dot form with no integer part (`.1`), and
/// exponent notation on either form or on a bare integer (`1e9`, `.1e-5`).
/// `float_literal`'s old grammar only accepted `digits.digits`.
#[test]
fn float_literal_accepts_leading_dot_and_exponent_forms() {
    let store = GraphStore::open_memory().unwrap();
    let cases: &[(&str, f64)] = &[
        (".1", 0.1),
        (".0", 0.0),
        ("1e9", 1e9),
        ("1E9", 1e9),
        (".1e9", 0.1e9),
        ("1e-5", 1e-5),
        (".1e-5", 0.1e-5),
    ];
    for (text, expected) in cases {
        let result = run(&store, &format!("RETURN {text} AS x"));
        match &result.rows[0][0] {
            Value::Literal(marsdb_query::Literal::Float(f)) => {
                assert!(
                    (*f - expected).abs() < 1e-15,
                    "{text}: got {f}, expected {expected}"
                );
            }
            other => panic!("{text}: expected a float literal, got {other:?}"),
        }
    }
    // A plain integer must stay an Int, not get swept into the widened
    // float grammar.
    let result = run(&store, "RETURN 42 AS x");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Int(n)) => assert_eq!(*n, 42),
        other => panic!("expected an int literal, got {other:?}"),
    }
}

/// `str::parse::<f64>()` silently returns `f64::INFINITY` for a magnitude
/// beyond f64's representable range instead of erroring -- real Cypher
/// requires this to be a compile-time error, not a silently-produced
/// `inf` literal.
#[test]
fn float_literal_overflow_is_a_syntax_error_not_infinity() {
    let err = marsdb_query::parse("RETURN 1.34E999")
        .expect_err("a float literal beyond f64's range must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("too large"), "unexpected error: {msg}");

    // Within range -- must still parse fine.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN 1.23456789e308 AS x");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Float(f)) => assert!(f.is_finite()),
        other => panic!("expected a finite float literal, got {other:?}"),
    }
}

#[test]
fn list_comprehension_bare_where_now_parses() {
    // Regression: previously `filter_expr`'s WHERE reused WithExpr, which
    // only ever wrapped a single Compare -- a bare boolean value (`WHERE
    // x`/`WHERE true`) failed to parse. Now that boolean logic is a real
    // ReturnExpr, this works.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [true, false, true] AS list RETURN [x IN list WHERE x] AS y",
    );
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
    let result = run(
        &store,
        "RETURN none(x IN [] WHERE true) AS a, none(x IN [] WHERE false) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
}

#[test]
fn list_slice_out_of_range_bounds_clamp_instead_of_null() {
    // Regression guard: unlike single-element indexing, out-of-range slice
    // bounds clamp to [0, len] rather than producing null.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3] AS list RETURN list[-100..100], list[5..10]",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
    assert_eq!(list_ints(&result.rows[0][1]), Vec::<i64>::new());
}

/// `marsdb_graph::TzId` <-> `marsdb_query::temporal::TzId` -- two
/// independent, same-shaped types (`temporal.rs` deliberately doesn't
/// depend on `marsdb_graph`), converted at this test-helper boundary.
fn to_temporal_tz(zone: &marsdb_graph::TzId) -> marsdb_query::temporal::TzId {
    match zone {
        marsdb_graph::TzId::Offset(o) => marsdb_query::temporal::TzId::Offset(*o),
        marsdb_graph::TzId::Named(name) => marsdb_query::temporal::TzId::Named(name.clone()),
    }
}

/// Renders a `Date`/`Duration`/`String` `Value` as text via the same
/// `marsdb_query::temporal` formatting functions the CLI/TCK output paths
/// use, so these tests check the exact ISO-8601 text a user would see,
/// not just the internal `PropertyValue` representation.
fn temporal_str(v: &Value) -> String {
    match v {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
        Value::Property(marsdb_graph::PropertyValue::Date(d)) => {
            marsdb_query::temporal::format_date(*d)
        }
        Value::Property(marsdb_graph::PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => marsdb_query::temporal::format_duration(*months, *days, *seconds, *nanos),
        Value::Property(marsdb_graph::PropertyValue::LocalTime(n)) => {
            marsdb_query::temporal::format_local_time(*n)
        }
        Value::Property(marsdb_graph::PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => marsdb_query::temporal::format_time(*nanos_of_day, *offset_seconds),
        Value::Property(marsdb_graph::PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => marsdb_query::temporal::format_local_date_time(*epoch_seconds, *nanos),
        Value::Property(marsdb_graph::PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => {
            marsdb_query::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone))
        }
        other => panic!("expected a temporal/String value, got {other:?}"),
    }
}

fn boolean(v: &Value) -> bool {
    match v {
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

// -- Temporal (date/duration) -----------------------------------------
//
// Real shapes pulled directly from the TCK's expressions/temporal
// feature files (Temporal1/2/3/4/5/6/7/8), not synthesized -- see the
// README's "Cypher coverage" section for exactly what's covered and
// what's deliberately out of scope (named time zones like
// 'Europe/Stockholm' -- only a fixed UTC offset is supported).

#[test]
fn date_construct_from_calendar_map() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN date({year: 1984, month: 10, day: 11}) AS d");
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11");
}

/// ISO week-date construction (`{year, week, dayOfWeek}`) -- TCK's
/// Temporal1 [1]. `year` here is the ISO week-numbering year, which can
/// diverge from the resulting date's calendar year near a year boundary.
#[test]
fn date_construct_from_week_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date({year: 1816, week: 1}), date({year: 1818, week: 53}), \
         date({dayOfWeek: 2, year: 1817, week: 1})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1816-01-01");
    assert_eq!(temporal_str(&row[1]), "1818-12-28");
    assert_eq!(temporal_str(&row[2]), "1816-12-31");

    // `week`/`dayOfWeek` default from a `date` base's own weekYear/week/
    // dayOfWeek, same as month/day already default from a base.
    let result = run(
        &store,
        "RETURN date({date: date('1816-12-30'), week: 2, dayOfWeek: 3}), \
         date({date: date('1816-12-31'), week: 2})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1817-01-08");
    assert_eq!(temporal_str(&row[1]), "1817-01-07");
}

/// Ordinal-date (`{year, ordinalDay}`) and quarter-date
/// (`{year, quarter, dayOfQuarter}`) construction -- TCK's Temporal1 [4].
#[test]
fn date_construct_from_ordinal_and_quarter_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date({year: 1984, ordinalDay: 202}), \
         date({year: 1984, quarter: 3, dayOfQuarter: 45}), \
         date({year: 1984, quarter: 3})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1984-07-20");
    assert_eq!(temporal_str(&row[1]), "1984-08-14");
    assert_eq!(temporal_str(&row[2]), "1984-07-01");
}

/// A bare positional temporal argument to `date()`/`localtime()`/
/// `time()`/`localdatetime()` projects the relevant part of a
/// *different* temporal type, same as the equivalent `{date: other}`/
/// `{time: other}`/`{datetime: other}` map form -- TCK's Temporal3
/// [1]/[2]/[3]/[7].
#[test]
fn temporal_constructors_accept_a_cross_type_positional_argument() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 11, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN date(other)",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");

    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(localtime(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:00");

    let result = run(
        &store,
        "WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS other \
         RETURN toString(time(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123Z");

    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(localdatetime(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11T12:00");
}

#[test]
fn date_construct_from_string_forms() {
    let store = GraphStore::open_memory().unwrap();
    // Temporal2 scenario [1] -- the plain calendar forms; week-date/
    // ordinal-date forms are covered separately, see
    // date_string_week_and_ordinal_date_forms_parse.
    let result = run(
        &store,
        "RETURN date('2015-07-21'), date('20150721'), date('2015-07'), date('201507'), date('2015')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "2015-07-21");
    assert_eq!(temporal_str(&row[1]), "2015-07-21");
    assert_eq!(temporal_str(&row[2]), "2015-07-01");
    assert_eq!(temporal_str(&row[3]), "2015-07-01");
    assert_eq!(temporal_str(&row[4]), "2015-01-01");
}

/// ISO week-date (`YYYY-Www[-D]`/`YYYYWww[D]`) and ordinal-date
/// (`YYYY-DDD`/`YYYYDDD`) string forms -- TCK's Date2/Date3, real Cypher
/// parses these the same as the equivalent `{week, dayOfWeek}`/
/// `{ordinalDay}` map construction (`temporal::parse_week_or_ordinal_date`).
#[test]
fn date_string_week_and_ordinal_date_forms_parse() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date('2015W302'), date('2015-W30-2'), date('2015W30'), date('2015-W30'), \
         date('2015202'), date('2015-202')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "2015-07-21");
    assert_eq!(temporal_str(&row[1]), "2015-07-21");
    assert_eq!(temporal_str(&row[2]), "2015-07-20");
    assert_eq!(temporal_str(&row[3]), "2015-07-20");
    assert_eq!(temporal_str(&row[4]), "2015-07-21");
    assert_eq!(temporal_str(&row[5]), "2015-07-21");
}

#[test]
fn temporal_constructors_reject_malformed_inputs_and_wrong_arity() {
    let store = GraphStore::open_memory().unwrap();
    for query in [
        "RETURN date('123é4')",
        "RETURN duration('Pgarbage')",
        "RETURN duration('P1Ygarbage')",
        "RETURN date('2020', '2021')",
        "RETURN duration('P1D', 'P2D')",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }
}

#[test]
fn date_map_requires_in_range_integer_fields() {
    let store = GraphStore::open_memory().unwrap();
    for query in [
        "RETURN date({year: 2020.9, month: 1, day: 2})",
        "RETURN date({year: 4294969280, month: 1, day: 1})",
        "RETURN date({year: 2020, month: 4294967297, day: 1})",
        "RETURN date({year: 2020, month: 1, day: 4294967297})",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }
}

#[test]
fn date_comparison() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1980, month: 12, day: 24}) AS x, date({year: 1984, month: 10, day: 11}) AS d \
         RETURN x > d, x < d, x >= d, x <= d, x = d",
    );
    let row = &result.rows[0];
    assert!(!boolean(&row[0]));
    assert!(boolean(&row[1]));
    assert!(!boolean(&row[2]));
    assert!(boolean(&row[3]));
    assert!(!boolean(&row[4]));
}

#[test]
fn date_component_access_via_stored_property() {
    // Temporal5 scenario [1]'s exact shape: construct via CREATE (so the
    // Date round-trips through storage), then access components off a
    // WITH-projected scalar.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Val {date: date({year: 1984, month: 10, day: 11})})",
    );
    let result = run(
        &store,
        "MATCH (v:Val) WITH v.date AS d \
         RETURN d.year, d.quarter, d.month, d.week, d.weekYear, d.day, d.ordinalDay, d.weekDay, d.dayOfQuarter",
    );
    let row = &result.rows[0];
    assert_eq!(int(&row[0]), 1984);
    assert_eq!(int(&row[1]), 4);
    assert_eq!(int(&row[2]), 10);
    assert_eq!(int(&row[3]), 41);
    assert_eq!(int(&row[4]), 1984);
    assert_eq!(int(&row[5]), 11);
    assert_eq!(int(&row[6]), 285);
    assert_eq!(int(&row[7]), 4);
    assert_eq!(int(&row[8]), 11);
}

#[test]
fn duration_construct_from_map_normalizes_and_formats() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN duration({days: 14, hours: 16, minutes: 12}), \
                duration({months: 5, days: 1.5}), \
                duration({months: 0.75}), \
                duration({weeks: 2.5}), \
                duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P14DT16H12M");
    assert_eq!(temporal_str(&row[1]), "P5M1DT12H");
    assert_eq!(temporal_str(&row[2]), "P22DT19H51M49.5S");
    assert_eq!(temporal_str(&row[3]), "P17DT12H");
    assert_eq!(temporal_str(&row[4]), "P12Y5M14DT16H13M10S");
}

#[test]
fn duration_construct_from_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN duration('P14DT16H12M'), duration('P0.75M'), duration('P2.5W')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P14DT16H12M");
    assert_eq!(temporal_str(&row[1]), "P22DT19H51M49.5S");
    assert_eq!(temporal_str(&row[2]), "P17DT12H");
}

#[test]
fn duration_equality_is_component_wise_not_calendar_aware() {
    // Temporal7 scenario [6] -- two durations with the same total months/
    // days/seconds/nanos are equal even if their *inputs* differed
    // (60s + 13m == 70s + 12m), but a different `days` component makes
    // them unequal even when hours "look like" they'd make up the gap.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70}) AS x \
         RETURN x = duration({years: 12, months: 5, days: 14, hours: 16, minutes: 13, seconds: 10}), \
                x = duration({years: 12, months: 5, days: 13, hours: 40, minutes: 13, seconds: 10})",
    );
    let row = &result.rows[0];
    assert!(boolean(&row[0]));
    assert!(!boolean(&row[1]));
}

#[test]
fn date_plus_and_minus_duration() {
    // Temporal8 scenario [1] row 1.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Duration {dur: duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, \
         seconds: 70, nanoseconds: 2})})",
    );
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS x \
         MATCH (d:Duration) RETURN x + d.dur AS sum, x - d.dur AS diff",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1997-03-25");
    assert_eq!(temporal_str(&row[1]), "1972-04-27");
}

#[test]
fn date_plus_duration_with_fractional_components_carries_extra_day() {
    // Regression guard for the bug an earlier version of
    // `add_duration_to_date` had: dropping a duration's `seconds`/`nanos`
    // remainder outright instead of folding any *whole* extra day out of
    // it. Temporal8 scenario [1] row 3.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Duration {dur: duration({years: 12.5, months: 5.5, days: 14.5, hours: 16.5, \
         minutes: 12.5, seconds: 70.5, nanoseconds: 3})})",
    );
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS x \
         MATCH (d:Duration) RETURN x + d.dur AS sum, x - d.dur AS diff",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1997-10-11");
    assert_eq!(temporal_str(&row[1]), "1971-10-12");
}

#[test]
fn duration_plus_minus_scale() {
    // Temporal8 scenarios [6]/[7].
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70, nanoseconds: 1}) AS x \
         RETURN x + x, x - x, x * 2, x / 2",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P24Y10M28DT32H26M20.000000002S");
    assert_eq!(temporal_str(&row[1]), "PT0S");
    assert_eq!(temporal_str(&row[2]), "P24Y10M28DT32H26M20.000000002S");
    assert_eq!(temporal_str(&row[3]), "P6Y2M22DT13H21M8S");
}

#[test]
fn temporal_arithmetic_overflow_returns_errors_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    let cases = [
        "RETURN duration({months: 9223372036854775807}) + duration({months: 1})",
        "RETURN duration({months: 9223372036854775807}) - duration({months: -1})",
        "RETURN date('9999-12-31') + duration({days: 9223372036854775807})",
        "RETURN date('9999-12-31') - duration({days: -9223372036854775808})",
    ];

    for cypher in cases {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().contains("overflow") || err.to_string().contains("out-of-range"),
            "unexpected error for {cypher:?}: {err}"
        );
    }
}

#[test]
fn duration_component_access() {
    // Temporal5 scenario [7].
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Val {date: duration({years: 1, months: 4, days: 10, hours: 1, minutes: 1, seconds: 1, \
         nanoseconds: 111111111})})",
    );
    let result = run(
        &store,
        "MATCH (v:Val) WITH v.date AS d \
         RETURN d.years, d.quarters, d.months, d.weeks, d.days, d.hours, d.minutes, d.seconds, \
                d.milliseconds, d.microseconds, d.nanoseconds",
    );
    let row = &result.rows[0];
    assert_eq!(int(&row[0]), 1);
    assert_eq!(int(&row[1]), 5);
    assert_eq!(int(&row[2]), 16);
    assert_eq!(int(&row[3]), 1);
    assert_eq!(int(&row[4]), 10);
    assert_eq!(int(&row[5]), 1);
    assert_eq!(int(&row[6]), 61);
    assert_eq!(int(&row[7]), 3661);
    assert_eq!(int(&row[8]), 3_661_111);
    assert_eq!(int(&row[9]), 3_661_111_111);
    assert_eq!(int(&row[10]), 3_661_111_111_111);
}

#[test]
fn to_string_and_round_trip() {
    // Temporal6 scenarios [1]/[6].
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS d \
         RETURN toString(d), date(toString(d)) = d",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1984-10-11");
    assert!(boolean(&row[1]));

    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: -14, hours: 16}) AS d \
         RETURN toString(d), duration(toString(d)) = d",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P12Y5M-14DT16H");
    assert!(boolean(&row[1]));
}

#[test]
fn to_string_rejects_invalid_types() {
    // TypeConversion4 scenario [10]'s five examples: list, map, node,
    // relationship, and path values are runtime type errors, not null.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n)-[:T]->(m)");
    for query in [
        "RETURN toString([])",
        "RETURN toString({})",
        "MATCH (n) RETURN toString(n)",
        "MATCH ()-[r:T]->() RETURN toString(r)",
        "MATCH p = ()-[:T]->() RETURN toString(p)",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }

    let result = run(&store, "RETURN toString(null)");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn stored_date_survives_the_storage_round_trip() {
    // Temporal4 scenario [1] -- a Date stored as a node property comes
    // back as the same Date (not degraded to a plain Int/String), the
    // real reason PropertyValue got a first-class Date variant instead of
    // reusing Int/String -- see PropertyValue's own doc comment.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE ({created: date({year: 1984, month: 10, day: 11})})",
    );
    let result = run(&store, "MATCH (n) RETURN n.created");
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11");
}

// -- Temporal: LocalTime/Time/LocalDateTime/DateTime -------------------
//
// Real shapes pulled directly from the TCK's Temporal1/2/5/7/8 feature
// files. Scope: fixed UTC offsets only (`'+01:00'`) -- named timezones
// (`'Europe/Stockholm'`) are a documented gap, covered by
// `datetime_named_timezone_is_rejected_not_silently_wrong` below.

#[test]
fn local_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123");

    let result = run(&store, "RETURN toString(localtime('21:40:32.142')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32.142");

    // No seconds/fraction given -> none printed (real Cypher's rule).
    let result = run(&store, "RETURN toString(localtime('21:40')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40");
}

#[test]
fn time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(time({hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123+01:00");

    let result = run(&store, "RETURN toString(time('21:40:32.142+0100')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32.142+01:00");

    // Zero offset prints as `Z`, not `+00:00` -- Temporal2 [3].
    let result = run(&store, "RETURN toString(time('2140-00:00')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40Z");
}

/// A `time()` string argument with no offset defaults to UTC (`Z`),
/// matching real Cypher's "statement default time zone" fallback rather
/// than erroring -- TCK's Temporal10, `time('14:30')`.
#[test]
fn time_string_with_no_offset_defaults_to_utc() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN toString(time('21:40:32'))");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32Z");
}

#[test]
fn local_date_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localdatetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11T12:31:14");

    let result = run(
        &store,
        "RETURN toString(localdatetime('2015-07-21T21:40:32.142')) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "2015-07-21T21:40:32.142");
}

#[test]
fn date_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'})) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14+01:00"
    );

    let result = run(
        &store,
        "RETURN toString(datetime('2015-07-21T21:40:32.142+0100')) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "2015-07-21T21:40:32.142+01:00"
    );
}

#[test]
fn datetime_named_timezone_construction_and_parsing() {
    let store = GraphStore::open_memory().unwrap();
    // Map construction, string parsing (with and without an explicit
    // offset alongside the bracket), and DST-aware resolution (October
    // = standard time, ordinalDay 202 = July = summer time) -- TCK's
    // Temporal1 [10] / Temporal2 [6].
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, \
         second: 14, nanosecond: 645876123, timezone: 'Europe/Stockholm'})), \
         toString(datetime({year: 1984, ordinalDay: 202, hour: 12, minute: 31, second: 14, \
         nanosecond: 645876123, timezone: 'Europe/Stockholm'})), \
         toString(datetime('2015-07-21T21:40:32.142+02:00[Europe/Stockholm]')), \
         toString(datetime('2015-07-21T21:40:32.142[Europe/London]'))",
    );
    let row = &result.rows[0];
    assert_eq!(
        temporal_str(&row[0]),
        "1984-10-11T12:31:14.645876123+01:00[Europe/Stockholm]"
    );
    assert_eq!(
        temporal_str(&row[1]),
        "1984-07-20T12:31:14.645876123+02:00[Europe/Stockholm]"
    );
    assert_eq!(
        temporal_str(&row[2]),
        "2015-07-21T21:40:32.142+02:00[Europe/Stockholm]"
    );
    // No explicit offset at all -- derived purely from the zone (BST).
    assert_eq!(
        temporal_str(&row[3]),
        "2015-07-21T21:40:32.142+01:00[Europe/London]"
    );

    // `.timezone` is the zone name; `.offset` is the resolved offset --
    // the two diverge only for a `Named` zone. TCK's Temporal5 [6].
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 11, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS d \
         RETURN d.timezone, d.offset",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "Europe/Stockholm");
    assert_eq!(temporal_str(&result.rows[0][1]), "+01:00");
}

/// `TIME` has no calendar date, so a named zone's DST-dependent offset
/// has nothing to resolve against -- unlike `DATETIME`, it still only
/// accepts a fixed UTC offset. A real, deliberately narrow scope line,
/// not a silent wrong answer.
#[test]
fn time_named_timezone_is_rejected_not_silently_wrong() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN time('21:40:32.142[Europe/Stockholm]')").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("named timezone"),
        "expected a named-timezone error, got: {err}"
    );

    let stmt2 = parse("RETURN time({hour: 12, timezone: 'Europe/Stockholm'})").unwrap();
    let err2 = Executor::new(&store).execute(&stmt2).unwrap_err();
    assert!(
        err2.to_string().contains("named timezone"),
        "expected a named-timezone error, got: {err2}"
    );
}

/// `time({time: namedZoneDateTime})` -- no *explicit* `timezone` key, the
/// zone was just carried through from the projected base -- silently
/// degrades to the resolved offset instead of erroring, unlike an
/// explicit named-zone request. TCK's Temporal3 [3] row 125.
#[test]
fn time_projected_from_a_named_zone_base_degrades_to_plain_offset() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS other \
         RETURN toString(time({time: other})), toString(time(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:00+01:00");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:00+01:00");
}

/// Projecting a `Named`-zone base with an *explicit* `timezone` override
/// shifts the wall-clock to preserve the same instant -- the target
/// offset is resolved *for the actual target date* (which the `day`
/// override can move to a different DST period than the base's own
/// date), not the base's original instant. A real bug found and fixed
/// this session: an earlier version resolved both the "from" and "to"
/// offsets against stale/inconsistent dates, producing wrong instants.
/// TCK's Temporal3 [9]/[10] (a representative sample of the row shapes).
#[test]
fn datetime_shift_into_named_zone_resolves_offsets_for_the_target_date() {
    let store = GraphStore::open_memory().unwrap();
    // Time-with-offset base, fresh year/month/day, explicit shift.
    let result = run(
        &store,
        "WITH time({hour: 12, minute: 31, second: 14, microsecond: 645876, timezone: '+01:00'}) AS other \
         RETURN toString(datetime({year: 1984, month: 10, day: 11, time: other, second: 42, \
         timezone: 'Pacific/Honolulu'}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T01:31:42.645876-10:00[Pacific/Honolulu]"
    );
    // Named-zone-base shifted into a *different* named zone, where the
    // `day` override moves the result across a DST boundary for the
    // *base's own* zone too (Stockholm: standard time in October, but
    // summer time by the overridden March 28) -- the "from" offset used
    // for the shift must reflect the *target* date, not the base's
    // original (October) instant.
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS otherDate, \
         datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS otherTime \
         RETURN toString(datetime({date: otherDate, time: otherTime, day: 28, second: 42, \
         timezone: 'Pacific/Honolulu'}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-03-28T00:00:42-10:00[Pacific/Honolulu]"
    );
}

/// With *no* explicit `timezone` override, a `Named`-zone base's zone
/// identity is preserved as-is and the wall-clock is *not* shifted, even
/// if a `day` override moves the result across a DST boundary for that
/// same zone -- the displayed offset is simply re-resolved for the new
/// date, the local time itself never changes. A real bug found and fixed
/// this session: an earlier version always re-resolved and shifted
/// whenever the zone's real offset differed for the new date, even
/// without an explicit override. TCK's Temporal3 [10] rows 336/337.
#[test]
fn datetime_no_override_preserves_zone_identity_without_shifting() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS otherDate, \
         datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS otherTime \
         RETURN toString(datetime({date: otherDate, time: otherTime, day: 28, second: 42}))",
    );
    // Same 12:00 wall-clock as the base, just re-displayed with the
    // zone's real (now summer-time) offset for the new date -- not
    // shifted to a different hour.
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-03-28T12:00:42+02:00[Europe/Stockholm]"
    );
}

/// `datetime(otherLocalDateTime)` -- a bare `LocalDateTime` argument has
/// no zone of its own, defaults to UTC -- TCK's Temporal3 [11].
#[test]
fn datetime_construct_from_bare_local_date_time_defaults_to_utc() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS other \
         RETURN toString(datetime(other)), toString(datetime({datetime: other}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-03-07T12:31:14.645Z");
    assert_eq!(temporal_str(&result.rows[0][1]), "1984-03-07T12:31:14.645Z");
}

/// `Time`'s comparison is by the UTC-equivalent instant-of-day, not the
/// raw wall-clock reading -- Temporal7 [3].
#[test]
fn time_comparison_is_instant_based_not_wall_clock() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH time({hour: 10, minute: 0, timezone: '+01:00'}) AS x, \
              time({hour: 9, minute: 35, second: 14, nanosecond: 645876123, timezone: '+00:00'}) AS d \
         RETURN x > d, x < d, x >= d, x <= d, x = d",
    );
    let bools: Vec<bool> = result.rows[0].iter().map(boolean).collect();
    assert_eq!(bools, vec![false, true, false, true, false]);
}

/// Two `DateTime`s at the same instant but different offsets are equal
/// -- real Cypher's rule (see `PropertyValue::DateTime`'s doc comment),
/// not the derived structural equality every other `PropertyValue`
/// variant gets.
#[test]
fn date_time_equality_is_instant_based_not_structural() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 2020, month: 1, day: 1, hour: 1, minute: 0, second: 0, timezone: '+01:00'}) AS x, \
              datetime({year: 2020, month: 1, day: 1, hour: 0, minute: 0, second: 0, timezone: '+00:00'}) AS d \
         RETURN x = d",
    );
    assert!(boolean(&result.rows[0][0]));
}

/// `DateTime`'s calendar/clock component access (`.hour`, `.day`, ...)
/// reflects the *local* (offset-adjusted) wall-clock reading that was
/// written, not the underlying UTC instant -- `epochSeconds`/
/// `epochMillis` are the one exception, always UTC. Temporal5 [4].
#[test]
fn date_time_component_access_uses_local_reading_except_epoch_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}) AS d \
         RETURN d.year, d.month, d.day, d.hour, d.minute, d.second, d.timezone, d.offset, d.offsetSeconds, d.offsetMinutes",
    );
    let ints: Vec<i64> = [0usize, 1, 2, 3, 4, 5]
        .iter()
        .map(|&i| match &result.rows[0][i] {
            Value::Property(marsdb_graph::PropertyValue::Int(n)) => *n,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(ints, vec![1984, 10, 11, 12, 31, 14]);
    assert_eq!(temporal_str(&result.rows[0][6]), "+01:00");
    assert_eq!(temporal_str(&result.rows[0][7]), "+01:00");
}

/// `date({date: other, ...overrides})` -- projects year/month/day from
/// another temporal value, individual keys override on top. Temporal3
/// [1].
#[test]
fn date_projects_from_another_temporal_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 11, day: 11}) AS other \
         RETURN toString(date({date: other})), toString(date({date: other, year: 28})), \
                toString(date({date: other, day: 28}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");
    assert_eq!(temporal_str(&result.rows[0][1]), "0028-11-11");
    assert_eq!(temporal_str(&result.rows[0][2]), "1984-11-28");

    // Projects from LocalDateTime/DateTime too, not just Date.
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, month: 11, day: 11, hour: 12}) AS other RETURN toString(date({date: other}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");
}

/// `localtime({time: other, ...overrides})` -- Temporal3 [2].
#[test]
fn local_time_projects_from_another_temporal_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS other \
         RETURN toString(localtime({time: other})), toString(localtime({time: other, second: 42}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:31:42.645876123");
}

/// `time({time: other, timezone: ...})` -- when the override timezone
/// differs from the base's own offset, the wall-clock shifts to
/// preserve the same instant (real Cypher's rule, not just relabeling
/// the offset) -- Temporal3 [3].
#[test]
fn time_projection_with_different_timezone_shifts_wall_clock_to_preserve_instant() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH time({hour: 12, minute: 31, second: 14, microsecond: 645876, timezone: '+01:00'}) AS other \
         RETURN toString(time({time: other})), toString(time({time: other, timezone: '+05:00'}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876+01:00");
    assert_eq!(temporal_str(&result.rows[0][1]), "16:31:14.645876+05:00");

    // An explicit field override applies *after* the zone shift, not
    // before -- Temporal3 [3]'s own compound example.
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(time({time: other, second: 42, timezone: '+05:00'}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "16:00:42+05:00");
}

/// `localdatetime({date: ..., time: ..., ...overrides})` -- combining a
/// date projected from one value and a time from another (or literal
/// fields), Temporal3 [4]/[5]/[6].
#[test]
fn local_date_time_projects_date_and_time_independently() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS d, \
              localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS t \
         RETURN toString(localdatetime({date: d, time: t})), \
                toString(localdatetime({date: d, time: t, day: 28, second: 42})), \
                toString(localdatetime({date: d, hour: 10, minute: 10, second: 10}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14.645876123"
    );
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-28T12:31:42.645876123"
    );
    assert_eq!(temporal_str(&result.rows[0][2]), "1984-10-11T10:10:10");
}

/// `datetime(...) + duration(...)` -- real calendar month arithmetic on
/// the *local* reading, seconds/nanos carrying across day boundaries
/// (unlike `Date`, which has no time-of-day to carry into). Temporal8.
#[test]
fn date_time_plus_duration() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}) \
                          + duration({months: 1, days: 5, hours: 2})) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-11-16T14:31:14+01:00"
    );
}

/// `Time`/`LocalTime` + `Duration` wraps at the 24h boundary -- there's
/// no calendar to carry an extra day into.
#[test]
fn time_plus_duration_wraps_at_midnight() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(time({hour: 23, minute: 0, timezone: 'Z'}) + duration({hours: 2})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "01:00Z");
}

/// `duration.between(a, b)` -- real calendar month arithmetic plus a
/// day/second/nanos remainder, mixing every pair of the 5 non-Duration
/// temporal types. Temporal10 [1]/[2].
#[test]
fn duration_between_mixed_types() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.between(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P30Y8M13D");

    // Either side lacking a date degrades to a plain time-of-day
    // difference -- the date side's real calendar date never enters
    // the calculation at all.
    let result = run(
        &store,
        "RETURN toString(duration.between(date('1984-10-11'), localtime('16:30'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT16H30M");

    let result = run(
        &store,
        "RETURN toString(duration.between(localdatetime('2015-07-21T21:40:32.142'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P-27DT-21H-40M-32.142S");
}

/// Two `DateTime`s at *different* offsets -- the month/day/second
/// breakdown must account for the real offset delta, not just the raw
/// local wall-clock digits (found as a real bug: naive local-to-local
/// subtraction here gave `P11M29DT23H59M55.999S` instead of the
/// correct `P1YT59M55.999S`, off by exactly the 1h offset difference).
/// Temporal10 [2].
#[test]
fn duration_between_two_datetimes_with_different_offsets_accounts_for_the_offset_delta() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.between(datetime('2014-07-21T21:40:36.143+0200'), \
                                           datetime('2015-07-21T21:40:32.142+0100'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P1YT59M55.999S");
}

/// The same offset-reconciliation rule applies even in the time-only
/// "degrade" mode (one side has no date) when *both* operands still
/// carry a real offset (`Time`/`DateTime`) -- found as a second real
/// bug alongside the one above.
#[test]
fn duration_between_time_only_mode_still_accounts_for_offset_when_both_sides_have_one() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(datetime('2014-07-21T21:40:36.143+0200'), \
                                             time('16:30+0100'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-4H-10M-36.143S");
}

/// `.inMonths`/`.inDays`/`.inSeconds` collapse the same underlying
/// computation into a single bucket -- `.inMonths` keeps just the
/// calendar month count, `.inDays`/`.inSeconds` discard the month
/// optimization entirely and use the *raw* total elapsed time (so
/// `.inDays` on a date+time target truncates away any sub-day
/// remainder rather than carrying it as leftover seconds). Temporal10
/// [3]/[4]/[5].
#[test]
fn duration_in_months_days_seconds_collapse_to_a_single_bucket() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inMonths(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P30Y8M");

    let result = run(
        &store,
        "RETURN toString(duration.inDays(date('1984-10-11'), localdatetime('2016-07-21T21:45:22.142'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P11606D");

    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT269112H");
}

/// `duration.between`'s own remainder-decomposition edge case: a
/// negative sub-second-only difference must still round-trip through
/// `toString` correctly (a real pre-existing invariant --
/// `format_seconds_fraction`'s `(0, -500_000_000) -> "-0.5"` case --
/// exercised here via the actual `duration.between` code path, not a
/// hand-built `Duration`). Temporal10 [6].
#[test]
fn duration_in_seconds_negative_sub_second_only_difference() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(localdatetime('2014-07-21T21:40:36.143'), \
                                             localdatetime('2014-07-21T21:40:36.142'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-0.001S");
}

/// Every no-arg `date()`/`localtime()`/`time()`/`localdatetime()`/
/// `datetime()` call within *one* query must return the same value --
/// real Cypher's guarantee, and the reason `duration.between(date(),
/// date())` is always exactly `PT0S`, never a few-microseconds-off
/// nonzero duration from two independent `now()` reads (found as a
/// real, if narrow, bug: each call was originally reading `chrono::
/// Utc::now()` fresh). Temporal10 [12].
#[test]
fn repeated_now_calls_within_one_query_return_the_same_instant() {
    let store = GraphStore::open_memory().unwrap();
    for value in [
        "localtime()",
        "time()",
        "date()",
        "localdatetime()",
        "datetime()",
    ] {
        let result = run(
            &store,
            &format!("RETURN toString(duration.inSeconds({value}, {value})) AS r"),
        );
        assert_eq!(
            temporal_str(&result.rows[0][0]),
            "PT0S",
            "{value} called twice in one query must be exactly PT0S"
        );
    }
}

/// `date.truncate(unit, value, map)` -- calendar-unit truncation
/// (`millennium`/`century`/`decade`/`year`/`quarter`/`month`/`week`/
/// `weekYear`/`day`), plus optional field overrides applied after
/// truncation. Temporal9 [1].
#[test]
fn date_truncate_calendar_units() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('millennium', date({year: 2017, month: 10, day: 11}), {day: 2})), \
                toString(date.truncate('century', date({year: 1984, month: 10, day: 11}), {})), \
                toString(date.truncate('decade', date({year: 1984, month: 10, day: 11}), {})), \
                toString(date.truncate('quarter', date({year: 1984, month: 11, day: 11}), {})), \
                toString(date.truncate('week', date({year: 1984, month: 10, day: 11}), {}))",
    );
    let strs: Vec<String> = (0..5).map(|i| temporal_str(&result.rows[0][i])).collect();
    assert_eq!(
        strs,
        vec![
            "2000-01-02",
            "1900-01-01",
            "1980-01-01",
            "1984-10-01",
            "1984-10-08"
        ]
    );
}

/// `weekYear` truncation crosses a real ISO week-year boundary (Jan 1
/// 1984 belongs to ISO week-year 1983) -- Temporal9 [1].
#[test]
fn date_truncate_week_year_crosses_iso_boundary() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('weekYear', datetime({year: 1984, month: 1, day: 1, hour: 12, timezone: '+01:00'}), {})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1983-01-03");
}

/// `datetime.truncate`/`localdatetime.truncate` -- a calendar-scale
/// unit truncates the date *and* resets the time to midnight; a
/// clock-scale unit leaves the date untouched. Temporal9 [2]/[3].
#[test]
fn date_time_truncate_calendar_vs_clock_units() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime.truncate('millennium', datetime({year: 2017, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}), {day: 2})), \
                toString(localdatetime.truncate('hour', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "2000-01-02T00:00+01:00");
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-11T12:00:00.000000002"
    );
}

/// `localtime.truncate`/`time.truncate` -- clock-only truncation, `time.
/// truncate` inherits the source's offset unless overridden. Temporal9
/// [4]/[5].
#[test]
fn local_time_and_time_truncate() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localtime.truncate('day', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2})), \
                toString(time.truncate('hour', time({hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "00:00:00.000000002");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:00+01:00");
}

/// `date.truncate('week', d, {dayOfWeek: N})` -- the `dayOfWeek`
/// override moves within the truncated result's own ISO week (found
/// as a real bug: `apply_date_overrides` didn't recognize `dayOfWeek`
/// at all and silently ignored it instead of applying it or erroring).
/// Temporal9 [1].
#[test]
fn date_truncate_day_of_week_override() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('week', date({year: 1984, month: 10, day: 11}), {dayOfWeek: 2})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-09");
}

/// A `.truncate()` map with a field the target type has no slot for
/// (`hour` on a `date.truncate` result, which is a bare `Date`) is a
/// real error, not silently ignored.
#[test]
fn date_truncate_rejects_a_time_only_override_field() {
    let store = GraphStore::open_memory().unwrap();
    let stmt =
        parse("RETURN date.truncate('year', date({year: 1984, month: 10, day: 11}), {hour: 5})")
            .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("unrecognized field"),
        "expected an unrecognized-field error, got: {err}"
    );
}

/// A `.truncate()` map overriding *only* `nanosecond` must keep the
/// truncated base's own millisecond/microsecond digits, not silently
/// reset them to zero -- found as a real bug (`{nanosecond: 2}` alone
/// was dropping the base's `.645` millisecond value entirely instead
/// of producing `.645000002`). Temporal9 [2]-[5].
#[test]
fn truncate_sub_second_override_keeps_the_bases_other_digits() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localdatetime.truncate('millisecond', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2})), \
                toString(localdatetime.truncate('microsecond', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14.645000002"
    );
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-11T12:31:14.645876002"
    );
}

#[test]
fn stored_time_and_date_time_survive_the_storage_round_trip() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE ({t: time({hour: 9, minute: 0, timezone: '+02:00'})})",
    );
    let result = run(&store, "MATCH (n) RETURN n.t");
    assert_eq!(temporal_str(&result.rows[0][0]), "09:00+02:00");
}

#[test]
fn create_with_list_property_round_trips_as_a_real_list() {
    // A list-valued property (`PropertyValue::List`, real Cypher/Neo4j's
    // own "homogeneous array property" shape) is storable -- and reads
    // back as a genuine `Value::List`, not an opaque
    // `Value::Property(PropertyValue::List(_))`, so every existing list
    // operation (indexing, `size()`, `IN`, `UNWIND`, ...) works
    // transparently on it, the same as a list literal/`collect()` result.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n {tags: [1, 2, 3]})");
    let result = run(&store, "MATCH (n) RETURN n.tags");
    let Value::List(items) = &result.rows[0][0] else {
        panic!("expected a list, got {:?}", result.rows[0][0]);
    };
    let ints: Vec<i64> = items.iter().map(int).collect();
    assert_eq!(ints, vec![1, 2, 3]);

    let indexed = run(&store, "MATCH (n) RETURN n.tags[1], size(n.tags)");
    assert_eq!(int(&indexed.rows[0][0]), 2);
    assert_eq!(int(&indexed.rows[0][1]), 3);

    let contains = run(&store, "MATCH (n) WHERE 2 IN n.tags RETURN n");
    assert_eq!(contains.rows.len(), 1);
}

/// Regression guard: a genuinely unstorable property shape (a map, a
/// node, an edge, a path) must still be a clear error, not a wrong
/// answer -- widening `value_to_storable_property` to accept `Value::
/// List` must not have widened it to accept everything.
#[test]
fn create_with_unsupported_map_property_errors_clearly_not_silently_nulls() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (n {tags: {a: 1}})").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("property"),
        "expected a clear error, got: {err}"
    );
}

/// `SET n = {...}` replaces every existing property (removing any key
/// not in the map); `SET n += {...}` merges instead (only the map's own
/// keys change, a `null` value removes just that key, everything else on
/// `n` stays untouched) -- TCK's Set4/Set5. Both also accept a bound
/// node/relationship as the RHS, not just a map literal (`SET r = a`
/// copies `a`'s own properties -- TCK's Merge6 [6]/Merge7 [4]).
#[test]
fn set_map_assign_replace_and_merge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {name: 'A', name2: 'B'})");

    // `=` replaces everything -- `name2` is gone, `baz` is new.
    let result = run(
        &store,
        "MATCH (n:X {name: 'A'}) SET n = {name: 'B', baz: 'C'} RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert_eq!(node.props.len(), 2);
    assert_eq!(
        node.props.get("name"),
        Some(&marsdb_graph::PropertyValue::String("B".to_string()))
    );
    assert_eq!(
        node.props.get("baz"),
        Some(&marsdb_graph::PropertyValue::String("C".to_string()))
    );
    assert!(!node.props.contains_key("name2"));

    // `+=` merges -- existing untouched keys survive.
    run(&store, "CREATE (:Y {name: 'A'})");
    let result = run(
        &store,
        "MATCH (n:Y {name: 'A'}) SET n += {name2: 'B'} RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert_eq!(node.props.len(), 2);

    // Copying properties from a bound node, not just a map literal.
    run(&store, "CREATE (:P {name: 'A'}), (:Q)");
    run(&store, "MATCH (p:P), (q:Q) SET q = p");
    let result = run(&store, "MATCH (q:Q) RETURN q.name");
    assert_eq!(str_value(&result.rows[0][0]), "A");
}

/// `SET (n).name = 'x'` -- a parenthesized-variable target, same meaning
/// as bare `n.name` (TCK's Set1 [3]/[4]).
#[test]
fn set_parenthesized_target() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)");
    let result = run(&store, "MATCH (n:A) SET (n).name = 'neo4j' RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "neo4j");
}

/// `CREATE (a {...}) SET a.prop = ...` -- CREATE followed directly by
/// another mutating clause, no `WITH` in between at all (TCK's Set1
/// [6]/[7]) -- a real gap `create_as_clause`'s own WITH-only lookahead
/// didn't cover. Also confirms a property-sourced list survives a list
/// concat (`a.numbers + [4, 5]`) and a list comprehension over a
/// property access (`[i IN n.numbers | ...]`, TCK's Set1 [5] -- needs
/// `list_element` to not reject a property access's `Kind::Scalar`
/// outright, the same widening `bind_unwind` already has).
#[test]
fn create_followed_directly_by_set_no_with() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "CREATE (a {numbers: [1, 2, 3]}) SET a.numbers = a.numbers + [4, 5] RETURN a.numbers",
    );
    let ints: Vec<i64> = match &result.rows[0][0] {
        Value::List(items) => items.iter().map(int).collect(),
        other => panic!("expected a list, got {other:?}"),
    };
    assert_eq!(ints, vec![1, 2, 3, 4, 5]);

    run(&store, "CREATE (:N)");
    let result = run(
        &store,
        "MATCH (n:N) SET n.numbers = [1, 2, 3] RETURN [i IN n.numbers | i / 2.0] AS x",
    );
    let floats: Vec<f64> = match &result.rows[0][0] {
        Value::List(items) => items.iter().map(as_float).collect(),
        other => panic!("expected a list, got {other:?}"),
    };
    assert_eq!(floats, vec![0.5, 1.0, 1.5]);

    // The pre-existing `CREATE ... WITH ...` (#110) and plain standalone
    // `CREATE ... RETURN ...`/bare-`CREATE` (#106) shapes must still work
    // unaffected -- create_as_clause's lookahead grew wider, not
    // narrower.
    let result = run(
        &store,
        "CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)",
    );
    assert!(result.rows.is_empty());
    let result = run(&store, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
}

/// Any of `SET`/`DELETE`/`CREATE`/`MERGE` can chain directly into any
/// other one of them, `WITH` or not (TCK's Merge1/Merge5/Merge9, e.g.
/// `CREATE (a), (b) MERGE (a)-[:X]->(b) RETURN count(a)`) -- previously
/// only `WITH` was a valid thing to chain a mutating clause into.
#[test]
fn mutating_clauses_chain_directly_into_each_other_no_with_needed() {
    let store = GraphStore::open_memory().unwrap();

    // CREATE directly into MERGE.
    let result = run(
        &store,
        "CREATE (a), (b) MERGE (a)-[:X]->(b) RETURN count(a)",
    );
    assert_eq!(int(&result.rows[0][0]), 1);

    // DELETE directly into MERGE.
    run(&store, "CREATE (:A {num: 1}), (:A {num: 2})");
    let result = run(&store, "MATCH (a:A) DELETE a MERGE (a2:A) RETURN a2.num");
    assert!(result.rows.iter().all(|row| matches!(row[0], Value::Null)));

    // CREATE, MERGE, and CREATE again, all directly chained.
    let result = run(
        &store,
        "CREATE (a:P), (b:Q) MERGE (a)-[:KNOWS]->(b) CREATE (b)-[:KNOWS]->(c:R) RETURN count(*)",
    );
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// Real Cypher allows `ON MATCH`/`ON CREATE` in either order, not just
/// `ON CREATE` before `ON MATCH` -- a repeated one of the same kind is
/// still a real error (TCK's Merge4).
#[test]
fn merge_on_match_on_create_either_order() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    run(
        &store,
        "MATCH () MERGE (a:L) ON MATCH SET a:M1 ON CREATE SET a:M2",
    );
    let result = run(&store, "MATCH (a:L) RETURN labels(a)");
    let mut labels = list_str_values(&result.rows[0][0]);
    labels.sort();
    assert_eq!(labels, vec!["L".to_string(), "M2".to_string()]);

    // A repeated ON CREATE is rejected at parse time, not execution.
    assert!(parse("MERGE (a:L) ON CREATE SET a:M1 ON CREATE SET a:M2").is_err());
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
    let result = run(
        &store,
        "MATCH (n:A) SET n.property1 = 'updated' RETURN n.property1",
    );
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
    let result = run(
        &store,
        "MATCH (a)-[r]-(b) DELETE r, a, b RETURN count(*) AS c",
    );
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
    assert!(
        err.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err}"
    );
    // A failed statement rolls back its whole write transaction (see
    // `Executor::execute`'s abort-on-error path) -- the delete itself must
    // NOT have taken effect, same as any other error mid-statement.
    let remaining = run(&store, "MATCH (n:A) RETURN n");
    assert_eq!(
        remaining.rows.len(),
        1,
        "a failed statement must roll back, not partially apply its delete"
    );
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
    assert!(
        err.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err}"
    );

    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE ()-[:T {num: 0}]->()");
    let stmt2 = parse("MATCH ()-[r]->() DELETE r RETURN r.num").unwrap();
    let err2 = Executor::new(&store2).execute(&stmt2).unwrap_err();
    assert!(
        err2.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err2}"
    );
}

#[test]
fn detach_delete_then_return() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    let result = run(
        &store,
        "MATCH (n:Person {name: 'Alice'}) DETACH DELETE n RETURN 42 AS num",
    );
    // `42` is a bare literal, not a node/edge property -- eval_return_expr
    // yields Value::Literal here, not Value::Property.
    assert!(matches!(
        &result.rows[0][0],
        Value::Literal(marsdb_query::Literal::Int(42))
    ));
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
            assert_eq!(
                int_value(&Value::Property(node.props.get("p2").unwrap().clone())),
                2
            );
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
    let result = run(
        &store,
        "MATCH (a:A)-[:R]->(x:X) SET a.touched = true RETURN DISTINCT x.tag",
    );
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
    assert!(matches!(
        &result.rows[0][0],
        Value::Literal(marsdb_query::Literal::Int(99))
    ));
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
            assert!(
                !node.props.contains_key("property1"),
                "property1 must be gone, not null: {:?}",
                node.props
            );
            assert_eq!(
                int_value(&Value::Property(
                    node.props.get("property2").unwrap().clone()
                )),
                46
            );
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
    let prop_set = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) SET a.num = 42 RETURN a",
    );
    assert!(matches!(prop_set.rows[0][0], Value::Null));
    let label_set = run(&store, "OPTIONAL MATCH (a:DoesNotExist) SET a:L RETURN a");
    assert!(matches!(label_set.rows[0][0], Value::Null));
    let prop_remove = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) REMOVE a.num RETURN a",
    );
    assert!(matches!(prop_remove.rows[0][0], Value::Null));
    let label_remove = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) REMOVE a:L RETURN a",
    );
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

#[test]
fn create_index_then_lookup_via_index_seek() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE (:Person {email: 'bob@x.com', age: 25})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "MATCH (n:Person {email: 'alice@x.com'}) RETURN n.age",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 30);
}

#[test]
fn create_index_unique_rejects_duplicate() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'same@x.com'})");
    run(&store, "CREATE (:Person {email: 'same@x.com'})");

    let stmt = parse("CREATE INDEX ON :Person(email) UNIQUE").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("unique"));
}

#[test]
fn match_without_declared_index_still_works() {
    // Regression guard: a plain node-pattern-property match with no
    // index declared must still hit the ordinary Filter-over-scan path,
    // not error or silently return nothing just because `apply_index_seeks`
    // now runs over every plan.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com'})");
    let result = run(
        &store,
        "MATCH (n:Person {email: 'alice@x.com'}) RETURN n.email",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "alice@x.com");
}

#[test]
fn where_clause_equality_fuses_into_index_seek() {
    // Unlike an inline pattern property (already covered by
    // create_index_then_lookup_via_index_seek), a WHERE-clause equality
    // compiles to a separate outer Filter -- apply_index_seeks' newer
    // predicate-extraction pass is what finds it.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE (:Person {email: 'bob@x.com', age: 25})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "MATCH (n:Person) WHERE n.email = 'alice@x.com' RETURN n.age",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 30);
}

#[test]
fn index_seek_combined_with_a_non_indexed_conjunct_keeps_the_residual_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 40})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "MATCH (n:Person) WHERE n.email = 'alice@x.com' AND n.age > 35 RETURN n.age",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 40);
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

fn plan_lines(result: &marsdb_query::QueryResult) -> Vec<String> {
    result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("expected an EXPLAIN plan line, got {other:?}"),
        })
        .collect()
}

#[test]
fn explain_shows_index_seek_and_residual_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "EXPLAIN MATCH (n:Person) WHERE n.email = 'alice@x.com' AND n.age > 20 RETURN n",
    );
    let lines = plan_lines(&result);
    assert!(lines.iter().any(|l| l.contains("IndexSeek(n:Person")
        && l.contains("email")
        && l.contains("alice@x.com")));
    assert!(lines.iter().any(|l| l.contains("Filter n.age > 20")));
}

#[test]
fn explain_falls_back_to_scan_when_no_index_declared() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com'})");

    let result = run(
        &store,
        "EXPLAIN MATCH (n:Person {email: 'alice@x.com'}) RETURN n",
    );
    let lines = plan_lines(&result);
    assert!(lines
        .iter()
        .any(|l| l.contains("NodeByLabelScan(n:Person)")));
    assert!(!lines.iter().any(|l| l.contains("IndexSeek")));
}

#[test]
fn explain_never_mutates_even_a_write_statement() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'a'})");

    let explained = run(&store, "EXPLAIN CREATE (:Person {name: 'b'})");
    assert_eq!(plan_lines(&explained).len(), 1);
    assert!(plan_lines(&explained)[0].contains("no query plan"));

    let count = run(&store, "MATCH (n:Person) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 1);
}

#[test]
fn explain_shows_expand_between_two_scans() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'a'})-[:KNOWS]->(:Person {name: 'b'})",
    );

    let result = run(
        &store,
        "EXPLAIN MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a, b",
    );
    let lines = plan_lines(&result);
    assert!(lines.iter().any(|l| l.contains("Expand(a)-[:KNOWS]->(b)")));
    assert!(lines
        .iter()
        .any(|l| l.contains("NodeByLabelScan(a:Person)")));
}

#[test]
fn syntax_error_for_malformed_query_text() {
    // Never reaches planning/execution at all -- pest itself rejects it.
    let err = parse("MATCH (n RETURN n").unwrap_err();
    assert!(err.to_string().starts_with("syntax error:"));
}

#[test]
fn semantic_error_for_structurally_invalid_but_parseable_query() {
    // Parses fine, but references a name never bound anywhere -- caught
    // by the pre-execution semantic pass, not a grammar failure.
    let stmt = parse("RETURN missing").unwrap();
    let store = GraphStore::open_memory().unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn type_error_for_a_runtime_value_shape_mismatch() {
    // The query is syntactically and structurally fine (both operands
    // are "a scalar" as far as the pre-execution semantic pass can tell)
    // -- the mismatch only exists once the actual values are in hand and
    // one turns out to be a bool, not a number.
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 + true").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("type error:"));
}

#[test]
fn bracketless_relationship_arrows_match_the_bracketed_forms() {
    // `-->`/`<--`/`--` are real Cypher's shorthand for an anonymous,
    // untyped, propertyless relationship -- brackets are only needed at
    // all to carry a var/type/range/props. Must behave identically to
    // `-[]->`/`<-[]-`/`-[]-`, not just parse.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {n: 'a'})-[:X]->(:B {n: 'b'})");

    let out = run(&store, "MATCH (a)-->(b) RETURN a.n, b.n");
    assert_eq!(out.rows.len(), 1);
    assert_eq!(str_value(&out.rows[0][0]), "a");
    assert_eq!(str_value(&out.rows[0][1]), "b");

    let in_ = run(&store, "MATCH (b)<--(a) RETURN a.n, b.n");
    assert_eq!(in_.rows.len(), 1);
    assert_eq!(str_value(&in_.rows[0][0]), "a");
    assert_eq!(str_value(&in_.rows[0][1]), "b");

    let either = run(&store, "MATCH (x)--(y) RETURN x.n, y.n");
    assert_eq!(either.rows.len(), 2, "undirected must match both endpoints");
}

#[test]
fn create_with_bracketless_arrow_requires_a_relationship_type() {
    // Unlike MATCH (where an untyped hop just means "any relationship"),
    // CREATE always makes exactly one new relationship -- real Cypher
    // requires an explicit `:TYPE` there, never defaults/infers one, and
    // that's true for `-->` the same as `-[]->` (TCK's Create2 scenario
    // [18], "Fail when creating a relationship without a type").
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (:A)-->(:B)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn create_with_a_typed_relationship_still_works() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:REL]->(:B)");
    let out = run(&store, "MATCH ()-[:REL]->() RETURN count(*)");
    assert_eq!(out.rows.len(), 1);
    assert_eq!(int_value(&out.rows[0][0]), 1);
}

#[test]
fn with_where_compares_two_variables_not_just_a_variable_against_a_literal() {
    // `WithExpr::Compare`'s RHS used to be `Literal`-only -- `WHERE a = b`
    // (comparing two bound node/property values against each other)
    // couldn't parse at all. A self-loop is the only fixed-pattern shape
    // that produces two identical node bindings without needing the
    // unsupported comma-separated cross-join `MATCH (a), (b)` the real
    // TCK scenario for this uses.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:S)-[:R]->(n)"); // self-loop: a == b
    run(&store, "CREATE (:S)-[:R]->(:S)"); // not a self-loop: a != b

    let result = run(&store, "MATCH (a)-->(b) WITH a, b WHERE a = b RETURN a, b");
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn with_where_compares_two_property_accesses() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {id: 1})-[:R]->(:Y {id: 1})");
    run(&store, "CREATE (:X {id: 2})-[:R]->(:Y {id: 3})");

    let result = run(
        &store,
        "MATCH (a:X)-->(b:Y) WITH a, b WHERE a.id = b.id RETURN a.id, b.id",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);
    assert_eq!(int_value(&result.rows[0][1]), 1);
}

#[test]
fn with_where_variable_against_literal_still_works() {
    // Regression guard: widening the RHS from `Literal` to `ReturnExpr`
    // must not break the far more common `WITH ... WHERE x.prop = 1` shape.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {id: 1})");
    run(&store, "CREATE (:N {id: 2})");

    let result = run(&store, "MATCH (n) WITH n WHERE n.id = 1 RETURN n.id");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);
}

#[test]
fn set_property_to_a_computed_expression() {
    // SetItem::Prop's RHS used to be Literal-only -- `SET n.prop =
    // <arithmetic/property-read/function-call>` couldn't parse at all.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'Andres'})");
    let result = run(
        &store,
        "MATCH (n:A) SET n.name = n.name + ' was here' RETURN n.name",
    );
    assert_eq!(str_value(&result.rows[0][0]), "Andres was here");

    run(&store, "CREATE (:B {x: 10})");
    let result2 = run(&store, "MATCH (n:B) SET n.y = n.x * 2 RETURN n.y");
    assert_eq!(int_value(&result2.rows[0][0]), 20);
}

#[test]
fn set_property_to_a_computed_null_removes_it() {
    // The null-removes-property rule is a *runtime* fact about the
    // evaluated value now, not a check against the `Literal::Null` AST
    // token -- `SET n.prop = coalesce(null, null)` must remove the
    // property too, the same as `SET n.prop = null` already did.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:C {keep: 1, drop: 5})");
    let result = run(
        &store,
        "MATCH (n:C) SET n.drop = coalesce(null, null) RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert!(!node.props.contains_key("drop"));
    assert_eq!(
        node.props.get("keep"),
        Some(&marsdb_graph::PropertyValue::Int(1))
    );
}

#[test]
fn set_property_to_a_param_still_works() {
    // Regression guard: widening SetItem::Prop's RHS to a general
    // ReturnExpr must not break $param substitution, the far more common
    // real-world SET shape.
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:D {n: 1})");
    let stmt = parse("MATCH (n:D) SET n.n = $v RETURN n.n").unwrap();
    let mut stmt = stmt;
    let mut params = HashMap::new();
    params.insert("v".to_string(), marsdb_graph::PropertyValue::Int(99));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int_value(&result.rows[0][0]), 99);
}

#[test]
fn builtin_keys_labels_properties_on_a_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:L1:L2 {a: 1, b: 'x'})");

    let keys = run(&store, "MATCH (n) RETURN keys(n)");
    assert_eq!(list_str_values(&keys.rows[0][0]), vec!["a", "b"]);

    let labels = run(&store, "MATCH (n) RETURN labels(n)");
    assert_eq!(list_str_values(&labels.rows[0][0]), vec!["L1", "L2"]);

    let props = run(&store, "MATCH (n) RETURN properties(n)");
    let Value::Map(m) = &props.rows[0][0] else {
        panic!("expected a map");
    };
    assert_eq!(m.len(), 2);
}

#[test]
fn builtin_type_on_a_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:KNOWS]->(:B)");
    let result = run(&store, "MATCH ()-[r]->() RETURN type(r)");
    assert_eq!(str_value(&result.rows[0][0]), "KNOWS");
}

#[test]
fn builtin_nodes_and_relationships_over_a_path() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:KNOWS]->(:B)");
    let result = run(
        &store,
        "MATCH p = (a:A)-[:KNOWS]->(b:B) RETURN nodes(p), relationships(p)",
    );
    let Value::List(nodes) = &result.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(nodes.len(), 2);
    let Value::List(rels) = &result.rows[0][1] else {
        panic!("expected a list");
    };
    assert_eq!(rels.len(), 1);
}

#[test]
fn builtin_size_list_and_string() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        int_value(&run(&store, "RETURN size([1,2,3])").rows[0][0]),
        3
    );
    assert_eq!(
        int_value(&run(&store, "RETURN size('hello')").rows[0][0]),
        5
    );
}

#[test]
fn builtin_range_inclusive_both_ends_and_negative_step() {
    let store = GraphStore::open_memory().unwrap();
    let up = run(&store, "RETURN range(1, 5)");
    let Value::List(items) = &up.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(
        items.iter().map(int_value).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );

    let down = run(&store, "RETURN range(10, 0, -2)");
    let Value::List(items) = &down.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(
        items.iter().map(int_value).collect::<Vec<_>>(),
        vec![10, 8, 6, 4, 2, 0]
    );

    let stmt = parse("RETURN range(1, 5, 0)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("step"));
}

#[test]
fn builtin_head_tail_last_on_a_list() {
    // List literal elements are `Value::Literal`, not `Value::Property`
    // (they're never round-tripped through storage) -- a local extractor,
    // not the shared `int_value` (which is deliberately strict about
    // that distinction for its other callers).
    fn any_int(v: &Value) -> i64 {
        match v {
            Value::Literal(marsdb_query::Literal::Int(i)) => *i,
            other => panic!("expected an int, got {other:?}"),
        }
    }

    let store = GraphStore::open_memory().unwrap();
    assert_eq!(any_int(&run(&store, "RETURN head([1,2,3])").rows[0][0]), 1);
    assert_eq!(any_int(&run(&store, "RETURN last([1,2,3])").rows[0][0]), 3);
    let tail = run(&store, "RETURN tail([1,2,3])");
    let Value::List(items) = &tail.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(items.iter().map(any_int).collect::<Vec<_>>(), vec![2, 3]);

    // Empty list is null, not an error -- same out-of-bounds convention
    // as list indexing elsewhere in this codebase.
    let empty_head = run(&store, "RETURN head([])");
    assert!(matches!(empty_head.rows[0][0], Value::Null));
}

#[test]
fn builtin_string_functions() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        str_value(&run(&store, "RETURN toUpper('hi')").rows[0][0]),
        "HI"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN toLower('HI')").rows[0][0]),
        "hi"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN trim('  hi  ')").rows[0][0]),
        "hi"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN reverse('abc')").rows[0][0]),
        "cba"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN replace('hello world', 'world', 'there')").rows[0][0]),
        "hello there"
    );
    let split = run(&store, "RETURN split('a,b,c', ',')");
    assert_eq!(list_str_values(&split.rows[0][0]), vec!["a", "b", "c"]);
    assert_eq!(
        str_value(&run(&store, "RETURN substring('hello', 1, 3)").rows[0][0]),
        "ell"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN left('hello', 3)").rows[0][0]),
        "hel"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN right('hello', 3)").rows[0][0]),
        "llo"
    );
}

#[test]
fn builtin_math_functions() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(int_value(&run(&store, "RETURN abs(-5)").rows[0][0]), 5);
    assert_eq!(
        float_value(&run(&store, "RETURN abs(-5.5)").rows[0][0]),
        5.5
    );
    assert_eq!(
        float_value(&run(&store, "RETURN ceil(4.1)").rows[0][0]),
        5.0
    );
    assert_eq!(
        float_value(&run(&store, "RETURN floor(4.9)").rows[0][0]),
        4.0
    );
    assert_eq!(
        float_value(&run(&store, "RETURN sqrt(16.0)").rows[0][0]),
        4.0
    );
    assert_eq!(int_value(&run(&store, "RETURN sign(-7)").rows[0][0]), -1);
    assert_eq!(int_value(&run(&store, "RETURN sign(0)").rows[0][0]), 0);
}

#[test]
fn builtin_to_float_and_to_boolean() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        float_value(&run(&store, "RETURN toFloat('12.5')").rows[0][0]),
        12.5
    );
    assert!(bool_value(
        &run(&store, "RETURN toBoolean('true')").rows[0][0]
    ));
    assert!(!bool_value(
        &run(&store, "RETURN toBoolean('false')").rows[0][0]
    ));
}

/// `toFloat()` on a `Bool` is a real type error, not `null` -- unlike an
/// unparseable *string*, which real Cypher does treat as `null` (a
/// string always at least plausibly could be numeric text, a boolean
/// never could be). TCK's TypeConversion3 [6].
#[test]
fn to_float_on_a_bool_is_a_type_error_not_null() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN toFloat(true)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("tofloat"));
    // sanity: an unparseable string still degrades to null, not an error
    assert!(matches!(
        run(&store, "RETURN toFloat('nope')").rows[0][0],
        Value::Null
    ));
}

/// Only a node, relationship, map, or temporal value has any `.prop` to
/// access at all -- a plain scalar or list is a real type error, not a
/// silent `null`. TCK's Graph6 [9] / Map1 [6].
#[test]
fn property_access_on_a_non_graph_scalar_or_list_is_a_type_error() {
    let store = GraphStore::open_memory().unwrap();
    for exp in ["123", "42.45", "true", "false", "'string'", "[123, true]"] {
        let stmt = parse(&format!(
            "WITH {exp} AS nonGraphElement RETURN nonGraphElement.num"
        ))
        .unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("propert"),
            "expected a property-access type error for {exp:?}, got: {err}"
        );
    }
    // sanity: null, maps, nodes, and temporal values still work normally
    assert!(matches!(
        run(&store, "WITH null AS x RETURN x.num").rows[0][0],
        Value::Null
    ));
    match &run(&store, "WITH {name: 'foo'} AS m RETURN m.name").rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "foo"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `type()` only ever accepts a relationship -- `MATCH (r) RETURN
/// type(r)` (`r` a *node*, from the pattern itself) is a compile-time
/// error even when the `MATCH` matches zero rows, not only a runtime one
/// a zero-row match would silently skip. TCK's Graph4 [7].
#[test]
fn type_on_a_node_is_a_compile_time_error_even_on_zero_rows() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (r) RETURN type(r)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("relationship"));
}

#[test]
fn property_presence_check_via_is_not_null() {
    // `exists(n.num)` (bare function-call form) isn't real openCypher --
    // grep against openCypher.bnf/the TCK corpus finds no such function,
    // only the unrelated `EXISTS { <pattern> }` subquery form. `IS NOT
    // NULL` is the real, spec-correct way to check property presence.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {num: 42})");
    assert!(bool_value(
        &run(&store, "MATCH (n) RETURN n.num IS NOT NULL").rows[0][0]
    ));
    assert!(!bool_value(
        &run(&store, "MATCH (n) RETURN n.missing IS NOT NULL").rows[0][0]
    ));
}

#[test]
fn builtin_id_returns_an_integer() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)");
    let result = run(&store, "MATCH (n) RETURN id(n)");
    // Just needs to be a real, non-negative integer -- the exact value is
    // an internal id, not something callers should depend on.
    assert!(int_value(&result.rows[0][0]) >= 0);
}

#[test]
fn unknown_function_name_is_a_semantic_error_not_a_panic() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN totallyMadeUpFunction(1)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn label_check_expression_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    run(&store, "CREATE (:Foo)");
    let result = run(&store, "MATCH (n) RETURN (n:Foo)");
    assert_eq!(result.rows.len(), 2);
    assert!(!bool_value(&result.rows[0][0]));
    assert!(bool_value(&result.rows[1][0]));
}

#[test]
fn label_check_expression_requires_every_listed_label() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A:B)");
    let result = run(&store, "MATCH (n:A:B) RETURN (n:A:B), (n:A:C)");
    assert!(bool_value(&result.rows[0][0]));
    assert!(!bool_value(&result.rows[0][1]));
}

#[test]
fn label_check_expression_on_a_null_binding_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "OPTIONAL MATCH (n:DoesNotExist) RETURN (n:Foo)");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn label_check_expression_on_a_non_node_is_a_semantic_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("WITH 5 AS x RETURN (x:Foo)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn union_dedups_by_default() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN 2 AS x UNION RETURN 1 AS x UNION RETURN 2 AS x",
    );
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn union_all_keeps_every_row() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN 2 AS x UNION ALL RETURN 1 AS x UNION ALL RETURN 2 AS x",
    );
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn union_combines_two_match_clauses() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)");
    run(&store, "CREATE (:B)");
    let result = run(
        &store,
        "MATCH (a:A) RETURN a AS a UNION MATCH (b:B) RETURN b AS a",
    );
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn union_with_different_columns_is_a_semantic_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AS a UNION RETURN 2 AS b").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn mixing_union_and_union_all_is_a_syntax_error() {
    let err = parse("RETURN 1 AS a UNION RETURN 2 AS a UNION ALL RETURN 3 AS a").unwrap_err();
    assert!(err.to_string().starts_with("syntax error:"));
}

#[test]
fn explain_over_a_union_shows_each_part_and_the_union_keyword() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "EXPLAIN RETURN 1 AS x UNION RETURN 2 AS x");
    let lines = plan_lines(&result);
    assert!(lines.iter().any(|l| l == "UNION"));
    assert_eq!(lines.iter().filter(|l| l.contains("RETURN x")).count(), 2);
}

#[test]
fn unwind_source_can_be_a_function_call() {
    // UnwindSource used to be Var(String)/List(Vec<Literal>) only --
    // `range(0, 2)` (or any other non-literal, non-bare-var expression)
    // couldn't parse at all.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "UNWIND range(0, 2) AS i RETURN i");
    assert_eq!(result.rows.len(), 3);
    assert_eq!(int_value(&result.rows[0][0]), 0);
    assert_eq!(int_value(&result.rows[2][0]), 2);
}

#[test]
fn unwind_source_can_be_a_property_access() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH {tags: [1, 2, 3]} AS m UNWIND m.tags AS t RETURN t",
    );
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn unwind_null_produces_zero_rows() {
    // Real Cypher: unwinding null behaves like unwinding an empty list,
    // not an error and not one null row.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "UNWIND null AS x RETURN x");
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn unwind_source_bare_literal_list_and_bound_variable_still_work() {
    // Regression guard for the two shapes UnwindSource used to hard-code.
    let store = GraphStore::open_memory().unwrap();
    let literal = run(&store, "UNWIND [1, 2, 3] AS x RETURN x");
    assert_eq!(literal.rows.len(), 3);

    run(&store, "CREATE (:N)");
    run(&store, "CREATE (:N)");
    let restored = run(
        &store,
        "MATCH (n) WITH collect(n) AS ns UNWIND ns AS n RETURN n",
    );
    assert_eq!(restored.rows.len(), 2);
    assert!(matches!(restored.rows[0][0], Value::Node(_)));
}

/// Renders a `Value::List`'s scalar elements as a compact string
/// (`Value` has no `PartialEq`, and this reads far better than a chain of
/// nested `matches!`/`if let`s for an 8-row order assertion).
fn list_repr(v: &Value) -> String {
    fn scalar_repr(v: &Value) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Literal(marsdb_query::Literal::Int(i)) => i.to_string(),
            Value::Literal(marsdb_query::Literal::String(s)) => format!("'{s}'"),
            other => panic!("unexpected list element {other:?}"),
        }
    }
    match v {
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(scalar_repr).collect::<Vec<_>>().join(", ")
        ),
        other => panic!("expected a list, got {other:?}"),
    }
}

#[test]
fn order_by_desc_sorts_lists_correctly() {
    // Real bug found while widening UNWIND's source to a general
    // expression (a previous session change): nested list literals like
    // `[[], ['a'], [1, 'a'], ...]` couldn't parse at all before that fix,
    // so this query -- and the list-vs-list ORDER BY path it exercises --
    // had never actually run. `compare_non_null` had no `Value::List` arm,
    // silently falling through to its scalar-only `_ => Ordering::Equal`
    // catch-all, so ORDER BY on a list column was a silent no-op
    // (stable-sort-over-always-Equal preserves input order) regardless of
    // ASC/DESC. Exact scenario + expected order from TCK's ReturnOrderBy1
    // `[10]`.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [[], ['a'], ['a', 1], [1], [1, 'a'], [1, null], [null, 1], [null, 2]] AS lists \
         RETURN lists ORDER BY lists DESC",
    );
    let got: Vec<String> = result.rows.iter().map(|row| list_repr(&row[0])).collect();
    assert_eq!(
        got,
        vec![
            "[null, 2]",
            "[null, 1]",
            "[1, null]",
            "[1, 'a']",
            "[1]",
            "['a', 1]",
            "['a']",
            "[]",
        ]
    );
}

#[test]
fn delete_target_can_be_a_map_value_access() {
    // Delete targets used to be bare identifiers only -- `DELETE
    // nodes.key`/`friends[$i]` (any expression that evaluates to a node/
    // relationship/path) couldn't parse at all.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:User)");
    run(&store, "CREATE (:User)");
    run(
        &store,
        "MATCH (u:User) WITH {key: u} AS nodes DELETE nodes.key",
    );
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_target_can_be_a_list_index() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:User)-[:FRIEND]->(:A)");
    run(&store, "CREATE (:User)-[:FRIEND]->(:B)");
    run(
        &store,
        "MATCH (:User)-[:FRIEND]->(n) WITH collect(n) AS friends DETACH DELETE friends[0]",
    );
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 3);
}

#[test]
fn detach_delete_whole_named_path() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X)-[:R]->()-[:R]->()-[:R]->()");
    run(&store, "MATCH p = (:X)-->()-->()-->() DETACH DELETE p");
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_arithmetic_expression_is_a_semantic_error() {
    // Structurally can never be a node/relationship/path (unlike
    // `map.key`, below) -- rejected at semantic-validation time,
    // independent of whether MATCH finds any rows to actually reach
    // materialize_delete with (TCK's Delete5 [9]).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    let stmt = parse("MATCH (n) DELETE 1 + 1").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn delete_a_non_graph_map_value_is_a_runtime_type_error() {
    // A map/property access types as Kind::Scalar structurally (it might
    // legitimately hold a node at runtime, e.g. `nodes.key` elsewhere in
    // this file) so it passes semantic validation -- only once the actual
    // value (a plain Int here, not a node/edge/path) is in hand does
    // delete_value catch it.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    let stmt = parse("MATCH (n) WITH {key: 5} AS m DELETE m.key").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("type error:"));
}

#[test]
fn delete_multiple_bare_variables_across_two_rows_of_the_same_edge() {
    // Regression guard: a bare-variable DELETE target must keep using the
    // raw row Binding (no existence check), not full expression
    // evaluation -- otherwise the second of two rows referencing the same
    // already-deleted-by-the-first-row entities would error instead of
    // silently deduping. Exact shape from TCK's Delete4 [1].
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:R]->()");
    let result = run(
        &store,
        "MATCH (a)-[r]-(b) DELETE r, a, b RETURN count(*) AS c",
    );
    assert_eq!(int_value(&result.rows[0][0]), 2);
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_multiple_path_targets_deletes_all_edges_before_any_node() {
    // TCK Delete5 [7]: two DELETE targets in one (non-DETACH) clause can
    // each hold one of a shared node's two edges -- deleting the first
    // target's node inline (before the second target's edge is gone)
    // would wrongly fail with "node has incident edges", even though the
    // whole DELETE, taken together, removes every edge that node had.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:User), (b:User) CREATE (a)-[:R]->(b), (b)-[:R]->(a)",
    );
    run(
        &store,
        "MATCH p = (:User)-[r]->(:User) \
         WITH {key: collect(p)} AS pathColls \
         DELETE pathColls.key[0], pathColls.key[1]",
    );
    let nodes = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&nodes.rows[0][0]), 0);
}

/// `<expr>.prop` where `<expr>` isn't a bare variable -- `startNode(r).id`
/// (a function-call result). TCK's Merge5 [11].
#[test]
fn property_access_on_a_function_calls_result() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a {id: 2}), (b {id: 1})");
    let result = run(
        &store,
        "MATCH (a {id: 2}), (b {id: 1}) MERGE (a)-[r:KNOWS]-(b) \
         RETURN startNode(r).id AS s, endNode(r).id AS e",
    );
    assert_eq!(int(&result.rows[0][0]), 2);
    assert_eq!(int(&result.rows[0][1]), 1);
}

/// `(list[1]).prop` -- property access on a node/map produced by indexing
/// into a list, not a bare variable. TCK's Map1 [3], Graph6 [4]/[8].
/// Missing properties read back as `null`, same as `var.missing` already
/// does for a bound variable.
#[test]
fn property_access_on_an_indexed_list_element() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({existing: 42, missing: null})");
    let result = run(
        &store,
        "MATCH (n) WITH [123, n] AS list \
         RETURN (list[1]).missing, (list[1]).missingToo, (list[1]).existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);

    let result = run(
        &store,
        "WITH [123, {existing: 42, notMissing: null}] AS list \
         RETURN (list[1]).missing, (list[1]).notMissing, (list[1]).existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);
}

/// `MERGE p = ...` -- named-path capture on MERGE itself, both when the
/// pattern is created fresh and when it's found by an ordinary match.
/// TCK's Merge1 [13], Merge5 [10].
#[test]
fn merge_named_path_capture_on_create_and_on_match() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "MERGE p = (a {num: 1}) RETURN p");
    match &result.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 1),
        other => panic!("expected a Path, got {other:?}"),
    }

    run(&store, "MERGE (a {num: 1}) MERGE (b {num: 2})");
    let result = run(
        &store,
        "MATCH (a {num: 1}), (b {num: 2}) MERGE p = (a)-[:R]->(b) RETURN p",
    );
    match &result.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 3),
        other => panic!("expected a Path, got {other:?}"),
    }
    // Re-running the same MERGE finds the just-created relationship
    // instead of creating a second one -- the found branch must also
    // capture the path, not just the create branch.
    let again = run(
        &store,
        "MATCH (a {num: 1}), (b {num: 2}) MERGE p = (a)-[:R]->(b) RETURN p",
    );
    match &again.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 3),
        other => panic!("expected a Path, got {other:?}"),
    }
    let count = run(&store, "MATCH ()-[r:R]->() RETURN count(r)");
    assert_eq!(int(&count.rows[0][0]), 1);
}

/// `WITH *` with nothing bound at all -- a legal no-op, unlike `RETURN *`
/// in the same situation (real Cypher's `NoVariablesInScope`, which only
/// applies to `RETURN *`). TCK's Create3 [2]/[3]: every pattern token is
/// anonymous, so there's genuinely nothing for `WITH *` to carry forward.
#[test]
fn with_star_tolerates_an_empty_scope() {
    // TCK Create3 [2]: 2 pre-existing nodes, +4 from the query itself
    // (MATCH matches both, each row creates 2 more) -- 6 total.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (), ()");
    run(&store, "MATCH () CREATE () WITH * CREATE ()");
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int(&count.rows[0][0]), 6);

    // TCK Create3 [3]: 2 pre-existing nodes, +10 -- 12 total.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (), ()");
    run(&store, "MATCH () CREATE () WITH * MATCH () CREATE ()");
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int(&count.rows[0][0]), 12);
}

/// `WITH ... WHERE (a)-->(b)` -- a pattern predicate combined with
/// ordinary comparisons via AND/OR, in a WITH's own WHERE (not just
/// MATCH's). TCK's WithWhere4 [2].
#[test]
fn with_where_pattern_predicate_combined_with_and_or() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:TheLabel {id: 0}), (b:TheLabel {id: 1}), (c:TheLabel {id: 2}) \
         CREATE (a)-[:T]->(b), (b)-[:T]->(c)",
    );
    let result = run(
        &store,
        "MATCH (a), (b) \
         WITH a, b \
         WHERE a.id = 0 AND (a)-[:T]->(b:TheLabel) OR (a)-[:T*]->(b:MissingLabel) \
         RETURN DISTINCT b.id AS id",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `WITH DISTINCT x AS y WHERE <expr referencing the pre-WITH scope>` --
/// unlike aggregation, `DISTINCT` alone doesn't make the pre-WITH scope
/// ambiguous (`WHERE` runs before the dedup, not after). TCK's
/// WithWhere1 [2].
#[test]
fn with_distinct_where_sees_pre_with_scope() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE ({name2: 'A'}), ({name2: 'A'}), ({name2: 'B'})",
    );
    let result = run(
        &store,
        "MATCH (a) WITH DISTINCT a.name2 AS name WHERE a.name2 = 'B' RETURN name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "B");
}

/// `nestedMap.name.name2` -- a chain of two `.prop` suffixes. TCK's
/// With2 [2].
#[test]
fn chained_property_access_on_a_nested_map_literal() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH {name: {name2: 'baz'}} AS nestedMap RETURN nestedMap.name.name2",
    );
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "baz"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `n['name']` -- dynamic property access on a node/relationship, with a
/// computed key expression, not just a literal string. TCK's Graph7
/// [1]-[3].
#[test]
fn dynamic_property_access_on_a_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({name: 'Apa'})");
    let result = run(
        &store,
        "MATCH (n {name: 'Apa'}) RETURN n['nam' + 'e'] AS value",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Apa"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `type(null)` -- a null-valued argument to a graph-object builtin
/// (`type`/`nodes`/`relationships`/`length`) is `null`, not a compile-time
/// type error. TCK's Graph4 [3], Path1 [1], Path2 [3].
#[test]
fn graph_builtins_on_null_argument_are_null_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN type(null), nodes(null), relationships(null), length(null)",
    );
    for cell in &result.rows[0] {
        assert!(matches!(cell, Value::Null), "expected null, got {cell:?}");
    }
}

/// `r.name` where `r` is bound to a path -- real Cypher's
/// `InvalidArgumentType`, not a silent null (a path was never a valid
/// property-access target). TCK's MatchWhere1 [14].
#[test]
fn property_access_on_a_path_is_a_type_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH r = (n)-[*]->() WHERE r.name = 'apa' RETURN r").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("path"),
        "unexpected error: {err}"
    );
}

/// `size(p)` where `p` is a path -- real Cypher rejects this at compile
/// time, not just at runtime (a zero-row MATCH could otherwise silently
/// skip ever evaluating it). TCK's List6 [5].
#[test]
fn size_on_a_path_is_a_compile_time_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH p = (a)-[*]->(b) RETURN size(p)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("size"),
        "unexpected error: {err}"
    );
}

/// `duration.between(...)`'s own component accessors must read the
/// stored `seconds`/`nanos` fields directly, not recombine them into one
/// signed total and re-split (which would silently reintroduce a
/// negative `nanos`, breaking the "nanos always non-negative, sign
/// lives in seconds" storage invariant). TCK's Temporal10 [1].
#[test]
fn duration_component_accessors_read_raw_fields_not_a_resplit_total() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration.between(localdatetime('2018-01-02T10:00:00.1'), \
         localdatetime('2018-01-01T10:00:00.2')) AS dur \
         RETURN dur, dur.days, dur.seconds, dur.nanosecondsOfSecond",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-23H-59M-59.9S");
    assert_eq!(int(&result.rows[0][1]), 0);
    assert_eq!(int(&result.rows[0][2]), -86400);
    assert_eq!(int(&result.rows[0][3]), 100_000_000);
}

/// `duration('P2012-02-02T14:37:21.545')` -- ISO-8601's alternate
/// "combined date-time" duration representation (date/time formatted
/// like a calendar date/time-of-day, each field meaning "this many
/// years/months/days/hours/minutes/seconds"), not the more common
/// `PnYnMnD` form. TCK's Temporal2 [7].
#[test]
fn duration_parses_the_combined_date_time_alternate_form() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN duration('P2012-02-02T14:37:21.545') AS d");
    assert_eq!(temporal_str(&result.rows[0][0]), "P2012Y2M2DT14H37M21.545S");
}

/// `datetime.fromepoch(seconds, nanos)`/`datetime.fromepochmillis(millis)`.
/// TCK's Temporal1 [11].
#[test]
fn datetime_from_epoch_and_epoch_millis() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN datetime.fromepoch(416779, 999999999) AS d1, \
         datetime.fromepochmillis(237821673987) AS d2",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1970-01-05T19:46:19.999999999Z"
    );
    assert_eq!(temporal_str(&result.rows[0][1]), "1977-07-15T13:34:33.987Z");
}

/// `MATCH p = (n)-->(x) WHERE length(p) = 1` -- a named path's own
/// inline WHERE can reference the path variable itself, which isn't in
/// the row until after path assembly, unlike an ordinary pattern's
/// WHERE. TCK's MatchWhere1 [12]/[13].
#[test]
fn named_path_where_references_the_path_variable_itself() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A {name: 'A'})-[:KNOWS]->(b:B {name: 'B'})",
    );
    let result = run(&store, "MATCH p = (n)-->(x) WHERE length(p) = 1 RETURN x");
    assert_eq!(result.rows.len(), 1);

    let result = run(&store, "MATCH p = (n)-->(x) WHERE length(p) = 10 RETURN x");
    assert_eq!(result.rows.len(), 0);
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

/// `MATCH ()-[r]->() DELETE r RETURN type(r)` -- a relationship's type
/// never changes, so real Cypher still allows reading it after the
/// relationship itself is deleted earlier in the same statement, unlike
/// labels()/property access (real DeletedEntityAccess errors). TCK's
/// Return2 [14].
#[test]
fn type_of_a_deleted_relationship_still_works() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T]->()");
    let result = run(&store, "MATCH ()-[r]->() DELETE r RETURN type(r)");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "T"),
        other => panic!("expected a string, got {other:?}"),
    }

    // Properties of a deleted relationship still correctly error.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T {num: 0}]->()");
    let stmt = parse("MATCH ()-[r]->() DELETE r RETURN r.num").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_err());
}

/// `MATCH (a)-[r*1..3]->(b) RETURN r` -- binding a variable to a
/// variable-length relationship pattern collects the traversed
/// relationships into a list, not a single edge. TCK's Match4 [1]/[6].
#[test]
fn variable_length_relationship_binds_a_list_of_edges() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T]->()");
    let result = run(&store, "MATCH (a)-[r*1..1]->(b) RETURN r");
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                Value::Edge(e) => assert_eq!(e.label, "T"),
                other => panic!("expected an Edge, got {other:?}"),
            }
        }
        other => panic!("expected a List, got {other:?}"),
    }

    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A), (b), (c) CREATE (a)-[:X]->(b), (b)-[:Y]->(c)",
    );
    let result = run(&store, "MATCH (a:A) MATCH (a)-[r*2]->() RETURN r");
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            let labels: Vec<&str> = items
                .iter()
                .map(|v| match v {
                    Value::Edge(e) => e.label.as_str(),
                    other => panic!("expected an Edge, got {other:?}"),
                })
                .collect();
            assert_eq!(labels, vec!["X", "Y"]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    // Matching a variable-length pattern against an *already-bound* list
    // variable (real Cypher: "match a path whose edges equal this list")
    // is a genuinely different, unsupported feature -- must stay
    // rejected, not silently produce the wrong count. The check lives in
    // the planner (build_match_plan), reached at execution time, not
    // parse time.
    let stmt = parse(
        "MATCH ()-[r1]->()-[r2]->() WITH [r1, r2] AS rs LIMIT 1 \
         MATCH (first)-[rs*]->(second) RETURN first, second",
    )
    .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("already-bound"));
}

/// `WITH null AS a OPTIONAL MATCH p = (a)-[r]->()` -- reusing an
/// already-bound-to-`null` variable as a node/relationship pattern token
/// is legal (matches nothing, same as any other `OPTIONAL MATCH` miss),
/// unlike reusing a variable bound to a real, wrong-typed value (`WITH 1
/// AS x MATCH (x)-->()`, still a real compile-time type error). TCK's
/// Path1 [1], Path2 [3].
#[test]
fn null_bound_variable_reused_as_pattern_token_is_legal() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH null AS a \
         OPTIONAL MATCH p = (a)-[r]->() \
         RETURN type(r), nodes(p), relationships(p), length(p)",
    );
    for cell in &result.rows[0] {
        assert!(matches!(cell, Value::Null), "expected null, got {cell:?}");
    }

    // A real, non-null, wrong-typed reused variable must still error.
    let stmt = parse("WITH 1 AS x MATCH (x)-[:R]->(n) RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("node"));
}

/// `MATCH (a)-[:TYPE* {prop: 'x'}]->(b)` -- filters *every* hop of the
/// variable-length traversal by the same inline property map, not just
/// the final one. A 2-hop path where only the second hop matches doesn't
/// survive as a 1-hop match either -- the whole path from the first
/// non-matching hop onward is excluded. TCK's Match4 [5].
#[test]
fn inline_properties_on_a_variable_length_relationship_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Artist:A), (b:Artist:B), (c:Artist:C) \
         CREATE (a)-[:WORKED_WITH {year: 1987}]->(b), \
                (b)-[:WORKED_WITH {year: 1988}]->(c)",
    );
    let result = run(
        &store,
        "MATCH (a:Artist)-[:WORKED_WITH* {year: 1988}]->(b:Artist) RETURN a, b",
    );
    assert_eq!(result.rows.len(), 1);
    let a_labels = match &result.rows[0][0] {
        Value::Node(n) => &n.labels,
        other => panic!("expected a Node, got {other:?}"),
    };
    let b_labels = match &result.rows[0][1] {
        Value::Node(n) => &n.labels,
        other => panic!("expected a Node, got {other:?}"),
    };
    assert!(a_labels.contains(&"B".to_string()));
    assert!(b_labels.contains(&"C".to_string()));
}
