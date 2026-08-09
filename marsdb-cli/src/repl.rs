use std::process::ExitCode;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use marsdb::Database;

use crate::format;

pub fn run(db: &Database) -> ExitCode {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("marsdb: failed to start REPL: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "MarsDB graph database. Enter Cypher statements terminated by `;`, \
         or `.help` for meta commands. Ctrl-D to exit."
    );

    let mut buffer = String::new();
    loop {
        let prompt = if buffer.is_empty() {
            "marsdb> "
        } else {
            "   ...> "
        };
        match rl.readline(prompt) {
            Ok(line) => {
                // Dot meta commands run immediately (no `;` needed), but
                // only from a fresh prompt -- a continuation line of a
                // half-typed statement is never intercepted.
                if buffer.is_empty() && line.trim_start().starts_with('.') {
                    let _ = rl.add_history_entry(line.trim());
                    match crate::meta::run(db, &line) {
                        Some(Ok(text)) => println!("{text}"),
                        Some(Err(e)) => eprintln!("marsdb: {e}"),
                        None => {}
                    }
                    continue;
                }
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
                        Err(e) => eprintln!("marsdb: {e}"),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("marsdb: readline error: {e}");
                break;
            }
        }
    }
    ExitCode::SUCCESS
}
