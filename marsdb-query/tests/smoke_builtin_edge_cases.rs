//! Smoke tests for builtin scalar/temporal function branches that
//! ordinary query coverage elsewhere in this crate never happens to
//! exercise: the `.transaction`/`.statement`/`.realtime` no-arg
//! temporal aliases (`call_builtin`'s dispatch table in
//! `executor/scalar_fns.rs`), and the relationship/map/error arms of a
//! handful of node-centric introspection functions.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;

/// `date.transaction()`/`.statement()`/`.realtime()` are three aliases
/// for the same no-arg "current instant" builtin, sharing one match arm
/// per temporal type -- calling any one alias covers all three. None of
/// these five families (date/localtime/time/localdatetime/datetime) are
/// otherwise called anywhere in this crate's test suite; every other
/// `now()`-family test uses the bare `date()`/`time()`/etc form instead.
#[test]
fn transaction_statement_realtime_temporal_aliases_all_return_a_value() {
    let store = GraphStore::open_memory().unwrap();
    for expr in [
        "date.transaction()",
        "date.statement()",
        "date.realtime()",
        "localtime.transaction()",
        "localtime.statement()",
        "localtime.realtime()",
        "time.transaction()",
        "time.statement()",
        "time.realtime()",
        "localdatetime.transaction()",
        "localdatetime.statement()",
        "localdatetime.realtime()",
        "datetime.transaction()",
        "datetime.statement()",
        "datetime.realtime()",
    ] {
        let result = run(&store, &format!("RETURN {expr} AS v"));
        assert!(
            !matches!(result.rows[0][0], marsdb_query::Value::Null),
            "{expr} returned Null"
        );
    }
}

/// An explicit `Null` argument short-circuits to `Null` rather than
/// computing "now" -- `now_or_null`'s other branch, shared by all fifteen
/// aliases above.
#[test]
fn transaction_alias_with_explicit_null_arg_returns_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN date.transaction(null) AS v");
    assert!(matches!(result.rows[0][0], marsdb_query::Value::Null));
}

#[test]
fn keys_labels_type_properties_id_on_a_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'a'})-[:KNOWS {since: 2020}]->(:Person {name: 'b'})",
    );
    let result = run(
        &store,
        "MATCH ()-[r:KNOWS]->() RETURN keys(r), type(r), properties(r), id(r)",
    );
    assert_eq!(list_str_values(&result.rows[0][0]), vec!["since"]);
    assert_eq!(str_value(&result.rows[0][1]), "KNOWS");
    assert!(matches!(result.rows[0][2], marsdb_query::Value::Map(_)));
    assert!(int_value(&result.rows[0][3]) >= 0);
}

#[test]
fn keys_and_properties_on_a_map_literal() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN keys({a: 1, b: 2}), properties({a: 1})");
    let mut ks = list_str_values(&result.rows[0][0]);
    ks.sort();
    assert_eq!(ks, vec!["a", "b"]);
    assert!(matches!(result.rows[0][1], marsdb_query::Value::Map(_)));
}

#[test]
fn keys_labels_type_properties_id_size_report_type_errors_on_wrong_argument_kind() {
    let store = GraphStore::open_memory().unwrap();
    for (expr, needle) in [
        ("keys(5)", "keys()"),
        ("labels(5)", "labels()"),
        ("labels({a: 1})", "labels()"),
        ("type(5)", "type()"),
        ("properties(5)", "properties()"),
        ("id('x')", "id()"),
        ("size({a: 1})", "size()"),
    ] {
        let stmt = marsdb_query::parse(&format!("RETURN {expr}")).unwrap();
        let err = marsdb_query::Executor::new(&store)
            .execute(&stmt)
            .unwrap_err();
        assert!(
            err.to_string().contains(needle),
            "expected {expr:?}'s error to mention {needle:?}, got: {err}"
        );
    }
}
