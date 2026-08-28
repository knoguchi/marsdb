//! Deterministic LDBC-style social-network workload: generator shared by
//! the `ldbc_style` correctness tests and `benches/ldbc_style_ops.rs`.
//!
//! The schema and statement stream are derived from a third-party
//! benchmark (orneryd/Mimir's `ldbc-style-benchmark.bench.ts`, itself an
//! informal imitation of LDBC SNB) so results stay comparable across
//! engines. This is NOT the official LDBC SNB data generator and nothing
//! here is an official LDBC benchmark result. The original's
//! `Math.random()` KNOWS pairing is replaced by a fixed-seed LCG, so a
//! given scale factor always produces the identical graph.
//!
//! Node counts scale with `sf` (defaults elsewhere: 0.005 for the default
//! test suite, 0.1 for the `--ignored` test and the benchmark): persons
//! 10_000·sf, posts 100_000·sf, comments 50_000·sf, KNOWS ~500_000·sf
//! (capped at 50·persons and by pair exhaustion), LIKES 200_000·sf, plus
//! fixed 10 countries / 20 cities / 10 companies / 10 universities /
//! 20 tags.

use std::collections::{HashMap, HashSet};

use marsdb::{parse, Database, ExecutionOptions, PropertyValue, Statement};

pub const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack", "Kate",
    "Leo", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Rose", "Sam", "Tina", "Uma", "Victor",
    "Wendy", "Xavier", "Yara", "Zack", "Anna", "Ben", "Cara", "Dan",
];
pub const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Wilson",
    "Taylor", "Anderson", "Thomas", "Jackson", "White", "Harris", "Martin", "Thompson", "Moore",
    "Allen", "Young",
];
pub const COUNTRIES: &[&str] = &[
    "USA",
    "UK",
    "Germany",
    "France",
    "Japan",
    "China",
    "Brazil",
    "India",
    "Canada",
    "Australia",
];
pub const CITIES: &[&str] = &[
    "New York",
    "London",
    "Berlin",
    "Paris",
    "Tokyo",
    "Beijing",
    "São Paulo",
    "Mumbai",
    "Toronto",
    "Sydney",
    "Los Angeles",
    "Manchester",
    "Munich",
    "Lyon",
    "Osaka",
    "Shanghai",
    "Rio",
    "Delhi",
    "Vancouver",
    "Melbourne",
];
pub const COMPANIES: &[&str] = &[
    "TechCorp",
    "DataSoft",
    "CloudInc",
    "AILabs",
    "WebDev",
    "MobileTech",
    "SecureNet",
    "GreenEnergy",
    "FinTech",
    "HealthIT",
];
pub const UNIVERSITIES: &[&str] = &[
    "MIT",
    "Stanford",
    "Harvard",
    "Oxford",
    "Cambridge",
    "ETH",
    "Caltech",
    "Princeton",
    "Yale",
    "Berkeley",
];
pub const TAGS: &[&str] = &[
    "tech",
    "science",
    "music",
    "sports",
    "travel",
    "food",
    "art",
    "politics",
    "business",
    "health",
    "AI",
    "blockchain",
    "cloud",
    "mobile",
    "gaming",
    "movies",
    "books",
    "fashion",
    "nature",
    "photography",
];
pub const BROWSERS: &[&str] = &["Chrome", "Firefox", "Safari", "Edge"];

/// Same constants as `rand`'s `Pcg`-family seed mixer, used purely as a
/// fixed-seed 64-bit LCG (top bits used for the modulo). The point is
/// determinism, not statistical quality.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn next_below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

/// Scaled entity counts plus the KNOWS pairing actually generated (the
/// pair set is LCG-determined, so tests take it from here rather than
/// re-deriving it). `allow(dead_code)`: this module is compiled once per
/// target, and the bench target loads the graph without ever reading
/// these fields back.
#[allow(dead_code)]
pub struct Dataset {
    pub persons: usize,
    pub posts: usize,
    pub comments: usize,
    pub likes: usize,
    /// Directed KNOWS pairs in creation order, `(from, to)` person ids.
    pub knows: Vec<(usize, usize)>,
}

