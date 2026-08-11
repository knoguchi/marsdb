package marsdbarrow

import (
	"fmt"
	"io"
	"math"
	"strings"
	"testing"

	"github.com/apache/arrow-go/v18/arrow"
	"github.com/apache/arrow-go/v18/arrow/array"
	marsdb "github.com/knoguchi/marsdb/marsdb-go"
)

func testDB(t *testing.T) *marsdb.Database {
	t.Helper()
	db, err := marsdb.InMemory()
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { db.Close() })
	return db
}

func TestRoundTripTypesAndExactness(t *testing.T) {
	db := testDB(t)
	if _, err := db.Execute(fmt.Sprintf(
		"CREATE (:N {i: %d, f: 1.5, s: 'héllo', b: true, d: date('1984-10-11')}), (:N {i: 2})",
		int64(math.MaxInt64))); err != nil {
		t.Fatal(err)
	}
	reader, err := Query(db,
		"MATCH (n:N) RETURN n.i AS i, n.f AS f, n.s AS s, n.b AS b, n.d AS d",
		nil, marsdb.Options{}, 0)
	if err != nil {
		t.Fatal(err)
	}
	rec, err := reader.Read()
	if err != nil {
		t.Fatal(err)
	}
	schema := rec.Schema()
	for i, want := range []arrow.DataType{
		arrow.PrimitiveTypes.Int64, arrow.PrimitiveTypes.Float64,
		arrow.BinaryTypes.String, arrow.FixedWidthTypes.Boolean,
		arrow.FixedWidthTypes.Date32,
	} {
		if got := schema.Field(i).Type; !arrow.TypeEqual(got, want) {
			t.Fatalf("column %d: got %v, want %v", i, got, want)
		}
	}
	ints := rec.Column(0).(*array.Int64)
	var sawMax bool
	for j := 0; j < ints.Len(); j++ {
		sawMax = sawMax || ints.Value(j) == math.MaxInt64
	}
	if !sawMax {
		t.Fatal("i64::MAX did not survive")
	}
	// The i:2 node has no other props -- nulls become validity.
	if rec.Column(2).(*array.String).NullN() != 1 {
		t.Fatal("expected one null in s")
	}
	if _, err := reader.Read(); err != io.EOF {
		t.Fatalf("expected io.EOF, got %v", err)
	}
}

func TestParamsBatchingAndBounds(t *testing.T) {
	db := testDB(t)
	if _, err := db.Execute("UNWIND range(0, 6) AS x CREATE (:B {i: x})"); err != nil {
		t.Fatal(err)
	}
	reader, err := Query(db, "MATCH (b:B) WHERE b.i >= $min RETURN b.i AS i",
		map[string]any{"min": 0}, marsdb.Options{}, 3)
	if err != nil {
		t.Fatal(err)
	}
	var sizes []int
	for {
		rec, err := reader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatal(err)
		}
		sizes = append(sizes, int(rec.NumRows()))
	}
	if fmt.Sprint(sizes) != "[3 3 1]" {
		t.Fatalf("batch sizes: %v", sizes)
	}
	if _, err := Query(db, "MATCH (b:B) RETURN b.i AS i", nil,
		marsdb.Options{MaxRows: 2}, 0); err == nil {
		t.Fatal("expected MaxRows violation")
	}
}

func TestStrictInferenceErrors(t *testing.T) {
	db := testDB(t)
	if _, err := db.Execute("CREATE (:P {x: 1}), (:P {x: 2.5})"); err != nil {
		t.Fatal(err)
	}
	_, err := Query(db, "MATCH (p:P) RETURN p.x AS mixed", nil, marsdb.Options{}, 0)
	if err == nil || !strings.Contains(err.Error(), "'mixed'") {
		t.Fatalf("expected typed error naming the column, got %v", err)
	}
	_, err = Query(db, "MATCH (p:P) RETURN p", nil, marsdb.Options{}, 0)
	if err == nil || !strings.Contains(err.Error(), "project scalar properties") {
		t.Fatalf("expected entity-column error, got %v", err)
	}
}
