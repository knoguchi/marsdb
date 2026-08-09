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

    def test_list_valued_node_property_round_trips(self):
        # A stored PropertyValue::List (real Cypher/Neo4j's own
        # "homogeneous array property" shape), not the query-layer-only
        # list literal the other test above already covers.
        db = marsdb.Database.in_memory()
        db.execute("CREATE (:Person {name: 'Ada', tags: [1, 2, 3]})")
        rows = db.execute("MATCH (p:Person) RETURN p.tags")

        self.assertEqual(rows, [{"p.tags": [1, 2, 3]}])


if __name__ == "__main__":
    unittest.main()
