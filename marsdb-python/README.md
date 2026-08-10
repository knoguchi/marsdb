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

## Errors

Everything raised derives from `marsdb.Error`; subclasses expose the
engine's own taxonomy so programs catch selectively instead of
string-matching:

| Exception | Raised for |
|---|---|
| `ProgrammingError` | syntax/semantic errors, unbound variables, missing `$params`, stray `COMMIT` |
| `DataError` | type errors, integer overflow, unstorable parameter values |
| `IntegrityError` | unique-index violations, deleting a connected node without `DETACH` |
| `OperationalError` | timeout, cancellation, `max_rows` exceeded, storage failures |

## Execution bounds

```python
db.execute("MATCH (n) RETURN n", max_rows=100_000, timeout_ms=5_000)
```

Both are checked *during* evaluation — a runaway query raises
`OperationalError` at the bound instead of materializing an unbounded
result first, so it can't OOM the process.

## Streaming (bulk export)

```python
db.execute_streaming(
    "MATCH (n:Person) RETURN n.name AS name",
    lambda row: writer.writerow(row),   # return False to stop early
)
```

Rows are pushed one at a time — bounded memory no matter how many rows
match. Accepts exactly the streamable shape (one plain `MATCH ...
RETURN`, `SKIP`/`LIMIT` fine) and raises `ProgrammingError` for `ORDER
BY`/aggregation/`DISTINCT`/`WITH` — those must see all rows before
emitting any, so streaming them would be pretend; use `execute`.

## Arrow (pyarrow / polars / pandas / DuckDB)

`query_arrow` returns the result as an Arrow stream — an object
implementing the Arrow PyCapsule protocol, accepted directly by any
Arrow consumer with zero per-value conversion:

```python
import pyarrow as pa

table = pa.table(db.query_arrow("MATCH (n:Person) RETURN n.name AS name, n.age AS age"))
df = table.to_pandas()          # or polars.from_arrow(table), duckdb.sql(...)
```

Column types are inferred strictly, per column over the whole result:
`int64` (full 64-bit, exact), `float64`, `string`, `bool`, `date32`,
month-day-nano interval for durations, ISO-8601 text for other
temporals, lists of one element type. A column mixing ints and floats
raises `DataError` — silent promotion to float would corrupt integers
beyond 2⁵³; cast in the query (`toFloat`/`toInteger`) instead. So do
node/relationship/map/path columns: project scalar properties.

The stream is single-use (hand it to one consumer); `batch_rows`
(default 8192) sets rows per record batch, and `.stats` carries the
statement's write counters. `pyarrow` itself is **not** a dependency of
this package — anything speaking the protocol works.

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
