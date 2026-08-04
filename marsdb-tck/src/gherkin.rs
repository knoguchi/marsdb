//! A small, purpose-built parser for the openCypher TCK's fixed Gherkin
//! step vocabulary -- not a general Cucumber/Gherkin engine. The real
//! vocabulary (confirmed by grepping every line-start across all 220
//! vendored files, not assumed) is small and fixed; a real Gherkin
//! dependency would be solving a much bigger problem than this needs.
//!
//! A malformed or unrecognized step never panics here -- it becomes an
//! `Err` for that one scenario (reported as `RunnerUnsupported` by the
//! runner), never a crash that would take down the whole file's other
//! scenarios.

#[derive(Debug, Clone)]
pub enum InitialGraph {
    Empty,
    Any,
    Named(String),
}

/// A `Then` assertion. `AnyError` deliberately doesn't capture *which*
/// error type the TCK expects (`SyntaxError`/`TypeError`/etc.) -- see the
/// crate-level docs on why: `QueryError` has no such taxonomy, and
/// matching it isn't in scope.
#[derive(Debug, Clone)]
pub enum Expected {
    Rows {
        row_order_matters: bool,
        list_order_matters: bool,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Empty,
    AnyError,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub feature_name: String,
    pub name: String,
    pub initial_graph: InitialGraph,
    /// One Cypher statement per "having executed" block, run in order
    /// before the query under test.
    pub setup_cypher: Vec<String>,
    /// Raw TCK literal text per parameter, parsed by the caller.
    pub params: Vec<(String, String)>,
    pub query: String,
    pub expected: Expected,
}

pub fn parse_feature(content: &str) -> Vec<Result<Scenario, String>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut cursor = Cursor { lines: &lines, pos: 0 };
    let mut feature_name = String::new();
    let mut out = Vec::new();

    while let Some(line) = cursor.peek() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            cursor.pos += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Feature:") {
            feature_name = rest.trim().to_string();
            cursor.pos += 1;
            continue;
        }
        if trimmed.starts_with('@') {
            // A tag line immediately preceding a Scenario: -- not used for
            // any filtering decision in v1, just skipped.
            cursor.pos += 1;
            continue;
        }
        if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
            match parse_scenario(&mut cursor, &feature_name) {
                Ok((scenario, Some(examples))) => out.extend(expand_outline(scenario, &examples)),
                Ok((scenario, None)) => out.push(Ok(scenario)),
                Err(e) => out.push(Err(e)),
            }
            continue;
        }
        // Anything else at this level (e.g. a Background: block, not used
        // anywhere in the vendored files as of the pinned commit) --
        // skip the line rather than fail the whole file.
        cursor.pos += 1;
    }
    out
}

struct Cursor<'a> {
    lines: &'a [&'a str],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.pos).copied()
    }

    fn peek_trimmed(&self) -> Option<&'a str> {
        self.peek().map(str::trim)
    }

    /// Reads a `"""`-delimited block starting on (or after) the current
    /// line, returning its inner text with the delimiter lines consumed.
    /// Also handles the inline form (`When executing query: MATCH ...`,
    /// content on the same line as the step keyword) via `inline`.
    fn read_block(&mut self) -> Result<String, String> {
        // Skip forward to the opening """ (it's always the very next
        // non-blank line in every vendored file, but be lenient).
        while let Some(l) = self.peek_trimmed() {
            if l.is_empty() {
                self.pos += 1;
                continue;
            }
            break;
        }
        let Some(open) = self.peek_trimmed() else {
            return Err("expected a \"\"\" block, hit EOF".to_string());
        };
        if open != "\"\"\"" {
            return Err(format!("expected \"\"\" to open a block, got {open:?}"));
        }
        self.pos += 1;
        let mut body = Vec::new();
        loop {
            let Some(l) = self.peek() else {
                return Err("unterminated \"\"\" block".to_string());
            };
            if l.trim() == "\"\"\"" {
                self.pos += 1;
                break;
            }
            body.push(l.trim());
            self.pos += 1;
        }
        Ok(body.join("\n"))
    }

    /// Reads a `| a | b |` table starting at the current line, one row
    /// per line, cells trimmed. Stops at the first non-table line.
    fn read_table(&mut self) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        while let Some(l) = self.peek_trimmed() {
            if !l.starts_with('|') {
                break;
            }
            rows.push(split_table_row(l));
            self.pos += 1;
        }
        rows
    }
}

