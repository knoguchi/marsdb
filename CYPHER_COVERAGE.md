# Cypher coverage

## TCK conformance

Real, measured numbers from the vendored openCypher TCK (`marsdb-tck/`,
220 feature files, 3880 scenarios) — reproduce with:

```
cargo run -p marsdb-tck --release
```

Results + coarse error-checking only (a scenario expecting an error passes
iff MarsDB errored *at all*, not the *right kind* of error; no side-effect
assertions) — see `marsdb-tck/src/main.rs`'s own doc comments for the full
scope.

| category | total | pass | wrong | unexp | reject | unsup | pass % |
|---|---|---|---|---|---|---|---|
| clauses/call | 52 | 16 | 0 | 0 | 36 | 0 | 30.8% |
| clauses/create | 78 | 45 | 0 | 2 | 31 | 0 | 57.7% |
| clauses/delete | 41 | 20 | 0 | 0 | 21 | 0 | 48.8% |
| clauses/match | 381 | 273 | 0 | 0 | 73 | 35 | 71.7% |
| clauses/match-where | 34 | 15 | 0 | 0 | 19 | 0 | 44.1% |
| clauses/merge | 75 | 25 | 0 | 3 | 47 | 0 | 33.3% |
| clauses/remove | 33 | 18 | 0 | 0 | 15 | 0 | 54.5% |
| clauses/return | 63 | 31 | 0 | 2 | 30 | 0 | 49.2% |
| clauses/return-orderby | 35 | 25 | 0 | 2 | 6 | 2 | 71.4% |
| clauses/return-skip-limit | 31 | 25 | 0 | 0 | 6 | 0 | 80.6% |
| clauses/set | 53 | 23 | 0 | 0 | 30 | 0 | 43.4% |
| clauses/union | 12 | 12 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/unwind | 14 | 9 | 0 | 0 | 3 | 2 | 64.3% |
| clauses/with | 29 | 10 | 0 | 1 | 17 | 1 | 34.5% |
| clauses/with-orderBy | 292 | 163 | 0 | 1 | 126 | 2 | 55.8% |
| clauses/with-skip-limit | 9 | 4 | 0 | 0 | 5 | 0 | 44.4% |
| clauses/with-where | 19 | 2 | 0 | 0 | 17 | 0 | 10.5% |
| expressions/aggregation | 35 | 21 | 0 | 4 | 10 | 0 | 60.0% |
| expressions/boolean | 150 | 140 | 0 | 0 | 10 | 0 | 93.3% |
| expressions/comparison | 72 | 54 | 0 | 0 | 18 | 0 | 75.0% |
| expressions/conditional | 13 | 13 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/existentialSubqueries | 10 | 1 | 0 | 0 | 9 | 0 | 10.0% |
| expressions/graph | 61 | 40 | 0 | 7 | 14 | 0 | 65.6% |
| expressions/list | 185 | 121 | 0 | 2 | 55 | 7 | 65.4% |
| expressions/literals | 131 | 71 | 0 | 0 | 51 | 9 | 54.2% |
| expressions/map | 44 | 16 | 0 | 6 | 17 | 5 | 36.4% |
| expressions/mathematical | 6 | 6 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/null | 44 | 37 | 0 | 0 | 1 | 6 | 84.1% |
| expressions/path | 7 | 2 | 0 | 0 | 5 | 0 | 28.6% |
| expressions/pattern | 50 | 19 | 0 | 0 | 31 | 0 | 38.0% |
| expressions/precedence | 104 | 56 | 0 | 0 | 48 | 0 | 53.8% |
| expressions/quantifier | 604 | 532 | 0 | 0 | 72 | 0 | 88.1% |
| expressions/string | 32 | 29 | 0 | 0 | 3 | 0 | 90.6% |
| expressions/temporal | 1004 | 169 | 0 | 356 | 479 | 0 | 16.8% |
| expressions/typeConversion | 47 | 43 | 0 | 1 | 3 | 0 | 91.5% |
| useCases/countingSubgraphMatches | 11 | 11 | 0 | 0 | 0 | 0 | 100.0% |
| useCases/triadicSelection | 19 | 0 | 0 | 0 | 19 | 0 | 0.0% |
| **TOTAL** | **3880** | **2097** | **0** | **387** | **1327** | **69** | **54.0%** |

