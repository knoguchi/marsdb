//! End-to-end coverage of LDBC SNB Interactive's 7 short-read queries
//! (IS1-IS7, from `ldbc/ldbc_snb_interactive_v1_impls`), run through
//! `$param` substitution against one shared, hand-crafted fixture matching
//! enough of the LDBC schema shape to exercise each query for real —
//! this is the actual target the 8-step feature push
//! (multi-label/params/functions+CASE/ORDER BY/undirected/variable-length/
//! WITH-chaining/OPTIONAL MATCH+VarEq) was building toward, not just
//! shape-mirroring unit tests for the individual mechanics (those live in
//! `smoke.rs`).
//!
//! Not the real LDBC SF0.1 datagen output — that's Java/Spark tooling,
//! explicitly out of scope. Small enough to eyeball, big enough that a
//! forgotten label or wrong direction produces a wrong *count*, not just a
//! panic — see `hop_node_first_label_is_actually_filtered` and
//! `multi_label_create_and_match` in smoke.rs for the exact bugs this
//! discipline caught during development.

use std::collections::{BTreeMap, HashMap};

use marsdb_graph::{GraphStore, NodeId, PropertyValue};
use marsdb_query::{parse, substitute_params, Executor, Literal, QueryResult, Value};

#[allow(dead_code)] // fixture fields kept for clarity/reuse even where a given test doesn't touch them directly
struct Fixture {
    store: GraphStore,
    alice: NodeId,
    bob: NodeId,
    carol: NodeId,
    springfield: NodeId,
    post1: NodeId,
    comment1: NodeId,
    comment2: NodeId,
    forum: NodeId,
}

fn build_fixture() -> Fixture {
    let store = GraphStore::open_memory().unwrap();

    let mut alice_props = BTreeMap::new();
    alice_props.insert("id".into(), PropertyValue::Int(1));
    alice_props.insert("firstName".into(), PropertyValue::String("Alice".into()));
    alice_props.insert("lastName".into(), PropertyValue::String("Anderson".into()));
    let alice = store.create_node(&["Person"], alice_props).unwrap();

    let mut bob_props = BTreeMap::new();
    bob_props.insert("id".into(), PropertyValue::Int(2));
    bob_props.insert("firstName".into(), PropertyValue::String("Bob".into()));
    bob_props.insert("lastName".into(), PropertyValue::String("Brown".into()));
    let bob = store.create_node(&["Person"], bob_props).unwrap();

    let mut carol_props = BTreeMap::new();
    carol_props.insert("id".into(), PropertyValue::Int(3));
    carol_props.insert("firstName".into(), PropertyValue::String("Carol".into()));
    carol_props.insert("lastName".into(), PropertyValue::String("Clark".into()));
    let carol = store.create_node(&["Person"], carol_props).unwrap();

    let mut springfield_props = BTreeMap::new();
    springfield_props.insert("id".into(), PropertyValue::Int(900));
    let springfield = store.create_node(&["City"], springfield_props).unwrap();
    store.create_edge("IS_LOCATED_IN", alice, springfield, BTreeMap::new()).unwrap();

    let mut knows_props = BTreeMap::new();
    knows_props.insert("creationDate".into(), PropertyValue::Int(1_000));
    store.create_edge("KNOWS", alice, bob, knows_props).unwrap();

    let mut post1_props = BTreeMap::new();
    post1_props.insert("id".into(), PropertyValue::Int(100));
    post1_props.insert("creationDate".into(), PropertyValue::Int(2_000));
    post1_props.insert("content".into(), PropertyValue::String("Hello world".into()));
    let post1 = store.create_node(&["Post", "Message"], post1_props).unwrap();
    store.create_edge("HAS_CREATOR", post1, alice, BTreeMap::new()).unwrap();

    let mut comment1_props = BTreeMap::new();
    comment1_props.insert("id".into(), PropertyValue::Int(101));
    comment1_props.insert("creationDate".into(), PropertyValue::Int(2_100));
    // No `content` — exercises IS4's coalesce(content, imageFile).
    comment1_props.insert("imageFile".into(), PropertyValue::String("pic.png".into()));
    let comment1 = store.create_node(&["Comment", "Message"], comment1_props).unwrap();
    store.create_edge("REPLY_OF", comment1, post1, BTreeMap::new()).unwrap();
    store.create_edge("HAS_CREATOR", comment1, bob, BTreeMap::new()).unwrap();

    let mut comment2_props = BTreeMap::new();
    comment2_props.insert("id".into(), PropertyValue::Int(102));
    comment2_props.insert("creationDate".into(), PropertyValue::Int(2_200));
    comment2_props.insert("content".into(), PropertyValue::String("I agree".into()));
    let comment2 = store.create_node(&["Comment", "Message"], comment2_props).unwrap();
    // Replies to comment1, not post1 directly — exercises IS6's REPLY_OF*0..
    // walking more than one hop to reach the root Post.
    store.create_edge("REPLY_OF", comment2, comment1, BTreeMap::new()).unwrap();
    store.create_edge("HAS_CREATOR", comment2, carol, BTreeMap::new()).unwrap();

    let mut forum_props = BTreeMap::new();
    forum_props.insert("id".into(), PropertyValue::Int(500));
    forum_props.insert("title".into(), PropertyValue::String("Tech Forum".into()));
    let forum = store.create_node(&["Forum"], forum_props).unwrap();
    store.create_edge("CONTAINER_OF", forum, post1, BTreeMap::new()).unwrap();
    store.create_edge("HAS_MODERATOR", forum, alice, BTreeMap::new()).unwrap();

    Fixture {
        store,
        alice,
        bob,
        carol,
        springfield,
        post1,
        comment1,
        comment2,
        forum,
    }
}

