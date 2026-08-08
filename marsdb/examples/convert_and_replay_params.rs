//! One-off tool: converts a literal-Cypher bulk-load dump (statements of
//! the shape `UNWIND [<huge literal list>] AS row ...`) into a
//! parameterized form (`UNWIND $rows AS row ...` + a `$rows` param), then
//! replays the parameterized form through group commit. Measures the
//! conversion cost separately from the replay cost, since only the
//! replay cost is what a repeat load actually pays.

use marsdb::{Database, Literal, PropertyValue, Value};
use marsdb_graph::GraphStore;
use marsdb_query::Executor;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Instant;

fn literal_to_pv(lit: Literal) -> PropertyValue {
    match lit {
        Literal::Int(i) => PropertyValue::Int(i),
        Literal::Float(f) => PropertyValue::Float(f),
        Literal::String(s) => PropertyValue::String(s),
        Literal::Bool(b) => PropertyValue::Bool(b),
        Literal::Null => PropertyValue::Null,
        Literal::Param(name) => unreachable!("param ${name} in a literal dump"),
    }
}

/// Lossless `Value -> PropertyValue`, unlike the engine's internal
/// property-storage conversion (which intentionally drops `Value::Map`,
/// since a map is never itself stored as a node/edge property) -- here a
/// `Value::Map` is exactly what a `$rows` element needs to become.
fn value_to_pv(v: Value) -> PropertyValue {
    match v {
        Value::Null => PropertyValue::Null,
        Value::Property(pv) => pv,
        Value::Literal(lit) => literal_to_pv(lit),
        Value::List(items) => PropertyValue::List(items.into_iter().map(value_to_pv).collect()),
        Value::Map(m) => {
            PropertyValue::Map(m.into_iter().map(|(k, v)| (k, value_to_pv(v))).collect())
        }
        Value::Node(_) | Value::Edge(_) | Value::Path(_) => {
            unreachable!("a literal dump's UNWIND source can't contain a node/edge/path")
        }
    }
}

/// Finds the index of the `]` matching the `[` at `open_idx`, honoring
/// quoted regions exactly like `marsdb_query::split_statements` does (a
/// `[`/`]` inside a string literal isn't a real bracket).
fn find_matching_bracket(s: &str, open_idx: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut i = open_idx;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q != b'`' {
                    i += 1;
                } else if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'\'' | b'"' | b'`' => quote = Some(b),
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let schema_file = &args[1];
    let data_file = &args[2];
    let schema = fs::read_to_string(schema_file)?;
    let script = fs::read_to_string(data_file)?;

    let path = "/tmp/param-load-convert.db";
    let _ = fs::remove_file(path);
    let db = Database::open(path)?;
    db.execute_batch(&schema)?;

    let stmts = marsdb_query::split_statements(&script);

    let t = Instant::now();
    let mut rewritten: Vec<(String, Option<PropertyValue>)> = Vec::with_capacity(stmts.len());
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        let after_unwind = trimmed.strip_prefix("UNWIND").map(str::trim_start);
        if let Some(open_idx) = after_unwind
            .filter(|rest| rest.starts_with('['))
            .map(|rest| trimmed.len() - rest.len())
        {
            if let Some(close_idx) = find_matching_bracket(trimmed, open_idx) {
                let literal_text = &trimmed[open_idx..=close_idx];
                let after = &trimmed[close_idx + 1..];
                let result = db.execute(&format!("RETURN {literal_text} AS rows"))?;
                let value = result.rows[0][0].clone();
                let pv = value_to_pv(value);
                rewritten.push((format!("UNWIND $rows{after}"), Some(pv)));
                continue;
            }
        }
        rewritten.push((trimmed.to_string(), None));
    }
    println!(
        "conversion (one-time, dominated by parsing/evaluating the original literals): {:?}",
        t.elapsed()
    );

    // Replay the parameterized form through group commit -- this is the
    // cost a *repeat* load actually pays, once the conversion above has
    // been done once and its result reused.
    let replay_path = "/tmp/param-load-replay.db";
    let _ = fs::remove_file(replay_path);
    let store = GraphStore::open_file(replay_path)?;
    let executor = Executor::new(&store);
    for stmt in marsdb_query::parse_many(&schema)? {
        executor.execute(&stmt)?;
    }

    let t2 = Instant::now();
    let group_size = 1000;
    for group in rewritten.chunks(group_size) {
        let write_txn = store.begin_write()?;
        for (stmt_text, param) in group {
            let mut stmt = marsdb_query::parse(stmt_text)?;
            if let Some(pv) = param {
                let mut params = HashMap::new();
                params.insert("rows".to_string(), pv.clone());
                marsdb_query::substitute_params(&mut stmt, &params)?;
            }
            executor.execute_in_write_transaction(&stmt, &write_txn)?;
        }
        GraphStore::commit(write_txn)?;
    }
    println!(
        "param-bound replay ({} statements, group_size={group_size}): {:?}",
        rewritten.len(),
        t2.elapsed()
    );

    Ok(())
}
