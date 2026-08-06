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
    UnexpectedBehavior,
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
    let features_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("openCypher/tck/features");
    if !features_dir.is_dir() {
        eprintln!(
            "{features_dir:?} doesn't exist -- the openCypher submodule isn't checked out.\nRun: git submodule update --init marsdb-tck/openCypher"
        );
        std::process::exit(1);
    }
    // Phase 1 diagnostic only (mars-0mn): compares the vendored ANTLR
    // grammar's parse-acceptance rate against pest's on the real TCK corpus,
    // before investing in Phase 2's clause-by-clause rewrite. Doesn't touch
    // the normal report() path below.
    if std::env::var("ANTLR_SPIKE").is_ok() {
        antlr_spike_report(&features_dir);
        return;
    }

    let filter = std::env::var("TCK_FILTER").ok();
    let mut reports = Vec::new();

    for entry in walk_feature_files(&features_dir) {
        let rel_path = entry.strip_prefix(&features_dir).unwrap();
        if let Some(filter) = &filter {
            if !rel_path.to_string_lossy().contains(filter.as_str()) {
                continue;
            }
        }
        let category = rel_path.parent().unwrap().to_string_lossy().to_string();
        let content =
            std::fs::read_to_string(&entry).unwrap_or_else(|e| panic!("read {entry:?}: {e}"));
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

/// Phase 1 diagnostic only (mars-0mn, see `main`'s `ANTLR_SPIKE` check) --
/// walks every TCK scenario's query text through both grammars in
/// parse-only mode (no execution) and reports accept/reject counts side by
/// side, plus scenarios where they disagree. Delete once Phase 2's
/// visitor-based parser replaces `marsdb_query::antlr_accepts`.
fn antlr_spike_report(features_dir: &Path) {
    let mut antlr_accept = 0usize;
    let mut antlr_reject = 0usize;
    let mut pest_accept = 0usize;
    let mut disagreements: Vec<(String, String, bool, bool)> = Vec::new();
    let mut disagreement_count = 0usize;
    let mut antlr_rejects: Vec<(String, String, bool)> = Vec::new();
    let mut antlr_wrongly_accepted: Vec<(String, String)> = Vec::new();
    let mut positive_cases = 0usize;
    let mut negative_cases = 0usize;
    let mut negative_syntax_cases = 0usize;
    let mut negative_syntax_wrongly_accepted = 0usize;
    let filter = std::env::var("TCK_FILTER").ok();

    for entry in walk_feature_files(features_dir) {
        if let Some(filter) = &filter {
            let rel_path = entry.strip_prefix(features_dir).unwrap();
            if !rel_path.to_string_lossy().contains(filter.as_str()) {
                continue;
            }
        }
        let content =
            std::fs::read_to_string(&entry).unwrap_or_else(|e| panic!("read {entry:?}: {e}"));
        for scenario_result in gherkin::parse_feature(&content) {
            let Ok(scenario) = scenario_result else {
                continue;
            };
            let antlr_ok = marsdb_query::antlr_accepts(&scenario.query);
            let pest_ok = marsdb_query::parse(&scenario.query).is_ok();
            let expects_error = matches!(scenario.expected, Expected::AnyError);
            let expects_syntax_error = scenario
                .expected_error_line
                .as_deref()
                .is_some_and(|l| l.contains("SyntaxError"));
            if expects_error {
                negative_cases += 1;
                if expects_syntax_error {
                    negative_syntax_cases += 1;
                }
            } else {
                positive_cases += 1;
            }
            if antlr_ok {
                antlr_accept += 1;
                if expects_error {
                    antlr_wrongly_accepted
                        .push((scenario.feature_name.clone(), scenario.query.clone()));
                    if expects_syntax_error {
                        negative_syntax_wrongly_accepted += 1;
                    }
                }
            } else {
                antlr_reject += 1;
                antlr_rejects.push((
                    scenario.feature_name.clone(),
                    scenario.query.clone(),
                    expects_error,
                ));
            }
            if pest_ok {
                pest_accept += 1;
            }
            if antlr_ok != pest_ok {
                disagreement_count += 1;
                disagreements.push((
                    scenario.feature_name.clone(),
                    scenario.query.clone(),
                    antlr_ok,
                    pest_ok,
                ));
            }
        }
    }

    let total = antlr_accept + antlr_reject;
    println!(
        "{total} total scenarios: {positive_cases} positive (should parse), {negative_cases} negative (should reject, Expected::AnyError), of which {negative_syntax_cases} are tagged SyntaxError specifically ({} semantic/type/other) -- antlr wrongly accepts {negative_syntax_wrongly_accepted} of the {negative_syntax_cases} SyntaxError cases",
        negative_cases - negative_syntax_cases,
    );
    println!(
        "pest:  {pest_accept:>5}/{total} accepted ({:.1}%)",
        100.0 * pest_accept as f64 / total as f64
    );
    println!(
        "antlr: {antlr_accept:>5}/{total} accepted ({:.1}%)",
        100.0 * antlr_accept as f64 / total as f64
    );
    println!(
        "\n{disagreement_count} disagreements -- written to /tmp/antlr_spike_disagreements.txt"
    );
    let mut out = String::new();
    for (feature, query, antlr_ok, pest_ok) in &disagreements {
        out.push_str(&format!(
            "[{feature}] antlr={} pest={} :: {}\n",
            antlr_ok,
            pest_ok,
            query.replace('\n', " ")
        ));
    }
    std::fs::write("/tmp/antlr_spike_disagreements.txt", out)
        .expect("write /tmp/antlr_spike_disagreements.txt");

    let wrongly_rejected = antlr_rejects
        .iter()
        .filter(|(_, _, expects_error)| !expects_error)
        .count();
    println!(
        "{} total antlr rejects ({} correctly reject invalid syntax, {} are real bugs) -- written to /tmp/antlr_all_rejects.txt",
        antlr_rejects.len(),
        antlr_rejects.len() - wrongly_rejected,
        wrongly_rejected,
    );
    let mut out = String::new();
    for (feature, query, expects_error) in &antlr_rejects {
        let tag = if *expects_error { "expected" } else { "BUG" };
        out.push_str(&format!(
            "[{tag}] [{feature}] :: {}\n",
            query.replace('\n', " ")
        ));
    }
    std::fs::write("/tmp/antlr_all_rejects.txt", out).expect("write /tmp/antlr_all_rejects.txt");

    println!(
        "{} wrongly accepted (should have rejected per Expected::AnyError) -- written to /tmp/antlr_wrongly_accepted.txt",
        antlr_wrongly_accepted.len()
    );
    let mut out = String::new();
    for (feature, query) in &antlr_wrongly_accepted {
        out.push_str(&format!("[{feature}] :: {}\n", query.replace('\n', " ")));
    }
    std::fs::write("/tmp/antlr_wrongly_accepted.txt", out)
        .expect("write /tmp/antlr_wrongly_accepted.txt");
}

fn walk_feature_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
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

/// Runs a `having executed:` setup block -- tries it as one `;`-separated
/// batch first (the common case, and what a real Cypher submission would
/// require). The openCypher TCK's own fixture convention often instead
/// writes each statement on its own line with *no* `;` at all
/// (`CREATE (:N)\nCREATE (:N)\n...`, e.g. Remove3's bulk-fixture
/// scenarios) -- MarsDB's own `queries` grammar rule can't safely accept
/// bare adjacency as a statement separator (`match_stmt`, one of
/// `statement`'s alternatives, can itself match zero-width, so any
/// repetition shaped like `(... | statement)*` is provably unbounded --
/// confirmed by pest's own static "cannot fail and will repeat
/// infinitely" check when this was tried at the grammar level), so this
/// is handled here instead: on failure, split into lines and only fall
/// back to running each one as its own statement if *every* line
/// independently parses as one complete, self-contained statement --
/// otherwise a real multi-line single statement (e.g. a pattern list
/// spanning lines) would be silently, wrongly split, so this bails out
/// and reports the original batch error instead of guessing.
fn execute_setup_block(db: &Database, cypher: &str) -> Result<(), marsdb::Error> {
    if db.execute_batch(cypher).is_ok() {
        return Ok(());
    }
    let lines: Vec<&str> = cypher
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() > 1 && lines.iter().all(|l| marsdb_query::parse(l).is_ok()) {
        for line in lines {
            db.execute(line)?;
        }
        return Ok(());
    }
    // Fall through to the original batch error for an honest failure
    // reason, not the fallback's own (misleading, since the fallback
    // never ran) success/failure.
    db.execute_batch(cypher).map(|_| ())
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
            execute_setup_block(&db, stmt)?;
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
            Ok(_) => (
                Outcome::UnexpectedBehavior,
                Some("expected an error, query succeeded".to_string()),
            ),
            Err(_) => (Outcome::Pass, None),
        },
        Expected::Empty => match result {
            Ok(r) if r.rows.is_empty() => (Outcome::Pass, None),
            Ok(r) => (
                Outcome::WrongResult,
                Some(format!("expected no rows, got {} row(s)", r.rows.len())),
            ),
            Err(e) => classify_query_error(e),
        },
        Expected::Rows {
            row_order_matters,
            list_order_matters,
            header,
            rows,
        } => match result {
            Ok(r) => compare_rows(&r, header, rows, *row_order_matters, *list_order_matters),
            Err(e) => classify_query_error(e),
        },
    }
}

