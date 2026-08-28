# Getting Started

The fastest way to try MarsDB is the CLI — no code required.

## Install

```
cargo install marsdb-cli
```

Or on macOS/Linux via Homebrew ([tap](https://github.com/knoguchi/homebrew-marsdb)):

```
brew install knoguchi/marsdb/marsdb
```

## Your first session

```
$ marsdb :memory:
MarsDB graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.
marsdb> CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'});
marsdb> MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name;
a.name | b.name
Alice | Bob
marsdb>
```

`:memory:` starts an in-memory database that disappears on exit. Give it
a file path instead — `marsdb graph.db` — and the data persists between
runs. Every statement ends with `;`; Ctrl-D exits the REPL.

## Running a script

A file of `;`-separated statements runs the same way piped through
stdin:

```
marsdb graph.db < setup.cypher
```

Or as a one-shot query against an existing database:

```
marsdb graph.db "MATCH (n:Person) RETURN n.name"
```

## Next steps

- Learn Cypher itself: [The Cypher Guide](./cypher-guide.md)
- Embed MarsDB in a program: [Embedding in Rust](./embedding-rust.md),
  [Python bindings](./python.md), or [Go bindings](./go.md)
- Every CLI flag: [CLI Reference](./cli.md)