/// Splits one `| a | b |` table row into cells, honoring Cucumber's own
/// cell-level escaping (`\|` -> a literal `|`, `\\` -> a literal `\`, `\n`
/// -> a literal newline) -- a separate escape layer *underneath* the
/// Cypher string-literal escaping `tck_value::parse_cell` does on a
/// cell's content afterward. Without this, a cell like `'a\\bcn5t...'`
/// (Cucumber-escaped `\\` for one real backslash) reached `parse_cell`
/// still double-escaped, corrupting the expected value it parsed to --
/// caught the same way the earlier `\n`/`\t` fix in `tck_value.rs` was:
/// a real query's actual output was correct, but structurally didn't
/// match the (mis-parsed) expected value.
fn split_table_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('|') => current.push('|'),
                Some('\\') => current.push('\\'),
                Some('n') => current.push('\n'),
                Some(other) => {
                    current.push('\\');
                    current.push(other);
                }
                None => current.push('\\'),
            },
            '|' => {
                cells.push(current.trim().to_string());
                current = String::new();
            }
            other => current.push(other),
        }
    }
    cells.push(current.trim().to_string());
    cells
}

/// Parses one `Scenario:`/`Scenario Outline:` block. For an outline, the
/// returned `Scenario` is a *template* -- `query`/`setup_cypher`/`expected`
/// still contain literal `<placeholder>` tokens -- paired with the
/// `Examples:` table (header + data rows) needed to expand it into real
/// scenarios; see `expand_outline`, called by `parse_feature`.
type Examples = (Vec<String>, Vec<Vec<String>>);

fn parse_scenario(cursor: &mut Cursor, feature_name: &str) -> Result<(Scenario, Option<Examples>), String> {
    let header = cursor.peek_trimmed().expect("caller checked this is a Scenario: line");
    let name = header.trim_start_matches("Scenario Outline:").trim_start_matches("Scenario:").trim().to_string();
    cursor.pos += 1;

    let mut initial_graph = None;
    let mut setup_cypher = Vec::new();
    let mut params = Vec::new();
    let mut query = None;
    let mut expected = None;
    let mut examples: Option<Examples> = None;
    let mut seen_first_then = false;

    loop {
        let Some(raw) = cursor.peek() else { break };
        let line = raw.trim();
        if line.is_empty() {
            cursor.pos += 1;
            // A blank line ends the scenario only once we've already
            // captured its primary assertion -- blank lines can appear
            // *within* a scenario's own step sequence too (rare, but
            // tolerated rather than assumed absent).
            if seen_first_then {
                if let Some(next) = cursor.peek_trimmed() {
                    if next.starts_with("Scenario:") || next.starts_with("Scenario Outline:") || next.starts_with('@') || next.starts_with("Feature:") {
                        break;
                    }
                } else {
                    break;
                }
            }
            continue;
        }
        if line.starts_with("Scenario:") || line.starts_with("Scenario Outline:") || line.starts_with('@') || line.starts_with("Feature:") {
            break;
        }

        if let Some(rest) = strip_given(line) {
            initial_graph = Some(parse_initial_graph(rest)?);
            cursor.pos += 1;
        } else if line.starts_with("And having executed:") || line.starts_with("And after having executed:") {
            cursor.pos += 1;
            setup_cypher.push(cursor.read_block()?);
        } else if line.starts_with("And parameters are:") || line.starts_with("And parameter values are:") {
            cursor.pos += 1;
            for row in cursor.read_table() {
                if row.len() == 2 {
                    params.push((row[0].clone(), row[1].clone()));
                }
            }
        } else if line.starts_with("And there exists a procedure") {
            // MarsDB has no CALL/procedure support at all -- the query
            // itself will fail to parse regardless (ParseRejected), so
            // this step's own mock-procedure signature/table is
            // structurally consumed and otherwise irrelevant.
            cursor.pos += 1;
            if cursor.peek_trimmed().is_some_and(|l| l.starts_with('|')) {
                cursor.read_table();
            }
        } else if line.starts_with("When executing query:") || line.starts_with("When executing control query:") {
            let is_control = line.contains("control query");
            cursor.pos += 1;
            let block = if let Some(inline) = line.split_once("query:").map(|(_, r)| r.trim()) {
                if inline.is_empty() { cursor.read_block()? } else { inline.to_string() }
            } else {
                cursor.read_block()?
            };
            if !is_control && query.is_none() {
                query = Some(block);
            }
            // A control query's own When/Then pair (used upstream to
            // observe side effects indirectly) is intentionally not
            // captured as this scenario's `query`/`expected` -- side
            // effects aren't asserted in v1 (see crate docs).
        } else if let Some(rest) = line.strip_prefix("Then the result should be") {
            cursor.pos += 1;
            // Always parsed (to consume its table and keep the cursor in
            // sync), but only *kept* the first time -- a scenario with a
            // trailing "When executing control query:" has a second
            // "Then the result should be ..." block of its own, which
            // must not clobber the primary query's already-captured
            // expectation.
            let parsed = parse_result_expectation(rest, cursor);
            if expected.is_none() {
                expected = Some(parsed);
            }
            seen_first_then = true;
        } else if line.starts_with("Then a") && line.contains("should be raised") {
            cursor.pos += 1;
            if expected.is_none() {
                expected = Some(Expected::AnyError);
            }
            seen_first_then = true;
        } else if line.starts_with("And no side effects") || line.starts_with("And the side effects should be:") {
            cursor.pos += 1;
            if cursor.peek_trimmed().is_some_and(|l| l.starts_with('|')) {
                cursor.read_table();
            }
        } else if line.starts_with("Examples:") {
            cursor.pos += 1;
            let table = cursor.read_table();
            let mut iter = table.into_iter();
            let header = iter.next().unwrap_or_default();
            examples = Some((header, iter.collect()));
        } else {
            // An unrecognized step -- don't loop forever or misparse the
            // rest of the file; skip just this one line.
            cursor.pos += 1;
        }
    }

    let initial_graph = initial_graph.ok_or("scenario has no Given ... graph step")?;
    let query = query.ok_or("scenario has no primary When executing query: step")?;
    let expected = expected.ok_or("scenario has no Then ... assertion")?;
    let scenario = Scenario { feature_name: feature_name.to_string(), name, initial_graph, setup_cypher, params, query, expected };
    Ok((scenario, examples))
}

