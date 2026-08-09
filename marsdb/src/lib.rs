//! Embeddable property-graph database with an openCypher query subset:
//! single binary, single file, optional in-memory mode.
//!
//! ```
//! let db = marsdb::Database::in_memory().unwrap();
//! db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})").unwrap();
//! let result = db.execute("MATCH (n:Person) RETURN n.name").unwrap();
//! assert_eq!(result.rows.len(), 2);
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use marsdb_query::Statement;

pub use marsdb_graph::GraphError;
pub use marsdb_graph::IntegrityReport;
pub use marsdb_graph::PropertyValue;
pub use marsdb_graph::TzId;
pub use marsdb_query::{
    temporal, CancellationToken, ExecutionEvent, ExecutionObserver, ExecutionOptions,
    ExecutionOutcome, Literal, PathElem, ProcedureProvider, ProcedureSignature, Procedures,
    QueryError, QueryResult, QueryStats, RowSink, Value,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("graph error: {0}")]
    Graph(#[from] marsdb_graph::GraphError),
    #[error("query error: {0}")]
    Query(#[from] marsdb_query::QueryError),
    #[error("transaction is no longer active")]
    TransactionClosed,
    #[error(
        "session transaction was idle for {idle:?} (limit {limit:?}) and has been rolled back"
    )]
    SessionTransactionTimedOut {
        idle: std::time::Duration,
        limit: std::time::Duration,
    },
}

pub struct Database {
    store: marsdb_graph::GraphStore,
    /// The Cypher-level session transaction (`BEGIN`/`COMMIT`/`ROLLBACK`
    /// statements, issue #142 — MarsDB extension, openCypher has no
    /// transaction statements). `BEGIN` opens it, every subsequent
    /// `execute`/`execute_batch` statement runs inside it (reads included,
    /// so they see the transaction's own writes), `COMMIT`/`ROLLBACK`
    /// close it. One per `Database` handle — the handle *is* the session;
    /// callers that need independent concurrent units of work should use
    /// [`Database::begin_transaction`]'s caller-owned handles instead.
    ///
    /// Same abort-on-error stance as [`Transaction`] for *execution*
    /// errors (a failed statement's partial effects must not be
    /// committable), but a statement that never ran at all — parse or
    /// `$param`-substitution failure — leaves the transaction open:
    /// nothing was applied, and killing an interactive session's
    /// transaction over a typo helps nobody.
    ///
    /// An open session transaction holds redb's single writer, so an
    /// abandoned one blocks every other writer in the process — caller-
    /// owned [`Database::begin_transaction`] handles and
    /// [`Database::execute_batch_grouped`] groups included — *forever*
    /// (redb's `begin_write` blocks, it doesn't error). The optional
    /// idle timeout ([`Database::set_session_transaction_timeout`]) is
    /// the mitigation: expiry is checked lazily, by whatever statement
    /// next comes through the session layer.
    session_txn: Mutex<Option<OpenSessionTxn>>,
    /// See [`Database::set_session_transaction_timeout`]. Behind its own
    /// lock (not folded into `session_txn`'s) so reconfiguring never
    /// waits on a statement executing inside an open transaction.
    session_txn_timeout: Mutex<Option<std::time::Duration>>,
}

/// An open Cypher-level session transaction — see `Database::session_txn`.
struct OpenSessionTxn {
    txn: marsdb_graph::WriteTransaction,
    /// Refreshed after every statement that runs inside the transaction
    /// (including the `BEGIN` that opened it); what the idle timeout
    /// measures against.
    last_used: Instant,
}

