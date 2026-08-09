package marsdb

import "testing"

func benchDB(b *testing.B, rows int) *Database {
	b.Helper()
	db, err := InMemory()
	if err != nil {
		b.Fatal(err)
	}
	if _, err := db.Execute("BEGIN"); err != nil {
		b.Fatal(err)
	}
	for i := 0; i < rows; i++ {
		if _, err := db.ExecuteWithParams(
			"CREATE (:N {i: $i, name: $s, score: $f})",
			map[string]any{"i": i, "s": "user-with-a-name", "f": 1.5},
		); err != nil {
			b.Fatal(err)
		}
	}
	if _, err := db.Execute("COMMIT"); err != nil {
		b.Fatal(err)
	}
	return db
}

// Batch lane: one cgo crossing, Rust-side encode + Go-side decode.
func BenchmarkBatchLane10k(b *testing.B) {
	db := benchDB(b, 10000)
	defer db.Close()
	b.ResetTimer()
	for b.Loop() {
		rows, err := db.Execute("MATCH (n:N) RETURN n.i AS i, n.name AS name, n.score AS score")
		if err != nil {
			b.Fatal(err)
		}
		if len(rows) != 10000 {
			b.Fatal(len(rows))
		}
	}
}

// Typed-accessor lane: per-value cgo crossings, no codec.
// (ExecuteStreaming decodes each row via the C value accessors.)
func BenchmarkAccessorLane10k(b *testing.B) {
	db := benchDB(b, 10000)
	defer db.Close()
	b.ResetTimer()
	for b.Loop() {
		count := 0
		err := db.ExecuteStreaming(
			"MATCH (n:N) RETURN n.i AS i, n.name AS name, n.score AS score",
			nil, Options{}, func(row map[string]any) bool {
				count++
				return true
			})
		if err != nil {
			b.Fatal(err)
		}
		if count != 10000 {
			b.Fatal(count)
		}
	}
}

// Node-heavy shape: full nodes with props (nested decode both lanes).
func BenchmarkBatchLaneNodes10k(b *testing.B) {
	db := benchDB(b, 10000)
	defer db.Close()
	b.ResetTimer()
	for b.Loop() {
		rows, err := db.Execute("MATCH (n:N) RETURN n")
		if err != nil {
			b.Fatal(err)
		}
		if len(rows) != 10000 {
			b.Fatal(len(rows))
		}
	}
}

func BenchmarkAccessorLaneNodes10k(b *testing.B) {
	db := benchDB(b, 10000)
	defer db.Close()
	b.ResetTimer()
	for b.Loop() {
		count := 0
		err := db.ExecuteStreaming("MATCH (n:N) RETURN n", nil, Options{},
			func(row map[string]any) bool { count++; return true })
		if err != nil {
			b.Fatal(err)
		}
		if count != 10000 {
			b.Fatal(count)
		}
	}
}
