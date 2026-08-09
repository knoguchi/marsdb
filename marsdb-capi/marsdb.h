/* C ABI for MarsDB, v2: typed opaque handles (the SQLite shape) plus a
 * binary batch lane. Hand-written, kept in lockstep with the #[no_mangle]
 * functions in src/lib.rs by hand; if you add/change a function there,
 * update this file too.
 *
 * Two tiers:
 *   1. Typed handles — prepare/bind/execute, step a cursor, read values
 *      through typed accessors. The primary, freezable ABI.
 *   2. Batch — one call returns the whole result as a compact
 *      self-describing binary blob (format below). One FFI crossing per
 *      query; intended for bindings whose per-call FFI cost dominates
 *      (Go) and for scripting consumers.
 *
 * Error model: fallible functions return int (0 = MARSDB_OK, 1 =
 * MARSDB_ERROR) and record a message retrievable via marsdb_last_error.
 *
 * Lifetime rules (sqlite-style):
 *   - const char* / const MarsdbValue* obtained from a MarsdbResult are
 *     valid only until the next marsdb_next() on that result (or its
 *     destroy). Copy anything you keep.
 *   - marsdb_last_error's string is valid until the next failing call
 *     on the same database handle.
 *   - A MarsdbStatement must not outlive the database it was prepared
 *     against.
 */
#ifndef MARSDB_CAPI_H
#define MARSDB_CAPI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MARSDB_OK 0
#define MARSDB_ERROR 1

typedef struct MarsdbDatabase MarsdbDatabase;
typedef struct MarsdbStatement MarsdbStatement;
typedef struct MarsdbResult MarsdbResult;
typedef struct MarsdbValue MarsdbValue;

typedef enum MarsdbValueType {
    MARSDB_TYPE_NULL = 0,
    MARSDB_TYPE_BOOL = 1,
    MARSDB_TYPE_INT64 = 2,
    MARSDB_TYPE_FLOAT64 = 3,
    MARSDB_TYPE_STRING = 4,
    MARSDB_TYPE_DATE = 5,     /* value_string gives ISO-8601 text */
    MARSDB_TYPE_DURATION = 6, /* value_string gives ISO-8601 text */
    MARSDB_TYPE_NODE = 7,
    MARSDB_TYPE_EDGE = 8,
    MARSDB_TYPE_LIST = 9,
    MARSDB_TYPE_MAP = 10,
    MARSDB_TYPE_PATH = 11,
} MarsdbValueType;

/* What a write statement changed — all zero for reads. properties_set
 * counts removals too; label changes are their own pair. */
typedef struct MarsdbQueryStats {
    uint64_t nodes_created;
    uint64_t nodes_deleted;
    uint64_t relationships_created;
    uint64_t relationships_deleted;
    uint64_t properties_set;
    uint64_t labels_added;
    uint64_t labels_removed;
} MarsdbQueryStats;

/* Owned byte buffer returned by the batch calls; release with
 * marsdb_buffer_free exactly once. data is NULL on error. */
typedef struct MarsdbBuffer {
    uint8_t *data;
    size_t len;
} MarsdbBuffer;

/* ---- lifecycle ------------------------------------------------------ */

/* Open (creating if absent) a single-file, on-disk database. NULL on
 * failure (bad UTF-8 path or underlying open error). */
MarsdbDatabase *marsdb_open(const char *path);

/* Open a purely in-memory database. Nothing is written to disk. */
MarsdbDatabase *marsdb_open_in_memory(void);

/* Reclaims a handle. NULL is a no-op; double-close is undefined
 * behavior. Destroy all statements/results first. */
void marsdb_close(MarsdbDatabase *db);

/* Message for the most recent failing call on this handle, or "" if
 * none. Borrowed: valid until the next failing call on the handle. */
const char *marsdb_last_error(const MarsdbDatabase *db);

/* ---- prepared statements -------------------------------------------- */

