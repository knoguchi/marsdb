//! C ABI for MarsDB. This is the thin layer non-Rust bindings (starting
//! with `marsdb-go`) link against: an opaque `Database` handle, a handful
//! of `extern "C"` functions, and JSON as the wire format for query
//! results (no target-language-specific marshaling lives here).
//!
//! `Value`/`PropertyValue` don't derive `serde::Serialize` (and can't
//! cheaply be made to, since `Value` lives in `marsdb-query` and mixes in
//! query-layer-only variants like `List`/`Path`), so JSON is built by hand
//! below rather than pulling in `serde_json` for a handful of call sites.

use std::ffi::{c_char, CStr, CString};

use marsdb::{Database, Literal, PathElem, PropertyValue, QueryResult, Value};

/// Opaque handle Go holds a `*mut` to. Never dereferenced on the Go side —
/// only ever passed back into `marsdb_execute`/`marsdb_close`.
pub struct MarsdbDatabase {
    inner: Database,
}

/// Returned by `marsdb_execute`: exactly one of `json`/`error` is
/// non-null. Both fields, when non-null, are Rust-allocated
/// (`CString::into_raw`) and must be released via `marsdb_free_string`,
/// never by a Go/C `free()`.
#[repr(C)]
pub struct MarsdbResult {
    pub json: *mut c_char,
    pub error: *mut c_char,
}

impl MarsdbResult {
    fn ok(json: String) -> Self {
        Self {
            json: CString::new(json).unwrap_or_default().into_raw(),
            error: std::ptr::null_mut(),
        }
    }

    fn err(msg: impl std::fmt::Display) -> Self {
        Self {
            json: std::ptr::null_mut(),
            // Error messages come from `thiserror` Display impls / Cypher
            // parse errors, not attacker-controlled binary data, but strip
            // interior NULs defensively rather than unwrap-panicking across
            // the FFI boundary on the off chance one sneaks in (e.g. via a
            // property value round-tripped into an error message).
            error: CString::new(msg.to_string().replace('\0', ""))
                .unwrap_or_default()
                .into_raw(),
        }
    }
}

/// Open (creating if absent) a single-file, on-disk database. Returns
/// NULL if `path` isn't valid UTF-8 or the underlying open fails.
///
/// # Safety
/// `path` must be NULL or point to a valid NUL-terminated byte string for
/// the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_open(path: *const c_char) -> *mut MarsdbDatabase {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };
    match Database::open(path) {
        Ok(inner) => Box::into_raw(Box::new(MarsdbDatabase { inner })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Open a purely in-memory database. Nothing is written to disk.
#[no_mangle]
pub extern "C" fn marsdb_open_in_memory() -> *mut MarsdbDatabase {
    match Database::in_memory() {
        Ok(inner) => Box::into_raw(Box::new(MarsdbDatabase { inner })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Reclaims a handle returned by `marsdb_open`/`marsdb_open_in_memory`.
/// Passing NULL is a no-op; passing a pointer twice, or one not returned
/// by this crate, is undefined behavior (same contract as `Box::from_raw`).
///
/// # Safety
/// A non-NULL `db` must be an owned, live handle returned by MarsDB and must
/// not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_close(db: *mut MarsdbDatabase) {
    if db.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(db) });
}

/// Run one Cypher statement, returning the result as JSON:
/// `{"columns": [...], "rows": [[...], ...]}`.
///
/// # Safety
/// `db` must be NULL or a live MarsDB handle, and `cypher` must be NULL or
/// point to a valid NUL-terminated byte string for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_execute(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
) -> MarsdbResult {
    if db.is_null() || cypher.is_null() {
        return MarsdbResult::err("null db or cypher pointer");
    }
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return MarsdbResult::err(format!("cypher is not valid UTF-8: {e}")),
    };
    let db = unsafe { &*db };
    // A panic must never cross an `extern "C"` boundary (Rust aborts the
    // process in that case). Convert any unexpected engine panic into the
    // same owned error channel as ordinary query failures.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| db.inner.execute(cypher))) {
        Ok(Ok(result)) => MarsdbResult::ok(result_to_json(&result)),
        Ok(Err(e)) => MarsdbResult::err(e),
        Err(_) => MarsdbResult::err("internal panic while executing query"),
    }
}