fn run_with_params(store: &GraphStore, cypher: &str, params: &HashMap<String, PropertyValue>) -> QueryResult {
    let mut stmt = parse(cypher).unwrap_or_else(|e| panic!("parse failed for {cypher:?}: {e}"));
    substitute_params(&mut stmt, params).unwrap_or_else(|e| panic!("param substitution failed: {e}"));
    Executor::new(store)
        .execute(&stmt)
        .unwrap_or_else(|e| panic!("execute failed for {cypher:?}: {e}"))
}

fn int_param(id: i64) -> HashMap<String, PropertyValue> {
    let mut p = HashMap::new();
    p.insert("id".to_string(), PropertyValue::Int(id));
    p
}

fn prop_str(v: &Value) -> String {
    match v {
        Value::Property(PropertyValue::String(s)) => s.clone(),
        other => panic!("expected a string property, got {other:?}"),
    }
}

fn prop_int(v: &Value) -> i64 {
    match v {
        Value::Property(PropertyValue::Int(i)) => *i,
        other => panic!("expected an int property, got {other:?}"),
    }
}

#[test]
fn is1_profile_of_a_person() {
    let f = build_fixture();
    let result = run_with_params(
        &f.store,
        "MATCH (n:Person {id: $id})-[:IS_LOCATED_IN]->(p:City) \
         RETURN n.firstName AS firstName, n.lastName AS lastName, p.id AS cityId",
        &int_param(1),
    );
    assert_eq!(result.rows.len(), 1, "IS1 must return exactly Alice's profile");
    assert_eq!(prop_str(&result.rows[0][0]), "Alice");
    assert_eq!(prop_str(&result.rows[0][1]), "Anderson");
    assert_eq!(prop_int(&result.rows[0][2]), 900);
}