fn scaled(base: usize, sf: f64) -> usize {
    ((base as f64 * sf) as usize).max(10)
}

fn pv_int(i: i64) -> PropertyValue {
    PropertyValue::Int(i)
}
fn pv_str(s: impl Into<String>) -> PropertyValue {
    PropertyValue::String(s.into())
}

struct Loader<'db> {
    // One transaction for the entire load: per-statement autocommit is
    // ~4x slower in debug builds (a per-statement redb commit), which
    // matters because the SF 0.005 variant runs in the default suite.
    tx: marsdb::Transaction<'db>,
    opts: ExecutionOptions,
}

impl Loader<'_> {
    fn run(&mut self, stmt: &Statement, params: HashMap<String, PropertyValue>) {
        self.tx
            .execute_prepared_statement(stmt, &params, &self.opts)
            .expect("ldbc_support load statement failed");
    }
}

/// Creates the property indexes and loads the whole scaled dataset into
/// `db` (expected to be freshly opened). Indexes come first so the
/// loader's own MATCH-by-id statements seek instead of scanning.
pub fn load(db: &Database, sf: f64) -> Dataset {
    for stmt in [
        "CREATE INDEX ON :Person(id)",
        "CREATE INDEX ON :Post(id)",
        "CREATE INDEX ON :Comment(id)",
        "CREATE INDEX ON :Tag(id)",
        "CREATE INDEX ON :City(id)",
        "CREATE INDEX ON :Country(id)",
        "CREATE INDEX ON :Company(id)",
        "CREATE INDEX ON :University(id)",
    ] {
        db.execute(stmt).expect("create index");
    }

    let persons = scaled(10_000, sf);
    let posts = scaled(100_000, sf);
    let comments = scaled(50_000, sf);
    let knows_target = scaled(500_000, sf).min(persons * 50);
    let likes = scaled(200_000, sf);

    let mut loader = Loader {
        tx: db.begin_transaction().expect("open load transaction"),
        opts: ExecutionOptions::default(),
    };
    let p = |q: &str| parse(q).expect("ldbc_support load query must parse");

    let create_country = p("CREATE (c:Country {id: $id, name: $name})");
    for (i, name) in COUNTRIES.iter().enumerate() {
        loader.run(
            &create_country,
            HashMap::from([
                ("id".into(), pv_int(i as i64)),
                ("name".into(), pv_str(*name)),
            ]),
        );
    }
    let create_city = p("CREATE (c:City {id: $cityId, name: $cityName}) WITH c MATCH (country:Country {id: $countryId}) CREATE (c)-[:IS_PART_OF]->(country)");
    for (i, name) in CITIES.iter().enumerate() {
        loader.run(
            &create_city,
            HashMap::from([
                ("cityId".into(), pv_int(i as i64)),
                ("cityName".into(), pv_str(*name)),
                ("countryId".into(), pv_int((i % COUNTRIES.len()) as i64)),
            ]),
        );
    }
    let create_company = p("CREATE (c:Company {id: $id, name: $name})");
    for (i, name) in COMPANIES.iter().enumerate() {
        loader.run(
            &create_company,
            HashMap::from([
                ("id".into(), pv_int(i as i64)),
                ("name".into(), pv_str(*name)),
            ]),
        );
    }
    let create_university = p("CREATE (u:University {id: $id, name: $name})");
    for (i, name) in UNIVERSITIES.iter().enumerate() {
        loader.run(
            &create_university,
            HashMap::from([
                ("id".into(), pv_int(i as i64)),
                ("name".into(), pv_str(*name)),
            ]),
        );
    }
    let create_tag = p("CREATE (t:Tag {id: $id, name: $name})");
    for (i, name) in TAGS.iter().enumerate() {
        loader.run(
            &create_tag,
            HashMap::from([
                ("id".into(), pv_int(i as i64)),
                ("name".into(), pv_str(*name)),
            ]),
        );
    }

    let create_person = p("CREATE (p:Person {id: $id, firstName: $firstName, lastName: $lastName, gender: $gender, birthday: $birthday, creationDate: $creationDate, browserUsed: $browser, locationIP: $ip}) WITH p MATCH (city:City {id: $cityId}) CREATE (p)-[:IS_LOCATED_IN]->(city)");
    let create_works = p("MATCH (p:Person {id: $personId}), (c:Company {id: $companyId}) CREATE (p)-[:WORKS_AT {workFrom: $workFrom}]->(c)");
    let create_studies = p("MATCH (p:Person {id: $personId}), (u:University {id: $uniId}) CREATE (p)-[:STUDIES_AT {classYear: $classYear}]->(u)");
    let create_interest = p(
        "MATCH (p:Person {id: $personId}), (t:Tag {id: $tagId}) CREATE (p)-[:HAS_INTEREST]->(t)",
    );
    for i in 0..persons {
        loader.run(
            &create_person,
            HashMap::from([
                ("id".into(), pv_int(i as i64)),
                (
                    "firstName".into(),
                    pv_str(FIRST_NAMES[i % FIRST_NAMES.len()]),
                ),
                (
                    "lastName".into(),
                    pv_str(LAST_NAMES[(i / FIRST_NAMES.len()) % LAST_NAMES.len()]),
                ),
                (
                    "gender".into(),
                    pv_str(if i % 2 == 0 { "male" } else { "female" }),
                ),
                (
                    "birthday".into(),
                    pv_str(format!(
                        "19{}-{:02}-{:02}",
                        70 + (i % 30),
                        (i % 12) + 1,
                        (i % 28) + 1
                    )),
                ),
                (
                    "creationDate".into(),
                    pv_str(format!("202{}-{:02}-01T12:00:00", i % 4, (i % 12) + 1)),
                ),
                ("browser".into(), pv_str(BROWSERS[i % 4])),
                (
                    "ip".into(),
                    pv_str(format!("192.168.{}.{}", i % 256, (i * 7) % 256)),
                ),
                ("cityId".into(), pv_int((i % CITIES.len()) as i64)),
            ]),
        );
        if i % 3 == 0 {
            loader.run(
                &create_works,
                HashMap::from([
                    ("personId".into(), pv_int(i as i64)),
                    ("companyId".into(), pv_int((i % COMPANIES.len()) as i64)),
                    ("workFrom".into(), pv_int((2010 + (i % 14)) as i64)),
                ]),
            );
        }
        if i % 4 == 0 {
            loader.run(
                &create_studies,
                HashMap::from([
                    ("personId".into(), pv_int(i as i64)),
                    ("uniId".into(), pv_int((i % UNIVERSITIES.len()) as i64)),
                    ("classYear".into(), pv_int((2005 + (i % 15)) as i64)),
                ]),
            );
        }
        for j in 0..(1 + (i % 5)) {
            loader.run(
                &create_interest,
                HashMap::from([
                    ("personId".into(), pv_int(i as i64)),
                    ("tagId".into(), pv_int(((i + j) % TAGS.len()) as i64)),
                ]),
            );
        }
    }

    let create_knows = p("MATCH (p1:Person {id: $p1}), (p2:Person {id: $p2}) CREATE (p1)-[:KNOWS {creationDate: $date}]->(p2)");
    let mut rng = Lcg(42);
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut knows = Vec::new();
    let mut attempts = 0usize;
    while knows.len() < knows_target && attempts < knows_target * 2 {
        attempts += 1;
        let a = rng.next_below(persons);
        let b = rng.next_below(persons);
        if a == b || !seen.insert((a.min(b), a.max(b))) {
            continue;
        }
        loader.run(
            &create_knows,
            HashMap::from([
                ("p1".into(), pv_int(a as i64)),
                ("p2".into(), pv_int(b as i64)),
                (
                    "date".into(),
                    pv_str(format!("202{}-{:02}-15", attempts % 4, (attempts % 12) + 1)),
                ),
            ]),
        );
        knows.push((a, b));
    }

    let create_post = p("MATCH (p:Person {id: $creatorId}) CREATE (post:Post {id: $postId, imageFile: $image, creationDate: $date, browserUsed: $browser, locationIP: $ip, content: $content, length: $length}) CREATE (post)-[:HAS_CREATOR]->(p)");
    let create_post_tag =
        p("MATCH (post:Post {id: $postId}), (t:Tag {id: $tagId}) CREATE (post)-[:HAS_TAG]->(t)");
    for i in 0..posts {
        loader.run(
            &create_post,
            HashMap::from([
                ("postId".into(), pv_int(i as i64)),
                ("creatorId".into(), pv_int((i % persons) as i64)),
                (
                    "image".into(),
                    if i % 5 == 0 {
                        pv_str(format!("image{i}.jpg"))
                    } else {
                        PropertyValue::Null
                    },
                ),
                (
                    "date".into(),
                    pv_str(format!(
                        "202{}-{:02}-{:02}",
                        i % 4,
                        (i % 12) + 1,
                        (i % 28) + 1
                    )),
                ),
                ("browser".into(), pv_str(BROWSERS[i % 4])),
                (
                    "ip".into(),
                    pv_str(format!("10.0.{}.{}", i % 256, (i * 3) % 256)),
                ),
                (
                    "content".into(),
                    pv_str(format!("Post content {i} - Lorem ipsum dolor sit amet...")),
                ),
                ("length".into(), pv_int((50 + (i % 200)) as i64)),
            ]),
        );
        for j in 0..(1 + (i % 3)) {
            loader.run(
                &create_post_tag,
                HashMap::from([
                    ("postId".into(), pv_int(i as i64)),
                    ("tagId".into(), pv_int(((i + j) % TAGS.len()) as i64)),
                ]),
            );
        }
    }

    let create_comment = p("MATCH (p:Person {id: $creatorId}), (post:Post {id: $postId}) CREATE (c:Comment {id: $commentId, creationDate: $date, content: $content, length: $length}) CREATE (c)-[:HAS_CREATOR]->(p) CREATE (c)-[:REPLY_OF]->(post)");
    for i in 0..comments {
        loader.run(
            &create_comment,
            HashMap::from([
                ("commentId".into(), pv_int(i as i64)),
                ("creatorId".into(), pv_int(((i * 3) % persons) as i64)),
                ("postId".into(), pv_int((i % posts) as i64)),
                (
                    "date".into(),
                    pv_str(format!(
                        "202{}-{:02}-{:02}",
                        i % 4,
                        (i % 12) + 1,
                        (i % 28) + 1
                    )),
                ),
                (
                    "content".into(),
                    pv_str(format!("Comment {i} - Great post!")),
                ),
                ("length".into(), pv_int((20 + (i % 100)) as i64)),
            ]),
        );
    }

    let create_like = p("MATCH (p:Person {id: $personId}), (post:Post {id: $postId}) CREATE (p)-[:LIKES {creationDate: $date}]->(post)");
    for i in 0..likes {
        loader.run(
            &create_like,
            HashMap::from([
                ("personId".into(), pv_int((i % persons) as i64)),
                ("postId".into(), pv_int(((i * 7) % posts) as i64)),
                (
                    "date".into(),
                    pv_str(format!(
                        "202{}-{:02}-{:02}",
                        i % 4,
                        (i % 12) + 1,
                        (i % 28) + 1
                    )),
                ),
            ]),
        );
    }

    loader.tx.commit().expect("commit load transaction");

    Dataset {
        persons,
        posts,
        comments,
        likes,
        knows,
    }
}

