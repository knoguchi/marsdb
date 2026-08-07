# Changelog

All notable changes to MarsDB are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- `Database::execute_batch_grouped(cypher, group_size)`: commits once
  every `group_size` statements instead of once per statement like
  `execute_batch` does. Every commit fsyncs, so bulk loads were
  fsync-bound; on a real 9,771-statement load script this cut load time
  from 69.1s to 13.4s at `group_size: 100` (measured). Trades
  crash-safety granularity for throughput — a failure or crash rolls
  back the whole group it's in, not just the failing statement.
- `$param` values now support the temporal `PropertyValue` variants
  (`Date`, `Duration`, `LocalTime`, `Time`, `LocalDateTime`, `DateTime`)
  in ordinary expression position (`RETURN $x`, `UNWIND $x`, `SET n +=
  $x`, nested inside a list/map param), instead of erroring with
  "passing a temporal value as a query parameter isn't supported yet".
  Found profiling a bulk-load script: parsing a literal-Cypher dump
  dominated load time (~70%, flamegraph-confirmed), and binding the
  data as `$rows` params instead of literal text — the standard "parse
  once, bind many" fix — needs this to work for any dataset with dated
  properties. On the same 9,771-statement script, parameterized replay
  through `execute_batch_grouped` took 5.25s, down from 12.1s for
  group-commit alone and 69.1s for the original per-statement path.
- `marsdb_query::split_statements`: made public (was already used
  internally by `parse_many`) — quote/backtick-aware `;`-splitting of
  raw Cypher text without building a parse tree, useful for tooling that
  needs statement boundaries in source text, not just parsed ASTs.

### Fixed
- `backup_to`/`StorageEngine::open_*` were missing four tables (`prop_to_id`,
  `id_to_prop`, `index_defs`, `property_index`) from the set they eagerly
  create/copy. Node and relationship data survived a backup intact, but
  every declared index — including unique constraints — silently
  disappeared from the restored database; the planner fell back to full
  scans and duplicate values that should have been rejected were
  silently accepted.

## [0.7.1] - 2026-08-07

### Added
- `marsdb --nl "<question>"` translates a plain-English question into
  Cypher via a local [Ollama](https://ollama.com) instance and runs it
  (read-only) — wires the existing `marsdb-nl2cypher` crate into the CLI.
- The CLI now reads a `;`-separated batch from stdin when it isn't a
  terminal (`marsdb mydata.db < script.cypher`), avoiding the OS's
  single-argument length cap (`ARG_MAX`, ~1MB on macOS) that a large
  script passed as the `QUERY` argument would otherwise hit.
- Every built-in function's argument count is now checked before
  evaluation, producing a clear arity error instead of a confusing
  type-mismatch or `None`-related failure when called with too few or
  too many arguments.

### Fixed
- `END` is now usable as a bound variable name (`MATCH (start)-[r]->(end)
  RETURN end`) — previously rejected outright, even though real Cypher
  allows it and it's the standard variable-naming convention for a
  relationship's two endpoints in generated/exported Cypher (e.g. Neo4j's
  own APOC export).
- `IndexSeek` now fires for a `$param`-bound or `UNWIND`-row-bound
  equality against an indexed property, not just a literal constant —
  previously silently fell back to a full label scan repeated once per
  incoming row, the exact shape any bulk-import script
  (`UNWIND rows AS row MATCH (n:Label {id: row.id}) ...`) uses.
- `IndexSeek` now also fires for a `WHERE` clause on a multi-hop
  pattern's start node (`MATCH (a)-->() WHERE a.prop = 'x'`) — previously
  only the inline-property form (`MATCH (a {prop: 'x'})-->()`) reached
  the index; the `WHERE`-clause form's filter sat above the whole
  traversal instead of directly on the scan it should narrow, an
  unindexed full label scan even with a matching index declared.
