// Package marsdb provides Go bindings for MarsDB, an embeddable
// property-graph database with an openCypher query subset. It links
// against the marsdb-capi cdylib/staticlib (../marsdb-capi) via cgo — see
// this package's README for the two-step build (build the Rust crate,
// then `go build`/`go test` here).
package marsdb

/*
#cgo CFLAGS: -I${SRCDIR}/../marsdb-capi
#cgo LDFLAGS: -L${SRCDIR}/../target/debug -lmarsdb_capi
#include <stdlib.h>
#include "marsdb.h"
*/
import "C"

import (
	"encoding/json"
	"errors"
	"strconv"
	"strings"
	"sync"
	"time"
	"unsafe"
)

// Database is a handle to an open MarsDB database, either a single
// on-disk file or a transient in-memory instance. The zero value is not
// usable; construct one with Open or InMemory.
type Database struct {
	mu  sync.RWMutex
	ptr *C.MarsdbDatabase
}

// Open opens (creating if absent) a single-file, on-disk database.
func Open(path string) (*Database, error) {
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))

	ptr := C.marsdb_open(cPath)
	if ptr == nil {
		return nil, errors.New("marsdb: failed to open database at " + path)
	}
	return &Database{ptr: ptr}, nil
}

// InMemory opens a purely in-memory database. Nothing is written to disk.
func InMemory() (*Database, error) {
	ptr := C.marsdb_open_in_memory()
	if ptr == nil {
		return nil, errors.New("marsdb: failed to open in-memory database")
	}
	return &Database{ptr: ptr}, nil
}

// Close releases the underlying database handle. It is safe to call more than
// once and waits for concurrent Execute calls to finish.
func (db *Database) Close() error {
	db.mu.Lock()
	defer db.mu.Unlock()
	if db.ptr == nil {
		return nil
	}
	C.marsdb_close(db.ptr)
	db.ptr = nil
	return nil
}

// row is the wire shape marsdb-capi's marsdb_execute produces: parallel
// columns/rows, decoded here into one map[string]any per row (keyed by
// column name) to match marsdb-python's dict-per-row shape.
type queryResult struct {
	Columns []string `json:"columns"`
	Rows    [][]any  `json:"rows"`
}

// Execute runs one Cypher statement, returning one map[string]any per
// matched row (column name -> value). CREATE/DELETE/SET statements
// return an empty slice.
//
// Nodes decode as map[string]any{"__type": "node", "id": ..., "labels":
// [...]any, "props": map[string]any{...}}; edges similarly with
// "__type": "edge" plus "src"/"dst". JSON integers decode as int64 (or
// uint64 when they exceed int64's range), while values containing a decimal
// point or exponent decode as float64. Dates and durations decode as their
// canonical ISO-8601 strings.
func (db *Database) Execute(cypher string) ([]map[string]any, error) {
	return db.execute(cypher, nil)
}

// ExecuteWithParams runs one Cypher statement with $name placeholders
// resolved from params. Values may be nil, bool, any Go integer or float
// type, string, or (arbitrarily nested) []any / map[string]any of those
// — anything encoding/json can marshal to null/bool/number/string/array/
// object. Go int64 values keep their full range end to end (params cross
// the C ABI as a JSON object, and integral JSON numbers are parsed as
// i64 on the Rust side); a uint64 above int64's range is rejected there
// rather than silently rounded.
func (db *Database) ExecuteWithParams(cypher string, params map[string]any) ([]map[string]any, error) {
	if params == nil {
		return db.execute(cypher, nil)
	}
	encoded, err := json.Marshal(params)
	if err != nil {
		return nil, errors.New("marsdb: params not JSON-encodable: " + err.Error())
	}
	return db.execute(cypher, encoded)
}

// Options bounds a statement's execution: MaxRows caps the result row
// count and Timeout caps wall time, both checked during evaluation — a
// runaway query fails at the bound instead of materializing an
// unbounded result first. Zero values mean unlimited.
type Options struct {
	MaxRows uint64
	Timeout time.Duration
}

