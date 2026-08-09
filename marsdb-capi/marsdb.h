/* C ABI for MarsDB. Hand-written (this repo doesn't use cbindgen anywhere
 * else) — kept in lockstep with the #[no_mangle] functions in src/lib.rs
 * by hand; if you add/change a function there, update this file too. */
#ifndef MARSDB_CAPI_H
#define MARSDB_CAPI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MarsdbDatabase MarsdbDatabase;

/* Exactly one of `json`/`error` is non-null. Both, when non-null, must be
 * released with marsdb_free_string — never with free(). */
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

/* Run one Cypher statement with $name placeholders resolved from
 * params_json, a JSON object mapping parameter names to values:
 *   {"name": "Alice", "age": 42, "tags": [1, 2]}
 * Same result contract as marsdb_execute. NULL params_json means no
 * parameters. JSON numbers: integral -> i64 (full 64-bit range
 * preserved), fractional -> f64; a number outside both exact ranges is
 * an error, never a silent precision loss. Arrays/objects become Cypher
 * list/map parameter values. */
MarsdbResult marsdb_execute_with_params(MarsdbDatabase *db, const char *cypher,
                                        const char *params_json);

/* marsdb_execute_with_params plus execution bounds, both checked DURING
 * evaluation (a runaway query fails at the bound instead of
 * materializing an unbounded result first). max_rows caps result rows,
 * timeout_ms caps wall time; 0 means unlimited for either. */
MarsdbResult marsdb_execute_ex(MarsdbDatabase *db, const char *cypher,
                               const char *params_json, uint64_t max_rows,
                               uint64_t timeout_ms);

/* Frees a string returned in MarsdbResult.json or MarsdbResult.error.
 * Required — these are allocated by Rust's global allocator, not malloc. */
void marsdb_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* MARSDB_CAPI_H */
