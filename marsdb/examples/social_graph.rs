//! A small social graph -- the kind of query a relational database makes
//! awkward and a graph database makes direct: variable-length traversal,
//! and connecting two nodes that already exist. Run:
//!
//!     cargo run -p marsdb --example social_graph
//!
//! Writes social_graph.svg -- the full KNOWS network.

use marsdb::Database;

#[path = "common/viz.rs"]
mod viz;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::in_memory()?;

    // A repeated variable name across comma-separated CREATE patterns
    // creates a fresh node each time instead of reusing it (see the
    // README's Cypher-coverage section). A single unbroken chain avoids
    // that: each hop continues from the node the previous hop created.
    db.execute(
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})\
                -[:KNOWS]->(c:Person {name: 'Carol'})\
                -[:KNOWS]->(d:Person {name: 'Dave'})",
    )?;
    db.execute("CREATE (:Person {name: 'Eve'})")?;

    // MATCH...CREATE connects nodes that already exist, which no CREATE
    // pattern alone can do. Alice ends up linked to Dave two ways and to
    // Eve, otherwise unreachable from her.
    for target in ["Dave", "Eve"] {
        db.execute(&format!(
            "MATCH (a:Person {{name: 'Alice'}}) WITH a \
             MATCH (t:Person {{name: '{target}'}}) \
             CREATE (a)-[:KNOWS]->(t)"
        ))?;
    }

    let direct =
        db.execute("MATCH (:Person {name: 'Alice'})-[:KNOWS]->(f:Person) RETURN f.name")?;
    println!("Alice's direct friends: {}", names(&direct));

    // Variable-length traversal, 1-2 hops out. The per-start-node
    // visited-set BFS means Dave (reachable two ways) shows up once;
    // MarsDB has no general RETURN DISTINCT, only inside an aggregate
    // like count(DISTINCT x).
    let network =
        db.execute("MATCH (:Person {name: 'Alice'})-[:KNOWS*1..2]->(f:Person) RETURN f.name")?;
    println!("Alice's network within 2 hops: {}", names(&network));

    // Who has the most outgoing KNOWS edges?
    let popular = db.execute(
        "MATCH (p:Person)-[:KNOWS]->(f:Person) \
         WITH p.name AS name, count(f) AS friends \
         RETURN name, friends ORDER BY friends DESC LIMIT 1",
    )?;
    if let Some(row) = popular.rows.first() {
        println!(
            "Most connected: {} ({} friends)",
            show(&row[0]),
            show(&row[1])
        );
    }

    let nodes = ["Alice", "Bob", "Carol", "Dave", "Eve"];
    let all_edges = db.execute("MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name")?;
    let idx = |v: &marsdb::Value| -> usize {
        let name = show(v);
        nodes.iter().position(|n| *n == name).expect("known node")
    };
    let edges: Vec<(usize, usize)> = all_edges
        .rows
        .iter()
        .map(|row| (idx(&row[0]), idx(&row[1])))
        .collect();
    viz::graph_viz("social_graph.svg", "Alice's social network", &nodes, &edges)?;
    println!("\nwrote social_graph.svg");

    Ok(())
}

fn names(result: &marsdb::QueryResult) -> String {
    result
        .rows
        .iter()
        .map(|row| show(&row[0]))
        .collect::<Vec<_>>()
        .join(", ")
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