/// Run one Cypher statement with `$name` placeholders resolved from
/// `params_json`, a JSON object mapping parameter names to values:
/// `{"name": "Alice", "age": 42, "tags": [1, 2], "addr": {"city": "..."}}`.
/// Same result contract as `marsdb_execute`. NULL `params_json` behaves
/// exactly like `marsdb_execute` (no parameters).
///
/// Value mapping: JSON null/bool/string map directly; a number is an i64
/// when it parses as one (full 64-bit range preserved — emit integers
/// without a decimal point to keep them integral) and an f64 otherwise;
/// arrays/objects become Cypher list/map parameter values. A number
/// outside both i64 and f64 exact ranges (e.g. a u64 above i64::MAX) is
/// an error rather than a silent precision loss.
///
/// # Safety
/// `db` must be NULL or a live MarsDB handle; `cypher` and `params_json`
/// must each be NULL or point to a valid NUL-terminated byte string for
/// the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_execute_with_params(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    params_json: *const c_char,
) -> MarsdbResult {
    if db.is_null() || cypher.is_null() {
        return MarsdbResult::err("null db or cypher pointer");
    }
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return MarsdbResult::err(format!("cypher is not valid UTF-8: {e}")),
    };
    let params = if params_json.is_null() {
        std::collections::HashMap::new()
    } else {
        let raw = match unsafe { CStr::from_ptr(params_json) }.to_str() {
            Ok(p) => p,
            Err(e) => return MarsdbResult::err(format!("params is not valid UTF-8: {e}")),
        };
        match parse_params(raw) {
            Ok(p) => p,
            Err(e) => return MarsdbResult::err(e),
        }
    };
    let db = unsafe { &*db };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        db.inner.execute_with_params(cypher, &params)
    })) {
        Ok(Ok(result)) => MarsdbResult::ok(result_to_json(&result)),
        Ok(Err(e)) => MarsdbResult::err(e),
        Err(_) => MarsdbResult::err("internal panic while executing query"),
    }
}

/// `marsdb_execute_with_params` plus execution bounds: `max_rows`
/// caps the result row count and `timeout_ms` caps wall time, both
/// checked *during* evaluation (a runaway query fails at the bound
/// instead of materializing an unbounded result first). 0 means
/// unlimited for either. Exceeding a bound returns an error whose
/// message begins with "query error: query resource limit exceeded"
/// (max_rows) or "query error: query timed out" (timeout_ms).
///
/// # Safety
/// Same contract as `marsdb_execute_with_params`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_execute_ex(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    params_json: *const c_char,
    max_rows: u64,
    timeout_ms: u64,
) -> MarsdbResult {
    if db.is_null() || cypher.is_null() {
        return MarsdbResult::err("null db or cypher pointer");
    }
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return MarsdbResult::err(format!("cypher is not valid UTF-8: {e}")),
    };
    let params = if params_json.is_null() {
        std::collections::HashMap::new()
    } else {
        let raw = match unsafe { CStr::from_ptr(params_json) }.to_str() {
            Ok(p) => p,
            Err(e) => return MarsdbResult::err(format!("params is not valid UTF-8: {e}")),
        };
        match parse_params(raw) {
            Ok(p) => p,
            Err(e) => return MarsdbResult::err(e),
        }
    };
    let mut options = marsdb::ExecutionOptions::default();
    if max_rows > 0 {
        options.max_result_rows = Some(usize::try_from(max_rows).unwrap_or(usize::MAX));
    }
    if timeout_ms > 0 {
        options.timeout = Some(std::time::Duration::from_millis(timeout_ms));
    }
    let db = unsafe { &*db };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        db.inner
            .execute_with_params_and_options(cypher, &params, &options)
    })) {
        Ok(Ok(result)) => MarsdbResult::ok(result_to_json(&result)),
        Ok(Err(e)) => MarsdbResult::err(e),
        Err(_) => MarsdbResult::err("internal panic while executing query"),
    }
}

fn parse_params(raw: &str) -> Result<std::collections::HashMap<String, PropertyValue>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("params is not valid JSON: {e}"))?;
    let serde_json::Value::Object(object) = parsed else {
        return Err("params must be a JSON object mapping parameter names to values".into());
    };
    object
        .into_iter()
        .map(|(name, value)| {
            let converted =
                json_to_property(value).map_err(|e| format!("parameter '{name}': {e}"))?;
            Ok((name, converted))
        })
        .collect()
}

