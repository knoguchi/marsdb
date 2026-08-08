# Cypher coverage

## TCK conformance

Measured against the vendored openCypher TCK (`marsdb-tck/`, 220 feature
files, 3880 scenarios):

```
cargo run -p marsdb-tck --release
```

To run one category/feature only, filter by path substring:

```
TCK_FILTER="clauses/create" cargo run -p marsdb-tck --release
```

The TCK runner checks results and, for error scenarios, that MarsDB
errored *at all* (not the specific error kind). See
`marsdb-tck/src/main.rs` for the full scope.

| category | total | pass | wrong | unexp | reject | unsup | pass % |
|---|---|---|---|---|---|---|---|
| clauses/call | 52 | 52 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/create | 78 | 78 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/delete | 41 | 41 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/match | 381 | 381 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/match-where | 34 | 34 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/merge | 75 | 75 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/remove | 33 | 33 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/return | 63 | 63 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/return-orderby | 35 | 35 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/return-skip-limit | 31 | 31 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/set | 53 | 53 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/union | 12 | 12 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/unwind | 14 | 14 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with | 29 | 29 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with-orderBy | 292 | 292 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with-skip-limit | 9 | 9 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with-where | 19 | 19 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/aggregation | 35 | 35 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/boolean | 150 | 150 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/comparison | 72 | 72 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/conditional | 13 | 13 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/existentialSubqueries | 10 | 10 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/graph | 61 | 61 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/list | 185 | 185 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/literals | 131 | 131 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/map | 44 | 44 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/mathematical | 6 | 6 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/null | 44 | 44 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/path | 7 | 7 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/pattern | 50 | 50 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/precedence | 104 | 104 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/quantifier | 604 | 604 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/string | 32 | 32 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/temporal | 1004 | 1004 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/typeConversion | 47 | 47 | 0 | 0 | 0 | 0 | 100.0% |
| useCases/countingSubgraphMatches | 11 | 11 | 0 | 0 | 0 | 0 | 100.0% |
| useCases/triadicSelection | 19 | 19 | 0 | 0 | 0 | 0 | 100.0% |
| **TOTAL** | **3880** | **3880** | **0** | **0** | **0** | **0** | **100.0%** |

Parser: ANTLR4-generated (`marsdb-query/grammar/`, see that directory's
own README). Replaced an earlier hand-rolled `pest` grammar; see git
history for that version of this table.

Columns:
- **pass** — correct result (or, for an error-expecting scenario, any error).
- **wrong** — ran and returned the wrong rows. The only column that means
  a real bug, not a gap. Currently 0.
- **unexp** — errored when success was expected, or vice versa.
- **reject** — MarsDB's grammar/semantic pass rejected the query before
  execution. The largest bucket, and mostly expected for a subset
  implementation, not a bug.
- **unsup** — the TCK scenario itself uses a fixture/step shape the test
  runner doesn't parse.

## Clauses

**Nodes/patterns**
- Multi-label nodes: `(n:Post:Message)`
- Relationship patterns: directed (`-[r:TYPE]->`), undirected (`-[r]-`),
  bracketless (`-->`/`--`), multi-type (`[:A|B]`), variable-length
  (`[:TYPE*min..max]`)
- `CREATE`/`MERGE` require exactly one explicit relationship type per hop
  (matches real Cypher — an edge being created can't be ambiguous about
  its type)
- Named-path capture: `MATCH p = (a)-[:KNOWS]->(b) RETURN p`
  (fixed-hop only)
- `shortestPath((a)-[:TYPE*..N]-(b))` — real BFS shortest path, not just
  first-found
- `length(p)` on a captured path

**WHERE**
- String predicates: `STARTS WITH`/`ENDS WITH`/`CONTAINS`
- Label predicate as a general expression, not just in `WHERE`:
  `RETURN a:Label AS result`
- Property-to-property comparison: `WHERE a.id = b.id`
- Node/relationship identity: `WHERE a = b` (only `=`/`<>` — no ordering
  between nodes/relationships)
- Pattern predicate as a boolean expression: `WHERE (n)-[:REL]->()`,
  correlated against already-bound variables
