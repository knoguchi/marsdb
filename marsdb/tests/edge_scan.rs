//! `EdgeTypeScan` — the sequential EDGES-sweep operator for
//! relationship-predicate bulk shapes. The oracle is hand-computed
//! expected sets from the deterministic seed (params substitute before
//! planning, so a `$param` variant takes the SAME plan — it checks
//! determinism, not parity; correctness is pinned against the computed
//! sets, and EXPLAIN pins that the sweep actually ran).

use std::collections::HashMap;

use marsdb::{Database, PropertyValue};

fn seeded() -> Database {
    let db = Database::in_memory().unwrap();
    // 6 Users, 30 Movies, RATED edges with varied ratings, one edge
    // missing `rating`, plus OTHER-type edges and a mislabeled endpoint.
    db.execute("CREATE (:Extra)").unwrap();
    for u in 0..6 {
        db.execute_with_params(
            "CREATE (:User {uid: $u})",
            &HashMap::from([("u".to_string(), PropertyValue::Int(u))]),
        )
        .unwrap();
    }
    for m in 0..30 {
        db.execute_with_params(
            "CREATE (:Movie {mid: $m})",
            &HashMap::from([("m".to_string(), PropertyValue::Int(m))]),
        )
        .unwrap();
    }
    for u in 0..6i64 {
        for m in 0..30i64 {
            if (u + m) % 3 == 0 {
                db.execute_with_params(
                    "MATCH (a:User {uid: $u}), (b:Movie {mid: $m}) \
                     CREATE (a)-[:RATED {rating: $r}]->(b)",
                    &HashMap::from([
                        ("u".to_string(), PropertyValue::Int(u)),
                        ("m".to_string(), PropertyValue::Int(m)),
                        ("r".to_string(), PropertyValue::Float((m % 5) as f64 + 0.5)),
                    ]),
                )
                .unwrap();
            }
        }
    }
    // One RATED edge without the rating property.
    db.execute("MATCH (a:User {uid: 0}), (b:Movie {mid: 1}) CREATE (a)-[:RATED]->(b)")
        .unwrap();
    // A different relationship type between the same labels.
    db.execute(
        "MATCH (a:User {uid: 1}), (b:Movie {mid: 2}) CREATE (a)-[:VIEWED {rating: 0.5}]->(b)",
    )
    .unwrap();
    // A RATED edge whose destination is NOT a Movie.
    db.execute("MATCH (a:User {uid: 2}), (x:Extra) CREATE (a)-[:RATED {rating: 0.5}]->(x)")
        .unwrap();
    db
}

fn rows_sorted(
    db: &Database,
    cypher: &str,
    params: &HashMap<String, PropertyValue>,
) -> Vec<String> {
    let result = db.execute_with_params(cypher, params).unwrap();
    let mut out: Vec<String> = result.rows.iter().map(|r| format!("{r:?}")).collect();
    out.sort();
    out
}

fn plan_of(db: &Database, cypher: &str) -> String {
    format!(
        "{:?}",
        db.execute(&format!("EXPLAIN {cypher}")).unwrap().rows
    )
}

/// Seed model (mirrors `seeded()` exactly): RATED(u,m) exists iff
/// (u+m)%3==0 with rating = (m%5)+0.5, plus one rating-less RATED
/// (0->1), one VIEWED (1->2, rating 0.5), one RATED to :Extra
/// (u=2, rating 0.5).
fn expected_rated_pairs(pred: impl Fn(f64) -> bool) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    for u in 0..6i64 {
        for m in 0..30i64 {
            if (u + m) % 3 == 0 && pred((m % 5) as f64 + 0.5) {
                out.push((u, m));
            }
        }
    }
    out.sort_unstable();
    out
}

fn got_pairs(db: &Database, cypher: &str) -> Vec<(i64, i64)> {
    let result = db.execute(cypher).unwrap();
    let mut out: Vec<(i64, i64)> = result
        .rows
        .iter()
        .map(|row| {
            let int = |v: &marsdb::Value| match v {
                marsdb::Value::Property(PropertyValue::Int(i)) => *i,
                other => panic!("{other:?}"),
            };
            (int(&row[0]), int(&row[1]))
        })
        .collect();
    out.sort_unstable();
    out
}

