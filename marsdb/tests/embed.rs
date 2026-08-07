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
    db.execute("CREATE (n:Person {personId: 42, name: 'Alice'})")
        .unwrap();

    let mut params = HashMap::new();
    params.insert("personId".to_string(), marsdb::PropertyValue::Int(42));
    let result = db
        .execute_with_params(
            "MATCH (n:Person {personId: $personId}) RETURN n.name",
            &params,
        )
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    // Missing param -> clean error, not a panic.
    let err = db
        .execute_with_params(
            "MATCH (n:Person {personId: $missing}) RETURN n.name",
            &HashMap::new(),
        )
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
    assert_eq!(
        results[2].rows.len(),
        2,
        "both prior CREATEs must be visible to the final MATCH"
    );
}

#[test]
fn execute_batch_semicolon_inside_string_literal_does_not_split() {
    let db = Database::in_memory().unwrap();
    let results = db
        .execute_batch("CREATE (n:Item {name: 'a;b'}); MATCH (n:Item) RETURN n.name")
        .unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[1].rows.len(), 1);
}

#[test]
fn execute_batch_bad_syntax_anywhere_runs_nothing() {
    let db = Database::in_memory().unwrap();
    let err = db
        .execute_batch("CREATE (a:Item); NOT VALID CYPHER")
        .unwrap_err();
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
    assert_eq!(
        result.rows.len(),
        1,
        "the first CREATE must stay committed even though a later statement failed"
    );
}

#[test]
fn explicit_transaction_commits_multiple_statements_atomically() {
    let db = Database::in_memory().unwrap();
    let mut tx = db.begin_transaction().unwrap();

    tx.execute("CREATE (a:Item {idx: 1})").unwrap();
    tx.execute("CREATE (b:Item {idx: 2})").unwrap();
    let inside = tx.execute("MATCH (n:Item) RETURN n.idx").unwrap();
    assert_eq!(
        inside.rows.len(),
        2,
        "a transaction must read its own writes"
    );

    tx.commit().unwrap();
    let after = db.execute("MATCH (n:Item) RETURN n.idx").unwrap();
    assert_eq!(after.rows.len(), 2);
}

#[test]
fn explicit_transaction_rollback_discards_all_statements() {
    let db = Database::in_memory().unwrap();
    let mut tx = db.begin_transaction().unwrap();
    tx.execute("CREATE (n:Item {idx: 1})").unwrap();
    tx.rollback().unwrap();

    let result = db.execute("MATCH (n:Item) RETURN n").unwrap();
    assert!(result.rows.is_empty());
}

#[test]
fn explicit_transaction_statement_error_aborts_and_closes_transaction() {
    let db = Database::in_memory().unwrap();
    let mut tx = db.begin_transaction().unwrap();
    tx.execute("CREATE (n:Item {idx: 1})").unwrap();

    let err = tx.execute("RETURN missing").unwrap_err();
    assert!(matches!(err, marsdb::Error::Query(_)));
    assert!(matches!(
        tx.execute("CREATE (n:Item {idx: 2})"),
        Err(marsdb::Error::TransactionClosed)
    ));
    assert!(matches!(tx.commit(), Err(marsdb::Error::TransactionClosed)));

    let result = db.execute("MATCH (n:Item) RETURN n").unwrap();
    assert!(
        result.rows.is_empty(),
        "a failed statement must abort earlier writes in the transaction"
    );
}

#[test]
fn explicit_transaction_supports_parameters() {
    use std::collections::HashMap;

    let db = Database::in_memory().unwrap();
    let mut params = HashMap::new();
    params.insert("idx".to_string(), marsdb::PropertyValue::Int(42));

    let mut tx = db.begin_transaction().unwrap();
    tx.execute_with_params("CREATE (n:Item {idx: $idx})", &params)
        .unwrap();
    let result = tx
        .execute_with_params("MATCH (n:Item {idx: $idx}) RETURN n", &params)
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    tx.commit().unwrap();
}