/// Expands a `Scenario Outline:` template into one real `Scenario` per
/// `Examples:` data row -- real Cucumber semantics: every `<col>` token
/// anywhere in the scenario's steps is a literal find-and-replace against
/// that row's value for `col`, not just in the query text (an expected-
/// result cell or a parameter value could reference one too, even if no
/// vendored file currently does).
fn expand_outline(template: Scenario, examples: &Examples) -> Vec<Result<Scenario, String>> {
    let (header, rows) = examples;
    rows.iter()
        .map(|row| {
            let subst = |s: &str| -> String {
                let mut out = s.to_string();
                for (col, val) in header.iter().zip(row) {
                    out = out.replace(&format!("<{col}>"), val);
                }
                out
            };
            let expected = match &template.expected {
                Expected::Rows { row_order_matters, list_order_matters, header, rows } => Expected::Rows {
                    row_order_matters: *row_order_matters,
                    list_order_matters: *list_order_matters,
                    header: header.iter().map(|s| subst(s)).collect(),
                    rows: rows.iter().map(|r| r.iter().map(|c| subst(c)).collect()).collect(),
                },
                other => other.clone(),
            };
            Ok(Scenario {
                feature_name: template.feature_name.clone(),
                name: template.name.clone(),
                initial_graph: template.initial_graph.clone(),
                setup_cypher: template.setup_cypher.iter().map(|s| subst(s)).collect(),
                params: template.params.iter().map(|(k, v)| (k.clone(), subst(v))).collect(),
                query: subst(&template.query),
                expected,
            })
        })
        .collect()
}

fn strip_given(line: &str) -> Option<&str> {
    // Always the scenario's first step in every vendored file -- never an
    // "And ..." continuation, confirmed by grepping the real vocabulary.
    line.strip_prefix("Given ")
}

fn parse_initial_graph(rest: &str) -> Result<InitialGraph, String> {
    if rest == "an empty graph" {
        Ok(InitialGraph::Empty)
    } else if rest == "any graph" {
        Ok(InitialGraph::Any)
    } else if let Some(name) = rest.strip_suffix(" graph") {
        Ok(InitialGraph::Named(name.trim_start_matches("the ").to_string()))
    } else {
        Err(format!("unrecognized Given .. graph step: {rest:?}"))
    }
}

fn parse_result_expectation(rest: &str, cursor: &mut Cursor) -> Expected {
    let rest = rest.trim();
    if rest.starts_with("empty") {
        return Expected::Empty;
    }
    let list_order_matters = !rest.contains("ignoring element order");
    let row_order_matters = rest.starts_with(", in order");
    let table = cursor.read_table();
    let mut iter = table.into_iter();
    let header = iter.next().unwrap_or_default();
    let rows: Vec<Vec<String>> = iter.collect();
    Expected::Rows { row_order_matters, list_order_matters, header, rows }
}
