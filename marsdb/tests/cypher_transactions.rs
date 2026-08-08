//! Cypher-level BEGIN/COMMIT/ROLLBACK (issue #142) — the session layer
//! `Database::execute`/`execute_batch` put in front of the executor. The
//! caller-owned `Database::begin_transaction` API has its own coverage;
//! these tests are about the *statement* surface: session state
//! transitions, error semantics (execution errors abort, parse errors
//! don't), and batch interaction.

use marsdb::Database;

fn count(db: &Database) -> usize {
    db.execute("MATCH (n) RETURN n").unwrap().rows.len()
}

#[test]
fn commit_makes_transaction_writes_durable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("txn.db");
    {
        let db = Database::open(&path).unwrap();
        db.execute("BEGIN").unwrap();
        db.execute("CREATE (:Person {name: 'Alice'})").unwrap();
        db.execute("CREATE (:Person {name: 'Bob'})").unwrap();
        db.execute("COMMIT").unwrap();
    }
    let db = Database::open(&path).unwrap();
    assert_eq!(count(&db), 2);
}

#[test]
fn rollback_discards_everything_since_begin() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:Person {name: 'Kept'})").unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:Person {name: 'Discarded'})").unwrap();
    db.execute("ROLLBACK").unwrap();
    let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn reads_inside_a_transaction_see_its_own_uncommitted_writes() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:Person {name: 'Alice'})").unwrap();
    let inside = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
    assert_eq!(inside.rows.len(), 1);
    db.execute("ROLLBACK").unwrap();
    assert_eq!(count(&db), 0);
}

#[test]
fn keywords_are_case_insensitive_and_tolerate_a_trailing_semicolon() {
    let db = Database::in_memory().unwrap();
    db.execute("  begin ; ").unwrap();
    db.execute("CREATE (:N)").unwrap();
    db.execute("Commit;").unwrap();
    assert_eq!(count(&db), 1);
    db.execute("BeGiN").unwrap();
    db.execute("CREATE (:N)").unwrap();
    db.execute("rollback").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn commit_and_rollback_without_an_open_transaction_error() {
    let db = Database::in_memory().unwrap();
    assert!(db.execute("COMMIT").is_err());
    assert!(db.execute("ROLLBACK").is_err());
}

#[test]
fn nested_begin_errors_and_leaves_the_transaction_usable() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    assert!(db.execute("BEGIN").is_err());
    db.execute("CREATE (:N)").unwrap();
    db.execute("COMMIT").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn an_execution_error_aborts_the_whole_transaction() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:Person {name: 'Alice'})").unwrap();
    // Runtime failure: `m` is unbound. The session transaction must be
    // aborted -- its earlier CREATE can never be committed.
    assert!(db.execute("MATCH (n:Person) DELETE m").is_err());
    assert!(db.execute("COMMIT").is_err(), "transaction should be gone");
    assert_eq!(count(&db), 0);
}

#[test]
fn a_parse_error_does_not_abort_the_transaction() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:Person {name: 'Alice'})").unwrap();
    // Never parsed, never ran, nothing partial to protect against -- an
    // interactive session's typo shouldn't nuke its transaction.
    assert!(db.execute("CREATE (:Person {").is_err());
    db.execute("COMMIT").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn autocommit_still_works_after_the_transaction_closes() {
    let db = Database::in_memory().unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:N)").unwrap();
    db.execute("COMMIT").unwrap();
    db.execute("CREATE (:N)").unwrap();
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:N)").unwrap();
    db.execute("ROLLBACK").unwrap();
    db.execute("CREATE (:N)").unwrap();
    assert_eq!(count(&db), 3);
}

#[test]
fn batch_with_begin_commit_is_one_atomic_unit() {
    let db = Database::in_memory().unwrap();
    let results = db
        .execute_batch("BEGIN; CREATE (:N); CREATE (:N); COMMIT")
        .unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(count(&db), 2);
}

