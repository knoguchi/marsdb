# Results and Language Boundaries

A query's answer has to leave the engine — into Rust structs, C
callers, Python objects, Go maps, and Arrow consumers. This chapter
covers the result model (`marsdb-query/src/{result,value}.rs`), the C
ABI (`marsdb-capi`), and the Arrow export (`marsdb/src/arrow.rs`),
with an eye on the theme that shapes all of them: every boundary
crossing has a cost, and the design goal is to pay each cost once.

## The result model

`QueryResult` is columns, rows of `Value`s, and `QueryStats` — the
per-statement write counters (nodes and relationships created and
deleted, properties set, labels added and removed) that answer "how
many did my `DELETE` delete," which a result's rows alone cannot.
Following the conventions of the wider graph-database ecosystem,
removing a property counts as setting it (`SET n.p = null` and
`REMOVE n.p` are literally the same operation internally).

`Value` is the query-layer value type, a superset of the storable
`PropertyValue`: it adds whole nodes and edges, paths, and the
list/map shapes that exist only at query time. The split is
load-bearing. A path is a single alternating `node, edge, ..., node`
vector — not parallel node and edge vectors, which would create an
unenforced length invariant across every construction site. A map
value can be projected and returned but never stored: the conversion
to a storable property rejects it, keeping Cypher's "no map
properties" rule enforced at one chokepoint rather than by every
write path.

## The C ABI

`marsdb-capi` compiles to a `cdylib`/`staticlib` exposing a C API in
the SQLite shape: opaque handles (`MarsdbDatabase`, statement, result),
integer status codes, and a per-handle `last_error` string. The header,
`marsdb.h`, is the documentation of record; the Rust file's job is to
keep the header's promises. Three invariants organize the unsafe code:

- **No panic crosses the boundary.** Every entry point that runs
  engine code wraps it in `catch_unwind` — a Rust panic unwinding into
  a C caller is undefined behavior, so the boundary converts panics
  into error returns.
- **Handles have documented lifetimes.** Value handles point into a
  result's rows, which are never mutated after construction;
  advancing to the next row invalidates them by contract (the SQLite
  convention), and the implementation's per-row arenas make the
  contract cheap.
- **Errors are pulled, not pushed.** Calls return status codes;
  `marsdb_last_error` returns the message. The database handle's
  error slot is behind a mutex because the header makes no
  single-caller promise for it.

For bulk results the ABI offers a **binary batch lane**: one call
returns an entire result as a compact self-describing buffer —
interned column and property names, varint integers — that a binding
decodes in its own language. One boundary crossing per *query* rather
than per value: the per-call FFI tax (marshalling, error checking,
in some runtimes lock acquisition) is paid once. A streaming callback
lane covers the opposite shape — unbounded exports under bounded
memory — pushing one row per callback through the executor's
streaming path.

Bindings layer on top: the Python binding (`marsdb-python`, PyO3)
links the engine directly into the interpreter process, and the Go
binding ([marsdb-go](https://github.com/knoguchi/marsdb-go), its own
repository) consumes this C ABI through cgo. Both follow the same
batch-lane strategy for results; both expose the same execution-bounds
and transaction surface.

## Arrow: the columnar boundary

For analytical consumers — dataframes, columnar compute — the row
result is the wrong shape, and per-value conversion is the wrong
cost. MarsDB's answer is a core-owned Arrow export
(`Database::query_arrow`, behind the opt-in `arrow` cargo feature):
the row-to-column transpose happens exactly once, in the engine, and
everything downstream of it is zero-copy:

- In Rust, a standard `RecordBatchReader`.
- Across C, the Arrow **C Data Interface** stream exported by
  `marsdb-capi` (`marsdb_stmt_execute_arrow`) — Arrow's ABI for
  handing ownership of column buffers across a language boundary
  without serialization.
- In Python, the PyCapsule protocol: any Arrow-aware library imports
  the stream directly.
- In Go, arrow-go's `cdata` import wraps the same stream.

The measured effect, from the Go binding on a 200k-row, three-column
result: the batch lane performs ~1.2 million binding-side allocations
(83 MB) per query; the Arrow lane performs ~900 (83 KB) — wall time
at parity, because engine execution dominates both, but three orders
of magnitude less allocator and GC pressure. The columnar boundary's
win is the allocation profile, and it grows as the engine's own share
of the time shrinks.

Column typing is strict, and deliberately so. Cypher columns are
dynamically typed, so the exporter infers each column's type over the
whole result — which is also why the export materializes before the
first batch is handed out: inference needs every row. The rules
mirror the precision discipline used everywhere else in the system:
integers export as `Int64` exactly; a column mixing integers and
floats is an **error**, not a silent promotion to `Float64` (which
corrupts integers beyond 2^53); dates are `Date32`, durations are
`Interval(MonthDayNano)`, other temporals are canonical ISO text;
homogeneous lists nest as `List<child>`; nulls become Arrow validity
bits, so a column stays typed by its non-null values. Node, edge,
map, and path columns are errors with an instruction — project scalar
properties instead — because flattening an entity into strings would
discard exactly the structure an analytical consumer would then have
to re-parse.

One dependency-hygiene note: the crate re-exports its arrow-rs types
for `marsdb-capi` and `marsdb-python` to consume, so no downstream
crate declares its own arrow-rs dependency and no version split can
produce two incompatible definitions of the same C structs.

The remaining chapters step back from the pipeline: how this system
is tested and measured, and what the measurements changed.
