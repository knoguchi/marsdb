//! Smoke tests: WITH projection/carry, UNWIND, UNION, CALL -- split from the original smoke.rs.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

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
fn unwind_non_list_var_errors_clearly() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    // `p` is bound to a node, not a list -- UNWIND needs a real list
    // (e.g. from collect()), not any bound variable.
    let stmt = parse("MATCH (p:Person) UNWIND p AS x RETURN x").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("list"));
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

/// Within a single write statement, a later clause reading a node must
/// see an earlier clause's own mutation to it, not a stale value -- the
/// cache is disabled entirely for write statements (see `Executor::
/// node_cache`'s docs), so this is really testing that the disable
/// actually takes effect, not just that caching is scoped per-statement.
#[test]
fn node_cache_is_disabled_within_a_single_write_statement() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item {name: 'old'})");

    // One statement: reads n.name (would populate/consult the cache if
    // it were mistakenly enabled for writes), then sets it, then a
    // second MATCH...RETURN in the same statement (via UNION) reads it
    // again -- must see 'new' both times it matters, never 'old' from a
    // stale cache entry.
    let result = run(
        &store,
        "MATCH (n:Item) WITH n, n.name AS before SET n.name = 'new' RETURN before, n.name AS after",
    );
    assert_eq!(result.rows.len(), 1);
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Property(marsdb_graph::PropertyValue::String(before)),
            Value::Property(marsdb_graph::PropertyValue::String(after)),
        ) => {
            assert_eq!(before, "old");
            assert_eq!(after, "new");
        }
        other => panic!("unexpected values {other:?}"),
    }
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
fn list_comprehension_plain_list_with_bare_identifier_is_not_misparsed_as_a_comprehension() {
    // `x` alone in a list (no `IN` following) must fall through to the
    // ordinary comma-separated list_expr alternative, not be swallowed
    // partway through a failed list_comprehension attempt.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH 1 AS x, 2 AS y RETURN [x, y]");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2]);
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
