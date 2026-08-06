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
/// a `CALL` at the point it's about to run (arity, coarse argument-type
/// compatibility, output column names for `YIELD`), before ever asking
/// `ProcedureProvider::call` to actually produce rows.
#[derive(Debug, Clone)]
pub struct ProcedureSignature {
    /// Declared input parameter names, in order -- used both for the
    /// `InvalidNumberOfArguments` arity check and for resolving a
    /// standalone call's *implicit* arguments (`CALL proc`, no parens) from
    /// same-named `$params`.
    pub inputs: Vec<String>,
    /// Each input's declared type, in the same order as `inputs` -- one of
    /// `INTEGER`/`FLOAT`/`NUMBER`/`STRING`/`BOOLEAN` (optionally suffixed
    /// with `?` for nullable, which every real procedure signature is in
    /// practice; the `?` doesn't change compatibility checking here since
    /// `Value::Null` is always accepted regardless of declared type).
    /// Unrecognized type names are tolerated (treated as "accept
    /// anything") rather than rejected -- this is a coarse compile-time
    /// sanity check, not a full type system.
    pub input_types: Vec<String>,
    /// Declared output column names, in order -- what `YIELD *`/an
    /// unqualified `YIELD name` (no `AS`) binds, and what `call`'s own
    /// returned rows are positionally shaped as.
    pub outputs: Vec<String>,
}

/// Implemented by whatever embeds MarsDB to make `CALL` resolve to real
/// behavior. `Executor` calls `signature` once per `CALL` (for compile-
/// time-shaped validation) and `call` once per input row (standalone: once
/// total, since there's no input row to iterate).
pub trait ProcedureProvider: Send + Sync {
    /// `None` means "no such procedure" -- `Executor` reports this as a
    /// `ProcedureError`/`ProcedureNotFound`-flavored `QueryError` (no
    /// dedicated `QueryError` variant for this taxonomy — see
    /// `CYPHER_COVERAGE.md`'s own error-taxonomy scope note; any error is
    /// enough for TCK's coarse checking).
    fn signature(&self, name: &str) -> Option<ProcedureSignature>;
    /// `args` is already fully evaluated and type-checked against
    /// `signature(name)`'s `input_types`, in declared-input order. Returns
    /// the procedure's output rows, each with exactly `signature(name)`'s
    /// `outputs.len()` values in that same order -- `Executor` never
    /// inspects a row's shape beyond that, so how a provider produces them
    /// (a fixed lookup table, real computation, ...) is entirely up to it.
    fn call(&self, name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, QueryError>;
}

/// `Arc<dyn ProcedureProvider>` wrapper -- same "manual `Debug`, derived
/// `Clone`" shape `executor::ExecutionObserver` already uses for its own
/// `Arc<dyn Fn(..)>`, so `ExecutionOptions` (which embeds this) can keep
/// deriving both.
#[derive(Clone)]
pub struct Procedures(pub Arc<dyn ProcedureProvider>);

impl std::fmt::Debug for Procedures {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Procedures(..)")
    }
}
