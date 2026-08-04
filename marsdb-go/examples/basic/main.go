// Runnable demo for the README: open an in-memory database, create a
// node/edge, query it back, print the results.
//
//	cargo build -p marsdb-capi
//	cd marsdb-go && CGO_LDFLAGS="-L$(pwd)/../target/debug -lmarsdb_capi" go run ./examples/basic
package main

import (
	"fmt"
	"log"

	marsdb "github.com/knoguchi/marsdb/marsdb-go"
)

func main() {
	db, err := marsdb.InMemory()
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
}
