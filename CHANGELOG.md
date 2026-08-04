# Changelog

All notable changes to MarsDB are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.5.0] - 2026-08-03

### Added
- `marsdb-tck`: runs the real openCypher Technology Compatibility Kit
  against MarsDB (220 vendored `.feature` files pulled in as a pinned git
  submodule, `Scenario Outline`+`Examples` fully expanded — 3880 real
  scenarios) via a purpose-built Gherkin-subset parser and structural
  result comparison. `cargo run --release -p marsdb-tck`; see
  `marsdb-tck/VENDOR.md`. 831/3880 pass, 0 `WrongResult`.
- `REMOVE n.prop` / `REMOVE n:Label`, and `SET n:Label` (`SET` previously
  only did property assignment) — like `DELETE`, can't yet be followed by
  `RETURN` in the same statement (a terminal tail, not a chainable clause).
- `STARTS WITH`/`ENDS WITH`/`CONTAINS` string predicates.
- `RETURN DISTINCT` — result-set-level dedup of the whole projected row,
  separate from `DISTINCT` inside an aggregate call (`count(DISTINCT x)`),
  which already worked.
- `LIMIT` push-down: `MATCH (n[:Label]) RETURN ... LIMIT k` with no
  `WHERE`/hops/`ORDER BY` stops the storage scan at the first `k` matches
  instead of scanning the whole table — see BENCHMARKS.md (flat ~19-22 µs
  across a 1,000x dataset size range, vs. linear before).
- `ORDER BY ... LIMIT k`: all three sites (`WITH`'s own, non-aggregating
  `RETURN`'s, aggregating `RETURN`'s) now use a top-k partial selection
  instead of a full sort of every row just to discard all but the first
  few.
- Real three-valued NULL logic in `WHERE`: `AND`/`OR`/`NOT` and every
  comparison correctly propagate "unknown" instead of collapsing it to
  `false` (`x = null`, or a comparison against a missing property, is
  unknown — never true or false).

