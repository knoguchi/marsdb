# The IR and the Planner

A `MATCH` pattern compiles to a small tree of logical operators
(`ir.rs`), which the planner (`planner.rs`) then rewrites with storage
statistics in view. This chapter also covers property indexes
themselves (`marsdb-graph/src/index.rs`), because the planner's most
important rewrites exist to exploit them.

## The operator inventory

`LogicalPlan` has nine operators. Four are leaves — ways to produce
initial rows:

- `AllNodesScan` — every node.
- `NodeByLabelScan` — every node with a label, via the label index.
- `IndexSeek` / `IndexRangeSeek` — nodes with `label` whose indexed
  property equals a value, or falls in a bounded range.
- `Seed` — no storage at all: start from the rows already bound by a
  previous part of the statement (the `WITH` continuation case).

One is a combined leaf-and-hop: `EdgeTypeScan`, a sequential sweep of
the whole `EDGES` table that binds an entire single-hop pattern at
once — more on it below. The rest transform rows: `Expand` (one fixed
hop through adjacency), `VarExpand` (`[:TYPE*1..3]`, a bounded BFS per
input row), and `Filter` (a predicate over the bound row).

The tree shape is deliberately Gremlin-like — scans feeding expansions
feeding filters — rather than Cypher-shaped, and the payoff is that
"which access path" becomes a local substitution: an `IndexSeek` can
replace a `Filter`-over-`NodeByLabelScan` without anything above it
noticing.

## Plan building is storage-free; rewriting is not

Planning happens in two phases with an explicit boundary.

`build_match_plan` runs with no storage access at all. It picks the
start variable's leaf (a `Seed` if the variable is already bound, else
a scan), chains an `Expand` per hop, synthesizes filters from inline
pattern properties and extra labels, and wraps remaining `WHERE`
conjuncts around the result. Because it cannot know which indexes
exist, it never emits an `IndexSeek`.

Then, inside the statement's transaction — where a `Txn` exists and
the question "is there an index on `(Person, name)`" has a definite
answer — the rewrite passes run: `apply_index_seeks` and the
start-point strategies. This split keeps every storage-dependent
decision inside the snapshot it will execute against, and keeps the
pure part unit-testable without a database.

Pattern semantics are enforced during building, and two bookkeeping
sets deserve mention because they encode real Cypher rules that are
easy to get wrong. *Edge isomorphism*: no single pattern may bind two
hops to the same relationship instance — the planner threads the set
of prior hops' relationship variables into each subsequent hop (and
into `VarExpand`'s BFS exclusion set) so a hop cannot walk back over
an edge the pattern already used. *Bound-variable repetition*:
`MATCH (n)-[r]->(n)` reuses `n` for both endpoints, which must mean
"the same node" — a repeated variable gets a fresh internal name plus
a synthesized `VarEq` filter, turning the identity constraint into
ordinary predicate machinery.

## Predicate pushdown

`build_match_plan` initially wraps the whole pattern in one `Filter`
holding the `WHERE` clause. Left there, a conjunct like
`start.prop = 'x'` in a multi-hop pattern sits *above* every `Expand`,
where the index rewrite — which only inspects what is immediately
under a `Filter` — could never see the scan it should replace. So the
builder splits the predicate into top-level `AND`-conjuncts, and each
conjunct that provably depends on only the start variable is pushed
down to wrap the start node's leaf directly.

The eligibility test (`conjunct_sole_var`) is deliberately narrow: it
recognizes the simple leaf shapes whose variable references are
manifest, and anything it does not recognize stays exactly where it
was. A pushdown that guesses can change results; one that declines
merely misses an optimization. The same conservatism repeats
throughout the planner, and it is a stance worth naming: **every
rewrite must be provably answer-preserving, and when in doubt, the
plan stays naive.**

## Property indexes and the seek rewrite

A property index is declared per `(label, property)` pair, optionally
unique. Its entries live in the shared `PROPERTY_INDEX` table under
`label_id ++ prop_id ++ encoded_value` keys, and the value encoding is
where the interesting engineering sits: an **order-preserving byte
encoding**, so that lexicographic byte comparison equals real value
ordering within a type. Signed integers get the standard sign-bit
flip (mapping two's-complement order onto unsigned big-endian byte
order); floats get the sortable-float transform (flip the sign bit
for non-negatives, flip every bit for negatives); strings are raw
UTF-8 (codepoint order — the "close enough without ICU collation"
trade-off most embedded databases take); a leading type tag keeps
different types from interleaving. This is what makes
`IndexRangeSeek` a contiguous key-range scan rather than a full-index
walk: `WHERE n.year > 2000` becomes byte bounds.

