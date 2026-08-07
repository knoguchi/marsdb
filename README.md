# MarsDB

[![CI](https://github.com/knoguchi/marsdb/actions/workflows/rust.yml/badge.svg)](https://github.com/knoguchi/marsdb/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/marsdb.svg)](https://crates.io/crates/marsdb)
[![docs.rs](https://img.shields.io/docsrs/marsdb)](https://docs.rs/marsdb)
[![codecov](https://codecov.io/gh/knoguchi/marsdb/graph/badge.svg)](https://codecov.io/gh/knoguchi/marsdb)
[![License](https://img.shields.io/crates/l/marsdb.svg)](#license)
[![openCypher TCK](https://img.shields.io/badge/openCypher_TCK-99.9%25-brightgreen)](CYPHER_COVERAGE.md)

An embeddable property-graph database with an openCypher query subset:
single binary, single file, optional in-memory mode. Full manual:
[knoguchi.github.io/marsdb](https://knoguchi.github.io/marsdb/).

```
$ marsdb :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
marsdb> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
marsdb> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```

## Install

**CLI** — installs the `marsdb` binary:

```
cargo install marsdb-cli
```

Or on macOS/Linux via Homebrew ([tap](https://github.com/knoguchi/homebrew-marsdb)):

```
brew install knoguchi/marsdb/marsdb
```

**Rust library**:

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

// Operational checks and crash-consistent backup. The backup destination
// must be new, so an existing file is never overwritten.
db.backup_to("path/to-backup.db")?;
let mut db = db;
let report = db.check_integrity()?;
assert_eq!(report.nodes, 3);

// Or run a `;`-separated batch, one transaction per statement, one
// QueryResult per statement back:
let results = db.execute_batch("CREATE (a:Person {name: 'Alice'}); CREATE (b:Person {name: 'Bob'})")?;
```

`ExecutionOptions::observer` accepts an `ExecutionObserver` callback for
dependency-free telemetry. Events contain duration, outcome category,
read/write classification, result-row count, and relationship expansions;
they deliberately exclude query text and error messages. Syntax and missing-
parameter rejections are reported too, and observer panics are contained.

More: `cargo run -p marsdb --example task_tracker` (CRUD + aggregation),
`--example social_graph` (variable-length traversal, `MATCH...CREATE`), or
`--example params_and_batch` (`$parameters`, `execute_batch`) — full
source in [`marsdb/examples/`](./marsdb/examples). Each also writes an SVG
chart of its query result (via [plotters](https://github.com/plotters-rs/plotters))
to the current directory.

**Python**:

```
pip install marsdb
```

```python
import marsdb
db = marsdb.Database.in_memory()  # or .open(path)
db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
db.execute("MATCH (n:Person) RETURN n.name")
# -> [{'n.name': 'Alice'}, {'n.name': 'Bob'}]
```

Prebuilt wheels cover macOS (arm64, x86_64) and Linux (x86_64, manylinux);
other platforms install from the source distribution and need a Rust
toolchain. To build from source directly:

```
cd marsdb-python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin && maturin develop
```

**Go**: not published yet — no `go get`-able module path. Bindings live in
[`marsdb-go`](./marsdb-go) (cgo, via a C ABI crate,
[`marsdb-capi`](./marsdb-capi)); see that README for the two-step build
(Rust cdylib, then `go build`) and a full example.

```go
db, _ := marsdb.InMemory() // or marsdb.Open(path)
db.Execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
rows, _ := db.Execute("MATCH (n:Person) RETURN n.name AS name")
// rows -> []map[string]any{{"name": "Alice"}, {"name": "Bob"}}
```

## CLI usage

```
marsdb                                  # in-memory REPL
marsdb mydata.db                        # file-backed REPL
marsdb mydata.db "MATCH (n) RETURN n"   # run one query, exit
marsdb :memory: "..."                   # explicit in-memory, one-shot
marsdb mydata.db "CREATE (a); CREATE (b); MATCH (n) RETURN n"  # ;-separated batch
```

## Natural language -> Cypher

`marsdb-nl2cypher` translates an English question into Cypher against a
database's actual schema (labels/relationship-types/properties in use,
introspected automatically), validates its syntax and variable/type binding,
and retries once with the exact validation error fed back if the first attempt
is invalid. No HTTP/LLM-SDK dependency in the core crate — bring your
own `LlmClient`:

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

A real, runnable example against a local [Ollama](https://ollama.com) instance:

```
ollama serve &
ollama pull llama3.2
cargo run -p marsdb-nl2cypher --example ollama_demo
```

MarsDB's narrower Cypher subset (vs. full Neo4j Cypher) is a deliberate fit
for this — a smaller grammar means fewer ways an LLM can generate something
unparseable. The prompt tells the model what's supported *and* what to
avoid (no bare `-->` shorthand, `MERGE` capped at one hop, etc.) — see
`marsdb-nl2cypher/src/lib.rs`'s `CAPABILITIES` constant.

## Architecture

```
marsdb-storage   thin trait boundary over redb (file + in-memory backends)
marsdb-graph     property graph model, CRUD, KV/adjacency encoding
marsdb-query     openCypher subset: ANTLR4 grammar -> AST -> IR -> executor
marsdb           embeddable public Rust API (Database::open/in_memory/execute)
marsdb-cli       the `marsdb` binary (REPL + one-shot mode)
marsdb-python    PyO3 bindings, builds via maturin
marsdb-capi      C ABI (opaque handle + JSON results), basis for non-Rust bindings
marsdb-go        Go bindings, via cgo against marsdb-capi
marsdb-nl2cypher natural-language -> Cypher: schema introspection, prompt building, validate-and-repair
```

Storage runs on [redb](https://github.com/cberner/redb), a pure-Rust
single-file MVCC embedded KV engine. Query execution compiles Cypher to a
small Gremlin-shaped logical IR (`AllNodesScan`, `NodeByLabelScan`,
`Seed`, `Expand`, `VarExpand`, `Filter`) so a future Gremlin frontend can
target the same executor. Every Cypher statement runs inside one
transaction — a read-only `MATCH ... RETURN` opens a `ReadTransaction`
(a consistent snapshot that runs alongside other concurrent readers or a
concurrent writer without contending for redb's single-writer lock),
everything else opens a `WriteTransaction`, committed or aborted as a
whole. `Database::begin_transaction` lets callers explicitly extend that
atomic boundary across multiple statements. MarsDB records its own
table/record format version in metadata when
the file is created or first opened by a version-aware build, and refuses
to open a database written by a newer unsupported format.

Numbers: [`BENCHMARKS.md`](./BENCHMARKS.md).

The logical read plan runs as a pull-based row stream through node-ID scans,
filters, and relationship expansions. A non-aggregating, non-distinct
`RETURN ... LIMIT k` without `ORDER BY` stops that pipeline after `k` rows,
so downstream limits avoid unnecessary expansions. Clause boundaries and
inherently blocking operations still materialize: `WITH`, optional-match
reconciliation, variable-length traversal results for each input row,
aggregation, `DISTINCT`, mutations, and the public `QueryResult`. Use
`ExecutionOptions` to put hard ceilings on intermediate rows, result rows,
relationship expansions, and elapsed time.

There is not yet a general cost-based optimizer. Two targeted optimizations
complement streaming: a direct `MATCH (n[:Label]) RETURN ... LIMIT k` scan
pushes the limit into storage; and every `ORDER BY ... LIMIT k` site
(`WITH`'s own, non-aggregating
`RETURN`'s, aggregating `RETURN`'s) uses a top-k partial selection
(`slice::select_nth_unstable_by` + a sort of just the k-sized prefix)
instead of a full sort of every row.

### Cypher coverage

Full breakdown — every supported clause/expression/temporal-type shape,
the error taxonomy, and a real, measured openCypher TCK conformance table
by category — lives in **[CYPHER_COVERAGE.md](CYPHER_COVERAGE.md)**.

Short version: 3878/3880 TCK scenarios pass (99.9%), 0 wrong-result
scenarios. The only 2 non-passing scenarios need dates at year
±999,999,999 — a real storage/library range limitation, not a bug (see
[CYPHER_COVERAGE.md](CYPHER_COVERAGE.md) for the exact explanation).
Recent changes: [CHANGELOG.md](CHANGELOG.md).

## Roadmap

- From-scratch storage engine (page format, B-tree, crash recovery) as an
  alternate `marsdb-storage` backend, independent of redb
- Gremlin frontend targeting the existing IR
- Composite indexes (single-property indexes already exist) with
  transactional maintenance
- Real query optimizer: cardinality estimates and cost-based
  join/traversal ordering (beyond the existing index-seek fusion and
  top-k `ORDER BY ... LIMIT` selection)
- Node/relationship-valued `$parameters` (map-valued already works)

## Testing

```
cargo test --workspace                                             # ~8s
cargo test -p marsdb-graph --test stress -- --ignored --nocapture  # ~15s, large-scale
cargo test -p marsdb-crash-harness -- --ignored --nocapture        # ~7s/30 runs, SIGKILL-and-verify
cargo bench -p marsdb-graph
cargo bench -p marsdb
```

`marsdb-crash-harness` is a process-crash durability check, not a full power-loss
test (OS stays up, page cache intact — a real power-loss test needs fault
injection like `dm-flakey`, not just `SIGKILL`): it spawns a child process
committing one transaction at a time, `SIGKILL`s it at an unpredictable point,
reopens the file fresh, and asserts every acknowledged commit survived intact
with no gaps or duplicates. `CRASH_TEST_RUNS=200` (default 30) for more
confidence at the cost of runtime.

The Cypher parser (the one part of MarsDB that takes raw, untrusted string
input directly) is fuzzed via `cargo-fuzz` — needs nightly:

```
cargo install cargo-fuzz
cd marsdb-query && cargo +nightly fuzz run parse -- -max_total_time=120
```

Only claim: never panics. A parse error (`Result::Err`) is the expected,
correct outcome for most fuzzer-generated input. Keep at least one real
Cypher string in `fuzz/corpus/parse/` — running against a genuinely empty
corpus made libFuzzer's own startup/seed-bootstrapping phase (not anything
in MarsDB) take upwards of an hour instead of seconds on this target; a
seeded corpus doesn't hit it.

`marsdb-tck` runs a real subset of the
[openCypher TCK](https://github.com/opencypher/openCypher) (pulled in as a
git submodule pinned to a fixed commit — see `marsdb-tck/VENDOR.md`; 220
`.feature` files, 3880 scenarios) against MarsDB, via a purpose-built
Gherkin-subset parser and structural result comparison (not string
matching):

```
git submodule update --init marsdb-tck/openCypher
cargo run --release -p marsdb-tck
```

Attempts every scenario in the vendored TCK — currently 3878/3880 pass,
and the 2 that don't (dates far outside the representable range, see
[CYPHER_COVERAGE.md](CYPHER_COVERAGE.md)) report as a distinct
"unexpected" outcome, not lumped in with genuine wrong answers. 0
scenarios currently return the wrong answer — the real, checked-for-real
signal this exists to produce, not the flat pass-rate over the whole
suite. Side-effect assertions and the TCK's typed error taxonomy aren't
checked (see the crate's doc comments for why).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the local checks to run
before opening a PR, and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for
community expectations. Found a security issue? See
[SECURITY.md](SECURITY.md) — please don't open a public issue for it.

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or
[MIT license](./LICENSE-MIT) at your option.
