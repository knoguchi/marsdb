# Changelog

All notable changes to MarsDB are documented here. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.9.0] - 2026-08-10

The release that declares the C API stable. Everything a non-Rust
binding needs — typed values, parameters, streaming, execution bounds,
transactions, Arrow results — now crosses one settled ABI surface
(`marsdb-capi/marsdb.h`, the documentation of record), exercised
end-to-end by the out-of-repo Go binding's CI on every run.

### Changed
- **C API v2 (breaking)**: typed opaque handles (the SQLite shape) plus
  a binary batch lane — one FFI crossing returns a whole result as a
  compact self-describing buffer (interned names, varint ints, format
  spec in `marsdb.h`). The JSON result channel is removed, not
  deprecated: every consumer moved to the typed surface.
- **Go bindings moved out** to
  [knoguchi/marsdb-go](https://github.com/knoguchi/marsdb-go) — one
  repo, two Go modules (`marsdb-go` core with zero deps,
  `marsdb-go/arrow` for columnar results). They consume the C ABI via
  cgo against a vendored `marsdb.h`; that repo's CI builds this repo's
  `marsdb-capi` from `main` and drift-checks the header, and this
  repo's CI keeps `--features arrow` compiling as the reciprocal
  pre-merge guard.

### Added
- **Arrow results, zero-copy at every boundary** (`arrow` cargo
  feature, off by default): `Database::query_arrow` transposes a result
  to Arrow `RecordBatch`es once, in core; downstream it's pointer
  handoff only — a Rust `RecordBatchReader`, the Arrow C Data Interface
  stream through `marsdb-capi` (`marsdb_query_arrow` /
  `marsdb_stmt_execute_arrow`), the PyCapsule protocol in Python, and
  arrow-go `cdata` import in Go. Column typing is strict per column
  over the whole result (`Int64` exact, no silent int→float promotion;
  node/edge/map/path columns error — project properties instead).
  Measured on a 200k-row 3-column result in Go: ~1.2M binding-side
  allocations (83 MB) via the batch lane vs ~900 (83 KB) via Arrow,
  wall time at parity (engine dominates both).
- **Streaming reads**: `Database::execute_streaming` pushes rows into a
  caller-supplied `RowSink` — bounded memory regardless of result size,
  exposed in Rust, Python, the C ABI (per-row callback), and Go.
  Accepts exactly the streamable shape (one plain `MATCH ... RETURN`,
  `SKIP`/`LIMIT` fine) and errors on `ORDER BY`/aggregation/`DISTINCT`/
  `WITH` rather than silently materializing.
- **Per-statement write counters**: `QueryResult::stats`
  (nodes/relationships created and deleted, properties set, labels
  added/removed) — the answer to "how many did my DELETE delete",
  surfaced through all bindings and the CLI.
- **Parameterized queries across the C ABI, Go, and Python**: typed
  prepared-statement binds (`$name`), scalars and flat lists; `int64`
  precision preserved end to end.
- **Execution bounds in all bindings**: row limits, relationship-
  expansion limit, timeout, cancellation — checked cooperatively during
  plan evaluation, not after materialization. Python errors are now
  structured exception classes instead of one flat error type.
- **Schema introspection procedures**: `CALL db.labels()`,
  `db.relationshipTypes()`, `db.propertyKeys()`, `db.indexes()`.
- **Planner: bounded index range scans** — `WHERE n.year > 2000 [AND
  n.year < 2010]` over an indexed `(label, prop)` compiles to
  `IndexRangeSeek` over the order-preserving key encoding, with the
  originating conjuncts kept as a residual filter (the storage lookup
  returns a superset for numeric bounds by design).
- **Planner: `EdgeTypeScan`** — relationship-predicate bulk shapes
  (`MATCH ()-[r:T]->() WHERE r.x ... DELETE r`) compile to one
  sequential `EDGES` sweep with the predicate evaluated from each
  record's own bytes, cost-gated against the anchored alternatives.
  Measured: ~5–6 ms for a warm 166k-record sweep vs ~110 ms for the
  same edges through per-edge adjacency gets.
- **Internals book**: a ten-chapter architecture tour
  (design → storage → encoding → write path → frontend → planner →
  executor → boundaries → testing → measured-trade-off case studies)
  in English and Japanese, with Mermaid diagrams, published with the
  manual.

### Fixed
- **wasm32**: generated parser token bitmasks widened to `u64` — the
  grammar compiles and runs on 32-bit targets.

## [0.8.0] - 2026-08-08

### Changed
- **On-disk format version 1 → 2**. Old files are rejected with a
  clear error; migration path is export from a v1 build, reimport here.
  Two format changes, one break:
  - **Directory record format**: node/edge records store properties as a
    sorted `(interned prop-id, offset)` directory with individually
    postcard-encoded values, replacing the whole-blob string-keyed map.
    Property names no longer appear in records. Single-property access
    reads one directory entry (no full-record decode, no name
    resolution); the executor's property lookups use this path, with
    prop names resolved to ids once per statement. Codec mechanism
    measured 79x at 1-of-20 properties touched, 7x even at full
    materialization; end-to-end on the recommendations read suite: 1.16x
    at size parity (see BENCHMARKS.md for why those differ).
  - **Composite-key adjacency**: `adj_out`/`adj_in` are plain tables
    keyed `(owner node, label, edge)` as fixed-width redb tuples, so a
    typed expansion narrows to a key prefix range — O(matching degree)
    instead of decoding a node's entire entry set. Edge deletion becomes
    two exact-key removes. `TableHandle`/`Txn` gain `range()`, which
    also readies `PROPERTY_INDEX` for future range predicates.

### Added
- **openCypher TCK conformance: 3880/3880 (100%)**, up from 3878. The
  last 2 scenarios needed dates at year ±999,999,999 (`Temporal10
  [9]`/`[10]`): `PropertyValue::Date` widened `i32` → `i64` epoch days
  (wire-compatible — postcard varints don't encode width, and the index
  key encoding was already 8-byte), the calendar core rewritten as
  hand-rolled proleptic-Gregorian integer math (chrono's `NaiveDate`
  caps at ±262k years; it remains only for `now()` capture and named
  IANA-zone resolution), and `duration.between` totals moved to i128
  nanoseconds. Parser/formatter round-trip ISO 8601 expanded years
  (`'+999999999-12-31'`); `localdatetime('<date-only>')` reads as
  midnight.
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

### Changed
- Every `GraphStore::*_in_txn` write helper (`create_node_in_txn`,
  `create_edge_in_txn`, `set_node_prop_in_txn`, `create_index_in_txn`, ...)
  now caches its redb table handles for the call instead of reopening the
  same table repeatedly as it works through label/property interning,
  the actual write, and index maintenance. Internal only — no API change.
  Found via a flamegraph: `WriteTransaction::open_table` summed to
  23.21% of replay time on the real 9,771-statement load script,
  the single biggest cost after group commit (#153) and parameterized
  loading (#154). Cut to 16.91%, wall time 4.89s → ~4.2s (measured).
- The executor now caches each node it decodes for the rest of a
  read-only statement instead of re-decoding it from storage on every
  access — `RETURN n.a, n.b ORDER BY n.c` decoded the same node three
  times before. Disabled entirely for write statements (a `SET`/`REMOVE`
  can change a node mid-statement, so a cached copy could go stale) and
  cleared between statements on a reused `Executor` (`execute_batch`,
  group commit). Internal only — no API change. Found via a flamegraph:
  the postcard decode of a node's full stored record summed to ~40% of
  read-path time on a real dataset. On the same dataset's canonical
  multi-hop read queries (100 repeats, for a stable measurement), total
  time dropped from 15.45s to ~9.7s (~37% faster); the two queries with
  the most repeated node reads (`crimson_tide_collaborative_filtering`,
  `inception_genre_similarity`) each dropped from ~65-70ms to ~39ms.

- Aggregating-expansion fast path: a `MATCH` of one or two typed
  `Expand` hops feeding a `WITH <node>, count(*)` (and/or
  `collect(<mid>.prop)`) now runs as a tight counting loop over the
  adjacency tables instead of materializing a binding row per
  intermediate path — the 2-hop count shape measured ~25x end-to-end on
  the recommendations suite, where row machinery (not storage) was ~99%
  of query time. Also covers filtered-scan leaves via direct seed
  enumeration, group-by-origin, and single-key ORDER BY + LIMIT
  pre-truncation. Conservative: any unrecognized shape falls back to the
  generic pipeline.
- Planner start-point selection: a `MATCH` pattern now starts traversal
  from its cheaper endpoint instead of always the written-first one,
  compared by O(1) cardinality (label counts, indexed-equality match
  counts, already-bound Seeds; new `TableHandle::len`/
  `GraphStore::node_count_in_txn`/`label_count_in_txn`). Written from
  the big side toward a small/indexed/carried endpoint, measured on a
  100-of-N selectivity benchmark: 1.17ms → 181µs at 1k nodes, 10.5ms →
  179µs at 10k, 102ms → 183µs at 100k — flat in dataset size, cost
  follows matches. Fixed-hop patterns only; named paths, shortestPath,
  and MERGE keep written order; EXPLAIN shows the executed choice.
- `IndexSeek` now fires for a var-free function-call equality
  (`WHERE n.joined = date('2020-01-10')` — the shape a `$param`-
  substituted temporal lookup takes after parameter substitution),
  instead of falling back to a per-row label scan. `rand()` is
  excluded (must evaluate per candidate row); the temporal
  now-functions are not (pinned to one per-statement snapshot).

### Fixed
- Opening a pre-versioning v1-era file (data tables present, no
  `schema_version` marker in `META`) now fails cleanly as an unsupported
  format-1 file instead of silently stamping it with the current format
  version — which would have made its old-encoding records decode as
  garbage on first access. A genuinely fresh file (no tables at all) is
  still stamped normally; table setup and the version marker commit
  atomically, so no half-initialized state can exist.
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
