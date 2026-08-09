//! C ABI for MarsDB, v2: typed opaque handles (the SQLite shape) plus a
//! binary batch lane. See marsdb.h for the full contract — the header is
//! the documentation of record; this file keeps its invariants.
//!
//! Lifetime/aliasing invariants the unsafe code relies on:
//! - A `MarsdbResult`'s `rows` are never mutated after construction, so
//!   raw pointers into them (handed out as `MarsdbValue` handles) stay
//!   valid until the result is destroyed. Handles and scratch CStrings
//!   are still *documented* as invalidated by `marsdb_next` (sqlite
//!   convention, and `next` clears the per-row arenas), so well-behaved
//!   callers never observe the difference.
//! - A panic must never cross the FFI boundary: every entry point that
//!   runs engine code wraps it in `catch_unwind`.

use std::collections::{BTreeMap, HashMap};
use std::ffi::{c_char, CStr, CString};
use std::sync::Mutex;

use marsdb::{
    Database, ExecutionOptions, Literal, PathElem, PropertyValue, QueryResult, QueryStats,
    Statement, Value,
};
use marsdb_graph::{Edge, Node};

pub const MARSDB_OK: i32 = 0;
pub const MARSDB_ERROR: i32 = 1;

// ---- database handle ----------------------------------------------------

pub struct MarsdbDatabase {
    inner: Database,
    /// `marsdb_last_error`'s backing store. Mutex, not RefCell: the
    /// header makes no single-caller promise for the *database* handle
    /// (results/statements are single-caller by contract; this isn't).
    last_error: Mutex<Option<CString>>,
}

impl MarsdbDatabase {
    fn set_error(&self, message: impl std::fmt::Display) -> i32 {
        let cleaned = message.to_string().replace('\0', "");
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(CString::new(cleaned).unwrap_or_default());
        MARSDB_ERROR
    }
}

/// # Safety
/// `path` must be NULL or a valid NUL-terminated string for the call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_open(path: *const c_char) -> *mut MarsdbDatabase {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(path) = (unsafe { CStr::from_ptr(path) }).to_str() else {
        return std::ptr::null_mut();
    };
    match Database::open(path) {
        Ok(inner) => Box::into_raw(Box::new(MarsdbDatabase {
            inner,
            last_error: Mutex::new(None),
        })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn marsdb_open_in_memory() -> *mut MarsdbDatabase {
    match Database::in_memory() {
        Ok(inner) => Box::into_raw(Box::new(MarsdbDatabase {
            inner,
            last_error: Mutex::new(None),
        })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// A non-NULL `db` must be a live handle from `marsdb_open*`, passed at
/// most once, with no live statements/results still referencing it.
#[no_mangle]
pub unsafe extern "C" fn marsdb_close(db: *mut MarsdbDatabase) {
    if !db.is_null() {
        drop(unsafe { Box::from_raw(db) });
    }
}

/// # Safety
/// `db` must be NULL or a live handle. Returned pointer is borrowed
/// (header lifetime rules).
#[no_mangle]
pub unsafe extern "C" fn marsdb_last_error(db: *const MarsdbDatabase) -> *const c_char {
    static EMPTY: &[u8] = b"\0";
    if db.is_null() {
        return EMPTY.as_ptr().cast();
    }
    let guard = unsafe { &*db }
        .last_error
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match &*guard {
        // The CString's allocation lives until the next set_error on
        // this handle -- exactly the documented borrow window.
        Some(message) => message.as_ptr(),
        None => EMPTY.as_ptr().cast(),
    }
}

// ---- prepared statements ------------------------------------------------

pub struct MarsdbStatement {
    /// The database this was prepared against; the header requires the
    /// caller to keep it alive past the statement.
    db: *const MarsdbDatabase,
    stmt: Statement,
    binds: HashMap<String, PropertyValue>,
    max_rows: u64,
    timeout_ms: u64,
}

impl MarsdbStatement {
    fn options(&self) -> ExecutionOptions {
        let mut options = ExecutionOptions::default();
        if self.max_rows > 0 {
            options.max_result_rows = Some(usize::try_from(self.max_rows).unwrap_or(usize::MAX));
        }
        if self.timeout_ms > 0 {
            options.timeout = Some(std::time::Duration::from_millis(self.timeout_ms));
        }
        options
    }
}

/// # Safety
/// `db` live handle; `cypher` valid NUL-terminated string; `out` valid
/// pointer to receive the statement.
#[no_mangle]
pub unsafe extern "C" fn marsdb_prepare(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    out: *mut *mut MarsdbStatement,
) -> i32 {
    if db.is_null() || cypher.is_null() || out.is_null() {
        return MARSDB_ERROR;
    }
    let db_ref = unsafe { &*db };
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return db_ref.set_error(format!("cypher is not valid UTF-8: {e}")),
    };
    match marsdb::parse(cypher) {
        Ok(stmt) => {
            unsafe {
                *out = Box::into_raw(Box::new(MarsdbStatement {
                    db,
                    stmt,
                    binds: HashMap::new(),
                    max_rows: 0,
                    timeout_ms: 0,
                }));
            }
            MARSDB_OK
        }
        Err(e) => db_ref.set_error(e),
    }
}

unsafe fn bind(stmt: *mut MarsdbStatement, name: *const c_char, value: PropertyValue) -> i32 {
    if stmt.is_null() || name.is_null() {
        return MARSDB_ERROR;
    }
    let stmt = unsafe { &mut *stmt };
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return unsafe { &*stmt.db }.set_error("parameter name is not valid UTF-8");
    };
    stmt.binds.insert(name.to_string(), value);
    MARSDB_OK
}

/// # Safety
/// `stmt` live statement; `name` valid NUL-terminated string. (Same for
/// every bind below.)
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_int64(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    value: i64,
) -> i32 {
    unsafe { bind(stmt, name, PropertyValue::Int(value)) }
}

/// # Safety
/// See `marsdb_bind_int64`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_double(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    value: f64,
) -> i32 {
    unsafe { bind(stmt, name, PropertyValue::Float(value)) }
}

/// # Safety
/// See `marsdb_bind_int64`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_bool(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    value: i32,
) -> i32 {
    unsafe { bind(stmt, name, PropertyValue::Bool(value != 0)) }
}

/// # Safety
/// See `marsdb_bind_int64`; `value` NULL (binds null) or valid
/// NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_string(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    value: *const c_char,
) -> i32 {
    if value.is_null() {
        return unsafe { bind(stmt, name, PropertyValue::Null) };
    }
    if stmt.is_null() {
        return MARSDB_ERROR;
    }
    let Ok(s) = (unsafe { CStr::from_ptr(value) }).to_str() else {
        return unsafe { &*(*stmt).db }.set_error("string parameter is not valid UTF-8");
    };
    let owned = s.to_string();
    unsafe { bind(stmt, name, PropertyValue::String(owned)) }
}

/// # Safety
/// See `marsdb_bind_int64`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_null(stmt: *mut MarsdbStatement, name: *const c_char) -> i32 {
    unsafe { bind(stmt, name, PropertyValue::Null) }
}

/// # Safety
/// See `marsdb_bind_int64`; `values` must point to `len` readable i64s.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_int64_list(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    values: *const i64,
    len: usize,
) -> i32 {
    if len > 0 && values.is_null() {
        return MARSDB_ERROR;
    }
    let items = if len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(values, len) }
            .iter()
            .map(|v| PropertyValue::Int(*v))
            .collect()
    };
    unsafe { bind(stmt, name, PropertyValue::List(items)) }
}

