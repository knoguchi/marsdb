# Performance

Numbers below are measured on a single MacBook (Apple Silicon, arm64),
release build, in-process (`cargo bench`,
[criterion](https://github.com/bheisler/criterion.rs)), against
`Database::in_memory()` unless noted otherwise. Full detail and every
dataset size are in
[`BENCHMARKS.md`](https://github.com/knoguchi/marsdb/blob/main/BENCHMARKS.md)
in the repo root.

## Compared to Neo4j

Loading the same real dataset — Neo4j's [recommendations example
graph](https://github.com/neo4j-graph-examples/recommendations) (movies
+ cast/crew from OMDb, users + ratings from MovieLens; 28,863 nodes,
166,261 relationships) — into both engines from the same generated
Cypher script, file-backed on both sides, wall-clock:

| Phase | MarsDB | Neo4j |
|---|---|---|
| Load | 64.9 s | 178.9 s |
| Query (5 read queries, lifted from Neo4j's own tutorial for this dataset) | 0.22 s | 1.27 s |
| Update (point update, bulk update, new relationship) | 0.09 s | 1.08 s |
| Delete (single relationship, `DETACH DELETE`, bulk) | 0.50 s | 1.12 s |

Neo4j ran in Docker (`neo4j:5.26`, official image); MarsDB ran natively.
Neo4j's numbers are phase totals via `cypher-shell`, not per-query
timings. This covers one dataset and one workload shape, not a general
claim across all query patterns — see `BENCHMARKS.md` for the full
methodology and the [marsdb-demo](https://github.com/knoguchi/marsdb-demo)
repo for the reproduction script.

## Typical query latencies

Point lookups and single-hop traversals, against a small in-memory
dataset:

| Operation | Result |
|---|---|
| `get_node` (point lookup by id) | 832 ns |
| 1-hop expansion, fanout 10 | 1.05 µs |
| 1-hop expansion, fanout 1,000 | 54.9 µs |
| `MATCH (n)-[:R]->(m) RETURN m.idx LIMIT 10`, 10,000-node dataset | 1.284 ms |

`MATCH (n:Label) RETURN ... LIMIT k` (no hop, no `WHERE`, no `ORDER BY`)
pushes the limit into the storage scan directly, so it stays flat
regardless of dataset size — about 22 µs whether the table has 100 rows
or 100,000:

| Operation | Result |
|---|---|
| `MATCH (n:Label) RETURN n LIMIT 10`, 100,000-node dataset | 22.4 µs |

## Property indexes

`MATCH (n:Item {idx: N}) RETURN n.idx`, with and without
`CREATE INDEX ON :Item(idx)` declared:

| Dataset size | Unindexed scan | Index seek | Speedup |
|---|---|---|---|
| 100 | 78.6 µs | 7.36 µs | 10.7x |
| 10,000 | 8.43 ms | 7.87 µs | 1,071x |
| 100,000 | 92.4 ms | 7.72 µs | ~12,000x |

The index seek stays flat regardless of dataset size — it reads exactly
the matching entries. Declare an index for any property you filter or
join on at meaningful scale; `CREATE INDEX ON :Label(prop)`, see the
[Cypher Language Reference](./cypher-support.md).

## Aggregation

`count`/`sum`/`avg`/`min`/`max`/`collect` and implicit `GROUP BY` use a
hash-based group lookup, scaling close to linearly with row count:

| Operation | Result (10,000 rows) |
|---|---|
| Global aggregate (1 group) | 54.5 ms |
| `GROUP BY cat` (10 groups) | 30.9 ms |
| `GROUP BY cat` (every row its own group) | 35.3 ms |
| `collect(n.idx)` | 20.1 ms |

## Concurrency

A `MATCH ... RETURN` opens a read transaction, not a write transaction —
concurrent readers run in parallel instead of queueing behind the
single-writer lock. 200 queries against a 1,000-node dataset, single
thread vs. split across `N` threads sharing one `Arc<Database>`:

| Threads | Result | Speedup vs. 1 thread |
|---|---|---|
| 1 (sequential) | 339.1 ms | — |
| 4 | 197.0 ms | 1.72x |
| 8 | 181.5 ms | 1.87x |

Sub-linear — it plateaus around 4-8 threads on the 14-core machine this
was measured on. Writers still serialize behind each other
(one write transaction at a time); see [Operations](./operations.md) for
the full concurrency model.

## Reproducing these numbers

```
cargo bench -p marsdb-graph
cargo bench -p marsdb
```

The second command runs `cypher_ops`, `ldbc_ops`, `aggregate_ops`,
`concurrency_ops`, and `index_ops`. Numbers above are single
measurements on one machine — expect variance run to run, and different
numbers entirely on different hardware.

## Scope of these numbers

- No disk-backed sustained-write benchmarks — the numbers above other
  than the Neo4j comparison ran against `Database::in_memory()`.
- The Neo4j comparison covers one dataset load/query/update/delete
  workflow, not a general benchmark suite — no JanusGraph, Neptune, or
  other graph database is compared here.
