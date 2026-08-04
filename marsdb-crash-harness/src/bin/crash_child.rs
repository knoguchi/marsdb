//! The victim process for `tests/crash_safety.rs` -- opens a file-backed
//! database and commits one `CREATE (:Counter {n: N})` per transaction in
//! an unbounded loop, printing `OK N` (and flushing) right after each
//! commit returns. The parent test SIGKILLs this process at an
//! unpredictable point and checks what actually landed on disk -- this
//! binary's only job is to keep committing until it's killed, nothing
//! more.

use std::io::Write;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: crash_child <db-path>");
    let db = marsdb::Database::open(&path).expect("open db");
    let stdout = std::io::stdout();
    let mut n: i64 = 1;
    loop {
        db.execute(&format!("CREATE (:Counter {{n: {n}}})"))
            .expect("create");
        let mut handle = stdout.lock();
        writeln!(handle, "OK {n}").ok();
        handle.flush().ok();
        n += 1;
    }
}
