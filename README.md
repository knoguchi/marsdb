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

**Python**:

```
cd marsdb-python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin && maturin develop
```

```python
import marsdb
db = marsdb.Database.in_memory()  # or .open(path)
db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
db.execute("MATCH (n:Person) RETURN n.name")
# -> [{'n.name': 'Alice'}, {'n.name': 'Bob'}]
```

Not yet on PyPI.

## CLI usage

```
marsdb                                  # in-memory REPL
marsdb mydata.db                        # file-backed REPL
marsdb mydata.db "MATCH (n) RETURN n"   # run one query, exit
marsdb :memory: "..."                   # explicit in-memory, one-shot
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
`Expand`, `Filter`, `Limit`) so a future Gremlin frontend can target the
same executor. Every Cypher statement runs inside one transaction, committed
or aborted as a whole.

Numbers: [`BENCHMARKS.md`](./BENCHMARKS.md).

## Roadmap

- Secondary index on node label (label scans are currently linear)
- Concurrent reads (split the executor's read path onto `ReadTransaction`)
- Multi-pattern `MATCH` (comma-separated joins)
- `LIMIT` short-circuiting
- Hand-rolled storage engine as an alternate `marsdb-storage` backend
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
