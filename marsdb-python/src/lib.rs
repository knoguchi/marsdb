use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// PEP 249-inspired exception hierarchy (the useful subset, not the
// ceremony): everything raised by MarsDB derives from `marsdb.Error`, so
// `except marsdb.Error` catches all of it, and the subclasses expose the
// engine's own error taxonomy (`QueryError`/`GraphError` variants) that
// the old flat RuntimeError threw away — programs can catch selectively
// instead of string-matching messages.
pyo3::create_exception!(
    marsdb,
    Error,
    pyo3::exceptions::PyException,
    "Base class for every MarsDB error."
);
pyo3::create_exception!(
    marsdb,
    ProgrammingError,
    Error,
    "The query itself is at fault: syntax error, semantic error, unbound \
     variable, missing $parameter, or misuse of an API (e.g. COMMIT with \
     no open transaction)."
);
pyo3::create_exception!(
    marsdb,
    DataError,
    Error,
    "A value's type or range is at fault: arithmetic on a non-number, \
     integer overflow, an unstorable parameter value."
);
pyo3::create_exception!(
    marsdb,
    OperationalError,
    Error,
    "The operation failed for runtime reasons outside the query text: \
     timeout, cancellation, a resource limit (max_rows), storage-level \
     failure, or a closed/timed-out transaction."
);
pyo3::create_exception!(
    marsdb,
    IntegrityError,
    Error,
    "A graph integrity rule rejected the write: unique-index violation, \
     or deleting a node that still has relationships without DETACH."
);

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
    /// statements return an empty list. `params`, when given, resolves
    /// `$name` placeholders: values may be None/bool/int/float/str, or
    /// (arbitrarily nested) lists and dicts of those. Ints keep their
    /// full 64-bit range; an int outside i64 raises.
    ///
    /// `max_rows` bounds the result: exceeding it raises
    /// `OperationalError` *during* evaluation — the query never
    /// materializes an unbounded result first, so a runaway query can't
    /// OOM the process. `timeout_ms` similarly bounds wall time
    /// (checked cooperatively during evaluation).
    #[pyo3(signature = (cypher, params = None, max_rows = None, timeout_ms = None))]
    fn execute<'py>(
        &self,
        py: Python<'py>,
        cypher: &str,
        params: Option<&Bound<'py, PyDict>>,
        max_rows: Option<usize>,
        timeout_ms: Option<u64>,
    ) -> PyResult<Bound<'py, PyList>> {
        let result = self.run(cypher, params, max_rows, timeout_ms)?;
        rows_to_py(py, &result)
    }

    /// `execute` plus the statement's write counters — the answer to
    /// "how many did my DELETE delete". Returns `(rows, stats)` where
    /// `stats` is a dict with `nodes_created`, `nodes_deleted`,
    /// `relationships_created`, `relationships_deleted`,
    /// `properties_set`, `labels_added`, `labels_removed` — all zero
    /// for read-only statements.
    #[pyo3(signature = (cypher, params = None, max_rows = None, timeout_ms = None))]
    fn execute_with_stats<'py>(
        &self,
        py: Python<'py>,
        cypher: &str,
        params: Option<&Bound<'py, PyDict>>,
        max_rows: Option<usize>,
        timeout_ms: Option<u64>,
    ) -> PyResult<(Bound<'py, PyList>, Bound<'py, PyDict>)> {
        let result = self.run(cypher, params, max_rows, timeout_ms)?;
        let stats = PyDict::new(py);
        let s = &result.stats;
        stats.set_item("nodes_created", s.nodes_created)?;
        stats.set_item("nodes_deleted", s.nodes_deleted)?;
        stats.set_item("relationships_created", s.relationships_created)?;
        stats.set_item("relationships_deleted", s.relationships_deleted)?;
        stats.set_item("properties_set", s.properties_set)?;
        stats.set_item("labels_added", s.labels_added)?;
        stats.set_item("labels_removed", s.labels_removed)?;
        Ok((rows_to_py(py, &result)?, stats))
    }

    /// Stream a read-only statement's rows to `on_row` instead of
    /// materializing a list — bounded memory no matter how many rows
    /// match; the bulk-export path. `on_row` receives one dict per row
    /// (same shape as an `execute` row); return `False` to stop the
    /// scan early. Accepts exactly the streamable shape (one plain
    /// `MATCH ... RETURN`, `SKIP`/`LIMIT` fine) and raises
    /// `ProgrammingError` — never silently materializes — for ORDER
    /// BY/aggregation/DISTINCT/WITH/write statements.
    #[pyo3(signature = (cypher, on_row, params = None, max_rows = None, timeout_ms = None))]
    fn execute_streaming<'py>(
        &self,
        py: Python<'py>,
        cypher: &str,
        on_row: &Bound<'py, PyAny>,
        params: Option<&Bound<'py, PyDict>>,
        max_rows: Option<usize>,
        timeout_ms: Option<u64>,
    ) -> PyResult<()> {
        if !on_row.is_callable() {
            return Err(ProgrammingError::new_err("on_row must be callable"));
        }
        let mut converted = std::collections::HashMap::new();
        if let Some(dict) = params {
            for (key, value) in dict.iter() {
                let name: String = key
                    .extract()
                    .map_err(|_| ProgrammingError::new_err("parameter names must be strings"))?;
                let prop = py_to_property(&value)
                    .map_err(|e| DataError::new_err(format!("parameter '{name}': {e}")))?;
                converted.insert(name, prop);
            }
        }
        let mut options = ::marsdb::ExecutionOptions::default();
        options.max_result_rows = max_rows;
        options.timeout = timeout_ms.map(std::time::Duration::from_millis);

        struct PySink<'a, 'py> {
            py: Python<'py>,
            on_row: &'a Bound<'py, PyAny>,
            columns: Vec<String>,
            failure: Option<PyErr>,
        }
        impl ::marsdb::RowSink for PySink<'_, '_> {
            fn columns(&mut self, columns: &[String]) {
                self.columns = columns.to_vec();
            }
            fn row(&mut self, row: Vec<::marsdb::Value>) -> std::ops::ControlFlow<()> {
                let build = || -> PyResult<Bound<'_, PyDict>> {
                    let dict = PyDict::new(self.py);
                    for (col, value) in self.columns.iter().zip(row.iter()) {
                        dict.set_item(col, value_to_py(self.py, value)?)?;
                    }
                    Ok(dict)
                };
                let dict = match build() {
                    Ok(d) => d,
                    Err(e) => {
                        self.failure = Some(e);
                        return std::ops::ControlFlow::Break(());
                    }
                };
                match self.on_row.call1((dict,)) {
                    // Only an explicit `False` stops the scan -- `None`
                    // (a bare callback with no return) keeps going.
                    Ok(value) => {
                        if matches!(value.extract::<bool>(), Ok(false)) {
                            std::ops::ControlFlow::Break(())
                        } else {
                            std::ops::ControlFlow::Continue(())
                        }
                    }
                    Err(e) => {
                        self.failure = Some(e);
                        std::ops::ControlFlow::Break(())
                    }
                }
            }
        }

        let mut sink = PySink {
            py,
            on_row,
            columns: vec![],
            failure: None,
        };
        self.inner
            .execute_streaming(cypher, &converted, &options, &mut sink)
            .map_err(to_py_err)?;
        match sink.failure {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl Database {
    fn run(
        &self,
        cypher: &str,
        params: Option<&Bound<'_, PyDict>>,
        max_rows: Option<usize>,
        timeout_ms: Option<u64>,
    ) -> PyResult<::marsdb::QueryResult> {
        let mut converted = std::collections::HashMap::new();
        if let Some(dict) = params {
            for (key, value) in dict.iter() {
                let name: String = key
                    .extract()
                    .map_err(|_| ProgrammingError::new_err("parameter names must be strings"))?;
                let prop = py_to_property(&value)
                    .map_err(|e| DataError::new_err(format!("parameter '{name}': {e}")))?;
                converted.insert(name, prop);
            }
        }
        let mut options = ::marsdb::ExecutionOptions::default();
        options.max_result_rows = max_rows;
        options.timeout = timeout_ms.map(std::time::Duration::from_millis);
        self.inner
            .execute_with_params_and_options(cypher, &converted, &options)
            .map_err(to_py_err)
    }
}

fn rows_to_py<'py>(
    py: Python<'py>,
    result: &::marsdb::QueryResult,
) -> PyResult<Bound<'py, PyList>> {
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

