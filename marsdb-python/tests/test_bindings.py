import unittest

import marsdb


class DatabaseTests(unittest.TestCase):
    def test_execute_converts_all_public_value_shapes(self):
        db = marsdb.Database.in_memory()
        rows = db.execute(
            "RETURN 9223372036854775807 AS integer, "
            "{answer: 42, nested: [true, null]} AS map, "
            "date('1984-10-11') AS date, "
            "duration('P1M2DT3H4M5.006S') AS duration"
        )

        self.assertEqual(
            rows,
            [
                {
                    "integer": 9223372036854775807,
                    "map": {"answer": 42, "nested": [True, None]},
                    "date": "1984-10-11",
                    "duration": "P1M2DT3H4M5.006S",
                }
            ],
        )

    def test_write_then_read_node(self):
        db = marsdb.Database.in_memory()
        self.assertEqual(db.execute("CREATE (:Person {name: 'Ada'})"), [])
        rows = db.execute("MATCH (p:Person) RETURN p")

        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["p"]["labels"], ["Person"])
        self.assertEqual(rows[0]["p"]["props"], {"name": "Ada"})

    def test_transaction_statements(self):
        db = marsdb.Database.in_memory()
        db.execute("BEGIN")
        db.execute("CREATE (:N)")
        # Reads inside the transaction see its own uncommitted writes.
        self.assertEqual(len(db.execute("MATCH (n:N) RETURN n")), 1)
        db.execute("ROLLBACK")
        self.assertEqual(db.execute("MATCH (n:N) RETURN n"), [])
        db.execute("BEGIN TRANSACTION")
        db.execute("CREATE (:N)")
        db.execute("COMMIT")
        self.assertEqual(len(db.execute("MATCH (n:N) RETURN n")), 1)

    def test_schema_introspection_procedures(self):
        db = marsdb.Database.in_memory()
        db.execute("CREATE INDEX ON :Person(name) UNIQUE")
        db.execute("CREATE (a:Person {name: 'Ada'})-[:KNOWS {since: 1980}]->(b:Person {name: 'Lin'})")

        self.assertEqual(
            db.execute("CALL db.labels()"),
            [{"label": "Person", "count": 2}],
        )
        self.assertEqual(
            db.execute("CALL db.relationshipTypes()"),
            [{"relationshipType": "KNOWS", "count": 1}],
        )
        self.assertEqual(
            db.execute("CALL db.propertyKeys()"),
            [{"propertyKey": "name"}, {"propertyKey": "since"}],
        )
        self.assertEqual(
            db.execute("CALL db.indexes()"),
            [{"label": "Person", "property": "name", "unique": True}],
        )


    def test_execute_with_params(self):
        db = marsdb.Database.in_memory()
        db.execute(
            "CREATE (:Person {name: $name, age: $age, score: $score, active: $active, tags: $tags})",
            {"name": "O'Hara \"Ada\"", "age": 9223372036854775807,
             "score": 1.5, "active": True, "tags": [1, 2, 3]},
        )
        rows = db.execute(
            "MATCH (p:Person {name: $name}) RETURN p.age, p.score, p.active, p.tags",
            {"name": "O'Hara \"Ada\""},
        )
        self.assertEqual(
            rows,
            [{"p.age": 9223372036854775807, "p.score": 1.5,
              "p.active": True, "p.tags": [1, 2, 3]}],
        )

    def test_param_errors(self):
        db = marsdb.Database.in_memory()
        # Missing param is the query's fault.
        with self.assertRaises(marsdb.ProgrammingError):
            db.execute("RETURN $missing")
        # Int beyond i64 must raise, never silently truncate.
        with self.assertRaises(marsdb.DataError):
            db.execute("RETURN $x", {"x": 2**63})
        # Unsupported value type named in the error.
        with self.assertRaises(marsdb.DataError) as ctx:
            db.execute("RETURN $x", {"x": object()})
        self.assertIn("unsupported parameter type", str(ctx.exception))

    def test_exception_taxonomy(self):
        db = marsdb.Database.in_memory()
        with self.assertRaises(marsdb.ProgrammingError):
            db.execute("NOT CYPHER (((")
        with self.assertRaises(marsdb.DataError):
            db.execute("RETURN 1 + 'x'")
        db.execute("CREATE INDEX ON :U(email) UNIQUE")
        db.execute("CREATE (:U {email: 'a@x'})")
        with self.assertRaises(marsdb.IntegrityError):
            db.execute("CREATE (:U {email: 'a@x'})")
        db.execute("CREATE (a:P)-[:R]->(b:P)")
        with self.assertRaises(marsdb.IntegrityError):
            db.execute("MATCH (p:P) DELETE p")
        with self.assertRaises(marsdb.ProgrammingError):
            db.execute("COMMIT")
        # Everything derives from marsdb.Error.
        with self.assertRaises(marsdb.Error):
            db.execute("garbage")

    def test_max_rows_bounds_the_result(self):
        db = marsdb.Database.in_memory()
        for i in range(10):
            db.execute("CREATE (:N {i: $i})", {"i": i})
        # Under the limit: fine.
        self.assertEqual(len(db.execute("MATCH (n:N) RETURN n", max_rows=10)), 10)
        # Over: OperationalError during evaluation, catchable as base too.
        with self.assertRaises(marsdb.OperationalError):
            db.execute("MATCH (n:N) RETURN n", max_rows=5)
        # LIMIT inside the query composes with the bound.
        rows = db.execute("MATCH (n:N) RETURN n LIMIT 3", max_rows=5)
        self.assertEqual(len(rows), 3)

    def test_map_valued_param(self):
        db = marsdb.Database.in_memory()
        rows = db.execute("RETURN $m.city AS city", {"m": {"city": "Kyoto"}})
        self.assertEqual(rows, [{"city": "Kyoto"}])


    def test_execute_with_stats(self):
        db = marsdb.Database.in_memory()
        rows, stats = db.execute_with_stats("CREATE (a:P {x: 1})-[:R]->(b:P)")
        self.assertEqual(rows, [])
        self.assertEqual(stats["nodes_created"], 2)
        self.assertEqual(stats["relationships_created"], 1)
        _, stats = db.execute_with_stats("MATCH (n:P) SET n.seen = true")
        self.assertEqual(stats["properties_set"], 2)
        _, stats = db.execute_with_stats("MATCH (a:P) DETACH DELETE a")
        self.assertEqual(stats["nodes_deleted"], 2)
        self.assertEqual(stats["relationships_deleted"], 1)
        _, stats = db.execute_with_stats("MATCH (n) RETURN n")
        self.assertTrue(all(v == 0 for v in stats.values()))


    def test_execute_streaming(self):
        db = marsdb.Database.in_memory()
        for i in range(20):
            db.execute("CREATE (:N {i: $i})", {"i": i})
        rows = []
        db.execute_streaming(
            "MATCH (n:N) RETURN n.i AS i SKIP 2 LIMIT 5",
            lambda row: rows.append(row),
        )
        self.assertEqual(len(rows), 5)
        # Returning False stops the scan early.
        stopped = []
        db.execute_streaming(
            "MATCH (n:N) RETURN n.i AS i",
            lambda row: (stopped.append(row), len(stopped) < 3)[1],
        )
        self.assertEqual(len(stopped), 3)
        # Non-streamable shapes raise instead of materializing.
        with self.assertRaises(marsdb.ProgrammingError):
            db.execute_streaming("MATCH (n:N) RETURN count(n)", lambda row: None)
        # A callback exception propagates as itself.
        with self.assertRaises(ZeroDivisionError):
            db.execute_streaming("MATCH (n:N) RETURN n.i", lambda row: 1 / 0)

    def test_list_valued_node_property_round_trips(self):
        # A stored PropertyValue::List (real Cypher/Neo4j's own
        # "homogeneous array property" shape), not the query-layer-only
        # list literal the other test above already covers.
        db = marsdb.Database.in_memory()
        db.execute("CREATE (:Person {name: 'Ada', tags: [1, 2, 3]})")
        rows = db.execute("MATCH (p:Person) RETURN p.tags")

        self.assertEqual(rows, [{"p.tags": [1, 2, 3]}])


