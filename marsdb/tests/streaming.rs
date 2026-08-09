//! `Database::execute_streaming` — the bounded-memory bulk-export path.
//! Contract under test: rows arrive one at a time through the sink,
//! `Break` stops the scan, SKIP/LIMIT stream, non-streamable shapes
//! error loudly instead of silently materializing.

use std::collections::HashMap;
use std::ops::ControlFlow;

use marsdb::{Database, ExecutionOptions, RowSink, Value};

struct Collect {
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
    break_after: Option<usize>,
}

impl Collect {
    fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            break_after: None,
        }
    }
}

impl RowSink for Collect {
    fn columns(&mut self, columns: &[String]) {
        self.columns = columns.to_vec();
    }
    fn row(&mut self, row: Vec<Value>) -> ControlFlow<()> {
        self.rows.push(row);
        if self.break_after.is_some_and(|n| self.rows.len() >= n) {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}

fn seeded(n: usize) -> Database {
    let db = Database::in_memory().unwrap();
    for i in 0..n {
        db.execute_with_params(
            "CREATE (:N {i: $i})",
            &HashMap::from([("i".to_string(), marsdb::PropertyValue::Int(i as i64))]),
        )
        .unwrap();
    }
    db
}

fn stream(db: &Database, cypher: &str, sink: &mut Collect) -> Result<(), marsdb::Error> {
    db.execute_streaming(cypher, &HashMap::new(), &ExecutionOptions::default(), sink)
}

#[test]
fn streams_all_rows_with_columns() {
    let db = seeded(50);
    let mut sink = Collect::new();
    stream(&db, "MATCH (n:N) RETURN n.i AS i", &mut sink).unwrap();
    assert_eq!(sink.columns, vec!["i"]);
    assert_eq!(sink.rows.len(), 50);
}

#[test]
fn where_skip_limit_stream() {
    let db = seeded(50);
    let mut sink = Collect::new();
    stream(
        &db,
        "MATCH (n:N) WHERE n.i >= 10 RETURN n.i AS i SKIP 5 LIMIT 7",
        &mut sink,
    )
    .unwrap();
    assert_eq!(sink.rows.len(), 7);
}

#[test]
fn break_stops_the_scan_cleanly() {
    let db = seeded(50);
    let mut sink = Collect::new();
    sink.break_after = Some(3);
    stream(&db, "MATCH (n:N) RETURN n", &mut sink).unwrap();
    assert_eq!(sink.rows.len(), 3);
}

#[test]
fn non_streamable_shapes_error_instead_of_materializing() {
    let db = seeded(5);
    for cypher in [
        "MATCH (n:N) RETURN n.i ORDER BY n.i",
        "MATCH (n:N) RETURN count(n)",
        "MATCH (n:N) RETURN DISTINCT n.i",
        "MATCH (n:N) WITH n.i AS i RETURN i",
        "OPTIONAL MATCH (n:N) RETURN n",
        "CREATE (:N)",
        "MATCH (n:N) RETURN n.i UNION MATCH (n:N) RETURN n.i",
    ] {
        let mut sink = Collect::new();
        let err = stream(&db, cypher, &mut sink).unwrap_err();
        assert!(
            err.to_string().contains("not streamable") || err.to_string().contains("read-only"),
            "{cypher}: {err}"
        );
        assert!(sink.rows.is_empty(), "{cypher} leaked rows");
    }
}

#[test]
fn max_rows_bound_applies_per_streamed_row() {
    let db = seeded(50);
    let options = ExecutionOptions {
        max_result_rows: Some(10),
        ..Default::default()
    };
    let mut sink = Collect::new();
    let err = db
        .execute_streaming("MATCH (n:N) RETURN n", &HashMap::new(), &options, &mut sink)
        .unwrap_err();
    assert!(err.to_string().contains("resource limit"), "{err}");
    assert!(sink.rows.len() <= 10);
}

#[test]
fn params_and_index_seeks_work() {
    let db = seeded(20);
    db.execute("CREATE INDEX ON :N(i)").unwrap();
    let mut sink = Collect::new();
    db.execute_streaming(
        "MATCH (n:N {i: $target}) RETURN n.i AS i",
        &HashMap::from([("target".to_string(), marsdb::PropertyValue::Int(7))]),
        &ExecutionOptions::default(),
        &mut sink,
    )
    .unwrap();
    assert_eq!(sink.rows.len(), 1);
}

#[test]
fn rejected_inside_an_open_session_transaction() {
    let db = seeded(3);
    db.execute("BEGIN").unwrap();
    let mut sink = Collect::new();
    assert!(stream(&db, "MATCH (n:N) RETURN n", &mut sink).is_err());
    db.execute("ROLLBACK").unwrap();
    stream(&db, "MATCH (n:N) RETURN n", &mut sink).unwrap();
    assert_eq!(sink.rows.len(), 3);
}
