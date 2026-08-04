//! Level-1 crash-safety check: process-crash durability (OS stays up,
//! page cache intact) -- NOT real power-loss (page cache lost too, only
//! `fsync`'d bytes survive). That's a different failure mode needing real
//! fault injection (e.g. `dm-flakey`); this only proves "if the OS
//! survives, a killed-mid-write MarsDB file isn't corrupted and never
//! loses an acknowledged commit."
//!
//! Spawns `crash_child` (see `src/bin/crash_child.rs`), lets it commit a
//! random number of `CREATE (:Counter {n: N})` transactions, SIGKILLs it
//! at an unpredictable point, then reopens the same file fresh and
//! checks the invariant: whatever `Counter.n` values exist must be
//! exactly the contiguous prefix `{1, 2, ..., K}` -- no gaps (a
//! transaction that should have been all-or-nothing left a partial
//! record) and no duplicates (a commit got recorded twice). This doesn't
//! require synchronizing with the child's actual commit completion
//! (racy and unnecessary) -- it only asserts on what's structurally true
//! regardless of exactly which commit the kill landed on.
//!
//! Ignored by default (spawns/kills real processes in a loop, slower
//! than the rest of the suite) -- run explicitly:
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

        // Randomize the kill point across runs -- let a varying number of
        // acknowledged commits happen first, then also race the next one
        // (don't wait for its "OK" line) so kills land at different
        // points in the write path run over run, not always the same one.
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
