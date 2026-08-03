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

    // execute_batch runs a `;`-separated batch, one transaction per
    // statement -- the whole batch is parsed up front (a syntax error
    // anywhere means nothing runs), but a runtime failure partway
    // through stops there, leaving earlier statements committed. See
    // Database::execute_batch's docs for the exact semantics.
    db.execute_batch(
        "CREATE (:Product {sku: 'A100', name: 'Widget', price: 9.99}); \
         CREATE (:Product {sku: 'B200', name: 'Gadget', price: 19.99}); \
         CREATE (:Product {sku: 'C300', name: 'Gizmo', price: 4.50})",
    )?;

    // A value from outside the program -- a request parameter, a config
    // value, whatever. Passing it as a $parameter instead of formatting
    // it into the query string means it's never parsed as Cypher syntax
    // at all, so there's no query-shape it could accidentally change --
    // the same reason you'd use a bound parameter in any SQL driver.
    let min_price = 10.0;
    let mut params = HashMap::new();
    params.insert("minPrice".to_string(), PropertyValue::Float(min_price));

    // ORDER BY only resolves against RETURN's projected column names, not
    // raw pattern variables -- `ORDER BY p.price` would fail with
    // "unbound variable: p" here, since the projected columns are named
    // "name"/"price" (via AS), not "p". Alias whatever you want to sort
    // by.
    let result = db.execute_with_params(
        "MATCH (p:Product) WHERE p.price >= $minPrice \
         RETURN p.name AS name, p.price AS price ORDER BY price",
        &params,
    )?;
    println!("Products at or above ${min_price:.2}:");
    for row in &result.rows {
        println!("  {} -- ${}", show(&row[0]), show(&row[1]));
    }

    let all = db.execute("MATCH (p:Product) RETURN p.name AS name, p.price AS price ORDER BY price")?;
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
        .map(|(name, price)| (name.as_str(), *price, if *price >= min_price { above } else { below }))
        .collect();
    viz::bar_chart("params_and_batch.svg", "Product prices (green >= $minPrice)", "price ($)", &colored)?;
    println!("\nwrote params_and_batch.svg");

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
