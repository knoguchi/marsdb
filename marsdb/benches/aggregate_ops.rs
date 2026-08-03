//! Benchmarks for aggregation (`count`/`sum`/`avg`/`collect`, `DISTINCT`,
//! implicit GROUP BY, `WITH...WHERE`). `resolve_grouped_rows` does a linear
//! scan over the groups formed so far, not a hash lookup (see its doc
//! comment in executor.rs) — these benchmarks are here specifically to show
//! that cost, not just confirm aggregation works.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb::Database;

/// `n` `Item` nodes, `idx` 0..n, `cat` = `idx % num_groups` (so
/// `num_groups` controls how many distinct groups a `GROUP BY cat` query
/// forms). One `CREATE` with `n` comma-separated patterns -- one
/// transaction for the whole fixture, not `n` of them.
fn items_db(n: usize, num_groups: usize) -> Database {
    let mut patterns = Vec::with_capacity(n);
    for i in 0..n {
        let cat = i % num_groups;
        patterns.push(format!("(n{i}:Item {{idx: {i}, cat: {cat}}})"));
    }
    let db = Database::in_memory().unwrap();
    db.execute(&format!("CREATE {}", patterns.join(", "))).unwrap();
    db
}

fn bench_global_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("global_aggregate_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, 1);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item) RETURN count(*), sum(n.idx), avg(n.idx), min(n.idx), max(n.idx)")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

fn bench_grouped_10_groups(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouped_aggregate_10_groups_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, 10);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item) WITH n.cat AS cat, count(n) AS c RETURN cat, c ORDER BY cat")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

/// Worst case for the linear group scan: every row is its own group (`cat`
/// = `idx`, all distinct), so group lookup is O(rows) per row, O(rows^2)
/// total -- this is the number the "hash-based grouping key" roadmap item
/// (see README) would fix.
fn bench_grouped_all_distinct(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouped_aggregate_all_distinct_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item) WITH n.cat AS cat, count(n) AS c RETURN cat, c").unwrap(),
                )
            });
        });
    }
    group.finish();
}

fn bench_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("collect_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, 1);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(db.execute("MATCH (n:Item) RETURN collect(n.idx)").unwrap()));
        });
    }
    group.finish();
}

/// `count(DISTINCT n.cat)` with `cat` all-distinct -- same O(rows) "seen"
/// list scan per row as the all-distinct grouping case above, but inside
/// a single accumulator instead of the group-lookup path.
fn bench_count_distinct_all_distinct(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_distinct_all_distinct_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(db.execute("MATCH (n:Item) RETURN count(DISTINCT n.cat)").unwrap()));
        });
    }
    group.finish();
}

fn bench_with_where_on_aggregate(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_where_on_aggregate_10_groups_by_dataset_size");
    for n in [100usize, 1_000, 10_000] {
        let db = items_db(n, 10);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute(
                        "MATCH (n:Item) WITH n.cat AS cat, count(n) AS c WHERE c > 1 RETURN cat, c ORDER BY cat",
                    )
                    .unwrap(),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_global_aggregate,
    bench_grouped_10_groups,
    bench_grouped_all_distinct,
    bench_collect,
    bench_count_distinct_all_distinct,
    bench_with_where_on_aggregate,
);
criterion_main!(benches);
