# Embedding in Rust

```
cargo add marsdb
```

```rust
let db = marsdb::Database::in_memory()?; // or Database::open("path/to.db")
db.execute("CREATE (a:Person {name: 'Alice'})")?;
let result = db.execute("MATCH (n:Person) RETURN n.name")?;

// Bound work from untrusted callers. A CancellationToken can also be
// cloned and cancelled from another thread.
let options = marsdb::ExecutionOptions {
    max_intermediate_rows: Some(100_000),
    max_result_rows: Some(10_000),
    max_relationship_expansions: Some(1_000_000),
    timeout: Some(std::time::Duration::from_secs(5)),
    ..Default::default()
};
let result = db.execute_with_options("MATCH (n) RETURN n", &options)?;

// Group statements into one atomic unit. Reads through `tx` see its earlier
// writes; any statement error aborts and closes the whole transaction.
let mut tx = db.begin_transaction()?;
tx.execute("CREATE (:Person {name: 'Bob'})")?;
tx.execute("CREATE (:Person {name: 'Carol'})")?;
tx.commit()?;

// Or run a `;`-separated batch, one transaction per statement, one
// QueryResult per statement back:
let results = db.execute_batch("CREATE (a:Person {name: 'Alice'}); CREATE (b:Person {name: 'Bob'})")?;
```

`ExecutionOptions::observer` accepts an `ExecutionObserver` callback for
dependency-free telemetry. Events contain duration, outcome category,
read/write classification, result-row count, and relationship expansions;
they deliberately exclude query text and error messages. Syntax and
missing-parameter rejections are reported too, and observer panics are
contained.

Backup, integrity checks, and other operational concerns that apply
regardless of which language you're calling from are covered in
[Operations](./operations.md).

## Stored procedures (`CALL`)

MarsDB ships no built-in procedures — `CALL proc(args) [YIELD ...]`
resolves against a `marsdb_query::ProcedureProvider` you supply via
`ExecutionOptions::procedures`:

```rust
use std::sync::Arc;
use marsdb::{ExecutionOptions, Procedures, ProcedureProvider, ProcedureSignature, Value};
// `ProcedureProvider::call`'s error type -- not re-exported by `marsdb`
// itself, so implementing the trait needs a direct `marsdb-query`
// dependency too (`cargo add marsdb-query`).
use marsdb_query::QueryError;

struct MyProcedures;

impl ProcedureProvider for MyProcedures {
    fn signature(&self, name: &str) -> Option<ProcedureSignature> {
        // Look up `name`'s declared inputs/outputs, or `None` if unknown.
        None
    }
    fn call(&self, name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, QueryError> {
        // Run the procedure, return its output rows.
        Ok(vec![])
    }
}

let options = ExecutionOptions {
    procedures: Some(Procedures(Arc::new(MyProcedures))),
    ..Default::default()
};
```

## More examples

```
cargo run -p marsdb --example task_tracker    # CRUD + aggregation
cargo run -p marsdb --example social_graph    # variable-length traversal, MATCH...CREATE
cargo run -p marsdb --example params_and_batch # $parameters, execute_batch
```

Full source in [`marsdb/examples/`](https://github.com/knoguchi/marsdb/tree/main/marsdb/examples).
Each also writes an SVG chart of its query result (via
[plotters](https://github.com/plotters-rs/plotters)) to the current
directory.

## Natural language → Cypher

`marsdb-nl2cypher` translates an English question into Cypher against a
database's actual schema (labels/relationship-types/properties in use,
introspected automatically), validates its syntax and variable/type
binding, and retries once with the exact validation error fed back if
the first attempt is invalid. No HTTP/LLM-SDK dependency in the core
crate — bring your own `LlmClient`:

```rust
use marsdb::Database;
use marsdb_nl2cypher::{translate_and_run, LlmClient};

let db = Database::in_memory()?;
db.execute("CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})")?;

let (cypher, result) = translate_and_run(&db, &my_llm_client, "who does Alice know?")?;
```

`translate_and_run` enforces read-only generated Cypher. Model-generated
writes are rejected before execution unless the caller explicitly uses
`translate_and_run_with_policy(..., ExecutionPolicy::AllowWrites)` after
performing its own authentication and authorization.

MarsDB's narrower Cypher subset (vs. full Neo4j Cypher) is a deliberate
fit for this — a smaller grammar means fewer ways an LLM can generate
something unparseable. The prompt tells the model what's supported *and*
what to avoid (no bare `-->` shorthand, `MERGE` capped at one hop, etc.)
— see `marsdb-nl2cypher/src/lib.rs`'s `CAPABILITIES` constant.

A runnable example against a local [Ollama](https://ollama.com)
instance:

```
ollama serve &
ollama pull llama3.2
cargo run -p marsdb-nl2cypher --example ollama_demo
```
