use std::io::{IsTerminal, Read};
use std::process::ExitCode;

use clap::Parser;
use marsdb::Database;

mod format;
mod nl;
mod repl;

/// MarsDB: an embeddable property-graph database. Single binary, single file.
#[derive(Parser)]
#[command(name = "marsdb")]
struct Cli {
    /// Database file path, or `:memory:` for a transient in-memory database.
    /// Omit entirely for an in-memory database.
    file: Option<String>,

    /// Cypher query to run once, non-interactively. Omit to start a REPL
    /// (or, if stdin isn't a terminal, read and run a `;`-separated batch
    /// from it instead).
    query: Option<String>,

    /// Shorthand for an in-memory database (same as passing `:memory:`).
    #[arg(long)]
    memory: bool,

    /// Ask a question in plain English instead of Cypher; translates it via
    /// a local Ollama instance and runs the result. Read-only.
    #[arg(long, value_name = "QUESTION")]
    nl: Option<String>,
}

fn run_batch(db: &Database, cypher: &str) -> ExitCode {
    match db.execute_batch(cypher) {
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
    }
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

    if let Some(question) = &cli.nl {
        return nl::run(&db, question);
    }

    if let Some(query) = &cli.query {
        return run_batch(&db, query);
    }

    let mut stdin = std::io::stdin();
    if !stdin.is_terminal() {
        let mut input = String::new();
        if let Err(e) = stdin.read_to_string(&mut input) {
            eprintln!("marsdb: failed to read stdin: {e}");
            return ExitCode::FAILURE;
        }
        return run_batch(&db, &input);
    }

    repl::run(&db)
}
