//! Whole-workload benchmarks over the deterministic LDBC-style social
//! network (`tests/ldbc_support/mod.rs`, a third-party-derived imitation
//! of LDBC SNB, not the official benchmark). Unlike `ldbc_ops.rs`'s
//! per-feature micro-benchmarks on synthetic chains, these run the
//! suite's 17 queries against the full SF 0.1 graph (~16k nodes / ~115k
//! relationships, property indexes on `id`).
//!
//! Every iteration re-executes the query in full: MarsDB has no query
//! result cache, so these are cold-execution numbers, not comparable to
//! an engine that memoizes repeated reads.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use marsdb::Database;

#[path = "../tests/ldbc_support/mod.rs"]
mod ldbc_support;

fn bench_workload(c: &mut Criterion) {
    let db = Database::in_memory().unwrap();
    ldbc_support::load(&db, 0.1);

    let mut group = c.benchmark_group("ldbc_style_sf01");
    // Bounded: IC1's var-length traversal alone runs ~500ms/execution.
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    for (name, query) in ldbc_support::BENCH_QUERIES {
        group.bench_function(*name, |b| {
            b.iter(|| black_box(db.execute(query).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_workload);
criterion_main!(benches);
