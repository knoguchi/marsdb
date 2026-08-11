# The Write Path

A single `CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b)` touches nine
of the thirteen tables: two records, two adjacency mirrors, a
statistics counter, up to four interning entries, a label-index entry
per label, and any property-index entries the labels' declared indexes
require. This chapter is about how `marsdb-graph` keeps all of that
consistent without ceremony: `store.rs` (the CRUD layer) and
`write_ctx.rs` (the table-handle cache that every write rides on).

## Three layers per operation

Every mutating operation exists in the same three forms, e.g. for node
creation:

- `create_node` — public convenience: opens a write transaction, calls
  the next layer, commits. One operation, one transaction.
- `create_node_in_txn` — takes a caller-supplied `&WriteTransaction`.
  This is what the query executor calls: the executor owns one
  transaction per statement, and every graph operation the statement
  performs flows through it.
- `create_node_ctx` — internal, takes `&mut WriteCtx`. This is where
  the actual work lives, and it is the composition point: an operation
  that needs another operation calls the `_ctx` form directly so both
  share one set of table handles.

The layering answers a real constraint, not a style preference. redb
errors at runtime (`TableAlreadyOpen`) if one write transaction holds
two live handles to the same table. Deleting a node must delete its
incident edges; if node-deletion called the public edge-deletion
wrapper, each call would open a fresh set of handles on the same
transaction and collide with the ones node-deletion already holds. The
`_ctx` layer exists so compound operations compose *inside* one
handle set.

## `WriteCtx`: lazy handles, measured

`WriteCtx` is a struct of thirteen `Option<Table>` fields with
accessor methods: first access opens the handle, later accesses reuse
it. Two decisions here were made by measurement, and both went against
the initially plausible option:

**Lazy, not eager.** Opening all thirteen handles up front sounds
tidier. Measured against a real 9,771-statement bulk load it was
*slower* than the pre-`WriteCtx` code (4.89 s → 6.35 s): most calls
touch a handful of tables (`set_edge_prop_in_txn` needs exactly one),
and eagerly opening the other unused handles costs more than the
redundant opens it was meant to eliminate. Lazy access means a call
pays only for the tables it uses, while still collapsing the *repeat*
opens a single call used to perform — node creation previously opened
`NODES` once, the label index once per label, and then the
property-index hook re-opened four more tables on top. Table opens are
not noise: a profile of that same bulk load attributed 23.67% of total
time to them.

**Scoped to one operation, not one transaction.** Stretching the cache
across a whole statement or transaction would save even more opens —
but any *read* that happens while a write is in flight (property
lookups in a `WHERE`, subquery evaluation) would then need to route
through the same cached handles, or it hits the very
`TableAlreadyOpen` the cache exists to avoid. That is a redesign of
the read-write interleaving across two crates, and the cache's scope
stops where the contained change stops. Knowing where to stop is
itself a design decision, and this one is documented in the module
header rather than left for the next person to rediscover.

## Anatomy of the mutations

**Creating a node**: intern each label, allocate the id (a counter
bump in `META` — durable only when the caller commits, so id
allocation sits inside the same crash-safety boundary as the record
it names), encode the record (interning property names as a side
effect), insert into `NODES`, add one `NODE_LABEL_INDEX` entry per
label, and hand the new node to the property-index hook
(`index::on_node_created`) which adds entries for any `(label, prop)`
pair with a declared index.

**Creating an edge**: verify both endpoints exist (the only
referential check the write path needs, since ids are never reused),
intern the type, allocate the id, insert the record, then the two
mirror-image adjacency entries — `(src, label, edge) → dst` in
`ADJ_OUT` and `(dst, label, edge) → src` in `ADJ_IN` — and bump the
type's edge count.

**Deleting an edge** is the reverse, with a detail that shows the
directory encoding paying off on the write path too: cleanup needs
only the header — type, src, dst — to compute the two adjacency keys,
so it reads exactly those bytes and never decodes properties or
resolves a property name.

**Deleting a node** ranges over both adjacency tables with the node's
prefix bounds to collect incident edges. Non-detach deletion with
incident edges refuses with a typed error before touching anything.
`DETACH DELETE` deletes each incident edge through the shared `_ctx`
path, then the record, the label-index entries, and the
property-index entries. The reported edge count comes from the
*deletions*, not the scan — a self-loop appears in both adjacency
directions but deletes once, and the statement statistics must say
one.

**The statistics counter** (`REL_TYPE_COUNTS`) is bumped in exactly
two places — edge birth and edge death — and *saturates* rather than
panics on the way down. That asymmetry with the loud-panic policy for
index invariants is deliberate and principled: the counter is a
planner statistic, a wrong value costs a suboptimal plan but never a
wrong answer, so it must degrade to a wrong estimate rather than take
the database down. Loudness is proportional to what the invariant
protects.

**Bulk deletion** (`delete_edges_in_txn`) exists for a `DELETE r`
statement's whole edge set: one `WriteCtx` across every id, label
names resolved once per distinct type rather than once per edge. It
measured roughly neutral on wall time — a scattered bulk delete's cost
lives in the executor's match phase, not here, and a tried
sort-ids-into-per-table-passes variant moved nothing — so the honest
justification recorded in its doc comment is API shape and strictly
less redundant work, not a claimed speedup.

## The integrity checker is the invariant spec

Reading `check_integrity` back to back with the mutations above is the
fastest way to internalize the write path's contract, because the
checker is the invariants written down as executable prose:

- both interning tables are exact inverses, with equal entry counts;
- every label id referenced by any node or edge record resolves;
- every label-index entry points at a live node that carries the label
  — and every node's every label has its index entry (both
  directions);
- every edge's endpoints are live nodes;
- every edge appears in both adjacency mirrors under the right key —
  and every adjacency entry corresponds to a live edge with matching
  header;
- id counters are at or above the maximum allocated id.

Note what the checker reads: node and edge *headers* only, never
properties — the same header-only decode the delete path uses. And
note its stance: any violation is `CorruptData`, an error, not a
repair. The write path maintains these invariants by construction —
every mutation and its index bookkeeping share one transaction — so a
violation means a bug, and the checker's job is to say so, not to
paper over it.

Property indexes — declaration, backfill, uniqueness, and the
order-preserving key encoding that makes range seeks work — get their
full treatment alongside the planner in chapter 6. The next chapter
climbs into `marsdb-query` at the top: how Cypher text becomes a
validated AST.