/// # Safety
/// See `marsdb_bind_int64_list`, with doubles.
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_double_list(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    values: *const f64,
    len: usize,
) -> i32 {
    if len > 0 && values.is_null() {
        return MARSDB_ERROR;
    }
    let items = if len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(values, len) }
            .iter()
            .map(|v| PropertyValue::Float(*v))
            .collect()
    };
    unsafe { bind(stmt, name, PropertyValue::List(items)) }
}

/// # Safety
/// See `marsdb_bind_int64_list`; each non-NULL entry must be a valid
/// NUL-terminated string (NULL entries bind null elements).
#[no_mangle]
pub unsafe extern "C" fn marsdb_bind_string_list(
    stmt: *mut MarsdbStatement,
    name: *const c_char,
    values: *const *const c_char,
    len: usize,
) -> i32 {
    if len > 0 && values.is_null() {
        return MARSDB_ERROR;
    }
    if stmt.is_null() {
        return MARSDB_ERROR;
    }
    let mut items = Vec::with_capacity(len);
    if len > 0 {
        for &ptr in unsafe { std::slice::from_raw_parts(values, len) } {
            if ptr.is_null() {
                items.push(PropertyValue::Null);
                continue;
            }
            let Ok(s) = (unsafe { CStr::from_ptr(ptr) }).to_str() else {
                return unsafe { &*(*stmt).db }
                    .set_error("string list parameter is not valid UTF-8");
            };
            items.push(PropertyValue::String(s.to_string()));
        }
    }
    unsafe { bind(stmt, name, PropertyValue::List(items)) }
}

/// # Safety
/// `stmt` NULL or live.
#[no_mangle]
pub unsafe extern "C" fn marsdb_clear_bindings(stmt: *mut MarsdbStatement) {
    if !stmt.is_null() {
        unsafe { &mut *stmt }.binds.clear();
    }
}

/// # Safety
/// `stmt` NULL or live.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_set_max_rows(stmt: *mut MarsdbStatement, max_rows: u64) {
    if !stmt.is_null() {
        unsafe { &mut *stmt }.max_rows = max_rows;
    }
}

/// # Safety
/// `stmt` NULL or live.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_set_timeout_ms(stmt: *mut MarsdbStatement, timeout_ms: u64) {
    if !stmt.is_null() {
        unsafe { &mut *stmt }.timeout_ms = timeout_ms;
    }
}

/// # Safety
/// `stmt` NULL or live (and then never used again).
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_destroy(stmt: *mut MarsdbStatement) {
    if !stmt.is_null() {
        drop(unsafe { Box::from_raw(stmt) });
    }
}

// ---- results ------------------------------------------------------------

/// Which underlying record a value handle points at. All pointers point
/// into the owning result's `rows` (never mutated after construction)
/// or into nested structures within them.
enum RefKind {
    Val(*const Value),
    Prop(*const PropertyValue),
    Node(*const Node),
    Edge(*const Edge),
}

pub struct MarsdbValue {
    kind: RefKind,
    owner: *const MarsdbResult,
}

pub struct MarsdbResult {
    columns: Vec<CString>,
    rows: Vec<Vec<Value>>,
    stats: QueryStats,
    /// -1 before the first `marsdb_next`.
    cursor: isize,
    /// Per-row arenas for handed-out handles and strings; cleared by
    /// `marsdb_next`. RefCell: accessors take `&self` across the ABI
    /// (results are single-caller by header contract). The Box per
    /// handle is load-bearing, not waste: raw pointers to the handles
    /// are handed across the ABI, and a `Vec<MarsdbValue>` would move
    /// them on reallocation.
    #[allow(clippy::vec_box)]
    handles: std::cell::RefCell<Vec<Box<MarsdbValue>>>,
    scratch: std::cell::RefCell<Vec<CString>>,
}

impl MarsdbResult {
    fn from_query_result(result: QueryResult) -> Self {
        Self {
            columns: result
                .columns
                .iter()
                .map(|c| CString::new(c.replace('\0', "")).unwrap_or_default())
                .collect(),
            rows: result.rows,
            stats: result.stats,
            cursor: -1,
            handles: std::cell::RefCell::new(Vec::new()),
            scratch: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn make_handle(&self, kind: RefKind) -> *const MarsdbValue {
        let boxed = Box::new(MarsdbValue { kind, owner: self });
        let ptr: *const MarsdbValue = &*boxed;
        self.handles.borrow_mut().push(boxed);
        ptr
    }

    fn intern_string(&self, s: &str) -> *const c_char {
        let c = CString::new(s.replace('\0', "")).unwrap_or_default();
        let ptr = c.as_ptr();
        self.scratch.borrow_mut().push(c);
        ptr
    }
}

fn run_statement(stmt: &MarsdbStatement) -> Result<QueryResult, marsdb::Error> {
    let db = unsafe { &*stmt.db };
    db.inner
        .execute_prepared_statement(&stmt.stmt, &stmt.binds, &stmt.options())
}

fn execute_to_result(
    db: &MarsdbDatabase,
    run: impl FnOnce() -> Result<QueryResult, marsdb::Error>,
    out: *mut *mut MarsdbResult,
) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
        Ok(Ok(result)) => {
            unsafe {
                *out = Box::into_raw(Box::new(MarsdbResult::from_query_result(result)));
            }
            MARSDB_OK
        }
        Ok(Err(e)) => db.set_error(e),
        Err(_) => db.set_error("internal panic while executing query"),
    }
}

/// # Safety
/// `stmt` live statement (whose database is still live); `out` valid.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_execute(
    stmt: *mut MarsdbStatement,
    out: *mut *mut MarsdbResult,
) -> i32 {
    if stmt.is_null() || out.is_null() {
        return MARSDB_ERROR;
    }
    let stmt = unsafe { &*stmt };
    let db = unsafe { &*stmt.db };
    execute_to_result(db, || run_statement(stmt), out)
}

/// # Safety
/// `db` live handle; `cypher` valid NUL-terminated string; `out` valid.
#[no_mangle]
pub unsafe extern "C" fn marsdb_query(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    out: *mut *mut MarsdbResult,
) -> i32 {
    if db.is_null() || cypher.is_null() || out.is_null() {
        return MARSDB_ERROR;
    }
    let db = unsafe { &*db };
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return db.set_error(format!("cypher is not valid UTF-8: {e}")),
    };
    execute_to_result(db, || db.inner.execute(cypher), out)
}

/// # Safety
/// `result` NULL or live.
#[no_mangle]
pub unsafe extern "C" fn marsdb_column_count(result: *const MarsdbResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { &*result }.columns.len()
}

/// # Safety
/// `result` NULL or live; returned pointer borrowed until destroy.
#[no_mangle]
pub unsafe extern "C" fn marsdb_column_name(
    result: *const MarsdbResult,
    i: usize,
) -> *const c_char {
    if result.is_null() {
        return std::ptr::null();
    }
    match unsafe { &*result }.columns.get(i) {
        Some(name) => name.as_ptr(),
        None => std::ptr::null(),
    }
}

