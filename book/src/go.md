# Go bindings

Unlike [`marsdb-python`](./python.md) (PyO3, in-process), Go has no
equivalent in-process FFI story with Rust, so the Go binding goes
through the small C ABI crate, [`marsdb-capi`](./c-api.md), via cgo.

The binding lives in its own repository,
[`marsdb-go`](https://github.com/knoguchi/marsdb-go), as two Go modules:

```
go get github.com/knoguchi/marsdb-go        # core binding, zero deps
go get github.com/knoguchi/marsdb-go/arrow  # Arrow results (arrow-go dep)
```

The split keeps the core module dependency-free — arrow-go is a
heavyweight dependency only columnar consumers should pay for.

## Build

The C header is vendored in the binding repo; only the library needs
building here. In a checkout of this repository:

```
cargo build -p marsdb-capi --features arrow
```

(`--features arrow` is required by the arrow module and harmless for the
core one.) Then, in a `marsdb-go` checkout:

```
export CGO_LDFLAGS="-L/path/to/marsdb/target/debug"
go test ./...
```

## Usage

```go
package main

import (
	"fmt"
	"log"

	marsdb "github.com/knoguchi/marsdb-go"
)

func main() {
	db, err := marsdb.InMemory() // or marsdb.Open("path/to.db")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	if _, err := db.Execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})"); err != nil {
		log.Fatal(err)
	}

	rows, err := db.Execute("MATCH (n:Person) RETURN n.name AS name ORDER BY n.name")
	if err != nil {
		log.Fatal(err)
	}
	for _, row := range rows {
		fmt.Println(row["name"])
	}
	// Alice
	// Bob
}
```

`Execute` returns `[]map[string]any`, one map per matched row keyed by
column name — the same dict-per-row shape as `marsdb-python`. A returned
node decodes as `map[string]any{"__type": "node", "id": ..., "labels":
[]any{...}, "props": map[string]any{...}}`; an edge similarly with
`"__type": "edge"` plus `"src"`/`"dst"`. Integer properties and IDs
retain their full precision as `int64` (or `uint64` for an ID above
`int64`'s range), while fractional values are `float64`. Dates and
durations are returned as canonical ISO-8601 strings such as
`"1984-10-11"` and `"P1M2D"`.

See the [marsdb-go README](https://github.com/knoguchi/marsdb-go) for
the full API tour — parameterized queries, transactions, streaming,
execution bounds — platform linking notes, and the Arrow module's
column-typing rules.
