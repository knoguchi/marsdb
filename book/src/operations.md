# Operations

Running MarsDB in practice — backup, integrity checks, crash safety, file
format compatibility, and concurrent access. These apply regardless of
which language you're calling from; see [Embedding in Rust](./embedding-rust.md),
[Python bindings](./python.md), [Go bindings](./go.md), or the [C API](./c-api.md)
for the exact call syntax in each.

## Backup

`Database::backup_to(path)` writes a transactionally consistent copy of
the database to `path`. The destination must not already exist — an
existing file is never overwritten, so a backup can't silently clobber
another database or a previous backup.

```rust
db.backup_to("path/to-backup.db")?;
```

## Integrity checks

`Database::check_integrity()` checks redb's physical storage and then
MarsDB's own logical graph invariants, returning an `IntegrityReport`:

```rust
let report = db.check_integrity()?;
```

| Field | Meaning |
| --- | --- |
| `physical_was_clean` | `false` means redb detected physical damage and repaired it before MarsDB's logical checks ran |
| `labels` | number of distinct labels |
| `nodes` | number of nodes |
| `edges` | number of edges |

`physical_was_clean: false` is worth treating as a signal, not just a
statistic — it means the underlying file had damage severe enough for
redb to notice and fix. The logical counts in the same report are only
meaningful once the physical layer is sound, which is why the physical
check runs first. `check_integrity` needs exclusive access — no other
transaction can be open on the database while it runs.

## Crash safety

Every Cypher statement runs inside one transaction, committed atomically.
Storage runs on [redb](https://github.com/cberner/redb), a pure-Rust MVCC
single-file engine. A dedicated test harness (`marsdb-crash-harness`, a
development tool, not something you run in production) SIGKILLs a
MarsDB process mid-workload at unpredictable points and verifies on
reopen that every commit the process had acknowledged actually survived.
This is a process-crash check — the OS stays up and the page cache stays
intact — not a power-loss test; genuine power-loss testing needs fault
injection at the block-device layer (e.g. `dm-flakey`).

## Storage format versioning

MarsDB records its own record/table format version inside the database
file, separate from redb's own file format. Opening a file written by an
unsupported format version — either an old pre-versioning file or one
written by a newer, incompatible version of MarsDB — fails cleanly with
an error at open time. MarsDB never silently reinterprets a file it
doesn't understand as if it were the current format.

## Concurrent access

A read-only `MATCH ... RETURN` statement opens a redb `ReadTransaction`,
which runs alongside any number of other concurrent readers and a
concurrent writer without contending for redb's single-writer lock.
Every other statement — any write, or a Cypher-level `BEGIN` session —
opens a `WriteTransaction`. Standard MVCC: one writer at a time, readers
never block on it and never block each other.

## Session transactions and idle timeout

MarsDB's Cypher `BEGIN` / `COMMIT` / `ROLLBACK` statements (a MarsDB
extension — openCypher itself has no transaction statements) open a
session-level write transaction on the `Database` handle. Every
statement executed on that handle after `BEGIN` runs inside it, reads
included, until `COMMIT` or `ROLLBACK` closes it.

An open write transaction holds redb's single writer. If a `BEGIN` is
left open — the caller forgets to `COMMIT`, or the connection is
dropped without closing it — every other writer on that database blocks
forever: redb's `begin_write` blocks rather than erroring out. This
applies to caller-owned transactions too (`begin_transaction` in Rust,
or the equivalent in other bindings), not just session transactions.

`Database::set_session_transaction_timeout(Some(duration))` mitigates
this for session transactions: once a session transaction has sat idle
longer than the configured limit, the *next* statement that arrives on
that handle rolls it back and returns a timeout error, instead of
running normally. There's no background timer — an abandoned
transaction with no further traffic on the handle keeps holding the
writer lock until another statement actually shows up. If you mix
session transactions with caller-owned transaction handles across
threads, expect the reclaim to happen on that next statement, not on a
clock tick. The timeout is disabled (`None`) by default.
