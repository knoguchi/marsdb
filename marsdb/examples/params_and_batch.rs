//! Two things you want once you're past "hello world": parameterized
//! queries instead of building Cypher via string interpolation, and
//! bulk-loading more than one statement without a round trip per
//! statement. Run:
//!
//!     cargo run -p marsdb --example params_and_batch
//!
//! Writes params_and_batch.svg -- all products, priced items above
//! $minPrice highlighted.

use std::collections::HashMap;

use marsdb::{Database, PropertyValue};

#[path = "common/viz.rs"]
mod viz;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::in_memory()?;

    // execute_batch parses the whole `;`-separated batch up front (a
    // syntax error anywhere means nothing runs) but commits one
    // transaction per statement, so a runtime failure partway through
    // leaves earlier statements committed. See Database::execute_batch.
    db.execute_batch(
        "CREATE (:Product {sku: 'A100', name: 'Widget', price: 9.99}); \
         CREATE (:Product {sku: 'B200', name: 'Gadget', price: 19.99}); \
         CREATE (:Product {sku: 'C300', name: 'Gizmo', price: 4.50})",
    )?;

    // Pass external values as a $parameter rather than formatting them
    // into the query string: a parameter is never parsed as Cypher
    // syntax, so it can't change the query's shape (same reason to use
    // a bound parameter in any SQL driver).
    let min_price = 10.0;
    let mut params = HashMap::new();
    params.insert("minPrice".to_string(), PropertyValue::Float(min_price));

    // ORDER BY resolves against RETURN's projected column names, not raw
    // pattern variables: `ORDER BY p.price` would fail here since the
    // projected columns are aliased "name"/"price", not "p".
    let result = db.execute_with_params(
        "MATCH (p:Product) WHERE p.price >= $minPrice \
         RETURN p.name AS name, p.price AS price ORDER BY price",
        &params,
    )?;
    println!("Products at or above ${min_price:.2}:");
    for row in &result.rows {
        println!("  {} -- ${}", show(&row[0]), show(&row[1]));
    }

    let all =
        db.execute("MATCH (p:Product) RETURN p.name AS name, p.price AS price ORDER BY price")?;
    let bars: Vec<(String, f64)> = all
        .rows
        .iter()
        .map(|row| {
            let price = match &row[1] {
                marsdb::Value::Property(PropertyValue::Float(f)) => *f,
                _ => 0.0,
            };
            (show(&row[0]), price)
        })
        .collect();
    let above = plotters::style::RGBColor(85, 168, 104);
    let below = plotters::style::RGBColor(180, 180, 180);
    let colored: Vec<(&str, f64, plotters::style::RGBColor)> = bars
        .iter()
        .map(|(name, price)| {
            (
                name.as_str(),
                *price,
                if *price >= min_price { above } else { below },
            )
        })
        .collect();
    viz::bar_chart(
        "params_and_batch.svg",
        "Product prices (green >= $minPrice)",
        "price ($)",
        &colored,
    )?;
    println!("\nwrote params_and_batch.svg");

    Ok(())
}

fn show(v: &marsdb::Value) -> String {
    match v {
        marsdb::Value::Property(marsdb::PropertyValue::String(s)) => s.clone(),
        marsdb::Value::Property(marsdb::PropertyValue::Int(i)) => i.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Float(f)) => f.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Bool(b)) => b.to_string(),
        marsdb::Value::Property(marsdb::PropertyValue::Null) | marsdb::Value::Null => {
            "null".to_string()
        }
        other => format!("{other:?}"),
    }
}
