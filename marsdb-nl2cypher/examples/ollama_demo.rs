//! Real, runnable NL -> Cypher demo against a local Ollama instance --
//! nothing in `marsdb-nl2cypher`'s core lib depends on Ollama or any HTTP
//! client (see the crate's docs); this is just one concrete `LlmClient`
//! implementation.
//!
//! Setup:
//!     ollama serve &
//!     ollama pull llama3.2
//!
//! Run:
//!     cargo run -p marsdb-nl2cypher --example ollama_demo
//!     OLLAMA_MODEL=qwen3-coder:30b cargo run -p marsdb-nl2cypher --example ollama_demo  # use a different model already pulled

use marsdb::Database;
use marsdb_nl2cypher::{translate_and_run, LlmClient};
use serde_json::json;

struct OllamaClient {
    model: String,
}

impl LlmClient for OllamaClient {
    fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response: serde_json::Value = ureq::post("http://localhost:11434/api/generate")
            .send_json(json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false,
            }))?
            .into_json()?;
        let text = response
            .get("response")
            .and_then(|v| v.as_str())
            .ok_or("Ollama response had no \"response\" field")?;
        Ok(text.to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::in_memory()?;
    db.execute(
        "CREATE (a:Person {name: 'Alice', role: 'Engineer'})-[:KNOWS]->(b:Person {name: 'Bob', role: 'Designer'})\
                -[:KNOWS]->(c:Person {name: 'Carol', role: 'Engineer'})",
    )?;
    db.execute("CREATE (:Person {name: 'Dave', role: 'Manager'})")?;

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
    println!("Using model: {model}\n");
    let client = OllamaClient { model };

    for question in [
        "How many people are there?",
        "Who does Alice know?",
        "How many engineers are there?",
    ] {
        println!("Q: {question}");
        match translate_and_run(&db, &client, question) {
            Ok((cypher, result)) => {
                println!("  Cypher: {cypher}");
                println!("  Columns: {:?}", result.columns);
                for row in &result.rows {
                    println!("  {row:?}");
                }
            }
            Err(e) => println!("  Failed: {e}"),
        }
        println!();
    }

    Ok(())
}
