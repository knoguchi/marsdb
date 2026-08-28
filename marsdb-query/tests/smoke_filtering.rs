//! Smoke tests: WHERE, ORDER BY, SKIP, LIMIT, DISTINCT.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

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

/// `WITH x AS y WHERE ...` sees both the pre-WITH binding (`x`) and the
/// new alias (`y`) -- the WHERE isn't scoped to only the
/// projected/aliased row the way a later clause is.
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

/// `WHERE a:A` -- a user-typed label predicate directly in pattern-level
/// `WHERE`, not just the planner-synthesized form multi-label node
/// patterns already use internally.
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

#[test]
fn order_by_multi_key_against_aliases_not_raw_bindings() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Charlie', age: 30})");
    run(&store, "CREATE (b:Person {name: 'Alice', age: 30})");
    run(&store, "CREATE (c:Person {name: 'Bob', age: 25})");

    // Sort keys are aliases (personAge/person_name), not raw pattern
    // vars, matching the shape every IS-query ORDER BY uses.
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

/// The tail hint lets the fast path pre-truncate groups when the FINAL
/// clause's RETURN carries ORDER BY count + SKIP/LIMIT. SKIP must remain
/// exact: the loop keeps skip+limit groups and the generic tail slices
/// precisely -- this pins that no double-skip / short-keep sneaks in.
#[test]
fn fast_path_tail_order_skip_limit_matches_generic() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (s:Movie {title: 'Seed'}), (a:Movie {title: 'A'}), (b:Movie {title: 'B'}), \
         (c:Movie {title: 'C'}), (u1:User {name: 'u1'}), (u2:User {name: 'u2'}), \
         (u3:User {name: 'u3'})",
    );
    // Rating counts back to recs: A=3, B=2, C=1.
    for (u, ms) in [
        ("u1", vec!["Seed", "A", "B"]),
        ("u2", vec!["Seed", "A", "B", "C"]),
        ("u3", vec!["Seed", "A"]),
    ] {
        for m in ms {
            run(
                &store,
                &format!(
                    "MATCH (u:User {{name:'{u}'}}), (m:Movie {{title:'{m}'}}) \
                     CREATE (u)-[:RATED]->(m)"
                ),
            );
        }
    }
    let fast = run(
        &store,
        "MATCH (m:Movie {title: 'Seed'})<-[:RATED]-(u:User)-[:RATED]->(rec:Movie) \
         WITH rec, count(*) AS c RETURN rec.title, c ORDER BY c DESC SKIP 1 LIMIT 1",
    );
    let generic = run(
        &store,
        "MATCH (m:Movie {title: 'Seed'})<-[:RATED]-(u:User)-[:RATED]->(rec:Movie) \
         WHERE rec.title <> '\u{0}never' \
         WITH rec, count(*) AS c RETURN rec.title, c ORDER BY c DESC SKIP 1 LIMIT 1",
    );
    assert_eq!(format!("{:?}", fast.rows), format!("{:?}", generic.rows));
    assert_eq!(fast.rows.len(), 1);
    match &fast.rows[0][1] {
        Value::Property(marsdb_graph::PropertyValue::Int(c)) => assert_eq!(*c, 2), // B, after skipping A
        other => panic!("unexpected {other:?}"),
    }
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

/// `WHERE (n)-[:REL]->()` as a boolean predicate (TCK's Pattern1): true
/// iff a match exists, without binding a row per match. Covers
/// existential/negated/conjunction shapes and the two-bound-endpoints
/// shape (`(n)-->(m)`, both `n`/`m` from an outer `MATCH (n), (m)`).
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

#[test]
fn with_where_comparison_followed_by_order_by_still_parses() {
    // Regression: widening `return_expr` to include a trailing
    // comparison broke `with_comparison`'s own WHERE shape -- its
    // operand greedily swallowed `y > 10`, leaving nothing for the
    // trailing compare_op to match.
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
/// `^"ASCENDING"`, matching just the prefix and leaving `ENDING` dangling
/// (longest alternative must come first in a pest `|` alternation).
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
    // it matched the first two letters of `ORDER`, mis-parsing `RETURN x
    // OR y ORDER BY z` as `RETURN (x OR y OR DER) BY z` -- caught by
    // `y ORDER BY y`, where nothing sits between the boolean expression
    // and ORDER BY.
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

/// `IN` used directly as a bare WHERE predicate (no comparison operator)
/// -- needs `general_bare_expr`'s widening from `add_expr` to
/// `null_predicate_expr`. Also caught `labels()`/`keys()` mis-typed as
/// `Kind::Scalar` in semantic inference, which wrongly rejected this
/// list comprehension.
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

#[test]
fn list_comprehension_bare_where_now_parses() {
    // Regression: `filter_expr`'s WHERE reused WithExpr, which only ever
    // wrapped a single Compare -- a bare boolean value (`WHERE x`/`WHERE
    // true`) failed to parse.
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
    // Parsing gap only -- none() on an empty list is vacuously true
    // regardless of the WHERE condition (already covered by
    // quantifier_none_on_empty_list_is_true).
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN none(x IN [] WHERE true) AS a, none(x IN [] WHERE false) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
}

#[test]
fn where_clause_equality_fuses_into_index_seek() {
    // Unlike an inline pattern property (already covered by
    // create_index_then_lookup_via_index_seek), a WHERE-clause equality
    // compiles to a separate outer Filter -- apply_index_seeks'
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
fn with_where_compares_two_variables_not_just_a_variable_against_a_literal() {
    // `WHERE a = b` compares two bound node/property values -- a
    // self-loop is the only fixed-pattern shape producing two identical
    // node bindings without the unsupported comma-separated cross-join
    // `MATCH (a), (b)` the real TCK scenario uses.
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
fn order_by_desc_sorts_lists_correctly() {
    // `compare_non_null` had no `Value::List` arm, falling through to a
    // scalar-only `_ => Ordering::Equal` catch-all -- ORDER BY on a list
    // column was a no-op regardless of ASC/DESC. TCK's ReturnOrderBy1
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
