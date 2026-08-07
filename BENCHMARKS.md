# Benchmarks

Measured 2026-08-03 on a single MacBook (Apple Silicon, arm64), release build,
in-process (`cargo bench`, [criterion](https://github.com/bheisler/criterion.rs)).
These numbers aren't a competitive comparison, they're here to track
regressions and show where the current architecture's cost is — with one
exception, a real dataset-load comparison against Neo4j, see
[Load comparison](#load-comparison-recommendations-dataset) below.

Reproduce: `cargo bench -p marsdb-graph` and `cargo bench -p marsdb` (runs
`cypher_ops`, `ldbc_ops`, `aggregate_ops`, `concurrency_ops`, and
`index_ops`).

## Storage layer (`marsdb-graph/benches/graph_ops.rs`)

| Operation | Result |
|---|---|
| `create_node` | 37.1 µs |
| `create_edge` | 50.6 µs |
| `get_node` (point lookup by id) | 832 ns |
| `neighbors`, 1-hop, fanout 1 | 620 ns |
| `neighbors`, 1-hop, fanout 10 | 1.05 µs |
| `neighbors`, 1-hop, fanout 100 | 6.42 µs |
| `neighbors`, 1-hop, fanout 1,000 | 54.9 µs |
| `all_nodes` scan, label matches 100% of rows, 100 rows | 57.7 µs |
| Same query, 1,000 rows | 652 µs |
| Same query, 10,000 rows | 6.97 ms |
| `all_nodes` scan, label matches 1% of rows, 100 rows | 1.77 µs |
| Same query, 1,000 rows | 7.60 µs |
| Same query, 10,000 rows | 68.3 µs |
| Same query, 100,000 rows | 801 µs |

`NODE_LABEL_INDEX` (label_id -> node_ids) backs label-filtered scans: a
lookup in that index plus one point-`get` per matching node, instead of
decoding every row in the table. This is a genuine trade, not a strict
win: on a query that only wants 1% of the table, it's roughly 30-80x
faster than a full scan at the same size (801 µs vs. the ~7 ms `all_nodes`
takes at 100% selectivity) and stays close to flat per matching row as the
table grows. `create_node` pays for the extra index write on every insert.
`AllNodesScan` (no label filter, e.g. `MATCH (n) RETURN n`) is untouched —
it never had a filter to index against, so it still does one sequential
pass either way.

### Transaction batching

Comparison of one write-transaction per node vs. one shared transaction for
1,000 node creates:

| Strategy | Result |
|---|---|
| One `WriteTransaction` per node | 26.2 ms |
| One shared `WriteTransaction` for all 1,000 | 8.67 ms (3.0x faster) |

MarsDB runs each Cypher statement inside a single transaction rather than
one transaction per graph operation, for two reasons this table shows: it's
faster, and it means a statement that touches multiple nodes/edges (e.g.
`CREATE (a)-[:R]->(b)`) either fully applies or fully rolls back — no
partially-created pattern if the process dies mid-statement.

## Cypher layer (`marsdb/benches/cypher_ops.rs`)

| Operation | Result |
|---|---|
| Parse only, 10-hop `CREATE` | 15.7 µs |
| Parse only, 100-hop `CREATE` | 135 µs |
| Parse only, 1,000-hop `CREATE` | 1.34 ms |
| Parse + execute, 10-hop `CREATE` | 3.58 ms |
| Parse + execute, 100-hop `CREATE` | 4.75 ms |
| Parse + execute, 1,000-hop `CREATE` | 15.9 ms |
| `MATCH (n)-[:R]->(m) RETURN m.idx LIMIT 10`, 100-node dataset | 30.4 µs |
| Same query, 1,000-node dataset | 138 µs |
| Same query, 10,000-node dataset | 1.284 ms |
| `MATCH (n:Label) RETURN n LIMIT 10` (no hop/WHERE/ORDER BY), 100-node dataset | 19.2 µs |
| Same query, 1,000-node dataset | 20.8 µs |
| Same query, 10,000-node dataset | 20.9 µs |
| Same query, 100,000-node dataset | 22.4 µs |

The last four rows push `LIMIT` through the pull pipeline straight into the
storage scan (no hop, no `WHERE`, no `ORDER BY` — see the Architecture
section in `README.md`). The result: flat, ~19-22 µs
regardless of dataset size, a ~1,000x-larger table costing barely 17% more
— confirming it really does stop at the first `LIMIT` matches, not scan
the whole table first. This needed two things to actually hold: the
LIMIT-aware scan in `execute_match`, and a fix caught *while writing this
benchmark* — `all_nodes_limited_in_txn`'s label-filtered path was
collecting every matching id from `NODE_LABEL_INDEX` before truncating to
`limit`, so it only skipped the (more expensive) per-id point-reads, not
the index walk itself; the first version of this table showed
21.9 µs -> 519 µs -> 4.98 ms, clearly *not* flat, which is what caught it.
`.take(limit)` on the multimap iterator itself fixed it — the numbers
above are post-fix.

The 1-hop query now benefits from streaming too: the scan supplies node ids
without decoding every node record, and `Expand` produces one row at a time.
Once ten rows survive the pipeline, `LIMIT 10` stops requesting input. The
previous materializing executor measured 267 µs / 2.66 ms / 27.9 ms for the
same sizes, so this change is about 8.8x / 19.3x / 21.7x faster. It still
scales with dataset size because the unindexed starting-node scan must
discover candidate ids; property indexes and optimizer-selected seeks are a
separate tranche.

## LDBC-era features (`marsdb/benches/ldbc_ops.rs`)

Benchmarks for the Cypher features added during the LDBC SNB Interactive
(IS1-IS7) push — `cypher_ops.rs` above only covers `CREATE` and a plain
1-hop `MATCH`, none of which exercise these. All queries run against the
same `(n0:Item)-[:R]->(n1:Item)-> ... ` chain fixture as the table above.

| Operation | Result |
|---|---|
| `WITH`-chaining (`MATCH...WITH...ORDER BY...LIMIT...MATCH...RETURN`), 100-node dataset | 499 µs |
| Same query, 1,000-node dataset | 4.91 ms |
| Same query, 10,000-node dataset | 51.6 ms |
| `OPTIONAL MATCH`, 100-node dataset | 491 µs |
| Same query, 1,000-node dataset | 5.07 ms |
| Same query, 10,000-node dataset | 52.4 ms |
| Undirected 1-hop (`-[:R]-`) + `LIMIT 10`, 100-node dataset | 466 µs |
| Same query, 1,000-node dataset | 4.71 ms |
| Same query, 10,000-node dataset | 49.2 ms |
| Variable-length `[:R*1..5]`, 1,000-node chain | 1.68 ms |
| Variable-length `[:R*1..30]`, 1,000-node chain | 1.76 ms |
| Variable-length `[:R*0..]` (unbounded, capped at 30 hops), 25-node chain | 116 µs |

`WITH`-chaining and `OPTIONAL MATCH` now cost noticeably more than the
plain directed 1-hop `MATCH` above (51.6/52.4 ms vs. 27.9 ms at 10,000
rows, ~1.9x) — each layers real extra work on top of one `Expand` (a
second bounded pass for `WITH`'s `ORDER BY`+`LIMIT`, or the
tag-group-pad bookkeeping `eval_optional_part` does), and both also
inherited the same per-`Expand` edge-isomorphism overhead the plain 1-hop
query did (see above). The undirected query is still the clear standalone
outlier in absolute terms (49.2 ms) but the *gap* to the plain directed
query narrowed a lot since this was first measured (was ~2.9x at 10,000
rows, now ~1.8x) — consistent with a roughly-constant per-`Expand`
overhead landing on both sides of that comparison, shrinking the relative
difference between them without changing what actually causes the
undirected query's own extra cost (`neighbors_in_txn` runs twice per row,
once per direction, plus a dedupe-by-edge-id pass).

Variable-length cost scales with the hop bound actually walked, not the
bound written in the query: `*1..5` and `*1..30` cost about the same
(1.68 ms vs 1.76 ms) on a 1,000-node chain because both terminate once the
chain runs out at ~30 hops in from any interior start node — see the depth
cap note below. The unbounded case uses a 25-node chain instead of the
1,000-node one: `*0..` on a chain longer than the 30-hop safety cap
(`executor.rs::VAR_EXPAND_DEPTH_CAP`) errors by design rather than silently
truncating, so it can't be measured at the same dataset sizes as the rest
of this table without tripping that guard.

Reproduce: `cargo bench -p marsdb --bench ldbc_ops`.

## Property indexes (`marsdb/benches/index_ops.rs`)

Measured 2026-08-04. `MATCH (n:Item {idx: N}) RETURN n.idx` with and
without `CREATE INDEX ON :Item(idx)` declared — the direct payoff of
`IndexSeek` (`marsdb-query/src/planner.rs::apply_index_seeks`) over a
label scan + filter:

| Dataset size | Unindexed scan | Index seek | Speedup |
|---|---|---|---|
| 100 | 78.6 µs | 7.36 µs | 10.7x |
| 1,000 | 828 µs | 7.52 µs | 110x |
| 10,000 | 8.43 ms | 7.87 µs | 1,071x |
| 100,000 | 92.4 ms | 7.72 µs | ~12,000x |

The index seek stays flat (7.4-7.9 µs) regardless of dataset size — it
reads exactly the matching entries via `PROPERTY_INDEX`, never touches
the other 99,999 rows — while the unindexed scan grows linearly, since
every row has to be decoded and filtered. This is the whole reason the
index exists; the gap widens with table size, not stays constant.

Cardinality-based index selection (choosing the most selective of
several indexed `WHERE` conjuncts, `GraphStore::index_match_count_in_txn`
— see `marsdb-query/src/planner.rs::apply_index_seeks`'s candidate
selection): 50,000 `Person` nodes, `country = 'US'` matching ~100% of
rows and `email` matching exactly one, both compared against
`WHERE n.country = 'US' AND n.email = '...'`:

| Indexes declared | Result |
|---|---|
| Only `country` (low selectivity — no better option exists) | 44.1 ms |
| Both `country` and `email` (planner picks `email`) | 12.0 µs |

~3,675x — this is the real, measured version of the same comparison
originally done ad hoc while building the feature (44ms → microseconds),
now a real, repeatable `criterion` benchmark instead of a one-off
scratch measurement.

`LIMIT` pushed into a non-unique index seek's own storage lookup
(`stream_index_seek`'s budget-aware `storage_limit`, stops the multimap
walk itself rather than materializing every match first): 1,000 `Tokyo`-
valued rows out of N total, `MATCH (n:Item {city: 'Tokyo'}) RETURN n.idx`
with and without `LIMIT 1`:

| Dataset size | `LIMIT 1` | Unbounded | Speedup |
|---|---|---|---|
| 1,000 | 8.53 µs | 831 µs | 97x |
| 10,000 | 8.54 µs | 8.68 ms | 1,016x |
| 100,000 | 8.74 µs | 88.2 ms | ~10,100x |

Same flat-vs-linear shape as the first table, for the same reason:
`LIMIT 1` stops after the first storage-level match instead of
collecting all 1,000 `Tokyo` rows before truncating.

Reproduce: `cargo bench -p marsdb --bench index_ops`.

## Aggregation (`marsdb/benches/aggregate_ops.rs`)

`resolve_grouped_rows` (the grouping core behind `count`/`sum`/`avg`/`min`/
`max`/`collect` and implicit `GROUP BY`) looks up a row's group via a
`HashMap` keyed by `HashKey` — a hashable stand-in for `Binding`/`Value`,
needed because `PropertyValue`/`Node`/`Edge` don't derive `Eq`/`Hash`
themselves (`PropertyValue::Float(f64)` can't; `HashKey` hashes floats by
bit pattern instead — see its doc comment in `aggregate.rs`). `DISTINCT`
dedup (`count(DISTINCT ...)`, `collect(DISTINCT ...)`, etc., and now
`RETURN DISTINCT`'s whole-row dedup too — see `executor.rs`'s `dedup_rows`)
uses the same `HashKey` in a `HashSet`, replacing what was originally a
linear scan/rescan (the 2026-08-02 measurement this table is based on
found the worst case — every row its own group — at 141 ms for 10,000
rows; the hash-based version below is 31.1 ms). All queries run against
`n` `Item` nodes created in one `CREATE` (one transaction), `cat` =
`idx % num_groups`.

| Operation | Result |
|---|---|
| Global aggregate (`count(*)`/`sum`/`avg`/`min`/`max`, 1 group), 100 rows | 479 µs |
| Same query, 1,000 rows | 5.04 ms |
| Same query, 10,000 rows | 54.5 ms |
| `GROUP BY cat` (10 groups), 100 rows | 284 µs |
| Same query, 1,000 rows | 2.92 ms |
| Same query, 10,000 rows | 30.9 ms |
| `GROUP BY cat` (every row its own group), 100 rows | 311 µs |
| Same query, 1,000 rows | 3.28 ms |
| Same query, 10,000 rows | 35.3 ms |
| `collect(n.idx)`, 100 rows | 181 µs |
| Same query, 1,000 rows | 1.87 ms |
| Same query, 10,000 rows | 20.1 ms |
| `count(DISTINCT n.cat)` (every row a distinct value), 100 rows | 181 µs |
| Same query, 1,000 rows | 1.95 ms |
| Same query, 10,000 rows | 21.4 ms |
| `WITH...WHERE` on an aggregate result (10 groups), 100 rows | 282 µs |
| Same query, 1,000 rows | 2.92 ms |
| Same query, 10,000 rows | 31.0 ms |

All of these scale close to linearly with row count (100->1,000->10,000 is
consistently ~10x->~10x across every operation above) — the property the
original hash-based rewrite was for. See the 0.3.0 entry in
`CHANGELOG.md` for the original linear-scan-vs-hash-based comparison this
change was measured against; that comparison isn't re-run here since the
old linear-scan code path no longer exists to benchmark against directly.

Reproduce: `cargo bench -p marsdb --bench aggregate_ops`.

## Concurrent reads (`marsdb/benches/concurrency_ops.rs`)

A `MATCH ... RETURN` opens a `ReadTransaction`, not a `WriteTransaction` —
concurrent readers run in parallel instead of queueing behind redb's
single-writer lock (see README). 200 `MATCH (n:Item) RETURN n.idx` queries
against a 1,000-node dataset, done on a single thread vs. split evenly
across `N` threads sharing one `Arc<Database>`:

| Threads | Result | Speedup vs. 1 thread |
|---|---|---|
| 1 (sequential) | 339.1 ms | — |
| 2 | 268.2 ms | 1.26x |
| 4 | 197.0 ms | 1.72x |
| 8 | 181.5 ms | 1.87x |

Real, but sub-linear, and it plateaus around 4-8 threads on the 14-core
(10P+4E) machine this was measured on — nowhere near "N threads = N times
faster." Two things this benchmark doesn't isolate: each `b.iter()` call
spawns fresh OS threads via `std::thread::scope` rather than reusing a
pool, and this hasn't been checked against a profiler for lock contention
inside redb's own read-transaction bookkeeping — either could be inflating
the per-thread overhead that caps the speedup here. The number that
matters regardless: 2+ threads reading concurrently are reliably faster
than 1, confirming the feature does what it's for, not just that it
compiles.

Reproduce: `cargo bench -p marsdb --bench concurrency_ops`.

## Load comparison: recommendations dataset

Measured 2026-08-06, same MacBook as above. Not a `criterion` micro-bench —
an end-to-end load of a real, fixed dataset: Neo4j's [recommendations
example graph](https://github.com/neo4j-graph-examples/recommendations)
(movies + cast/crew from OMDb, users + ratings from MovieLens), extracted
from its own `neo4j-admin database load` dump via a real Neo4j 5.26
instance and `apoc.export.cypher.all`, then loaded into both engines from
the *same* generated Cypher script (source-data provenance and extraction
steps are in [marsdb-demo](https://github.com/knoguchi/marsdb-demo)'s
`recommendations` demo). File-backed database on both sides, wall-clock
time for the whole load, single run each (not averaged over multiple
trials).

Neo4j ran in Docker (`neo4j:5.26`, official image); MarsDB ran natively.
That's a real, uncontrolled difference in the comparison, not something
this table corrects for.

| | Nodes | Relationships | Load time |
|---|---|---|---|
| MarsDB | 28,863 | 166,261 | 59.7 s |
| Neo4j, with the script's own `CREATE CONSTRAINT`s (unique indexes) | 28,863 | 166,261 | 162.8 s |
| Neo4j, constraints swapped for plain (non-unique) indexes matching MarsDB's | 28,863 | 166,261 | 144.7 s |

Both engines loaded via plain Cypher (`UNWIND` batches of `MERGE`/`CREATE`,
no bulk/CSV-import fast path on either side), auto-committing one
transaction per statement. Node/relationship counts are byte-identical
across all three rows — same source data, same final graph, only load
time differs.

Dropping Neo4j's uniqueness constraints for plain indexes only recovered
~18s (162.8s -> 144.7s) — most of the gap isn't constraint-checking
overhead. What's actually different between the two engines' write paths
at this workload isn't isolated by this measurement.

## Scope of these numbers

- No disk-backed sustained-write benchmarks — everything above ran against
  `Database::in_memory()`; file-backed throughput under real fsync pressure
  hasn't been measured separately.
- The Neo4j comparison above covers one dataset *load*, nothing else —
  no query-latency comparison, no other dataset, no JanusGraph/Neptune/
  other graph database.
- No benchmarks yet for `CASE`/function calls (`coalesce()`/`toInteger()`)
  in isolation — they're cheap scalar operations exercised inside the
  `WITH`-chaining query above, but not measured standalone.
- No benchmarks for `REMOVE`, `SET`-label, or the `STARTS WITH`/`ENDS
  WITH`/`CONTAINS` string predicates yet.
