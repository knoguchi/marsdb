# Introduction

MarsDB is an embeddable property-graph database with an openCypher query
subset: single binary, single file, optional in-memory mode. No server
process, no network protocol — it links into your Rust, Python, or Go
program (or runs standalone via the `marsdb` CLI) the same way SQLite
does for relational data.

```
$ marsdb :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
marsdb> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
marsdb> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
```

## Why a graph database

Relational databases model connections as foreign keys and join tables;
finding them back out means more joins, one per hop, and the query gets
harder to write and slower to run as the path gets longer. A graph
database stores the connection itself as a first-class thing, so
"friends of friends," "the shortest path between these two people," or
"everyone reachable within three hops" are direct pattern matches, not
a chain of joins.

## Why MarsDB specifically

- **A real subset of openCypher, not a lookalike.** Checked against the
  [openCypher Technology Compatibility Kit](https://github.com/opencypher/openCypher)
  on every push. See the [Cypher Language Reference](./cypher-support.md)
  for exactly what's covered.
- **Embeddable, not a server.** `Database::open("path/to.db")` or
  `Database::in_memory()` and you're running queries — no daemon to
  manage, no port to bind, no client/server protocol.
- **Crash-safe by construction.** Every Cypher statement runs inside one
  transaction; storage runs on [redb](https://github.com/cberner/redb),
  a pure-Rust MVCC single-file engine.
- **Bindings, not just a Rust crate.** Python (PyO3, prebuilt wheels) and
  Go (cgo against a small C ABI) both work today.

## Where to go next

- New to MarsDB? Start with [Getting Started](./getting-started.md).
- Want a Cypher walkthrough? [The Cypher Guide](./cypher-guide.md).
- Integrating into a program? [Embedding in Rust](./embedding-rust.md),
  [Python bindings](./python.md), or [Go bindings](./go.md).
- Wondering exactly which Cypher features are supported? See the
  [Cypher Language Reference](./cypher-support.md).
- Curious how it's built? See [Performance](./benchmarks.md), or the
  [Internals](./internals/index.md) appendix.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/knoguchi/marsdb/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/knoguchi/marsdb/blob/main/LICENSE-MIT)
at your option.
