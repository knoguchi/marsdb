use std::process::ExitCode;

use marsdb::Database;
use marsdb_nl2cypher::{translate_and_run, LlmClient};
use serde_json::json;

use crate::format;

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

pub fn run(db: &Database, question: &str) -> ExitCode {
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
    let client = OllamaClient { model };

    match translate_and_run(db, &client, question) {
        Ok((cypher, result)) => {
            println!("{cypher}");
            format::print_table(&result);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mars: {e}");
            ExitCode::FAILURE
        }
    }
}
