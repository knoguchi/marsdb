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

pub use marsdb_graph::PropertyValue;
pub use marsdb_query::{temporal, Literal, PathElem, QueryResult, Value};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("graph error: {0}")]
    Graph(#[from] marsdb_graph::GraphError),
    #[error("query error: {0}")]
    Query(#[from] marsdb_query::QueryError),
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

    pub fn execute(&self, cypher: &str) -> Result<QueryResult, Error> {
        self.execute_with_params(cypher, &HashMap::new())
    }

    /// Run a Cypher statement with `$name` placeholders resolved from
    /// `params` before execution.
    pub fn execute_with_params(
        &self,
        cypher: &str,
        params: &HashMap<String, PropertyValue>,
    ) -> Result<QueryResult, Error> {
        let mut stmt = marsdb_query::parse(cypher)?;
        marsdb_query::substitute_params(&mut stmt, params)?;
        let result = marsdb_query::Executor::new(&self.store).execute(&stmt)?;
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
}
