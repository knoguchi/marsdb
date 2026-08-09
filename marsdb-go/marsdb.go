// Package marsdb provides Go bindings for MarsDB, an embeddable
// property-graph database with an openCypher query subset. It links
// against the marsdb-capi cdylib/staticlib (../marsdb-capi) via cgo — see
// this package's README for the two-step build (build the Rust crate,
// then `go build`/`go test` here).
//
// Results travel over marsdb-capi's binary batch lane: one cgo crossing
// per query returns the whole result as a compact self-describing blob
// (interned names, varint ints — see marsdb.h's format spec), decoded by
// the pure-Go stdlib-only decoder in batch.go. Parameters go through
// typed prepared-statement binds — scalars and flat lists; nested
// list/map parameter values are not supported through the C ABI.
package marsdb

/*
#cgo CFLAGS: -I${SRCDIR}/../marsdb-capi
#cgo LDFLAGS: -L${SRCDIR}/../target/debug -lmarsdb_capi
#include <stdlib.h>
#include "marsdb.h"
*/
import "C"

import (
	"errors"
	"fmt"
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

// Stats reports what a write statement changed — all zero for reads.
// PropertiesSet counts removals too (removing a property is setting it
// away); label changes are their own pair.
type Stats struct {
	NodesCreated         uint64
	NodesDeleted         uint64
	RelationshipsCreated uint64
	RelationshipsDeleted uint64
	PropertiesSet        uint64
	LabelsAdded          uint64
	LabelsRemoved        uint64
}

// Options bounds a statement's execution: MaxRows caps the result row
// count and Timeout caps wall time, both checked during evaluation — a
// runaway query fails at the bound instead of materializing an
// unbounded result first. Zero values mean unlimited.
type Options struct {
	MaxRows uint64
	Timeout time.Duration
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

// Close releases the underlying database handle. It is safe to call more
// than once and waits for concurrent calls to finish.
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

func (db *Database) lastError() error {
	return errors.New("marsdb: " + C.GoString(C.marsdb_last_error(db.ptr)))
}

// Execute runs one Cypher statement, returning one map[string]any per
// matched row (column name -> value). CREATE/DELETE/SET statements
// return an empty slice.
//
// Nodes decode as map[string]any{"__type": "node", "id": ..., "labels":
// []any, "props": map[string]any{...}}; edges similarly with
// "__type": "edge" plus "src"/"dst". Integer properties and IDs retain
// full precision as int64 (uint64 for an ID above int64's range);
// fractional values are float64. Dates and durations are canonical
// ISO-8601 strings.
func (db *Database) Execute(cypher string) ([]map[string]any, error) {
	rows, _, err := db.executeBatch(cypher, nil, Options{})
	return rows, err
}

// ExecuteWithParams runs one Cypher statement with $name placeholders
// resolved from params. Values may be nil, bool, any Go integer or
// float type, string, or a FLAT []any of those (`WHERE x IN $list`).
// Nested lists/maps as parameter VALUES are not supported through the
// C ABI (map-shaped and nested data still round-trips fine in
// results). int64 values keep their full range end to end.
func (db *Database) ExecuteWithParams(cypher string, params map[string]any) ([]map[string]any, error) {
	rows, _, err := db.executeBatch(cypher, params, Options{})
	return rows, err
}

// ExecuteWithOptions runs one Cypher statement with $name parameters
// (nil for none — same value rules as ExecuteWithParams) under the
// given execution bounds.
func (db *Database) ExecuteWithOptions(cypher string, params map[string]any, opts Options) ([]map[string]any, error) {
	rows, _, err := db.executeBatch(cypher, params, opts)
	return rows, err
}

// ExecuteStats is Execute plus the statement's write counters — the
// answer to "how many did my DELETE delete".
func (db *Database) ExecuteStats(cypher string, params map[string]any) ([]map[string]any, Stats, error) {
	return db.executeBatch(cypher, params, Options{})
}

// prepared wraps a live C statement handle; callers must destroy() it.
type prepared struct {
	ptr *C.MarsdbStatement
}

func (db *Database) prepare(cypher string, params map[string]any, opts Options) (*prepared, error) {
	cCypher := C.CString(cypher)
	defer C.free(unsafe.Pointer(cCypher))
	var stmt *C.MarsdbStatement
	if C.marsdb_prepare(db.ptr, cCypher, &stmt) != C.MARSDB_OK {
		return nil, db.lastError()
	}
	p := &prepared{ptr: stmt}
	if opts.MaxRows > 0 {
		C.marsdb_stmt_set_max_rows(stmt, C.uint64_t(opts.MaxRows))
	}
	if opts.Timeout > 0 {
		C.marsdb_stmt_set_timeout_ms(stmt, C.uint64_t(opts.Timeout/time.Millisecond))
	}
	for name, value := range params {
		if err := p.bind(name, value); err != nil {
			p.destroy()
			return nil, err
		}
	}
	return p, nil
}

func (p *prepared) destroy() {
	C.marsdb_stmt_destroy(p.ptr)
}

func (p *prepared) bind(name string, value any) error {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	switch v := value.(type) {
	case nil:
		C.marsdb_bind_null(p.ptr, cName)
	case bool:
		b := C.int(0)
		if v {
			b = 1
		}
		C.marsdb_bind_bool(p.ptr, cName, b)
	case int:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case int8:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case int16:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case int32:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case int64:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case uint, uint8, uint16, uint32:
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(toUint64(v)))
	case uint64:
		if v > 1<<63-1 {
			return fmt.Errorf("marsdb: parameter %q: uint64 %d exceeds int64 range", name, v)
		}
		C.marsdb_bind_int64(p.ptr, cName, C.int64_t(v))
	case float32:
		C.marsdb_bind_double(p.ptr, cName, C.double(v))
	case float64:
		C.marsdb_bind_double(p.ptr, cName, C.double(v))
	case string:
		cValue := C.CString(v)
		defer C.free(unsafe.Pointer(cValue))
		C.marsdb_bind_string(p.ptr, cName, cValue)
	case []any:
		return p.bindList(name, cName, v)
	default:
		return fmt.Errorf(
			"marsdb: parameter %q: unsupported type %T -- use nil/bool/int/float/string or a flat []any of those",
			name, value)
	}
	return nil
}

func toUint64(v any) uint64 {
	switch v := v.(type) {
	case uint:
		return uint64(v)
	case uint8:
		return uint64(v)
	case uint16:
		return uint64(v)
	case uint32:
		return uint64(v)
	default:
		return 0
	}
}

// Flat lists bind through the typed list binds; a heterogeneous or
// nested list is rejected rather than silently coerced.
func (p *prepared) bindList(name string, cName *C.char, items []any) error {
	if len(items) == 0 {
		C.marsdb_bind_int64_list(p.ptr, cName, nil, 0)
		return nil
	}
	switch items[0].(type) {
	case int, int8, int16, int32, int64:
		values := make([]C.int64_t, len(items))
		for i, item := range items {
			switch n := item.(type) {
			case int:
				values[i] = C.int64_t(n)
			case int8:
				values[i] = C.int64_t(n)
			case int16:
				values[i] = C.int64_t(n)
			case int32:
				values[i] = C.int64_t(n)
			case int64:
				values[i] = C.int64_t(n)
			default:
				return fmt.Errorf("marsdb: parameter %q: mixed-type list not supported", name)
			}
		}
		C.marsdb_bind_int64_list(p.ptr, cName, &values[0], C.size_t(len(values)))
	case float32, float64:
		values := make([]C.double, len(items))
		for i, item := range items {
			switch f := item.(type) {
			case float32:
				values[i] = C.double(f)
			case float64:
				values[i] = C.double(f)
			default:
				return fmt.Errorf("marsdb: parameter %q: mixed-type list not supported", name)
			}
		}
		C.marsdb_bind_double_list(p.ptr, cName, &values[0], C.size_t(len(values)))
	case string:
		cStrings := make([]*C.char, len(items))
		defer func() {
			for _, s := range cStrings {
				if s != nil {
					C.free(unsafe.Pointer(s))
				}
			}
		}()
		for i, item := range items {
			s, ok := item.(string)
			if !ok {
				return fmt.Errorf("marsdb: parameter %q: mixed-type list not supported", name)
			}
			cStrings[i] = C.CString(s)
		}
		C.marsdb_bind_string_list(p.ptr, cName, &cStrings[0], C.size_t(len(cStrings)))
	default:
		return fmt.Errorf(
			"marsdb: parameter %q: unsupported list element type %T (flat int/float/string lists only)",
			name, items[0])
	}
	return nil
}

func (db *Database) executeBatch(cypher string, params map[string]any, opts Options) ([]map[string]any, Stats, error) {
	db.mu.RLock()
	defer db.mu.RUnlock()
	if db.ptr == nil {
		return nil, Stats{}, errors.New("marsdb: database is closed")
	}

	var buffer C.MarsdbBuffer
	if len(params) == 0 && opts == (Options{}) {
		cCypher := C.CString(cypher)
		defer C.free(unsafe.Pointer(cCypher))
		if C.marsdb_query_batch(db.ptr, cCypher, &buffer) != C.MARSDB_OK {
			return nil, Stats{}, db.lastError()
		}
	} else {
		p, err := db.prepare(cypher, params, opts)
		if err != nil {
			return nil, Stats{}, err
		}
		defer p.destroy()
		if C.marsdb_stmt_execute_batch(p.ptr, &buffer) != C.MARSDB_OK {
			return nil, Stats{}, db.lastError()
		}
	}
	defer C.marsdb_buffer_free(buffer)
	bytes := C.GoBytes(unsafe.Pointer(buffer.data), C.int(buffer.len))
	return decodeBatch(bytes)
}
