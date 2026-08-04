//! Benchmarks the actual point of concurrent reads: N `MATCH` queries done
//! serially on one thread vs. the same N queries spread across multiple
//! threads against one shared `Arc<Database>`. `MATCH ... RETURN` opens a
//! `ReadTransaction` (see `Executor::execute`/`is_read_only`), so these
//! should run in parallel instead of queueing behind redb's single-writer
//! lock the way every statement did before this feature.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marsdb::Database;

const TOTAL_READS: usize = 200;

fn fixture(n: usize) -> Arc<Database> {
    let mut patterns = Vec::with_capacity(n);
    for i in 0..n {
        patterns.push(format!("(n{i}:Item {{idx: {i}}})"));
    }
    let db = Database::in_memory().unwrap();
    db.execute(&format!("CREATE {}", patterns.join(", ")))
        .unwrap();
    Arc::new(db)
}

fn run_reads(db: &Database, count: usize) {
    for _ in 0..count {
        black_box(db.execute("MATCH (n:Item) RETURN n.idx").unwrap());
    }
}

fn bench_sequential_vs_concurrent_reads(c: &mut Criterion) {
    let db = fixture(1_000);
    let mut group = c.benchmark_group("reads_by_thread_count");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(3));

    group.bench_function(BenchmarkId::from_parameter("1_thread_sequential"), |b| {
        b.iter(|| run_reads(&db, TOTAL_READS));
    });

    for threads in [2usize, 4, 8] {
        group.bench_function(
            BenchmarkId::from_parameter(format!("{threads}_threads")),
            |b| {
                b.iter(|| {
                    let per_thread = TOTAL_READS / threads;
                    thread::scope(|s| {
                        for _ in 0..threads {
                            let db = Arc::clone(&db);
                            s.spawn(move || run_reads(&db, per_thread));
                        }
                    });
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_sequential_vs_concurrent_reads);
criterion_main!(benches);