#[test]
fn is3_friends_of_a_person() {
    let f = build_fixture();
    // Real IS3 uses `-[r:KNOWS]-` (undirected) since KNOWS is symmetric —
    // querying from Bob (the target of the stored directed edge) must
    // still find Alice.
    let result = run_with_params(
        &f.store,
        "MATCH (n:Person {id: $id})-[r:KNOWS]-(friend) \
         RETURN friend.id AS friendId, friend.firstName AS firstName \
         ORDER BY toInteger(friendId) ASC",
        &int_param(2),
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(prop_int(&result.rows[0][0]), 1);
    assert_eq!(prop_str(&result.rows[0][1]), "Alice");
}

#[test]
fn is4_content_of_a_message() {
    let f = build_fixture();
    // comment1 has no `content`, only `imageFile` — coalesce must pick it.
    let result = run_with_params(
        &f.store,
        "MATCH (m:Message {id: $id}) RETURN coalesce(m.content, m.imageFile) AS messageContent",
        &int_param(101),
    );
    assert_eq!(result.rows.len(), 1, "IS4 must match :Message even though the node is only ever created as :Comment");
    assert_eq!(prop_str(&result.rows[0][0]), "pic.png");
}

#[test]
fn is5_creator_of_a_message() {
    let f = build_fixture();
    let result = run_with_params(
        &f.store,
        "MATCH (m:Message {id: $id})-[:HAS_CREATOR]->(p:Person) \
         RETURN p.id AS personId, p.firstName AS firstName",
        &int_param(100),
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(prop_int(&result.rows[0][0]), 1);
    assert_eq!(prop_str(&result.rows[0][1]), "Alice");
}

#[test]
fn is6_forum_of_a_message() {
    let f = build_fixture();
    // comment2 -[:REPLY_OF]-> comment1 -[:REPLY_OF]-> post1 -- two hops to
    // reach the root Post, then out to its containing Forum and moderator.
    let result = run_with_params(
        &f.store,
        "MATCH (m:Message {id: $id})-[:REPLY_OF*0..]->(p:Post)<-[:CONTAINER_OF]-(f:Forum)-[:HAS_MODERATOR]->(mod:Person) \
         RETURN f.id AS forumId, f.title AS forumTitle, mod.id AS moderatorId",
        &int_param(102),
    );
    assert_eq!(result.rows.len(), 1, "IS6 must walk both REPLY_OF hops to reach the Post");
    assert_eq!(prop_int(&result.rows[0][0]), 500);
    assert_eq!(prop_str(&result.rows[0][1]), "Tech Forum");
    assert_eq!(prop_int(&result.rows[0][2]), 1);
}

#[test]
fn is2_recent_messages_of_a_person() {
    let f = build_fixture();
    // Real IS2 keys off $personId as the message *author*; our fixture's
    // Bob authored comment1, which replies to post1 -- REPLY_OF*0.. from
    // comment1 reaches post1 in one hop.
    let result = run_with_params(
        &f.store,
        "MATCH (:Person {id: $id})<-[:HAS_CREATOR]-(message) \
         WITH message, message.id AS messageId, message.creationDate AS messageCreationDate \
         ORDER BY messageCreationDate DESC, messageId ASC \
         LIMIT 10 \
         MATCH (message)-[:REPLY_OF*0..]->(post:Post), (post)-[:HAS_CREATOR]->(person) \
         RETURN messageId, coalesce(message.content, message.imageFile) AS messageContent, \
                messageCreationDate, post.id AS postId, person.id AS personId \
         ORDER BY messageCreationDate DESC, messageId ASC",
        &int_param(2),
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(prop_int(&result.rows[0][0]), 101);
    assert_eq!(prop_str(&result.rows[0][1]), "pic.png");
    assert_eq!(prop_int(&result.rows[0][2]), 2_100);
    assert_eq!(prop_int(&result.rows[0][3]), 100);
    assert_eq!(prop_int(&result.rows[0][4]), 1);
}

#[test]
fn is7_replies_of_a_message() {
    let f = build_fixture();
    // comment2 replies to comment1 -- comment1's author (Bob) is the
    // "original message author" from comment2's perspective. Bob and
    // Carol (comment2's author) aren't KNOWS-connected in this fixture,
    // so the flag must be false; add the edge and confirm it flips true.
    let no_knows = run_with_params(
        &f.store,
        "MATCH (m:Message {id: $id})<-[:REPLY_OF]-(c:Comment)-[:HAS_CREATOR]->(p:Person) \
         OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a:Person)-[r:KNOWS]-(p) \
         RETURN c.id AS commentId, p.id AS replyAuthorId, \
                CASE r WHEN null THEN false ELSE true END AS knowsFlag",
        &int_param(101),
    );
    assert_eq!(no_knows.rows.len(), 1);
    assert_eq!(prop_int(&no_knows.rows[0][0]), 102);
    assert_eq!(prop_int(&no_knows.rows[0][1]), 3);
    match &no_knows.rows[0][2] {
        Value::Literal(Literal::Bool(b)) => assert!(!b, "Bob and Carol don't KNOW each other yet"),
        other => panic!("unexpected knowsFlag {other:?}"),
    }

    f.store.create_edge("KNOWS", f.carol, f.bob, BTreeMap::new()).unwrap();
    let with_knows = run_with_params(
        &f.store,
        "MATCH (m:Message {id: $id})<-[:REPLY_OF]-(c:Comment)-[:HAS_CREATOR]->(p:Person) \
         OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a:Person)-[r:KNOWS]-(p) \
         RETURN CASE r WHEN null THEN false ELSE true END AS knowsFlag",
        &int_param(101),
    );
    match &with_knows.rows[0][0] {
        Value::Literal(Literal::Bool(b)) => assert!(*b, "Carol now KNOWS Bob, the flag must flip to true"),
        other => panic!("unexpected knowsFlag {other:?}"),
    }
}
