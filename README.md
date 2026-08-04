# MarsDB

An embeddable property-graph database with an openCypher query subset:
single binary, single file, optional in-memory mode.

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

// Or run a `;`-separated batch, one transaction per statement, one
// QueryResult per statement back:
let results = db.execute_batch("CREATE (a:Person {name: 'Alice'}); CREATE (b:Person {name: 'Bob'})")?;
```

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
introspected automatically), validates the result by really parsing it, and
retries once with the exact parse error fed back if the first attempt
doesn't parse. No HTTP/LLM-SDK dependency in the core crate — bring your
own `LlmClient`:

```rust
use marsdb::Database;
use marsdb_nl2cypher::{translate_and_run, LlmClient};

let db = Database::in_memory()?;
db.execute("CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})")?;

let (cypher, result) = translate_and_run(&db, &my_llm_client, "who does Alice know?")?;
```

A real, runnable example against a local [Ollama](https://ollama.com) instance:

```
ollama serve &
ollama pull llama3.2
cargo run -p marsdb-nl2cypher --example ollama_demo
```

MarsDB's narrower Cypher subset (vs. full Neo4j Cypher) is a deliberate fit
for this — a smaller grammar means fewer ways an LLM can generate something
unparseable. The prompt tells the model what's supported *and* what to
avoid (no bare `-->` shorthand, no `RETURN DISTINCT`, `MERGE` capped at one
hop, etc.) — see `marsdb-nl2cypher/src/lib.rs`'s `CAPABILITIES` constant.

## Architecture

```
marsdb-storage   thin trait boundary over redb (file + in-memory backends)
marsdb-graph     property graph model, CRUD, KV/adjacency encoding
marsdb-query     openCypher subset: pest grammar -> AST -> IR -> executor
marsdb           embeddable public Rust API (Database::open/in_memory/execute)
marsdb-cli       the `marsdb` binary (REPL + one-shot mode)
marsdb-python    PyO3 bindings, builds via maturin
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
whole.

Numbers: [`BENCHMARKS.md`](./BENCHMARKS.md).

Execution materializes a `Vec` of bindings at every step rather than
pulling rows lazily — there's no general query optimizer or streaming
iterator model. Two narrow, hand-written exceptions: a `MATCH (n[:Label])
RETURN ... LIMIT k` with no `WHERE`/hops/`ORDER BY` pushes the `LIMIT`
straight into the storage scan (`GraphStore::all_nodes_limited_in_txn`
stops once it has `k` nodes, whether or not a label narrows it first —
correct only because nothing downstream could still drop a row); and
every `ORDER BY ... LIMIT k` site (`WITH`'s own, non-aggregating
`RETURN`'s, aggregating `RETURN`'s) uses a top-k partial selection
(`slice::select_nth_unstable_by` + a sort of just the k-sized prefix)
instead of a full sort of every row. Both are real, targeted wins, not a
general "push predicates/limits through the plan" framework — see the
Roadmap.

### Cypher coverage

