mod gherkin;
mod tck_value;

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use gherkin::{Expected, InitialGraph, Scenario};
use marsdb::{Database, PropertyValue};
use marsdb_query::QueryError;
use tck_value::{tck_eq, value_to_tck, TckScalar, TckValue};

const BINARY_TREE_1: &str = include_str!("../graphs/binary-tree-1.cypher");
const BINARY_TREE_2: &str = include_str!("../graphs/binary-tree-2.cypher");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Outcome {
    Pass,
    WrongResult,
    UnexpectedOutcome,
    ParseRejected,
    RunnerUnsupported,
}

struct ScenarioReport {
    category: String, // e.g. "clauses/match"
    feature_name: String,
    name: String,
    outcome: Outcome,
    detail: Option<String>,
}

fn main() {
    let features_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("features");
    let mut reports = Vec::new();

    for entry in walk_feature_files(&features_dir) {
        let category = entry
            .strip_prefix(&features_dir)
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(&entry).unwrap_or_else(|e| panic!("read {entry:?}: {e}"));
        for scenario_result in gherkin::parse_feature(&content) {
            match scenario_result {
                Ok(scenario) => {
                    let (outcome, detail) = run_scenario(&scenario);
                    reports.push(ScenarioReport {
                        category: category.clone(),
                        feature_name: scenario.feature_name,
                        name: scenario.name,
                        outcome,
                        detail,
                    });
                }
                Err(reason) => {
                    reports.push(ScenarioReport {
                        category: category.clone(),
                        feature_name: entry.file_name().unwrap().to_string_lossy().to_string(),
                        name: "<unparsed scenario>".to_string(),
                        outcome: Outcome::RunnerUnsupported,
                        detail: Some(reason),
                    });
                }
            }
        }
    }

    report(&reports);
}

fn walk_feature_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_feature_files(&path));
        } else if path.extension().is_some_and(|e| e == "feature") {
            out.push(path);
        }
    }
    out
}

fn run_scenario(scenario: &Scenario) -> (Outcome, Option<String>) {
    let db = Database::in_memory().expect("in-memory database always opens");

    let setup_result = (|| -> Result<(), marsdb::Error> {
        match &scenario.initial_graph {
            InitialGraph::Empty | InitialGraph::Any => {}
            InitialGraph::Named(name) => {
                let fixture = match name.as_str() {
                    "binary-tree-1" => BINARY_TREE_1,
                    "binary-tree-2" => BINARY_TREE_2,
                    _ => return Err(unsupported(format!("unknown named graph: {name}"))),
                };
                db.execute_batch(fixture)?;
            }
        }
        for stmt in &scenario.setup_cypher {
            db.execute_batch(stmt)?;
        }
        Ok(())
    })();

    if let Err(e) = setup_result {
        return classify_setup_error(e);
    }

    let params: HashMap<String, PropertyValue> = match convert_params(&scenario.params) {
        Ok(p) => p,
        Err(reason) => return (Outcome::RunnerUnsupported, Some(reason)),
    };

    let result = db.execute_with_params(&scenario.query, &params);

    match &scenario.expected {
        Expected::AnyError => match result {
            Ok(_) => (Outcome::UnexpectedOutcome, Some("expected an error, query succeeded".to_string())),
            Err(_) => (Outcome::Pass, None),
        },
        Expected::Empty => match result {
            Ok(r) if r.rows.is_empty() => (Outcome::Pass, None),
            Ok(r) => (Outcome::WrongResult, Some(format!("expected no rows, got {} row(s)", r.rows.len()))),
            Err(e) => classify_query_error(e),
        },
        Expected::Rows { row_order_matters, list_order_matters, header, rows } => match result {
            Ok(r) => compare_rows(&r, header, rows, *row_order_matters, *list_order_matters),
            Err(e) => classify_query_error(e),
        },
    }
}

fn classify_setup_error(e: marsdb::Error) -> (Outcome, Option<String>) {
    if let marsdb::Error::Query(QueryError::Parse(msg)) = &e {
        if msg.starts_with("__unsupported__") {
            return (Outcome::RunnerUnsupported, Some(msg.trim_start_matches("__unsupported__").to_string()));
        }
    }
    (Outcome::ParseRejected, Some(format!("setup failed: {e}")))
}

fn unsupported(reason: String) -> marsdb::Error {
    // Piggybacks on QueryError::Parse purely as a carrier so this closure
    // can return one Result type -- classify_setup_error() unwraps the
    // sentinel prefix back out. Not a real parse error.
    marsdb::Error::Query(QueryError::Parse(format!("__unsupported__{reason}")))
}

fn classify_query_error(e: marsdb::Error) -> (Outcome, Option<String>) {
    match &e {
        marsdb::Error::Query(QueryError::Parse(_)) => (Outcome::ParseRejected, Some(e.to_string())),
        _ => (Outcome::UnexpectedOutcome, Some(e.to_string())),
    }
}