/// # Safety
/// `result` NULL or live, single-caller (not thread-safe).
#[no_mangle]
pub unsafe extern "C" fn marsdb_next(result: *mut MarsdbResult) -> i32 {
    if result.is_null() {
        return 0;
    }
    let result = unsafe { &mut *result };
    result.handles.borrow_mut().clear();
    result.scratch.borrow_mut().clear();
    let next = result.cursor + 1;
    if (next as usize) < result.rows.len() {
        result.cursor = next;
        1
    } else {
        result.cursor = result.rows.len() as isize;
        0
    }
}

#[repr(C)]
#[derive(Default)]
pub struct MarsdbQueryStats {
    pub nodes_created: u64,
    pub nodes_deleted: u64,
    pub relationships_created: u64,
    pub relationships_deleted: u64,
    pub properties_set: u64,
    pub labels_added: u64,
    pub labels_removed: u64,
}

/// # Safety
/// `result` NULL or live.
#[no_mangle]
pub unsafe extern "C" fn marsdb_result_stats(result: *const MarsdbResult) -> MarsdbQueryStats {
    if result.is_null() {
        return MarsdbQueryStats::default();
    }
    let s = &unsafe { &*result }.stats;
    MarsdbQueryStats {
        nodes_created: s.nodes_created,
        nodes_deleted: s.nodes_deleted,
        relationships_created: s.relationships_created,
        relationships_deleted: s.relationships_deleted,
        properties_set: s.properties_set,
        labels_added: s.labels_added,
        labels_removed: s.labels_removed,
    }
}

/// # Safety
/// `result` NULL or live (and then never used again).
#[no_mangle]
pub unsafe extern "C" fn marsdb_result_destroy(result: *mut MarsdbResult) {
    if !result.is_null() {
        drop(unsafe { Box::from_raw(result) });
    }
}

/// # Safety
/// `result` NULL or live; returned handle borrowed until next/destroy.
#[no_mangle]
pub unsafe extern "C" fn marsdb_row_value(
    result: *const MarsdbResult,
    col: usize,
) -> *const MarsdbValue {
    if result.is_null() {
        return std::ptr::null();
    }
    let r = unsafe { &*result };
    if r.cursor < 0 {
        return std::ptr::null();
    }
    let Some(row) = r.rows.get(r.cursor as usize) else {
        return std::ptr::null();
    };
    let Some(value) = row.get(col) else {
        return std::ptr::null();
    };
    r.make_handle(RefKind::Val(value))
}

// ---- value accessors ----------------------------------------------------

pub const MARSDB_TYPE_NULL: i32 = 0;
pub const MARSDB_TYPE_BOOL: i32 = 1;
pub const MARSDB_TYPE_INT64: i32 = 2;
pub const MARSDB_TYPE_FLOAT64: i32 = 3;
pub const MARSDB_TYPE_STRING: i32 = 4;
pub const MARSDB_TYPE_DATE: i32 = 5;
pub const MARSDB_TYPE_DURATION: i32 = 6;
pub const MARSDB_TYPE_NODE: i32 = 7;
pub const MARSDB_TYPE_EDGE: i32 = 8;
pub const MARSDB_TYPE_LIST: i32 = 9;
pub const MARSDB_TYPE_MAP: i32 = 10;
pub const MARSDB_TYPE_PATH: i32 = 11;

fn prop_type(p: &PropertyValue) -> i32 {
    match p {
        PropertyValue::Null => MARSDB_TYPE_NULL,
        PropertyValue::Bool(_) => MARSDB_TYPE_BOOL,
        PropertyValue::Int(_) => MARSDB_TYPE_INT64,
        PropertyValue::Float(_) => MARSDB_TYPE_FLOAT64,
        // Non-Date/Duration temporals surface as their canonical ISO
        // text, exactly as the JSON channel did.
        PropertyValue::String(_)
        | PropertyValue::LocalTime(_)
        | PropertyValue::Time { .. }
        | PropertyValue::LocalDateTime { .. }
        | PropertyValue::DateTime { .. } => MARSDB_TYPE_STRING,
        PropertyValue::Date(_) => MARSDB_TYPE_DATE,
        PropertyValue::Duration { .. } => MARSDB_TYPE_DURATION,
        PropertyValue::List(_) => MARSDB_TYPE_LIST,
        PropertyValue::Map(_) => MARSDB_TYPE_MAP,
    }
}

fn deref_kind(kind: &RefKind) -> i32 {
    match kind {
        RefKind::Node(_) => MARSDB_TYPE_NODE,
        RefKind::Edge(_) => MARSDB_TYPE_EDGE,
        RefKind::Prop(p) => prop_type(unsafe { &**p }),
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Null => MARSDB_TYPE_NULL,
            Value::Node(_) => MARSDB_TYPE_NODE,
            Value::Edge(_) => MARSDB_TYPE_EDGE,
            Value::List(_) => MARSDB_TYPE_LIST,
            Value::Map(_) => MARSDB_TYPE_MAP,
            Value::Path(_) => MARSDB_TYPE_PATH,
            Value::Property(p) => prop_type(p),
            Value::Literal(l) => match l {
                Literal::Null | Literal::Param(_) => MARSDB_TYPE_NULL,
                Literal::Bool(_) => MARSDB_TYPE_BOOL,
                Literal::Int(_) => MARSDB_TYPE_INT64,
                Literal::Float(_) => MARSDB_TYPE_FLOAT64,
                Literal::String(_) => MARSDB_TYPE_STRING,
            },
        },
    }
}

/// # Safety
/// `value` NULL or a live handle from this ABI. (Same for every value
/// accessor below.)
#[no_mangle]
pub unsafe extern "C" fn marsdb_value_type(value: *const MarsdbValue) -> i32 {
    if value.is_null() {
        return MARSDB_TYPE_NULL;
    }
    deref_kind(&unsafe { &*value }.kind)
}

fn as_prop(kind: &RefKind) -> Option<&PropertyValue> {
    match kind {
        RefKind::Prop(p) => Some(unsafe { &**p }),
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Property(p) => Some(p),
            _ => None,
        },
        _ => None,
    }
}

fn as_literal(kind: &RefKind) -> Option<&Literal> {
    match kind {
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Literal(l) => Some(l),
            _ => None,
        },
        _ => None,
    }
}

/// # Safety
/// See `marsdb_value_type`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_value_int64(value: *const MarsdbValue) -> i64 {
    if value.is_null() {
        return 0;
    }
    let kind = &unsafe { &*value }.kind;
    match (as_prop(kind), as_literal(kind)) {
        (Some(PropertyValue::Int(i)), _) => *i,
        (_, Some(Literal::Int(i))) => *i,
        _ => 0,
    }
}

/// # Safety
/// See `marsdb_value_type`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_value_double(value: *const MarsdbValue) -> f64 {
    if value.is_null() {
        return 0.0;
    }
    let kind = &unsafe { &*value }.kind;
    match (as_prop(kind), as_literal(kind)) {
        (Some(PropertyValue::Float(f)), _) => *f,
        (Some(PropertyValue::Int(i)), _) => *i as f64,
        (_, Some(Literal::Float(f))) => *f,
        (_, Some(Literal::Int(i))) => *i as f64,
        _ => 0.0,
    }
}