try:
    import pyarrow as pa

    HAVE_PYARROW = True
except ImportError:  # dev-only dependency; the binding itself needs none
    HAVE_PYARROW = False


@unittest.skipUnless(HAVE_PYARROW, "pyarrow not installed")
class TestArrowExport(unittest.TestCase):
    def test_pycapsule_protocol_round_trip(self):
        import datetime

        db = marsdb.Database.in_memory()
        db.execute(
            "CREATE (:N {i: 9223372036854775807, f: 1.5, s: 'héllo', b: true, "
            "d: date('1984-10-11'), tags: [1, 2, 3]})"
        )
        db.execute("CREATE (:N {i: 2, f: 2.5, s: 'x', b: false, tags: []})")
        table = pa.table(
            db.query_arrow(
                "MATCH (n:N) RETURN n.i AS i, n.f AS f, n.s AS s, n.b AS b, "
                "n.d AS d, n.tags AS tags"
            )
        )
        self.assertEqual(table.schema.field("i").type, pa.int64())
        self.assertEqual(table.schema.field("f").type, pa.float64())
        self.assertEqual(table.schema.field("s").type, pa.string())
        self.assertEqual(table.schema.field("b").type, pa.bool_())
        self.assertEqual(table.schema.field("d").type, pa.date32())
        self.assertEqual(table.schema.field("tags").type, pa.list_(pa.int64()))
        # int64 exactness: i64::MAX survives (a float path would corrupt it).
        self.assertIn(9223372036854775807, table.column("i").to_pylist())
        self.assertIn(datetime.date(1984, 10, 11), table.column("d").to_pylist())
        # The node without `d` becomes a null, not a type change.
        self.assertIn(None, table.column("d").to_pylist())

    def test_stream_is_single_use(self):
        db = marsdb.Database.in_memory()
        db.execute("CREATE (:N {i: 1})")
        res = db.query_arrow("MATCH (n:N) RETURN n.i AS i")
        pa.table(res)
        with self.assertRaises(marsdb.ProgrammingError):
            pa.table(res)

    def test_strict_inference_errors(self):
        db = marsdb.Database.in_memory()
        db.execute("CREATE (:P {x: 1}), (:P {x: 2.5})")
        # Mixed Int/Float raises instead of silently promoting.
        with self.assertRaises(marsdb.DataError):
            db.query_arrow("MATCH (p:P) RETURN p.x AS mixed")
        # Whole entities are not exportable.
        with self.assertRaises(marsdb.DataError):
            db.query_arrow("MATCH (p:P) RETURN p")

    def test_batching_stats_and_bounds(self):
        db = marsdb.Database.in_memory()
        for i in range(7):
            db.execute("CREATE (:B {i: $i})", params={"i": i})
        reader = pa.RecordBatchReader.from_stream(
            db.query_arrow("MATCH (b:B) RETURN b.i AS i", batch_rows=3)
        )
        self.assertEqual([len(b) for b in reader], [3, 3, 1])
        res = db.query_arrow("CREATE (:M)")
        self.assertEqual(res.stats["nodes_created"], 1)
        with self.assertRaises(marsdb.OperationalError):
            db.query_arrow("MATCH (b:B) RETURN b.i", max_rows=2)


if __name__ == "__main__":
    unittest.main()
