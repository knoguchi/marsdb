//! sqlite-style dot meta commands for the REPL (`.schema`, `.labels`,
//! ...). Thin formatters over the built-in `CALL db.*` introspection
//! procedures — the procedures are the real surface (available to every
//! binding through Cypher); this is CLI sugar over them. Returns the
//! rendered text instead of printing so the REPL owns the printing and
//! tests can assert on output.

use marsdb::{Database, Literal, Value};

const HELP: &str = "\
.schema     labels, relationship types, and indexes in one view
.labels     node labels with node counts
.types      relationship types with edge counts
.props      property keys
.indexes    declared indexes
.help       this list";

/// `None` = not a meta command (caller treats the line as Cypher).
pub fn run(db: &Database, line: &str) -> Option<Result<String, marsdb::Error>> {
    let cmd = line.trim().trim_end_matches(';').trim_end();
    let out = match cmd {
        ".help" => Ok(HELP.to_string()),
        ".labels" => named_counts(db, "CALL db.labels()", "label"),
        ".types" => named_counts(db, "CALL db.relationshipTypes()", "type"),
        ".props" => single_column(db, "CALL db.propertyKeys()"),
        ".indexes" => indexes(db),
        ".schema" => schema(db),
        other if other.starts_with('.') => Ok(format!("unknown command: {other}\n{HELP}")),
        _ => return None,
    };
    Some(out)
}

/// Two-column `(name, count)` procedures, rendered as `name  (N)` lines.
fn named_counts(db: &Database, call: &str, kind: &str) -> Result<String, marsdb::Error> {
    let result = db.execute(call)?;
    if result.rows.is_empty() {
        return Ok(format!("no {kind}s"));
    }
    let lines: Vec<String> = result
        .rows
        .iter()
        .map(|row| format!("{}  ({})", scalar(&row[0]), scalar(&row[1])))
        .collect();
    Ok(lines.join("\n"))
}

fn single_column(db: &Database, call: &str) -> Result<String, marsdb::Error> {
    let result = db.execute(call)?;
    if result.rows.is_empty() {
        return Ok("none".to_string());
    }
    let lines: Vec<String> = result.rows.iter().map(|row| scalar(&row[0])).collect();
    Ok(lines.join("\n"))
}

fn indexes(db: &Database) -> Result<String, marsdb::Error> {
    let result = db.execute("CALL db.indexes()")?;
    if result.rows.is_empty() {
        return Ok("no indexes".to_string());
    }
    let lines: Vec<String> = result
        .rows
        .iter()
        .map(|row| {
            let unique = matches!(&row[2], Value::Literal(Literal::Bool(true)));
            format!(
                ":{}({}){}",
                scalar(&row[0]),
                scalar(&row[1]),
                if unique { " UNIQUE" } else { "" }
            )
        })
        .collect();
    Ok(lines.join("\n"))
}

fn schema(db: &Database) -> Result<String, marsdb::Error> {
    Ok(format!(
        "labels:\n{}\n\nrelationship types:\n{}\n\nindexes:\n{}",
        indent(&named_counts(db, "CALL db.labels()", "label")?),
        indent(&named_counts(db, "CALL db.relationshipTypes()", "type")?),
        indent(&indexes(db)?),
    ))
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The introspection procedures only produce string/int scalars.
fn scalar(value: &Value) -> String {
    match value {
        Value::Literal(Literal::String(s)) => s.clone(),
        Value::Literal(Literal::Int(n)) => n.to_string(),
        Value::Literal(Literal::Bool(b)) => b.to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Database {
        let db = Database::in_memory().unwrap();
        db.execute_batch(
            "CREATE INDEX ON :Person(name) UNIQUE; \
             CREATE (a:Person {name: 'Alice', age: 40}); \
             CREATE (b:Movie {title: 'Heat'}); \
             MATCH (a:Person), (b:Movie) CREATE (a)-[:WATCHED {stars: 5}]->(b)",
        )
        .unwrap();
        db
    }

    #[test]
    fn labels_types_props_indexes() {
        let db = seeded();
        assert_eq!(
            run(&db, ".labels").unwrap().unwrap(),
            "Movie  (1)\nPerson  (1)"
        );
        assert_eq!(run(&db, ".types").unwrap().unwrap(), "WATCHED  (1)");
        assert_eq!(
            run(&db, ".props").unwrap().unwrap(),
            "age\nname\nstars\ntitle"
        );
        assert_eq!(
            run(&db, ".indexes").unwrap().unwrap(),
            ":Person(name) UNIQUE"
        );
    }

    #[test]
    fn schema_combines_all_sections() {
        let out = run(&seeded(), ".schema").unwrap().unwrap();
        assert!(out.contains("labels:\n  Movie  (1)\n  Person  (1)"));
        assert!(out.contains("relationship types:\n  WATCHED  (1)"));
        assert!(out.contains("indexes:\n  :Person(name) UNIQUE"));
    }

    #[test]
    fn unknown_dot_command_shows_help_and_cypher_passes_through() {
        let db = Database::in_memory().unwrap();
        assert!(run(&db, ".bogus").unwrap().unwrap().contains(".schema"));
        assert!(run(&db, "MATCH (n) RETURN n").is_none());
        // Trailing semicolon tolerated -- REPL lines usually end with one.
        assert!(run(&db, ".help;").unwrap().unwrap().contains(".labels"));
    }
}
