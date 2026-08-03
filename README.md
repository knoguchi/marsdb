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
```

**Python** — not yet published to PyPI. Build locally from `marsdb-python/`:

```
cd marsdb-python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop
```

```python
import marsdb
db = marsdb.Database.in_memory()  # or .open(path)
db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
db.execute("MATCH (n:Person) RETURN n.name")
# -> [{'n.name': 'Alice'}, {'n.name': 'Bob'}]
```

## CLI usage

Mirrors `sqlite3`'s contract:

```
marsdb                          # in-memory REPL
marsdb mydata.db                # file-backed REPL
marsdb mydata.db "MATCH (n) RETURN n"   # run one query, exit
marsdb :memory: "..."           # explicit in-memory, one-shot
```

## Architecture

```
marsdb-storage   thin trait boundary over redb (file + in-memory backends)
marsdb-graph     property graph model, CRUD, KV/adjacency encoding
marsdb-query     openCypher subset: pest grammar -> AST -> IR -> executor
marsdb           embeddable public Rust API (Database::open/in_memory/execute)
marsdb-cli       the `marsdb` binary (REPL + one-shot mode)
marsdb-python    PyO3 bindings (not in the main Cargo workspace, builds via maturin)
```

Storage sits on [redb](https://github.com/cberner/redb) — a pure-Rust,
single-file, MVCC embedded KV engine — rather than a hand-rolled page/WAL
implementation. A from-scratch storage engine (page format, on-disk B-tree,
rollback-journal crash recovery) is a planned v2, implementing the same
`marsdb-storage` trait, once it passes the same crash-safety bar redb gets
for free today.

Query execution: Cypher parses to a small, deliberately Gremlin-shaped
logical IR (`AllNodesScan`, `NodeByLabelScan`, `Expand`, `Filter`, `Limit`)
so a future Gremlin frontend can compile into the same executor without a
rewrite.

**Crash safety**: one Cypher statement = one redb write transaction,
committed or aborted as a whole — not one transaction per node/edge write.
See [`BENCHMARKS.md`](./BENCHMARKS.md) for why that matters for both
correctness and performance.

## v1 limitations (honest list)

- **`MATCH` supports a single linear pattern only** — no comma-separated
  multi-pattern joins (`MATCH (a),(b)`). `CREATE` does support
  comma-separated patterns.
- **No secondary indexes.** Label-filtered scans (`MATCH (n:Person)`) are
  full linear table scans — see `BENCHMARKS.md` for the real cost at scale.
- **Reads serialize behind redb's single-writer lock.** Every statement,
  including pure `MATCH...RETURN`, currently opens a `WriteTransaction`
  rather than a `ReadTransaction`, so there's no concurrent-reader
  throughput yet. Note this isn't a storage-engine ceiling: redb already
  provides MVCC single-writer/many-concurrent-readers, the same shape
  SQLite gets from WAL mode (readers and the one writer run concurrently,
  no blocking either direction). Restoring concurrent reads in Mars means
  splitting the query executor's read path onto `ReadTransaction`, not
  building new storage-engine capability.
- **`LIMIT` doesn't short-circuit.** The planner eagerly evaluates the full
  scan/expand before truncating.
- **Cypher coverage**: `CREATE`, `MATCH`/`WHERE`/`RETURN`/`DELETE`/`DETACH
  DELETE`/`SET`, `LIMIT`. No `OPTIONAL MATCH`, aggregations, `ORDER BY`,
  variable-length paths, `UNION`, `WITH`-chains, `MERGE`, built-in
  functions, query parameters, or index/constraint DDL.
- **SIGKILL crash-safety at large-transaction scale isn't conclusively
  verified** — a real fault-injection harness (kill at every fsync point)
  is needed to settle this properly; ad-hoc `kill -9` testing wasn't
  reliable enough to be definitive.

None of these are surprises — they're the deliberate v1 scope trade-offs,
tracked here so they don't get overclaimed later.

## Benchmarks

See [`BENCHMARKS.md`](./BENCHMARKS.md). Micro-benchmarks only — no
comparison against other graph databases.

## Testing

```
cargo test --workspace                              # fast, ~1s
cargo test -p marsdb-graph --test stress -- --ignored --nocapture   # large-scale, ~15s
cargo bench -p marsdb-graph                          # storage benchmarks
cargo bench -p marsdb                                # Cypher-layer benchmarks
```

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or
[MIT license](./LICENSE-MIT) at your option.
