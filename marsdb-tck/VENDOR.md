# Vendoring the openCypher TCK

`TCK-LICENSE`/`TCK-NOTICE` are copied from upstream's repo root, alongside
this vendored content, per the Apache-2.0 attribution requirement — separate
from MarsDB's own `LICENSE-MIT`/`LICENSE-APACHE` at the repo root, which
apply to MarsDB's own code, not this vendored third-party test suite.

`features/` is a copy of `tck/features/` from
[opencypher/openCypher](https://github.com/opencypher/openCypher), pinned at
commit `677cbafabb8c3c5eed458fd3b1ec0daec8d67d23` (2026-03-20). 220
`.feature` files, 1615 `Scenario:`/`Scenario Outline:` headers (per this
crate's own Gherkin parser, the authoritative count — a plain
`grep -c '^\s*Scenario:'` undercounts by 276, since it doesn't match
`Scenario Outline:` at all), license headers (Apache-2.0 + the openCypher
attribution notice) kept intact in every file, as copied.

Each `Scenario Outline:` is parsed and run as exactly one scenario, using
the literal, unsubstituted `<placeholder>` text still in its query (e.g.
`ORDER BY <sort>`) — the `Examples:` table that real Cucumber execution
would use to expand one outline into N instantiated scenarios isn't read
at all; the table is silently skipped as an unrecognized block. This means
outline-based scenarios almost always end up `ParseRejected` (the
placeholder text isn't valid Cypher) rather than exercising the N real
variants upstream defines. A known v1 simplification, not a bug — full
`Examples:` expansion would be the natural follow-up if outline coverage
ever needs to be real.

To re-vendor against a newer upstream commit:

```
git clone --depth 1 https://github.com/opencypher/openCypher.git /tmp/oc
rm -rf features
cp -r /tmp/oc/tck/features features
```

## `graphs/*.cypher` — rewritten, not verbatim

`tck/graphs/binary-tree-1/binary-tree-1.cypher` and `binary-tree-2/...` (the
two named-graph fixtures scenarios can reference via `Given the
binary-tree-1 graph`) can't be vendored as-is. Upstream, each is four
`CREATE` blocks in one Cypher script, with later blocks referencing node
variables bound in earlier blocks (e.g. `(a)-[:KNOWS]->(b1)`, where `a`/`b1`
were created by an earlier `CREATE`) — real Cypher allows chaining multiple
`CREATE` clauses within one statement, seeing each other's bindings.
MarsDB's grammar doesn't support that shape (`create_stmt` is a complete,
standalone statement) — and splitting the blocks into separate MarsDB
statements doesn't work either, since separate statements share no
variable bindings at all.

The fixtures here are rewritten instead: one `CREATE` for all 13 nodes
(fresh, no cross-references — unaffected by the above), followed by one
`MATCH (x {name:'..'}) WITH x MATCH (y {name:'..'}) CREATE (x)-[:TYPE]->(y)`
statement per edge, matching both endpoints by their `name` property
(unique within each fixture) instead of relying on Cypher-level variable
bindings across statements. Same shape as this repo's own
`MATCH...CREATE` feature exists for, and the same one `marsdb/examples/
social_graph.rs` hits for the identical reason. Verified to produce the
correct topology (13 nodes, 16 edges each) by running both against a real
MarsDB instance and checking `MATCH (n) RETURN count(*)` /
`MATCH ()-[r]->() RETURN count(*)`.