fn classify_setup_error(e: marsdb::Error) -> (Outcome, Option<String>) {
    if let marsdb::Error::Query(QueryError::Semantic(msg)) = &e {
        if msg.starts_with("__unsupported__") {
            return (
                Outcome::RunnerUnsupported,
                Some(msg.trim_start_matches("__unsupported__").to_string()),
            );
        }
    }
    (Outcome::ParseRejected, Some(format!("setup failed: {e}")))
}

fn unsupported(reason: String) -> marsdb::Error {
    // Piggybacks on QueryError::Semantic purely as a carrier so this
    // closure can return one Result type -- classify_setup_error() unwraps
    // the sentinel prefix back out. Not a real semantic error.
    marsdb::Error::Query(QueryError::Semantic(format!("__unsupported__{reason}")))
}

fn classify_query_error(e: marsdb::Error) -> (Outcome, Option<String>) {
    match &e {
        // Both "never parsed" and "parsed but structurally rejected"
        // read as the same known-gap signal from the TCK harness's own
        // point of view (see `Outcome::ParseRejected`'s doc comment) --
        // only a `Type` error (a real value turned out the wrong shape)
        // is different enough to fall through to `UnexpectedBehavior`.
        marsdb::Error::Query(QueryError::Syntax(_) | QueryError::Semantic(_)) => {
            (Outcome::ParseRejected, Some(e.to_string()))
        }
        _ => (Outcome::UnexpectedBehavior, Some(e.to_string())),
    }
}