/// Python -> `PropertyValue` for `$param` values — the inverse of the
/// output mapping (`property_to_py`), minus temporal types (pass those
/// as ISO strings and construct with `date($p)`/`duration($p)` in the
/// query). `bool` is checked before `int`: Python's `bool` IS an `int`
/// subclass, and an `extract::<i64>` on `True` would happily produce 1.
fn py_to_property(value: &Bound<'_, PyAny>) -> Result<::marsdb::PropertyValue, String> {
    use ::marsdb::PropertyValue;
    if value.is_none() {
        return Ok(PropertyValue::Null);
    }
    if let Ok(b) = value.extract::<bool>() {
        return Ok(PropertyValue::Bool(b));
    }
    if value.is_instance_of::<pyo3::types::PyInt>() {
        return value
            .extract::<i64>()
            .map(PropertyValue::Int)
            .map_err(|_| "int is outside the 64-bit signed range".to_string());
    }
    if let Ok(f) = value.extract::<f64>() {
        if value.is_instance_of::<pyo3::types::PyFloat>() {
            return Ok(PropertyValue::Float(f));
        }
    }
    if let Ok(s) = value.extract::<String>() {
        return Ok(PropertyValue::String(s));
    }
    if let Ok(list) = value.cast::<PyList>() {
        let items: Result<Vec<_>, _> = list.iter().map(|item| py_to_property(&item)).collect();
        return Ok(PropertyValue::List(items?));
    }
    if let Ok(dict) = value.cast::<PyDict>() {
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in dict.iter() {
            let key: String = k
                .extract()
                .map_err(|_| "map keys must be strings".to_string())?;
            map.insert(key, py_to_property(&v)?);
        }
        return Ok(PropertyValue::Map(map));
    }
    Err(format!(
        "unsupported parameter type {} -- use None/bool/int/float/str or nested list/dict",
        value
            .get_type()
            .name()
            .map_or_else(|_| "<unknown>".to_string(), |n| n.to_string())
    ))
}

