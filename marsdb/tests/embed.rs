//! Proves the crate is embeddable independent of the CLI subprocess.

use marsdb::Database;

#[test]
fn in_memory_roundtrip() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
        .unwrap();
    let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn file_backed_persists_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");

    {
        let db = Database::open(&path).unwrap();
        db.execute("CREATE (a:Person {name: 'Alice'})").unwrap();
    }

    let db = Database::open(&path).unwrap();
    let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn two_in_memory_databases_do_not_share_data() {
    let a = Database::in_memory().unwrap();
    let b = Database::in_memory().unwrap();
    a.execute("CREATE (n:Person {name: 'Alice'})").unwrap();
    let result = b.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn execute_with_params_substitutes_dollar_placeholders() {
    use std::collections::HashMap;

    let db = Database::in_memory().unwrap();
    db.execute("CREATE (n:Person {personId: 42, name: 'Alice'})").unwrap();

    let mut params = HashMap::new();
    params.insert("personId".to_string(), marsdb::PropertyValue::Int(42));
    let result = db
        .execute_with_params("MATCH (n:Person {personId: $personId}) RETURN n.name", &params)
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    // Missing param -> clean error, not a panic.
    let err = db
        .execute_with_params("MATCH (n:Person {personId: $missing}) RETURN n.name", &HashMap::new())
        .unwrap_err();
    assert!(err.to_string().contains("missing"));
}

#[test]
fn execute_batch_runs_each_statement_and_returns_one_result_per_statement() {
    let db = Database::in_memory().unwrap();
    let results = db
        .execute_batch(
            "CREATE (a:Person {name: 'Alice'}); \
             CREATE (b:Person {name: 'Bob'}); \
             MATCH (n:Person) RETURN n.name",
        )
        .unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].columns.is_empty(), "CREATE returns no columns");
    assert!(results[1].columns.is_empty());
    assert_eq!(results[2].rows.len(), 2, "both prior CREATEs must be visible to the final MATCH");
}

#[test]
fn execute_batch_semicolon_inside_string_literal_does_not_split() {
    let db = Database::in_memory().unwrap();
    let results = db.execute_batch("CREATE (n:Item {name: 'a;b'}); MATCH (n:Item) RETURN n.name").unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[1].rows.len(), 1);
}

#[test]
fn execute_batch_bad_syntax_anywhere_runs_nothing() {
    let db = Database::in_memory().unwrap();
    let err = db.execute_batch("CREATE (a:Item); NOT VALID CYPHER").unwrap_err();
    assert!(matches!(err, marsdb::Error::Query(_)));
    // Nothing committed -- the parse error was caught before any statement ran.
    let result = db.execute("MATCH (n:Item) RETURN n").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn execute_batch_stops_at_first_runtime_failure_but_keeps_earlier_commits() {
    let db = Database::in_memory().unwrap();
    let err = db
        .execute_batch("CREATE (a:Item {idx: 1}); MATCH (missing) DELETE nonexistent; CREATE (b:Item {idx: 2})")
        .unwrap_err();
    assert!(matches!(err, marsdb::Error::Query(_)));
    let result = db.execute("MATCH (n:Item) RETURN n.idx").unwrap();
    assert_eq!(result.rows.len(), 1, "the first CREATE must stay committed even though a later statement failed");
}

/// `MATCH ... RETURN` opens a `ReadTransaction`, not a `WriteTransaction`
/// (see `Executor::execute`/`is_read_only`) -- proves that path is actually
/// thread-safe under real concurrent access, not just single-threaded.
#[test]
fn concurrent_reads_from_multiple_threads_all_see_correct_results() {
    use std::sync::Arc;
    use std::thread;

    let db = Arc::new(Database::in_memory().unwrap());
    for i in 0..200 {
        db.execute(&format!("CREATE (n:Item {{idx: {i}}})")).unwrap();
    }

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                for _ in 0..50 {
                    let result = db.execute("MATCH (n:Item) RETURN n.idx").unwrap();
                    assert_eq!(result.rows.len(), 200, "every concurrent reader must see all 200 rows");
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

/// A writer committing new nodes while readers are concurrently querying
/// must never panic, deadlock, or hand a reader a torn/partial view --
/// each reader's `ReadTransaction` snapshot is either fully before or
/// fully after any given writer commit (redb's MVCC guarantee), so every
/// observed row count must be one of the values the writer actually
/// passed through, never something in between within one CREATE.
#[test]
fn concurrent_write_and_reads_never_panic_or_see_torn_state() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    let db = Arc::new(Database::in_memory().unwrap());
    let stop = Arc::new(AtomicBool::new(false));

    let writer = {
        let db = Arc::clone(&db);
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            for i in 0..300 {
                db.execute(&format!("CREATE (n:Item {{idx: {i}}})")).unwrap();
            }
            stop.store(true, Ordering::SeqCst);
        })
    };

    let readers: Vec<_> = (0..4)
        .map(|_| {
            let db = Arc::clone(&db);
            let stop = Arc::clone(&stop);
            thread::spawn(move || {
                let mut last_count = 0usize;
                while !stop.load(Ordering::SeqCst) {
                    let result = db.execute("MATCH (n:Item) RETURN n").unwrap();
                    assert!(
                        result.rows.len() >= last_count,
                        "row count must never go backwards mid-write (would mean a torn/inconsistent read)"
                    );
                    last_count = result.rows.len();
                }
            })
        })
        .collect();

    writer.join().unwrap();
    for r in readers {
        r.join().unwrap();
    }

    let result = db.execute("MATCH (n:Item) RETURN n").unwrap();
    assert_eq!(result.rows.len(), 300);
}
