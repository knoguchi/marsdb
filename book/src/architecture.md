# Architecture

```
marsdb-storage   thin trait boundary over redb (file + in-memory backends)
marsdb-graph     property graph model, CRUD, KV/adjacency encoding
marsdb-query     openCypher subset: ANTLR4 grammar -> AST -> IR -> executor
marsdb           embeddable public Rust API (Database::open/in_memory/execute)
marsdb-cli       the `marsdb` binary (REPL + one-shot mode)
marsdb-python    PyO3 bindings, builds via maturin
marsdb-capi      C ABI (opaque handle + JSON results), basis for non-Rust bindings
marsdb-nl2cypher natural-language -> Cypher: schema introspection, prompt building, validate-and-repair
```

Go bindings live in a separate repository,
[knoguchi/marsdb-go](https://github.com/knoguchi/marsdb-go), linking
against `marsdb-capi` via cgo — see [Go bindings](./go.md).

## Storage

Storage runs on [redb](https://github.com/cberner/redb), a pure-Rust
single-file MVCC embedded KV engine. Every Cypher statement runs inside
one transaction — a read-only `MATCH ... RETURN` opens a
`ReadTransaction` (a consistent snapshot that runs alongside other
concurrent readers or a concurrent writer without contending for redb's
single-writer lock), everything else opens a `WriteTransaction`,
committed or aborted as a whole. `Database::begin_transaction` lets
callers explicitly extend that atomic boundary across multiple
statements. MarsDB records its own table/record format version in
metadata when the file is created or first opened by a version-aware
build, and refuses to open a database written by a newer unsupported
format.

A from-scratch storage engine (page format, B-tree, crash recovery) as
an alternate `marsdb-storage` backend independent of redb is on the
[roadmap](https://github.com/knoguchi/marsdb#roadmap), not built yet —
the trait boundary in `marsdb-storage` exists specifically so a second
backend could slot in later without touching `marsdb-graph`/
`marsdb-query`.

## Query execution

Query execution compiles Cypher to a small Gremlin-shaped logical IR
(`AllNodesScan`, `NodeByLabelScan`, `Seed`, `Expand`, `VarExpand`,
`Filter`, `IndexSeek`) so a future Gremlin frontend could target the same
executor. The parser is ANTLR4-generated (`marsdb-query/grammar/`),
replacing an earlier hand-rolled `pest` grammar.

The logical read plan runs as a pull-based row stream through node-ID
scans, filters, and relationship expansions. A non-aggregating,
non-distinct `RETURN ... LIMIT k` without `ORDER BY` stops that pipeline
after `k` rows, so downstream limits avoid unnecessary expansions.
Clause boundaries and inherently blocking operations still materialize:
`WITH`, optional-match reconciliation, variable-length traversal results
for each input row, aggregation, `DISTINCT`, mutations, and the public
`QueryResult`. Use `ExecutionOptions` to put hard ceilings on
intermediate rows, result rows, relationship expansions, and elapsed
time.

There is not yet a general cost-based optimizer. Two targeted
optimizations complement streaming: a direct `MATCH (n[:Label])
RETURN ... LIMIT k` scan pushes the limit into storage; and every
`ORDER BY ... LIMIT k` site uses a top-k partial selection
(`slice::select_nth_unstable_by` + a sort of just the k-sized prefix)
instead of a full sort of every row. Declared property indexes
(`CREATE INDEX ON :Label(prop)`) are used automatically when a `WHERE`/
inline-property equality matches one — see the [index seek
benchmarks](./benchmarks.md#property-indexes).

## Cypher coverage

See [Cypher Language Support](./cypher-support.md) for the full,
TCK-measured breakdown.