/// `marsdb::Error` -> the exception hierarchy above. The interesting
/// mappings: `Type` errors are the value's fault (`DataError`);
/// syntax/semantic/unbound/missing-param are the query's fault
/// (`ProgrammingError`); unique-index violations and non-detach deletes
/// of connected nodes are the graph-integrity analog of a relational
/// constraint failure (`IntegrityError`); everything environmental —
/// timeouts, cancellation, resource limits, storage errors, transaction
/// lifecycle — is `OperationalError`.
fn to_py_err(e: ::marsdb::Error) -> PyErr {
    use ::marsdb::{Error as E, QueryError as Q};
    let message = e.to_string();
    let query_err = |q: &Q, message: String| match q {
        Q::Syntax(_) | Q::Semantic(_) | Q::UnboundVariable(_) | Q::MissingParam(_) => {
            ProgrammingError::new_err(message)
        }
        Q::Type(_) => DataError::new_err(message),
        Q::Graph(g) => graph_err(g, message),
        Q::Cancelled | Q::Timeout | Q::ResourceLimit(_) => OperationalError::new_err(message),
    };
    match &e {
        E::Query(q) => query_err(q, message),
        E::Graph(g) => graph_err(g, message),
        E::TransactionClosed => ProgrammingError::new_err(message),
        E::SessionTransactionTimedOut { .. } => OperationalError::new_err(message),
    }
}

