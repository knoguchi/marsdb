# marsdb-go-arrow

Arrow results for [MarsDB](https://github.com/knoguchi/marsdb) in Go —
the companion module to `marsdb-go` for columnar consumers.

Results cross the C boundary once, as an Arrow C Data Interface
stream, and are imported zero-copy by
[arrow-go](https://github.com/apache/arrow-go)'s `cdata` package: no
per-value decode, no per-row Go allocations. Measured against the
batch lane on a 200k-row, 3-column result: ~1.2M Go-side allocations
(83 MB) per query drop to ~900 (83 KB). Wall time is at parity today —
engine execution dominates — the allocation profile is the win.

This is a separate Go module so `marsdb-go` itself stays
dependency-free; arrow-go is a heavyweight dependency only columnar
consumers should pay for.

## Build

Requires `marsdb-capi` built **with its `arrow` cargo feature** (the
Arrow C functions are absent otherwise — you'd get a link error, not a
runtime error):

```bash
cargo build -p marsdb-capi --features arrow
```

Linking and library-path setup are otherwise identical to
[marsdb-go's README](../marsdb-go/README.md).

## Usage

```go
import (
    "io"

    "github.com/apache/arrow-go/v18/arrow/array"
    marsdb "github.com/knoguchi/marsdb/marsdb-go"
    marsdbarrow "github.com/knoguchi/marsdb/marsdb-go-arrow"
)

db, _ := marsdb.Open("graph.marsdb")
reader, err := marsdbarrow.Query(db,
    "MATCH (n:Person) WHERE n.age > $min RETURN n.name AS name, n.age AS age",
    map[string]any{"min": 30}, marsdb.Options{}, 0)
if err != nil { ... }
for {
    rec, err := reader.Read()
    if err == io.EOF {
        break
    }
    ages := rec.Column(1).(*array.Int64).Int64Values() // zero-copy slice view
    ...
}
```

Each `Read` returns one record of up to `batchRows` rows (0 = 8192)
and `io.EOF` at the end. A record is valid until the next `Read`;
`Retain` it to keep it longer.

## Column typing

Strict, per column over the whole result: `Int64` (64-bit exact),
`Float64`, `Utf8`, `Boolean`, `Date32`, `Interval(MonthDayNano)` for
durations, `Utf8` ISO text for other temporals, `List<child>` for
homogeneous lists, `Null` for all-null columns. A column mixing ints
and floats is an error — silent `Float64` promotion corrupts integers
beyond 2^53; cast in the query (`toFloat`/`toInteger`) instead. So are
node/relationship/map/path columns: project scalar properties.

## License

Same as the repository.
