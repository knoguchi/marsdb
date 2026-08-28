//! Purpose-built parser for the openCypher TCK's fixed Gherkin step
//! vocabulary, not a general Cucumber/Gherkin engine.
//!
//! A malformed or unrecognized step becomes an `Err` for that one
//! scenario (reported as `RunnerUnsupported`), not a panic that would
//! take down the rest of the file.

#[derive(Debug, Clone)]
pub enum InitialGraph {
    Empty,
    Any,
    Named(String),
}

/// A `Then` assertion. `AnyError` doesn't capture which error type the
/// TCK expects (`SyntaxError`/`TypeError`/etc.) -- `QueryError` has no
/// such taxonomy to match against.
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
    /// One per `And there exists a procedure ...` step -- see
    /// `ProcedureFixture`'s own docs.
    pub procedures: Vec<ProcedureFixture>,
    pub query: String,
    pub expected: Expected,
    /// Raw `Then a <Kind>Error should be raised...` line, kept alongside
    /// `Expected::AnyError` for diagnostics that need the specific error
    /// kind -- not used by normal scenario running.
    pub expected_error_line: Option<String>,
}

/// `And there exists a procedure NAME(in1 :: TYPE1, ...) :: (out1 ::
/// TYPE1, ...):` plus its own `| in1 | ... | out1 | ... |` mock-result
/// table. The TCK's convention: the header lists every declared input
/// name followed by every declared output name, in declared order.
/// `header`/`rows` are kept as raw cell text; this module only extracts
/// structure, values are parsed by the caller (`procedure::TckProcedureProvider`,
/// via `tck_value::parse_cell`).
#[derive(Debug, Clone)]
pub struct ProcedureFixture {
    pub name: String,
    pub input_names: Vec<String>,
    /// Each input's declared type text (`INTEGER?`, `NUMBER?`, ...), same
    /// order as `input_names` -- passed through to `ProcedureSignature::
    /// input_types` for `Executor`'s coarse argument-type check.
    pub input_types: Vec<String>,
    pub output_names: Vec<String>,
    /// Column names, in table order (not necessarily `input_names ++
    /// output_names`, even though every vendored fixture happens to write
    /// it that way) -- looked up by name, not position.
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn parse_feature(content: &str) -> Vec<Result<Scenario, String>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut cursor = Cursor {
        lines: &lines,
        pos: 0,
    };
    let mut feature_name = String::new();
    let mut background: (Option<InitialGraph>, Vec<String>) = (None, Vec::new());
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
            // A tag line before Scenario: -- not used for filtering, skipped.
            cursor.pos += 1;
            continue;
        }
        if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
            match parse_scenario(&mut cursor, &feature_name, &background) {
                Ok((scenario, Some(examples))) => out.extend(expand_outline(scenario, &examples)),
                Ok((scenario, None)) => out.push(Ok(scenario)),
                Err(e) => out.push(Err(e)),
            }
            continue;
        }
        if trimmed.starts_with("Background:") {
            // Shared setup every Scenario:/Scenario Outline: in this file
            // runs before its own steps. If it fails to parse, fall back
            // to no background rather than failing the whole file --
            // each scenario then fails on its own missing `Given` instead
            // of silently mis-running.
            background = parse_background(&mut cursor).unwrap_or_default();
            continue;
        }
        // Anything else at this level -- skip the line rather than fail
        // the whole file.
        cursor.pos += 1;
    }
    out
}

