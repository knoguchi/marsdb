//! Shared helpers for the smoke_* integration-test binaries -- split
//! from the original single smoke.rs. `dead_code` allowed because each
//! test binary compiles this module separately and uses only a subset.
#![allow(dead_code)]

use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, PathElem, Value};

pub fn run(store: &GraphStore, cypher: &str) -> marsdb_query::QueryResult {
    let stmt = parse(cypher).unwrap_or_else(|e| panic!("parse failed for {cypher:?}: {e}"));
    Executor::new(store)
        .execute(&stmt)
        .unwrap_or_else(|e| panic!("execute failed for {cypher:?}: {e}"))
}

pub fn as_int(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        other => panic!("expected an int, got {other:?}"),
    }
}

pub fn as_float(v: &Value) -> f64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => *f,
        other => panic!("expected a float, got {other:?}"),
    }
}

pub fn int_value(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        other => panic!("expected an int, got {other:?}"),
    }
}

pub fn str_value(v: &Value) -> String {
    match v {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
        other => panic!("expected a string, got {other:?}"),
    }
}

pub fn float_value(v: &Value) -> f64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Float(f)) => *f,
        other => panic!("expected a float, got {other:?}"),
    }
}

pub fn bool_value(v: &Value) -> bool {
    match v {
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected a bool, got {other:?}"),
    }
}

pub fn list_str_values(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items.iter().map(str_value).collect(),
        other => panic!("expected a list, got {other:?}"),
    }
}

pub fn date(year: i32, month: u32, day: u32) -> marsdb_graph::PropertyValue {
    let epoch_day = (chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap()
        - chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
    .num_days() as i32;
    marsdb_graph::PropertyValue::Date(epoch_day)
}

pub fn date_time_epoch(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> i64 {
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, min, sec)
        .unwrap()
        .and_utc()
        .timestamp()
}

pub fn node_labels(v: &Value) -> Vec<String> {
    let Value::Node(node) = v else {
        panic!("expected a node, got {v:?}");
    };
    node.labels.clone()
}

pub fn path_elems(v: &Value) -> &[PathElem] {
    match v {
        Value::Path(elems) => elems,
        other => panic!("expected a path, got {other:?}"),
    }
}

pub fn node_name(elem: &PathElem) -> &str {
    match elem {
        PathElem::Node(n) => match n.props.get("name") {
            Some(marsdb_graph::PropertyValue::String(s)) => s.as_str(),
            other => panic!("expected node to have a string 'name' prop, got {other:?}"),
        },
        other => panic!("expected a node, got {other:?}"),
    }
}

pub fn int(v: &Value) -> i64 {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
        Value::Literal(marsdb_query::Literal::Int(i)) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

pub fn list_ints(v: &Value) -> Vec<i64> {
    match v {
        Value::List(items) => items.iter().map(int).collect(),
        other => panic!("expected List, got {other:?}"),
    }
}

pub fn bool_val(v: &Value) -> bool {
    match v {
        Value::Property(marsdb_graph::PropertyValue::Bool(b)) => *b,
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

/// `marsdb_graph::TzId` <-> `marsdb_query::temporal::TzId` -- two
/// independent, same-shaped types (`temporal.rs` deliberately doesn't
/// depend on `marsdb_graph`), converted at this test-helper boundary.
pub fn to_temporal_tz(zone: &marsdb_graph::TzId) -> marsdb_query::temporal::TzId {
    match zone {
        marsdb_graph::TzId::Offset(o) => marsdb_query::temporal::TzId::Offset(*o),
        marsdb_graph::TzId::Named(name) => marsdb_query::temporal::TzId::Named(name.clone()),
    }
}

/// Renders a `Date`/`Duration`/`String` `Value` as text via the same
/// `marsdb_query::temporal` formatting functions the CLI/TCK output paths
/// use, so these tests check the exact ISO-8601 text a user would see,
/// not just the internal `PropertyValue` representation.
pub fn temporal_str(v: &Value) -> String {
    match v {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
        Value::Property(marsdb_graph::PropertyValue::Date(d)) => {
            marsdb_query::temporal::format_date(*d)
        }
        Value::Property(marsdb_graph::PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => marsdb_query::temporal::format_duration(*months, *days, *seconds, *nanos),
        Value::Property(marsdb_graph::PropertyValue::LocalTime(n)) => {
            marsdb_query::temporal::format_local_time(*n)
        }
        Value::Property(marsdb_graph::PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => marsdb_query::temporal::format_time(*nanos_of_day, *offset_seconds),
        Value::Property(marsdb_graph::PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => marsdb_query::temporal::format_local_date_time(*epoch_seconds, *nanos),
        Value::Property(marsdb_graph::PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => {
            marsdb_query::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone))
        }
        other => panic!("expected a temporal/String value, got {other:?}"),
    }
}

pub fn boolean(v: &Value) -> bool {
    match v {
        Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
        other => panic!("expected Bool, got {other:?}"),
    }
}

// -- Temporal (date/duration) -----------------------------------------
//
// Real shapes pulled directly from the TCK's expressions/temporal
// feature files (Temporal1/2/3/4/5/6/7/8), not synthesized -- see the
// README's "Cypher coverage" section for exactly what's covered and
// what's deliberately out of scope (named time zones like
// 'Europe/Stockholm' -- only a fixed UTC offset is supported).

pub fn plan_lines(result: &marsdb_query::QueryResult) -> Vec<String> {
    result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("expected an EXPLAIN plan line, got {other:?}"),
        })
        .collect()
}

/// Renders a `Value::List`'s scalar elements as a compact string
/// (`Value` has no `PartialEq`, and this reads far better than a chain of
/// nested `matches!`/`if let`s for an 8-row order assertion).
pub fn list_repr(v: &Value) -> String {
    fn scalar_repr(v: &Value) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Literal(marsdb_query::Literal::Int(i)) => i.to_string(),
            Value::Literal(marsdb_query::Literal::String(s)) => format!("'{s}'"),
            other => panic!("unexpected list element {other:?}"),
        }
    }
    match v {
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(scalar_repr).collect::<Vec<_>>().join(", ")
        ),
        other => panic!("expected a list, got {other:?}"),
    }
}