// ExecuteWithOptions runs one Cypher statement with $name parameters
// (nil for none — same value rules as ExecuteWithParams) under the
// given execution bounds.
func (db *Database) ExecuteWithOptions(cypher string, params map[string]any, opts Options) ([]map[string]any, error) {
	var encoded []byte
	if params != nil {
		var err error
		encoded, err = json.Marshal(params)
		if err != nil {
			return nil, errors.New("marsdb: params not JSON-encodable: " + err.Error())
		}
	}
	return db.executeOpts(cypher, encoded, opts)
}

func (db *Database) execute(cypher string, paramsJSON []byte) ([]map[string]any, error) {
	db.mu.RLock()
	defer db.mu.RUnlock()
	if db.ptr == nil {
		return nil, errors.New("marsdb: database is closed")
	}
	cCypher := C.CString(cypher)
	defer C.free(unsafe.Pointer(cCypher))

	var result C.MarsdbResult
	if paramsJSON == nil {
		result = C.marsdb_execute(db.ptr, cCypher)
	} else {
		cParams := C.CString(string(paramsJSON))
		defer C.free(unsafe.Pointer(cParams))
		result = C.marsdb_execute_with_params(db.ptr, cCypher, cParams)
	}
	return db.decode(result)
}

func (db *Database) executeOpts(cypher string, paramsJSON []byte, opts Options) ([]map[string]any, error) {
	db.mu.RLock()
	defer db.mu.RUnlock()
	if db.ptr == nil {
		return nil, errors.New("marsdb: database is closed")
	}
	cCypher := C.CString(cypher)
	defer C.free(unsafe.Pointer(cCypher))

	var cParams *C.char
	if paramsJSON != nil {
		cParams = C.CString(string(paramsJSON))
		defer C.free(unsafe.Pointer(cParams))
	}
	result := C.marsdb_execute_ex(db.ptr, cCypher, cParams,
		C.uint64_t(opts.MaxRows), C.uint64_t(opts.Timeout/time.Millisecond))
	return db.decode(result)
}

func (db *Database) decode(result C.MarsdbResult) ([]map[string]any, error) {
	if result.error != nil {
		defer C.marsdb_free_string(result.error)
		return nil, errors.New("marsdb: " + C.GoString(result.error))
	}
	defer C.marsdb_free_string(result.json)

	raw := C.GoString(result.json)
	var qr queryResult
	decoder := json.NewDecoder(strings.NewReader(raw))
	decoder.UseNumber()
	if err := decoder.Decode(&qr); err != nil {
		return nil, errors.New("marsdb: malformed result JSON: " + err.Error())
	}

	rows := make([]map[string]any, len(qr.Rows))
	for i, row := range qr.Rows {
		m := make(map[string]any, len(qr.Columns))
		for j, col := range qr.Columns {
			if j < len(row) {
				value, err := normalizeJSONNumbers(row[j])
				if err != nil {
					return nil, errors.New("marsdb: malformed result number: " + err.Error())
				}
				m[col] = value
			}
		}
		rows[i] = m
	}
	return rows, nil
}

// normalizeJSONNumbers recursively converts the json.Number values produced
// by Decoder.UseNumber. This preserves all MarsDB i64 property values and u64
// graph IDs exactly instead of silently rounding them through float64.
func normalizeJSONNumbers(value any) (any, error) {
	switch value := value.(type) {
	case json.Number:
		if strings.ContainsAny(value.String(), ".eE") {
			return value.Float64()
		}
		if i, err := value.Int64(); err == nil {
			return i, nil
		}
		u, err := strconv.ParseUint(value.String(), 10, 64)
		if err != nil {
			return nil, err
		}
		return u, nil
	case []any:
		for i, item := range value {
			normalized, err := normalizeJSONNumbers(item)
			if err != nil {
				return nil, err
			}
			value[i] = normalized
		}
		return value, nil
	case map[string]any:
		for key, item := range value {
			normalized, err := normalizeJSONNumbers(item)
			if err != nil {
				return nil, err
			}
			value[key] = normalized
		}
		return value, nil
	default:
		return value, nil
	}
}