fn json_to_property(value: serde_json::Value) -> Result<PropertyValue, String> {
    Ok(match value {
        serde_json::Value::Null => PropertyValue::Null,
        serde_json::Value::Bool(b) => PropertyValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                PropertyValue::Int(i)
            } else if n.is_u64() {
                // An integral value above i64::MAX. `as_f64` would happily
                // hand back a lossy approximation -- checked before the
                // float branch precisely so that can't happen silently.
                return Err(format!("integer {n} exceeds the i64 range"));
            } else if let Some(f) = n.as_f64() {
                PropertyValue::Float(f)
            } else {
                return Err(format!("number {n} is not representable"));
            }
        }
        serde_json::Value::String(s) => PropertyValue::String(s),
        serde_json::Value::Array(items) => PropertyValue::List(
            items
                .into_iter()
                .map(json_to_property)
                .collect::<Result<_, _>>()?,
        ),
        serde_json::Value::Object(entries) => PropertyValue::Map(
            entries
                .into_iter()
                .map(|(k, v)| Ok((k, json_to_property(v)?)))
                .collect::<Result<_, String>>()?,
        ),
    })
}

/// Frees a string previously returned in `MarsdbResult.json` or
/// `MarsdbResult.error`. The caller MUST NOT free these with anything
/// other than this function — they were allocated by Rust's global
/// allocator via `CString::into_raw`, not `malloc`.
///
/// # Safety
/// A non-NULL `s` must be a live pointer returned in a MarsDB result and may
/// be passed to this function exactly once.
#[no_mangle]
pub unsafe extern "C" fn marsdb_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    drop(unsafe { CString::from_raw(s) });
}

fn result_to_json(result: &QueryResult) -> String {
    let mut out = String::from("{\"columns\":[");
    for (i, col) in result.columns.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_json_string(&mut out, col);
    }
    out.push_str("],\"rows\":[");
    for (i, row) in result.rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('[');
        for (j, value) in row.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            push_value_json(&mut out, value);
        }
        out.push(']');
    }
    out.push(']');
    // Write-statement counters -- omitted entirely for pure reads so
    // read-heavy consumers don't pay for keys that are always zero.
    let stats = &result.stats;
    if !stats.is_empty() {
        out.push_str(&format!(
            ",\"stats\":{{\"nodes_created\":{},\"nodes_deleted\":{},\"relationships_created\":{},\"relationships_deleted\":{},\"properties_set\":{},\"labels_added\":{},\"labels_removed\":{}}}",
            stats.nodes_created,
            stats.nodes_deleted,
            stats.relationships_created,
            stats.relationships_deleted,
            stats.properties_set,
            stats.labels_added,
            stats.labels_removed,
        ));
    }
    out.push('}');
    out
}

/// A node as `{"__type":"node","id":...,"labels":[...],"props":{...}}`,
/// an edge similarly with `"__type":"edge"` plus `src`/`dst`, everything
/// else as its natural JSON scalar/array shape — chosen so a Go (or any
/// other JSON-consuming) caller can tell a node/edge apart from a plain
/// map without a second out-of-band type signal.
fn push_value_json(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Property(p) => push_property_json(out, p),
        Value::Literal(l) => push_literal_json(out, l),
        Value::Node(n) => {
            out.push_str("{\"__type\":\"node\",\"id\":");
            out.push_str(&n.id.0.to_string());
            out.push_str(",\"labels\":[");
            for (i, label) in n.labels.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_json_string(out, label);
            }
            out.push_str("],\"props\":{");
            for (i, (k, v)) in n.props.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_json_string(out, k);
                out.push(':');
                push_property_json(out, v);
            }
            out.push_str("}}");
        }
        Value::Edge(e) => {
            out.push_str("{\"__type\":\"edge\",\"id\":");
            out.push_str(&e.id.0.to_string());
            out.push_str(",\"label\":");
            push_json_string(out, &e.label);
            out.push_str(",\"src\":");
            out.push_str(&e.src.0.to_string());
            out.push_str(",\"dst\":");
            out.push_str(&e.dst.0.to_string());
            out.push_str(",\"props\":{");
            for (i, (k, v)) in e.props.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_json_string(out, k);
                out.push(':');
                push_property_json(out, v);
            }
            out.push_str("}}");
        }
        Value::List(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_value_json(out, item);
            }
            out.push(']');
        }
        Value::Map(m) => {
            out.push('{');
            for (i, (k, v)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_json_string(out, k);
                out.push(':');
                push_value_json(out, v);
            }
            out.push('}');
        }
        // Flattened to the same node/edge dicts used elsewhere, alternating
        // node, edge, node, ... — mirrors marsdb-python's path handling
        // (see marsdb-python/src/lib.rs) so both bindings agree on shape.
        Value::Path(elems) => {
            out.push('[');
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                match elem {
                    PathElem::Node(n) => push_value_json(out, &Value::Node(n.clone())),
                    PathElem::Edge(e) => push_value_json(out, &Value::Edge(e.clone())),
                }
            }
            out.push(']');
        }
    }
}

