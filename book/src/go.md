# Go bindings

Unlike [`marsdb-python`](./python.md) (PyO3, in-process), Go has no
equivalent in-process FFI story with Rust, so [`marsdb-go`](https://github.com/knoguchi/marsdb/tree/main/marsdb-go)
goes through the small C ABI crate, [`marsdb-capi`](./c-api.md), via cgo.

**Not published anywhere yet** — no `go get`-able module path, no tagged
release. Use it by cloning the repo and building both pieces locally.

## Build

Two steps: build the Rust cdylib, then build the Go package against it.

```
# 1. Build marsdb-capi (produces target/debug/libmarsdb_capi.dylib on macOS)
cargo build -p marsdb-capi

# 2. Build/test the Go package
cd marsdb-go
go build ./...
go test ./...
```

`marsdb.go`'s cgo preamble already points `-L`/`-I` at
`../target/debug`/`../marsdb-capi` relative to this directory via cgo's
`${SRCDIR}` substitution, so the two commands above work as-is right
after a debug build on macOS. On Linux, add the shared-library directory
at runtime:

```
LD_LIBRARY_PATH="$(pwd)/../target/debug" go test ./...
```

If you built `marsdb-capi` in release mode instead
(`cargo build -p marsdb-capi --release`), override the link path:

```
CGO_LDFLAGS="-L$(pwd)/../target/release -lmarsdb_capi" go build ./...
```

## Usage

```go
package main

import (
	"fmt"
	"log"

	marsdb "github.com/knoguchi/marsdb/marsdb-go"
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

A runnable copy lives in [`examples/basic`](https://github.com/knoguchi/marsdb/tree/main/marsdb-go/examples/basic):

```
cargo build -p marsdb-capi
cd marsdb-go && go run ./examples/basic
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

## What's not here yet

Only `Open`/`InMemory`/`Execute`/`Close` — `execute_batch` (multi-statement,
one transaction each) and `execute_with_params` (`$param` substitution)
exist on the Rust/C ABI side's natural extension points but aren't wired
through `marsdb-capi` or this package yet. Not yet set up to produce a
redistributable Go binary to a machine without this exact local build
layout either — see the package README for the exact gap
(static-linking `libmarsdb_capi.a`, or `@rpath`-relative dylib linking).
