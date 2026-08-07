# Introduction

MarsDB is an embeddable property-graph database with an openCypher query
subset: single binary, single file, optional in-memory mode. No server
process, no network protocol — it links into your Rust, Python, or Go
program (or runs standalone via the `mars` CLI) the same way SQLite
does for relational data.

```
$ mars :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
mars> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
mars> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```

## Why MarsDB

- **openCypher subset, measured against the real spec.** MarsDB is
  checked against the [openCypher Technology Compatibility Kit
  (TCK)](https://github.com/opencypher/openCypher) — 3,880 real
  conformance scenarios — not just its own test suite. See [Cypher
  Language Support](./cypher-support.md) for the exact, current
  pass rate and what's covered.
- **Embeddable, not a server.** `Database::open("path/to.db")` or
  `Database::in_memory()` and you're running queries — no daemon to
  manage, no port to bind, no client/server protocol.
- **Crash-safe by construction.** Every Cypher statement runs inside one
  transaction; `marsdb-storage` runs on [redb](https://github.com/cberner/redb),
  a pure-Rust MVCC single-file engine, and a `SIGKILL`-and-reopen crash
  harness checks that every acknowledged commit survives intact.
- **Bindings, not just a Rust crate.** Python (PyO3, prebuilt wheels) and
  Go (cgo against a small C ABI crate) both work today — see [Python
  bindings](./python.md) and [Go bindings](./go.md).

## Where to go next

- New to MarsDB? Start with [Install & CLI](./cli.md), or jump straight
  to [Embedding in Rust](./embedding-rust.md) if you're integrating it
  into a program.
- Wondering exactly which Cypher features are supported? See [Cypher
  Language Support](./cypher-support.md).
- Curious how it's built? See [Architecture](./architecture.md) and
  [Benchmarks](./benchmarks.md).

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/knoguchi/marsdb/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/knoguchi/marsdb/blob/main/LICENSE-MIT)
at your option.
