//! Indexed range predicates (`WHERE n.year > 2000` with an index on the
//! prop) compile to a bounded `IndexRangeSeek` instead of a label scan +
//! filter. Correctness contract: results identical to the unindexed
//! filter (the storage scan is a superset pre-filter; the residual
//! `Filter` stays the source of truth), including Cypher's cross-type
//! int/float comparison semantics.

use marsdb::Database;

fn seeded(indexed: bool) -> Database {
    let db = Database::in_memory().unwrap();
    if indexed {
        db.execute("CREATE INDEX ON :M(year)").unwrap();
    }
    db.execute_batch(
        "CREATE (:M {t: 'a', year: 1990}); \
         CREATE (:M {t: 'b', year: 2000}); \
         CREATE (:M {t: 'c', year: 2005}); \
         CREATE (:M {t: 'd', year: 2005.5}); \
         CREATE (:M {t: 'e', year: 2010}); \
         CREATE (:M {t: 'f', year: 'not a year'}); \
         CREATE (:M {t: 'g'})",
    )
    .unwrap();
    db
}

fn titles(db: &Database, cypher: &str) -> Vec<String> {
    let result = db.execute(cypher).unwrap();
    let mut out: Vec<String> = result
        .rows
        .iter()
        .map(|row| format!("{:?}", row[0]))
        .collect();
    out.sort();
    out
}

/// Every range shape must return identical rows with and without the
/// index — the seek only changes the plan, never the answer.
#[test]
fn indexed_and_unindexed_results_agree() {
    let with = seeded(true);
    let without = seeded(false);
    for cypher in [
        "MATCH (m:M) WHERE m.year > 2000 RETURN m.t",
        "MATCH (m:M) WHERE m.year >= 2000 RETURN m.t",
        "MATCH (m:M) WHERE m.year < 2005 RETURN m.t",
        "MATCH (m:M) WHERE m.year <= 2005 RETURN m.t",
        "MATCH (m:M) WHERE m.year > 2000 AND m.year < 2010 RETURN m.t",
        "MATCH (m:M) WHERE m.year > 2004.9 RETURN m.t",
        "MATCH (m:M) WHERE m.year > 2005.5 RETURN m.t",
        "MATCH (m:M) WHERE m.t > 'b' RETURN m.t",
        "MATCH (m:M) WHERE m.year > 99999999999999999 RETURN m.t",
    ] {
        assert_eq!(titles(&with, cypher), titles(&without, cypher), "{cypher}");
    }
}

#[test]
fn cross_type_numerics_match_cypher_semantics() {
    let db = seeded(true);
    // > 2000 (int bound) must include the float 2005.5.
    let got = titles(&db, "MATCH (m:M) WHERE m.year > 2000 RETURN m.t");
    assert_eq!(got.len(), 3, "{got:?}"); // c, d, e
                                         // > 2005.4 (float bound) must include int 2010 and float 2005.5.
    let got = titles(&db, "MATCH (m:M) WHERE m.year > 2005.4 RETURN m.t");
    assert_eq!(got.len(), 2, "{got:?}"); // d, e
                                         // Strings and missing props never match a numeric range.
    let got = titles(&db, "MATCH (m:M) WHERE m.year >= 1990 RETURN m.t");
    assert_eq!(got.len(), 5, "{got:?}");
}

#[test]
fn explain_shows_the_range_seek() {
    let db = seeded(true);
    let plan = db
        .execute("EXPLAIN MATCH (m:M) WHERE m.year > 2000 AND m.year <= 2010 RETURN m.t")
        .unwrap();
    let text = format!("{:?}", plan.rows);
    assert!(text.contains("IndexRangeSeek"), "{text}");
    assert!(!text.contains("NodeByLabelScan"), "{text}");

    // Without an index: plain scan + filter, no range node.
    let db = seeded(false);
    let plan = db
        .execute("EXPLAIN MATCH (m:M) WHERE m.year > 2000 RETURN m.t")
        .unwrap();
    let text = format!("{:?}", plan.rows);
    assert!(text.contains("NodeByLabelScan"), "{text}");
    assert!(!text.contains("IndexRangeSeek"), "{text}");
}

/// The keyset-pagination shape: strictly-greater cursor + LIMIT, and it
/// streams (composes with execute_streaming).
#[test]
fn keyset_pagination_pages_through_everything() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE INDEX ON :P(seq)").unwrap();
    for i in 0..25 {
        db.execute_with_params(
            "CREATE (:P {seq: $i})",
            &std::collections::HashMap::from([("i".to_string(), marsdb::PropertyValue::Int(i))]),
        )
        .unwrap();
    }
    let mut seen = Vec::new();
    let mut last: i64 = -1;
    loop {
        let page = db
            .execute_with_params(
                "MATCH (p:P) WHERE p.seq > $last RETURN p.seq AS seq LIMIT 10",
                &std::collections::HashMap::from([(
                    "last".to_string(),
                    marsdb::PropertyValue::Int(last),
                )]),
            )
            .unwrap();
        if page.rows.is_empty() {
            break;
        }
        for row in &page.rows {
            let v = match &row[0] {
                marsdb::Value::Property(marsdb::PropertyValue::Int(i)) => *i,
                other => panic!("{other:?}"),
            };
            seen.push(v);
            last = last.max(v);
        }
    }
    seen.sort_unstable();
    assert_eq!(seen, (0..25).collect::<Vec<i64>>());
}