/* Parse once; execute many. Binds persist across executions until
 * marsdb_clear_bindings. */
int marsdb_prepare(MarsdbDatabase *db, const char *cypher, MarsdbStatement **out);
int marsdb_bind_int64(MarsdbStatement *stmt, const char *name, int64_t value);
int marsdb_bind_double(MarsdbStatement *stmt, const char *name, double value);
int marsdb_bind_bool(MarsdbStatement *stmt, const char *name, int value);
int marsdb_bind_string(MarsdbStatement *stmt, const char *name, const char *value);
int marsdb_bind_null(MarsdbStatement *stmt, const char *name);
/* Flat list parameters (`WHERE x IN $list`). Nested containers are not
 * bindable through the C ABI. */
int marsdb_bind_int64_list(MarsdbStatement *stmt, const char *name,
                           const int64_t *values, size_t len);
int marsdb_bind_double_list(MarsdbStatement *stmt, const char *name,
                            const double *values, size_t len);
int marsdb_bind_string_list(MarsdbStatement *stmt, const char *name,
                            const char *const *values, size_t len);
void marsdb_clear_bindings(MarsdbStatement *stmt);

/* Execution bounds, checked DURING evaluation (a runaway query fails at
 * the bound instead of materializing an unbounded result). 0 = unlimited. */
void marsdb_stmt_set_max_rows(MarsdbStatement *stmt, uint64_t max_rows);
void marsdb_stmt_set_timeout_ms(MarsdbStatement *stmt, uint64_t timeout_ms);

int marsdb_stmt_execute(MarsdbStatement *stmt, MarsdbResult **out);
void marsdb_stmt_destroy(MarsdbStatement *stmt);

/* ---- one-shot queries ----------------------------------------------- */

int marsdb_query(MarsdbDatabase *db, const char *cypher, MarsdbResult **out);

/* ---- typed results -------------------------------------------------- */

size_t marsdb_column_count(const MarsdbResult *result);
/* Borrowed until result destroy. i out of range returns NULL. */
const char *marsdb_column_name(const MarsdbResult *result, size_t i);

/* Advance the row cursor. Returns 1 while a row is available, 0 at the
 * end. Invalidates every pointer previously obtained from this result. */
int marsdb_next(MarsdbResult *result);

MarsdbQueryStats marsdb_result_stats(const MarsdbResult *result);
void marsdb_result_destroy(MarsdbResult *result);

/* Value of the current row's column `col`. NULL if no current row or
 * col out of range. Valid until the next marsdb_next/destroy. */
const MarsdbValue *marsdb_row_value(const MarsdbResult *result, size_t col);

MarsdbValueType marsdb_value_type(const MarsdbValue *value);
int64_t marsdb_value_int64(const MarsdbValue *value); /* 0 if not INT64 */
double marsdb_value_double(const MarsdbValue *value); /* also INT64 as double */
int marsdb_value_bool(const MarsdbValue *value);
/* STRING content; DATE/DURATION as canonical ISO-8601 text; NULL for
 * other types. Borrowed (see lifetime rules). */
const char *marsdb_value_string(const MarsdbValue *value);

/* NODE accessors (0/NULL for non-nodes). */
uint64_t marsdb_node_id(const MarsdbValue *value);
size_t marsdb_node_label_count(const MarsdbValue *value);
const char *marsdb_node_label(const MarsdbValue *value, size_t i);
size_t marsdb_node_prop_count(const MarsdbValue *value);
const char *marsdb_node_prop_name(const MarsdbValue *value, size_t i);
const MarsdbValue *marsdb_node_prop_value(const MarsdbValue *value, size_t i);

/* EDGE accessors (0/NULL for non-edges). Props share the node trio's
 * shape. */
