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

## Architecture

```
marsdb-storage   thin trait boundary over redb (file + in-memory backends)
marsdb-graph     property graph model, CRUD, KV/adjacency encoding
marsdb-query     openCypher subset: pest grammar -> AST -> IR -> executor
marsdb           embeddable public Rust API (Database::open/in_memory/execute)
marsdb-cli       the `marsdb` binary (REPL + one-shot mode)
marsdb-python    PyO3 bindings, builds via maturin
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

### Cypher coverage

`CREATE`, multi-label nodes (`(n:Post:Message)`), `$parameters`,
backslash-escaped string literals (`\' \" \\ \n \r \t \b \f`),
`MATCH`/`OPTIONAL MATCH`, undirected (`-[r:TYPE]-`) and variable-length
(`[:TYPE*min..max]`) relationship patterns, `WHERE`, one `WITH` boundary
per statement (projection/rename, its own `WHERE`/`WITH...WHERE`/`ORDER
BY`/`LIMIT`), `RETURN`/`DELETE`/`DETACH DELETE`/`SET`/`MATCH ... CREATE`
(adds an edge between two already-matched nodes — a node token whose
variable is already bound reuses that node instead of creating a new
one), multi-key `ORDER BY`, `LIMIT`, `CASE`, the built-in functions
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
arbitrarily.

Verified against all 7 of LDBC SNB Interactive's short-read reference
queries (IS1-IS7) — see `marsdb-query/tests/ldbc_is_queries.rs`. Not
verified: LDBC's complex queries (IC1-14: named paths/`shortestPath()`,
and the full query set beyond one hand-crafted
grouping+`WITH...WHERE`+`ORDER BY`+`LIMIT`+`collect()` checkpoint —
see `marsdb-query/tests/smoke.rs`), comma-separated patterns *within one*
`MATCH`/`CREATE` clause beyond a single linear chain (general
cross-joins — different from the cross-join WITH-chaining above, which
works), chaining past one `WITH` boundary, or `MERGE` patterns with more
than one relationship hop (whole-pattern atomicity across multiple
simultaneously-unbound hops isn't attempted).

## Roadmap

- `LIMIT` short-circuiting
- `RETURN DISTINCT` (result-set-level dedup; `DISTINCT` inside an
  aggregate call already works)
- List-valued `$parameters`, to unblock `UNWIND $items AS x`
- Named paths/`shortestPath()`
- From-scratch storage engine (page format, B-tree, crash recovery) as an
  alternate `marsdb-storage` backend, independent of redb
- Gremlin frontend targeting the existing IR

## Testing

```
cargo test --workspace                                             # ~1s
cargo test -p marsdb-graph --test stress -- --ignored --nocapture  # ~15s, large-scale
cargo bench -p marsdb-graph
cargo bench -p marsdb
```

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or
[MIT license](./LICENSE-MIT) at your option.
