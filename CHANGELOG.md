# Changelog

All notable changes to MarsDB are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