#[test]
fn batch_with_rollback_discards() {
    let db = Database::in_memory().unwrap();
    db.execute_batch("CREATE (:Kept); BEGIN; CREATE (:Discarded); ROLLBACK")
        .unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn batch_failure_inside_a_transaction_aborts_it() {
    let db = Database::in_memory().unwrap();
    assert!(db
        .execute_batch("BEGIN; CREATE (:N); MATCH (n) DELETE m; COMMIT")
        .is_err());
    assert_eq!(count(&db), 0);
    // Session must be clean again -- no dangling open transaction.
    assert!(db.execute("COMMIT").is_err());
    db.execute("CREATE (:N)").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn explicit_transaction_handles_reject_session_statements() {
    let db = Database::in_memory().unwrap();
    let mut txn = db.begin_transaction().unwrap();
    // A caller-owned `Transaction` has its own commit()/rollback()
    // methods; the session statements have no session to act on there.
    assert!(txn.execute("COMMIT").is_err());
}

#[test]
fn explain_of_a_session_statement_errors() {
    let db = Database::in_memory().unwrap();
    assert!(db.execute("EXPLAIN BEGIN").is_err());
}

#[test]
fn idle_timeout_rolls_back_and_reports_on_the_next_statement() {
    let db = Database::in_memory().unwrap();
    db.set_session_transaction_timeout(Some(std::time::Duration::from_millis(50)));
    db.execute("BEGIN").unwrap();
    db.execute("CREATE (:N)").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(80));
    let err = db.execute("COMMIT").unwrap_err();
    assert!(
        matches!(err, marsdb::Error::SessionTransactionTimedOut { .. }),
        "expected the explicit timeout error, got: {err}"
    );
    // Transaction gone, its writes discarded, session usable again.
    assert_eq!(count(&db), 0);
    db.execute("CREATE (:N)").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn activity_refreshes_the_idle_clock() {
    let db = Database::in_memory().unwrap();
    db.set_session_transaction_timeout(Some(std::time::Duration::from_millis(200)));
    db.execute("BEGIN").unwrap();
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        db.execute("CREATE (:N)").unwrap();
    }
    db.execute("COMMIT").unwrap();
    assert_eq!(count(&db), 3);
}

#[test]
fn timeout_discovered_by_a_plain_statement_not_just_commit() {
    let db = Database::in_memory().unwrap();
    db.set_session_transaction_timeout(Some(std::time::Duration::from_millis(50)));
    db.execute("BEGIN").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(80));
    assert!(matches!(
        db.execute("CREATE (:N)").unwrap_err(),
        marsdb::Error::SessionTransactionTimedOut { .. }
    ));
    // The discovering statement paid the error; the next one autocommits.
    db.execute("CREATE (:N)").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn clearing_the_timeout_disables_expiry() {
    let db = Database::in_memory().unwrap();
    db.set_session_transaction_timeout(Some(std::time::Duration::from_millis(20)));
    db.set_session_transaction_timeout(None);
    db.execute("BEGIN").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(60));
    db.execute("CREATE (:N)").unwrap();
    db.execute("COMMIT").unwrap();
    assert_eq!(count(&db), 1);
}

#[test]
fn expiring_the_session_transaction_unblocks_a_waiting_writer() {
    use std::sync::Arc;
    // A caller-owned begin_transaction() blocks on redb's single writer
    // while the session transaction is open; expiry (triggered here by
    // the session's own next statement) must release it.
    let db = Arc::new(Database::in_memory().unwrap());
    db.set_session_transaction_timeout(Some(std::time::Duration::from_millis(50)));
    db.execute("BEGIN").unwrap();

    let db2 = Arc::clone(&db);
    let waiter = std::thread::spawn(move || {
        let mut txn = db2.begin_transaction().unwrap(); // blocks until expiry
        txn.execute("CREATE (:FromWaiter)").unwrap();
        txn.commit().unwrap();
    });

    std::thread::sleep(std::time::Duration::from_millis(80));
    assert!(matches!(
        db.execute("MATCH (n) RETURN n").unwrap_err(),
        marsdb::Error::SessionTransactionTimedOut { .. }
    ));
    waiter.join().unwrap();
    assert_eq!(count(&db), 1);
}