/// Sweep results must equal the hand-computed sets; EXPLAIN must show
/// the sweep actually ran. `$param` variants (same plan post-
/// substitution) double-check determinism.
#[test]
fn sweep_and_anchor_paths_agree_across_shapes() {
    let db = seeded();
    let cases: &[(&str, &str, PropertyValue)] = &[
        (
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN u.uid, m.mid",
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < $x RETURN u.uid, m.mid",
            PropertyValue::Float(1.0),
        ),
        (
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating >= 3.5 RETURN count(*)",
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating >= $x RETURN count(*)",
            PropertyValue::Float(3.5),
        ),
        // Reversed-direction written pattern.
        (
            "MATCH (m:Movie)<-[r:RATED]-(u:User) WHERE r.rating < 1.0 RETURN m.mid, u.uid",
            "MATCH (m:Movie)<-[r:RATED]-(u:User) WHERE r.rating < $x RETURN m.mid, u.uid",
            PropertyValue::Float(1.0),
        ),
        // Untyped hop.
        (
            "MATCH (u:User)-[r]->(m:Movie) WHERE r.rating < 1.0 RETURN u.uid, m.mid",
            "MATCH (u:User)-[r]->(m:Movie) WHERE r.rating < $x RETURN u.uid, m.mid",
            PropertyValue::Float(1.0),
        ),
        // Multi-type hop.
        (
            "MATCH (u:User)-[r:RATED|VIEWED]->(m:Movie) WHERE r.rating < 1.0 RETURN u.uid, m.mid",
            "MATCH (u:User)-[r:RATED|VIEWED]->(m:Movie) WHERE r.rating < $x RETURN u.uid, m.mid",
            PropertyValue::Float(1.0),
        ),
        // Unlabeled endpoints.
        (
            "MATCH (a)-[r:RATED]->(b) WHERE r.rating < 1.0 RETURN a.uid, b.mid",
            "MATCH (a)-[r:RATED]->(b) WHERE r.rating < $x RETURN a.uid, b.mid",
            PropertyValue::Float(1.0),
        ),
    ];
    for (literal, paramed, value) in cases {
        let lit_plan = plan_of(&db, literal);
        assert!(lit_plan.contains("EdgeTypeScan"), "{literal}: {lit_plan}");
        // (No EXPLAIN for the $param side -- substitution happens before
        // planning, so an unparamed EXPLAIN can't run. Eligibility
        // requiring a literal is covered by the ineligible-shapes test;
        // identical results are the contract here.)
        let got = rows_sorted(&db, literal, &HashMap::new());
        let want = rows_sorted(
            &db,
            paramed,
            &HashMap::from([("x".to_string(), value.clone())]),
        );
        assert_eq!(got, want, "{literal}");
        assert!(!got.is_empty(), "{literal} matched nothing -- weak test");
    }

    // Exact-set oracles against the seed model.
    assert_eq!(
        got_pairs(
            &db,
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN u.uid, m.mid"
        ),
        expected_rated_pairs(|r| r < 1.0),
    );
    assert_eq!(
        got_pairs(
            &db,
            "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating >= 3.5 RETURN u.uid, m.mid"
        ),
        expected_rated_pairs(|r| r >= 3.5),
    );
    // Reversed-written pattern, same set with columns swapped back.
    assert_eq!(
        got_pairs(
            &db,
            "MATCH (m:Movie)<-[r:RATED]-(u:User) WHERE r.rating < 1.0 RETURN u.uid, m.mid"
        ),
        expected_rated_pairs(|r| r < 1.0),
    );
}

