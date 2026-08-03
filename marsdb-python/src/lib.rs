use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// A MarsDB database: either a single on-disk file or a transient
/// in-memory instance.
#[pyclass]
struct Database {
    inner: ::marsdb::Database,
}

#[pymethods]
impl Database {
    /// Open (creating if absent) a single-file, on-disk database.
    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        let inner = ::marsdb::Database::open(path).map_err(to_py_err)?;
        Ok(Self { inner })
    }

    /// Open a purely in-memory database. Nothing is written to disk.
    #[staticmethod]
    fn in_memory() -> PyResult<Self> {
        let inner = ::marsdb::Database::in_memory().map_err(to_py_err)?;
        Ok(Self { inner })
    }

    /// Run one Cypher statement, returning a list of dicts (column name ->
    /// value) — one dict per matched row. `CREATE`/`DELETE`/`SET`
    /// statements return an empty list.
    fn execute<'py>(&self, py: Python<'py>, cypher: &str) -> PyResult<Bound<'py, PyList>> {
        let result = self.inner.execute(cypher).map_err(to_py_err)?;
        let rows = PyList::empty(py);
        for row in &result.rows {
            let dict = PyDict::new(py);
            for (col, value) in result.columns.iter().zip(row.iter()) {
                dict.set_item(col, value_to_py(py, value)?)?;
            }
            rows.append(dict)?;
        }
        Ok(rows)
    }
}

fn to_py_err(e: ::marsdb::Error) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

fn value_to_py<'py>(py: Python<'py>, value: &::marsdb::Value) -> PyResult<Bound<'py, PyAny>> {
    Ok(match value {
        ::marsdb::Value::Null => py.None().into_bound(py),
        ::marsdb::Value::Property(p) => property_to_py(py, p)?,
        ::marsdb::Value::Literal(l) => literal_to_py(py, l)?,
        ::marsdb::Value::Node(n) => {
            let dict = PyDict::new(py);
            dict.set_item("id", n.id.0)?;
            dict.set_item("labels", n.labels.clone())?;
            let props = PyDict::new(py);
            for (k, v) in &n.props {
                props.set_item(k, property_to_py(py, v)?)?;
            }
            dict.set_item("props", props)?;
            dict.into_any()
        }
        ::marsdb::Value::Edge(e) => {
            let dict = PyDict::new(py);
            dict.set_item("id", e.id.0)?;
            dict.set_item("label", &e.label)?;
            dict.set_item("src", e.src.0)?;
            dict.set_item("dst", e.dst.0)?;
            let props = PyDict::new(py);
            for (k, v) in &e.props {
                props.set_item(k, property_to_py(py, v)?)?;
            }
            dict.set_item("props", props)?;
            dict.into_any()
        }
        ::marsdb::Value::List(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(value_to_py(py, item)?)?;
            }
            list.into_any()
        }
        // A path is a list of node/edge dicts, alternating -- the same
        // dict shape Value::Node/Edge above already produce, so a caller
        // walking a path sees exactly what they'd see walking `RETURN`ed
        // nodes/edges individually.
        ::marsdb::Value::Path(elems) => {
            let list = PyList::empty(py);
            for elem in elems {
                let value = match elem {
                    ::marsdb::PathElem::Node(n) => ::marsdb::Value::Node(n.clone()),
                    ::marsdb::PathElem::Edge(e) => ::marsdb::Value::Edge(e.clone()),
                };
                list.append(value_to_py(py, &value)?)?;
            }
            list.into_any()
        }
    })
}

fn property_to_py<'py>(py: Python<'py>, p: &marsdb_graph::PropertyValue) -> PyResult<Bound<'py, PyAny>> {
    Ok(match p {
        marsdb_graph::PropertyValue::Null => py.None().into_bound(py),
        marsdb_graph::PropertyValue::Bool(b) => b.into_pyobject(py)?.to_owned().into_any(),
        marsdb_graph::PropertyValue::Int(i) => i.into_pyobject(py)?.into_any(),
        marsdb_graph::PropertyValue::Float(f) => f.into_pyobject(py)?.into_any(),
        marsdb_graph::PropertyValue::String(s) => s.into_pyobject(py)?.into_any(),
    })
}

fn literal_to_py<'py>(py: Python<'py>, l: &::marsdb::Literal) -> PyResult<Bound<'py, PyAny>> {
    Ok(match l {
        ::marsdb::Literal::Null => py.None().into_bound(py),
        ::marsdb::Literal::Bool(b) => b.into_pyobject(py)?.to_owned().into_any(),
        ::marsdb::Literal::Int(i) => i.into_pyobject(py)?.into_any(),
        ::marsdb::Literal::Float(f) => f.into_pyobject(py)?.into_any(),
        ::marsdb::Literal::String(s) => s.into_pyobject(py)?.into_any(),
        ::marsdb::Literal::Param(name) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
    })
}

/// MarsDB: an embeddable property-graph database with an openCypher query
/// subset.
#[pymodule]
fn marsdb(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Database>()?;
    Ok(())
}
