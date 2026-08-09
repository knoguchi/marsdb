# marsdb (Python)

Python bindings for [MarsDB](https://github.com/knoguchi/marsdb), an
embeddable property-graph database with an openCypher query subset:
single file on disk (or fully in-memory), no server, ACID transactions.
In-process via PyO3 — no C ABI, no sockets, queries run on the calling
thread.

**Not published to PyPI yet** — build from the repo with
[maturin](https://github.com/PyO3/maturin):

```
cd marsdb-python
maturin develop --release
```

## Quickstart

```python
import marsdb

db = marsdb.Database.in_memory()          # or marsdb.Database.open("graph.db")
db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS {since: 2001}]->(b:Person {name: 'Bob'})")

for row in db.execute("MATCH (a:Person)-[:KNOWS]->(b) RETURN a.name, b.name"):
    print(row["a.name"], "knows", row["b.name"])
```

`execute` runs one Cypher statement and returns a `list` of `dict`s, one
per result row, keyed by column name.

## Parameterized queries

`execute` takes an optional `params` dict resolving `$name` placeholders
— no string interpolation, no escaping bugs:

```python
db.execute(
    "MATCH (p:Person {name: $name}) RETURN p.age",
    {"name": "O'Hara"},
)
```

Values may be `None`/`bool`/`int`/`float`/`str`, or nested `list`/`dict`
of those. Ints keep their full 64-bit range; an int outside i64 raises
instead of truncating. Map-valued params work (`$m.city`).

## Value mapping

| Cypher | Python |
|---|---|
| integer | `int` (full 64-bit, no precision loss) |
| float / string / boolean / null | `float` / `str` / `bool` / `None` |
| list / map | `list` / `dict` |
| node | `{"id": ..., "labels": [...], "props": {...}}` |
| relationship | `{"id": ..., "label": ..., "src": ..., "dst": ..., "props": {...}}` |
| date / duration | ISO-8601 `str` |

## Transactions

`BEGIN` / `COMMIT` / `ROLLBACK` are statements (`BEGIN TRANSACTION`
also accepted). One session per `Database` handle; reads inside a
transaction see its own uncommitted writes; a statement that fails at
execution time rolls the whole transaction back.

```python
db.execute("BEGIN")
db.execute("CREATE (:Account {id: 1, balance: 100})")
db.execute("CREATE (:Account {id: 2, balance: 0})")
db.execute("COMMIT")          # or ROLLBACK to discard both
```

## Schema introspection

```python
db.execute("CALL db.labels()")             # [{'label': 'Person', 'count': 2}]
db.execute("CALL db.relationshipTypes()")  # [{'relationshipType': 'KNOWS', 'count': 1}]
db.execute("CALL db.propertyKeys()")       # [{'propertyKey': 'name'}, ...]
db.execute("CALL db.indexes()")            # [{'label': ..., 'property': ..., 'unique': ...}]
```

## Examples

- [`examples/commit_graph.py`](https://github.com/knoguchi/marsdb/blob/main/marsdb-python/examples/commit_graph.py)
  — build a graph of a git repo's history (commits, files, `TOUCHES`
  edges).
- [`examples/visualize_commit_graph.py`](https://github.com/knoguchi/marsdb/blob/main/marsdb-python/examples/visualize_commit_graph.py)
  — render it as a PNG.

## Tests

```
maturin develop && python -m unittest discover tests
```

Cypher coverage, benchmarks, and the full manual live in the
[main repository](https://github.com/knoguchi/marsdb).
