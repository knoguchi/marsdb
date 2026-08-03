use marsdb::{Literal, Value};
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
            let props: Vec<String> = n.props.iter().map(|(k, v)| format!("{k}: {}", format_property(v))).collect();
            format!("(:{} {{{}}})", n.labels.join(":"), props.join(", "))
        }
        Value::Edge(e) => format!("[:{}]", e.label),
        Value::Property(p) => format_property(p),
        Value::Literal(l) => format_literal(l),
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
