# Benchmarks

Measured 2026-08-02 on a single MacBook (Apple Silicon, arm64), release build,
in-process (`cargo bench`, [criterion](https://github.com/bheisler/criterion.rs)).
Not a comparison against any other graph database — no other engine was
benchmarked under the same conditions, so don't read these as competitive
claims. They're here to track regressions and to be honest about where the
current v1 architecture's ceiling is.

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

`all_nodes` is a **linear scan** — there's no secondary index on label yet
(see Limitations below), so this scales O(n) with total table size, not with
the number of matching rows. At 10M rows this would be roughly 6 seconds, not
milliseconds.

### Transaction batching

Direct comparison of one write-transaction per node vs. one shared
transaction for 1,000 node creates:

| Strategy | Result |
|---|---|
| One `WriteTransaction` per node | 17.6 ms |
| One shared `WriteTransaction` for all 1,000 | 6.17 ms (**2.85x faster**) |

This is why the query executor drives an entire Cypher statement through a
single transaction (see the crash-safety boundary in the architecture notes)
rather than one transaction per graph operation — it's both the correctness
fix and the performance win.

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

**`LIMIT` does not short-circuit in v1.** The last row above is the tell:
21 ms for a query that only returns 10 rows, because the planner eagerly
scans and expands the full dataset before truncating. Not a bug — a known
consequence of the "no cost-based optimizer, nothing to misoptimize" v1
scope — but a real cost if you run `LIMIT`-heavy queries against large
datasets today.

## What these numbers don't tell you

- No concurrent-access benchmarks. v1's query executor drives every
  statement — reads included — through a `WriteTransaction`, so reads
  currently serialize behind redb's single-writer lock instead of running
  concurrently. See Limitations in the README.
- No disk-backed sustained-write benchmarks (all of the above ran against
  `Database::in_memory()`); file-backed throughput under real fsync pressure
  hasn't been separately measured.
- No comparison against Neo4j, JanusGraph, Neptune, or any other graph
  database. Don't extrapolate a competitive ranking from this file.
