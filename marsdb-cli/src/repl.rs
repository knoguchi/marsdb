use std::process::ExitCode;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use marsdb::Database;

use crate::format;

pub fn run(db: &Database) -> ExitCode {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("mars: failed to start REPL: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("Mars graph database. Enter Cypher statements terminated by `;`. Ctrl-D to exit.");

    let mut buffer = String::new();
    loop {
        let prompt = if buffer.is_empty() { "mars> " } else { "  ...> " };
        match rl.readline(prompt) {
            Ok(line) => {
                buffer.push_str(&line);
                buffer.push(' ');
                if line.trim_end().ends_with(';') {
                    let query = buffer.trim().trim_end_matches(';').trim().to_string();
                    buffer.clear();
                    if query.is_empty() {
                        continue;
                    }
                    let _ = rl.add_history_entry(query.as_str());
                    match db.execute(&query) {
                        Ok(result) => format::print_table(&result),
                        Err(e) => eprintln!("mars: {e}"),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("mars: readline error: {e}");
                break;
            }
        }
    }
    ExitCode::SUCCESS
}
