//! NL -> Cypher against the real recommendations dataset (28,863 nodes,
//! 166,261 relationships), not the toy graph ollama_demo.rs uses.
//!
//! Setup: ollama serve & ; ollama pull qwen3-coder:30b (or set OLLAMA_MODEL)
//! Run:   cargo run -p marsdb-nl2cypher --example recommendations_nl -- /path/to/recommendations.db

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
            .body_mut()
            .read_json()?;
        let text = response
            .get("response")
            .and_then(|v| v.as_str())
            .ok_or("Ollama response had no \"response\" field")?;
        Ok(text.to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .expect("usage: recommendations_nl <db-path>");
    let db = Database::open(&db_path)?;

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3-coder:30b".to_string());
    println!("Using model: {model}, db: {db_path}\n");
    let client = OllamaClient { model };

    for question in [
        "How many movies are there?",
        "Who directed Inception?",
        "What movies is Robert De Niro in? Show 5.",
        "What is the average rating for movies in the Horror genre?",
        "Who are the top 5 users by number of ratings given?",
    ] {
        println!("Q: {question}");
        match translate_and_run(&db, &client, question) {
            Ok((cypher, result)) => {
                println!("  Cypher: {cypher}");
                println!("  Columns: {:?}", result.columns);
                for row in result.rows.iter().take(10) {
                    println!("  {row:?}");
                }
            }
            Err(e) => println!("  Failed: {e}"),
        }
        println!();
    }

    Ok(())
}