Column meanings:
- **pass** — matched (or, for an error-expecting scenario, errored at all).
- **wrong** — ran successfully but returned the wrong rows. **The category
  that means a real bug**, not a coverage gap — currently zero everywhere.
- **unexp** — errored when success was expected, or vice versa. As of the
  typed `QueryError` taxonomy, this specifically means MarsDB *ran* the
  query and hit a real, data-dependent type mismatch (not just "never
  parsed this shape") — see [error taxonomy](#error-taxonomy) below.
- **reject** — MarsDB's grammar/semantic pass rejected the query outright
  (never reached execution) — the largest bucket, and mostly an *expected*
  signal for a subset implementation, not a bug.
- **unsup** — the *scenario itself* (not the query) uses a fixture/step
  shape the TCK runner doesn't parse at all (a named graph beyond the two
  vendored, or an untested step form).

`expressions/temporal` is 1004 of the 3880 total scenarios (26%) and pulls
the aggregate average down — `LOCAL TIME`/`TIME`/`LOCAL DATETIME`/
`DATETIME` are supported, but only with a *fixed* UTC offset (`'+01:00'`);
a named timezone (`'Europe/Stockholm'`) needs a real IANA timezone
database, deliberately out of scope (no DST/zone-rule awareness anywhere
in `marsdb-query::temporal`), so the scenarios exercising those still
fail at the `reject` stage. See [Temporal types](#temporal-types) below
for exactly what *is* supported (construction, arithmetic, component
access, comparison, string round-tripping — for all six temporal types,
not just `Date`/`Duration`).

## Clauses

`CREATE`, multi-label nodes (`(n:Post:Message)`), `$parameters`,
backslash-escaped string literals (`\' \" \\ \n \r \t \b \f`),
`MATCH`/`OPTIONAL MATCH`, undirected (`-[r:TYPE]-`), bracketless
(`-->`/`<--`/`--`, the anonymous-untyped-relationship shorthand — brackets
are only needed at all to carry a var/type/range/props), and
variable-length (`[:TYPE*min..max]`) relationship patterns. `CREATE`
requires an explicit relationship type on every hop (`-[:KNOWS]->`,
`-->` alone is rejected with a clear semantic error) — unlike `MATCH`,
where an untyped hop legitimately means "any relationship," a brand new
edge with no type is meaningless, matching real Cypher's own rule.
`WHERE` (including the
string predicates `STARTS WITH`/`ENDS WITH`/`CONTAINS`, a user-typed label
predicate `WHERE n:Label`/`n:Label1:Label2`, a property compared against
*another* property rather than a constant `WHERE a.id = b.id`, and
node/relationship identity comparison `WHERE a = b`/`WHERE a <> b` — only
`=`/`<>` are meaningful for identity, no ordering exists between two
nodes/relationships, so `WHERE a < b` is a real error), one `WITH`
boundary per statement (projection/rename, its own `WHERE`/
`WITH...WHERE`/`ORDER BY`/`LIMIT`), `RETURN`/`DELETE`/`DETACH DELETE`/
`SET`/`REMOVE`/`MATCH ... CREATE` (adds an edge between two
already-matched nodes — a node token whose variable is already bound
reuses that node instead of creating a new one). `SET`/`REMOVE` cover
both properties (`SET n.prop = 'x'`/`REMOVE n.prop`) and labels
(`SET n:Label`/`REMOVE n:Label`); `SET n.prop = null` removes the
property rather than storing a literal null, matching real Cypher.
`DELETE`/`DETACH DELETE`/`SET`/`REMOVE`/`MATCH ... CREATE` can each
optionally be followed by one trailing `RETURN` in the same statement
(`MATCH (n) SET n.prop = 1 RETURN n`, `MATCH (n) DELETE n RETURN
count(n)`) — the shape the real TCK scenarios for these clauses
overwhelmingly use. Not yet supported: chaining more than one mutating
clause before the final `RETURN`, or a `WITH` between the mutating
clause and that `RETURN`.

Multi-key `ORDER BY`, `SKIP`/`LIMIT` (on both `RETURN` and `WITH`; `SKIP`
always applies after `ORDER BY` and before `LIMIT` regardless of clause
order in the query text, matching real Cypher), `CASE`, and implicit-GROUP-BY aggregation
(`count()`/`count(*)`/`sum()`/`avg()`/`min()`/`max()`/`collect()`, with
`DISTINCT` inside an aggregate call). `RETURN DISTINCT` (result-set-level
dedup of the whole projected row, applied after grouping for an
aggregating `RETURN` — a separate mechanism from `DISTINCT` inside one
aggregate call, which only affects that aggregate's own accumulation).

Built-in scalar functions: `coalesce()`, `toInteger()`/`toFloat()`/
`toString()`/`toBoolean()`, `keys()`/`labels()`/`type()`/`properties()`/
`id()` (node/relationship introspection), `size()` (list length or
string character count), `length()` (path edge count), `nodes()`/
`relationships()` (a path's elements), `head()`/`tail()`/`last()`/
`range(start, end[, step])` (list construction/slicing — `range` is
both-ends-inclusive, matching real Cypher, not Rust's exclusive-end
convention), `exists()` (property presence — the pattern-existence form,
`exists((n)-->())`, isn't supported), string functions `toUpper()`/
`toLower()`/`trim()`/`ltrim()`/`rtrim()`/`reverse()`/`replace()`/
`split()`/`substring()`/`left()`/`right()`, and math functions `abs()`/
`ceil()`/`floor()`/`round()`/`sqrt()`/`sign()`. `date()`/`duration()`
are their own thing — see [Temporal types](#temporal-types) below.

Two independent `MATCH` parts across one `WITH` boundary
(`MATCH (a) WITH a MATCH (b) ...`, where `b`'s pattern doesn't chain from
`a`) correctly cross-join, carrying `a` alongside every row `b` produces.
`UNWIND <list> AS x` (fans a list out into one row per element,
cross-joined against existing rows; its own `WHERE` works without needing
a second `WITH`) — `<list>` is an inline Cypher-text list literal
(`[1, 2, 'a', $p]`) or a variable bound by a preceding
`WITH ... collect(...)`; `UNWIND $param` where `$param` itself names a
list isn't supported yet (no list-valued parameters — every `$param` is a
single scalar). `MERGE <pattern> [ON CREATE SET ...] [ON MATCH SET ...]`
(match-or-create: tries the pattern as an ordinary MATCH first, creates
exactly one new instance if nothing matched) — capped at one relationship
hop (`MERGE (n:Label {props})` or `MERGE (a)-[:TYPE]->(b)`); an
unconstrained node pattern that isn't already bound (`MERGE (n)`, no
label or property) is rejected rather than matching/creating arbitrarily.
Named-path capture (`MATCH p = (a)-[:KNOWS]->(b) RETURN p`, fixed-hop
patterns only) and `shortestPath((a)-[:TYPE*..N]-(b))` (real
shortest-path search via BFS, not just the first path found — both
endpoints must already be matched by a preceding clause), plus
`length(p)` to measure one.

## Expressions

Arithmetic (`+ - * / %`, real precedence — `* / %` bind tighter than
`+ -`, explicit `(...)` grouping to override it) in `RETURN`/`WITH`
items, `ORDER BY` keys, and function arguments — `+` also concatenates
two strings; an aggregate can't be nested inside a wrapping arithmetic
expression (`1 + count(x)` is rejected, not silently wrong — `count(x)`
alone as a whole return item is fine); not yet usable inside a `WHERE`
clause's comparison operands, only in `RETURN`/`WITH`/`ORDER BY`.

List literals (`[1, 2, 3+1]`), indexing (`list[0]`, negative indices count
from the end, out-of-bounds is `null`), and slicing (`list[1..3]`,
open-ended `list[2..]`/`list[..3]` — unlike indexing, out-of-range slice
bounds clamp to `[0, len]` instead of producing `null`) in `RETURN`/`WITH`.
List comprehensions (`[x IN list WHERE cond | expr]`, both `WHERE` and
`| expr` independently optional — `[x IN list]` is a legal no-op filter).
Quantifiers `ALL(x IN list WHERE cond)`/`ANY(...)`/`NONE(...)`/
`SINGLE(...)` (same `WHERE` shape as list comprehensions, no `WHERE` means
"every element's own truthiness"), with real three-valued NULL logic — a
definite `true`/`false` among the elements decides the answer even with
other `null` elements present; only "no definite answer, but at least one
`null`" actually yields `null` (e.g. `all(x IN [0, null] WHERE x = 2)` is
`false`, not `null`, since `0 = 2` is a definite `false`).

A leading `WITH` with no preceding `MATCH` (`WITH [1,2,3] AS list RETURN
list[1]`) is also valid on its own. Map literals (`{a: 1, b: 2+1}`) in
`RETURN`/`WITH`, and property-style access on a map-valued variable
(`WITH {a: 1} AS m RETURN m.a`) — general postfix property access on an
arbitrary non-identifier expression (`list[0].prop`) isn't supported yet,
only `identifier.prop`.

Boolean logic (`AND`/`OR`/`XOR`/`NOT`) and comparisons (`=`/`<>`/`<`/`<=`/
`>`/`>=`/`STARTS WITH`/`ENDS WITH`/`CONTAINS`) are first-class `RETURN`/
`WITH` expressions (`RETURN true AND (1 < 2)`), not just a separate
`WHERE`-only grammar — this is also what list comprehensions' and
quantifiers' `WHERE` clauses use, so a bare `WHERE x`/`WHERE true` parses.
Real three-valued NULL logic throughout (`false AND null` is `false`, not
`null` — a definite operand still decides the answer), list/map equality
is real structural comparison (`[1,2] = [1,2]` is `true`, not `null`), and
lists order lexicographically (`[1,0] >= [1]` is `true`). A type-mismatched
comparison's result depends on the operator: `=`/`<>` are always definite
(`1 = 'a'` is `false`, `1 <> 'a'` is `true`), while ordering and the
string predicates are `null` (no defined answer), not `false`. Chained
comparisons (`1 < x < 10`, in `RETURN`/`WITH` — folds into
`(1 < x) AND (x < 10)`, real Cypher's own semantics) and
`IS NULL`/`IS NOT NULL` (`x IS NULL`, both in `RETURN`/`WITH` and in a
pattern's own `WHERE`, e.g. `WHERE n.prop IS NOT NULL`). `WHERE` has real
three-valued NULL logic (`AND`/`OR`/`NOT` and every comparison correctly
propagate "unknown" rather than collapsing it to `false`) — `CASE`'s
`WHEN` and `DISTINCT` dedup deliberately don't, since they need a
definite yes/no, not "unknown".

A general comparison operator (`x > d`, `1 + 1 = 2`) is valid anywhere a
`RETURN`/`WITH` expression is, not just `WHERE` (needed to express
`date1 > date2`, since `WHERE`'s comparison RHS is literal-only) —
including a correctly three-valued (`null`-propagating) `List`/`Map`
equality and lexicographic `List` ordering, and a `NaN`-safe
`<`/`<=`/`>`/`>=` that returns `false` (not `null`) the way real Cypher
does. `{key: <expr>, ...}` map literals are a general expression
(`RETURN {a: 1, b: 2}`, `WITH {x: 1} AS m RETURN m.x`, `map['key']`
dynamic access) usable anywhere a `RETURN`/`WITH` expression is,
including as a `CREATE`/`MERGE` pattern's inline property map value
(`CREATE (n {tags: [1,2,3]})` now parses — though MarsDB's storage layer
still can't persist a list/map-valued property, so that specific example
fails at execution time with a clear error, not silently as `null`).

## Temporal types

All six Cypher temporal types are supported: `Date`, `Duration`,
`LocalTime`, `Time`, `LocalDateTime`, `DateTime` (each a first-class
`PropertyValue` storage variant, not `Int`/`String` reused — see
`marsdb-graph/src/model.rs`'s doc comment). `Time`/`DateTime` only accept
a *fixed* UTC offset (`'+01:00'`, `{timezone: '+01:00'}`) — a named
timezone (`'Europe/Stockholm'`) is rejected with a clear error, not
misparsed; see the gap list below.

Construction: `date()`/`localtime()`/`time()`/`localdatetime()`/
`datetime()` with no arguments (the current UTC value); from a string
(`date('2015-07-21')`, `time('21:40:32.142+01:00')`, ... — calendar-only
date forms, `YYYY-MM-DD`/`YYYYMMDD`/`YYYY-MM`/`YYYYMM`/`YYYY`, no
week-date/ordinal-date forms); from a map (`date({year, month, day})`,
`time({hour, minute, second, millisecond, microsecond, nanosecond,
timezone})`, ...); or from another value of the same type (identity,
e.g. round-tripping through `toString`). `duration({years, months,
weeks, days, hours, minutes, seconds, milliseconds, microseconds,
nanoseconds})`/`duration('P1Y2M3DT4H5M6S')` (real ISO-8601 duration
normalization, verified line-by-line against the TCK's examples,
including fractional units — `duration({months: 0.75})` correctly
becomes `P22DT19H51M49.5S`).

Comparison (`<` `<=` `>` `>=` `=` `<>` on two values of the same type;
`=`/`<>` only on two `Duration`s — component equality, no defined
ordering): `Time`/`DateTime` compare by the UTC-equivalent instant, not
the raw wall-clock reading, so two values at different offsets can
compare equal even though they print differently. Arithmetic (`+`/`-` a
`Duration` to/from any of the other five types, `duration +/- duration`,
`duration * number`, `duration / number`) — `Date`/`LocalDateTime`/
`DateTime` arithmetic uses real calendar month math (Jan 31 + 1 month
clamps to Feb 28/29); `Time`/`LocalTime` + `Duration` wraps at the 24h
boundary instead (no calendar to carry an extra day into), truncating
the duration's `months`/`days` components.

Component access: `d.year`/`.month`/`.day`/`.quarter`/`.ordinalDay`/
`.weekDay`/`.week`/`.weekYear`/`.dayOfQuarter` (`Date`, and the
calendar half of `LocalDateTime`/`DateTime`); `.hour`/`.minute`/
`.second`/`.millisecond`/`.microsecond`/`.nanosecond` (`LocalTime`, and
the clock half of `Time`/`LocalDateTime`/`DateTime`); `.timezone`/
`.offset`/`.offsetSeconds`/`.offsetMinutes` (`Time`/`DateTime` only);
`.epochSeconds`/`.epochMillis` (`DateTime` only, always the UTC instant
— every other `DateTime` component reflects the *local*, offset-adjusted
reading that was written, not the UTC one); the full `Duration` set
(`.years`/`.quarters`/`.months`/`.weeks`/`.days`/`.hours`/`.minutes`/
`.seconds`/`.milliseconds`/`.microseconds`/`.nanoseconds`, each the
*whole* duration re-expressed in that one unit alone, not a
calendar-style breakdown — `duration({years: 1, months:
4}).months` is `16`, not `4` — plus `.quartersOfYear`/
`.monthsOfQuarter`/`.monthsOfYear`/`.daysOfWeek`/`.minutesOfHour`/
`.secondsOfMinute`/`.millisecondsOfSecond`/`.microsecondsOfSecond`/
`.nanosecondsOfSecond`, each unit's remainder within the next one up).
`toString()` round-trips for every type (`date(toString(d)) = d`, ...).

**Not** supported: named timezones (`'Europe/Stockholm'`) for `Time`/
`DateTime` — needs a real IANA timezone database (DST transition rules,
zone-name lookup), deliberately out of scope, no dependency pulled in
for it; week-date/ordinal-date/quarter construction (`date({year: 2015,
week: 1})`, `date('2015-W30-2')`, `date('2015-202')` — only the calendar
year/month/day forms, for the date half of every type); projecting one
temporal value from another (`date({date: d, day: 5})`);
`duration.between(...)`/`.inDays(...)`/etc, `.truncate(...)`, and the
alternate ISO-8601 combined date-time duration syntax
(`duration('P2012-02-02T14:37:21.545')`). See `marsdb-query/src/
temporal.rs`'s module doc comment for the same list in code.

## Indexes & EXPLAIN

`CREATE INDEX ON :Label(prop)` (optionally `UNIQUE`) declares a property
index, backfilled immediately from existing nodes. An equality against an
indexed `(label, prop)` compiles to a direct index seek instead of a label
scan + filter, whether it's a node-pattern property (`MATCH (n:Label
{prop: literal})`) or a `WHERE n.prop = literal` predicate — including one
buried in a `WHERE a = 1 AND b = 2`-shaped conjunction, where only the
indexed side seeks and the rest survives as a residual filter. When
several conjuncts each have a declared index, the planner picks the most
selective one by cheap, exact cardinality (a redb per-key entry count, not
an estimate) rather than whichever appears first. `EXPLAIN <statement>`
prints the compiled plan (scan vs seek, which conjunct fused, what's left
as a residual filter) without running it — always against a read-only
snapshot, even for a statement that would otherwise write. No `CREATE
CONSTRAINT`/composite indexes/range scans yet.

## Error taxonomy

`QueryError` is a typed taxonomy, not one flat string bucket: `Syntax`
(the query text itself never parsed), `Semantic` (it parsed fine but is
structurally invalid — an unsupported pattern shape, a misplaced
aggregate — knowable from the query alone, no data needed), `Type` (only
knowable once a real value from stored data or a `$parameter` turns out
to be the wrong shape — arithmetic on a non-number, indexing a non-list),
plus `Graph`/`UnboundVariable`/`MissingParam`/`Cancelled`/`Timeout`/
`ResourceLimit`. `ExecutionOutcome` (the `ExecutionObserver` telemetry
enum) mirrors the same split (`SyntaxError`/`SemanticError`/`TypeError`/
...) instead of one undifferentiated `ParseOrSemanticError`.

## Verified against

All 7 of LDBC SNB Interactive's short-read reference queries (IS1-IS7) —
see `marsdb-query/tests/ldbc_is_queries.rs`.

Not verified: LDBC's complex queries (IC1-14: the full query set beyond
one hand-crafted grouping+`WITH...WHERE`+`ORDER BY`+`LIMIT`+`collect()`
checkpoint — see `marsdb-query/tests/smoke.rs`), comma-separated patterns
*within one* `MATCH`/`CREATE` clause beyond a single linear chain (general
cross-joins — different from the cross-join WITH-chaining above, which
works), chaining past one `WITH` boundary, `MERGE` patterns with more than
one relationship hop (whole-pattern atomicity across multiple
simultaneously-unbound hops isn't attempted), named-path capture over a
variable-length pattern (only `shortestPath()` tracks the hop-by-hop chain
needed to reconstruct a path over `*`-traversal), or `shortestPath()` with
a minimum hop count greater than 1 (a plain visited-set BFS can't
correctly answer "shortest path of at least N hops" for N > 1 without a
different algorithm).