fn convert_params(params: &[(String, String)]) -> Result<HashMap<String, PropertyValue>, String> {
    let mut out = HashMap::new();
    for (name, literal_text) in params {
        let tck_val =
            tck_value::parse_cell(literal_text).map_err(|e| format!("param {name}: {e}"))?;
        let pv = match tck_val {
            TckValue::Null => PropertyValue::Null,
            TckValue::Scalar(TckScalar::Int(i)) => PropertyValue::Int(i),
            TckValue::Scalar(TckScalar::Float(f)) => PropertyValue::Float(f),
            TckValue::Scalar(TckScalar::Str(s)) => PropertyValue::String(s),
            TckValue::Scalar(TckScalar::Bool(b)) => PropertyValue::Bool(b),
            other => {
                return Err(format!(
                    "param {name}: list/node/rel-valued params aren't supported: {other:?}"
                ))
            }
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
            Some(format!(
                "column count mismatch: expected {:?}, got {:?}",
                expected_header, result.columns
            )),
        );
    }
    if result.rows.len() != expected_rows.len() {
        return (
            Outcome::WrongResult,
            Some(format!(
                "row count mismatch: expected {}, got {}",
                expected_rows.len(),
                result.rows.len()
            )),
        );
    }

    let mut expected_parsed: Vec<Vec<TckValue>> = Vec::with_capacity(expected_rows.len());
    for row in expected_rows {
        let mut parsed_row = Vec::with_capacity(row.len());
        for cell in row {
            match tck_value::parse_cell(cell) {
                Ok(v) => parsed_row.push(v),
                Err(e) => {
                    return (
                        Outcome::RunnerUnsupported,
                        Some(format!("couldn't parse expected cell {cell:?}: {e}")),
                    )
                }
            }
        }
        expected_parsed.push(parsed_row);
    }
    let actual_parsed: Vec<Vec<TckValue>> = result
        .rows
        .iter()
        .map(|row| row.iter().map(value_to_tck).collect())
        .collect();

    let row_eq = |a: &[TckValue], b: &[TckValue]| {
        a.iter()
            .zip(b)
            .all(|(x, y)| tck_eq(x, y, list_order_matters))
    };

    let matched = if row_order_matters {
        actual_parsed
            .iter()
            .zip(&expected_parsed)
            .all(|(a, e)| row_eq(a, e))
    } else {
        let mut remaining: Vec<&Vec<TckValue>> = expected_parsed.iter().collect();
        actual_parsed.iter().all(|a| {
            let Some(pos) = remaining.iter().position(|e| row_eq(a, e)) else {
                return false;
            };
            remaining.remove(pos);
            true
        })
    };

    if matched {
        (Outcome::Pass, None)
    } else {
        (
            Outcome::WrongResult,
            Some(format!(
                "expected {expected_rows:?}, got {:?}",
                result
                    .rows
                    .iter()
                    .map(|r| format!("{r:?}"))
                    .collect::<Vec<_>>()
            )),
        )
    }
}

