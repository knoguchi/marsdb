//! Smoke tests for property-index encoding (`marsdb-graph/src/index.rs`'s
//! `encode_index_value`) on types no other index test in this crate's
//! suite happens to index: `Duration`, `Time`, `LocalDateTime`,
//! `DateTime`, and a scalar `List`. Every other `CREATE INDEX` test uses
//! a string or int/float property.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;

#[test]
fn index_seek_on_duration_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Task(estimate)");
    run(
        &store,
        "CREATE (:Task {name: 'a', estimate: duration('P1D')}), \
                (:Task {name: 'b', estimate: duration('P2D')})",
    );
    let explained = run(
        &store,
        "EXPLAIN MATCH (n:Task) WHERE n.estimate = duration('P1D') RETURN n.name",
    );
    assert!(plan_lines(&explained)
        .iter()
        .any(|l| l.contains("IndexSeek(n:Task") && l.contains("estimate")));

    let result = run(
        &store,
        "MATCH (n:Task) WHERE n.estimate = duration('P1D') RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}

#[test]
fn index_seek_on_time_property_by_utc_instant() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Event(startsAt)");
    run(
        &store,
        "CREATE (:Event {name: 'a', startsAt: time('12:00+01:00')}), \
                (:Event {name: 'b', startsAt: time('15:00+01:00')})",
    );
    // A structurally different `Time` at the same UTC-equivalent instant
    // must still hit the index (encoded by instant, not raw wall-clock
    // fields -- see `encode_index_value`'s own `Time` doc comment).
    let result = run(
        &store,
        "MATCH (n:Event) WHERE n.startsAt = time('11:00Z') RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}

#[test]
fn index_seek_on_localdatetime_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Event(loggedAt)");
    run(
        &store,
        "CREATE (:Event {name: 'a', loggedAt: localdatetime('2020-01-01T00:00:00')}), \
                (:Event {name: 'b', loggedAt: localdatetime('2021-01-01T00:00:00')})",
    );
    let result = run(
        &store,
        "MATCH (n:Event) WHERE n.loggedAt = localdatetime('2020-01-01T00:00:00') RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}

#[test]
fn index_seek_on_datetime_property_ignores_offset() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Event(occurredAt)");
    run(
        &store,
        "CREATE (:Event {name: 'a', occurredAt: datetime('2020-01-01T12:00:00+01:00')})",
    );
    // Same instant, different written offset -- DateTime equality/
    // indexing is instant-only (`epoch_seconds`/`nanos`, offset excluded).
    let result = run(
        &store,
        "MATCH (n:Event) WHERE n.occurredAt = datetime('2020-01-01T11:00:00Z') RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}

#[test]
fn index_seek_on_scalar_list_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Item(tags)");
    run(
        &store,
        "CREATE (:Item {name: 'a', tags: ['x', 'y']}), (:Item {name: 'b', tags: ['x']})",
    );
    let result = run(
        &store,
        "MATCH (n:Item) WHERE n.tags = ['x', 'y'] RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}
