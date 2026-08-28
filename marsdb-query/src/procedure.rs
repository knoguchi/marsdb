//! Pluggable stored-procedure support (`CALL proc(...) YIELD ...`).
//!
//! MarsDB itself ships no built-in procedures -- this only defines the
//! interface an embedder (or a test harness, see `marsdb-tck`) implements
//! to make `CALL` resolve to something real. `Executor` never invents
//! procedure behavior on its own; every `CALL` fails with "procedure not
//! found" unless a provider is supplied via `ExecutionOptions::procedures`.

use std::sync::Arc;

use crate::error::QueryError;
use crate::value::Value;

/// A procedure's declared shape -- everything `Executor` needs to validate
/// a `CALL` before invoking `ProcedureProvider::call`: arity, coarse
/// argument-type compatibility, and output column names for `YIELD`.
#[derive(Debug, Clone)]
pub struct ProcedureSignature {
    /// Declared input parameter names, in order -- used for the arity
    /// check and for resolving a standalone call's implicit arguments
    /// (`CALL proc`, no parens) from same-named `$params`.
    pub inputs: Vec<String>,
    /// Each input's declared type, in the same order as `inputs`:
    /// `INTEGER`/`FLOAT`/`NUMBER`/`STRING`/`BOOLEAN`, optionally `?`-suffixed
    /// for nullable (`Value::Null` is always accepted regardless).
    /// Unrecognized type names are treated as "accept anything" -- this is
    /// a coarse compile-time check, not a full type system.
    pub input_types: Vec<String>,
    /// Declared output column names, in order -- what `YIELD *`/an
    /// unqualified `YIELD name` binds, and how `call`'s rows are shaped.
    pub outputs: Vec<String>,
}

/// Implemented by whatever embeds MarsDB to make `CALL` resolve to real
/// behavior. `Executor` calls `signature` once per `CALL` for validation
/// and `call` once per input row (once total for a standalone call).
pub trait ProcedureProvider: Send + Sync {
    /// `None` means "no such procedure".
    fn signature(&self, name: &str) -> Option<ProcedureSignature>;
    /// `args` is already evaluated and type-checked against
    /// `signature(name)`'s `input_types`, in declared-input order. Returns
    /// output rows, each with exactly `outputs.len()` values in order.
    fn call(&self, name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, QueryError>;
}

/// `Arc<dyn ProcedureProvider>` wrapper with manual `Debug` so
/// `ExecutionOptions` (which embeds this) can keep deriving both.
#[derive(Clone)]
pub struct Procedures(pub Arc<dyn ProcedureProvider>);

impl std::fmt::Debug for Procedures {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Procedures(..)")
    }
}