impl Database {
    /// Open (creating if absent) a single-file, on-disk database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            store: marsdb_graph::GraphStore::open_file(path)?,
            session_txn: Mutex::new(None),
            session_txn_timeout: Mutex::new(None),
        })
    }

    /// Open a purely in-memory database. Nothing is written to disk.
    pub fn in_memory() -> Result<Self, Error> {
        Ok(Self {
            store: marsdb_graph::GraphStore::open_memory()?,
            session_txn: Mutex::new(None),
            session_txn_timeout: Mutex::new(None),
        })
    }

    /// Begin an explicit multi-statement write transaction. Reads executed
    /// through the returned handle see earlier writes in the same transaction.
    pub fn begin_transaction(&self) -> Result<Transaction<'_>, Error> {
        Ok(Transaction {
            db: self,
            inner: Some(self.store.begin_write()?),
        })
    }

    /// Create a transactionally consistent database backup. The destination
    /// must not already exist and is never overwritten.
    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.store.backup_to(path)?;
        Ok(())
    }

    /// Check redb's physical storage and MarsDB's logical graph invariants.
    /// The database must not have any active transactions while this runs.
    pub fn check_integrity(&mut self) -> Result<IntegrityReport, Error> {
        Ok(self.store.check_integrity()?)
    }

    pub fn execute(&self, cypher: &str) -> Result<QueryResult, Error> {
        self.execute_with_params(cypher, &HashMap::new())
    }

    /// Execute one statement with cooperative timeout, cancellation, and
    /// row/expansion limits. Limits are checked during plan evaluation, not
    /// only after the full result has already been materialized.
    pub fn execute_with_options(
        &self,
        cypher: &str,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, Error> {
        self.execute_with_params_and_options(cypher, &HashMap::new(), options)
    }

    /// Run a Cypher statement with `$name` placeholders resolved from
    /// `params` before execution.
    pub fn execute_with_params(
        &self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
    ) -> Result<QueryResult, Error> {
        self.execute_with_params_and_options(cypher, params, &ExecutionOptions::default())
    }

    pub fn execute_with_params_and_options(
        &self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, Error> {
        let stmt = prepare_statement(cypher, params, options)?;
        let options = with_call_params(options, params);
        self.execute_prepared(&stmt, &options)
    }

    /// One already-parsed statement, session-aware: `BEGIN`/`COMMIT`/
    /// `ROLLBACK` act on `session_txn` (see its docs for the whole
    /// model), anything else runs inside the open session transaction
    /// when there is one, else autocommits exactly as before the session
    /// layer existed. The session lock is held across a statement only
    /// when a transaction is actually open (statements inside one
    /// transaction are sequential by definition); the no-session path
    /// releases it before executing, so concurrent readers on one
    /// `Database` handle still run in parallel.
    fn execute_prepared(
        &self,
        stmt: &Statement,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, Error> {
        let empty = QueryResult::default;
        {
            let mut session = self
                .session_txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            // Idle expiry, checked lazily by whatever statement comes
            // through next -- BEGIN/COMMIT/ROLLBACK included. The
            // discovering statement gets the explicit timeout error (a
            // later COMMIT reporting a generic "no open transaction"
            // would read as "so my writes autocommitted?" -- exactly
            // wrong); the state is cleared, so the statement *after* the
            // error runs normally. Aborting here is also what releases
            // redb's single writer for anything blocked on it.
            if let Some(open) = session.as_ref() {
                let limit = *self
                    .session_txn_timeout
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(limit) = limit {
                    let idle = open.last_used.elapsed();
                    if idle > limit {
                        let open = session.take().expect("checked is_some");
                        let _ = marsdb_graph::GraphStore::abort(open.txn);
                        return Err(Error::SessionTransactionTimedOut { idle, limit });
                    }
                }
            }
            match stmt {
                Statement::Begin => {
                    return if session.is_some() {
                        Err(marsdb_query::QueryError::Semantic(
                            "BEGIN: this session already has an open transaction".into(),
                        )
                        .into())
                    } else {
                        *session = Some(OpenSessionTxn {
                            txn: self.store.begin_write()?,
                            last_used: Instant::now(),
                        });
                        Ok(empty())
                    }
                }
                Statement::Commit => {
                    return match session.take() {
                        Some(open) => {
                            marsdb_graph::GraphStore::commit(open.txn)?;
                            Ok(empty())
                        }
                        None => Err(marsdb_query::QueryError::Semantic(
                            "COMMIT: this session has no open transaction".into(),
                        )
                        .into()),
                    }
                }
                Statement::Rollback => {
                    return match session.take() {
                        Some(open) => {
                            marsdb_graph::GraphStore::abort(open.txn)?;
                            Ok(empty())
                        }
                        None => Err(marsdb_query::QueryError::Semantic(
                            "ROLLBACK: this session has no open transaction".into(),
                        )
                        .into()),
                    }
                }
                _ => {
                    if let Some(open) = session.as_mut() {
                        let result = marsdb_query::Executor::new(&self.store)
                            .execute_in_write_transaction_with_options(stmt, &open.txn, options);
                        // Same stance as `Transaction`: an execution error
                        // may have applied partial effects, which must
                        // never be committable -- abort the whole session
                        // transaction, keep the original error.
                        if result.is_err() {
                            if let Some(open) = session.take() {
                                let _ = marsdb_graph::GraphStore::abort(open.txn);
                            }
                        } else {
                            open.last_used = Instant::now();
                        }
                        return Ok(result?);
                    }
                }
            }
        }
        Ok(marsdb_query::Executor::new(&self.store).execute_with_options(stmt, options)?)
    }

    /// Idle limit for the Cypher-level session transaction
    /// (`BEGIN`/`COMMIT`/`ROLLBACK` -- see `session_txn`'s docs). `None`
    /// (the default, matching what real deployments ship for their
    /// equivalent knobs) disables it. When set, a session transaction
    /// idle longer than `limit` is rolled back by the next statement to
    /// arrive, which returns [`Error::SessionTransactionTimedOut`];
    /// statements after that run normally. There is no background timer
    /// -- an abandoned transaction with *no* further traffic on this
    /// handle keeps holding redb's single writer, so an embedder mixing
    /// session transactions with caller-owned
    /// [`Database::begin_transaction`] handles across threads should set
    /// this AND expect the reclaim to happen on the next session-layer
    /// statement, not on a clock.
    pub fn set_session_transaction_timeout(&self, limit: Option<std::time::Duration>) {
        *self
            .session_txn_timeout
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = limit;
    }

    /// Stream a read-only statement's rows to `sink` instead of
    /// materializing them — bounded memory no matter how many rows
    /// match; the bulk-export path. Accepts exactly the streamable
    /// shape (one plain `MATCH ... RETURN`, `SKIP`/`LIMIT` fine, no
    /// ORDER BY/aggregation/DISTINCT/WITH) and errors — never silently
    /// materializes — on anything else; see
    /// `Executor::execute_streaming_with_options` for the full contract.
    /// Not available while this session has an open `BEGIN` transaction
    /// (a stream holds a read snapshot for caller-controlled time; the
    /// session's write transaction is not that snapshot).
    pub fn execute_streaming(
        &self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
        options: &ExecutionOptions,
        sink: &mut dyn RowSink,
    ) -> Result<(), Error> {
        {
            let session = self
                .session_txn
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if session.is_some() {
                return Err(marsdb_query::QueryError::Semantic(
                    "execute_streaming is not available inside an open session transaction".into(),
                )
                .into());
            }
        }
        let stmt = prepare_statement(cypher, params, options)?;
        let options = with_call_params(options, params);
        marsdb_query::Executor::new(&self.store)
            .execute_streaming_with_options(&stmt, &options, sink)?;
        Ok(())
    }

    /// Runs a `;`-separated batch of statements (e.g.
    /// `"CREATE (a); CREATE (b); MATCH (n) RETURN n"`), returning one
    /// `QueryResult` per statement in order.
    ///
    /// The whole batch is parsed up front — a syntax error anywhere in it
    /// means nothing runs at all. Execution, though, is one transaction
    /// per statement (same crash-safety model as a single `execute()`
    /// call) — unless the batch itself opens one: `BEGIN`/`COMMIT`/
    /// `ROLLBACK` statements work in a batch exactly as they do fed one
    /// at a time (`session_txn`'s docs), so `"BEGIN; CREATE (a);
    /// CREATE (b); COMMIT"` is one atomic unit. If a statement fails at
    /// *run* time (e.g. an unbound variable), this returns `Err`
    /// immediately rather than continuing: outside a transaction every
    /// statement before it is already committed and stays that way;
    /// inside one, the whole open transaction is aborted.
    pub fn execute_batch(&self, cypher: &str) -> Result<Vec<QueryResult>, Error> {
        let stmts = marsdb_query::parse_many(cypher)?;
        let options = ExecutionOptions::default();
        stmts
            .iter()
            .map(|stmt| self.execute_prepared(stmt, &options))
            .collect()
    }

    /// Same as [`execute_batch`](Self::execute_batch), but commits once
    /// every `group_size` statements instead of once per statement — the
    /// group-commit pattern real databases use for bulk loads, trading
    /// crash-safety granularity for throughput. Each commit is an fsync;
    /// on a 9,771-statement real-world load script, `execute_batch` took
    /// 69.1s, `execute_batch_grouped` took 13.4s at `group_size: 100` and
    /// 12.1s at `group_size: 9771` (measured, not estimated) — most of the
    /// win is already there by a few hundred statements per group; there's
    /// little reason to go larger just to shrink the group count further.
    ///
    /// If a statement fails, the group it's in is rolled back in full —
    /// not partially applied — while every earlier group that already
    /// committed stays committed. On a crash, the same holds: only
    /// fully-committed groups survive. Use `execute_batch` instead when a
    /// failure or crash needs to preserve everything up to that exact
    /// statement; use this for a script you'd simply re-run from scratch
    /// on failure anyway.
    pub fn execute_batch_grouped(
        &self,
        cypher: &str,
        group_size: usize,
    ) -> Result<Vec<QueryResult>, Error> {
        let stmts = marsdb_query::parse_many(cypher)?;
        let executor = marsdb_query::Executor::new(&self.store);
        let mut results = Vec::with_capacity(stmts.len());
        for group in stmts.chunks(group_size.max(1)) {
            let write_txn = self.store.begin_write()?;
            let mut group_results = Vec::with_capacity(group.len());
            for stmt in group {
                match executor.execute_in_write_transaction(stmt, &write_txn) {
                    Ok(result) => group_results.push(result),
                    Err(e) => {
                        let _ = marsdb_graph::GraphStore::abort(write_txn);
                        return Err(e.into());
                    }
                }
            }
            marsdb_graph::GraphStore::commit(write_txn)?;
            results.extend(group_results);
        }
        Ok(results)
    }
}

/// Caller-managed atomic unit of work. Any statement error immediately
/// aborts the whole transaction, preventing a later accidental commit of
/// partial statement effects.
pub struct Transaction<'db> {
    db: &'db Database,
    inner: Option<marsdb_graph::WriteTransaction>,
}

impl Transaction<'_> {
    pub fn execute(&mut self, cypher: &str) -> Result<QueryResult, Error> {
        self.execute_with_params_and_options(cypher, &HashMap::new(), &ExecutionOptions::default())
    }

    pub fn execute_with_params(
        &mut self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
    ) -> Result<QueryResult, Error> {
        self.execute_with_params_and_options(cypher, params, &ExecutionOptions::default())
    }

    pub fn execute_with_options(
        &mut self,
        cypher: &str,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, Error> {
        self.execute_with_params_and_options(cypher, &HashMap::new(), options)
    }

    pub fn execute_with_params_and_options(
        &mut self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, Error> {
        let Some(write_txn) = self.inner.as_ref() else {
            return Err(Error::TransactionClosed);
        };
        let outcome = (|| {
            let stmt = prepare_statement(cypher, params, options)?;
            let options = with_call_params(options, params);
            Ok(marsdb_query::Executor::new(&self.db.store)
                .execute_in_write_transaction_with_options(&stmt, write_txn, &options)?)
        })();
        if outcome.is_err() {
            if let Some(write_txn) = self.inner.take() {
                marsdb_graph::GraphStore::abort(write_txn)?;
            }
        }
        outcome
    }

    pub fn commit(mut self) -> Result<(), Error> {
        let write_txn = self.inner.take().ok_or(Error::TransactionClosed)?;
        marsdb_graph::GraphStore::commit(write_txn)?;
        Ok(())
    }

    pub fn rollback(mut self) -> Result<(), Error> {
        let write_txn = self.inner.take().ok_or(Error::TransactionClosed)?;
        marsdb_graph::GraphStore::abort(write_txn)?;
        Ok(())
    }
}

/// See `ExecutionOptions::params`'s own docs -- a standalone `CALL proc`
/// with no parens needs the raw params map at execution time, after
/// `prepare_statement` has already consumed it for ordinary `$param`
/// substitution.
fn with_call_params(
    options: &ExecutionOptions,
    params: &HashMap<String, PropertyValue>,
) -> ExecutionOptions {
    let mut options = options.clone();
    options.params = params.clone();
    options
}

fn prepare_statement(
    cypher: &str,
    params: &HashMap<String, PropertyValue>,
    options: &ExecutionOptions,
) -> Result<marsdb_query::Statement, Error> {
    let started = Instant::now();
    let mut stmt = match marsdb_query::parse(cypher) {
        Ok(stmt) => stmt,
        Err(error) => {
            observe_rejected_statement(options, started, None, &error);
            return Err(error.into());
        }
    };
    if let Err(error) = marsdb_query::substitute_params(&mut stmt, params) {
        observe_rejected_statement(
            options,
            started,
            Some(marsdb_query::is_read_only(&stmt)),
            &error,
        );
        return Err(error.into());
    }
    Ok(stmt)
}

fn observe_rejected_statement(
    options: &ExecutionOptions,
    started: Instant,
    statement_read_only: Option<bool>,
    error: &marsdb_query::QueryError,
) {
    if let Some(observer) = &options.observer {
        observer.observe(&ExecutionEvent {
            elapsed: started.elapsed(),
            statement_read_only,
            result_rows: None,
            relationship_expansions: 0,
            outcome: ExecutionOutcome::from_error(error),
        });
    }
}
