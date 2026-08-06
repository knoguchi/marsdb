# Python bindings

```
pip install marsdb
```

```python
import marsdb
db = marsdb.Database.in_memory()  # or .open(path)
db.execute("CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})")
db.execute("MATCH (n:Person) RETURN n.name")
# -> [{'n.name': 'Alice'}, {'n.name': 'Bob'}]
```

Prebuilt wheels cover macOS (arm64, x86_64) and Linux (x86_64, manylinux);
other platforms install from the source distribution and need a Rust
toolchain. Built via [PyO3](https://pyo3.rs) — in-process, no separate
server or IPC.

## Build from source

```
cd marsdb-python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin && maturin develop
```
