# C API

`marsdb-capi` is a small, hand-written C ABI (opaque handle + JSON
results) — the basis for non-Rust bindings like [Go](./go.md). Header:
[`marsdb-capi/marsdb.h`](https://github.com/knoguchi/marsdb/blob/main/marsdb-capi/marsdb.h).

```c
typedef struct MarsdbDatabase MarsdbDatabase;

/* Exactly one of `json`/`error` is non-null. Both, when non-null, must be
 * released with marsdb_free_string -- never with free(). */
typedef struct MarsdbResult {
    char *json;
    char *error;
} MarsdbResult;

/* Open (creating if absent) a single-file, on-disk database. Returns NULL
 * on failure (bad UTF-8 path, or the underlying open erroring). */
MarsdbDatabase *marsdb_open(const char *path);

/* Open a purely in-memory database. Nothing is written to disk. */
MarsdbDatabase *marsdb_open_in_memory(void);

/* Reclaims a handle from marsdb_open/marsdb_open_in_memory. NULL is a
 * no-op; double-close or closing a foreign pointer is undefined behavior. */
void marsdb_close(MarsdbDatabase *db);

/* Run one Cypher statement. Result JSON shape:
 *   {"columns": [...], "rows": [[...], ...]}
 * where each value is its natural JSON scalar, or for nodes/edges:
 *   {"__type": "node", "id": ..., "labels": [...], "props": {...}}
 *   {"__type": "edge", "id": ..., "label": ..., "src": ..., "dst": ..., "props": {...}}
 * Dates and durations use canonical ISO-8601 strings.
 */
MarsdbResult marsdb_execute(MarsdbDatabase *db, const char *cypher);

/* Frees a string returned in MarsdbResult.json or MarsdbResult.error.
 * Required -- these are allocated by Rust's global allocator, not malloc. */
void marsdb_free_string(char *s);
```

Build the shared/static library with `cargo build -p marsdb-capi` (add
`--release` for an optimized build) — produces `libmarsdb_capi.{dylib,so,a}`
under `target/{debug,release}/`.

This is intentionally minimal (`Open`/`Execute`/`Close`, JSON results) —
enough to build a real language binding on top, which is exactly what
[`marsdb-go`](./go.md) does. `execute_batch`/`execute_with_params` exist
on the Rust side but aren't exposed through this C ABI yet.
