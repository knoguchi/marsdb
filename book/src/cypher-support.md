# Cypher Language Support

MarsDB implements a subset of openCypher, checked against the
[openCypher Technology Compatibility Kit (TCK)](https://github.com/opencypher/openCypher)
— 220 feature files, 3,880 scenarios, vendored as a git submodule and run
on every push. The full, exhaustive
breakdown — every supported clause/expression/temporal-type shape, the
error taxonomy, and this same table with contributor-level detail — lives
in [`CYPHER_COVERAGE.md`](https://github.com/knoguchi/marsdb/blob/main/CYPHER_COVERAGE.md)
in the repo root. This page is the condensed, end-user version.

## Conformance, by category

| category | total | pass | pass % |
|---|---|---|---|
| clauses/call | 52 | 52 | 100.0% |
| clauses/create | 78 | 78 | 100.0% |
| clauses/delete | 41 | 41 | 100.0% |
| clauses/match | 381 | 381 | 100.0% |
| clauses/match-where | 34 | 34 | 100.0% |
| clauses/merge | 75 | 75 | 100.0% |
| clauses/remove | 33 | 33 | 100.0% |
| clauses/return | 63 | 63 | 100.0% |
| clauses/return-orderby | 35 | 35 | 100.0% |
| clauses/return-skip-limit | 31 | 31 | 100.0% |
| clauses/set | 53 | 53 | 100.0% |
| clauses/union | 12 | 12 | 100.0% |
| clauses/unwind | 14 | 14 | 100.0% |
| clauses/with | 29 | 29 | 100.0% |
| clauses/with-orderBy | 292 | 292 | 100.0% |
| clauses/with-skip-limit | 9 | 9 | 100.0% |
| clauses/with-where | 19 | 19 | 100.0% |
| expressions/aggregation | 35 | 35 | 100.0% |
| expressions/boolean | 150 | 150 | 100.0% |
| expressions/comparison | 72 | 72 | 100.0% |
| expressions/conditional | 13 | 13 | 100.0% |
| expressions/existentialSubqueries | 10 | 10 | 100.0% |
| expressions/graph | 61 | 61 | 100.0% |
| expressions/list | 185 | 185 | 100.0% |
| expressions/literals | 131 | 131 | 100.0% |
| expressions/map | 44 | 44 | 100.0% |
| expressions/mathematical | 6 | 6 | 100.0% |
| expressions/null | 44 | 44 | 100.0% |
| expressions/path | 7 | 7 | 100.0% |
| expressions/pattern | 50 | 50 | 100.0% |
| expressions/precedence | 104 | 104 | 100.0% |
| expressions/quantifier | 604 | 604 | 100.0% |
| expressions/string | 32 | 32 | 100.0% |
| expressions/temporal | 1004 | 1004 | 100.0% |
| expressions/typeConversion | 47 | 47 | 100.0% |
| useCases/countingSubgraphMatches | 11 | 11 | 100.0% |
| useCases/triadicSelection | 19 | 19 | 100.0% |
| **TOTAL** | **3880** | **3880** | **100.0%** |

Reproduce this table yourself:

```
git submodule update --init marsdb-tck/openCypher
cargo run --release -p marsdb-tck
```

## What's supported

- **Patterns**: `MATCH`/`OPTIONAL MATCH`, undirected/bracketless/multi-type/
  variable-length relationship patterns, named-path capture (including
  over variable-length hops), `shortestPath()`, pattern comprehension,
  pattern predicates.
- **Reading & filtering**: `WHERE` (property/identity/label comparisons,
  pattern predicates, `STARTS WITH`/`ENDS WITH`/`CONTAINS`), `exists {
  ... }` (both the simple pattern form and the full nested-subquery
  form, including nested `exists {}`), `UNWIND`, any number of chained
  `WITH` boundaries, `ORDER BY`/`SKIP`/`LIMIT`, `DISTINCT`.
- **Writing**: `CREATE`, `MERGE` (with `ON CREATE`/`ON MATCH`), `SET`,
  `REMOVE`, `DELETE`/`DETACH DELETE`, `MATCH ... CREATE`, arbitrary
  chaining of mutating clauses via trailing `WITH` or `RETURN`.
  `MERGE` is capped at one relationship hop.
- **Aggregation**: implicit `GROUP BY`, `count`/`sum`/`avg`/`min`/`max`/
  `collect`/`percentileCont`/`percentileDisc`, `DISTINCT` inside an
  aggregate call, aggregate expressions composed with arithmetic.
- **Functions**: the standard scalar/string/math/list function set
  (`coalesce`, `toInteger`/`toFloat`/`toString`/`toBoolean`, `keys`/
  `labels`/`type`/`properties`/`id`, `size`/`length`/`nodes`/
  `relationships`/`head`/`tail`/`last`/`range`, `toUpper`/`toLower`/
  `trim`/`replace`/`split`/`substring`, `abs`/`ceil`/`floor`/`round`/
  `sqrt`/`sign`).
- **Temporal types**: `Date`/`LocalTime`/`Time`/`LocalDateTime`/
  `DateTime`/`Duration` — construction from strings, maps, or another
  temporal value; comparison; calendar-aware arithmetic; full component
  access; ISO-8601 string round-tripping.
- **Stored procedures**: `CALL proc(args) [YIELD ...]`, both standalone
  and in-query forms, against a caller-supplied `ProcedureProvider` —
  see [Embedding in Rust](./embedding-rust.md#stored-procedures-call).
  MarsDB itself ships no built-in procedures.
- **Parameters**: `$name` scalars, lists (including nested lists), and
  maps.

## Known limitations

- **Node/relationship-valued `$parameters`** aren't supported. Scalar,
  list, and map parameters already work. This is on the
  [roadmap](https://github.com/knoguchi/marsdb#roadmap).
- **No cost-based query optimizer**. Index seeks and top-k `ORDER BY
  ... LIMIT` selection exist, but join/traversal ordering isn't
  cost-estimated. See [Architecture](./architecture.md). This is on the
  [roadmap](https://github.com/knoguchi/marsdb#roadmap).
- **`CALL` needs an embedder-supplied `ProcedureProvider`** — there's no
  built-in procedure catalog.
