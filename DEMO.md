# Demo: this repo's own commit history, as a MarsDB graph

Dogfooding exercise: model this repo's git history as a property graph
inside MarsDB, then run real queries against it — file-churn analysis,
commits per author. Everything below is copy-pasteable; running it
rebuilds the exact numbers shown here.

## What gets built

- One `(:Commit {hash, author, subject})` node per commit.
- One `(:File {path})` node per file ever touched by any commit.
- One `(c:Commit)-[:TOUCHES]->(f:File)` edge per (commit, file) pair.

This graph is why `MATCH ... CREATE` exists in MarsDB at all: standalone
`CREATE` can never connect two nodes that already exist — every node
token it sees always becomes a fresh one. Building 52 `Commit` nodes and
58 `File` nodes once, then wiring 315 edges between specific
already-existing pairs, needs a way to say "match this existing node,
match that existing node, connect them" — which is exactly what
`MATCH ... CREATE` adds (see the README's Cypher-coverage section).

Needs the `marsdb` binary on `PATH` (`cargo install marsdb-cli`, or run
`cargo build --release -p marsdb-cli` from a checkout of this repo and use
`./target/release/marsdb` in place of `marsdb` below) and Python 3 (only
used to shell out to `git log`, no dependencies beyond the standard
library).

## 1. Generate the Cypher batch

```
python3 examples/commit_graph.py > commit_graph.cypher
```

This shells out to `git log --pretty=format:@@%h|%an|%s --all --name-only`
and turns it into one big `;`-separated batch:

- One `CREATE` with all `Commit` nodes as comma-separated patterns
  (independent nodes, no shared variables needed).
- One `CREATE` with all `File` nodes, same shape.
- One `MATCH (c:Commit {hash: ...}) WITH c MATCH (f:File {path: ...})
  CREATE (c)-[:TOUCHES]->(f)` per (commit, file) pair — `WITH c` is what
  carries the already-bound `c` across into the second `MATCH`, so the
  `CREATE` at the end sees both `c` and `f` as existing nodes to connect,
  not new ones to create.

**Known limitation this script works around**: MarsDB's `string_literal`
grammar has no escape mechanism at all (`@{ "'" ~ (!"'" ~ ANY)* ~ "'" }` —
no `\'`), so a literal `'` inside a Cypher string is structurally
impossible to represent. Real commit messages have apostrophes and quoted
words ("aren't", `'not benchmarked yet'`), so the script strips `'`
characters out of author names and commit subjects before generating the
`CREATE` patterns. This is a real, open gap (see the README roadmap), not
specific to this demo — anything with real English text will hit it.

## 2. Load it

```
marsdb commit_graph.db "$(cat commit_graph.cypher)"
```

One `execute_batch()` call, ~317 statements, one transaction per
statement (see README's architecture section) — takes about 1.5s for
this repo's history.

## 3. Query it

**Sanity-check the counts:**

```
$ marsdb commit_graph.db "MATCH (c:Commit) RETURN count(*)"
count(*)
52

$ marsdb commit_graph.db "MATCH (f:File) RETURN count(*)"
count(*)
58

$ marsdb commit_graph.db "MATCH (:Commit)-[:TOUCHES]->(:File) RETURN count(*)"
count(*)
315
```

**Most-churned files** (implicit `GROUP BY` via an aggregate, `WITH...WHERE`
to filter the grouped result, `ORDER BY` + `LIMIT`):

```
$ marsdb commit_graph.db "MATCH (c:Commit)-[:TOUCHES]->(f:File) \
    WITH f.path AS path, count(c) AS touches WHERE touches > 1 \
    RETURN path, touches ORDER BY touches DESC LIMIT 10"
path | touches
README.md | 19
marsdb-query/src/executor.rs | 16
marsdb/Cargo.toml | 16
BENCHMARKS.md | 15
marsdb-query/src/cypher.pest | 14
marsdb-query/src/parser.rs | 14
marsdb-query/src/ast.rs | 12
marsdb-query/tests/smoke.rs | 12
Cargo.toml | 10
marsdb-cli/Cargo.toml | 10
```

**Commits per author:**

```
$ marsdb commit_graph.db "MATCH (c:Commit) \
    WITH c.author AS author, count(*) AS commits \
    RETURN author, commits ORDER BY commits DESC"
author | commits
Kenji Noguchi | 52
```

(Single-author repo, so this one's not very interesting here — but it's
the same query shape you'd run on a real multi-contributor project.)

## Why this is a real test, not just a toy

This graph is 35x the edge count of the first version of this demo (a
9-edge crate-dependency graph), and running it clean was itself a
correctness check: `MATCH ... CREATE` depends on `WITH`-chained parts
correctly carrying bound variables forward across a disjoint second
`MATCH` (`c` must survive into the `f` match) — a real bug in that exact
mechanism (`scan()` silently discarding carried rows instead of
cross-joining) was found and fixed while building the first version of
this demo, and also turned out to be a latent crash in `OPTIONAL MATCH`
that nothing had exercised yet. Loading 315 edges through it here without
incident is evidence that fix holds up, not just a demo of the feature
existing.