/// The 17 benchmark queries, verbatim from the third-party suite.
pub const BENCH_QUERIES: &[(&str, &str)] = &[
    ("IS1_person_profile", "MATCH (p:Person {id: 1}) RETURN p.firstName, p.lastName, p.birthday, p.locationIP, p.browserUsed, p.gender, p.creationDate"),
    ("IS2_recent_messages", "MATCH (p:Person {id: 1})<-[:HAS_CREATOR]-(m) WHERE m:Post OR m:Comment RETURN m.id, m.content, m.creationDate ORDER BY m.creationDate DESC LIMIT 10"),
    ("IS3_friends", "MATCH (p:Person {id: 1})-[:KNOWS]-(friend:Person) RETURN friend.id, friend.firstName, friend.lastName"),
    ("IS4_message_content", "MATCH (m:Post {id: 100}) RETURN m.content, m.creationDate"),
    ("IS5_message_creator", "MATCH (m:Post {id: 100})-[:HAS_CREATOR]->(p:Person) RETURN p.id, p.firstName, p.lastName"),
    ("IS6_message_tags", "MATCH (m:Post {id: 100})-[:HAS_TAG]->(t:Tag) RETURN t.name"),
    ("IS7_message_replies", "MATCH (m:Post {id: 100})<-[:REPLY_OF]-(c:Comment)-[:HAS_CREATOR]->(p:Person) RETURN c.id, c.content, p.firstName, p.lastName ORDER BY c.creationDate DESC"),
    ("IC1_friends_with_name", "MATCH (p:Person {id: 1})-[:KNOWS*1..3]-(friend:Person) WHERE friend.firstName = 'Alice' RETURN DISTINCT friend.id, friend.lastName LIMIT 20"),
    ("IC2_messages_from_friends", "MATCH (p:Person {id: 1})-[:KNOWS]-(friend:Person)<-[:HAS_CREATOR]-(m) WHERE m:Post OR m:Comment RETURN friend.firstName, m.content, m.creationDate ORDER BY m.creationDate DESC LIMIT 20"),
    ("IC3_friends_in_countries", "MATCH (p:Person {id: 1})-[:KNOWS*1..2]-(friend:Person)-[:IS_LOCATED_IN]->(city:City)-[:IS_PART_OF]->(country:Country) WHERE country.name IN ['USA', 'UK', 'Germany'] RETURN friend.firstName, friend.lastName, country.name, count(*) as cnt ORDER BY cnt DESC LIMIT 20"),
    ("IC4_popular_tags", "MATCH (p:Person {id: 1})-[:KNOWS]-(friend:Person)-[:HAS_INTEREST]->(t:Tag) RETURN t.name, count(*) as popularity ORDER BY popularity DESC LIMIT 10"),
    ("IC5_friends_of_friends", "MATCH (p:Person {id: 1})-[:KNOWS]->(friend:Person)-[:KNOWS]->(fof:Person) WHERE NOT (p)-[:KNOWS]-(fof) AND p <> fof RETURN DISTINCT fof.firstName, fof.lastName LIMIT 20"),
    ("agg_posts_per_person", "MATCH (p:Person)<-[:HAS_CREATOR]-(post:Post) RETURN p.id, count(post) as postCount ORDER BY postCount DESC LIMIT 10"),
    ("agg_avg_friends_per_city", "MATCH (p:Person)-[:IS_LOCATED_IN]->(city:City) OPTIONAL MATCH (p)-[:KNOWS]-(friend) WITH city, p, count(friend) as friendCount RETURN city.name, avg(friendCount) as avgFriends, count(p) as personCount ORDER BY avgFriends DESC"),
    ("agg_tag_cooccurrence", "MATCH (post:Post)-[:HAS_TAG]->(t1:Tag), (post)-[:HAS_TAG]->(t2:Tag) WHERE t1.id < t2.id RETURN t1.name, t2.name, count(*) as coCount ORDER BY coCount DESC LIMIT 10"),
    ("write_create_delete_person", "CREATE (p:Person {id: 999999, firstName: 'Test', lastName: 'User'}) WITH p DELETE p"),
    ("write_create_delete_knows", "MATCH (p1:Person {id: 1}), (p2:Person {id: 2}) CREATE (p1)-[r:TEMP_KNOWS]->(p2) DELETE r"),
];