fn convert_params(params: &[(String, String)]) -> Result<HashMap<String, PropertyValue>, String> {
    let mut out = HashMap::new();
    for (name, literal_text) in params {
        let tck_val = tck_value::parse_cell(literal_text).map_err(|e| format!("param {name}: {e}"))?;
        let pv = match tck_val {
            TckValue::Null => PropertyValue::Null,
            TckValue::Scalar(TckScalar::Int(i)) => PropertyValue::Int(i),
            TckValue::Scalar(TckScalar::Float(f)) => PropertyValue::Float(f),
            TckValue::Scalar(TckScalar::Str(s)) => PropertyValue::String(s),
            TckValue::Scalar(TckScalar::Bool(b)) => PropertyValue::Bool(b),
            other => return Err(format!("param {name}: list/node/rel-valued params aren't supported: {other:?}")),
        };
        out.insert(name.clone(), pv);
    }
    Ok(out)
}

fn compare_rows(
    result: &marsdb::QueryResult,
    expected_header: &[String],
    expected_rows: &[Vec<String>],
    row_order_matters: bool,
    list_order_matters: bool,
) -> (Outcome, Option<String>) {
    if result.columns.len() != expected_header.len() {
        return (
            Outcome::WrongResult,
            Some(format!("column count mismatch: expected {:?}, got {:?}", expected_header, result.columns)),
        );
    }
    if result.rows.len() != expected_rows.len() {
        return (
            Outcome::WrongResult,
            Some(format!("row count mismatch: expected {}, got {}", expected_rows.len(), result.rows.len())),
        );
    }

    let mut expected_parsed: Vec<Vec<TckValue>> = Vec::with_capacity(expected_rows.len());
    for row in expected_rows {
        let mut parsed_row = Vec::with_capacity(row.len());
        for cell in row {
            match tck_value::parse_cell(cell) {
                Ok(v) => parsed_row.push(v),
                Err(e) => return (Outcome::RunnerUnsupported, Some(format!("couldn't parse expected cell {cell:?}: {e}"))),
            }
        }
        expected_parsed.push(parsed_row);
    }
    let actual_parsed: Vec<Vec<TckValue>> =
        result.rows.iter().map(|row| row.iter().map(value_to_tck).collect()).collect();

    let row_eq = |a: &[TckValue], b: &[TckValue]| a.iter().zip(b).all(|(x, y)| tck_eq(x, y, list_order_matters));

    let matched = if row_order_matters {
        actual_parsed.iter().zip(&expected_parsed).all(|(a, e)| row_eq(a, e))
    } else {
        let mut remaining: Vec<&Vec<TckValue>> = expected_parsed.iter().collect();
        actual_parsed.iter().all(|a| {
            let Some(pos) = remaining.iter().position(|e| row_eq(a, e)) else { return false };
            remaining.remove(pos);
            true
        })
    };

    if matched {
        (Outcome::Pass, None)
    } else {
        (
            Outcome::WrongResult,
            Some(format!("expected {expected_rows:?}, got {:?}", result.rows.iter().map(|r| format!("{r:?}")).collect::<Vec<_>>())),
        )
    }
}

fn report(reports: &[ScenarioReport]) {
    let mut by_category: BTreeMap<&str, BTreeMap<Outcome, usize>> = BTreeMap::new();
    for r in reports {
        *by_category.entry(&r.category).or_default().entry(r.outcome).or_default() += 1;
    }

    println!("{:<32} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}", "category", "total", "pass", "wrong", "unexp", "reject", "unsup");
    let mut totals: BTreeMap<Outcome, usize> = BTreeMap::new();
    for (category, counts) in &by_category {
        let total: usize = counts.values().sum();
        for (outcome, n) in counts {
            *totals.entry(*outcome).or_default() += n;
        }
        println!(
            "{:<32} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
            category,
            total,
            counts.get(&Outcome::Pass).unwrap_or(&0),
            counts.get(&Outcome::WrongResult).unwrap_or(&0),
            counts.get(&Outcome::UnexpectedOutcome).unwrap_or(&0),
            counts.get(&Outcome::ParseRejected).unwrap_or(&0),
            counts.get(&Outcome::RunnerUnsupported).unwrap_or(&0),
        );
    }
    let grand_total: usize = totals.values().sum();
    println!(
        "{:<32} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "TOTAL",
        grand_total,
        totals.get(&Outcome::Pass).unwrap_or(&0),
        totals.get(&Outcome::WrongResult).unwrap_or(&0),
        totals.get(&Outcome::UnexpectedOutcome).unwrap_or(&0),
        totals.get(&Outcome::ParseRejected).unwrap_or(&0),
        totals.get(&Outcome::RunnerUnsupported).unwrap_or(&0),
    );

    let wrong: Vec<&ScenarioReport> = reports.iter().filter(|r| r.outcome == Outcome::WrongResult).collect();
    if !wrong.is_empty() {
        println!("\n--- WrongResult scenarios (real bugs, not coverage gaps) ---");
        for r in &wrong {
            println!("[{}] {} :: {}", r.category, r.feature_name, r.name);
            if let Some(d) = &r.detail {
                println!("    {d}");
            }
        }
    }
}
