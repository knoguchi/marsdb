//! Correctness proof for complex query pipelines on the deterministic
//! LDBC-style social-network workload (see `ldbc_support/mod.rs` for the
//! generator and its third-party provenance).
//!
//! Method: pull the raw graph back out with primitive single-edge scans,
//! recompute each complex query's answer independently in Rust (BFS,
//! grouping, joins), and assert the Cypher pipeline matches. Nothing is
//! a golden value: a generator change reflows into both sides. Covers
//! multi-stage shapes the TCK only exercises at toy scale (OPTIONAL
//! MATCH + WITH-grouped count fed into avg, correlated comma patterns
//! with clause-wide relationship uniqueness, var-length DISTINCT
//! counting).
//!
//! `ldbc_style_workload` (SF 0.005, ~800 nodes / ~5k rels, ~2s debug)
//! runs in the default suite; `ldbc_style_workload_large` (SF 0.1, ~16k
//! nodes / ~115k rels) is `--ignored`:
//!
//! ```text
//! cargo test -p marsdb --test ldbc_style -- --ignored --nocapture
//! ```

mod ldbc_support;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use ldbc_support::{load, Dataset, BENCH_QUERIES};
use marsdb::{Database, PropertyValue, Value};

fn int(v: &Value) -> i64 {
    match v {
        Value::Property(PropertyValue::Int(i)) => *i,
        other => panic!("expected Int, got {other:?}"),
    }
}

