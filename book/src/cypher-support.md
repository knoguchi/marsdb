# Cypher Language Reference

MarsDB implements a subset of Cypher. It passes the full openCypher
TCK. Exhaustive breakdown: [`CYPHER_COVERAGE.md`](https://github.com/knoguchi/marsdb/blob/main/CYPHER_COVERAGE.md).

## Supported

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

## Conformance testing

Checked against the
[openCypher Technology Compatibility Kit (TCK)](https://github.com/opencypher/openCypher)
on every push: 3,880/3,880 scenarios pass across 220 feature files.
Per-category numbers are in
[`CYPHER_COVERAGE.md`](https://github.com/knoguchi/marsdb/blob/main/CYPHER_COVERAGE.md).

```
git submodule update --init marsdb-tck/openCypher
cargo run --release -p marsdb-tck
```