- Pattern comprehension: `[(n)-[:T]->(b) | b.name]` — can introduce new
  variables, unlike a pattern predicate
- `exists { (n)-->(m) WHERE ... }` (simple form: pattern + optional inline
  `WHERE`)
- `exists { MATCH ... RETURN ... }` (full form: arbitrary read-only nested
  statement, own `WITH`/aggregation/nested `exists{}` allowed; an updating
  clause inside is a compile-time error)

**WITH / chaining**
- Any number of chained `WITH` boundaries per statement
- `WITH *` carries every bound variable forward unchanged
- `MATCH ... CREATE` reuses an already-bound node instead of creating a
  new one
- `CREATE`/`SET`/`DELETE`/`DETACH DELETE`/`REMOVE` can each be followed by
  a trailing `RETURN`, or continue via `WITH` into further clauses
- Two independent `MATCH` parts across a `WITH` boundary correctly
  cross-join
- `UNWIND <list> AS x` — `<list>` may be a literal, a bound variable, a
  list-valued property, or a `$param` (including nested lists)
- `$param` may be map-valued (`{name: 'A'}`, nested maps/lists too) —
  parameter-passing only, never stored as an indexed property. A
  node/relationship-valued `$param` is not supported.

**Mutation**
- `SET`/`REMOVE` cover both properties and labels; `SET n.prop = null`
  removes the property (doesn't store a literal null)
- `MERGE <pattern> [ON CREATE SET ...] [ON MATCH SET ...]` — capped at one
  relationship hop; `MERGE (n)` with no constraints is valid
- `CALL proc(args) [YIELD ...]` resolves against a caller-supplied
  `marsdb_query::ProcedureProvider` — MarsDB ships no built-in procedures
  itself

**Aggregation / ordering**
- `count()`/`count(*)`/`sum()`/`avg()`/`min()`/`max()`/`collect()`/
  `percentileCont()`/`percentileDisc()`, with `DISTINCT` inside the call —
  including grouping by list/map values structurally, not just scalars
- `RETURN DISTINCT` dedups the whole projected row, after grouping
- Multi-key `ORDER BY`, `SKIP`/`LIMIT` on both `RETURN` and `WITH` (`SKIP`
  always applies after `ORDER BY`, regardless of clause order in text)
- `CASE`

**Built-in functions**
- `coalesce()`, `toInteger()`/`toFloat()`/`toString()`/`toBoolean()`
- `keys()`/`labels()`/`type()`/`properties()`/`id()`
- `size()` (list length or string length), `length()` (path edge count)
- `nodes()`/`relationships()` (path elements)
- `head()`/`tail()`/`last()`/`range(start, end[, step])` (both-ends
  inclusive, unlike Rust's range)
- `exists()` — property presence only. `exists((n)-->())` as a function
  call is not real Cypher syntax; use `exists { (n)-->() }` instead.
- String: `toUpper()`/`toLower()`/`trim()`/`ltrim()`/`rtrim()`/`reverse()`/
  `replace()`/`split()`/`substring()`/`left()`/`right()`
- Math: `abs()`/`ceil()`/`floor()`/`round()`/`sqrt()`/`sign()`
- `date()`/`duration()` and friends — see [Temporal types](#temporal-types)

## Expressions

- Arithmetic `+ - * / %` plus `^` (exponentiation, left-associative: `4 ^
  3 ^ 2` is `(4 ^ 3) ^ 2`, always produces a float). `+` also concatenates
  strings and appends/prepends/concatenates lists.
- An aggregate can combine with other expressions only if every
  non-aggregate value is also an explicit grouping key (`RETURN me.age,
  count(you.age) + 3` is fine; `RETURN me.age + count(you.age)` alone is a
  compile error — real Cypher's `AmbiguousAggregationExpression`).
- List literals, indexing (`list[0]`, negative indices, out-of-bounds is
  `null`), slicing (`list[1..3]`, open-ended — out-of-range bounds clamp
  instead of producing `null`)
- List comprehensions: `[x IN list WHERE cond | expr]` (`WHERE` and
  `| expr` both optional)
- Quantifiers `ALL`/`ANY`/`NONE`/`SINGLE(x IN list WHERE cond)` with real
  three-valued NULL logic (a definite true/false decides the answer even
  with `null` elements present)
- Map literals (`{a: 1, b: 2}`), property-style access on a map variable
  (`m.a`) — general postfix access on a non-identifier expression
  (`list[0].prop`) isn't supported, only `identifier.prop`
- Boolean logic (`AND`/`OR`/`XOR`/`NOT`) and comparisons are first-class
  `RETURN`/`WITH` expressions, not just `WHERE`-only syntax
- Three-valued NULL logic throughout (`false AND null` is `false`).
  `=`/`<>` are always definite even across types (`1 = 'a'` is `false`);
  ordering and string predicates return `null` on a type mismatch instead.
- List/map equality is structural (`[1,2] = [1,2]` is `true`); lists order
  lexicographically (`[1,0] >= [1]` is `true`)
- Chained comparisons: `1 < x < 10` folds into `(1 < x) AND (x < 10)`
- `IS NULL`/`IS NOT NULL`
- A list-valued node/edge property (`CREATE (n {tags: [1,2,3]})`)
  round-trips as a real list — indexing, `size()`, `IN`, `UNWIND` all work
  on it. A map-valued property is a real error (matches real Cypher: only
  scalars or homogeneous scalar arrays are storable).
- Grouping/`DISTINCT` by a map value works (hashed by sorted entries, so
  key order doesn't matter)

## Temporal types

All six Cypher temporal types are supported as first-class storage
variants (not `Int`/`String` reused): `Date`, `Duration`, `LocalTime`,
`Time`, `LocalDateTime`, `DateTime`.

- `DateTime` accepts a fixed UTC offset (`'+01:00'`) or a named IANA
  timezone (`'Europe/Stockholm'`), resolved via `chrono-tz` on demand
  (correct across DST transitions, since a `Named` zone's offset isn't
  cached). `Time` only accepts a fixed offset — it carries no calendar
  date, so there's nothing to resolve a named zone's DST-dependent offset
  against.
- Construction: no-arg (current UTC value), from a string (calendar date,
  ISO week-date, ordinal-date), from a map (calendar/week-date/
  ordinal-date/quarter-date forms), from another value of the same type,
  or projected from a different temporal type (`date(existingDateTime)`).
  `time()`/`datetime()` from a string with no offset default to UTC.
- `duration({years, months, weeks, days, hours, minutes, seconds, ...})`
  and `duration('P1Y2M3DT4H5M6S')` — real ISO-8601 normalization,
  including fractional units.
- Comparison: `Time`/`DateTime` compare by UTC-equivalent instant, not raw
  wall-clock reading. `=`/`<>` work on `Duration` (component equality); no
  ordering is defined for `Duration`.
- Arithmetic: `+`/`-` a `Duration` to/from any of the other five types.
  `Date`/`LocalDateTime`/`DateTime` use real calendar month math (Jan 31 +
  1 month clamps to Feb 28/29); `Time`/`LocalTime` wrap at 24h instead.
- Component access: `.year`/`.month`/`.day`/`.quarter`/`.ordinalDay`/
  `.weekDay`/`.week`/`.weekYear`/`.dayOfQuarter` (calendar types);
  `.hour`/`.minute`/`.second`/`.millisecond`/`.microsecond`/`.nanosecond`
  (clock types); `.timezone`/`.offset`/`.offsetSeconds`/`.offsetMinutes`
  (`Time`/`DateTime`); `.epochSeconds`/`.epochMillis` (`DateTime` only,
  always UTC). `Duration` exposes each unit as the whole duration
  re-expressed in it (`.months`) and each remainder-within-the-next-unit
  (`.monthsOfYear`).
- `toString()` round-trips for every type.
- Field projection between temporal values via a map key (`date({date: d,
  day: 5})`): named-key fields become defaults, other explicit keys
  override. Changing `timezone` on a value that already carries an offset
  shifts the wall-clock to preserve the same instant.
- `duration.between(a, b)` / `.inMonths` / `.inDays` / `.inSeconds` — real
  calendar-aware duration between any two of the 5 non-`Duration` types.
  `.inDays`/`.inSeconds` use raw elapsed time (no month optimization).
  Every no-arg temporal constructor call within one query shares a single
  captured instant.
- `<type>.truncate(unit, value, map?)` — rounds down to the start of
  `unit`, then applies optional field overrides on top.

**Not supported:**
- Named timezones for `Time` (only `DateTime`)
- Alternate ISO-8601 combined date-time duration syntax
  (`duration('P2012-02-02T14:37:21.545')`)
- (Dates now cover Cypher's full `±999,999,999`-year range: epoch days
  are `i64` and the calendar math is hand-rolled proleptic-Gregorian
  integer arithmetic — `chrono` remains only for `now()` capture and
  named-IANA-zone resolution, which is inherently bounded by the IANA
  database's own applicability.)

See `marsdb-query/src/temporal.rs`'s module doc comment for the same list
in code.

## Indexes & EXPLAIN

- `CREATE INDEX ON :Label(prop)` (optionally `UNIQUE`) — backfilled
  immediately from existing nodes
- An equality against an indexed `(label, prop)` compiles to an index
  seek instead of a label scan + filter, including one conjunct inside a
  larger `WHERE a = 1 AND b = 2` (only the indexed side seeks; the rest
  survives as a residual filter)
- When several conjuncts each have an index, the planner picks the most
  selective by exact cardinality (a redb per-key entry count), not
  whichever appears first
- `EXPLAIN <statement>` prints the compiled plan without running it,
  always against a read-only snapshot

Not yet supported: `CREATE CONSTRAINT`, composite indexes, range scans.

## Transactions

`BEGIN` / `COMMIT` / `ROLLBACK` as statements — a MarsDB extension
(openCypher has no transaction statements; real deployments do this at
the protocol/session layer). One session per `Database` handle: `BEGIN`
opens a write transaction, every subsequent statement (reads included —
they see the transaction's own uncommitted writes) runs inside it, and
`COMMIT`/`ROLLBACK` end it. A statement that fails at *execution* time
aborts the whole transaction (its partial effects must never be
committable); a parse/`$param` error leaves it open (nothing ran).
Works identically fed one statement at a time (the CLI REPL) or inside
one `;`-separated batch: `BEGIN; CREATE (a); CREATE (b); COMMIT` is one
atomic unit. Not valid inside a caller-owned `Database::begin_transaction`
handle (that API has its own `commit()`/`rollback()` methods) or
`execute_batch_grouped` (whose grouping is its own transaction policy).

## Error taxonomy

`QueryError` is typed, not one flat string:
- `Syntax` — the query text never parsed
- `Semantic` — parsed, but structurally invalid (unsupported pattern
  shape, misplaced aggregate) — knowable from the query text alone
- `Type` — only knowable once real data or a `$parameter` turns out to be
  the wrong shape (arithmetic on a non-number, indexing a non-list)
- `Graph` / `UnboundVariable` / `MissingParam` / `Cancelled` / `Timeout` /
  `ResourceLimit`

`ExecutionOutcome` (the `ExecutionObserver` telemetry enum) mirrors the
same split.

## Verified against

All 7 of LDBC SNB Interactive's short-read reference queries (IS1-IS7) —
see `marsdb-query/tests/ldbc_is_queries.rs`.

Not verified:
- LDBC's complex queries (IC1-14) beyond one hand-crafted
  grouping+`WITH...WHERE`+`ORDER BY`+`LIMIT`+`collect()` check (see
  `marsdb-query/tests/smoke.rs`)
- Comma-separated patterns *within one* `MATCH`/`CREATE` clause beyond a
  single linear chain (general cross-joins — different from the
  `WITH`-chaining cross-join above, which works)
- `MERGE` patterns with more than one relationship hop (no
  whole-pattern atomicity across multiple simultaneously-unbound hops)
- `shortestPath()` with a minimum hop count greater than 1 (plain
  visited-set BFS can't answer "shortest path of at least N hops" for
  N > 1)