fn string(v: &Value) -> String {
    match v {
        Value::Property(PropertyValue::String(s)) => s.clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

fn float(v: &Value) -> f64 {
    match v {
        Value::Property(PropertyValue::Float(f)) => *f,
        Value::Property(PropertyValue::Int(i)) => *i as f64,
        other => panic!("expected a number, got {other:?}"),
    }
}

/// The raw graph, read back through primitive single-hop scans only:
/// the trusted baseline every pipeline assertion below recomputes from.
struct Primitives {
    /// person id -> (firstName, lastName)
    person_names: HashMap<i64, (String, String)>,
    /// person id -> city name
    person_city: HashMap<i64, String>,
    /// directed KNOWS pairs
    knows: Vec<(i64, i64)>,
    /// post id -> creator person id
    post_creator: HashMap<i64, i64>,
    /// comment id -> creator person id
    comment_creator: HashMap<i64, i64>,
    /// post id -> tag ids
    post_tags: HashMap<i64, Vec<i64>>,
}

fn read_primitives(db: &Database) -> Primitives {
    let mut person_names = HashMap::new();
    for row in db
        .execute("MATCH (p:Person) RETURN p.id, p.firstName, p.lastName")
        .unwrap()
        .rows
    {
        person_names.insert(int(&row[0]), (string(&row[1]), string(&row[2])));
    }
    let mut person_city = HashMap::new();
    for row in db
        .execute("MATCH (p:Person)-[:IS_LOCATED_IN]->(c:City) RETURN p.id, c.name")
        .unwrap()
        .rows
    {
        person_city.insert(int(&row[0]), string(&row[1]));
    }
    let knows = db
        .execute("MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.id, b.id")
        .unwrap()
        .rows
        .iter()
        .map(|row| (int(&row[0]), int(&row[1])))
        .collect();
    let mut post_creator = HashMap::new();
    for row in db
        .execute("MATCH (post:Post)-[:HAS_CREATOR]->(p:Person) RETURN post.id, p.id")
        .unwrap()
        .rows
    {
        post_creator.insert(int(&row[0]), int(&row[1]));
    }
    let mut comment_creator = HashMap::new();
    for row in db
        .execute("MATCH (c:Comment)-[:HAS_CREATOR]->(p:Person) RETURN c.id, p.id")
        .unwrap()
        .rows
    {
        comment_creator.insert(int(&row[0]), int(&row[1]));
    }
    let mut post_tags: HashMap<i64, Vec<i64>> = HashMap::new();
    for row in db
        .execute("MATCH (post:Post)-[:HAS_TAG]->(t:Tag) RETURN post.id, t.id")
        .unwrap()
        .rows
    {
        post_tags
            .entry(int(&row[0]))
            .or_default()
            .push(int(&row[1]));
    }
    Primitives {
        person_names,
        person_city,
        knows,
        post_creator,
        comment_creator,
        post_tags,
    }
}

fn undirected_adjacency(knows: &[(i64, i64)]) -> HashMap<i64, Vec<i64>> {
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for &(a, b) in knows {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    adj
}

fn verify(db: &Database, ds: &Dataset) {
    let g = read_primitives(db);

    // The stored graph is exactly what the generator emitted: a storage
    // round-trip check, and the license for everything below to treat
    // the primitives as ground truth.
    assert_eq!(g.person_names.len(), ds.persons);
    assert_eq!(g.post_creator.len(), ds.posts);
    assert_eq!(g.comment_creator.len(), ds.comments);
    let likes = db
        .execute("MATCH ()-[r:LIKES]->() RETURN count(r) as c")
        .unwrap();
    assert_eq!(int(&likes.rows[0][0]), ds.likes as i64);
    let stored: BTreeSet<(i64, i64)> = g.knows.iter().copied().collect();
    let generated: BTreeSet<(i64, i64)> = ds
        .knows
        .iter()
        .map(|&(a, b)| (a as i64, b as i64))
        .collect();
    assert_eq!(stored, generated, "stored KNOWS set != generated KNOWS set");

    // Every benchmark query at least executes.
    for (name, q) in BENCH_QUERIES {
        db.execute(q)
            .unwrap_or_else(|e| panic!("benchmark query {name} failed: {e}"));
    }

    // --- posts per person: full grouped count vs the post->creator map.
    let mut expected_posts: HashMap<i64, i64> = HashMap::new();
    for creator in g.post_creator.values() {
        *expected_posts.entry(*creator).or_default() += 1;
    }
    let result = db
        .execute("MATCH (p:Person)<-[:HAS_CREATOR]-(post:Post) RETURN p.id, count(post)")
        .unwrap();
    assert_eq!(result.rows.len(), expected_posts.len());
    for row in &result.rows {
        assert_eq!(int(&row[1]), expected_posts[&int(&row[0])]);
    }

    // --- average friends per city: OPTIONAL MATCH + WITH-grouped count
    // fed into avg()/count(). Wrong pre-grouping produces null averages
    // or counts inflated by the row expansion.
    let adj = undirected_adjacency(&g.knows);
    let mut city_degrees: HashMap<&str, Vec<i64>> = HashMap::new();
    for (person, city) in &g.person_city {
        let degree = adj.get(person).map_or(0, |n| n.len() as i64);
        city_degrees.entry(city).or_default().push(degree);
    }
    let result = db
        .execute(
            "MATCH (p:Person)-[:IS_LOCATED_IN]->(city:City) \
             OPTIONAL MATCH (p)-[:KNOWS]-(friend) \
             WITH city, p, count(friend) as friendCount \
             RETURN city.name, avg(friendCount) as avgFriends, count(p) as personCount \
             ORDER BY avgFriends DESC",
        )
        .unwrap();
    assert_eq!(result.rows.len(), city_degrees.len());
    for row in &result.rows {
        let city = string(&row[0]);
        let degrees = &city_degrees[city.as_str()];
        let expected_avg = degrees.iter().sum::<i64>() as f64 / degrees.len() as f64;
        assert!(
            (float(&row[1]) - expected_avg).abs() < 1e-9,
            "avgFriends for {city}: got {}, expected {expected_avg}",
            float(&row[1])
        );
        assert_eq!(int(&row[2]), degrees.len() as i64, "personCount for {city}");
    }

    // --- tag co-occurrence: correlated comma-separated MATCH parts.
    // Expected pair counts from the post->tags map; the full (un-LIMITed)
    // grouped result must match exactly.
    let mut expected_pairs: BTreeMap<(i64, i64), i64> = BTreeMap::new();
    let mut expected_ordered_pairs = 0i64;
    for tags in g.post_tags.values() {
        for &a in tags {
            for &b in tags {
                if a < b {
                    *expected_pairs.entry((a, b)).or_default() += 1;
                }
                if a != b {
                    expected_ordered_pairs += 1;
                }
            }
        }
    }
    let result = db
        .execute(
            "MATCH (post:Post)-[:HAS_TAG]->(t1:Tag), (post)-[:HAS_TAG]->(t2:Tag) \
             WHERE t1.id < t2.id RETURN t1.id, t2.id, count(*)",
        )
        .unwrap();
    let got_pairs: BTreeMap<(i64, i64), i64> = result
        .rows
        .iter()
        .map(|row| ((int(&row[0]), int(&row[1])), int(&row[2])))
        .collect();
    assert_eq!(got_pairs, expected_pairs);

    // Clause-wide relationship uniqueness, unmasked: with no WHERE filter
    // the match count must equal the ordered distinct-tag pairs exactly.
    // A t1 = t2 self-pair (both hops binding the same HAS_TAG edge) would
    // inflate it.
    let result = db
        .execute(
            "MATCH (post:Post)-[:HAS_TAG]->(t1:Tag), (post)-[:HAS_TAG]->(t2:Tag) \
             RETURN count(*) as c",
        )
        .unwrap();
    assert_eq!(int(&result.rows[0][0]), expected_ordered_pairs);

    // --- IC1: DISTINCT entities over var-length traversal. A node is
    // reachable by some edge-distinct walk of length 1..=3 iff its BFS
    // distance is <= 3 (a shortest path never repeats an edge), so the
    // expected set is plain BFS. Person 1 itself can never be an Alice
    // (FIRST_NAMES[1 % 30] = "Bob"), so the start-node-in-a-triangle
    // subtlety can't affect the filtered count.
    let mut dist: HashMap<i64, u32> = HashMap::from([(1, 0)]);
    let mut queue = VecDeque::from([1i64]);
    while let Some(n) = queue.pop_front() {
        let d = dist[&n];
        if d == 3 {
            continue;
        }
        for &m in adj.get(&n).map_or(&Vec::new(), |v| v) {
            dist.entry(m).or_insert_with(|| {
                queue.push_back(m);
                d + 1
            });
        }
    }
    let expected_alices = dist
        .iter()
        .filter(|&(id, &d)| d >= 1 && g.person_names[id].0 == "Alice")
        .count() as i64;
    let result = db
        .execute(
            "MATCH (p:Person {id: 1})-[:KNOWS*1..3]-(friend:Person) \
             WHERE friend.firstName = 'Alice' RETURN count(DISTINCT friend) as c",
        )
        .unwrap();
    assert_eq!(int(&result.rows[0][0]), expected_alices);

    // --- IC2: friends' message count (2-hop join + label-predicate OR).
    let empty = Vec::new();
    let friends: HashSet<i64> = adj.get(&1).unwrap_or(&empty).iter().copied().collect();
    let expected_messages = g
        .post_creator
        .values()
        .chain(g.comment_creator.values())
        .filter(|creator| friends.contains(creator))
        .count() as i64;
    let result = db
        .execute(
            "MATCH (p:Person {id: 1})-[:KNOWS]-(friend:Person)<-[:HAS_CREATOR]-(m) \
             WHERE m:Post OR m:Comment RETURN count(*) as c",
        )
        .unwrap();
    assert_eq!(int(&result.rows[0][0]), expected_messages);

    // --- IC5: negated pattern predicate over a directed 2-hop. Expected
    // from the directed pair list; DISTINCT is on the (firstName,
    // lastName) projection, which collides across persons by design.
    let mut out_edges: HashMap<i64, Vec<i64>> = HashMap::new();
    for &(a, b) in &g.knows {
        out_edges.entry(a).or_default().push(b);
    }
    let undirected: HashSet<(i64, i64)> = g
        .knows
        .iter()
        .flat_map(|&(a, b)| [(a, b), (b, a)])
        .collect();
    let mut expected_fof_names: BTreeSet<(String, String)> = BTreeSet::new();
    for &friend in out_edges.get(&1).map_or(&Vec::new(), |v| v) {
        for &fof in out_edges.get(&friend).map_or(&Vec::new(), |v| v) {
            if fof != 1 && !undirected.contains(&(1, fof)) {
                let (first, last) = g.person_names[&fof].clone();
                expected_fof_names.insert((first, last));
            }
        }
    }
    let result = db
        .execute(
            "MATCH (p:Person {id: 1})-[:KNOWS]->(friend:Person)-[:KNOWS]->(fof:Person) \
             WHERE NOT (p)-[:KNOWS]-(fof) AND p <> fof \
             RETURN DISTINCT fof.firstName, fof.lastName",
        )
        .unwrap();
    let got_fof_names: BTreeSet<(String, String)> = result
        .rows
        .iter()
        .map(|row| (string(&row[0]), string(&row[1])))
        .collect();
    assert_eq!(
        result.rows.len(),
        got_fof_names.len(),
        "DISTINCT rows must be distinct"
    );
    assert_eq!(got_fof_names, expected_fof_names);
}

#[test]
fn ldbc_style_workload() {
    let db = Database::in_memory().unwrap();
    let ds = load(&db, 0.005);
    verify(&db, &ds);
}

#[test]
#[ignore = "~35s debug: SF 0.1 (~16k nodes / ~115k rels) version of ldbc_style_workload"]
fn ldbc_style_workload_large() {
    let db = Database::in_memory().unwrap();
    let ds = load(&db, 0.1);
    verify(&db, &ds);
}
