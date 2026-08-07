//! Runs each `# name: <label>` / statement block in a file (queries.cypher/
//! updates.cypher/deletes.cypher's shared format) against an existing
//! database, printing name, elapsed time, and row count per block.
//! Not shipped -- scratch tool for marsdb-demo's benchmarks/recommendations.
//!
//! Usage: run_named_queries <db-path> <blocks-file> [repeat-count]
//!
//! `repeat-count` (default 1) runs the whole file that many times without
//! printing each pass -- a single pass over a handful of ms-scale read
//! queries finishes in under a second, not enough wall-clock for a sampling
//! profiler (`cargo flamegraph`) to collect a meaningful number of samples.
//! Only meant for a read-only `blocks-file` (repeating writes/deletes
//! against the same rows isn't idempotent).

use std::env;
use std::fs;
use std::time::Instant;

use marsdb::Database;

fn run_once(db: &Database, text: &str, print: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut name = None;
    let mut stmt = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# name:") {
            name = Some(rest.trim().to_string());
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        stmt.push_str(line);
        stmt.push('\n');
        if trimmed.ends_with(';') {
            let label = name.take().unwrap_or_else(|| "unnamed".to_string());
            let t0 = Instant::now();
            let result = db.execute(&stmt);
            if print {
                match &result {
                    Ok(r) => println!("{label}: {:?} ({} rows)", t0.elapsed(), r.rows.len()),
                    Err(e) => println!("{label}: ERROR {e}"),
                }
            }
            result?;
            stmt.clear();
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let db_path = &args[1];
    let blocks_path = &args[2];
    let repeat: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(1);

    let db = Database::open(db_path)?;
    let text = fs::read_to_string(blocks_path)?;

    let t0 = Instant::now();
    for i in 0..repeat {
        run_once(&db, &text, i == 0)?;
    }
    println!("{repeat} pass(es) in {:?}", t0.elapsed());
    Ok(())
}
