# Case Studies in Measured Trade-offs

Every chapter so far has cited measurements in passing. This one
collects the decisions those measurements *drove* — features kept or
removed and intuitions overturned — because together they reveal this
codebase's engineering methodology:

> Knowing the mechanism tells you the *direction* of an effect.
> Only measuring tells you the *magnitude* — and magnitude is what
> decides whether the complexity pays.

## The label index: a trade, not a win

The label index (`label_id -> node_ids`) turns a label-filtered scan
into an index lookup plus one point-get per match. At 1% selectivity
it is roughly 30–80x faster than the full scan (801 µs versus ~7 ms
at comparable sizes, staying near-flat per matching row as the table
grows). The same benchmark run recorded the costs: every
`create_node` pays an extra index write, and a scan whose label
matches *every* row is slower through the index than a single
sequential pass — N point-gets lose to one sweep when N is the whole
table. The index shipped because the common query shape benefits from
it, while `BENCHMARKS.md` records both the read benefit and the write
and full-scan costs.

An automated review bot later suggested making the index's read path
defensive — silently skipping index entries pointing at missing
nodes. The suggestion was rejected on invariant grounds (chapter 4:
the entry cannot dangle by construction, and if it ever does, that is
corruption that should produce an error rather than be silently
ignored).

## Fixed-width keys: the erasure tax

Chapter 2's adjacency layout depends on a result that was not apparent
from the design alone: encoding the composite key as a packed byte string
instead of a native fixed-width tuple *doubled* the database file in
one measurement, and a related erasure to `&[u8]` measured +34% —
redb keeps fixed-width tuple keys in fixed slots, and byte-erasing
them forfeits that packing. Measurement showed that byte erasure
disabled redb's fixed-slot packing.
The first cut of the composite-key change made exactly this mistake,
and the number is what sent it back.

## The record directory: optimize the read you actually do

The directory encoding (chapter 3) beat whole-record decoding by 79x
for reading one property of twenty — and by 7x even at full
materialization, which is the surprise half: the directory was
designed for partial reads, but eliminating per-property name
allocation and map construction won even the case the old format was
supposedly good at. The supporting measurement that shaped the
implementation: table-handle opens were 23.67% of a bulk load, which
is why decode resolvers hold one handle across a record rather than
opening per property.

## `WriteCtx`: the tidy version was slower

Caching table handles per write operation (chapter 4) had an eager
variant — open all thirteen handles up front — that was strictly
simpler and benchmarked *worse than the code it replaced*: 4.89 s to
6.35 s on the 9,771-statement load. Most operations touch a handful
of tables, and opening the unused handles cost more than the
redundant opens the cache saved. The lazy variant kept the win. The
lesson generalizes: an optimization's overhead lives on the same
axis as its savings, and only the measured difference says which
dominates.

## Start-point reversal: pricing the plan, then checking the price

The planner's anchor-cost model (chapter 6) exists because one query
shape — a huge filtered label against a small far endpoint — was 9x
faster in written order than reversed, while a naive
row-count-comparison rule reversed it. The model's unit weights are
themselves measured (~0.66 µs per filter evaluation, ~0.65 µs per
edge walk — the same order of magnitude, so equal weights are
reasonable), and its
verdict on the motivating query (118k work items written versus 200k
reversed) agrees with the observed 9x difference. A cost model is a
hypothesis; this one had to explain an existing measurement before it
was trusted.

## The edge sweep: sequential beats clever

`EdgeTypeScan` (chapter 6) earns its place with one comparison: a
warm sequential sweep of 166k edge records — per-record predicate
decode included — costs ~5–6 ms, where the same edges through
per-edge adjacency point lookups cost ~110 ms: a twenty-fold difference
from the access pattern alone. The operator's narrow eligibility rules
are the other
half of the lesson: a 20x mechanism is only worth having if every
shape it is *allowed* to run on provably preserves answers.

## Group commit: the knee is early

The grouped batch loader (chapter 1) commits every N statements. The
measurement that set the guidance: 69.1 s per-statement, 13.4 s at
groups of 100, 12.1 s committing the entire 9,771-statement script
once. The fsync amortization is nearly exhausted by a few hundred
statements per group — so the documentation recommends modest groups,
which also keep the crash-loss window small. Without the three-point
curve, the natural instinct ("bigger groups, faster load") would
trade durability granularity for a win that mostly is not there.

## The optimization that was removed

A row-representation optimization for the executor's binding rows was
fully implemented and benchmarked end to end — and moved the numbers
by roughly 2%. The profile said why: read cost in the affected
workloads is dominated by node decoding and traversal, not by the
representation of result rows. The change was reverted, not shipped
— carrying permanent complexity for two points is a losing trade —
and the measurement was kept: knowing where the cost *is not* is what
directed later work at the decode and traversal paths, where the
directory encoding and the edge sweep found their wins.

The optimization worked as designed, but its measured effect was too
small to justify the added complexity. A
database, like any long-lived system, is shaped as much by the
features it declined to keep as by the ones it kept.

## Neutral results get recorded too

The bulk edge-delete API (chapter 4) measured approximately *neutral*
on wall time — the cost of a scattered bulk delete lives in the match
phase, and a tried sort-into-per-table-passes variant moved nothing.
The function stayed, justified in its doc comment by API shape and
strictly less redundant work, with the neutral measurement stated
rather than implied away. Recording "this did not help" is what makes
the next engineer's search space smaller; a ledger that only lists
wins teaches nothing about where wins are not.

---

That closes the internals tour. The map, once more, in one breath: a
single redb file holding records, mirrored adjacency, and indexes; a
statement pipeline from ANTLR grammar through a validated AST to a
traversal-shaped plan, rewritten against real statistics inside its
own transaction; an executor that enforces Cypher's semantics under
cooperative bounds; results that cross each language boundary once;
supported by a conformance suite, a crash harness, and a record of
benchmark results used to evaluate design changes.
