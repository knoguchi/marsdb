//! Level-1 crash-safety check: process-crash durability (OS stays up,
//! page cache intact), not power-loss (needs real fault injection, e.g.
//! `dm-flakey`). Proves a killed-mid-write MarsDB file isn't corrupted
//! and never loses an acknowledged commit.
//!
//! Spawns `crash_child` (`src/bin/crash_child.rs`), lets it commit a
//! random number of `CREATE (:Counter {n: N})` transactions, SIGKILLs it
//! at an unpredictable point, then reopens the file and checks that the
//! surviving `Counter.n` values are exactly the contiguous prefix
//! `{1, ..., K}`: no gaps (a partial transaction) and no duplicates (a
//! commit recorded twice). No synchronization with the child's commit
//! completion is needed -- the invariant holds regardless of which
//! commit the kill lands on.
//!
//! Ignored by default (spawns/kills real processes in a loop) -- run:
//!     cargo test -p marsdb-crash-harness --test crash_safety -- --ignored --nocapture

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
#[ignore]
fn kill_mid_write_never_corrupts_or_loses_an_acknowledged_commit() {
    let runs: u32 = std::env::var("CRASH_TEST_RUNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    for run in 0..runs {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("crash.db");

        let mut child = Command::new(env!("CARGO_BIN_EXE_crash_child"))
            .arg(&db_path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn crash_child");

        // Vary the kill point across runs: read a varying number of "OK"
        // lines first, then don't wait for the next one, so kills land
        // at different points in the write path run over run.
        let mut reader = BufReader::new(child.stdout.take().expect("child stdout"));
        let mut line = String::new();
        let commits_before_kill = (run % 7) + 1;
        for _ in 0..commits_before_kill {
            line.clear();
            reader.read_line(&mut line).ok();
        }
        std::thread::sleep(Duration::from_micros((run as u64 % 5) * 200));

        // Child::kill() sends SIGKILL on Unix -- uncatchable, same as a
        // real crash/power-loss from the process's own point of view.
        child.kill().expect("SIGKILL child");
        child.wait().ok();

        let db = marsdb::Database::open(&db_path).expect("reopen after kill must not error");
        let result = db
            .execute("MATCH (c:Counter) RETURN c.n")
            .expect("query after kill");
        let mut values: Vec<i64> = result
            .rows
            .iter()
            .map(|row| match &row[0] {
                marsdb::Value::Property(marsdb::PropertyValue::Int(i)) => *i,
                other => panic!("run {run}: expected an int Counter.n, got {other:?}"),
            })
            .collect();
        values.sort_unstable();

        let expected: Vec<i64> = (1..=values.len() as i64).collect();
        assert_eq!(
            values, expected,
            "run {run}: Counter.n values must be the contiguous prefix 1..=K with no gaps/duplicates, got {values:?}"
        );
    }
}