/// `Given .../And having executed:` steps only -- no vendored
/// `Background:` block uses params/procedures/other step shapes
/// `parse_scenario`'s own loop handles.
fn parse_background(cursor: &mut Cursor) -> Result<(Option<InitialGraph>, Vec<String>), String> {
    cursor.pos += 1; // consume the `Background:` line itself
    let mut initial_graph = None;
    let mut setup_cypher = Vec::new();
    while let Some(raw) = cursor.peek() {
        let line = raw.trim();
        if line.is_empty() {
            cursor.pos += 1;
            continue;
        }
        if line.starts_with("Scenario:")
            || line.starts_with("Scenario Outline:")
            || line.starts_with('@')
            || line.starts_with("Feature:")
        {
            break;
        }
        if let Some(rest) = strip_given(line) {
            initial_graph = Some(parse_initial_graph(rest)?);
            cursor.pos += 1;
        } else if line.starts_with("And having executed:")
            || line.starts_with("And after having executed:")
        {
            cursor.pos += 1;
            setup_cypher.push(cursor.read_block()?);
        } else {
            cursor.pos += 1;
        }
    }
    Ok((initial_graph, setup_cypher))
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
/// cell-level escaping (`\|` -> `|`, `\\` -> `\`, `\n` -> a newline) -- a
/// separate escape layer underneath the Cypher string-literal escaping
/// `tck_value::parse_cell` does on a cell's content afterward.
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
/// returned `Scenario` is a template -- `query`/`setup_cypher`/`expected`
/// still contain literal `<placeholder>` tokens -- paired with the
/// `Examples:` table needed to expand it into real scenarios; see
/// `expand_outline`, called by `parse_feature`.
type Examples = (Vec<String>, Vec<Vec<String>>);

fn parse_scenario(
    cursor: &mut Cursor,
    feature_name: &str,
    background: &(Option<InitialGraph>, Vec<String>),
) -> Result<(Scenario, Option<Examples>), String> {
    let header = cursor
        .peek_trimmed()
        .expect("caller checked this is a Scenario: line");
    let name = header
        .trim_start_matches("Scenario Outline:")
        .trim_start_matches("Scenario:")
        .trim()
        .to_string();
    cursor.pos += 1;

    let mut initial_graph = None;
    let mut setup_cypher = Vec::new();
    let mut params = Vec::new();
    let mut procedures = Vec::new();
    let mut query = None;
    let mut expected = None;
    let mut expected_error_line = None;
    let mut examples: Option<Examples> = None;
    let mut seen_first_then = false;

    while let Some(raw) = cursor.peek() {
        let line = raw.trim();
        if line.is_empty() {
            cursor.pos += 1;
            // A blank line ends the scenario only once its primary
            // assertion is already captured -- blank lines can also
            // appear within a scenario's own step sequence.
            if seen_first_then {
                if let Some(next) = cursor.peek_trimmed() {
                    if next.starts_with("Scenario:")
                        || next.starts_with("Scenario Outline:")
                        || next.starts_with('@')
                        || next.starts_with("Feature:")
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
            continue;
        }
        if line.starts_with("Scenario:")
            || line.starts_with("Scenario Outline:")
            || line.starts_with('@')
            || line.starts_with("Feature:")
        {
            break;
        }

        if let Some(rest) = strip_given(line) {
            initial_graph = Some(parse_initial_graph(rest)?);
            cursor.pos += 1;
        } else if line.starts_with("And having executed:")
            || line.starts_with("And after having executed:")
        {
            cursor.pos += 1;
            setup_cypher.push(cursor.read_block()?);
        } else if line.starts_with("And parameters are:")
            || line.starts_with("And parameter values are:")
        {
            cursor.pos += 1;
            for row in cursor.read_table() {
                if row.len() == 2 {
                    params.push((row[0].clone(), row[1].clone()));
                }
            }
        } else if line.starts_with("And there exists a procedure") {
            procedures.push(parse_procedure_fixture(line, cursor)?);
        } else if line.starts_with("When executing query:")
            || line.starts_with("When executing control query:")
        {
            let is_control = line.contains("control query");
            cursor.pos += 1;
            let block = if let Some(inline) = line.split_once("query:").map(|(_, r)| r.trim()) {
                if inline.is_empty() {
                    cursor.read_block()?
                } else {
                    inline.to_string()
                }
            } else {
                cursor.read_block()?
            };
            if !is_control && query.is_none() {
                query = Some(block);
            }
            // A control query's own When/Then pair isn't captured as this
            // scenario's `query`/`expected` -- side effects aren't
            // asserted in v1 (see crate docs).
        } else if let Some(rest) = line.strip_prefix("Then the result should be") {
            cursor.pos += 1;
            // Always parsed (to consume its table and keep the cursor in
            // sync) but only kept the first time -- a trailing "When
            // executing control query:" has its own second "Then the
            // result should be ..." block, which must not clobber the
            // primary query's already-captured expectation.
            let parsed = parse_result_expectation(rest, cursor);
            if expected.is_none() {
                expected = Some(parsed);
            }
            seen_first_then = true;
        } else if line.starts_with("Then a") && line.contains("should be raised") {
            let raw_line = line.to_string();
            cursor.pos += 1;
            if expected.is_none() {
                expected = Some(Expected::AnyError);
                expected_error_line = Some(raw_line);
            }
            seen_first_then = true;
        } else if line.starts_with("And no side effects")
            || line.starts_with("And the side effects should be:")
        {
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

    // A `Background:` block's own `Given .../And having executed:` run
    // before the scenario's own -- a scenario with no `Given` of its own
    // inherits the background's.
    let initial_graph = initial_graph
        .or_else(|| background.0.clone())
        .ok_or("scenario has no Given ... graph step")?;
    let setup_cypher: Vec<String> = background.1.iter().cloned().chain(setup_cypher).collect();
    let query = query.ok_or("scenario has no primary When executing query: step")?;
    let expected = expected.ok_or("scenario has no Then ... assertion")?;
    let scenario = Scenario {
        feature_name: feature_name.to_string(),
        name,
        initial_graph,
        setup_cypher,
        params,
        procedures,
        query,
        expected,
        expected_error_line,
    };
    Ok((scenario, examples))
}

/// `And there exists a procedure NAME(in1 :: TYPE1, ...) :: (out1 ::
/// TYPE1, ...):` -- `line` is the signature line itself (cursor still
/// positioned there); an optional `| ... |` table immediately follows.
fn parse_procedure_fixture(line: &str, cursor: &mut Cursor) -> Result<ProcedureFixture, String> {
    cursor.pos += 1;
    let rest = line
        .trim_start_matches("And there exists a procedure")
        .trim()
        .trim_end_matches(':')
        .trim();
    // The inputs list can itself contain `::` (once per parameter, `name
    // :: STRING?, in :: INTEGER?`), so split on the first `::` *after*
    // the inputs list's own closing `)`, not the first `::` anywhere in
    // the line. No parameter type nests parens, so a plain `find` for
    // the matching `)` (not a depth-counting scan) is enough.
    let open = rest
        .find('(')
        .ok_or_else(|| format!("procedure signature missing '(': {line:?}"))?;
    let name = rest[..open].trim().to_string();
    let close = rest[open..]
        .find(')')
        .map(|i| open + i)
        .ok_or_else(|| format!("procedure signature missing ')': {line:?}"))?;
    let inputs_text = rest[open + 1..close].trim();
    let after = rest[close + 1..]
        .trim()
        .strip_prefix("::")
        .ok_or_else(|| format!("procedure signature missing '::' after inputs: {line:?}"))?
        .trim();
    let outputs_text = after.trim_start_matches('(').trim_end_matches(')').trim();
    let (input_names, input_types) = parse_procedure_params(inputs_text)?;
    let (output_names, _output_types) = parse_procedure_params(outputs_text)?;
    let mut header = Vec::new();
    let mut rows = Vec::new();
    if cursor.peek_trimmed().is_some_and(|l| l.starts_with('|')) {
        let mut table = cursor.read_table();
        if !table.is_empty() {
            header = table.remove(0);
            rows = table;
        }
    }
    Ok(ProcedureFixture {
        name,
        input_names,
        input_types,
        output_names,
        header,
        rows,
    })
}

/// `name1 :: TYPE1, name2 :: TYPE2, ...` (either the input or the output
/// half of a procedure signature) -- empty for `()`.
fn parse_procedure_params(text: &str) -> Result<(Vec<String>, Vec<String>), String> {
    if text.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut names = Vec::new();
    let mut types = Vec::new();
    for part in text.split(',') {
        let (name, ty) = part.split_once("::").ok_or_else(|| {
            format!("expected 'name :: TYPE' in procedure signature, got {part:?}")
        })?;
        names.push(name.trim().to_string());
        types.push(ty.trim().to_string());
    }
    Ok((names, types))
}

/// Expands a `Scenario Outline:` template into one real `Scenario` per
/// `Examples:` data row -- real Cucumber semantics: every `<col>` token
/// anywhere in the scenario's steps is a literal find-and-replace against
/// that row's value for `col`, not just in the query text.
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
                Expected::Rows {
                    row_order_matters,
                    list_order_matters,
                    header,
                    rows,
                } => Expected::Rows {
                    row_order_matters: *row_order_matters,
                    list_order_matters: *list_order_matters,
                    header: header.iter().map(|s| subst(s)).collect(),
                    rows: rows
                        .iter()
                        .map(|r| r.iter().map(|c| subst(c)).collect())
                        .collect(),
                },
                other => other.clone(),
            };
            Ok(Scenario {
                feature_name: template.feature_name.clone(),
                name: template.name.clone(),
                initial_graph: template.initial_graph.clone(),
                setup_cypher: template.setup_cypher.iter().map(|s| subst(s)).collect(),
                params: template
                    .params
                    .iter()
                    .map(|(k, v)| (k.clone(), subst(v)))
                    .collect(),
                // No vendored outline scenario references a `<col>` token
                // inside its own procedure fixture text, so a plain clone
                // (not `subst`) is correct here.
                procedures: template.procedures.clone(),
                query: subst(&template.query),
                expected,
                expected_error_line: template.expected_error_line.clone(),
            })
        })
        .collect()
}

fn strip_given(line: &str) -> Option<&str> {
    // Always the scenario's first step in every vendored file, never an
    // "And ..." continuation.
    line.strip_prefix("Given ")
}

fn parse_initial_graph(rest: &str) -> Result<InitialGraph, String> {
    if rest == "an empty graph" {
        Ok(InitialGraph::Empty)
    } else if rest == "any graph" {
        Ok(InitialGraph::Any)
    } else if let Some(name) = rest.strip_suffix(" graph") {
        Ok(InitialGraph::Named(
            name.trim_start_matches("the ").to_string(),
        ))
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
    Expected::Rows {
        row_order_matters,
        list_order_matters,
        header,
        rows,
    }
}
