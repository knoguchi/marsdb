use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb::Database;

/// `MATCH (n:Item {idx: N}) RETURN n.idx` with and without a declared
/// index on `idx` — the direct payoff of `IndexSeek` over a label scan +
/// filter (`marsdb-query/src/planner.rs::apply_index_seeks`).
fn bench_indexed_vs_unindexed_equality_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality_match_indexed_vs_unindexed");
    for n in [100usize, 1_000, 10_000, 100_000] {
        let target = n / 2;
        let query = format!("MATCH (n:Item {{idx: {target}}}) RETURN n.idx");

        let unindexed = Database::in_memory().unwrap();
        let create = (0..n)
            .map(|i| format!("(:Item {{idx: {i}}})"))
            .collect::<Vec<_>>()
            .join(", ");
        unindexed.execute(&format!("CREATE {create}")).unwrap();
        group.bench_with_input(BenchmarkId::new("unindexed_scan", n), &n, |b, _| {
            b.iter(|| black_box(unindexed.execute(&query).unwrap()));
        });

        let indexed = Database::in_memory().unwrap();
        indexed.execute("CREATE INDEX ON :Item(idx)").unwrap();
        indexed.execute(&format!("CREATE {create}")).unwrap();
        group.bench_with_input(BenchmarkId::new("index_seek", n), &n, |b, _| {
            b.iter(|| black_box(indexed.execute(&query).unwrap()));
        });
    }
    group.finish();
}

/// Two indexed equality conjuncts with wildly different selectivity
/// (`country = 'US'` matches ~100% of rows, `email` matches exactly one)
/// — measures the planner's cardinality-based choice
/// (`GraphStore::index_match_count_in_txn`) against the old "seek on
/// whichever conjunct appears first" behavior, simulated here by
/// declaring only the low-selectivity index so the planner has no better
/// option to pick.
fn bench_cardinality_based_index_selection(c: &mut Criterion) {
    let n = 50_000;
    let target = n / 2;
    let create = format!(
        "CREATE {}",
        (0..n)
            .map(|i| format!("(:Person {{country: 'US', email: 'user{i}@x.com'}})"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let query = format!(
        "MATCH (n:Person) WHERE n.country = 'US' AND n.email = 'user{target}@x.com' RETURN n.email"
    );

    let mut group = c.benchmark_group("cardinality_based_index_selection");

    // Only the low-selectivity index exists -- the planner has no choice,
    // this is what "always seek on the first indexed conjunct" cost.
    let low_selectivity_only = Database::in_memory().unwrap();
    low_selectivity_only
        .execute("CREATE INDEX ON :Person(country)")
        .unwrap();
    low_selectivity_only.execute(&create).unwrap();
    group.bench_function("only_low_selectivity_index_declared", |b| {
        b.iter(|| black_box(low_selectivity_only.execute(&query).unwrap()));
    });

    // Both indexes exist -- the planner picks `email` by real cardinality.
    let both_indexed = Database::in_memory().unwrap();
    both_indexed
        .execute("CREATE INDEX ON :Person(country)")
        .unwrap();
    both_indexed
        .execute("CREATE INDEX ON :Person(email)")
        .unwrap();
    both_indexed.execute(&create).unwrap();
    group.bench_function("both_indexed_cardinality_picks_email", |b| {
        b.iter(|| black_box(both_indexed.execute(&query).unwrap()));
    });

    group.finish();
}

/// `LIMIT` on a non-unique index seek stops the storage-level scan early
/// (`stream_index_seek`'s budget-aware `storage_limit`) instead of
/// materializing every match first — compares a `LIMIT 1` against the
/// unbounded form on the same heavily-duplicated index value.
fn bench_limit_on_nonunique_index_seek(c: &mut Criterion) {
    let mut group = c.benchmark_group("limit_on_nonunique_index_seek");
    for n in [1_000usize, 10_000, 100_000] {
        let db = Database::in_memory().unwrap();
        db.execute("CREATE INDEX ON :Item(city)").unwrap();
        let create = (0..n)
            .map(|i| format!("(:Item {{city: 'Tokyo', idx: {i}}})"))
            .collect::<Vec<_>>()
            .join(", ");
        db.execute(&format!("CREATE {create}")).unwrap();

        group.bench_with_input(BenchmarkId::new("limit_1", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item {city: 'Tokyo'}) RETURN n.idx LIMIT 1")
                        .unwrap(),
                )
            });
        });
        group.bench_with_input(BenchmarkId::new("unbounded", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    db.execute("MATCH (n:Item {city: 'Tokyo'}) RETURN n.idx")
                        .unwrap(),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_indexed_vs_unindexed_equality_match,
    bench_cardinality_based_index_selection,
    bench_limit_on_nonunique_index_seek,
);
criterion_main!(benches);