`apply_index_seeks` rewrites `Filter(n.prop = literal)` over
`NodeByLabelScan(n, Label)` into `IndexSeek` when `INDEX_DEFS` says an
index exists — and this is why the AST's narrow
`Compare(prop, op, literal)` shape from chapter 5 matters: the
rewrite is a pattern match on exactly that shape. The consumed
conjunct is removed and the *rest* of the predicate is rebuilt above
the seek. Range predicates rewrite similarly into `IndexRangeSeek`,
with one honesty rule: for numeric bounds the storage lookup returns
a *superset* (both int and float type regions, lossy conversions
widened outward), so the originating conjuncts always survive as a
residual `Filter` — the seek narrows, the filter remains the source
of truth.

## Choosing where to start

For a pattern like `MATCH (a:Common)-->(b:Rare {id: 1})`, compiling
left-to-right scans every `Common` node and expands — when starting
from the one indexed `Rare` node and walking adjacency *backwards*
touches only matching rows. Since `ADJ_IN` mirrors `ADJ_OUT`, the
plan is direction-symmetric and the choice of anchor is purely a cost
decision with identical results.

`plan_reversed_pattern` prices both anchorings with a two-sided
estimate built entirely from O(1) statistics — label counts, the
total node count, and the per-type edge counts maintained by the
write path:

```text
cost(anchor A, other B) = rows_A · (1 + filtered_A)
                        + E_A    · (1 + filtered_B)
E_A = E · rows_A / label_rows_A
```

The terms are the traversal's real work items: scan the anchor's
estimated rows; evaluate the anchor's pushed-down filter per row;
walk the anchor's share of the hop's edges (the type's total,
prorated by how much an index narrowed the anchor's label, under a
uniform-degree assumption); evaluate the far endpoint's stranded
filter once per walked edge. Row scans, filter evaluations, and edge
walks are weighted equally — not by assumption, but because the two
non-scan halves were measured at ~0.66 µs and ~0.65 µs per item on a
real dataset: the same order, so unit weights are honest. The
`filtered` flags credit pushable-but-unindexed predicate work
(a `CONTAINS`, a range, a `$param` equality) whose selectivity is
unknowable at plan time and is deliberately not guessed at. On the
benchmark shape that motivated the model — 9,125 movies filtered by
title `CONTAINS` against 671 users over 100k edges — the estimate
prices the written order at 118k work items versus 200k reversed,
and the written order is the measured 9x-faster execution.

Reversal fires only when the far endpoint prices *strictly* cheaper;
ties keep written order, for determinism and because reversal is
never free to reason about. And the pass disqualifies itself
entirely for any pattern with a variable-length hop, a named path,
or `shortestPath` — those expose traversal *order* to the user, and
a rewrite that changes observable order is not answer-preserving.

## The third strategy: sweep the edges

Some single-hop shapes defeat both anchorings — bulk operations like
`MATCH (a)-[r:RATED]->(b) WHERE r.rating < 2 DELETE r`, where any
node-side anchor walks enormous adjacency with a per-edge storage
get. `plan_edge_scan` prices a third option: one sequential sweep of
the `EDGES` table, evaluating the relationship predicate directly
from each swept record's bytes — no adjacency, no per-edge point
gets. Sequential-versus-random is the whole story: a warm sweep of
166k edge records, per-record predicate decode included, measured
~5–6 ms against ~110 ms for the same edges through per-edge
adjacency gets.

Eligibility is the planner's conservatism at its most explicit:
exactly one fixed hop, a written direction, all three variables fresh,
at most one label per endpoint, and at least one conjunct on the
relationship variable that the sweep can decide from raw bytes with a
*definite* answer. That last clause has a three-valued-logic subtlety:
`NOT` is admitted only over `IS NULL`, because negating a comparison
whose unknown collapsed to false would flip unknowns to true — which
Cypher forbids. The cost gate compares the O(1) edge count against
the best anchored estimate; everything the sweep cannot decide stays
in a residual `Filter` above.

## EXPLAIN

`EXPLAIN <statement>` runs the frontend and both planning phases —
inside a real transaction, so index checks and statistics behave
exactly as execution would — and renders the operator tree instead of
executing it. The payoff is seeing whether an `IndexSeek` fired and
which residual `Filter` survived. Clause kinds that never compile to
a `LogicalPlan` (`UNWIND`, `WITH`, `CREATE` — row operations with no
traversal shape, and `MERGE`, whose match-half plan depends on each
row's own bindings) print one-line labels, with binding scope still
threaded through them so a `MATCH` after them explains with the right
`Seed`-versus-scan choice.

The next chapter is the executor: what actually happens when this
tree runs.
