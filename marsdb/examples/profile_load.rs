//! Flamegraph target for the *replay-only* phase (group commit over an
//! already-parameterized statement list) -- run convert_only.rs first to
//! produce the converted file. Args: <db_path> <schema_file> <converted_file>

use marsdb::PropertyValue;
use marsdb_graph::GraphStore;
use marsdb_query::Executor;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let db_path = &args[1];
    let schema_file = &args[2];
    let converted_file = &args[3];

    let schema = fs::read_to_string(schema_file)?;
    let bytes = fs::read(converted_file)?;
    let rewritten: Vec<(String, Option<PropertyValue>)> = postcard::from_bytes(&bytes)?;

    let _ = fs::remove_file(db_path);
    let store = GraphStore::open_file(db_path)?;
    let executor = Executor::new(&store);
    for stmt in marsdb_query::parse_many(&schema)? {
        executor.execute(&stmt)?;
    }

    let t = Instant::now();
    let group_size = 1000;
    // The same template text repeats across many rows/groups (every
    // `UNWIND $rows AS row CREATE ...` for a given node/edge type is
    // byte-identical, only $rows differs) -- parse each unique text once,
    // substitute into a clone of the pristine AST per use, instead of
    // re-parsing text on every iteration.
    let mut template_cache: HashMap<&str, marsdb_query::Statement> = HashMap::new();
    for group in rewritten.chunks(group_size) {
        let write_txn = store.begin_write()?;
        for (stmt_text, param) in group {
            let template = match template_cache.get(stmt_text.as_str()) {
                Some(stmt) => stmt,
                None => {
                    let parsed = marsdb_query::parse(stmt_text)?;
                    template_cache.entry(stmt_text.as_str()).or_insert(parsed)
                }
            };
            let mut stmt = template.clone();
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
        "replay-only: {} statements, group_size={group_size}: {:?}",
        rewritten.len(),
        t.elapsed()
    );
    Ok(())
}
