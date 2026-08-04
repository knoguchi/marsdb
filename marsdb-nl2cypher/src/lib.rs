//! Natural-language -> Cypher translation for MarsDB.
//!
//! Introspects a database's schema (labels, relationship types, and
//! property keys actually in use — see [`introspect_schema`]), builds a
//! grounded prompt (see [`build_prompt`]), calls a caller-supplied
//! [`LlmClient`], and validates the result by actually parsing it with
//! `marsdb_query::parse` — with one repair attempt if the first response
//! doesn't parse, feeding the exact parse error back (this codebase's
//! error messages are unusually descriptive, which this leans on
//! directly).
//!
//! No HTTP/LLM-SDK dependency here — bring your own [`LlmClient`]. See
//! `examples/ollama_demo.rs` for a real, runnable one against a local
//! Ollama instance.

use std::collections::BTreeSet;
use std::fmt;

use marsdb::{Database, QueryResult, Value};

/// Implemented by the caller for whatever LLM they want to use (OpenAI,
/// Anthropic, a local Ollama instance, ...). Deliberately synchronous —
/// nothing in the MarsDB workspace uses an async runtime, and a blocking
/// call is fine for a translate-then-run call, not a high-throughput
/// server path.
pub trait LlmClient {
    fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct LabelInfo {
    pub label: String,
    pub count: usize,
    /// Sorted union of property keys observed across every node with this
    /// label — good enough for prompt grounding, not a strict schema (a
    /// property present on only one node still shows up here).
    pub properties: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RelTypeInfo {
    pub rel_type: String,
    pub count: usize,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaSummary {
    pub node_labels: Vec<LabelInfo>,
    pub rel_types: Vec<RelTypeInfo>,
}

#[derive(Debug)]
pub enum Nl2CypherError {
    Llm(Box<dyn std::error::Error>),
    Execute(marsdb::Error),
    /// Neither the first attempt nor the one repair attempt produced
    /// parseable Cypher — both are included so the caller (or a human
    /// reading the error) can see exactly what the LLM tried and why it
    /// didn't work, not just "it failed."
    InvalidCypher { attempts: Vec<(String, String)> },
}

impl fmt::Display for Nl2CypherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nl2CypherError::Llm(e) => write!(f, "LLM call failed: {e}"),
            Nl2CypherError::Execute(e) => write!(f, "query execution failed: {e}"),
            Nl2CypherError::InvalidCypher { attempts } => {
                writeln!(f, "generated Cypher never parsed, after {} attempt(s):", attempts.len())?;
                for (i, (cypher, err)) in attempts.iter().enumerate() {
                    writeln!(f, "  attempt {}: {cypher:?}\n    error: {err}", i + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Nl2CypherError {}

/// Runs `MATCH (n) RETURN n` and `MATCH ()-[r]->() RETURN r` and
/// aggregates over the results — an O(graph size) scan, same cost a
/// dedicated schema-tracking index would still pay for a feature that
/// only needs to run once before a translation, not per query. No
/// dedicated schema API exists in `marsdb-graph`/`marsdb` to build this
/// from instead (labels are interned internally but nothing public
/// enumerates them), and building one wouldn't avoid this same scan
/// anyway, so this stays entirely in terms of the existing public
/// `Database::execute` API.
pub fn introspect_schema(db: &Database) -> Result<SchemaSummary, Nl2CypherError> {
    let mut labels: std::collections::BTreeMap<String, (usize, BTreeSet<String>)> = Default::default();
    let nodes = db.execute("MATCH (n) RETURN n").map_err(Nl2CypherError::Execute)?;
    for row in &nodes.rows {
        if let Value::Node(n) = &row[0] {
            for label in &n.labels {
                let entry = labels.entry(label.clone()).or_default();
                entry.0 += 1;
                entry.1.extend(n.props.keys().cloned());
            }
        }
    }

    let mut rel_types: std::collections::BTreeMap<String, (usize, BTreeSet<String>)> = Default::default();
    let edges = db.execute("MATCH ()-[r]->() RETURN r").map_err(Nl2CypherError::Execute)?;
    for row in &edges.rows {
        if let Value::Edge(e) = &row[0] {
            let entry = rel_types.entry(e.label.clone()).or_default();
            entry.0 += 1;
            entry.1.extend(e.props.keys().cloned());
        }
    }

    Ok(SchemaSummary {
        node_labels: labels
            .into_iter()
            .map(|(label, (count, properties))| LabelInfo { label, count, properties: properties.into_iter().collect() })
            .collect(),
        rel_types: rel_types
            .into_iter()
            .map(|(rel_type, (count, properties))| RelTypeInfo {
                rel_type,
                count,
                properties: properties.into_iter().collect(),
            })
            .collect(),
    })
}

/// Deliberately hand-written and kept short, not derived from
/// `README.md` at build/runtime — every token here is a token in every
/// prompt. Beyond what's supported, this specifically calls out the
/// sharp edges an LLM trained on general Neo4j Cypher will otherwise
/// reach for (the bare `-->` shorthand, multi-hop `MERGE`, `*` inside a
/// named path) — telling the model what *not* to try is a real, standard
/// technique for cutting down failed generations against a narrower
/// dialect than what it was trained on.
const CAPABILITIES: &str = "\
MarsDB supports a subset of openCypher. Rules that matter for generating valid queries:
- Relationships must be written explicitly as -[:TYPE]-> or -[:TYPE]-. The bare --> shorthand
  does not exist here -- always name the relationship type in brackets.
- MATCH, OPTIONAL MATCH, WHERE, WITH (at most one WITH boundary per statement), RETURN
  (optionally RETURN DISTINCT), ORDER BY, LIMIT.
- CREATE. MERGE with ON CREATE SET / ON MATCH SET, capped at exactly one relationship hop --
  never write a MERGE pattern with two or more relationships.
- UNWIND <list> AS x, where <list> is an inline literal list (e.g. [1, 2, 3]) or a variable
  bound by a preceding WITH ... collect(...).
- Aggregation: count(), count(*), sum(), avg(), min(), max(), collect(), with implicit
  GROUP BY -- every non-aggregating RETURN/WITH item automatically becomes a grouping key,
  there is no GROUP BY keyword. DISTINCT also works inside an aggregate call, e.g.
  count(DISTINCT x).
- Arithmetic: + - * / % with real precedence (* / % bind tighter than + -), usable in RETURN/
  WITH items, ORDER BY keys, and function arguments -- but never nested inside an aggregate's
  surrounding expression, e.g. 1 + count(x) is rejected (count(x) itself is fine as one whole
  return item). + also concatenates two strings. Not usable inside a WHERE clause's comparison
  operands yet -- only in RETURN/WITH/ORDER BY.
- Named path capture: MATCH p = (a)-[:TYPE]->(b) RETURN p -- fixed-hop patterns only, never
  a variable-length (*) hop inside a named path.
- shortestPath((a)-[:TYPE*..N]-(b)): both endpoints must already be matched by a preceding
  MATCH in the same query -- never use shortestPath() on a node that isn't already bound.
- String literals use single quotes with backslash escapes: \\' \\\" \\\\ \\n \\r \\t \\b \\f.
- Parameters are written as $name.
A query meant to produce output must end in a RETURN clause.
";

/// Builds the prompt sent to the LLM. `prior_attempt`, when `Some((cypher,
/// parse_error))`, turns this into a repair prompt instead of a fresh one
/// — see [`translate`].
pub fn build_prompt(schema: &SchemaSummary, question: &str, prior_attempt: Option<(&str, &str)>) -> String {
    let mut prompt = String::new();
    prompt.push_str("You translate natural-language questions into Cypher queries for MarsDB.\n\n");
    prompt.push_str(CAPABILITIES);
    prompt.push_str("\nDatabase schema:\n");
    if schema.node_labels.is_empty() && schema.rel_types.is_empty() {
        prompt.push_str("  (empty database -- no nodes or relationships exist yet)\n");
    }
    for l in &schema.node_labels {
        let props = if l.properties.is_empty() { "(none observed)".to_string() } else { l.properties.join(", ") };
        prompt.push_str(&format!("  (:{}) -- {} node(s), properties seen: {props}\n", l.label, l.count));
    }
    for r in &schema.rel_types {
        let props = if r.properties.is_empty() { "(none observed)".to_string() } else { r.properties.join(", ") };
        prompt.push_str(&format!("  -[:{}]-> -- {} edge(s), properties seen: {props}\n", r.rel_type, r.count));
    }

    if let Some((prev_cypher, err)) = prior_attempt {
        prompt.push_str(&format!(
            "\nYour previous attempt did not parse:\n{prev_cypher}\n\nParse error:\n{err}\n\nFix it and try again.\n"
        ));
    }

    prompt.push_str(&format!(
        "\nQuestion: {question}\n\nRespond with ONLY the Cypher query itself -- no explanation, no markdown code fence.\n"
    ));
    prompt
}

/// Strips a markdown code fence if the LLM wrapped its answer in one
/// despite being asked not to (common enough in practice to handle
/// defensively rather than let it silently break parsing).
fn extract_cypher(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed.to_string();
    };
    let rest = rest.strip_prefix("cypher").unwrap_or(rest);
    let rest = rest.trim_start_matches('\n');
    match rest.rfind("```") {
        Some(end) => rest[..end].trim().to_string(),
        None => rest.trim().to_string(),
    }
}

/// Translates `question` into Cypher against `schema`, validating the
/// result by actually parsing it. One repair attempt on failure — not an
/// open-ended retry loop, matching this codebase's stance throughout of
/// erroring clearly over retrying indefinitely hoping it works.
pub fn translate(client: &dyn LlmClient, schema: &SchemaSummary, question: &str) -> Result<String, Nl2CypherError> {
    let prompt = build_prompt(schema, question, None);
    let raw = client.complete(&prompt).map_err(Nl2CypherError::Llm)?;
    let cypher = extract_cypher(&raw);
    if marsdb_query::parse(&cypher).is_ok() {
        return Ok(cypher);
    }
    let first_err = marsdb_query::parse(&cypher).unwrap_err().to_string();

    let repair_prompt = build_prompt(schema, question, Some((&cypher, &first_err)));
    let raw2 = client.complete(&repair_prompt).map_err(Nl2CypherError::Llm)?;
    let cypher2 = extract_cypher(&raw2);
    match marsdb_query::parse(&cypher2) {
        Ok(_) => Ok(cypher2),
        Err(second_err) => Err(Nl2CypherError::InvalidCypher {
            attempts: vec![(cypher, first_err), (cypher2, second_err.to_string())],
        }),
    }
}

/// The common case in one call: introspect, translate, execute.
pub fn translate_and_run(
    db: &Database,
    client: &dyn LlmClient,
    question: &str,
) -> Result<(String, QueryResult), Nl2CypherError> {
    let schema = introspect_schema(db)?;
    let cypher = translate(client, &schema, question)?;
    let result = db.execute(&cypher).map_err(Nl2CypherError::Execute)?;
    Ok((cypher, result))
}
