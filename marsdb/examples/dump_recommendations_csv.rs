//! Export the loaded recommendations database as Kùzu-loadable CSVs --
//! node file per label, rel file per type, RFC-4180 quoting (bios contain
//! newlines/quotes/commas). Not shipped -- scratch tool for the
//! cross-engine comparison against marsdb-demo's benchmark.
//!
//! Usage: dump_recommendations_csv <db-path> <out-dir>

use marsdb::{Database, Value};
use marsdb_graph::PropertyValue;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

fn csv_field(v: &Value) -> String {
    let raw = match v {
        Value::Null => String::new(),
        Value::Property(pv) => pv_to_string(pv),
        Value::List(items) => {
            let inner: Vec<String> = items
                .iter()
                .map(|item| match item {
                    Value::Property(pv) => pv_to_string(pv),
                    other => format!("{other:?}"),
                })
                .collect();
            format!("[{}]", inner.join(","))
        }
        other => format!("{other:?}"),
    };
    format!("\"{}\"", raw.replace('"', "\"\""))
}

fn pv_to_string(pv: &PropertyValue) -> String {
    match pv {
        PropertyValue::Null => String::new(),
        PropertyValue::Bool(b) => b.to_string(),
        PropertyValue::Int(i) => i.to_string(),
        PropertyValue::Float(f) => f.to_string(),
        PropertyValue::String(s) => s.clone(),
        PropertyValue::Date(days) => {
            // epoch-day -> ISO date, no chrono needed for the export
            let epoch = *days;
            let mut y = 1970i64;
            let mut d = epoch;
            loop {
                let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
                let len = if leap { 366 } else { 365 };
                if d >= len {
                    d -= len;
                    y += 1;
                } else if d < 0 {
                    y -= 1;
                    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
                    d += if leap { 366 } else { 365 };
                } else {
                    break;
                }
            }
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let months = [
                31,
                if leap { 29 } else { 28 },
                31,
                30,
                31,
                30,
                31,
                31,
                30,
                31,
                30,
                31,
            ];
            let mut m = 0usize;
            while d >= months[m] {
                d -= months[m];
                m += 1;
            }
            format!("{y:04}-{:02}-{:02}", m + 1, d + 1)
        }
        PropertyValue::List(items) => {
            // Kùzu CSV list syntax: [a,b,c]
            let inner: Vec<String> = items.iter().map(pv_to_string).collect();
            format!("[{}]", inner.join(","))
        }
        other => format!("{other:?}"),
    }
}

fn dump_query(
    db: &Database,
    cypher: &str,
    header: &str,
    path: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let result = db.execute(cypher)?;
    let mut w = BufWriter::new(File::create(path)?);
    writeln!(w, "{header}")?;
    for row in &result.rows {
        let fields: Vec<String> = row.iter().map(csv_field).collect();
        writeln!(w, "{}", fields.join(","))?;
    }
    w.flush()?;
    Ok(result.rows.len())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let db = Database::open(&args[1])?;
    let out = args[2].trim_end_matches('/').to_string();
    std::fs::create_dir_all(&out)?;

    for (name, cypher, header) in [
        (
            "movies",
            "MATCH (m:Movie) RETURN m.movieId, m.title, m.year, m.imdbRating, m.released, m.tmdbId, m.imdbId, m.runtime, m.budget, m.revenue, m.plot, m.poster, m.url, m.languages, m.countries",
            "movieId,title,year,imdbRating,released,tmdbId,imdbId,runtime,budget,revenue,plot,poster,url,languages,countries",
        ),
        (
            "people",
            "MATCH (p:Person) RETURN p.tmdbId, p.name, p.born, p.died, p.bornIn, p.imdbId, p.bio, p.poster, p.url",
            "tmdbId,name,born,died,bornIn,imdbId,bio,poster,url",
        ),
        (
            "users",
            "MATCH (u:User) RETURN u.userId, u.name",
            "userId,name",
        ),
        ("genres", "MATCH (g:Genre) RETURN g.name", "name"),
        (
            "rated",
            "MATCH (u:User)-[r:RATED]->(m:Movie) RETURN u.userId, m.movieId, r.rating, r.timestamp",
            "userId,movieId,rating,timestamp",
        ),
        (
            "acted_in",
            "MATCH (p:Person)-[r:ACTED_IN]->(m:Movie) RETURN p.tmdbId, m.movieId, r.role",
            "tmdbId,movieId,role",
        ),
        (
            "directed",
            "MATCH (p:Person)-[:DIRECTED]->(m:Movie) RETURN p.tmdbId, m.movieId",
            "tmdbId,movieId",
        ),
        (
            "in_genre",
            "MATCH (m:Movie)-[:IN_GENRE]->(g:Genre) RETURN m.movieId, g.name",
            "movieId,name",
        ),
    ] {
        let n = dump_query(&db, cypher, header, &format!("{out}/{name}.csv"))?;
        println!("{name}: {n} rows");
    }
    Ok(())
}
