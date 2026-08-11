# Graph Encoding

The previous chapter placed thirteen tables in a file; this one opens
the bytes inside them. The material lives in `marsdb-graph`: the value
model (`model.rs`), the record encoding (`encode.rs`), name interning
(`labels.rs`, `props.rs`), and id allocation (`id.rs`).

## The value model

`PropertyValue` is both the runtime scalar type and the persisted one —
there is no separate wire representation to keep in sync. Its variants:
`Null`, `Bool`, `Int` (i64), `Float` (f64), `String`, six temporal
types, homogeneous `List`s, and a `Map` variant that exists solely so
map-shaped *query parameters* can travel through the system — Cypher
forbids storing a map as a property, and the query layer rejects it
before anything reaches storage.

Values serialize with [postcard](https://docs.rs/postcard), a compact
serde format (varint integers, no field names). That choice carries a
one-way compatibility rule: postcard encodes an enum
discriminant by *declaration order*, so new `PropertyValue` variants
append at the end, and existing ones are never reordered or removed —
otherwise every already-stored property silently decodes as the wrong
variant. The enum's declaration order is on-disk ABI.

The temporal variants show a design rule for storing typed values in a
dynamically typed system: **the type must survive the storage
boundary**. A date could be stashed as `Int(epoch_day)` and would
round-trip fine — but then a stored integer and a stored date would be
indistinguishable on the way back out, and a value that must still
print, compare, and expose components *as a date* after a round trip
cannot afford that. So each temporal type is a first-class variant,
with representations chosen for comparison-by-integer rather than
library convenience:

- `Date` is days since the Unix epoch — an `i64`, not a date-library
  type, so the storage format cannot be broken by a dependency's
  internal change, and comparison is integer comparison. (`i64` rather
  than `i32` because Cypher's expanded year range ±999,999,999 reaches
  ±365 billion epoch days.)
- `Duration` is the four-component normalized form — months, days,
  seconds, nanos — because the components are *not* fungible: without a
  reference date, "3 months" has no fixed length in days, so collapsing
  `duration({months: 1})` and `duration({days: 30})` into one comparable
  scalar would be silently wrong the moment either is added to a date.
- `DateTime` stores the UTC instant plus the zone kept only for
  display; equality and ordering use the instant alone, so two
  `DateTime`s at the same instant in different zones compare equal —
  matching Cypher — even though they print differently. A named zone's
  offset is *not* cached (the same zone has different offsets across a
  DST transition); it is re-derived on demand.

Each of these is a small instance of the same discipline: pick the
representation that makes the *invariant* (comparison semantics,
normalization, range) structural, and push the presentation problems
(parsing, formatting, calendars) up into the query layer where they
belong.

## Interning: strings become integers

Labels and property names are interned: the first write that mentions
`Person` allocates a `u32` id for it and records the mapping in both
directions (`LABEL_TO_ID`/`ID_TO_LABEL`; property names identically in
their own pair). From then on, every record, adjacency key, and index
key speaks the id. Allocation happens inside the caller's write
transaction via the same counter mechanism as node and edge ids
(`id.rs::next_id`, a read-increment-write on the `META` table), which
means id allocation sits inside the statement's crash-safety boundary:
an aborted statement's freshly interned label vanishes with it, and no
committed state can reference an unallocated id.

The read direction has one subtlety. Interning tables are created
lazily by the first write that needs them, and a read transaction on a
never-written database finds the table *missing* rather than empty.
The lookup functions treat that specific error as "not found" — it is
exactly equivalent — rather than propagating it; the gap was found by
a real test (declaring an index on a never-used property), not by
speculation.

## The record encoding

A node record is not a serialized map. It is a **directory**:

```text
node:  [label_count: u8][label_id: u32 × n]
       [prop_count: u16]
       [(prop_id: u32, offset: u32) × m]     <- sorted by prop_id
       [values: postcard-encoded, packed in directory order]
edge:  [label_id: u32][src: u64][dst: u64]
       [prop_count / directory / values as above]
```

Offsets are relative to the values region, and value *i*'s length is
`offset[i+1] − offset[i]` (the last runs to the end) — which is why
values must be packed in directory order: the lengths are implied, not
stored.

The simpler alternative — serializing the whole name-keyed property map
as one postcard blob — was the first
implementation. The directory earns its complexity on the read path:
fetching *one* property from a record is a binary search over the
directory plus one postcard decode of just that value. No map is
built, no sibling property is touched, and no property-name string is
allocated — names appear nowhere in the record; ids resolve back to
names only when a caller actually needs the full name-keyed shape. The
difference is measured in this repository (an encoding-comparison
benchmark under `marsdb-storage/examples/`): **79x** faster for
reading 1 property of 20, and still **7x** faster even when fully
materializing every property. Single-property access is what executors
do constantly — every `WHERE n.age > 30` evaluation, every projection
of `n.name` — so this is the read path that matters.

Encode and decode take the interning and resolution functions as
closures rather than a transaction type, so the same functions serve
the write path (interning through the write context) and the read path
(resolving through a read snapshot) without the codec knowing either
exists. One practical detail with a measured justification: the
resolver closure holds its table handle open across an entire record's
worth of resolutions, because opening a redb table handle was itself a
measured hot cost — 23.67% of a bulk load, in a profile that predates
the fix — and a resolver that re-opened per property would reintroduce
exactly that.

## Adjacency keys, again

Chapter 2 described `ADJ_OUT`/`ADJ_IN`'s composite keys from the
table's point of view; `model.rs` holds the other half. `AdjEntry`
(edge id, other endpoint, label id) is the in-memory traversal
candidate — everything one hop needs, readable from an adjacency entry
alone without touching the `NODES` or `EDGES` tables. Two helper
functions define the prefix bounds: `adj_node_bounds(owner)` covers
every entry a node owns (the untyped expansion), and
`adj_label_bounds(owner, label)` covers one relationship type (the
typed expansion). All traversal in the executor ultimately bottoms out
in a range scan between one of these pairs of bounds.

The fixed-width warning from chapter 2 bears repeating from this side:
these keys are native `(u64, u32, u64)` tuples because redb keeps
fixed-width tuples in fixed slots. A byte-packed `[u8; 20]` encoding
of the same information — attractive for symmetry with other
byte-encoded keys — was measured at 2x total database file size.

## What is *not* here

Deletes remove records, adjacency entries, and index entries in the
same transaction, and there is no tombstone
or vacuum machinery; MVCC old versions are redb's concern, reclaimed by
its copy-on-write B-tree. Ids are never reused, which keeps "dangling
id" a state that only a bug (not a design feature) can produce — and
the integrity checker treats it accordingly.

The next chapter follows a write statement through `GraphStore` and
`WriteCtx` to see how records, adjacency, counters, and four kinds of
index entry are kept consistent inside one transaction.
