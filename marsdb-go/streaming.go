package marsdb

/*
#include <stdint.h>
#include <stdlib.h>
#include "marsdb.h"

// cgo can't pass a Go function as a C function pointer directly; these
// exported trampolines (defined in Go below) are what the C side calls,
// with a runtime/cgo.Handle to the per-call state as user_data.
extern void marsdbGoColumns(void *user_data, char *columns_json);
extern int marsdbGoRow(void *user_data, char *row_json);
*/
import "C"

import (
	"encoding/json"
	"errors"
	"runtime/cgo"
	"strings"
	"unsafe"
)

// streamState carries one ExecuteStreaming call's state across the C
// boundary via a cgo.Handle.
type streamState struct {
	columns []string
	onRow   func(map[string]any) bool
	err     error
}

//export marsdbGoColumns
func marsdbGoColumns(userData unsafe.Pointer, columnsJSON *C.char) {
	state := cgo.Handle(uintptr(userData)).Value().(*streamState)
	if err := json.Unmarshal([]byte(C.GoString(columnsJSON)), &state.columns); err != nil {
		state.err = errors.New("marsdb: malformed columns JSON: " + err.Error())
	}
}

//export marsdbGoRow
func marsdbGoRow(userData unsafe.Pointer, rowJSON *C.char) C.int {
	state := cgo.Handle(uintptr(userData)).Value().(*streamState)
	if state.err != nil {
		return 1
	}
	var values []any
	decoder := json.NewDecoder(strings.NewReader(C.GoString(rowJSON)))
	decoder.UseNumber()
	if err := decoder.Decode(&values); err != nil {
		state.err = errors.New("marsdb: malformed row JSON: " + err.Error())
		return 1
	}
	row := make(map[string]any, len(state.columns))
	for i, col := range state.columns {
		if i < len(values) {
			value, err := normalizeJSONNumbers(values[i])
			if err != nil {
				state.err = errors.New("marsdb: malformed row number: " + err.Error())
				return 1
			}
			row[col] = value
		}
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
	cCypher := C.CString(cypher)
	defer C.free(unsafe.Pointer(cCypher))

	var cParams *C.char
	if params != nil {
		encoded, err := json.Marshal(params)
		if err != nil {
			return errors.New("marsdb: params not JSON-encodable: " + err.Error())
		}
		cParams = C.CString(string(encoded))
		defer C.free(unsafe.Pointer(cParams))
	}

	state := &streamState{onRow: onRow}
	handle := cgo.NewHandle(state)
	defer handle.Delete()

	cErr := C.marsdb_execute_streaming(
		db.ptr, cCypher, cParams,
		C.uint64_t(opts.MaxRows), C.uint64_t(opts.Timeout/1e6),
		C.MarsdbColumnsCallback(C.marsdbGoColumns),
		C.MarsdbRowCallback(C.marsdbGoRow),
		unsafe.Pointer(uintptr(handle)), //nolint:govet // opaque token round-trip, never dereferenced
	)
	if cErr != nil {
		defer C.marsdb_free_string(cErr)
		return errors.New("marsdb: " + C.GoString(cErr))
	}
	return state.err
}
