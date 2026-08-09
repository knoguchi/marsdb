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
