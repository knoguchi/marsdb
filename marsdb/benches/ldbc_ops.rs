//! Benchmarks for the Cypher features added during the LDBC SNB Interactive
//! (IS1-IS7) push: WITH-chaining, OPTIONAL MATCH, undirected patterns, and
//! variable-length patterns. `cypher_ops.rs` only covers CREATE and a plain
//! 1-hop MATCH, which none of these exercise.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb::Database;

/// `(n0:Item {idx:0})-[:R]->(n1:Item {idx:1})-> ... ->(n_hops:Item {idx:hops})`,
/// same shape as `cypher_ops.rs::make_chain_create` so results are comparable.
fn chain_db(hops: usize) -> Database {
    let mut chain = "(n0:Item {idx: 0})".to_string();
    for i in 1..=hops {
        chain.push_str(&format!("-[:R]->(n{i}:Item {{idx: {i}}})"));
    }
    let db = Database::in_memory().unwrap();
    db.execute(&format!("CREATE {chain}")).unwrap();
    db
}

fn bench_with_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_chaining_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = chain_db(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute(
                        "MATCH (n:Item)-[:R]->(m:Item) \
                         WITH m, m.idx AS idx ORDER BY idx DESC LIMIT 10 \
                         MATCH (m)-[:R]->(k:Item) \
                         RETURN idx, k.idx",
                    )
                    .unwrap(),
                )
            });
        });
    }
    group.finish();
}

fn bench_optional_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("optional_match_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = chain_db(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item) OPTIONAL MATCH (n)-[:R]->(m:Item) RETURN n.idx, m.idx")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

fn bench_undirected_1hop(c: &mut Criterion) {
    let mut group = c.benchmark_group("undirected_1hop_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = chain_db(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item)-[:R]-(m:Item) RETURN m.idx LIMIT 10")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

fn bench_variable_length(c: &mut Criterion) {
    // 1,000-node chain for bounded ranges — the bound stops the BFS well
    // short of the depth cap regardless of chain length. Unbounded (`*0..`)
    // needs its own chain within the 30-hop safety cap (see
    // `executor.rs::VAR_EXPAND_DEPTH_CAP`), or it errors by design instead
    // of silently truncating.
    let db_1000 = chain_db(1_000);
    let db_25 = chain_db(25);
    let mut group = c.benchmark_group("variable_length_by_max_hops");
    for (label, db, cypher) in [
        ("1..5", &db_1000, "MATCH (n:Item {idx: 0})-[:R*1..5]->(m:Item) RETURN m.idx"),
        ("1..30", &db_1000, "MATCH (n:Item {idx: 0})-[:R*1..30]->(m:Item) RETURN m.idx"),
        ("0..unbounded_25node_chain", &db_25, "MATCH (n:Item {idx: 0})-[:R*0..]->(m:Item) RETURN m.idx"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &cypher, |b, cypher| {
            b.iter(|| black_box(db.execute(cypher).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_with_chaining,
    bench_optional_match,
    bench_undirected_1hop,
    bench_variable_length,
);
criterion_main!(benches);