/// # Safety
/// See `marsdb_value_type`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_value_bool(value: *const MarsdbValue) -> i32 {
    if value.is_null() {
        return 0;
    }
    let kind = &unsafe { &*value }.kind;
    match (as_prop(kind), as_literal(kind)) {
        (Some(PropertyValue::Bool(b)), _) => i32::from(*b),
        (_, Some(Literal::Bool(b))) => i32::from(*b),
        _ => 0,
    }
}

/// The canonical text for string-like values — STRING content plus
/// every temporal type's ISO-8601 form (the same strings the JSON
/// channel emitted).
fn string_form(p: &PropertyValue) -> Option<String> {
    Some(match p {
        PropertyValue::String(s) => s.clone(),
        PropertyValue::Date(d) => marsdb::temporal::format_date(*d),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => marsdb::temporal::format_duration(*months, *days, *seconds, *nanos),
        PropertyValue::LocalTime(nanos_of_day) => {
            marsdb::temporal::format_local_time(*nanos_of_day)
        }
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => marsdb::temporal::format_time(*nanos_of_day, *offset_seconds),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => marsdb::temporal::format_local_date_time(*epoch_seconds, *nanos),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => marsdb::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone)),
        _ => return None,
    })
}

fn to_temporal_tz(zone: &marsdb_graph::TzId) -> marsdb::temporal::TzId {
    match zone {
        marsdb_graph::TzId::Offset(o) => marsdb::temporal::TzId::Offset(*o),
        marsdb_graph::TzId::Named(name) => marsdb::temporal::TzId::Named(name.clone()),
    }
}

/// # Safety
/// See `marsdb_value_type`; returned pointer borrowed until the owning
/// result's next/destroy.
#[no_mangle]
pub unsafe extern "C" fn marsdb_value_string(value: *const MarsdbValue) -> *const c_char {
    if value.is_null() {
        return std::ptr::null();
    }
    let handle = unsafe { &*value };
    let owner = unsafe { &*handle.owner };
    if let Some(p) = as_prop(&handle.kind) {
        if let Some(s) = string_form(p) {
            return owner.intern_string(&s);
        }
    }
    if let Some(Literal::String(s)) = as_literal(&handle.kind) {
        return owner.intern_string(s);
    }
    std::ptr::null()
}

