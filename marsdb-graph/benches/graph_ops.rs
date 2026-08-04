use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb_graph::{Direction, GraphStore, PropertyValue};

fn bench_create_node(c: &mut Criterion) {
    let store = GraphStore::open_memory().unwrap();
    c.bench_function("create_node", |b| {
        b.iter(|| {
            let mut props = BTreeMap::new();
            props.insert("name".to_string(), PropertyValue::String("x".to_string()));
            black_box(store.create_node(&["Item"], props).unwrap())
        });
    });
}

fn bench_create_edge(c: &mut Criterion) {
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    let b_node = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    c.bench_function("create_edge", |b| {
        b.iter(|| {
            black_box(
                store
                    .create_edge("REL", a, b_node, BTreeMap::new())
                    .unwrap(),
            )
        });
    });
}

fn bench_get_node(c: &mut Criterion) {
    let store = GraphStore::open_memory().unwrap();
    let id = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    c.bench_function("get_node", |b| {
        b.iter(|| black_box(store.get_node(id).unwrap()));
    });
}

fn bench_neighbors_1hop(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbors_1hop_by_fanout");
    for fanout in [1u64, 10, 100, 1_000] {
        let store = GraphStore::open_memory().unwrap();
        let center = store.create_node(&["Item"], BTreeMap::new()).unwrap();
        for _ in 0..fanout {
            let n = store.create_node(&["Item"], BTreeMap::new()).unwrap();
            store
                .create_edge("REL", center, n, BTreeMap::new())
                .unwrap();
        }
        group.bench_with_input(BenchmarkId::from_parameter(fanout), &fanout, |b, _| {
            b.iter(|| black_box(store.neighbors(center, Direction::Out, None).unwrap()));
        });
    }
    group.finish();
}

fn bench_all_nodes_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_nodes_scan_by_table_size");
    for n in [100i64, 1_000, 10_000] {
        let store = GraphStore::open_memory().unwrap();
        for i in 0..n {
            let mut props = BTreeMap::new();
            props.insert("idx".to_string(), PropertyValue::Int(i));
            store.create_node(&["Item"], props).unwrap();
        }
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(store.all_nodes(Some("Item")).unwrap()));
        });
    }
    group.finish();
}

/// `bench_all_nodes_scan` above is 100% selectivity (every node is `Item`)
/// — the worst case for an index, since it still pays a per-row point
/// lookup instead of one sequential scan. This is the case the index is
/// actually for: 1 in 100 nodes carries the target label, so most of the
/// table is irrelevant to the query.
fn bench_all_nodes_scan_low_selectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_nodes_scan_1pct_selectivity_by_table_size");
    for n in [100i64, 1_000, 10_000, 100_000] {
        let store = GraphStore::open_memory().unwrap();
        for i in 0..n {
            let mut props = BTreeMap::new();
            props.insert("idx".to_string(), PropertyValue::Int(i));
            let label = if i % 100 == 0 { "Target" } else { "Other" };
            store.create_node(&[label], props).unwrap();
        }
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(store.all_nodes(Some("Target")).unwrap()));
        });
    }
    group.finish();
}

/// Follow-up on an earlier open question: does batching many writes into
/// one transaction (via the `*_in_txn` API the query executor uses) matter
/// versus one transaction per node? Answers it directly instead of guessing.
fn bench_bulk_create_txn_strategy(c: &mut Criterion) {
    const N: i64 = 1_000;
    let mut group = c.benchmark_group("bulk_create_1000_nodes");

    group.bench_function("one_txn_per_node", |b| {
        b.iter(|| {
            let store = GraphStore::open_memory().unwrap();
            for i in 0..N {
                let mut props = BTreeMap::new();
                props.insert("idx".to_string(), PropertyValue::Int(i));
                store.create_node(&["Item"], props).unwrap();
            }
        });
    });

    group.bench_function("one_txn_total", |b| {
        b.iter(|| {
            let store = GraphStore::open_memory().unwrap();
            let write_txn = store.begin_write().unwrap();
            for i in 0..N {
                let mut props = BTreeMap::new();
                props.insert("idx".to_string(), PropertyValue::Int(i));
                GraphStore::create_node_in_txn(&write_txn, &["Item"], props).unwrap();
            }
            GraphStore::commit(write_txn).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_create_node,
    bench_create_edge,
    bench_get_node,
    bench_neighbors_1hop,
    bench_all_nodes_scan,
    bench_all_nodes_scan_low_selectivity,
    bench_bulk_create_txn_strategy,
);
criterion_main!(benches);