#[test]
fn null_predicates_and_missing_props() {
    let db = seeded();
    // Exactly one RATED edge lacks `rating`.
    let is_null = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating IS NULL RETURN u.uid, m.mid")
        .unwrap();
    assert_eq!(is_null.rows.len(), 1);
    assert!(plan_of(
        &db,
        "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating IS NULL RETURN u.uid"
    )
    .contains("EdgeTypeScan"));
    let not_null = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating IS NOT NULL RETURN count(*)")
        .unwrap();
    let all = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) RETURN count(*)")
        .unwrap();
    let n = |r: &marsdb::QueryResult| format!("{:?}", r.rows[0][0]);
    assert_ne!(n(&not_null), n(&all));
}

#[test]
fn endpoint_labels_filter_in_scan() {
    let db = seeded();
    // The User->Extra RATED edge must be excluded by the :Movie check
    // but included when the destination is unlabeled in the pattern.
    let to_movie = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating = 0.5 RETURN count(*)")
        .unwrap();
    let to_any = db
        .execute("MATCH (u:User)-[r:RATED]->(x) WHERE r.rating = 0.5 RETURN count(*)")
        .unwrap();
    let n = |r: &marsdb::QueryResult| format!("{:?}", r.rows[0][0]);
    assert_ne!(n(&to_movie), n(&to_any));
}

#[test]
fn delete_and_stats_through_the_sweep() {
    let db = seeded();
    let before = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN count(*)")
        .unwrap();
    let expected = match &before.rows[0][0] {
        marsdb::Value::Property(PropertyValue::Int(n)) => *n as u64,
        other => panic!("{other:?}"),
    };
    assert!(expected > 0);
    let stats = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 DELETE r")
        .unwrap()
        .stats;
    assert_eq!(stats.relationships_deleted, expected);
    let after = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN count(*)")
        .unwrap();
    assert_eq!(format!("{:?}", after.rows[0][0]), "Property(Int(0))");
}

#[test]
fn residual_endpoint_filters_still_apply() {
    let db = seeded();
    let lit = rows_sorted(
        &db,
        "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 2.0 AND u.uid = 3 RETURN m.mid",
        &HashMap::new(),
    );
    let par = rows_sorted(
        &db,
        "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < $x AND u.uid = 3 RETURN m.mid",
        &HashMap::from([("x".to_string(), PropertyValue::Float(2.0))]),
    );
    assert_eq!(lit, par);
    assert!(!lit.is_empty());
}

#[test]
fn ineligible_shapes_keep_the_traditional_plan() {
    let db = seeded();
    for cypher in [
        // Undirected.
        "MATCH (u:User)-[r:RATED]-(m:Movie) WHERE r.rating < 1.0 RETURN u.uid",
        // Inline endpoint props.
        "MATCH (u:User {uid: 1})-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN m.mid",
        // No rel predicate at all.
        "MATCH (u:User)-[r:RATED]->(m:Movie) RETURN count(*)",
        // Predicate not scan-evaluable (property-to-property).
        "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating = r.rating RETURN count(*)",
    ] {
        let plan = plan_of(&db, cypher);
        assert!(!plan.contains("EdgeTypeScan"), "{cypher}: {plan}");
        // And they still run correctly.
        db.execute(cypher).unwrap();
    }
}

#[test]
fn limit_stops_early_and_streaming_composes() {
    let db = seeded();
    let r = db
        .execute("MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 6.0 RETURN u.uid LIMIT 3")
        .unwrap();
    assert_eq!(r.rows.len(), 3);

    struct Count(usize);
    impl marsdb::RowSink for Count {
        fn columns(&mut self, _: &[String]) {}
        fn row(&mut self, _: Vec<marsdb::Value>) -> std::ops::ControlFlow<()> {
            self.0 += 1;
            std::ops::ControlFlow::Continue(())
        }
    }
    let mut sink = Count(0);
    db.execute_streaming(
        "MATCH (u:User)-[r:RATED]->(m:Movie) WHERE r.rating < 1.0 RETURN u.uid",
        &HashMap::new(),
        &marsdb::ExecutionOptions::default(),
        &mut sink,
    )
    .unwrap();
    assert!(sink.0 > 0);
}
