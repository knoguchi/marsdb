use std::cell::RefCell;

use marsdb::Database;
use marsdb_nl2cypher::{introspect_schema, translate, LlmClient, Nl2CypherError, SchemaSummary};

/// Returns each entry in `responses` in order, one per `complete()` call —
/// lets a test script exactly what the "LLM" says on the first call vs. a
/// repair-round second call, with no real network I/O.
struct FakeLlmClient {
    responses: RefCell<std::vec::IntoIter<String>>,
}

impl FakeLlmClient {
    fn new(responses: Vec<&str>) -> Self {
        Self { responses: RefCell::new(responses.into_iter().map(String::from).collect::<Vec<_>>().into_iter()) }
    }
}

impl LlmClient for FakeLlmClient {
    fn complete(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.responses.borrow_mut().next().ok_or_else(|| "FakeLlmClient: no more scripted responses".into())
    }
}

#[test]
fn translate_succeeds_on_first_valid_response() {
    let client = FakeLlmClient::new(vec!["MATCH (n:Person) RETURN n.name"]);
    let schema = SchemaSummary::default();
    let cypher = translate(&client, &schema, "who are all the people?").unwrap();
    assert_eq!(cypher, "MATCH (n:Person) RETURN n.name");
}

#[test]
fn translate_strips_a_markdown_code_fence() {
    let client = FakeLlmClient::new(vec!["```cypher\nMATCH (n:Person) RETURN n.name\n```"]);
    let schema = SchemaSummary::default();
    let cypher = translate(&client, &schema, "who are all the people?").unwrap();
    assert_eq!(cypher, "MATCH (n:Person) RETURN n.name");
}

#[test]
fn translate_repairs_after_one_bad_attempt() {
    // The first response uses the bare `-->` shorthand, which doesn't
    // parse -- the repair round should get a real chance to fix it.
    let client = FakeLlmClient::new(vec![
        "MATCH (a:Person)-->(b:Person) RETURN a.name, b.name",
        "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name",
    ]);
    let schema = SchemaSummary::default();
    let cypher = translate(&client, &schema, "who knows whom?").unwrap();
    assert_eq!(cypher, "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name");
}

#[test]
fn translate_gives_up_after_the_repair_attempt_also_fails() {
    let client = FakeLlmClient::new(vec![
        "MATCH (a)-->(b) RETURN a",
        "MATCH (a)-->(b) RETURN a", // still broken -- no repair actually happened
    ]);
    let schema = SchemaSummary::default();
    let err = translate(&client, &schema, "anything").unwrap_err();
    let Nl2CypherError::InvalidCypher { attempts } = err else {
        panic!("expected InvalidCypher, got a different error variant");
    };
    assert_eq!(attempts.len(), 2);
    assert!(attempts[0].1.to_lowercase().contains("expected") || !attempts[0].1.is_empty());
}

#[test]
fn introspect_schema_reports_labels_rel_types_and_properties() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:Person {name: 'Alice', age: 30})-[:KNOWS {since: 2020}]->(:Person {name: 'Bob'})").unwrap();
    db.execute("CREATE (:Company {name: 'Acme'})").unwrap();

    let schema = introspect_schema(&db).unwrap();

    let person = schema.node_labels.iter().find(|l| l.label == "Person").expect("Person label present");
    assert_eq!(person.count, 2);
    assert_eq!(person.properties, vec!["age".to_string(), "name".to_string()]);

    let company = schema.node_labels.iter().find(|l| l.label == "Company").expect("Company label present");
    assert_eq!(company.count, 1);
    assert_eq!(company.properties, vec!["name".to_string()]);

    let knows = schema.rel_types.iter().find(|r| r.rel_type == "KNOWS").expect("KNOWS rel type present");
    assert_eq!(knows.count, 1);
    assert_eq!(knows.properties, vec!["since".to_string()]);
}

#[test]
fn introspect_schema_on_empty_database() {
    let db = Database::in_memory().unwrap();
    let schema = introspect_schema(&db).unwrap();
    assert!(schema.node_labels.is_empty());
    assert!(schema.rel_types.is_empty());
}
