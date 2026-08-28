//! Smoke tests for the row-slots fast path (`slots::try_compile_slotted`
//! / `Executor::stream_plan_auto`): correctness of the slot-indexed
//! `Expand`+`Filter` execution, and that a passenger binding the current
//! plan never references (an `OPTIONAL MATCH`'s carried-forward outer
//! variable) survives the `BindingRow` <-> `SlotRow` boundary intact.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use std::collections::BTreeMap;

#[test]
fn multi_hop_expand_with_interior_filter_matches_only_the_qualifying_chain() {
    // a -KNOWS-> b1(age 35) -KNOWS-> c1   (b1.age > 30: kept)
    // a -KNOWS-> b2(age 20) -KNOWS-> c2   (b2.age > 30 is false: dropped)
    // No VarExpand/MatchRelList/general-Expr predicate anywhere in this
    // pattern, so this exercises the slotted Expand+Filter chain, not
    // the legacy HashMap-row fallback.
    let store = GraphStore::open_memory().unwrap();
    let a = store
        .create_node(&["Person"], props(&[("name", "Alice")]))
        .unwrap();
    let b1 = store
        .create_node(&["Person"], props_int(&[("age", 35)], &[("name", "Bob")]))
        .unwrap();
    let c1 = store
        .create_node(&["Person"], props(&[("name", "Carol")]))
        .unwrap();
    let b2 = store
        .create_node(&["Person"], props_int(&[("age", 20)], &[("name", "Dave")]))
        .unwrap();
    let c2 = store
        .create_node(&["Person"], props(&[("name", "Eve")]))
        .unwrap();
    store.create_edge("KNOWS", a, b1, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", b1, c1, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", a, b2, BTreeMap::new()).unwrap();
    store.create_edge("KNOWS", b2, c2, BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person) \
         WHERE b.age > 30 \
         RETURN a.name, b.name, c.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Bob");
    assert_eq!(str_value(&result.rows[0][2]), "Carol");
}

#[test]
fn optional_match_preserves_a_carried_variable_its_own_pattern_never_references() {
    // `extra` is bound alongside `p` before the OPTIONAL MATCH and is
    // never mentioned by the OPTIONAL pattern `(p)-[:LIKES]->(x:Thing)`
    // itself -- exactly the passenger-binding shape `try_compile_slotted`
    // must seed from the incoming row's own keys rather than only from
    // names the plan references, or `extra` would be silently dropped on
    // the SlotRow round trip. Two `p`s: one with a LIKES edge (OPTIONAL
    // finds a row), one without (OPTIONAL null-pads `x`) -- `extra` must
    // come through correctly either way.
    let store = GraphStore::open_memory().unwrap();
    let p_with = store
        .create_node(&["Person"], props(&[("name", "HasLike")]))
        .unwrap();
    let thing = store
        .create_node(&["Thing"], props(&[("name", "Widget")]))
        .unwrap();
    store
        .create_edge("LIKES", p_with, thing, BTreeMap::new())
        .unwrap();
    store
        .create_node(&["Person"], props(&[("name", "NoLike")]))
        .unwrap();
    store
        .create_node(&["Extra"], props(&[("tag", "kept")]))
        .unwrap();

    let result = run(
        &store,
        "MATCH (p:Person), (extra:Extra) \
         WITH p, extra \
         OPTIONAL MATCH (p)-[:LIKES]->(x:Thing) \
         RETURN p.name, extra.tag, x.name \
         ORDER BY p.name",
    );
    assert_eq!(result.rows.len(), 2);
    // "HasLike" sorts before "NoLike".
    assert_eq!(str_value(&result.rows[0][0]), "HasLike");
    assert_eq!(str_value(&result.rows[0][1]), "kept");
    assert_eq!(str_value(&result.rows[0][2]), "Widget");
    assert_eq!(str_value(&result.rows[1][0]), "NoLike");
    assert_eq!(str_value(&result.rows[1][1]), "kept");
    assert!(matches!(result.rows[1][2], marsdb_query::Value::Null));
}

#[test]
fn named_path_shortest_path_and_merge_are_unaffected() {
    // These all disqualify (or bypass) slot compilation entirely --
    // named path and shortestPath have no `LogicalPlan` at all, and
    // `MERGE` plans per-row via `IndexSeekValue::RowExpr`. Regression
    // guard that `stream_plan_auto` correctly falls back to the
    // unmodified legacy path for each rather than silently producing
    // wrong rows.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );

    let named_path = run(&store, "MATCH p = (a:Person)-->(b:Person) RETURN length(p)");
    assert_eq!(named_path.rows.len(), 1);
    assert_eq!(int_value(&named_path.rows[0][0]), 1);

    let shortest = run(
        &store,
        "MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'}) \
         MATCH p = shortestPath((a)-[*]-(b)) RETURN length(p)",
    );
    assert_eq!(shortest.rows.len(), 1);
    assert_eq!(int_value(&shortest.rows[0][0]), 1);

    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) MERGE (a)-[:KNOWS]->(c:Person {name: 'Carol'})",
    );
    let merged = run(
        &store,
        "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(x) RETURN count(*)",
    );
    assert_eq!(int_value(&merged.rows[0][0]), 2);
}

fn props(pairs: &[(&str, &str)]) -> BTreeMap<String, marsdb_graph::PropertyValue> {
    pairs
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                marsdb_graph::PropertyValue::String(v.to_string()),
            )
        })
        .collect()
}

fn props_int(
    int_pairs: &[(&str, i64)],
    str_pairs: &[(&str, &str)],
) -> BTreeMap<String, marsdb_graph::PropertyValue> {
    let mut map: BTreeMap<String, marsdb_graph::PropertyValue> = int_pairs
        .iter()
        .map(|(k, v)| (k.to_string(), marsdb_graph::PropertyValue::Int(*v)))
        .collect();
    for (k, v) in str_pairs {
        map.insert(
            k.to_string(),
            marsdb_graph::PropertyValue::String(v.to_string()),
        );
    }
    map
}