/// `marsdb_graph::TzId` <-> `marsdb::temporal::TzId` -- two independent,
/// same-shaped types (`temporal.rs` deliberately doesn't depend on
/// `marsdb_graph`), converted at this formatting boundary.
fn to_temporal_tz(zone: &marsdb::TzId) -> marsdb::temporal::TzId {
    match zone {
        marsdb::TzId::Offset(o) => marsdb::temporal::TzId::Offset(*o),
        marsdb::TzId::Named(name) => marsdb::temporal::TzId::Named(name.clone()),
    }
}

fn push_property_json(out: &mut String, p: &PropertyValue) {
    match p {
        PropertyValue::Null => out.push_str("null"),
        PropertyValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        PropertyValue::Int(i) => out.push_str(&i.to_string()),
        PropertyValue::Float(f) => push_json_float(out, *f),
        PropertyValue::String(s) => push_json_string(out, s),
        // JSON has no temporal scalar types. Use the canonical Cypher/
        // ISO-8601 text forms, which round-trip through date()/duration()
        // and are also what the CLI displays.
        PropertyValue::Date(d) => push_json_string(out, &marsdb::temporal::format_date(*d)),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => push_json_string(
            out,
            &marsdb::temporal::format_duration(*months, *days, *seconds, *nanos),
        ),
        PropertyValue::LocalTime(nanos_of_day) => {
            push_json_string(out, &marsdb::temporal::format_local_time(*nanos_of_day))
        }
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => push_json_string(
            out,
            &marsdb::temporal::format_time(*nanos_of_day, *offset_seconds),
        ),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => push_json_string(
            out,
            &marsdb::temporal::format_local_date_time(*epoch_seconds, *nanos),
        ),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => push_json_string(
            out,
            &marsdb::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone)),
        ),
        PropertyValue::List(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_property_json(out, item);
            }
            out.push(']');
        }
        // Only ever reached for a `$parameter` echoed back into a result
        // (e.g. `RETURN $mapParam`) -- never a real stored node/edge
        // property (`PropertyValue::Map`'s own doc comment).
        PropertyValue::Map(entries) => {
            out.push('{');
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_json_string(out, key);
                out.push(':');
                push_property_json(out, value);
            }
            out.push('}');
        }
    }
}

fn push_literal_json(out: &mut String, l: &Literal) {
    match l {
        Literal::Null => out.push_str("null"),
        Literal::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Literal::Int(i) => out.push_str(&i.to_string()),
        Literal::Float(f) => push_json_float(out, *f),
        Literal::String(s) => push_json_string(out, s),
        Literal::Param(name) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
    }
}

