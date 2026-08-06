# Benchmarks

Measured on a single MacBook (Apple Silicon, arm64), release build,
in-process (`cargo bench`, [criterion](https://github.com/bheisler/criterion.rs)).
No other graph database was benchmarked under the same conditions —
these numbers aren't a competitive comparison, they're here to track
regressions and show where the current architecture's cost is. Full
detail, every dataset size, and the story behind each number:
[`BENCHMARKS.md`](https://github.com/knoguchi/marsdb/blob/main/BENCHMARKS.md)
in the repo root.

Reproduce: `cargo bench -p marsdb-graph` and `cargo bench -p marsdb`
(runs `cypher_ops`, `ldbc_ops`, `aggregate_ops`, `concurrency_ops`, and
`index_ops`).

## Storage layer

| Operation | Result |
|---|---|
| `create_node` | 37.1 µs |
| `create_edge` | 50.6 µs |
| `get_node` (point lookup by id) | 832 ns |
| `neighbors`, 1-hop, fanout 10 | 1.05 µs |
| `neighbors`, 1-hop, fanout 1,000 | 54.9 µs |
| `all_nodes` scan, label matches 1% of rows, 100,000 rows | 801 µs |

`NODE_LABEL_INDEX` backs label-filtered scans — roughly 30-80x faster
than a full scan when only a small fraction of rows match, staying close
to flat per matching row as the table grows.

## Cypher layer

| Operation | Result |
|---|---|
| Parse + execute, 10-hop `CREATE` | 3.58 ms |
| `MATCH (n)-[:R]->(m) RETURN m.idx LIMIT 10`, 10,000-node dataset | 1.284 ms |
| `MATCH (n:Label) RETURN n LIMIT 10` (no hop/`WHERE`/`ORDER BY`), 100,000-node dataset | 22.4 µs |

The last row is the direct payoff of pushing `LIMIT` into the storage
scan: flat ~19-22 µs regardless of dataset size (a ~1,000x-larger table
costs barely 17% more) — it really does stop at the first `LIMIT`
matches instead of scanning the whole table first.

## Property indexes

`MATCH (n:Item {idx: N}) RETURN n.idx` with and without
`CREATE INDEX ON :Item(idx)` declared:

| Dataset size | Unindexed scan | Index seek | Speedup |
|---|---|---|---|
| 100 | 78.6 µs | 7.36 µs | 10.7x |
| 10,000 | 8.43 ms | 7.87 µs | 1,071x |
| 100,000 | 92.4 ms | 7.72 µs | ~12,000x |

The index seek stays flat regardless of dataset size — it reads exactly
the matching entries, never touches the rest of the table.

## Aggregation

`resolve_grouped_rows` (the grouping core behind `count`/`sum`/`avg`/
`min`/`max`/`collect` and implicit `GROUP BY`) uses a hash-based group
lookup rather than a linear scan:

| Operation | Result (10,000 rows) |
|---|---|
| Global aggregate (1 group) | 54.5 ms |
| `GROUP BY cat` (10 groups) | 30.9 ms |
| `GROUP BY cat` (every row its own group) | 35.3 ms |
| `collect(n.idx)` | 20.1 ms |

All of these scale close to linearly with row count.

## Concurrent reads

A `MATCH ... RETURN` opens a `ReadTransaction`, not a `WriteTransaction`
— concurrent readers run in parallel instead of queueing behind redb's
single-writer lock. 200 queries against a 1,000-node dataset, single
thread vs. split across `N` threads sharing one `Arc<Database>`:

| Threads | Result | Speedup vs. 1 thread |
|---|---|---|
| 1 (sequential) | 339.1 ms | — |
| 4 | 197.0 ms | 1.72x |
| 8 | 181.5 ms | 1.87x |

Real, but sub-linear — plateaus around 4-8 threads on the 14-core
machine this was measured on.

## Scope of these numbers

- No disk-backed sustained-write benchmarks — everything above ran
  against `Database::in_memory()`.
- No comparison against Neo4j, JanusGraph, Neptune, or any other graph
  database.