fn report(reports: &[ScenarioReport]) {
    let mut by_category: BTreeMap<&str, BTreeMap<Outcome, usize>> = BTreeMap::new();
    for r in reports {
        *by_category
            .entry(&r.category)
            .or_default()
            .entry(r.outcome)
            .or_default() += 1;
    }

    println!(
        "{:<32} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "category", "total", "pass", "wrong", "unexp", "reject", "unsup"
    );
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
            counts.get(&Outcome::UnexpectedBehavior).unwrap_or(&0),
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
        totals.get(&Outcome::UnexpectedBehavior).unwrap_or(&0),
        totals.get(&Outcome::ParseRejected).unwrap_or(&0),
        totals.get(&Outcome::RunnerUnsupported).unwrap_or(&0),
    );

    let wrong: Vec<&ScenarioReport> = reports
        .iter()
        .filter(|r| r.outcome == Outcome::WrongResult)
        .collect();
    if !wrong.is_empty() {
        println!("\n--- WrongResult scenarios (real bugs, not coverage gaps) ---");
        for r in &wrong {
            println!("[{}] {} :: {}", r.category, r.feature_name, r.name);
            if let Some(d) = &r.detail {
                println!("    {d}");
            }
        }
    }

    let unexpected: Vec<&ScenarioReport> = reports
        .iter()
        .filter(|r| r.outcome == Outcome::UnexpectedBehavior)
        .collect();
    if !unexpected.is_empty() {
        println!("\n--- UnexpectedOutcome scenarios (errored/succeeded when the opposite was expected) ---");
        for r in &unexpected {
            println!("[{}] {} :: {}", r.category, r.feature_name, r.name);
            if let Some(d) = &r.detail {
                println!("    {d}");
            }
        }
    }

    if std::env::var("TCK_DUMP_REJECTS").is_ok() {
        let rejected: Vec<&ScenarioReport> = reports
            .iter()
            .filter(|r| r.outcome == Outcome::ParseRejected)
            .collect();
        println!("\n--- ParseRejected scenarios ({}) ---", rejected.len());
        for r in &rejected {
            println!("[{}] {} :: {}", r.category, r.feature_name, r.name);
            if let Some(d) = &r.detail {
                println!("    {d}");
            }
        }
    }

    if std::env::var("TCK_DUMP_UNSUP").is_ok() {
        let unsup: Vec<&ScenarioReport> = reports
            .iter()
            .filter(|r| r.outcome == Outcome::RunnerUnsupported)
            .collect();
        println!("\n--- RunnerUnsupported scenarios ({}) ---", unsup.len());
        for r in &unsup {
            println!("[{}] {} :: {}", r.category, r.feature_name, r.name);
            if let Some(d) = &r.detail {
                println!("    {d}");
            }
        }
    }
}
