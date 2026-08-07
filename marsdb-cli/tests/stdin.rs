//! Piped (non-terminal) stdin runs as a `;`-separated batch, same as the
//! `QUERY` positional argument -- the CLI's own alternative to loading a
//! large script without hitting a single command-line argument's real
//! OS-level length cap (`marsdb`'s CLI has no `-f file` flag).

use std::io::Write;
use std::process::{Command, Stdio};

fn run_stdin(db_path: &str, cypher: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mars"))
        .arg(db_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn marsdb");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(cypher.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("marsdb didn't exit");
    assert!(
        output.status.success(),
        "marsdb exited with {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn piped_stdin_runs_as_a_batch() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("stdin.db");
    let db_path = db_path.to_str().unwrap();

    run_stdin(
        db_path,
        "CREATE (:Person {name: 'Alice'});\nCREATE (:Person {name: 'Bob'});\n",
    );
    let out = run_stdin(db_path, "MATCH (p:Person) RETURN p.name ORDER BY p.name");
    assert!(out.contains("Alice"));
    assert!(out.contains("Bob"));
}

#[test]
fn piped_stdin_survives_a_multi_megabyte_batch() {
    // Past a real single shell argv string's OS-level cap (macOS's
    // ARG_MAX is ~1MB) -- the exact scenario stdin support exists for.
    // Statement *count* stays low (a handful, not thousands) so this
    // stays fast regardless of build profile -- it's proving stdin
    // handles a large *byte size*, not stress-testing execution
    // throughput (that's what the benches are for).
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("bulk.db");
    let db_path = db_path.to_str().unwrap();

    let padding = "x".repeat(2_000);
    let mut script = String::new();
    for i in 0..1_000 {
        script.push_str(&format!("CREATE (:Item {{idx: {i}, pad: '{padding}'}});\n"));
    }
    assert!(script.len() > 1_500_000, "got {} bytes", script.len());
    run_stdin(db_path, &script);

    let out = run_stdin(db_path, "MATCH (n:Item) RETURN count(n)");
    assert!(out.contains("1000"), "got: {out}");
}
