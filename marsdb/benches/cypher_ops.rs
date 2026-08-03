use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb::Database;

fn make_chain_create(hops: usize) -> String {
    let mut chain = "(n0:Item {idx: 0})".to_string();
    for i in 1..=hops {
        chain.push_str(&format!("-[:R]->(n{i}:Item {{idx: {i}}})"));
    }
    format!("CREATE {chain}")
}

/// Isolates parsing cost from execution cost — answers the "where does the
/// time go" question cleanly, in-process (no subprocess/CLI confounders).
fn bench_parse_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_only_by_hop_count");
    for hops in [10usize, 100, 1_000] {
        let cypher = make_chain_create(hops);
        group.bench_with_input(BenchmarkId::from_parameter(hops), &hops, |b, _| {
            b.iter(|| black_box(marsdb_query::parse(&cypher).unwrap()));
        });
    }
    group.finish();
}

fn bench_execute_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute_create_by_hop_count");
    for hops in [10usize, 100, 1_000] {
        let cypher = make_chain_create(hops);
        group.bench_with_input(BenchmarkId::from_parameter(hops), &hops, |b, _| {
            b.iter(|| {
                let db = Database::in_memory().unwrap();
                black_box(db.execute(&cypher).unwrap())
            });
        });
    }
    group.finish();
}

fn bench_execute_match_1hop(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute_match_1hop_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = Database::in_memory().unwrap();
        db.execute(&make_chain_create(n)).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item)-[:R]->(m:Item) RETURN m.idx LIMIT 10")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_only,
    bench_execute_create,
    bench_execute_match_1hop,
);
criterion_main!(benches);