- Parsing a large multi-statement batch in one call (`parse_many`, used
  by `execute_batch` and the CLI's stdin path) no longer holds every
  statement's parse tree in memory at once — peak memory now scales with
  the largest single statement, not the whole batch. A 29MB/9,771-statement
  script dropped from ~13GB to ~600MB peak RSS.
- `marsdb-nl2cypher`'s generated-Cypher extraction now strips a stray
  trailing markdown code fence even when the model's response has no
  matching opening fence (some models emit one without the other).

Found via a real ~29k-node/166k-relationship dataset load/query
benchmark — see BENCHMARKS.md's "Full lifecycle comparison" section and
[marsdb-demo](https://github.com/knoguchi/marsdb-demo)'s
`benchmarks/recommendations`.

## [0.7.0] - 2026-08-07

### Breaking
- **On-disk database files created by any MarsDB release through v0.6.0
  can no longer be opened.** redb 3.0 dropped support for its own v2 file
  format (what every prior MarsDB release wrote, since none of them opted
  into the v3 format redb 2.6 already supported); redb 4.x only reads v3+.
  No migration path is provided — recreate the database. Pre-1.0, no
  known production data depends on this.

### Changed
- `redb` (the storage engine) bumped 2.x -> 4.1.0, plus `thiserror` 1->2,
  `rustyline` 14->18, `criterion` 0.5->0.8, `ureq` 2->3, and the pinned
  GitHub Actions (`checkout`, `setup-python`, `setup-go`, `codecov-action`,
  etc.) to their current major versions.
- crates.io metadata (`keywords`, `categories`, `documentation` pointing at
  the mdBook manual) added to every published crate.
- MSRV declared and enforced: `rust-version = "1.82"`, verified against
  clippy's own `incompatible_msrv` lint (the actual stdlib APIs used
  workspace-wide), not guessed from syntax alone.
- Dependabot enabled (weekly, grouped) for both `Cargo.lock`s (root
  workspace + `marsdb-python`'s own) and GitHub Actions. GitHub
  Discussions enabled.
- `marsdb-query/grammar/README.md` and `CYPHER_COVERAGE.md` rewritten:
  cut from dense, narrative prose (some referencing internal issue IDs
  with no external meaning) down to scannable, self-contained reference
  docs.

### Fixed
- README's Go install instructions were stale (claimed no `go get`-able
  module path); `go get github.com/knoguchi/marsdb/marsdb-go` already
  resolves via the public Git host, verified directly.
- OSS project hygiene: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
  `SECURITY.md`, issue/PR templates, `CODEOWNERS` added at the repo root
  (previously only in the manual, which GitHub doesn't look at for these).

## [0.6.0] - 2026-08-06

### Added
- `marsdb-capi`, a workspace-built C ABI with opaque database handles and
  JSON query results, plus `marsdb-go`, a cgo binding with in-memory and
  persistent databases, exact 64-bit integer decoding, temporal values,
  synchronized close/query access, examples, tests, and CI coverage.
- Parser replaced: ANTLR4-generated grammar (`marsdb-query/grammar/`)
  instead of the hand-rolled `pest` grammar — closer to real openCypher
  grammar coverage, the foundation for everything else in this release.
- List-valued node/edge properties, and `$parameters` naming a list
  (including nested lists) or a map (`PropertyValue::Map`,
  parameter-passing only).
- `SET n = {...}` / `SET n += {...}` map assignment; `MERGE`/`CREATE`/
  `DELETE`/`SET`/`REMOVE` can now chain directly into each other (via
  `WITH` or a trailing `RETURN`) instead of only ever being one mutating
  clause immediately before a single terminal `RETURN`; `ON MATCH`/
  `ON CREATE` accepted in either order in `MERGE`.
- Variable-length relationship patterns: binding `[r*1..3]` to a real
  list-of-relationships variable, matching a variable-length pattern
  against an *already-bound* list (`MATCH (a)-[rs*]->(b)` after
  `WITH [r1, r2] AS rs`), and inline properties on the hop itself
  (`[:TYPE* {prop: val}]`, checked against every hop in the traversal).
- Named-path capture over patterns mixing a variable-length hop with
  other hops, and over a hop that has both named-path capture *and* its
  own real relationship-list variable at once.
- `exists { MATCH ... RETURN ... }`, the full nested-subquery form of
  existential subqueries (alongside the already-supported simple
  pattern-only form) — its own aggregation/`WITH`/nested `exists {}` all
  allowed, correlated against the enclosing row.
- `CALL proc(args) [YIELD ...]` (standalone, implicit-argument, and
  in-query forms) against a pluggable `ProcedureProvider` supplied via
  `ExecutionOptions::procedures` — MarsDB itself ships no built-in
  procedures.
- A real manual: [knoguchi.github.io/marsdb](https://knoguchi.github.io/marsdb/)
  (mdBook, deployed via GitHub Pages), README badges (CI, crates.io,
  docs.rs, Codecov, license, openCypher TCK conformance), and measured
  test coverage in CI (`cargo-llvm-cov` + Codecov).

### Fixed
- Integer arithmetic overflow now returns a query error instead of panicking;
  unexpected engine panics are also contained at the C ABI boundary rather
  than unwinding into or aborting a foreign-language host.
- `duration.between()`/`duration.inSeconds()` component-accessor math, and
  combined ISO-8601 date-time duration string parsing.
- The `null` literal now types as `Kind::Unknown` instead of `Kind::Scalar`,
  fixing several compile-time-rejected cases that should have deferred to
  runtime (e.g. `WITH null AS a MATCH (a)-->()`).
- A variable-length hop's own traversed edges weren't excluded from
  *later* hops of the same pattern (only the reverse direction was
  handled), causing a silent double-count in some named-path-capture
  patterns.
- List-concat (`+`) type inference hardcoded the result's element kind to
  `Scalar` regardless of the operands' real element kinds, wrongly
  rejecting a later `CREATE` off a concatenated node/relationship list.
- `marsdb-go`'s list-valued property JSON round-trip.
- `marsdb-python`'s `PropertyValue` conversion was missing a match arm
  for the newly-added `Map` variant (silently broke that binding's CI
  job for a while — see the new `cargo check`-only CI step added
  alongside the fix, which catches this class of bug without needing
  marsdb-python's full maturin/link pipeline).

### openCypher TCK conformance
3878/3880 scenarios pass (99.9%), 0 wrong-result scenarios — up from
94.4% right after the ANTLR migration landed. The only 2 non-passing
scenarios need dates at year ±999,999,999, a real storage/library range
limitation (`PropertyValue::Date`'s `i32` epoch-day width, and
`chrono::NaiveDate`'s own ~262,000-year internal cap), not a bug — see
`CYPHER_COVERAGE.md`.

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
