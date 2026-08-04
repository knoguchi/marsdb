use marsdb::{Literal, PathElem, Value};
use marsdb_graph::PropertyValue;

pub fn print_table(result: &marsdb::QueryResult) {
    if result.columns.is_empty() {
        return; // CREATE / DELETE / SET produce no rows to show.
    }
    println!("{}", result.columns.join(" | "));
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(format_value).collect();
        println!("{}", cells.join(" | "));
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Node(n) => {
            let props: Vec<String> = n
                .props
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_property(v)))
                .collect();
            let label_part = if n.labels.is_empty() {
                String::new()
            } else {
                format!(":{} ", n.labels.join(":"))
            };
            format!("({label_part}{{{}}})", props.join(", "))
        }
        Value::Edge(e) => format!("[:{}]", e.label),
        Value::Property(p) => format_property(p),
        Value::Literal(l) => format_literal(l),
        Value::List(items) => {
            let cells: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", cells.join(", "))
        }
        Value::Map(m) => {
            let cells: Vec<String> = m
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_value(v)))
                .collect();
            format!("{{{}}}", cells.join(", "))
        }
        Value::Path(elems) => {
            let parts: Vec<String> = elems
                .iter()
                .map(|e| match e {
                    PathElem::Node(n) => format!("(:{})", n.labels.join(":")),
                    PathElem::Edge(e) => format!("-[:{}]->", e.label),
                })
                .collect();
            parts.join("")
        }
        Value::Null => "null".to_string(),
    }
}

fn format_property(p: &PropertyValue) -> String {
    match p {
        PropertyValue::Null => "null".to_string(),
        PropertyValue::Bool(b) => b.to_string(),
        PropertyValue::Int(i) => i.to_string(),
        PropertyValue::Float(f) => f.to_string(),
        PropertyValue::String(s) => s.clone(),
        PropertyValue::Date(d) => marsdb::temporal::format_date(*d),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => marsdb::temporal::format_duration(*months, *days, *seconds, *nanos),
        PropertyValue::LocalTime(nanos_of_day) => {
            marsdb::temporal::format_local_time(*nanos_of_day)
        }
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => marsdb::temporal::format_time(*nanos_of_day, *offset_seconds),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => marsdb::temporal::format_local_date_time(*epoch_seconds, *nanos),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            offset_seconds,
        } => marsdb::temporal::format_date_time(*epoch_seconds, *nanos, *offset_seconds),
    }
}

fn format_literal(l: &Literal) -> String {
    match l {
        Literal::Int(i) => i.to_string(),
        Literal::Float(f) => f.to_string(),
        Literal::String(s) => s.clone(),
        Literal::Bool(b) => b.to_string(),
        Literal::Null => "null".to_string(),
        Literal::Param(name) => format!("${name}"),
    }
}
