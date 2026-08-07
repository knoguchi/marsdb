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
use std::time::Instant;

pub use marsdb_graph::IntegrityReport;
pub use marsdb_graph::PropertyValue;
pub use marsdb_graph::TzId;
pub use marsdb_query::{
    temporal, CancellationToken, ExecutionEvent, ExecutionObserver, ExecutionOptions,
    ExecutionOutcome, Literal, PathElem, ProcedureProvider, ProcedureSignature, Procedures,
    QueryResult, Value,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("graph error: {0}")]
    Graph(#[from] marsdb_graph::GraphError),
    #[error("query error: {0}")]
    Query(#[from] marsdb_query::QueryError),
    #[error("transaction is no longer active")]
    TransactionClosed,
}

pub struct Database {
    store: marsdb_graph::GraphStore,
}

impl Database {
    /// Open (creating if absent) a single-file, on-disk database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            store: marsdb_graph::GraphStore::open_file(path)?,
        })
    }

    /// Open a purely in-memory database. Nothing is written to disk.
    pub fn in_memory() -> Result<Self, Error> {
        Ok(Self {
            store: marsdb_graph::GraphStore::open_memory()?,
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
        let result =
            marsdb_query::Executor::new(&self.store).execute_with_options(&stmt, &options)?;
        Ok(result)
    }

    /// Runs a `;`-separated batch of statements (e.g.
    /// `"CREATE (a); CREATE (b); MATCH (n) RETURN n"`), returning one
    /// `QueryResult` per statement in order.
    ///
    /// The whole batch is parsed up front — a syntax error anywhere in it
    /// means nothing runs at all. Execution, though, is one transaction
    /// per statement (same crash-safety model as a single `execute()`
    /// call): if a statement fails at *run* time (e.g. an unbound
    /// variable), every statement before it in the batch is already
    /// committed and stays that way — this returns `Err` immediately
    /// rather than continuing, but doesn't roll anything back.
    pub fn execute_batch(&self, cypher: &str) -> Result<Vec<QueryResult>, Error> {
        let stmts = marsdb_query::parse_many(cypher)?;
        let executor = marsdb_query::Executor::new(&self.store);
        stmts
            .iter()
            .map(|stmt| Ok(executor.execute(stmt)?))
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
