//! Converts a literal-Cypher bulk-load dump (statements of the shape
//! `UNWIND [<huge literal list>] AS row ...`) into a parameterized form
//! (`UNWIND $rows AS row ...` + a serialized `$rows` param) for
//! `profile_load.rs` to replay.
//!
//! Split into its own binary so a flamegraph of the replay phase isn't
//! buried under the conversion phase's cost: re-parsing every literal
//! once would dominate any profile it's part of.
//!
//! Usage: `cargo run -p marsdb --example convert_only --release -- \
//!   <schema.cypher> <data.cypher> <out.postcard>`, then
//! `cargo flamegraph --example profile_load -- <db_path> <schema.cypher> \
//!   <out.postcard>`.

use marsdb::{Database, Literal, PropertyValue, Value};
use std::env;
use std::fs;

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
    let out_file = &args[3];
    let schema = fs::read_to_string(schema_file)?;
    let script = fs::read_to_string(data_file)?;

    let path = "/tmp/param-load-convert.db";
    let _ = fs::remove_file(path);
    let db = Database::open(path)?;
    db.execute_batch(&schema)?;

    let stmts = marsdb_query::split_statements(&script);
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

    let bytes = postcard::to_allocvec(&rewritten)?;
    fs::write(out_file, &bytes)?;
    println!(
        "converted {} statements -> {out_file} ({} bytes)",
        rewritten.len(),
        bytes.len()
    );
    Ok(())
}
