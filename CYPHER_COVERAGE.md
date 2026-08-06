# Cypher coverage

## TCK conformance

Real, measured numbers from the vendored openCypher TCK (`marsdb-tck/`,
220 feature files, 3880 scenarios) — reproduce with:

```
cargo run -p marsdb-tck --release
```

For fast iteration, restrict to a category/feature via `TCK_FILTER` (substring
match against each scenario's path relative to `openCypher/tck/features/`):

```
TCK_FILTER="clauses/create" cargo run -p marsdb-tck --release
```

Results + coarse error-checking only (a scenario expecting an error passes
iff MarsDB errored *at all*, not the *right kind* of error; no side-effect
assertions) — see `marsdb-tck/src/main.rs`'s own doc comments for the full
scope.

| category | total | pass | wrong | unexp | reject | unsup | pass % |
|---|---|---|---|---|---|---|---|
| clauses/call | 52 | 16 | 0 | 0 | 36 | 0 | 30.8% |
| clauses/create | 78 | 78 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/delete | 41 | 40 | 0 | 0 | 1 | 0 | 97.6% |
| clauses/match | 381 | 357 | 0 | 0 | 24 | 0 | 93.7% |
| clauses/match-where | 34 | 31 | 0 | 3 | 0 | 0 | 91.2% |
| clauses/merge | 75 | 75 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/remove | 33 | 33 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/return | 63 | 60 | 0 | 1 | 2 | 0 | 95.2% |
| clauses/return-orderby | 35 | 34 | 0 | 0 | 1 | 0 | 97.1% |
| clauses/return-skip-limit | 31 | 31 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/set | 53 | 53 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/union | 12 | 12 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/unwind | 14 | 12 | 0 | 0 | 0 | 2 | 85.7% |
| clauses/with | 29 | 27 | 0 | 0 | 2 | 0 | 93.1% |
| clauses/with-orderBy | 292 | 292 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with-skip-limit | 9 | 9 | 0 | 0 | 0 | 0 | 100.0% |
| clauses/with-where | 19 | 17 | 0 | 0 | 2 | 0 | 89.5% |
| expressions/aggregation | 35 | 35 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/boolean | 150 | 150 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/comparison | 72 | 72 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/conditional | 13 | 13 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/existentialSubqueries | 10 | 5 | 0 | 0 | 5 | 0 | 50.0% |
| expressions/graph | 61 | 57 | 0 | 0 | 4 | 0 | 93.4% |
| expressions/list | 185 | 184 | 0 | 0 | 0 | 1 | 99.5% |
| expressions/literals | 131 | 131 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/map | 44 | 39 | 0 | 0 | 0 | 5 | 88.6% |
| expressions/mathematical | 6 | 6 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/null | 44 | 44 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/path | 7 | 2 | 0 | 0 | 5 | 0 | 28.6% |
| expressions/pattern | 50 | 49 | 0 | 0 | 1 | 0 | 98.0% |
| expressions/precedence | 104 | 104 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/quantifier | 604 | 596 | 0 | 0 | 8 | 0 | 98.7% |
| expressions/string | 32 | 32 | 0 | 0 | 0 | 0 | 100.0% |
| expressions/temporal | 1004 | 999 | 1 | 3 | 1 | 0 | 99.5% |
| expressions/typeConversion | 47 | 47 | 0 | 0 | 0 | 0 | 100.0% |
| useCases/countingSubgraphMatches | 11 | 11 | 0 | 0 | 0 | 0 | 100.0% |
| useCases/triadicSelection | 19 | 19 | 0 | 0 | 0 | 0 | 100.0% |
| **TOTAL** | **3880** | **3772** | **1** | **7** | **92** | **8** | **97.2%** |

Parser: ANTLR4-generated (`marsdb-query/grammar/`, see that directory's
own README for provenance/regen), replacing an earlier hand-rolled `pest`
grammar at Phase 3 cutover (`mars-cuk`) — see git history for the pest-era
snapshot of this table if useful for comparison.

Column meanings:
- **pass** — matched (or, for an error-expecting scenario, errored at all).
- **wrong** — ran successfully but returned the wrong rows. **The category
  that means a real bug**, not a coverage gap — one known, deliberately
  unfixed case: `expressions/temporal`'s `Temporal10 [1]`: `duration.
  between`'s result correctly renders as the same ISO-8601 text real
  Cypher produces, but its raw `.seconds`/`.nanosecondsOfSecond` component
  fields can differ from Java's exact internal split for a negative,
  sub-second-remainder duration specifically (`seconds: -86400,
  nanosecondsOfSecond: 100000000` there vs MarsDB's `-86399, -900000000`)
  — MarsDB's `Duration` always keeps `nanos`'s sign matching `seconds`' (a
  real, deliberately enforced invariant every other Duration operation in
  this codebase depends on), so matching Java's split here would mean
  either breaking that invariant or letting two different internal
  representations of the same duration coexist (silently wrong
  equality/hashing) — not worth it for one accessor pair on one edge
  case. See `duration_between`'s docs in `marsdb-query/src/temporal.rs`.
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

`expressions/temporal` is 1004 of the 3880 total scenarios (26%). Calendar
construction/parsing (`year`/`month`/`day`, ISO week-date, ordinal-date,
and quarter-date forms, both as map keys and as strings), component
access, arithmetic, comparison, and cross-type conversion (`date(anExisting
DateTime)`, etc.) are all supported for every one of the six temporal
types, not just `Date`/`Duration` — see [Temporal types](#temporal-types)
below. `DateTime` also supports named IANA timezones (`'Europe/
Stockholm'`), not just a fixed UTC offset, via `chrono-tz`'s embedded
database — real, DST-aware offset resolution, not a fixed lookup table
(`chrono-tz`'s IANA data even covers pre-standardization local-mean-time
offsets, e.g. Stockholm in 1818 resolves to `+00:53:28`). `Time` still
only accepts a fixed offset — it carries no calendar date, so a named
zone's DST-dependent offset has nothing to resolve against, a real
structural limit of the type rather than a missing feature. The remaining
temporal `unexp`/`reject` scenarios are narrow, specific gaps: dates
outside `chrono`'s representable range (`'-999999999-01-01'`) and the
alternate ISO-8601 combined date-time duration syntax
(`duration('P2012-02-02T14:37:21.545')`, as opposed to the plain
`'P1Y2M3DT4H5M6S'` form, which is fully supported).

The remaining non-temporal `unexp` scenarios are each their own narrow,
specific gap, not one common cause: `Return2 [14]` (returning the *type*
of an already-deleted relationship — MarsDB's `DeletedEntityAccess` check
fires eagerly on any access to a deleted entity, real Cypher's own `type()`
apparently doesn't need the record itself so it's exempt), and
`MatchWhere1 [12,13,15]` (a path-length predicate and an aggregate used
directly inside pattern-level `WHERE`, both real Cypher restrictions this
codebase doesn't enforce at compile time).

## Clauses

`CREATE`, multi-label nodes (`(n:Post:Message)`), `$parameters`,
backslash-escaped string literals (`\' \" \\ \n \r \t \b \f`),
`MATCH`/`OPTIONAL MATCH`, undirected (`-[r:TYPE]-`), bracketless
(`-->`/`<--`/`--`, the anonymous-untyped-relationship shorthand — brackets
are only needed at all to carry a var/type/range/props), multi-type
(`[:A|B]`/`[:A|:B]`, both separator forms — matches if the edge's type is
any of the listed alternatives), and variable-length (`[:TYPE*min..max]`)
relationship patterns. `CREATE`/`MERGE` require exactly one explicit
relationship type on every hop (`-[:KNOWS]->`, `-->` or `-[:A|B]->` alone
is rejected with a clear semantic error) — unlike `MATCH`, where an
untyped/multi-typed hop legitimately means "any relationship"/"any of
these," a brand new edge can't be ambiguous about which single type it
gets, matching real Cypher's own rule. `WHERE` (including the string
predicates `STARTS WITH`/`ENDS WITH`/`CONTAINS`, a user-typed label
predicate `WHERE n:Label`/`n:Label1:Label2` — also usable as a general
expression anywhere one is, not just `WHERE` (`RETURN a:B AS result`) —
a property compared against *another* property rather than a constant
`WHERE a.id = b.id`, node/relationship identity comparison
`WHERE a = b`/`WHERE a <> b` — only `=`/`<>` are meaningful for identity,
no ordering exists between two nodes/relationships, so `WHERE a < b` is a
real error — and a pattern predicate used as a boolean expression,
`WHERE (n)-[:REL]->()`, existential and negatable/AND/OR-combinable,
correlated against whatever's already bound: `WHERE (n)-[]->(m)` with
both `n`/`m` already bound by an outer `MATCH` means "is there a real edge
between these two specific nodes," not "does `n` have any outgoing edge
at all"), pattern comprehension (`[(n)-[:T]->(b) | b.name]`/
`[p = (n)-->() | p]` — unlike a pattern predicate, can introduce brand-new
node/relationship variables, and enumerates every match instead of just
checking existence), `exists { (n)-->(m) WHERE ... }` (the "simple" form
only — a pattern with an optional inline `WHERE`; the "full" form,
`exists { MATCH ... RETURN ... }`, a real nested subquery, isn't
supported yet), any number of chained `WITH` boundaries per statement
(projection/rename, its own `WHERE`/`WITH...WHERE`/`ORDER BY`/`LIMIT`,
and `WITH *` — every currently-bound variable carries forward unchanged,
optionally alongside more items), `RETURN`/`RETURN *`/`DELETE`/
`DETACH DELETE`/`SET`/`REMOVE`/
`MATCH ... CREATE` (adds an edge between two already-matched nodes — a
node token whose variable is already bound reuses that node instead of
creating a new one) or a standalone `CREATE ... RETURN ...` with no
preceding `MATCH` at all. `SET`/`REMOVE` cover both properties
(`SET n.prop = 'x'`/`REMOVE n.prop`) and labels (`SET n:Label`/
`REMOVE n:Label`); `SET n.prop = null` removes the property rather than
storing a literal null, matching real Cypher. `CREATE`/`SET`/`DELETE`/
`DETACH DELETE`/`REMOVE` can each optionally be followed by one trailing
`RETURN` in the same statement (`MATCH (n) SET n.prop = 1 RETURN n`,
`MATCH (n) DELETE n RETURN count(n)`), *or* continue the query past the
mutation via a `WITH` (`MATCH (n) SET n.prop = 1 WITH n WHERE ...
RETURN ...`) — real Cypher allows arbitrarily chaining these, not just
one mutating clause immediately before a single terminal `RETURN`.

Multi-key `ORDER BY`, `SKIP`/`LIMIT` (on both `RETURN` and `WITH`; `SKIP`
always applies after `ORDER BY` and before `LIMIT` regardless of clause
order in the query text, matching real Cypher), `CASE`, and implicit-GROUP-BY aggregation
(`count()`/`count(*)`/`sum()`/`avg()`/`min()`/`max()`/`collect()`/
`percentileCont()`/`percentileDisc()`, with `DISTINCT` inside an
aggregate call — including grouping/`DISTINCT` by a list or map value,
structurally, not just a scalar). `RETURN DISTINCT`
(result-set-level dedup of the whole projected row, applied after
grouping for an aggregating `RETURN` — a separate mechanism from
`DISTINCT` inside one aggregate call, which only affects that aggregate's
own accumulation).

Built-in scalar functions: `coalesce()`, `toInteger()`/`toFloat()`/
`toString()`/`toBoolean()`, `keys()`/`labels()`/`type()`/`properties()`/
`id()` (node/relationship introspection), `size()` (list length or
string character count), `length()` (path edge count), `nodes()`/
`relationships()` (a path's elements), `head()`/`tail()`/`last()`/
`range(start, end[, step])` (list construction/slicing — `range` is
both-ends-inclusive, matching real Cypher, not Rust's exclusive-end
convention), `exists()` (property presence only — pest's own bare
`exists((n)-->())`-as-a-function-call form was never real Cypher syntax,
removed rather than preserved; real Cypher's own pattern-existence form
is `exists { (n)-->() }`, see below), string functions `toUpper()`/
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
(`[1, 2, 'a', $p]`), a variable bound by a preceding
`WITH ... collect(...)`/a list-valued node or relationship property, or
`$param` itself naming a list (including nested lists) directly. A
map/node/relationship-*valued* `$param`, though, still isn't supported —
the public parameter type (`PropertyValue`, marsdb-graph) has no
Map/Node/Edge variant at all, unlike a list-valued property (a
different, already-supported thing, see below). `MERGE <pattern> [ON
CREATE SET ...] [ON MATCH SET ...]`
(match-or-create: tries the pattern as an ordinary MATCH first, creates
exactly one new instance if nothing matched) — capped at one relationship
hop (`MERGE (n:Label {props})` or `MERGE (a)-[:TYPE]->(b)`); a completely
unconstrained node pattern (`MERGE (n)`, no label or property) is valid,
real Cypher too — searches for/creates any node with no constraints at
all. Named-path capture (`MATCH p = (a)-[:KNOWS]->(b) RETURN p`,
fixed-hop patterns only) and `shortestPath((a)-[:TYPE*..N]-(b))` (real
shortest-path search via BFS, not just the first path found — both
endpoints must already be matched by a preceding clause), plus
`length(p)` to measure one.

## Expressions

Arithmetic (`+ - * / %`, plus `^` exponentiation and general unary minus
— real precedence: unary minus binds tightest, then `^` (left-
associative — `4 ^ 3 ^ 2` is `(4 ^ 3) ^ 2`, confirmed against the TCK's
own fixture, not assumed from general math convention, which is
right-associative and would be wrong here), then `* / %`, then `+ -`,
explicit `(...)` grouping to override any of it) in `RETURN`/`WITH`
items, `ORDER BY` keys, function arguments, and a pattern-level `WHERE`'s
comparison operands (`WHERE n.x + 1 = 6`, via the same general-expression
comparison form map/date comparisons already use, see below) — `+` also
concatenates two strings, or appends/prepends/concatenates when either
side is a list (`[1,2] + 3` is `[1,2,3]`, `[1,2] + [3]` is `[1,2,3]`);
`^` always produces a float, even for two integer operands, matching real
Cypher's own rule. An aggregate can be composed with other expressions
(`RETURN a, count(a) + 3`, `RETURN count(*) * 10 AS c`) as long as every
non-aggregate value used alongside it is itself an explicit grouping key
— another `RETURN`/`WITH` item's own top-level expression, verbatim, not
just something that happens to be in scope (`RETURN me.age, me.age +
count(you.age)` is fine; `RETURN me.age + count(you.age)` alone is a
compile-time error, real Cypher's `AmbiguousAggregationExpression`).
Composing an aggregate inside a list comprehension/quantifier's
*projection* is a flat rejection regardless (no defined "aggregate once
per list element" semantics) — its *source* can still be an aggregate
(`[x IN collect(p) | head(nodes(x))]`), evaluated once per group same as
anywhere else.

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
`RETURN`/`WITH` expression is, not just `WHERE` — including a correctly
three-valued (`null`-propagating) `List`/`Map` equality and lexicographic
`List` ordering, and a `NaN`-safe `<`/`<=`/`>`/`>=` that returns `false`
(not `null`) the way real Cypher does. `{key: <expr>, ...}` map literals
are a general expression (`RETURN {a: 1, b: 2}`, `WITH {x: 1} AS m RETURN
m.x`, `map['key']` dynamic access) usable anywhere a `RETURN`/`WITH`
expression is, including as a `CREATE`/`MERGE` pattern's inline property
map value.

A list-valued node/edge property (`CREATE (n {tags: [1, 2, 3]})`, real
Cypher/Neo4j's own "homogeneous array property" shape) is fully
supported and round-trips as a genuine list, not an opaque scalar —
`n.tags[0]`, `size(n.tags)`, `x IN n.tags`, and `UNWIND n.tags AS x` all
work transparently on it, the same as a list literal or `collect()`
result. A *map*-valued property (`CREATE (n {tags: {a: 1}})`) is still a
real error, not silently null — matching real Cypher, which only ever
allows scalars or homogeneous scalar arrays as a stored property, never a
nested map. Grouping/`DISTINCT` by a map value (including one containing
a list) also works, hashed by its sorted `(key, value)` entries so
`{a: 1, b: 2}` and `{b: 2, a: 1}` correctly group together.

## Temporal types

All six Cypher temporal types are supported: `Date`, `Duration`,
`LocalTime`, `Time`, `LocalDateTime`, `DateTime` (each a first-class
`PropertyValue` storage variant, not `Int`/`String` reused — see
`marsdb-graph/src/model.rs`'s doc comment). `DateTime` accepts either a
*fixed* UTC offset (`'+01:00'`, `{timezone: '+01:00'}`) or a named IANA
timezone (`'Europe/Stockholm'`, `{timezone: 'Europe/Stockholm'}`), stored
as `marsdb_graph::TzId` (`Offset(i32)` or `Named(String)`) — a `Named`
zone's real offset is never cached, it's re-resolved on demand via
`chrono-tz` for whichever instant it's needed at, since the same zone has
different offsets across a DST transition. `Time` only accepts a fixed
offset — no calendar date to resolve a named zone's DST-dependent offset
against, a real structural limit of the type, not a missing feature.

Construction: `date()`/`localtime()`/`time()`/`localdatetime()`/
`datetime()` with no arguments (the current UTC value); from a string,
either a plain calendar date (`date('2015-07-21')`,
`time('21:40:32.142+01:00')`, `YYYY-MM-DD`/`YYYYMMDD`/`YYYY-MM`/`YYYYMM`/
`YYYY`), an ISO week-date (`date('2015-W30-2')`/`date('2015W302')`, day
defaults to `1` when omitted), or an ordinal-date (`date('2015-202')`/
`date('2015202')`); from a map, either the plain calendar form
(`date({year, month, day})`) or the week-date (`date({year, week,
dayOfWeek})`) / ordinal-date (`date({year, ordinalDay})`) / quarter-date
(`date({year, quarter, dayOfQuarter})`) alternates — `time({hour, minute,
second, millisecond, microsecond, nanosecond, timezone})`, ...; from
another value of the *same* type (identity, e.g. round-tripping through
`toString`); or from a value of a *different* temporal type
(`date(existingDateTime)`, `localtime(existingTime)`, ...), which
projects just the relevant part, same as the equivalent `{date: ...}`/
`{time: ...}` map form. `time()`/`datetime()` from a string with no
offset default to UTC rather than erroring (matches real Cypher's
"statement default time zone" fallback). `duration({years, months,
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

Projecting one temporal value's fields from another via a `date`/`time`/
`datetime` map key (`date({date: d, day: 5})`, `localtime({time: t,
second: 42})`, `localdatetime({date: d, time: t})`, `time({time: t,
timezone: '+05:00'})`, ...): the named key's calendar and/or clock
fields become the defaults, any other explicit key overrides just that
field on top. Changing `timezone` on a projected `Time`/`DateTime` whose
source already carried an offset shifts the wall-clock to preserve the
same instant (real Cypher's rule — `{time: t, timezone: '+05:00'}` on a
`+01:00` source advances the hour by 4), and any further explicit
hour/minute/second override applies *after* that shift, not before.

`duration.between(a, b)`/`.inMonths(a, b)`/`.inDays(a, b)`/
`.inSeconds(a, b)` — a real calendar-aware duration between any two of
the 5 non-`Duration` temporal types (25 cross-type combinations),
matching real Cypher's own rules: real calendar month arithmetic
(`.between`/`.inMonths`), instant-aware reconciliation when both
operands carry a UTC offset (`Time`/`DateTime` at *different* offsets),
and — when either operand has no calendar date at all (`LocalTime`/
`Time`) — the whole calculation degrades to a plain time-of-day
difference, disregarding any date the *other* operand happens to have.
`.inDays`/`.inSeconds` discard the month optimization and use the raw,
un-optimized elapsed time instead (so `.inDays` on a date+time target
truncates away any sub-day remainder rather than carrying it). Every
no-arg `date()`/`localtime()`/`time()`/`localdatetime()`/`datetime()`
call within one query shares a single captured instant (real Cypher's
guarantee — `duration.between(date(), date())` is always exactly
`PT0S`).

`<type>.truncate(unit, value, map?)` — rounds `value` down to the start
of `unit` (`millennium`/`century`/`decade`/`year`/`quarter`/`month`/
`week`/`weekYear`/`day` for the calendar half, `hour`/`minute`/`second`/
`millisecond`/`microsecond` for the clock half — `day` resets the whole
time-of-day to midnight), then applies an optional trailing map's field
overrides on top (same override semantics as temporal projection —
`dayOfWeek` moves within the truncated result's own ISO week, and a
sub-second override touching only `nanosecond` keeps the truncated
base's own millisecond/microsecond digits rather than resetting them).
`datetime.truncate`/`localdatetime.truncate` truncate *both* halves at
once for a calendar-scale unit (resetting time to midnight) or just the
clock half for a clock-scale one.

**Not** supported: named timezones for `Time` (only `DateTime` can hold
one — see above); the alternate ISO-8601 combined date-time duration
syntax (`duration('P2012-02-02T14:37:21.545')`); dates outside `chrono`'s
representable range. See `marsdb-query/src/temporal.rs`'s module doc
comment for the same list in code.

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
works), `MERGE` patterns with more than
one relationship hop (whole-pattern atomicity across multiple
simultaneously-unbound hops isn't attempted), named-path capture over a
variable-length pattern (only `shortestPath()` tracks the hop-by-hop chain
needed to reconstruct a path over `*`-traversal), or `shortestPath()` with
a minimum hop count greater than 1 (a plain visited-set BFS can't
correctly answer "shortest path of at least N hops" for N > 1 without a
different algorithm).