/// JSON has no NaN/Infinity token; emit `null` for those rather than
/// producing invalid JSON (matches serde_json's `f64` behavior when asked
/// to tolerate non-finite floats, which we can't pull in serde_json just
/// for).
fn push_json_float(out: &mut String, f: f64) {
    if f.is_finite() {
        let rendered = f.to_string();
        out.push_str(&rendered);
        // Rust renders an integral f64 such as 1.0 as "1". Keep an
        // explicit decimal marker so consumers using JSON's lexical form
        // can preserve MarsDB's Int-vs-Float distinction.
        if !rendered.contains(['.', 'e', 'E']) {
            out.push_str(".0");
        }
    } else {
        out.push_str("null");
    }
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_json_covers_scalars_temporals_and_escaping() {
        let db = Database::in_memory().unwrap();
        let result = db
            .execute(
                "RETURN 9223372036854775807 AS max, 1.5 AS float, 1.0 AS whole_float, \
                 'line\\n\\\"quote\\\"' AS text, date('1984-10-11') AS date, \
                 duration('P1M2DT3H4M5.006S') AS duration",
            )
            .unwrap();

        assert_eq!(
            result_to_json(&result),
            r#"{"columns":["max","float","whole_float","text","date","duration"],"rows":[[9223372036854775807,1.5,1.0,"line\n\"quote\"","1984-10-11","P1M2DT3H4M5.006S"]]}"#
        );
    }

    #[test]
    fn execute_reports_invalid_utf8_and_null_inputs() {
        let null_result = unsafe { marsdb_execute(std::ptr::null_mut(), std::ptr::null()) };
        assert!(null_result.json.is_null());
        assert!(!null_result.error.is_null());
        unsafe { marsdb_free_string(null_result.error) };

        let db = marsdb_open_in_memory();
        assert!(!db.is_null());
        let invalid_utf8 = [0xff_u8, 0];
        let result = unsafe { marsdb_execute(db, invalid_utf8.as_ptr().cast()) };
        assert!(result.json.is_null());
        assert!(!result.error.is_null());
        unsafe {
            marsdb_free_string(result.error);
            marsdb_close(db);
        }
    }

    #[test]
    fn execute_with_params_substitutes_and_preserves_types() {
        let db = marsdb_open_in_memory();
        let cypher = CString::new(
            "RETURN $big AS big, $f AS f, $s AS s, $flag AS flag, $nothing AS nothing, \
             $tags AS tags, $addr AS addr",
        )
        .unwrap();
        let params = CString::new(
            r#"{"big": 9223372036854775807, "f": 1.5, "s": "it's \"quoted\"", "flag": true,
                "nothing": null, "tags": [1, 2], "addr": {"city": "Kyoto"}}"#,
        )
        .unwrap();
        let result = unsafe { marsdb_execute_with_params(db, cypher.as_ptr(), params.as_ptr()) };
        assert!(result.error.is_null());
        let json = unsafe { CStr::from_ptr(result.json) }.to_str().unwrap();
        assert!(json.contains("9223372036854775807"), "{json}");
        assert!(json.contains("1.5"), "{json}");
        assert!(json.contains(r#""it's \"quoted\"""#), "{json}");
        assert!(json.contains("true"), "{json}");
        assert!(json.contains("null"), "{json}");
        assert!(json.contains("[1,2]"), "{json}");
        assert!(json.contains(r#"{"city":"Kyoto"}"#), "{json}");
        unsafe {
            marsdb_free_string(result.json);
            marsdb_close(db);
        }
    }

    #[test]
    fn execute_with_params_rejects_bad_params() {
        let db = marsdb_open_in_memory();
        let cypher = CString::new("RETURN $x AS x").unwrap();

        // Not an object.
        let params = CString::new("[1, 2]").unwrap();
        let result = unsafe { marsdb_execute_with_params(db, cypher.as_ptr(), params.as_ptr()) };
        let message = unsafe { CStr::from_ptr(result.error) }.to_str().unwrap();
        assert!(message.contains("must be a JSON object"), "{message}");
        unsafe { marsdb_free_string(result.error) };

        // A u64 above i64::MAX must error, not silently lose precision.
        let params = CString::new(r#"{"x": 18446744073709551615}"#).unwrap();
        let result = unsafe { marsdb_execute_with_params(db, cypher.as_ptr(), params.as_ptr()) };
        let message = unsafe { CStr::from_ptr(result.error) }.to_str().unwrap();
        assert!(message.contains("parameter 'x'"), "{message}");
        unsafe { marsdb_free_string(result.error) };

        // Missing parameter surfaces the engine's own error.
        let result = unsafe { marsdb_execute_with_params(db, cypher.as_ptr(), std::ptr::null()) };
        assert!(result.json.is_null());
        assert!(!result.error.is_null());
        unsafe {
            marsdb_free_string(result.error);
            marsdb_close(db);
        }
    }

    #[test]
    fn execute_ex_bounds_rows() {
        let db = marsdb_open_in_memory();
        let seed = CString::new("CREATE (:N), (:N), (:N)").unwrap();
        let result = unsafe { marsdb_execute(db, seed.as_ptr()) };
        assert!(result.error.is_null());
        unsafe { marsdb_free_string(result.json) };

        let cypher = CString::new("MATCH (n:N) RETURN n").unwrap();
        // At the bound: fine (params NULL, timeout unlimited).
        let result = unsafe { marsdb_execute_ex(db, cypher.as_ptr(), std::ptr::null(), 3, 0) };
        assert!(result.error.is_null());
        unsafe { marsdb_free_string(result.json) };
        // Over the bound: resource-limit error, checked during evaluation.
        let result = unsafe { marsdb_execute_ex(db, cypher.as_ptr(), std::ptr::null(), 2, 0) };
        assert!(result.json.is_null());
        let message = unsafe { CStr::from_ptr(result.error) }.to_str().unwrap();
        assert!(message.contains("resource limit"), "{message}");
        unsafe {
            marsdb_free_string(result.error);
            marsdb_close(db);
        }
    }

    #[test]
    fn execute_returns_engine_arithmetic_errors() {
        let db = marsdb_open_in_memory();
        let cypher = CString::new("RETURN -9223372036854775808 / -1").unwrap();
        let result = unsafe { marsdb_execute(db, cypher.as_ptr()) };
        assert!(result.json.is_null());
        assert!(!result.error.is_null());
        let message = unsafe { CStr::from_ptr(result.error) }.to_str().unwrap();
        assert!(message.contains("integer arithmetic overflow"));
        unsafe {
            marsdb_free_string(result.error);
            marsdb_close(db);
        }
    }
}
