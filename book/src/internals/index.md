# MarsDB Internals

This part of the book is for developers, or anyone curious about
MarsDB's internals. The earlier chapters tell you how to *use* MarsDB;
this part tells you how it *works*: the on-disk layout, the transaction
machinery, the path a Cypher statement takes from text to rows, and the
reasoning behind the design decisions along the way.

A few crates, one storage engine, one query pipeline: ACID transactions,
crash safety, secondary indexes, a cost-aware planner, streaming
execution, and zero-copy results across three language boundaries.
Every performance figure in these chapters comes from this repository's
benchmark suite.

## Chapters

1. [Design Overview](./design-overview.md) — what MarsDB is, the crate
   stack, the life of a statement, and the transaction model.
2. [The Storage Layer](./storage.md) — redb, the thirteen tables, the
   transaction abstraction, backup and integrity.
3. [Graph Encoding](./graph-encoding.md) — the value model, interning,
   the record directory format, adjacency keys.
4. [The Write Path](./write-path.md) — the CRUD layer, table-handle
   caching, mutation anatomy, the integrity checker as invariant spec.
5. [The Query Frontend](./query-frontend.md) — grammar, AST design,
   parameters, semantic validation.
6. [The IR and the Planner](./ir-and-planner.md) — logical operators,
   pushdown, property indexes, index seeks, cost-based start-point
   selection, `EXPLAIN`.
7. [The Executor](./executor.md) — plan evaluation, bounded execution,
   three-valued logic, aggregation, the streaming lane.
8. [Results and Language Boundaries](./results-and-boundaries.md) —
   the result model, the C ABI, zero-copy Arrow export.
9. [Testing and Measurement](./testing-and-measurement.md) — the TCK,
   the crash harness, the benchmark ledger.
10. [Case Studies in Measured Trade-offs](./case-studies.md) — the
    measurements that made (and unmade) design decisions.
