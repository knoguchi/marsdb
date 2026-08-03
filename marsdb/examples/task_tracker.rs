//! A minimal task tracker -- the smallest realistic use of MarsDB as an
//! embedded database in your own app: create records, update one,
//! filter, and aggregate. Run:
//!
//!     cargo run -p marsdb --example task_tracker
//!
//! Writes task_tracker.svg -- a bar chart of open vs. done tasks.

use marsdb::Database;

#[path = "common/viz.rs"]
mod viz;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `Database::open("tasks.db")` for a real file instead -- everything
    // else below is identical either way.
    let db = Database::in_memory()?;

    db.execute("CREATE (:Task {title: 'Write the report', done: false})")?;
    db.execute("CREATE (:Task {title: 'Review PR', done: false})")?;
    db.execute("CREATE (:Task {title: 'Fix the bug', done: true})")?;

    db.execute("MATCH (t:Task {title: 'Review PR'}) SET t.done = true")?;

    let open = db.execute("MATCH (t:Task {done: false}) RETURN t.title")?;
    println!("Open tasks:");
    for row in &open.rows {
        println!("  - {}", show(&row[0]));
    }

    // `t.done` (a bare property, non-aggregating) becomes the implicit
    // grouping key alongside the aggregate count(*) -- see the README's
    // Cypher-coverage section on implicit GROUP BY.
    let by_status = db.execute("MATCH (t:Task) RETURN t.done, count(*)")?;
    println!("\nBy status:");
    for row in &by_status.rows {
        println!("  done={}: {}", show(&row[0]), show(&row[1]));
    }

    let count_for = |done: bool| -> f64 {
        by_status
            .rows
            .iter()
            .find(|row| matches!(&row[0], marsdb::Value::Property(marsdb::PropertyValue::Bool(b)) if *b == done))
            .map(|row| match &row[1] {
                marsdb::Value::Property(marsdb::PropertyValue::Int(i)) => *i as f64,
                _ => 0.0,
            })
            .unwrap_or(0.0)
    };
    viz::bar_chart(
        "task_tracker.svg",
        "Tasks by status",
        "count",
        &[
            ("Open", count_for(false), plotters::style::RGBColor(221, 132, 82)),
            ("Done", count_for(true), plotters::style::RGBColor(85, 168, 104)),
        ],
    )?;
    println!("\nwrote task_tracker.svg");

    Ok(())
}

fn show(v: &marsdb::Value) -> String {
    match v {
        marsdb::Value::Property(marsdb::PropertyValue::String(s)) => s.clone(),
        marsdb::Value::Property(marsdb::PropertyValue::Int(i)) => i.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Float(f)) => f.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Bool(b)) => b.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Null) | marsdb::Value::Null => "null".to_string(),
        other => format!("{other:?}"),
    }
}