uint64_t marsdb_edge_id(const MarsdbValue *value);
uint64_t marsdb_edge_src(const MarsdbValue *value);
uint64_t marsdb_edge_dst(const MarsdbValue *value);
const char *marsdb_edge_label(const MarsdbValue *value);
size_t marsdb_edge_prop_count(const MarsdbValue *value);
const char *marsdb_edge_prop_name(const MarsdbValue *value, size_t i);
const MarsdbValue *marsdb_edge_prop_value(const MarsdbValue *value, size_t i);

/* LIST / MAP / PATH accessors. PATH elements alternate NODE and EDGE
 * values. */
size_t marsdb_list_len(const MarsdbValue *value);
const MarsdbValue *marsdb_list_get(const MarsdbValue *value, size_t i);
size_t marsdb_map_len(const MarsdbValue *value);
const char *marsdb_map_key(const MarsdbValue *value, size_t i);
const MarsdbValue *marsdb_map_get(const MarsdbValue *value, size_t i);
size_t marsdb_path_len(const MarsdbValue *value);
const MarsdbValue *marsdb_path_get(const MarsdbValue *value, size_t i);

/* ---- streaming ------------------------------------------------------ */

/* Called once per row with a one-row result view: marsdb_column_* and
 * marsdb_row_value work on it exactly as on a full result (the cursor
 * is already positioned; do not call marsdb_next/destroy on it).
 * Pointers from the view are valid only during the callback. Return 0
 * to continue, nonzero to stop the scan early (clean stop, not an
 * error). */
typedef int (*MarsdbRowCallback)(void *user_data, const MarsdbResult *row_view);

/* Stream a read-only statement's rows — bounded memory no matter how
 * many rows match. Accepts exactly the streamable shape (one plain
 * MATCH ... RETURN, SKIP/LIMIT fine) and errors — never silently
 * materializes — on ORDER BY/aggregation/DISTINCT/WITH/writes. */
int marsdb_stream(MarsdbDatabase *db, const char *cypher,
                  MarsdbRowCallback on_row, void *user_data);
int marsdb_stmt_stream(MarsdbStatement *stmt, MarsdbRowCallback on_row,
                       void *user_data);

/* ---- batch lane ------------------------------------------------------
 *
 * One call, whole result as a compact self-describing binary blob.
 * Format (version 1; all varints unsigned LEB128; "svarint" = zigzag):
 *
 *   u8      version (= 1)
 *   varint  string-table entry count
 *           entries: varint byte_len, then UTF-8 bytes. Column names,
 *           property names, labels, and relationship types are interned
 *           here ONCE per batch; rows reference them by index.
 *   varint  column_count; column name ids (varint table indexes)
 *   varint  row_count
 *   rows:   column_count tag-prefixed values each:
 *     0x00 null
 *     0x01 bool: u8
 *     0x02 int: svarint
 *     0x03 float: 8 bytes, f64 little-endian bits
 *     0x04 string: varint byte_len + UTF-8 bytes (inline — values are
 *          data; only NAMES intern)
 *     0x05 date: ISO-8601 text, varint len + bytes
 *     0x06 duration: ISO-8601 text, varint len + bytes
 *     0x07 node: varint id, varint label_count + label ids,
 *          varint prop_count + (name id, value) pairs
 *     0x08 edge: varint id, varint src, varint dst, type id,
 *          varint prop_count + (name id, value) pairs
 *     0x09 list: varint len + values
 *     0x0a map: varint len + (name id, value) pairs
 *     0x0b path: varint len + alternating node/edge values
 *   stats:  7 varints (nodes_created, nodes_deleted,
 *           relationships_created, relationships_deleted,
 *           properties_set, labels_added, labels_removed)
 */
int marsdb_query_batch(MarsdbDatabase *db, const char *cypher, MarsdbBuffer *out);
int marsdb_stmt_execute_batch(MarsdbStatement *stmt, MarsdbBuffer *out);
void marsdb_buffer_free(MarsdbBuffer buffer);

#ifdef __cplusplus
}
#endif

#endif /* MARSDB_CAPI_H */
