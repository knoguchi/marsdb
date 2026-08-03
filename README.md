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
`MATCH`/`OPTIONAL MATCH`, undirected (`-[r:TYPE]-`) and variable-length
(`[:TYPE*min..max]`) relationship patterns, `WHERE`, one `WITH` boundary
per statement (projection/rename, its own `WHERE`/`WITH...WHERE`/`ORDER
BY`/`LIMIT`), `RETURN`/`DELETE`/`DETACH DELETE`/`SET`, multi-key `ORDER
BY`, `LIMIT`, `CASE`, the built-in functions `coalesce()`/`toInteger()`,
and implicit-GROUP-BY aggregation (`count()`/`count(*)`/`sum()`/`avg()`/
`min()`/`max()`/`collect()`, with `DISTINCT`).

Verified against all 7 of LDBC SNB Interactive's short-read reference
queries (IS1-IS7) — see `marsdb-query/tests/ldbc_is_queries.rs`. Not
verified: LDBC's complex queries (IC1-14: `UNWIND`, named
paths/`shortestPath()`, and the full query set beyond one hand-crafted
grouping+`WITH...WHERE`+`ORDER BY`+`LIMIT`+`collect()` checkpoint —
see `marsdb-query/tests/smoke.rs`), comma-separated `MATCH` patterns
beyond a single linear chain (general cross-joins), or chaining past one
`WITH` boundary.

## Roadmap

- `LIMIT` short-circuiting
- `UNWIND`, named paths/`shortestPath()`
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
