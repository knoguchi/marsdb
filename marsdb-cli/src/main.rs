use std::process::ExitCode;

use clap::Parser;
use marsdb::Database;

mod format;
mod repl;

/// MarsDB: an embeddable property-graph database. Single binary, single file.
#[derive(Parser)]
#[command(name = "marsdb")]
struct Cli {
    /// Database file path, or `:memory:` for a transient in-memory database.
    /// Omit entirely for an in-memory database.
    file: Option<String>,

    /// Cypher query to run once, non-interactively. Omit to start a REPL.
    query: Option<String>,

    /// Shorthand for an in-memory database (same as passing `:memory:`).
    #[arg(long)]
    memory: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let use_memory = cli.memory || cli.file.as_deref() == Some(":memory:");
    let db = if use_memory {
        Database::in_memory()
    } else if let Some(path) = &cli.file {
        Database::open(path)
    } else {
        Database::in_memory()
    };
    let db = match db {
        Ok(db) => db,
        Err(e) => {
            eprintln!("marsdb: failed to open database: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(query) = &cli.query {
        return match db.execute_batch(query) {
            Ok(results) => {
                for result in &results {
                    format::print_table(result);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("marsdb: {e}");
                ExitCode::FAILURE
            }
        };
    }

    repl::run(&db)
}
