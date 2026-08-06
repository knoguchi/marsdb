//! Mock-procedure support for `CALL` scenarios (`clauses/call/*.feature`)
//! -- each scenario's own `And there exists a procedure ...` step(s)
//! (parsed into `gherkin::ProcedureFixture`) become a tiny fixed lookup
//! table: `TckProcedureProvider::call` filters that table's rows by
//! equality against the actual call arguments (on the declared input
//! columns only) and projects the declared output columns, confirmed
//! against every vendored `clauses/call/*.feature` scenario (e.g. Call5
//! `[8]`'s `CALL test.my.proc('Stefan', 1) YIELD *` only returns the one
//! row whose `name`/`id` columns match). This is a fixture-driven test
//! double, not a real procedure implementation -- MarsDB itself has no
//! built-in procedures (see `marsdb_query::ProcedureProvider`'s own
//! docs).

use std::collections::HashMap;

use marsdb::{Literal, ProcedureProvider, ProcedureSignature, Value};
use marsdb_query::QueryError;

use crate::gherkin::ProcedureFixture;
use crate::tck_value::{self, tck_eq, value_to_tck, TckScalar, TckValue};

pub struct TckProcedureProvider {
    procs: HashMap<String, ProcedureFixture>,
}

impl TckProcedureProvider {
    pub fn new(fixtures: Vec<ProcedureFixture>) -> Self {
        Self {
            procs: fixtures.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }
}

impl ProcedureProvider for TckProcedureProvider {
    fn signature(&self, name: &str) -> Option<ProcedureSignature> {
        let fx = self.procs.get(name)?;
        Some(ProcedureSignature {
            inputs: fx.input_names.clone(),
            input_types: fx.input_types.clone(),
            outputs: fx.output_names.clone(),
        })
    }

    fn call(&self, name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, QueryError> {
        self.call_inner(name, args)
            .map_err(|e| QueryError::Semantic(format!("__unsupported__{e}")))
    }
}

impl TckProcedureProvider {
    /// Own error type is plain `String` (a fixture-parsing/lookup
    /// problem, not a real Cypher runtime error) -- `call` above wraps it
    /// in the `__unsupported__`-prefixed `QueryError::Semantic` sentinel
    /// `main.rs`'s own `classify_setup_error`/`unsupported` already use
    /// for "this is a harness gap, not the engine getting something
    /// wrong."
    fn call_inner(&self, name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, String> {
        let fx = self
            .procs
            .get(name)
            .ok_or_else(|| format!("procedure '{name}' not found"))?;
        let column_index = |col: &str| -> Result<usize, String> {
            fx.header
                .iter()
                .position(|h| h == col)
                .ok_or_else(|| format!("procedure '{name}' fixture table has no '{col}' column"))
        };
        let input_idxs = fx
            .input_names
            .iter()
            .map(|n| column_index(n))
            .collect::<Result<Vec<_>, _>>()?;
        let output_idxs = fx
            .output_names
            .iter()
            .map(|n| column_index(n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut out = Vec::new();
        'row: for row in &fx.rows {
            for (arg, &idx) in args.iter().zip(&input_idxs) {
                let cell = tck_value::parse_cell(&row[idx])
                    .map_err(|e| format!("procedure '{name}' fixture cell: {e}"))?;
                if !call_values_equal(arg, &cell) {
                    continue 'row;
                }
            }
            let projected = output_idxs
                .iter()
                .map(|&idx| {
                    let cell = tck_value::parse_cell(&row[idx])
                        .map_err(|e| format!("procedure '{name}' fixture cell: {e}"))?;
                    tck_to_call_value(&cell)
                })
                .collect::<Result<Vec<_>, _>>()?;
            out.push(projected);
        }
        Ok(out)
    }
}

/// `TckValue` -> a real `marsdb_query::Value` a procedure call can return
/// -- only the scalar/list shapes any vendored `clauses/call/*.feature`
/// fixture table actually uses (TCK's Call1-6 never declare a node/rel/
/// map-typed procedure column); anything else is a clear error rather
/// than a silent wrong conversion.
fn tck_to_call_value(v: &TckValue) -> Result<Value, String> {
    Ok(match v {
        TckValue::Null => Value::Null,
        TckValue::Scalar(TckScalar::Int(i)) => Value::Literal(Literal::Int(*i)),
        TckValue::Scalar(TckScalar::Float(f)) => Value::Literal(Literal::Float(*f)),
        TckValue::Scalar(TckScalar::Str(s)) => Value::Literal(Literal::String(s.clone())),
        TckValue::Scalar(TckScalar::Bool(b)) => Value::Literal(Literal::Bool(*b)),
        TckValue::List(items) => Value::List(
            items
                .iter()
                .map(tck_to_call_value)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        other => return Err(format!("procedure fixture value not supported: {other:?}")),
    })
}

/// Equality used to filter a fixture table's rows by the actual call
/// arguments -- `value_to_tck` normalizes `arg` regardless of whether it's
/// a literal or a property-sourced value (`Value::Literal`/`Value::
/// Property` both reach this from different call sites: an explicit
/// argument expression's own evaluation, or a `PropertyValue` looked up
/// by name for the implicit-argument form, TCK's Call2 `[3]`), and real
/// Cypher's own numeric equality crosses INTEGER/FLOAT (TCK's Call3
/// `[5]`/`[6]`: an `INTEGER` argument matches a `FLOAT?`-declared
/// column's `42.0` row) -- `tck_eq`'s derived-`PartialEq` fallback
/// wouldn't give either of those for free (different `Value` shapes,
/// different `TckScalar` variants).
fn call_values_equal(arg: &Value, cell: &TckValue) -> bool {
    let arg = value_to_tck(arg);
    match (&arg, cell) {
        (TckValue::Scalar(TckScalar::Int(x)), TckValue::Scalar(TckScalar::Float(y)))
        | (TckValue::Scalar(TckScalar::Float(y)), TckValue::Scalar(TckScalar::Int(x))) => {
            *x as f64 == *y
        }
        _ => tck_eq(&arg, cell, true),
    }
}
