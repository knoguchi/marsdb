package marsdb

import (
	"sort"
	"testing"
)

func TestOpenInMemoryCreateAndQuery(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	if _, err := db.Execute("CREATE (a:Person {name: 'Alice', age: 30})-[:KNOWS]->(b:Person {name: 'Bob', age: 25})"); err != nil {
		t.Fatalf("CREATE: %v", err)
	}

	rows, err := db.Execute("MATCH (n:Person) RETURN n.name AS name ORDER BY n.name")
	if err != nil {
		t.Fatalf("MATCH: %v", err)
	}
	if len(rows) != 2 {
		t.Fatalf("expected 2 rows, got %d: %+v", len(rows), rows)
	}

	var names []string
	for _, row := range rows {
		names = append(names, row["name"].(string))
	}
	sort.Strings(names)
	if names[0] != "Alice" || names[1] != "Bob" {
		t.Fatalf("unexpected names: %v", names)
	}

	// Full node RETURN — exercises the node __type/labels/props JSON shape.
	nodeRows, err := db.Execute("MATCH (n:Person {name: 'Alice'}) RETURN n")
	if err != nil {
		t.Fatalf("MATCH n: %v", err)
	}
	if len(nodeRows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(nodeRows))
	}
	node := nodeRows[0]["n"].(map[string]any)
	if node["__type"] != "node" {
		t.Fatalf("expected __type node, got %v", node["__type"])
	}
	labels := node["labels"].([]any)
	if len(labels) != 1 || labels[0] != "Person" {
		t.Fatalf("unexpected labels: %v", labels)
	}
	props := node["props"].(map[string]any)
	if props["name"] != "Alice" {
		t.Fatalf("unexpected props: %v", props)
	}
	if props["age"].(int64) != 30 {
		t.Fatalf("unexpected age: %v", props["age"])
	}

	t.Logf("name rows: %+v", rows)
	t.Logf("node row: %+v", nodeRows)
}

func TestOpenOnDisk(t *testing.T) {
	dir := t.TempDir()
	path := dir + "/test.marsdb"

	db, err := Open(path)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	if _, err := db.Execute("CREATE (:Thing {n: 1})"); err != nil {
		t.Fatalf("CREATE: %v", err)
	}
	if err := db.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	// Reopen and confirm the write persisted.
	db2, err := Open(path)
	if err != nil {
		t.Fatalf("reopen: %v", err)
	}
	defer db2.Close()
	rows, err := db2.Execute("MATCH (t:Thing) RETURN t.n AS n")
	if err != nil {
		t.Fatalf("MATCH after reopen: %v", err)
	}
	if len(rows) != 1 || rows[0]["n"].(int64) != 1 {
		t.Fatalf("unexpected rows after reopen: %+v", rows)
	}
}

func TestExecutePreservesIntegersAndFormatsTemporals(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	rows, err := db.Execute("RETURN 9223372036854775807 AS max, -9223372036854775808 AS min, 1.5 AS f, 1.0 AS whole_float, date('1984-10-11') AS d, duration('P1M2DT3H4M5.006S') AS dur")
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}
	if len(rows) != 1 {
		t.Fatalf("expected one row, got %d", len(rows))
	}
	row := rows[0]
	if row["max"] != int64(9223372036854775807) {
		t.Fatalf("max integer lost precision: %T(%v)", row["max"], row["max"])
	}
	if row["min"] != int64(-9223372036854775808) {
		t.Fatalf("min integer lost precision: %T(%v)", row["min"], row["min"])
	}
	if row["f"] != float64(1.5) {
		t.Fatalf("unexpected float: %T(%v)", row["f"], row["f"])
	}
	if row["whole_float"] != float64(1.0) {
		t.Fatalf("whole float changed type: %T(%v)", row["whole_float"], row["whole_float"])
	}
	if row["d"] != "1984-10-11" || row["dur"] != "P1M2DT3H4M5.006S" {
		t.Fatalf("unexpected temporal values: %+v", row)
	}
}

func TestListValuedNodePropertyRoundTrips(t *testing.T) {
	// A stored PropertyValue::List (real Cypher/Neo4j's own "homogeneous
	// array property" shape) -- confirms the C-ABI's JSON array for a
	// list property decodes through encoding/json with no Go-side
	// special-casing needed.
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	if _, err := db.Execute("CREATE (:Person {name: 'Ada', tags: [1, 2, 3]})"); err != nil {
		t.Fatalf("CREATE: %v", err)
	}
	rows, err := db.Execute("MATCH (p:Person) RETURN p.tags AS tags")
	if err != nil {
		t.Fatalf("MATCH: %v", err)
	}
	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}
	tags := rows[0]["tags"].([]any)
	if len(tags) != 3 || tags[0].(int64) != 1 || tags[1].(int64) != 2 || tags[2].(int64) != 3 {
		t.Fatalf("unexpected tags: %+v", tags)
	}
}