### Fixed
25 real wrong-answer bugs found via the new TCK suite, all independently
verified via direct CLI execution before being trusted, not just the
harness's own report:
- Label-less nodes no longer get an implicit `:Node` label on `CREATE`.
- A self-referencing `CREATE` pattern (`(a)-[:LOOP]->(a)`, or a
  comma-separated pattern reusing an earlier one's variable) now correctly
  creates one node with a self-loop, not two separate nodes — the root
  cause behind what looked like "undirected matching double-counts
  self-loops."
- Relationship inline properties (`-[:KNOWS {name: 'x'}]->`) are now
  filtered on (previously parsed but silently ignored).
- Relationship uniqueness (edge isomorphism): a hop can no longer silently
  re-match an edge an earlier hop in the same pattern already used;
  reusing the *same* relationship variable twice in one pattern is now
  rejected at compile time, matching real Cypher's
  `RelationshipUniquenessViolation`.
- A relationship variable carried across a `WITH` boundary
  (`WITH r1 AS r2 MATCH ()-[r2]->()`) is no longer silently rebound and
  cross-joined against every relationship in the graph.
- A node variable repeating *within* one pattern (`MATCH (n)-[r]->(n)`)
  is now recognized as a repeat, not just across a `WITH` boundary.
- Integer equality no longer loses precision past 2^53 from an `f64` cast.
- Non-aggregating `RETURN ... ORDER BY` can now reference a variable in
  scope but not returned; aggregating `ORDER BY` can now reference a
  returned-but-unaliased expression verbatim.
- Negative `LIMIT` is now rejected (real Cypher: compile-time error)
  instead of silently clamping to zero rows.
- `DELETE` on a null binding (from a non-matching `OPTIONAL MATCH`) is now
  a no-op instead of an error, per spec.
- `UNWIND null AS x` now yields zero rows instead of treating `null` as an
  unbound variable name.
- `Expand`/`VarExpand` from a null starting node (padded by an earlier
  `OPTIONAL MATCH`) now yields zero rows for that branch instead of
  erroring.

### Known gaps (documented in README, not fixed this release)
- No compile-time semantic validation — an undefined variable/function or
  a wrong-type function argument is only caught while evaluating an actual
  row, so a query whose `MATCH` matches zero rows never gets checked.
- `SET`/`DELETE`/`REMOVE` can't be followed by `RETURN` in the same
  statement.

## [0.4.0] - 2026-08-03

### Added
- `MERGE <pattern> [ON CREATE SET ...] [ON MATCH SET ...]` — match-or-
  create, capped at one relationship hop. The search phase reuses the
  same planner/executor path an ordinary `MATCH` uses, so a one-hop
  pattern correctly searches the *connected* sub-pattern rather than
  each node independently.
- `UNWIND <list> AS x [WHERE ...]` — fans a list out into one row per
  element. `<list>` is an inline Cypher-text list literal or a variable
  bound by a preceding `WITH ... collect(...)`; collected nodes/edges
  restore real graph identity on the way back out, so a `MATCH` after
  the `UNWIND` can keep traversing.
- Named-path capture (`MATCH p = (a)-[:KNOWS]->(b) RETURN p`, fixed-hop
  patterns only) and `shortestPath((a)-[:TYPE*..N]-(b))` (a real
  shortest-path BFS between two already-matched endpoints, not just the
  first path found), plus `length(p)`.
- Backslash-escaped string literals (`\' \" \\ \n \r \t \b \f`) — fixes
  a real bug, not just a missing feature: an unescaped `'` inside a
  string used to silently mis-terminate the literal instead of erroring.

## [0.3.1] - 2026-08-03

### Fixed
- Every crate's `Cargo.toml` and `marsdb-python`'s `pyproject.toml` was
  missing a `readme` field, so crates.io/PyPI showed "no README" for
  0.3.0 despite one existing at the repo root. Package metadata only —
  no code changes.

## [0.3.0] - 2026-08-03

### Added
- Cypher aggregation: `count()`/`count(*)`/`sum()`/`avg()`/`min()`/`max()`/
  `collect()`, `DISTINCT`, and implicit `GROUP BY` (every non-aggregating
  item in an aggregating `RETURN`/`WITH` becomes a grouping key — real
  Cypher has no `GROUP BY` keyword).
- `WITH ... WHERE` — Cypher's HAVING-equivalent, filtering on an already-
  projected/aggregated row (e.g. `WITH p, count(f) AS c WHERE c > 10 ...`).
- `Database::execute_batch()` and the CLI's one-shot arg now accept a
  `;`-separated batch of statements, one transaction per statement.
- Concurrent reads: `MATCH ... RETURN` now opens a redb `ReadTransaction`
  instead of a `WriteTransaction`, so concurrent readers run in parallel
  instead of queueing behind the single-writer lock. Writes are still
  exclusive (unchanged).
- Linux x86_64 (manylinux) wheel on PyPI, alongside the existing macOS
  arm64/x86_64 wheels.
- `marsdb-graph`: secondary index on node label (`NODE_LABEL_INDEX`),
  speeding up label-filtered scans at the cost of slightly slower writes
  and full-table scans — see BENCHMARKS.md for the measured trade-off.

### Changed
- Aggregation's grouping-key lookup and `DISTINCT`'s dedup set now use a
  hash-based lookup (`HashKey`) instead of a linear scan — real
  measured improvement on BENCHMARKS.md: the worst case (every row its own
  group) went from 141 ms to 33.7 ms at 10,000 rows.
- `ReturnExpr::Call` is now a struct variant (`{ name, args, distinct }`)
  instead of a tuple — affects any code constructing/matching it directly
  (not a concern for `Database`/CLI/Python users, only for code embedding
  `marsdb-query` directly).

### Fixed
- `CASE`-WHEN comparing two node/edge-valued expressions now compares by
  identity instead of always falling through to "not equal" (pre-existing
  bug, unrelated to any single release — fixed alongside `DISTINCT`, which
  needed the same identity-equality logic anyway).
- `marsdb-python`'s build broke silently after `$parameter` support added
  `Literal::Param` (the Python bindings are excluded from the Cargo
  workspace, so `cargo test --workspace` never caught it) — fixed, and the
  release process now explicitly checks `marsdb-python` on its own before
  any release ships.

## [0.2.0] - 2026-08-02

### Added
- Multi-label nodes (`(n:Post:Message)`), `$parameter` support,
  `coalesce()`/`toInteger()`, multi-key `ORDER BY`, undirected
  (`-[r:TYPE]-`) and variable-length (`[:TYPE*min..max]`) relationship
  patterns, `WITH`-chaining (one boundary per statement), `OPTIONAL MATCH`.
- Verified against all 7 of LDBC SNB Interactive's short-read reference
  queries (IS1-IS7).
- Criterion benchmarks for the storage and Cypher layers (`BENCHMARKS.md`).

### Fixed
- Non-atomic multi-op statements: `CREATE (a)-[:R]->(b)` previously ran as
  3 separate transactions, not 1 — violated the crash-safety guarantee
  that one Cypher statement is one commit. Fixed via `*_in_txn` method
  variants and a single `WriteTransaction` per statement.

## [0.1.0] - 2026-08-02

Initial release: `CREATE`/`MATCH`/`DELETE`/`DETACH DELETE`/`SET`, `WHERE`,
`LIMIT`, single-linear-pattern `MATCH`, comma-separated `CREATE` patterns.
Rust library, CLI (REPL + one-shot), Python bindings.