fn graph_err(g: &::marsdb::GraphError, message: String) -> PyErr {
    use ::marsdb::GraphError as G;
    match g {
        G::UniqueConstraintViolation { .. } | G::NodeHasEdges(_) => {
            IntegrityError::new_err(message)
        }
        _ => OperationalError::new_err(message),
    }
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
        ::marsdb::Value::Map(items) => {
            let dict = PyDict::new(py);
            for (key, item) in items {
                dict.set_item(key, value_to_py(py, item)?)?;
            }
            dict.into_any()
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

fn property_to_py<'py>(
    py: Python<'py>,
    p: &marsdb_graph::PropertyValue,
) -> PyResult<Bound<'py, PyAny>> {
    Ok(match p {
        marsdb_graph::PropertyValue::Null => py.None().into_bound(py),
        marsdb_graph::PropertyValue::Bool(b) => b.into_pyobject(py)?.to_owned().into_any(),
        marsdb_graph::PropertyValue::Int(i) => i.into_pyobject(py)?.into_any(),
        marsdb_graph::PropertyValue::Float(f) => f.into_pyobject(py)?.into_any(),
        marsdb_graph::PropertyValue::String(s) => s.into_pyobject(py)?.into_any(),
        // Keep temporal values lossless and consistent with the C/Go
        // bindings. Python's timedelta cannot represent calendar months.
        marsdb_graph::PropertyValue::Date(days) => ::marsdb::temporal::format_date(*days)
            .into_pyobject(py)?
            .into_any(),
        marsdb_graph::PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => ::marsdb::temporal::format_duration(*months, *days, *seconds, *nanos)
            .into_pyobject(py)?
            .into_any(),
        marsdb_graph::PropertyValue::LocalTime(nanos_of_day) => {
            ::marsdb::temporal::format_local_time(*nanos_of_day)
                .into_pyobject(py)?
                .into_any()
        }
        marsdb_graph::PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => ::marsdb::temporal::format_time(*nanos_of_day, *offset_seconds)
            .into_pyobject(py)?
            .into_any(),
        marsdb_graph::PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => ::marsdb::temporal::format_local_date_time(*epoch_seconds, *nanos)
            .into_pyobject(py)?
            .into_any(),
        marsdb_graph::PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => ::marsdb::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone))
            .into_pyobject(py)?
            .into_any(),
        // A list-valued node/edge property (real Cypher/Neo4j's own
        // "homogeneous array property" shape) -- converts to a Python
        // list, recursing per-element the same as `value_to_py`'s own
        // list handling above.
        marsdb_graph::PropertyValue::List(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(property_to_py(py, item)?)?;
            }
            list.into_any()
        }
        // Only ever reached for a `$parameter` echoed back into a result
        // (e.g. `RETURN $mapParam`) -- never a real stored node/edge
        // property (`PropertyValue::Map`'s own doc comment). Converts to
        // a Python dict, recursing per-value the same as `List` above.
        marsdb_graph::PropertyValue::Map(entries) => {
            let dict = PyDict::new(py);
            for (key, value) in entries {
                dict.set_item(key, property_to_py(py, value)?)?;
            }
            dict.into_any()
        }
    })
}

/// `marsdb_graph::TzId` <-> `marsdb::temporal::TzId` -- two independent,
/// same-shaped types (`temporal.rs` deliberately doesn't depend on
/// `marsdb_graph`), converted at this Python-binding formatting boundary.
fn to_temporal_tz(zone: &marsdb_graph::TzId) -> ::marsdb::temporal::TzId {
    match zone {
        marsdb_graph::TzId::Offset(o) => ::marsdb::temporal::TzId::Offset(*o),
        marsdb_graph::TzId::Named(name) => ::marsdb::temporal::TzId::Named(name.clone()),
    }
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
fn marsdb(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Database>()?;
    m.add("Error", py.get_type::<Error>())?;
    m.add("ProgrammingError", py.get_type::<ProgrammingError>())?;
    m.add("DataError", py.get_type::<DataError>())?;
    m.add("OperationalError", py.get_type::<OperationalError>())?;
    m.add("IntegrityError", py.get_type::<IntegrityError>())?;
    Ok(())
}
