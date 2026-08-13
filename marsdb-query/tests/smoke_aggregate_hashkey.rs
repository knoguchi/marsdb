//! Smoke tests for `aggregate.rs`'s `HashKey` conversion -- the
//! `DISTINCT`/grouping identity used for both `RETURN DISTINCT`/
//! `count(DISTINCT ...)` and implicit `GROUP BY`. Ordinary scalar types
//! (int/float/string/bool/null) are exercised everywhere else in this
//! crate's test suite; these specifically target the composite/temporal
//! shapes (path, map, list-valued property, duration, time, local-date-
//! time, date-time) and a couple of aggregate error messages.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;

#[test]
fn distinct_by_path() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'a'})-[:KNOWS]->(:Person {name: 'b'})",
    );
    // Both matches of the same 1-hop pattern find the identical path
    // twice over (undirected `--`), so DISTINCT must collapse them.
    let result = run(
        &store,
        "MATCH p = (:Person {name: 'a'})-[:KNOWS]-(:Person {name: 'b'}) RETURN DISTINCT p",
    );
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn distinct_by_map_literal() {
    let store = GraphStore::open_memory().unwrap();
    // Same map, different written key order -- must still collapse to
    // one row (`Value::Map`'s `HashKey` is a `BTreeMap`-ordered
    // canonical encoding, not insertion order).
    let result = run(
        &store,
        "UNWIND [{a: 1, b: 2}, {b: 2, a: 1}, {a: 1, b: 3}] AS m RETURN DISTINCT m",
    );
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn distinct_by_list_valued_property() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Item {tags: ['a', 'b']}), (:Item {tags: ['a', 'b']}), (:Item {tags: ['c']})",
    );
    let result = run(&store, "MATCH (n:Item) RETURN DISTINCT n.tags");
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn distinct_by_duration_time_localdatetime_datetime() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(
        &store,
        "UNWIND [duration('P1D'), duration('P1D'), duration('P2D')] AS d RETURN DISTINCT d",
    );
    assert_eq!(result.rows.len(), 2);

    // Two structurally-different `Time`s at the same UTC-equivalent
    // instant hash equal (`TimeInstant`'s own doc comment).
    let result = run(
        &store,
        "UNWIND [time('12:00+01:00'), time('11:00Z'), time('13:00+02:00')] AS t RETURN DISTINCT t",
    );
    assert_eq!(result.rows.len(), 1);

    let result = run(
        &store,
        "UNWIND [localdatetime('2020-01-01T00:00:00'), localdatetime('2020-01-01T00:00:00'), \
                 localdatetime('2020-01-02T00:00:00')] AS d RETURN DISTINCT d",
    );
    assert_eq!(result.rows.len(), 2);

    // Same instant, different `offset_seconds` -- DateTime equality is
    // instant-only (`DateTimeInstant`'s own doc comment).
    let result = run(
        &store,
        "UNWIND [datetime('2020-01-01T12:00:00+01:00'), datetime('2020-01-01T11:00:00Z')] AS d \
         RETURN DISTINCT d",
    );
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn sum_avg_percentile_report_type_errors_on_non_numeric_input() {
    let store = GraphStore::open_memory().unwrap();
    for (expr, needle) in [
        ("sum(n.name)", "sum()"),
        ("avg(n.name)", "avg()"),
        ("percentileCont(n.name, 0.5)", "percentileCont()"),
        ("percentileDisc(n.name, 0.5)", "percentileDisc()"),
        ("percentileCont(1.0, n.name)", "percentile"),
    ] {
        run(&store, "MATCH (n) DETACH DELETE n");
        run(&store, "CREATE (:N {name: 'not a number'})");
        let stmt = marsdb_query::parse(&format!("MATCH (n:N) RETURN {expr}")).unwrap();
        let err = marsdb_query::Executor::new(&store)
            .execute(&stmt)
            .unwrap_err();
        assert!(
            err.to_string().contains(needle),
            "expected {expr:?}'s error to mention {needle:?}, got: {err}"
        );
    }
}