#[test]
fn backup_is_queryable_and_preserves_relationships() {
    let source = Database::in_memory().unwrap();
    source
        .execute(
            "CREATE (a:Person {name: 'Alice'})-[:KNOWS {since: 2020}]->(b:Person {name: 'Bob'})",
        )
        .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.redb");
    source.backup_to(&path).unwrap();

    let backup = Database::open(&path).unwrap();
    let result = backup
        .execute("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN a.name, r.since, b.name")
        .unwrap();
    assert_eq!(result.rows.len(), 1);

    let err = source.backup_to(&path).unwrap_err();
    assert!(matches!(err, marsdb::Error::Graph(_)));
}

#[test]
fn backup_preserves_declared_indexes_and_unique_constraints() {
    let source = Database::in_memory().unwrap();
    source
        .execute("CREATE INDEX ON :Person(email) UNIQUE")
        .unwrap();
    source
        .execute("CREATE (:Person {email: 'alice@x.com'})")
        .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup.redb");
    source.backup_to(&path).unwrap();

    let backup = Database::open(&path).unwrap();

    // The index itself must still be seekable, not silently gone.
    let plan = backup
        .execute("EXPLAIN MATCH (p:Person {email: 'alice@x.com'}) RETURN p")
        .unwrap();
    let plan_text = plan
        .rows
        .iter()
        .map(|row| format!("{row:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        plan_text.contains("IndexSeek"),
        "expected an IndexSeek in the restored database's plan, got: {plan_text}"
    );

    // And the unique constraint must still be enforced, not just the plan shape.
    let err = backup
        .execute("CREATE (:Person {email: 'alice@x.com'})")
        .unwrap_err();
    assert!(err.to_string().contains("unique"), "got: {err}");
}

#[test]
fn integrity_check_reports_graph_counts() {
    let mut db = Database::in_memory().unwrap();
    db.execute("CREATE (:Person)-[:KNOWS]->(:Person)").unwrap();

    let report = db.check_integrity().unwrap();
    assert!(report.physical_was_clean);
    assert_eq!(report.labels, 2);
    assert_eq!(report.nodes, 2);
    assert_eq!(report.edges, 1);

    // The database remains usable after the offline integrity pass.
    assert_eq!(db.execute("MATCH (n) RETURN n").unwrap().rows.len(), 2);
}

#[test]
fn execution_observer_reports_success_and_failure_without_query_text() {
    use std::sync::{Arc, Mutex};

    use marsdb::{ExecutionObserver, ExecutionOptions, ExecutionOutcome};

    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:Item)-[:NEXT]->(:Item)").unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&events);
    let options = ExecutionOptions {
        observer: Some(ExecutionObserver::new(move |event| {
            captured.lock().unwrap().push(event.clone());
        })),
        ..Default::default()
    };

    db.execute_with_options("MATCH (a)-[:NEXT]->(b) RETURN b", &options)
        .unwrap();
    assert!(db.execute_with_options("RETURN missing", &options).is_err());
    assert!(db
        .execute_with_options("NOT VALID CYPHER", &options)
        .is_err());
    assert!(db
        .execute_with_options("RETURN $missing", &options)
        .is_err());

    let events = events.lock().unwrap();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0].outcome, ExecutionOutcome::Success);
    assert_eq!(events[0].result_rows, Some(1));
    assert_eq!(events[0].statement_read_only, Some(true));
    assert!(events[0].relationship_expansions >= 1);
    assert_eq!(events[1].outcome, ExecutionOutcome::SemanticError);
    assert_eq!(events[1].result_rows, None);
    assert_eq!(events[2].outcome, ExecutionOutcome::SyntaxError);
    assert_eq!(events[2].statement_read_only, None);
    assert_eq!(events[3].outcome, ExecutionOutcome::MissingParameter);
    assert_eq!(events[3].statement_read_only, Some(true));
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
        db.execute(&format!("CREATE (n:Item {{idx: {i}}})"))
            .unwrap();
    }

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                for _ in 0..50 {
                    let result = db.execute("MATCH (n:Item) RETURN n.idx").unwrap();
                    assert_eq!(
                        result.rows.len(),
                        200,
                        "every concurrent reader must see all 200 rows"
                    );
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
                db.execute(&format!("CREATE (n:Item {{idx: {i}}})"))
                    .unwrap();
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
