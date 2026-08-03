# Benchmarks

Measured 2026-08-02 on a single MacBook (Apple Silicon, arm64), release build,
in-process (`cargo bench`, [criterion](https://github.com/bheisler/criterion.rs)).
No other graph database was benchmarked under the same conditions — these
numbers aren't a competitive comparison, they're here to track regressions
and show where the current architecture's cost is.

Reproduce: `cargo bench -p marsdb-graph` and `cargo bench -p marsdb`.

## Storage layer (`marsdb-graph/benches/graph_ops.rs`)

| Operation | Result |
|---|---|
| `create_node` | 22.3 µs |
| `create_edge` | 48.7 µs |
| `get_node` (point lookup by id) | 1.6 µs |
| `neighbors`, 1-hop, fanout 1 | 1.2 µs |
| `neighbors`, 1-hop, fanout 10 | 1.7 µs |
| `neighbors`, 1-hop, fanout 100 | 7.1 µs |
| `neighbors`, 1-hop, fanout 1,000 | 55.6 µs |
| `all_nodes` scan, 100 rows | 62.6 µs |
| `all_nodes` scan, 1,000 rows | 619.5 µs |
| `all_nodes` scan, 10,000 rows | 6.22 ms |

`all_nodes` is a linear scan of the whole node table filtered by label —
there's no secondary index on label, so cost scales with total table size,
not with the number of matching rows. At 10M rows this would be roughly
6 seconds, not milliseconds. A label index is on the roadmap (see README).

### Transaction batching

Comparison of one write-transaction per node vs. one shared transaction for
1,000 node creates:

| Strategy | Result |
|---|---|
| One `WriteTransaction` per node | 17.6 ms |
| One shared `WriteTransaction` for all 1,000 | 6.17 ms (2.85x faster) |

MarsDB runs each Cypher statement inside a single transaction rather than
one transaction per graph operation, for two reasons this table shows: it's
faster, and it means a statement that touches multiple nodes/edges (e.g.
`CREATE (a)-[:R]->(b)`) either fully applies or fully rolls back — no
partially-created pattern if the process dies mid-statement.

## Cypher layer (`marsdb/benches/cypher_ops.rs`)

| Operation | Result |
|---|---|
| Parse only, 10-hop `CREATE` | 14.1 µs |
| Parse only, 100-hop `CREATE` | 124.9 µs |
| Parse only, 1,000-hop `CREATE` | 1.22 ms |
| Parse + execute, 10-hop `CREATE` | 3.21 ms |
| Parse + execute, 100-hop `CREATE` | 4.23 ms |
| Parse + execute, 1,000-hop `CREATE` | 13.3 ms |
| `MATCH (n)-[:R]->(m) RETURN m.idx LIMIT 10`, 100-node dataset | 228 µs |
| Same query, 1,000-node dataset | 2.03 ms |
| Same query, 10,000-node dataset | 21.0 ms |

`LIMIT` does not short-circuit: the last row shows 21 ms for a query that
only returns 10 rows, because the query planner evaluates the full scan and
expand before truncating to the limit. `LIMIT` short-circuiting is on the
roadmap.

## Scope of these numbers

- No concurrent-access benchmarks — every statement currently runs through a
  single-writer transaction (see README), so this doesn't measure
  concurrent-reader throughput.
- No disk-backed sustained-write benchmarks — everything above ran against
  `Database::in_memory()`; file-backed throughput under real fsync pressure
  hasn't been measured separately.
- No comparison against Neo4j, JanusGraph, Neptune, or any other graph
  database.
