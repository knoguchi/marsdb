package marsdb

/*
#include <stdint.h>
#include <stdlib.h>
#include "marsdb.h"

// cgo can't pass a Go function as a C function pointer directly; this
// exported trampoline (defined in Go below) is what the C side calls,
// with a runtime/cgo.Handle to the per-call state as user_data. The
// handle travels as uintptr_t and becomes void* only here in C -- Go
// never converts uintptr to unsafe.Pointer, keeping `go vet` clean.
extern int marsdbGoRow(void *user_data, MarsdbResult *row_view);

static int marsdb_stream_go(MarsdbDatabase *db, const char *cypher, uintptr_t handle) {
	return marsdb_stream(db, cypher, (MarsdbRowCallback)marsdbGoRow, (void *)handle);
}
static int marsdb_stmt_stream_go(MarsdbStatement *stmt, uintptr_t handle) {
	return marsdb_stmt_stream(stmt, (MarsdbRowCallback)marsdbGoRow, (void *)handle);
}
*/
import "C"

import (
	"errors"
	"runtime/cgo"
	"unsafe"
)

// streamState carries one ExecuteStreaming call's state across the C
// boundary via a cgo.Handle.
type streamState struct {
	onRow func(map[string]any) bool
	err   error
}

// decodeCValue converts a typed value handle (valid only during the
// current callback) into the same Go shapes the batch decoder produces.
// Streaming rows are typically small, so the per-value cgo crossings
// here are acceptable; bulk paths use the batch lane instead.
func decodeCValue(v *C.MarsdbValue) any {
	if v == nil {
		return nil
	}
	switch C.marsdb_value_type(v) {
	case C.MARSDB_TYPE_NULL:
		return nil
	case C.MARSDB_TYPE_BOOL:
		return C.marsdb_value_bool(v) != 0
	case C.MARSDB_TYPE_INT64:
		return int64(C.marsdb_value_int64(v))
	case C.MARSDB_TYPE_FLOAT64:
		return float64(C.marsdb_value_double(v))
	case C.MARSDB_TYPE_STRING, C.MARSDB_TYPE_DATE, C.MARSDB_TYPE_DURATION:
		return C.GoString(C.marsdb_value_string(v))
	case C.MARSDB_TYPE_NODE:
		propCount := C.marsdb_node_prop_count(v)
		props := make(map[string]any, propCount)
		for i := C.size_t(0); i < propCount; i++ {
			name := C.GoString(C.marsdb_node_prop_name(v, i))
			props[name] = decodeCValue(C.marsdb_node_prop_value(v, i))
		}
		labelCount := C.marsdb_node_label_count(v)
		labels := make([]any, 0, labelCount)
		for i := C.size_t(0); i < labelCount; i++ {
			labels = append(labels, C.GoString(C.marsdb_node_label(v, i)))
		}
		return map[string]any{
			"__type": "node",
			"id":     idValue(uint64(C.marsdb_node_id(v))),
			"labels": labels,
			"props":  props,
		}
	case C.MARSDB_TYPE_EDGE:
		propCount := C.marsdb_edge_prop_count(v)
		props := make(map[string]any, propCount)
		for i := C.size_t(0); i < propCount; i++ {
			name := C.GoString(C.marsdb_edge_prop_name(v, i))
			props[name] = decodeCValue(C.marsdb_edge_prop_value(v, i))
		}
		return map[string]any{
			"__type": "edge",
			"id":     idValue(uint64(C.marsdb_edge_id(v))),
			"src":    idValue(uint64(C.marsdb_edge_src(v))),
			"dst":    idValue(uint64(C.marsdb_edge_dst(v))),
			"label":  C.GoString(C.marsdb_edge_label(v)),
			"props":  props,
		}
	case C.MARSDB_TYPE_LIST:
		length := C.marsdb_list_len(v)
		items := make([]any, 0, length)
		for i := C.size_t(0); i < length; i++ {
			items = append(items, decodeCValue(C.marsdb_list_get(v, i)))
		}
		return items
	case C.MARSDB_TYPE_MAP:
		length := C.marsdb_map_len(v)
		m := make(map[string]any, length)
		for i := C.size_t(0); i < length; i++ {
			m[C.GoString(C.marsdb_map_key(v, i))] = decodeCValue(C.marsdb_map_get(v, i))
		}
		return m
	case C.MARSDB_TYPE_PATH:
		length := C.marsdb_path_len(v)
		elems := make([]any, 0, length)
		for i := C.size_t(0); i < length; i++ {
			elems = append(elems, decodeCValue(C.marsdb_path_get(v, i)))
		}
		return elems
	default:
		return nil
	}
}

//export marsdbGoRow
func marsdbGoRow(userData unsafe.Pointer, rowView *C.MarsdbResult) C.int {
	state := cgo.Handle(uintptr(userData)).Value().(*streamState)
	if state.err != nil {
		return 1
	}
	columnCount := C.marsdb_column_count(rowView)
	row := make(map[string]any, columnCount)
	for i := C.size_t(0); i < columnCount; i++ {
		name := C.GoString(C.marsdb_column_name(rowView, i))
		row[name] = decodeCValue(C.marsdb_row_value(rowView, i))
	}
	if state.onRow(row) {
		return 0
	}
	return 1
}

// ExecuteStreaming streams a read-only statement's rows to onRow, one
// map per row — bounded memory no matter how many rows match; the
// bulk-export path. onRow returns false to stop the scan early (clean
// stop, not an error). Accepts exactly the streamable shape (one plain
// MATCH ... RETURN, SKIP/LIMIT fine) and errors — never silently
// materializes — for ORDER BY/aggregation/DISTINCT/WITH/writes.
// params and opts behave as in ExecuteWithOptions.
func (db *Database) ExecuteStreaming(cypher string, params map[string]any, opts Options, onRow func(map[string]any) bool) error {
	db.mu.RLock()
	defer db.mu.RUnlock()
	if db.ptr == nil {
		return errors.New("marsdb: database is closed")
	}

	state := &streamState{onRow: onRow}
	handle := cgo.NewHandle(state)
	defer handle.Delete()

	var status C.int
	if len(params) == 0 && opts == (Options{}) {
		cCypher := C.CString(cypher)
		defer C.free(unsafe.Pointer(cCypher))
		status = C.marsdb_stream_go(db.ptr, cCypher, C.uintptr_t(handle))
	} else {
		p, err := db.prepare(cypher, params, opts)
		if err != nil {
			return err
		}
		defer p.destroy()
		status = C.marsdb_stmt_stream_go(p.ptr, C.uintptr_t(handle))
	}
	if status != C.MARSDB_OK {
		return db.lastError()
	}
	return state.err
}
