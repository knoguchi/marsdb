//! `lookup_by_index_range_in_txn` — order-preserving index range scans.
//! Contract: superset for numeric bounds (both int and float regions,
//! lossy conversions widened outward), single-region for non-numerics,
//! index order within a region.

use std::collections::BTreeMap;

use marsdb_graph::{GraphStore, NodeId, PropertyValue, Txn};

fn seeded() -> (GraphStore, Vec<NodeId>) {
    let store = GraphStore::open_memory().unwrap();
    store.create_index("N", "v", false).unwrap();
    let mut ids = Vec::new();
    for v in [
        PropertyValue::Int(1),
        PropertyValue::Int(5),
        PropertyValue::Int(10),
        PropertyValue::Float(4.5),
        PropertyValue::Float(7.5),
        PropertyValue::String("apple".into()),
        PropertyValue::String("banana".into()),
    ] {
        let mut props = BTreeMap::new();
        props.insert("v".to_string(), v);
        ids.push(store.create_node(&["N"], props).unwrap());
    }
    (store, ids)
}

fn range(
    store: &GraphStore,
    lo: Option<(&PropertyValue, bool)>,
    hi: Option<(&PropertyValue, bool)>,
) -> Vec<u64> {
    let read = store.begin_read().unwrap();
    let mut got: Vec<u64> =
        GraphStore::lookup_by_index_range_in_txn(Txn::Read(&read), "N", "v", lo, hi, None)
            .unwrap()
            .into_iter()
            .map(|id| id.0)
            .collect();
    got.sort_unstable();
    got
}

#[test]
fn int_bound_scans_both_numeric_regions() {
    let (store, ids) = seeded();
    // v > 4 (int, exclusive): ints 5, 10 and floats 4.5, 7.5.
    let got = range(&store, Some((&PropertyValue::Int(4), false)), None);
    let expected: Vec<u64> = [1, 2, 3, 4].iter().map(|&i| ids[i].0).collect();
    assert_eq!(got, expected);
}

#[test]
fn float_bounds_widen_over_the_int_region() {
    let (store, ids) = seeded();
    // 4.4 <= v <= 7.6: exact matches are 5, 4.5, 7.5 -- the superset may
    // also include the widened ints 4..=8, but must contain the true set.
    let got = range(
        &store,
        Some((&PropertyValue::Float(4.4), true)),
        Some((&PropertyValue::Float(7.6), true)),
    );
    for want in [ids[1].0, ids[3].0, ids[4].0] {
        assert!(got.contains(&want), "missing {want} in {got:?}");
    }
    // And never anything outside the widened envelope.
    assert!(!got.contains(&ids[0].0)); // 1
    assert!(!got.contains(&ids[2].0)); // 10
    assert!(!got.contains(&ids[5].0)); // strings never in a numeric range
}

#[test]
fn string_bounds_scan_only_the_string_region() {
    let (store, ids) = seeded();
    let apple = PropertyValue::String("apple".into());
    let got = range(&store, Some((&apple, false)), None);
    assert_eq!(got, vec![ids[6].0]); // banana only, no numerics
    let got = range(&store, Some((&apple, true)), None);
    assert_eq!(got, vec![ids[5].0, ids[6].0]);
}

#[test]
fn limit_stops_early_and_unindexed_is_empty() {
    let (store, _) = seeded();
    let read = store.begin_read().unwrap();
    let got = GraphStore::lookup_by_index_range_in_txn(
        Txn::Read(&read),
        "N",
        "v",
        Some((&PropertyValue::Int(0), true)),
        None,
        Some(2),
    )
    .unwrap();
    assert_eq!(got.len(), 2);
    let none = GraphStore::lookup_by_index_range_in_txn(
        Txn::Read(&read),
        "N",
        "nope",
        Some((&PropertyValue::Int(0), true)),
        None,
        None,
    )
    .unwrap();
    assert!(none.is_empty());
}
