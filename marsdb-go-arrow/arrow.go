// Package marsdbarrow exports MarsDB query results as Arrow records —
// the companion module to marsdb-go for columnar consumers.
//
// Results cross the C boundary once, as an Arrow C Data Interface
// stream (marsdb_query_arrow / marsdb_stmt_execute_arrow in
// marsdb-capi, built with its `arrow` cargo feature), and are imported
// zero-copy by arrow-go's cdata package: no per-value decode, no
// per-row Go allocations. Against the batch lane this trades ~1.2M
// Go-side allocations (83 MB) per 200k-row result for ~900 (83 KB);
// wall-time parity today (engine execution dominates), allocation
// profile is the win.
//
// This is a separate module so marsdb-go itself stays dependency-free:
// arrow-go is a heavyweight dependency only columnar consumers should
// pay for.
//
// Column typing is strict, per column over the whole result: Int64
// (64-bit exact), Float64, Utf8, Boolean, Date32,
// Interval(MonthDayNano) for durations, Utf8 ISO text for other
// temporals, List<child> for homogeneous lists, Null for all-null
// columns. A column mixing ints and floats is an error (silent Float64
// promotion corrupts integers beyond 2^53 — cast in the query), as are
// node/edge/map/path columns: project scalar properties instead.
package marsdbarrow

/*
#cgo CFLAGS: -I${SRCDIR}/../marsdb-capi
#cgo LDFLAGS: -L${SRCDIR}/../target/debug -lmarsdb_capi
#include "marsdb.h"
*/
import "C"

import (
	"unsafe"

	"github.com/apache/arrow-go/v18/arrow/arrio"
	"github.com/apache/arrow-go/v18/arrow/cdata"
	marsdb "github.com/knoguchi/marsdb/marsdb-go"
)

// DefaultBatchRows is the rows-per-record default used when batchRows
// is 0, matching the other language surfaces.
const DefaultBatchRows = 8192

// Query runs one Cypher statement with $name parameters (nil for none;
// same value rules as marsdb.ExecuteWithParams) under the given
// execution bounds, returning the result as a reader of Arrow records.
//
// Each Read returns one record of up to batchRows rows (0 means
// DefaultBatchRows) and io.EOF at the end. A record is valid until the
// next Read; Retain it to keep it longer. The result is fully
// materialized before Query returns (Cypher columns are dynamically
// typed — inference needs every row), so the reader never blocks and
// does not depend on db afterwards.
func Query(db *marsdb.Database, cypher string, params map[string]any, opts marsdb.Options, batchRows int) (arrio.Reader, error) {
	if batchRows <= 0 {
		batchRows = DefaultBatchRows
	}
	var reader arrio.Reader
	err := db.WithStatementHandle(cypher, params, opts, func(stmt unsafe.Pointer) error {
		var stream C.struct_ArrowArrayStream
		if C.marsdb_stmt_execute_arrow((*C.MarsdbStatement)(stmt), C.size_t(batchRows), &stream) != C.MARSDB_OK {
			return db.LastError()
		}
		r, err := cdata.ImportCRecordReader((*cdata.CArrowArrayStream)(unsafe.Pointer(&stream)), nil)
		if err != nil {
			return err
		}
		reader = r
		return nil
	})
	if err != nil {
		return nil, err
	}
	return reader, nil
}