func TestExecuteErrorSurfacesAsGoError(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	if _, err := db.Execute("NOT VALID CYPHER ((("); err == nil {
		t.Fatal("expected an error for invalid Cypher, got nil")
	}
	if _, err := db.Execute("RETURN -9223372036854775808 / -1"); err == nil {
		t.Fatal("expected an error for integer overflow, got nil")
	}
}

func TestTransactionStatements(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	mustExec := func(cypher string) {
		t.Helper()
		if _, err := db.Execute(cypher); err != nil {
			t.Fatalf("Execute(%q): %v", cypher, err)
		}
	}
	countNodes := func() int {
		t.Helper()
		rows, err := db.Execute("MATCH (n:N) RETURN n")
		if err != nil {
			t.Fatalf("count query: %v", err)
		}
		return len(rows)
	}

	mustExec("BEGIN")
	mustExec("CREATE (:N)")
	// Reads inside the transaction see its own uncommitted writes.
	if got := countNodes(); got != 1 {
		t.Fatalf("expected 1 node inside transaction, got %d", got)
	}
	mustExec("ROLLBACK")
	if got := countNodes(); got != 0 {
		t.Fatalf("expected rollback to discard the node, got %d", got)
	}
	mustExec("BEGIN TRANSACTION")
	mustExec("CREATE (:N)")
	mustExec("COMMIT")
	if got := countNodes(); got != 1 {
		t.Fatalf("expected 1 node after commit, got %d", got)
	}
}

func TestSchemaIntrospectionProcedures(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	for _, cypher := range []string{
		"CREATE INDEX ON :Person(name) UNIQUE",
		"CREATE (a:Person {name: 'Ada'})-[:KNOWS {since: 1980}]->(b:Person {name: 'Lin'})",
	} {
		if _, err := db.Execute(cypher); err != nil {
			t.Fatalf("Execute(%q): %v", cypher, err)
		}
	}

	labels, err := db.Execute("CALL db.labels()")
	if err != nil {
		t.Fatalf("db.labels(): %v", err)
	}
	if len(labels) != 1 || labels[0]["label"] != "Person" || labels[0]["count"] != int64(2) {
		t.Fatalf("unexpected labels: %+v", labels)
	}

	types, err := db.Execute("CALL db.relationshipTypes()")
	if err != nil {
		t.Fatalf("db.relationshipTypes(): %v", err)
	}
	if len(types) != 1 || types[0]["relationshipType"] != "KNOWS" || types[0]["count"] != int64(1) {
		t.Fatalf("unexpected relationship types: %+v", types)
	}

	indexes, err := db.Execute("CALL db.indexes()")
	if err != nil {
		t.Fatalf("db.indexes(): %v", err)
	}
	if len(indexes) != 1 || indexes[0]["label"] != "Person" ||
		indexes[0]["property"] != "name" || indexes[0]["unique"] != true {
		t.Fatalf("unexpected indexes: %+v", indexes)
	}
}

func TestExecuteWithParams(t *testing.T) {
	db, err := InMemory()
	if err != nil {
		t.Fatalf("InMemory: %v", err)
	}
	defer db.Close()

	if _, err := db.ExecuteWithParams(
		"CREATE (:Person {name: $name, age: $age, score: $score, tags: $tags})",
		map[string]any{
			"name":  `O'Hara "Ada"`,
			"age":   int64(9223372036854775807),
			"score": 1.5,
			"tags":  []any{int64(1), int64(2)},
		},
	); err != nil {
		t.Fatalf("ExecuteWithParams(create): %v", err)
	}

	rows, err := db.ExecuteWithParams(
		"MATCH (p:Person {name: $name}) RETURN p.age AS age, p.score AS score, p.tags AS tags",
		map[string]any{"name": `O'Hara "Ada"`},
	)
	if err != nil {
		t.Fatalf("ExecuteWithParams(match): %v", err)
	}
	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}
	if rows[0]["age"] != int64(9223372036854775807) {
		t.Fatalf("age lost precision: %T(%v)", rows[0]["age"], rows[0]["age"])
	}
	if rows[0]["score"] != float64(1.5) {
		t.Fatalf("unexpected score: %v", rows[0]["score"])
	}

	// Map-valued param.
	rows, err = db.ExecuteWithParams("RETURN $m.city AS city", map[string]any{
		"m": map[string]any{"city": "Kyoto"},
	})
	if err != nil {
		t.Fatalf("map param: %v", err)
	}
	if rows[0]["city"] != "Kyoto" {
		t.Fatalf("unexpected city: %v", rows[0]["city"])
	}

	// A uint64 above int64's range must error, never silently round.
	if _, err := db.ExecuteWithParams("RETURN $x AS x", map[string]any{
		"x": uint64(18446744073709551615),
	}); err == nil {
		t.Fatal("expected an error for uint64 above i64 range, got nil")
	}

	// Missing parameter surfaces the engine's error.
	if _, err := db.ExecuteWithParams("RETURN $missing AS m", map[string]any{}); err == nil {
		t.Fatal("expected an error for a missing parameter, got nil")
	}
}