fn as_node(kind: &RefKind) -> Option<&Node> {
    match kind {
        RefKind::Node(n) => Some(unsafe { &**n }),
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Node(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

fn as_edge(kind: &RefKind) -> Option<&Edge> {
    match kind {
        RefKind::Edge(e) => Some(unsafe { &**e }),
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Edge(e) => Some(e),
            _ => None,
        },
        _ => None,
    }
}

fn nth_prop(
    props: &BTreeMap<String, PropertyValue>,
    i: usize,
) -> Option<(&String, &PropertyValue)> {
    props.iter().nth(i)
}

macro_rules! with_handle {
    ($value:ident, $default:expr, |$handle:ident| $body:expr) => {{
        if $value.is_null() {
            return $default;
        }
        let $handle = unsafe { &*$value };
        $body
    }};
}

/// # Safety
/// See `marsdb_value_type`. (Same for every node/edge/list/map/path
/// accessor below.)
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_id(value: *const MarsdbValue) -> u64 {
    with_handle!(value, 0, |h| as_node(&h.kind).map_or(0, |n| n.id.0))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_label_count(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| as_node(&h.kind).map_or(0, |n| n.labels.len()))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_label(value: *const MarsdbValue, i: usize) -> *const c_char {
    with_handle!(value, std::ptr::null(), |h| {
        match as_node(&h.kind).and_then(|n| n.labels.get(i)) {
            Some(label) => unsafe { &*h.owner }.intern_string(label),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_prop_count(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| as_node(&h.kind).map_or(0, |n| n.props.len()))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_prop_name(
    value: *const MarsdbValue,
    i: usize,
) -> *const c_char {
    with_handle!(value, std::ptr::null(), |h| {
        match as_node(&h.kind).and_then(|n| nth_prop(&n.props, i)) {
            Some((name, _)) => unsafe { &*h.owner }.intern_string(name),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_node_prop_value(
    value: *const MarsdbValue,
    i: usize,
) -> *const MarsdbValue {
    with_handle!(value, std::ptr::null(), |h| {
        match as_node(&h.kind).and_then(|n| nth_prop(&n.props, i)) {
            Some((_, p)) => unsafe { &*h.owner }.make_handle(RefKind::Prop(p)),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_id(value: *const MarsdbValue) -> u64 {
    with_handle!(value, 0, |h| as_edge(&h.kind).map_or(0, |e| e.id.0))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_src(value: *const MarsdbValue) -> u64 {
    with_handle!(value, 0, |h| as_edge(&h.kind).map_or(0, |e| e.src.0))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_dst(value: *const MarsdbValue) -> u64 {
    with_handle!(value, 0, |h| as_edge(&h.kind).map_or(0, |e| e.dst.0))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_label(value: *const MarsdbValue) -> *const c_char {
    with_handle!(value, std::ptr::null(), |h| {
        match as_edge(&h.kind) {
            Some(e) => unsafe { &*h.owner }.intern_string(&e.label),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_prop_count(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| as_edge(&h.kind).map_or(0, |e| e.props.len()))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_prop_name(
    value: *const MarsdbValue,
    i: usize,
) -> *const c_char {
    with_handle!(value, std::ptr::null(), |h| {
        match as_edge(&h.kind).and_then(|e| nth_prop(&e.props, i)) {
            Some((name, _)) => unsafe { &*h.owner }.intern_string(name),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_edge_prop_value(
    value: *const MarsdbValue,
    i: usize,
) -> *const MarsdbValue {
    with_handle!(value, std::ptr::null(), |h| {
        match as_edge(&h.kind).and_then(|e| nth_prop(&e.props, i)) {
            Some((_, p)) => unsafe { &*h.owner }.make_handle(RefKind::Prop(p)),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_list_len(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| {
        match &h.kind {
            RefKind::Val(v) => match unsafe { &**v } {
                Value::List(items) => items.len(),
                Value::Property(PropertyValue::List(items)) => items.len(),
                _ => 0,
            },
            RefKind::Prop(p) => match unsafe { &**p } {
                PropertyValue::List(items) => items.len(),
                _ => 0,
            },
            _ => 0,
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_list_get(
    value: *const MarsdbValue,
    i: usize,
) -> *const MarsdbValue {
    with_handle!(value, std::ptr::null(), |h| {
        let owner = unsafe { &*h.owner };
        match &h.kind {
            RefKind::Val(v) => match unsafe { &**v } {
                Value::List(items) => match items.get(i) {
                    Some(item) => owner.make_handle(RefKind::Val(item)),
                    None => std::ptr::null(),
                },
                Value::Property(PropertyValue::List(items)) => match items.get(i) {
                    Some(item) => owner.make_handle(RefKind::Prop(item)),
                    None => std::ptr::null(),
                },
                _ => std::ptr::null(),
            },
            RefKind::Prop(p) => match unsafe { &**p } {
                PropertyValue::List(items) => match items.get(i) {
                    Some(item) => owner.make_handle(RefKind::Prop(item)),
                    None => std::ptr::null(),
                },
                _ => std::ptr::null(),
            },
            _ => std::ptr::null(),
        }
    })
}

fn as_map(kind: &RefKind) -> Option<&BTreeMap<String, Value>> {
    match kind {
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Map(m) => Some(m),
            _ => None,
        },
        _ => None,
    }
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_map_len(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| as_map(&h.kind).map_or(0, |m| m.len()))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_map_key(value: *const MarsdbValue, i: usize) -> *const c_char {
    with_handle!(value, std::ptr::null(), |h| {
        match as_map(&h.kind).and_then(|m| m.keys().nth(i)) {
            Some(key) => unsafe { &*h.owner }.intern_string(key),
            None => std::ptr::null(),
        }
    })
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_map_get(value: *const MarsdbValue, i: usize) -> *const MarsdbValue {
    with_handle!(value, std::ptr::null(), |h| {
        match as_map(&h.kind).and_then(|m| m.values().nth(i)) {
            Some(item) => unsafe { &*h.owner }.make_handle(RefKind::Val(item)),
            None => std::ptr::null(),
        }
    })
}

fn as_path(kind: &RefKind) -> Option<&Vec<PathElem>> {
    match kind {
        RefKind::Val(v) => match unsafe { &**v } {
            Value::Path(elems) => Some(elems),
            _ => None,
        },
        _ => None,
    }
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_path_len(value: *const MarsdbValue) -> usize {
    with_handle!(value, 0, |h| as_path(&h.kind).map_or(0, |p| p.len()))
}

/// # Safety
/// See `marsdb_node_id`.
#[no_mangle]
pub unsafe extern "C" fn marsdb_path_get(
    value: *const MarsdbValue,
    i: usize,
) -> *const MarsdbValue {
    with_handle!(value, std::ptr::null(), |h| {
        let owner = unsafe { &*h.owner };
        match as_path(&h.kind).and_then(|p| p.get(i)) {
            Some(PathElem::Node(n)) => owner.make_handle(RefKind::Node(n)),
            Some(PathElem::Edge(e)) => owner.make_handle(RefKind::Edge(e)),
            None => std::ptr::null(),
        }
    })
}

// ---- streaming ----------------------------------------------------------

pub type MarsdbRowCallback =
    unsafe extern "C" fn(user_data: *mut std::ffi::c_void, row_view: *const MarsdbResult) -> i32;

/// Adapts the C callback to the engine's `RowSink`: a persistent
/// one-row `MarsdbResult` view, refilled per row, so the callback uses
/// the exact same accessor family as a full result.
struct CallbackSink {
    view: MarsdbResult,
    on_row: MarsdbRowCallback,
    user_data: *mut std::ffi::c_void,
}

impl marsdb::RowSink for CallbackSink {
    fn columns(&mut self, columns: &[String]) {
        self.view.columns = columns
            .iter()
            .map(|c| CString::new(c.replace('\0', "")).unwrap_or_default())
            .collect();
    }
    fn row(&mut self, row: Vec<Value>) -> std::ops::ControlFlow<()> {
        self.view.handles.borrow_mut().clear();
        self.view.scratch.borrow_mut().clear();
        self.view.rows = vec![row];
        self.view.cursor = 0;
        if unsafe { (self.on_row)(self.user_data, &self.view) } != 0 {
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    }
}

fn stream_with_sink(
    db: &MarsdbDatabase,
    run: impl FnOnce(&mut CallbackSink) -> Result<(), marsdb::Error>,
    on_row: MarsdbRowCallback,
    user_data: *mut std::ffi::c_void,
) -> i32 {
    let mut sink = CallbackSink {
        view: MarsdbResult::from_query_result(QueryResult::default()),
        on_row,
        user_data,
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut sink))) {
        Ok(Ok(())) => MARSDB_OK,
        Ok(Err(e)) => db.set_error(e),
        Err(_) => db.set_error("internal panic while streaming query"),
    }
}

/// # Safety
/// `db` live; `cypher` valid NUL-terminated; `on_row` a valid function
/// pointer for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stream(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    on_row: MarsdbRowCallback,
    user_data: *mut std::ffi::c_void,
) -> i32 {
    if db.is_null() || cypher.is_null() {
        return MARSDB_ERROR;
    }
    let db = unsafe { &*db };
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return db.set_error(format!("cypher is not valid UTF-8: {e}")),
    };
    stream_with_sink(
        db,
        |sink| {
            db.inner
                .execute_streaming(cypher, &HashMap::new(), &ExecutionOptions::default(), sink)
        },
        on_row,
        user_data,
    )
}

/// # Safety
/// `stmt` live statement (database still live); `on_row` valid.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_stream(
    stmt: *mut MarsdbStatement,
    on_row: MarsdbRowCallback,
    user_data: *mut std::ffi::c_void,
) -> i32 {
    if stmt.is_null() {
        return MARSDB_ERROR;
    }
    let stmt = unsafe { &*stmt };
    let db = unsafe { &*stmt.db };
    stream_with_sink(
        db,
        |sink| {
            db.inner
                .execute_streaming_prepared(&stmt.stmt, &stmt.binds, &stmt.options(), sink)
        },
        on_row,
        user_data,
    )
}

// ---- batch lane ----------------------------------------------------------

#[repr(C)]
pub struct MarsdbBuffer {
    pub data: *mut u8,
    pub len: usize,
}

fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn write_svarint(out: &mut Vec<u8>, v: i64) {
    write_varint(out, ((v << 1) ^ (v >> 63)) as u64);
}

#[derive(Default)]
struct StringTable {
    map: HashMap<String, u64>,
    list: Vec<String>,
}

impl StringTable {
    fn intern(&mut self, s: &str) -> u64 {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.list.len() as u64;
        self.list.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }
}

const TAG_NULL: u8 = 0x00;
const TAG_BOOL: u8 = 0x01;
const TAG_INT: u8 = 0x02;
const TAG_FLOAT: u8 = 0x03;
const TAG_STRING: u8 = 0x04;
const TAG_DATE: u8 = 0x05;
const TAG_DURATION: u8 = 0x06;
const TAG_NODE: u8 = 0x07;
const TAG_EDGE: u8 = 0x08;
const TAG_LIST: u8 = 0x09;
const TAG_MAP: u8 = 0x0a;
const TAG_PATH: u8 = 0x0b;

fn encode_inline_str(out: &mut Vec<u8>, tag: u8, s: &str) {
    out.push(tag);
    write_varint(out, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
}

fn encode_prop(out: &mut Vec<u8>, table: &mut StringTable, p: &PropertyValue) {
    match p {
        PropertyValue::Null => out.push(TAG_NULL),
        PropertyValue::Bool(b) => {
            out.push(TAG_BOOL);
            out.push(u8::from(*b));
        }
        PropertyValue::Int(i) => {
            out.push(TAG_INT);
            write_svarint(out, *i);
        }
        PropertyValue::Float(f) => {
            out.push(TAG_FLOAT);
            out.extend_from_slice(&f.to_bits().to_le_bytes());
        }
        PropertyValue::String(s) => encode_inline_str(out, TAG_STRING, s),
        PropertyValue::Date(_) => {
            let text = string_form(p).expect("date always has a text form");
            encode_inline_str(out, TAG_DATE, &text);
        }
        PropertyValue::Duration { .. } => {
            let text = string_form(p).expect("duration always has a text form");
            encode_inline_str(out, TAG_DURATION, &text);
        }
        // Other temporals: ISO text as STRING, same as the accessors.
        PropertyValue::LocalTime(_)
        | PropertyValue::Time { .. }
        | PropertyValue::LocalDateTime { .. }
        | PropertyValue::DateTime { .. } => {
            let text = string_form(p).expect("temporal types always have a text form");
            encode_inline_str(out, TAG_STRING, &text);
        }
        PropertyValue::List(items) => {
            out.push(TAG_LIST);
            write_varint(out, items.len() as u64);
            for item in items {
                encode_prop(out, table, item);
            }
        }
        PropertyValue::Map(entries) => {
            out.push(TAG_MAP);
            write_varint(out, entries.len() as u64);
            for (key, item) in entries {
                write_varint(out, table.intern(key));
                encode_prop(out, table, item);
            }
        }
    }
}

fn encode_node(out: &mut Vec<u8>, table: &mut StringTable, n: &Node) {
    out.push(TAG_NODE);
    write_varint(out, n.id.0);
    write_varint(out, n.labels.len() as u64);
    for label in &n.labels {
        write_varint(out, table.intern(label));
    }
    write_varint(out, n.props.len() as u64);
    for (name, p) in &n.props {
        write_varint(out, table.intern(name));
        encode_prop(out, table, p);
    }
}

fn encode_edge(out: &mut Vec<u8>, table: &mut StringTable, e: &Edge) {
    out.push(TAG_EDGE);
    write_varint(out, e.id.0);
    write_varint(out, e.src.0);
    write_varint(out, e.dst.0);
    write_varint(out, table.intern(&e.label));
    write_varint(out, e.props.len() as u64);
    for (name, p) in &e.props {
        write_varint(out, table.intern(name));
        encode_prop(out, table, p);
    }
}

fn encode_value(out: &mut Vec<u8>, table: &mut StringTable, v: &Value) {
    match v {
        Value::Null => out.push(TAG_NULL),
        Value::Property(p) => encode_prop(out, table, p),
        Value::Literal(l) => match l {
            Literal::Null | Literal::Param(_) => out.push(TAG_NULL),
            Literal::Bool(b) => {
                out.push(TAG_BOOL);
                out.push(u8::from(*b));
            }
            Literal::Int(i) => {
                out.push(TAG_INT);
                write_svarint(out, *i);
            }
            Literal::Float(f) => {
                out.push(TAG_FLOAT);
                out.extend_from_slice(&f.to_bits().to_le_bytes());
            }
            Literal::String(s) => encode_inline_str(out, TAG_STRING, s),
        },
        Value::Node(n) => encode_node(out, table, n),
        Value::Edge(e) => encode_edge(out, table, e),
        Value::List(items) => {
            out.push(TAG_LIST);
            write_varint(out, items.len() as u64);
            for item in items {
                encode_value(out, table, item);
            }
        }
        Value::Map(entries) => {
            out.push(TAG_MAP);
            write_varint(out, entries.len() as u64);
            for (key, item) in entries {
                write_varint(out, table.intern(key));
                encode_value(out, table, item);
            }
        }
        Value::Path(elems) => {
            out.push(TAG_PATH);
            write_varint(out, elems.len() as u64);
            for elem in elems {
                match elem {
                    PathElem::Node(n) => encode_node(out, table, n),
                    PathElem::Edge(e) => encode_edge(out, table, e),
                }
            }
        }
    }
}

/// Batch format v1 — must match marsdb.h's spec block byte for byte.
fn encode_batch(result: &QueryResult) -> Vec<u8> {
    let mut table = StringTable::default();
    let column_ids: Vec<u64> = result.columns.iter().map(|c| table.intern(c)).collect();
    // Encode the body first (interning as it goes); the string table
    // isn't complete until every row has been walked, so it's spliced
    // in front afterwards.
    let mut body = Vec::new();
    write_varint(&mut body, column_ids.len() as u64);
    for id in &column_ids {
        write_varint(&mut body, *id);
    }
    write_varint(&mut body, result.rows.len() as u64);
    for row in &result.rows {
        for value in row {
            encode_value(&mut body, &mut table, value);
        }
    }
    let s = &result.stats;
    for counter in [
        s.nodes_created,
        s.nodes_deleted,
        s.relationships_created,
        s.relationships_deleted,
        s.properties_set,
        s.labels_added,
        s.labels_removed,
    ] {
        write_varint(&mut body, counter);
    }

    let mut out = Vec::with_capacity(body.len() + 64);
    out.push(1u8); // version
    write_varint(&mut out, table.list.len() as u64);
    for entry in &table.list {
        write_varint(&mut out, entry.len() as u64);
        out.extend_from_slice(entry.as_bytes());
    }
    out.extend_from_slice(&body);
    out
}

fn buffer_from(bytes: Vec<u8>) -> MarsdbBuffer {
    let boxed = bytes.into_boxed_slice();
    let len = boxed.len();
    MarsdbBuffer {
        data: Box::into_raw(boxed).cast(),
        len,
    }
}

/// # Safety
/// `db` live; `cypher` valid NUL-terminated; `out` valid.
#[no_mangle]
pub unsafe extern "C" fn marsdb_query_batch(
    db: *mut MarsdbDatabase,
    cypher: *const c_char,
    out: *mut MarsdbBuffer,
) -> i32 {
    if db.is_null() || cypher.is_null() || out.is_null() {
        return MARSDB_ERROR;
    }
    let db = unsafe { &*db };
    let cypher = match unsafe { CStr::from_ptr(cypher) }.to_str() {
        Ok(c) => c,
        Err(e) => return db.set_error(format!("cypher is not valid UTF-8: {e}")),
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| db.inner.execute(cypher))) {
        Ok(Ok(result)) => {
            unsafe { *out = buffer_from(encode_batch(&result)) };
            MARSDB_OK
        }
        Ok(Err(e)) => db.set_error(e),
        Err(_) => db.set_error("internal panic while executing query"),
    }
}

/// # Safety
/// `stmt` live statement (database still live); `out` valid.
#[no_mangle]
pub unsafe extern "C" fn marsdb_stmt_execute_batch(
    stmt: *mut MarsdbStatement,
    out: *mut MarsdbBuffer,
) -> i32 {
    if stmt.is_null() || out.is_null() {
        return MARSDB_ERROR;
    }
    let stmt = unsafe { &*stmt };
    let db = unsafe { &*stmt.db };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_statement(stmt))) {
        Ok(Ok(result)) => {
            unsafe { *out = buffer_from(encode_batch(&result)) };
            MARSDB_OK
        }
        Ok(Err(e)) => db.set_error(e),
        Err(_) => db.set_error("internal panic while executing query"),
    }
}

/// # Safety
/// `buffer` must have come from a batch call, freed at most once.
#[no_mangle]
pub unsafe extern "C" fn marsdb_buffer_free(buffer: MarsdbBuffer) {
    if !buffer.data.is_null() {
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(buffer.data, buffer.len)) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cstr(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    unsafe fn exec(db: *mut MarsdbDatabase, cypher: &str) -> *mut MarsdbResult {
        let c = cstr(cypher);
        let mut out: *mut MarsdbResult = std::ptr::null_mut();
        assert_eq!(unsafe { marsdb_query(db, c.as_ptr(), &mut out) }, MARSDB_OK);
        out
    }

    unsafe fn last_error(db: *const MarsdbDatabase) -> String {
        unsafe { CStr::from_ptr(marsdb_last_error(db)) }
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn typed_roundtrip_scalars_and_nested() {
        unsafe {
            let db = marsdb_open_in_memory();
            marsdb_result_destroy(exec(
                db,
                "CREATE (:N {i: 9223372036854775807, f: 1.5, s: 'héllo', b: true, \
                 tags: [1, 2, 3], d: date('1984-10-11')})",
            ));
            let r = exec(
                db,
                "MATCH (n:N) RETURN n, n.i AS i, n.f AS f, n.s AS s, n.b AS b, \
                 n.tags AS tags, n.d AS d, {city: 'Kyoto', zip: 1} AS m",
            );
            assert_eq!(marsdb_column_count(r), 8);
            assert_eq!(
                CStr::from_ptr(marsdb_column_name(r, 1)).to_str().unwrap(),
                "i"
            );
            assert_eq!(marsdb_next(r), 1);

            // Node with props.
            let node = marsdb_row_value(r, 0);
            assert_eq!(marsdb_value_type(node), MARSDB_TYPE_NODE);
            assert_eq!(marsdb_node_label_count(node), 1);
            assert_eq!(
                CStr::from_ptr(marsdb_node_label(node, 0)).to_str().unwrap(),
                "N"
            );
            assert_eq!(marsdb_node_prop_count(node), 6);
            // Props are BTreeMap-ordered: b, d, f, i, s, tags.
            assert_eq!(
                CStr::from_ptr(marsdb_node_prop_name(node, 0))
                    .to_str()
                    .unwrap(),
                "b"
            );
            let i_prop = marsdb_node_prop_value(node, 3);
            assert_eq!(marsdb_value_type(i_prop), MARSDB_TYPE_INT64);
            assert_eq!(marsdb_value_int64(i_prop), i64::MAX);

            // Scalars by column.
            assert_eq!(marsdb_value_int64(marsdb_row_value(r, 1)), i64::MAX);
            assert_eq!(marsdb_value_double(marsdb_row_value(r, 2)), 1.5);
            assert_eq!(
                CStr::from_ptr(marsdb_value_string(marsdb_row_value(r, 3)))
                    .to_str()
                    .unwrap(),
                "héllo"
            );
            assert_eq!(marsdb_value_bool(marsdb_row_value(r, 4)), 1);

            // List.
            let tags = marsdb_row_value(r, 5);
            assert_eq!(marsdb_value_type(tags), MARSDB_TYPE_LIST);
            assert_eq!(marsdb_list_len(tags), 3);
            assert_eq!(marsdb_value_int64(marsdb_list_get(tags, 2)), 3);

            // Date as ISO text with its own type.
            let d = marsdb_row_value(r, 6);
            assert_eq!(marsdb_value_type(d), MARSDB_TYPE_DATE);
            assert_eq!(
                CStr::from_ptr(marsdb_value_string(d)).to_str().unwrap(),
                "1984-10-11"
            );

            // Map.
            let m = marsdb_row_value(r, 7);
            assert_eq!(marsdb_value_type(m), MARSDB_TYPE_MAP);
            assert_eq!(marsdb_map_len(m), 2);
            assert_eq!(
                CStr::from_ptr(marsdb_map_key(m, 0)).to_str().unwrap(),
                "city"
            );
            assert_eq!(
                CStr::from_ptr(marsdb_value_string(marsdb_map_get(m, 0)))
                    .to_str()
                    .unwrap(),
                "Kyoto"
            );

            assert_eq!(marsdb_next(r), 0);
            marsdb_result_destroy(r);
            marsdb_close(db);
        }
    }

    #[test]
    fn prepare_bind_execute_reuse() {
        unsafe {
            let db = marsdb_open_in_memory();
            marsdb_result_destroy(exec(db, "CREATE (:P {k: 1}), (:P {k: 2}), (:P {k: 3})"));

            let cypher = cstr("MATCH (p:P) WHERE p.k > $min RETURN p.k AS k");
            let mut stmt: *mut MarsdbStatement = std::ptr::null_mut();
            assert_eq!(marsdb_prepare(db, cypher.as_ptr(), &mut stmt), MARSDB_OK);

            let name = cstr("min");
            assert_eq!(marsdb_bind_int64(stmt, name.as_ptr(), 1), MARSDB_OK);
            let mut r: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_stmt_execute(stmt, &mut r), MARSDB_OK);
            let mut count = 0;
            while marsdb_next(r) == 1 {
                count += 1;
            }
            assert_eq!(count, 2);
            marsdb_result_destroy(r);

            // Rebind, execute again -- parse happened once.
            assert_eq!(marsdb_bind_int64(stmt, name.as_ptr(), 2), MARSDB_OK);
            let mut r: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_stmt_execute(stmt, &mut r), MARSDB_OK);
            assert_eq!(marsdb_next(r), 1);
            assert_eq!(marsdb_value_int64(marsdb_row_value(r, 0)), 3);
            assert_eq!(marsdb_next(r), 0);
            marsdb_result_destroy(r);

            // Cleared bindings -> missing-param error via last_error.
            marsdb_clear_bindings(stmt);
            let mut r: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_stmt_execute(stmt, &mut r), MARSDB_ERROR);
            assert!(last_error(db).contains("missing value for parameter"));

            marsdb_stmt_destroy(stmt);
            marsdb_close(db);
        }
    }

    #[test]
    fn list_bind_and_bounds() {
        unsafe {
            let db = marsdb_open_in_memory();
            for i in 0..10 {
                marsdb_result_destroy(exec(db, &format!("CREATE (:N {{i: {i}}})")));
            }
            let cypher = cstr("MATCH (n:N) WHERE n.i IN $wanted RETURN n.i");
            let mut stmt: *mut MarsdbStatement = std::ptr::null_mut();
            assert_eq!(marsdb_prepare(db, cypher.as_ptr(), &mut stmt), MARSDB_OK);
            let name = cstr("wanted");
            let values = [2i64, 4, 6];
            assert_eq!(
                marsdb_bind_int64_list(stmt, name.as_ptr(), values.as_ptr(), values.len()),
                MARSDB_OK
            );
            let mut r: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_stmt_execute(stmt, &mut r), MARSDB_OK);
            let mut count = 0;
            while marsdb_next(r) == 1 {
                count += 1;
            }
            assert_eq!(count, 3);
            marsdb_result_destroy(r);

            // max_rows bound fails cleanly with the engine's message.
            let all = cstr("MATCH (n:N) RETURN n");
            let mut stmt2: *mut MarsdbStatement = std::ptr::null_mut();
            assert_eq!(marsdb_prepare(db, all.as_ptr(), &mut stmt2), MARSDB_OK);
            marsdb_stmt_set_max_rows(stmt2, 5);
            let mut r: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_stmt_execute(stmt2, &mut r), MARSDB_ERROR);
            assert!(
                last_error(db).contains("resource limit"),
                "{}",
                last_error(db)
            );

            marsdb_stmt_destroy(stmt);
            marsdb_stmt_destroy(stmt2);
            marsdb_close(db);
        }
    }

    #[test]
    fn stats_and_error_paths() {
        unsafe {
            let db = marsdb_open_in_memory();
            let r = exec(db, "CREATE (a:P)-[:R]->(b:P)");
            let stats = marsdb_result_stats(r);
            assert_eq!(stats.nodes_created, 2);
            assert_eq!(stats.relationships_created, 1);
            marsdb_result_destroy(r);

            // Syntax error -> ERROR + message.
            let bad = cstr("NOT CYPHER (((");
            let mut out: *mut MarsdbResult = std::ptr::null_mut();
            assert_eq!(marsdb_query(db, bad.as_ptr(), &mut out), MARSDB_ERROR);
            assert!(last_error(db).contains("syntax"), "{}", last_error(db));

            marsdb_close(db);
        }
    }

    #[test]
    fn streaming_typed_view() {
        unsafe extern "C" fn on_row(
            user_data: *mut std::ffi::c_void,
            row_view: *const MarsdbResult,
        ) -> i32 {
            let state = unsafe { &mut *(user_data as *mut (Vec<i64>, Option<usize>)) };
            let v = unsafe { marsdb_row_value(row_view, 0) };
            state.0.push(unsafe { marsdb_value_int64(v) });
            i32::from(state.1.is_some_and(|stop| state.0.len() >= stop))
        }

        unsafe {
            let db = marsdb_open_in_memory();
            for i in 0..10 {
                marsdb_result_destroy(exec(db, &format!("CREATE (:N {{i: {i}}})")));
            }
            let cypher = cstr("MATCH (n:N) RETURN n.i AS i");
            let mut state: (Vec<i64>, Option<usize>) = (vec![], None);
            assert_eq!(
                marsdb_stream(
                    db,
                    cypher.as_ptr(),
                    on_row,
                    (&mut state as *mut (Vec<i64>, Option<usize>)).cast(),
                ),
                MARSDB_OK
            );
            assert_eq!(state.0.len(), 10);

            // Early stop.
            let mut state: (Vec<i64>, Option<usize>) = (vec![], Some(3));
            assert_eq!(
                marsdb_stream(
                    db,
                    cypher.as_ptr(),
                    on_row,
                    (&mut state as *mut (Vec<i64>, Option<usize>)).cast(),
                ),
                MARSDB_OK
            );
            assert_eq!(state.0.len(), 3);

            // Prepared streaming with a bind.
            let pc = cstr("MATCH (n:N) WHERE n.i >= $min RETURN n.i AS i");
            let mut stmt: *mut MarsdbStatement = std::ptr::null_mut();
            assert_eq!(marsdb_prepare(db, pc.as_ptr(), &mut stmt), MARSDB_OK);
            let name = cstr("min");
            marsdb_bind_int64(stmt, name.as_ptr(), 7);
            let mut state: (Vec<i64>, Option<usize>) = (vec![], None);
            assert_eq!(
                marsdb_stmt_stream(
                    stmt,
                    on_row,
                    (&mut state as *mut (Vec<i64>, Option<usize>)).cast(),
                ),
                MARSDB_OK
            );
            assert_eq!(state.0, vec![7, 8, 9]);
            marsdb_stmt_destroy(stmt);

            // Non-streamable -> typed refusal.
            let agg = cstr("MATCH (n:N) RETURN count(n)");
            let mut state: (Vec<i64>, Option<usize>) = (vec![], None);
            assert_eq!(
                marsdb_stream(
                    db,
                    agg.as_ptr(),
                    on_row,
                    (&mut state as *mut (Vec<i64>, Option<usize>)).cast(),
                ),
                MARSDB_ERROR
            );
            assert!(last_error(db).contains("not streamable"));
            assert!(state.0.is_empty());

            marsdb_close(db);
        }
    }

    #[test]
    fn batch_encodes_and_hand_decodes() {
        // A tiny reference decoder proving the format spec in marsdb.h
        // is followable from the header alone.
        struct Reader<'a>(&'a [u8], usize);
        impl Reader<'_> {
            fn u8(&mut self) -> u8 {
                let b = self.0[self.1];
                self.1 += 1;
                b
            }
            fn varint(&mut self) -> u64 {
                let mut v = 0u64;
                let mut shift = 0;
                loop {
                    let b = self.u8();
                    v |= u64::from(b & 0x7f) << shift;
                    if b & 0x80 == 0 {
                        return v;
                    }
                    shift += 7;
                }
            }
            fn svarint(&mut self) -> i64 {
                let v = self.varint();
                ((v >> 1) as i64) ^ -((v & 1) as i64)
            }
            fn bytes(&mut self, n: usize) -> &[u8] {
                let s = &self.0[self.1..self.1 + n];
                self.1 += n;
                s
            }
        }

        unsafe {
            let db = marsdb_open_in_memory();
            marsdb_result_destroy(exec(
                db,
                "CREATE (:U {name: 'Ada', score: 9223372036854775807})-[:KNOWS {w: 1.5}]->(:U {name: 'Lin'})",
            ));
            let cypher = cstr("MATCH (a:U)-[r:KNOWS]->(b:U) RETURN a, r, b.name AS name");
            let mut buffer = MarsdbBuffer {
                data: std::ptr::null_mut(),
                len: 0,
            };
            assert_eq!(
                marsdb_query_batch(db, cypher.as_ptr(), &mut buffer),
                MARSDB_OK
            );
            let bytes = std::slice::from_raw_parts(buffer.data, buffer.len).to_vec();
            marsdb_buffer_free(buffer);
            marsdb_close(db);

            let mut r = Reader(&bytes, 0);
            assert_eq!(r.u8(), 1, "version");
            let table_len = r.varint() as usize;
            let mut table = Vec::with_capacity(table_len);
            for _ in 0..table_len {
                let n = r.varint() as usize;
                table.push(String::from_utf8(r.bytes(n).to_vec()).unwrap());
            }
            let column_count = r.varint() as usize;
            let columns: Vec<&str> = (0..column_count)
                .map(|_| table[r.varint() as usize].as_str())
                .collect();
            assert_eq!(columns, ["a", "r", "name"]);
            let row_count = r.varint() as usize;
            assert_eq!(row_count, 1);

            // a: node
            assert_eq!(r.u8(), 0x07);
            let _id = r.varint();
            assert_eq!(r.varint(), 1, "one label");
            assert_eq!(table[r.varint() as usize], "U");
            assert_eq!(r.varint(), 2, "two props");
            // BTreeMap order: name, score.
            assert_eq!(table[r.varint() as usize], "name");
            assert_eq!(r.u8(), 0x04);
            let n = r.varint() as usize;
            assert_eq!(r.bytes(n), b"Ada");
            assert_eq!(table[r.varint() as usize], "score");
            assert_eq!(r.u8(), 0x02);
            assert_eq!(r.svarint(), i64::MAX, "int64 exact through the batch");

            // r: edge
            assert_eq!(r.u8(), 0x08);
            let _ = (r.varint(), r.varint(), r.varint()); // id, src, dst
            assert_eq!(table[r.varint() as usize], "KNOWS");
            assert_eq!(r.varint(), 1);
            assert_eq!(table[r.varint() as usize], "w");
            assert_eq!(r.u8(), 0x03);
            assert_eq!(
                f64::from_bits(u64::from_le_bytes(r.bytes(8).try_into().unwrap())),
                1.5
            );

            // name: inline string
            assert_eq!(r.u8(), 0x04);
            let n = r.varint() as usize;
            assert_eq!(r.bytes(n), b"Lin");

            // stats trailer: 7 varints, all zero for a read.
            for _ in 0..7 {
                assert_eq!(r.varint(), 0);
            }
            assert_eq!(r.1, bytes.len(), "fully consumed");
        }
    }
}
