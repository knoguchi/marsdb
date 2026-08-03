# The openCypher TCK submodule

`openCypher/` is a git submodule pointing at
[opencypher/openCypher](https://github.com/opencypher/openCypher), pinned at
commit `677cbafabb8c3c5eed458fd3b1ec0daec8d67d23` (2026-03-20). The runner
reads `.feature` files from `openCypher/tck/features/` directly — 220 files,
3880 real scenarios once every `Scenario Outline:` + `Examples:` table is
expanded (see below) — 1615 raw `Scenario:`/`Scenario Outline:` headers
(a plain `grep -c '^\s*Scenario:'` undercounts even that by 276, since it
doesn't match `Scenario Outline:` at all), 276 of which are outlines
expanding to 2541 real instances via their `Examples:` tables.

A submodule (not a vendored copy) so the build has no bearing on Apache-2.0
attribution bookkeeping — the checked-out submodule *is* the real upstream
repo, `LICENSE`/`NOTICE` included, not a partial copy needing its own
attribution files. It also means no drift risk from hand-copying and no
repo-size cost from tracking 220 files directly. The tradeoff: cloning this
repo needs an extra step to pull the TCK content in:

```
git submodule update --init marsdb-tck/openCypher
```

`marsdb-tck` fails fast with that same instruction if the submodule isn't
checked out, rather than silently reporting zero scenarios.

`Scenario Outline:` + `Examples:` (Cucumber's scenario templating) is
expanded for real: `gherkin.rs` parses the outline as a template (`query`/
`setup_cypher`/`expected` still containing literal `<placeholder>` tokens)
paired with its `Examples:` table, then emits one real `Scenario` per data
row with every `<col>` token replaced by that row's value — in the query
text, `having executed` setup, parameter values, and expected-result cells
alike (substitution isn't scoped to just the query, matching real Cucumber
semantics; most vendored outlines only use `<col>` in the query, but
nothing about the format guarantees that).

To move to a newer upstream commit:

```
cd marsdb-tck/openCypher
git fetch
git checkout <new-commit>
cd ../..
git add marsdb-tck/openCypher
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
