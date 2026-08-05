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
    out.push_str("]}");
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
