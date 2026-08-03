# Benchmarks

Measured 2026-08-02 on a single MacBook (Apple Silicon, arm64), release build,
in-process (`cargo bench`, [criterion](https://github.com/bheisler/criterion.rs)).
No other graph database was benchmarked under the same conditions — these
numbers aren't a competitive comparison, they're here to track regressions
and show where the current architecture's cost is.

Reproduce: `cargo bench -p marsdb-graph` and `cargo bench -p marsdb` (runs
both `cypher_ops` and `ldbc_ops`).

## Storage layer (`marsdb-graph/benches/graph_ops.rs`)

| Operation | Result |
|---|---|
| `create_node` | 36.7 µs |
| `create_edge` | 48.7 µs |
| `get_node` (point lookup by id) | 1.7 µs |
| `neighbors`, 1-hop, fanout 1 | 1.2 µs |
| `neighbors`, 1-hop, fanout 10 | 1.7 µs |
| `neighbors`, 1-hop, fanout 100 | 7.0 µs |
| `neighbors`, 1-hop, fanout 1,000 | 55.3 µs |
| `all_nodes` scan, label matches 100% of rows, 100 rows | 81.6 µs |
| Same query, 1,000 rows | 877.8 µs |
| Same query, 10,000 rows | 9.08 ms |
| `all_nodes` scan, label matches 1% of rows, 100 rows | 2.9 µs |
| Same query, 1,000 rows | 10.9 µs |
| Same query, 10,000 rows | 91.5 µs |
| Same query, 100,000 rows | 1.03 ms |

`NODE_LABEL_INDEX` (label_id -> node_ids) backs label-filtered scans: a
lookup in that index plus one point-`get` per matching node, instead of
decoding every row in the table. This is a genuine trade, not a strict
win, and both sides show up above. On a query that only wants 1% of the
table, it's roughly 60-100x faster than the old full scan (91.5 µs vs. the
~6-9 ms a linear scan takes at the same table size) and stays close to flat
per matching row as the table grows to 100,000. But `create_node` got
~65% slower (22.3 µs -> 36.7 µs, extra index writes on every insert), and a
query where every row matches the label got ~30-45% slower (62.6 µs -> 81.6
µs at 100 rows, 6.22 ms -> 9.08 ms at 10,000) — the index still does N
random point-`get`s where the old code did one sequential pass, and there's
no matching-row count cheap enough to skip the index and fall back to a
plain scan when selectivity is high. `AllNodesScan` (no label filter, e.g.
`MATCH (n) RETURN n`) is untouched — it never had a filter to index against,
so it still does the one sequential pass and isn't affected either way.

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

## LDBC-era features (`marsdb/benches/ldbc_ops.rs`)

Benchmarks for the Cypher features added during the LDBC SNB Interactive
(IS1-IS7) push — `cypher_ops.rs` above only covers `CREATE` and a plain
1-hop `MATCH`, none of which exercise these. All queries run against the
same `(n0:Item)-[:R]->(n1:Item)-> ... ` chain fixture as the table above.

| Operation | Result |
|---|---|
| `WITH`-chaining (`MATCH...WITH...ORDER BY...LIMIT...MATCH...RETURN`), 100-node dataset | 654 µs |
| Same query, 1,000-node dataset | 6.18 ms |
| Same query, 10,000-node dataset | 62.3 ms |
| `OPTIONAL MATCH`, 100-node dataset | 603 µs |
| Same query, 1,000-node dataset | 6.16 ms |
| Same query, 10,000-node dataset | 62.4 ms |
| Undirected 1-hop (`-[:R]-`) + `LIMIT 10`, 100-node dataset | 606 µs |
| Same query, 1,000-node dataset | 5.94 ms |
| Same query, 10,000-node dataset | 61.8 ms |
| Variable-length `[:R*1..5]`, 1,000-node chain | 1.98 ms |
| Variable-length `[:R*1..30]`, 1,000-node chain | 2.08 ms |
| Variable-length `[:R*0..]` (unbounded, capped at 30 hops), 25-node chain | 154 µs |

`WITH`-chaining, `OPTIONAL MATCH`, and the undirected pattern all land in the
same range as each other and close to the plain directed 1-hop `MATCH` in
the table above (2.03 ms at 1,000 rows, 21.0 ms at 10,000) — the dominant
cost in all of them is still the unindexed label scan, not the new
mechanism layered on top. The undirected query is the one clear outlier
(61.8 ms vs 21.0 ms at 10,000 rows, ~3x): it runs `neighbors_in_txn` twice
per row (once per direction) plus a dedupe-by-edge-id pass, so it pays
roughly double the traversal work on top of the same scan.

Variable-length cost scales with the hop bound actually walked, not the
bound written in the query: `*1..5` and `*1..30` cost about the same
(1.98 ms vs 2.08 ms) on a 1,000-node chain because both terminate once the
chain runs out at ~30 hops in from any interior start node — see the depth
cap note below. The unbounded case uses a 25-node chain instead of the
1,000-node one: `*0..` on a chain longer than the 30-hop safety cap
(`executor.rs::VAR_EXPAND_DEPTH_CAP`) errors by design rather than silently
truncating (see README roadmap / `LogicalPlan::VarExpand`), so it can't be
measured at the same dataset sizes as the rest of this table without
tripping that guard.

Reproduce: `cargo bench -p marsdb --bench ldbc_ops`.

## Scope of these numbers

- No concurrent-access benchmarks — every statement currently runs through a
  single-writer transaction (see README), so this doesn't measure
  concurrent-reader throughput.
- No disk-backed sustained-write benchmarks — everything above ran against
  `Database::in_memory()`; file-backed throughput under real fsync pressure
  hasn't been measured separately.
- No comparison against Neo4j, JanusGraph, Neptune, or any other graph
  database.
- No benchmarks yet for `CASE`/function calls (`coalesce()`/`toInteger()`)
  in isolation — they're cheap scalar operations exercised inside the
  `WITH`-chaining query above, but not measured standalone.