`CREATE`, multi-label nodes (`(n:Post:Message)`), `$parameters`,
backslash-escaped string literals (`\' \" \\ \n \r \t \b \f`),
`MATCH`/`OPTIONAL MATCH`, undirected (`-[r:TYPE]-`) and variable-length
(`[:TYPE*min..max]`) relationship patterns, `WHERE` (including the
string predicates `STARTS WITH`/`ENDS WITH`/`CONTAINS`), one `WITH`
boundary per statement (projection/rename, its own `WHERE`/
`WITH...WHERE`/`ORDER BY`/`LIMIT`), `RETURN`/`DELETE`/`DETACH DELETE`/
`SET`/`REMOVE`/`MATCH ... CREATE` (adds an edge between two
already-matched nodes — a node token whose variable is already bound
reuses that node instead of creating a new one). `SET`/`REMOVE` cover
both properties (`SET n.prop = 'x'`/`REMOVE n.prop`) and labels
(`SET n:Label`/`REMOVE n:Label`) — but, like `DELETE`, can't be
followed by anything else in the same statement (no `SET ... RETURN`
in one query yet — each is a terminal tail, not a chainable clause).
Multi-key `ORDER BY`, `LIMIT`, `CASE`, the built-in functions
`coalesce()`/`toInteger()`, and implicit-GROUP-BY aggregation
(`count()`/`count(*)`/`sum()`/`avg()`/`min()`/`max()`/`collect()`, with
`DISTINCT` — inside an aggregate call only; a standalone `RETURN DISTINCT`
result-set modifier doesn't exist yet). Two independent `MATCH` parts
across one `WITH` boundary (`MATCH (a) WITH a MATCH (b) ...`, where `b`'s
pattern doesn't chain from `a`) correctly cross-join, carrying `a`
alongside every row `b` produces. `UNWIND <list> AS x` (fans a list out
into one row per element, cross-joined against existing rows; its own
`WHERE` works without needing a second `WITH`) — `<list>` is an inline
Cypher-text list literal (`[1, 2, 'a', $p]`) or a variable bound by a
preceding `WITH ... collect(...)`; `UNWIND $param` where `$param` itself
names a list isn't supported yet (no list-valued parameters — every
`$param` is a single scalar). `MERGE <pattern> [ON CREATE SET ...] [ON
MATCH SET ...]` (match-or-create: tries the pattern as an ordinary MATCH
first, creates exactly one new instance if nothing matched) — capped at
one relationship hop (`MERGE (n:Label {props})` or `MERGE (a)-[:TYPE]->
(b)`); an unconstrained node pattern that isn't already bound (`MERGE
(n)`, no label or property) is rejected rather than matching/creating
arbitrarily. Named-path capture (`MATCH p = (a)-[:KNOWS]->(b) RETURN p`,
fixed-hop patterns only) and `shortestPath((a)-[:TYPE*..N]-(b))` (real
shortest-path search via BFS, not just the first path found — both
endpoints must already be matched by a preceding clause), plus
`length(p)` to measure one.

Verified against all 7 of LDBC SNB Interactive's short-read reference
queries (IS1-IS7) — see `marsdb-query/tests/ldbc_is_queries.rs`. Not
verified: LDBC's complex queries (IC1-14: the full query set beyond one
hand-crafted grouping+`WITH...WHERE`+`ORDER BY`+`LIMIT`+`collect()`
checkpoint — see `marsdb-query/tests/smoke.rs`), comma-separated patterns
*within one* `MATCH`/`CREATE` clause beyond a single linear chain (general
cross-joins — different from the cross-join WITH-chaining above, which
works), chaining past one `WITH` boundary, `MERGE` patterns with more than
one relationship hop (whole-pattern atomicity across multiple
simultaneously-unbound hops isn't attempted), named-path capture over a
variable-length pattern (only `shortestPath()` tracks the hop-by-hop chain
needed to reconstruct a path over `*`-traversal), or `shortestPath()` with
a minimum hop count greater than 1 (a plain visited-set BFS can't
correctly answer "shortest path of at least N hops" for N > 1 without a
different algorithm). One more gap the TCK surfaced directly: no
compile-time semantic validation (an undefined variable/function, or a
wrong-type function argument, is only caught while evaluating an actual
row — a query whose `MATCH` matches zero rows never gets checked at all).
`WHERE` does have real three-valued NULL logic (`AND`/`OR`/`NOT` and every
comparison correctly propagate "unknown" rather than collapsing it to
`false`) — CASE's `WHEN` and DISTINCT dedup deliberately don't, since they
need a definite yes/no, not "unknown".

## Roadmap

- `RETURN DISTINCT` (result-set-level dedup; `DISTINCT` inside an
  aggregate call already works)
- List-valued `$parameters`, to unblock `UNWIND $items AS x`
- From-scratch storage engine (page format, B-tree, crash recovery) as an
  alternate `marsdb-storage` backend, independent of redb
- Gremlin frontend targeting the existing IR
- Real query optimizer (cost-based join ordering, a semantic-validation
  binder pass) — the two targeted wins below (LIMIT push-down, ORDER
  BY+LIMIT top-k) are hand-special-cased, not a general framework

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

Attempts every scenario, including the ~75% that use Cypher features MarsDB
doesn't implement at all (temporal/spatial types, list comprehensions,
`CALL`, arithmetic expressions, ...) — those report as a distinct
"rejected at parse time" outcome, not lumped in with genuine wrong answers.
Of the scenarios MarsDB's grammar accepts at all, it gets the *right*
answer in the large majority — the real, checked-for-real signal this
exists to produce, not the flat pass-rate over the whole suite. Side-effect
assertions and the TCK's typed error taxonomy aren't checked (see the
crate's doc comments for why).

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or
[MIT license](./LICENSE-MIT) at your option.
