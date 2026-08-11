# Design Overview

MarsDB is an embeddable property-graph database: a Rust library that
runs inside your process, stores an entire graph in a single file (or
purely in memory), and answers openCypher queries. There is no server,
no network protocol, and no background thread — every piece of work the
database does happens inside some caller's function call. If you have
used SQLite, the shape is familiar; MarsDB applies it to the property
graph model: nodes with labels and properties, directed typed
relationships with properties, and a query language built around
pattern matching.

This chapter walks the whole system once at low magnification: the
crate stack, the life of a single statement, and the transaction model.
Every later chapter zooms into one region of this map.

## The crate stack

MarsDB is a small workspace of layered crates. Each layer talks only to
the one below it:

```
marsdb-cli       the `mars` binary: REPL and one-shot execution
marsdb           public embedding API: Database, Transaction, sessions
marsdb-query     Cypher text -> AST -> logical plan -> executor
marsdb-graph     property-graph model: records, encoding, indexes, CRUD
marsdb-storage   thin boundary over the embedded KV engine (redb)
```

Two observations about this stack shape most of what follows.

**The storage engine is bought, not built.** `marsdb-storage` wraps
[redb](https://github.com/cberner/redb), a pure-Rust single-file
embedded key-value store with ACID transactions, MVCC snapshots, and a
B-tree file format. Writing a durable, crash-safe storage engine is a
multi-year project in its own right, and it is the layer where bugs are
least acceptable and hardest to find. Building on a proven engine lets
the interesting graph-database work — encoding, planning, execution —
sit on a foundation whose fsync discipline someone else has already
debugged. The cost is accepting redb's constraints, the most important
being its concurrency model: any number of concurrent readers, but only
one writer at a time, process-wide. That single-writer rule echoes
through the entire design, from the session layer down to lock-ordering
comments in the executor. `marsdb-graph` never imports redb directly;
it goes through `marsdb-storage`'s small trait boundary, which keeps
the dependency surface explicit and would localize the damage if the
engine ever had to change.

**The query language is compiled to an engine-shaped IR, not
interpreted off the AST.** `marsdb-query` parses Cypher (an ANTLR
grammar generating a parse tree, visited into an AST), validates it,
and lowers each `MATCH` pattern into a small tree of logical operators
— `AllNodesScan`, `NodeByLabelScan`, `IndexSeek`, `IndexRangeSeek`,
`EdgeTypeScan`, `Seed`, `Expand`, `VarExpand`, `Filter` — that the
executor evaluates against storage. The operators are
traversal-shaped rather than Cypher-shaped on purpose: they describe
*how to walk the graph*, not *what the query text said*, which is what
lets the planner substitute one access path for another (a label scan
for an index seek, an anchored expansion for an edge sweep) without
the executor caring where the plan came from.

## The life of a statement

Everything the database does is reachable from one entry point:
`Database::execute` (and its variants taking parameters and options) in
`marsdb/src/lib.rs`. Tracing one call end to end touches every layer:

1. **Parse.** The Cypher text goes through the generated parser and the
   AST-building visitor. A syntax error stops here; nothing has touched
   storage yet.

2. **Substitute parameters.** `$name` placeholders are replaced in the
   AST from the caller's parameter map. This is structural substitution,
   not string splicing — a parameter value can never change the shape of
   the query, which is why parameterized queries are immune to
   injection by construction.

3. **Route through the session layer.** The `Database` handle checks
   whether a Cypher-level `BEGIN` transaction is currently open on it.
   If so, the statement runs inside that transaction (and sees its
   uncommitted writes). If not, the statement autocommits: it gets a
   transaction of its own for exactly its own duration. The lock
   guarding this check is released before autocommit execution begins,
   so concurrent readers on one handle actually run concurrently.

4. **Open the right kind of transaction.** The executor classifies the
   statement: a read-only shape (`MATCH ... RETURN` with no write
   clause) opens a redb *read* transaction — a consistent MVCC snapshot
   that coexists with other readers and at most one concurrent writer —
   while anything that writes opens the *write* transaction, of which
   redb allows exactly one at a time.

5. **Plan.** The pattern is lowered to the logical-operator tree, then
   rewritten with storage in view: filters over label scans become
   index seeks where a declared index exists, start points may be
   reversed based on cost statistics, predicates and limits are pushed
   down toward the scans. Planning happens per execution, inside the
   statement's transaction, so the plan reflects the indexes and
   statistics that this exact snapshot can see.

6. **Evaluate.** The executor walks the operator tree, binding
   variables to nodes, relationships, and values row by row, then
   applies the statement's tail: projection, aggregation, ordering,
   limits, or the write clauses (`CREATE`, `SET`, `DELETE`, ...), which
   mutate storage through `marsdb-graph`'s CRUD layer inside the same
   transaction.

7. **Commit or abort.** Success commits — redb makes the whole
   statement's effects durable atomically. Any execution error aborts
   the transaction: a statement that fails halfway leaves no trace.
   This all-or-nothing boundary per statement is the database's basic
   crash-safety contract, and it holds for a statement that created
   three thousand nodes just as for one that created one.

The important property of this pipeline is where the boundaries sit.
Parse and parameter errors happen before any transaction exists;
planning happens inside the transaction so it can trust its snapshot;
and a single statement is always exactly one atomic unit unless the
caller has explicitly said otherwise — which brings us to the
transaction model.

## The transaction model

MarsDB exposes one storage-level reality — redb's
single-writer/many-readers MVCC — through three caller-facing forms,
all defined in `marsdb/src/lib.rs`:

**Autocommit** is the default described above: one statement, one
transaction. There is nothing to configure and no way to observe a
half-applied statement.

**Caller-owned transactions** (`Database::begin_transaction`) return a
`Transaction` handle owning a write transaction across multiple
`execute` calls, committed or rolled back explicitly. Reads through the
handle see the transaction's own uncommitted writes. Any statement
error aborts the whole transaction immediately rather than leaving it
open — a deliberate stance: a failed statement may have applied partial
effects before failing, and those must never be committable. The error
handling makes the invalid state unrepresentable instead of trusting
every caller to remember a rollback.

**Session transactions** are the same idea driven from *inside* the
query language: `BEGIN`, `COMMIT`, and `ROLLBACK` as ordinary
statements (an extension — openCypher itself has no transaction
statements). This is what makes transactions usable from the CLI and
from language bindings that only have an "execute string" API. Each
`Database` handle is one session; `BEGIN` opens a write transaction
that subsequent statements run inside until `COMMIT` or `ROLLBACK`.
The same abort-on-execution-error stance applies, with one carve-out:
a statement that never *ran* — a parse error, a missing parameter —
leaves the transaction open, because nothing was applied and killing an
interactive session's transaction over a typo helps nobody.

The session form carries a sharp edge that follows directly from the
single-writer rule: an open session transaction *is* the process's one
write slot, so an abandoned one blocks every other writer forever —
redb's `begin_write` blocks rather than erroring. MarsDB mitigates
this with an optional idle timeout, and the mechanism is worth noticing
because it exemplifies a design rule used throughout: **no background
threads**. Expiry is checked lazily by the next statement to arrive on
the session, not by a reaper thread. The database never does work
outside a caller's call stack, which keeps the embedding story simple
(no shutdown ordering, no thread-safety obligations imposed on the
host process) at the cost of "next statement" being the soonest an
expired transaction can actually be reclaimed.

Batches compose with all of this. `execute_batch` runs a
semicolon-separated script with the whole batch parsed up front (a
syntax error anywhere means nothing runs), one transaction per
statement — unless the script itself says `BEGIN ... COMMIT`, which
works exactly as it does interactively. `execute_batch_grouped` trades
crash-safety granularity for load throughput by committing once per
group of statements rather than per statement. The trade is real and
measured: each commit is an fsync, and on a 9,771-statement load
script, per-statement commits took 69.1 s while groups of 100 took
13.4 s — and committing the entire script as one group only improved
that to 12.1 s. Most of the win arrives by a few hundred statements
per group; the numbers, not intuition, are what capped the recommended
group size.

## Design principles worth naming

Three habits recur so often in this codebase that later chapters will
mostly be showing you instances of them.

**Errors are loud, invariants are enforced by construction.** Where an
invariant holds by design — an index entry that cannot dangle because
the only two functions that touch the indexed table maintain the index
in the same transaction — the code panics if it is ever violated,
rather than silently repairing state that "should be impossible."
Defensive code for unreachable states hides bugs; a panic surfaces
them.

**Performance claims require measurements.** Every trade-off described
in this book — index maintenance cost versus scan speedup, group-commit
throughput, access-path selection — is documented with numbers from
the repository's benchmark suite, measured on stated workloads. Several
design decisions went *against* the initially attractive option because
the measurement said so; the closing case-studies chapter collects
these, including an optimization that was fully built, measured at ~2%,
and deleted.

**The database describes itself.** The planner's choices are
observable (`EXPLAIN`), the schema is introspectable (`CALL
db.labels()` and friends), and execution is bounded and observable
(row limits, timeouts, cancellation, an observer hook). A database you
cannot interrogate is a database you debug by superstition.

With the map established, the next chapter starts at the bottom of the
stack: what is actually in the file.
