//! Smoke tests: MATCH patterns: traversal, var-length, paths, OPTIONAL MATCH, EXISTS -- split from the original smoke.rs.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, PathElem, Value};

#[test]
fn traversal_with_label_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:BLOCKS]->(c:Person {name: 'Carol'})",
    );

    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person) RETURN b.name",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Bob"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn exists_full_subquery_form_runs_an_arbitrary_correlated_statement() {
    // TCK ExistentialSubquery2 [1]/[2], ExistentialSubquery3 [1]:
    // `exists { MATCH ... RETURN ... }` -- unlike the simple pattern-only
    // form, this can carry its own aggregation/WHERE, and nest another
    // `exists {}` inside it. Only `a` has any outgoing edge at all in this
    // graph, so every assertion below narrows down to exactly that one row.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A {prop: 1})-[:R]->(:B {prop: 1}), (a)-[:R]->(:C {prop: 2}), \
         (a)-[:R]->(:D {prop: 3})",
    );

    let result = run(
        &store,
        "MATCH (n) WHERE exists { MATCH (n)-->() RETURN true } RETURN n.prop",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);

    // Nested exists {} inside a full-form exists {} -- for n = a, m = the
    // {prop: 1} node satisfies both the outer edge and the prop equality.
    let result = run(
        &store,
        "MATCH (n) WHERE exists { \
             MATCH (m) WHERE exists { (n)-[]->(m) WHERE n.prop = m.prop } \
             RETURN true \
         } RETURN n.prop",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);

    // Full form with its own aggregation -- separate graph so `a`'s
    // {prop: 1} neighbor (`b`) also gets one further outgoing edge, giving
    // `a` exactly 3 connections and `b` only 1.
    let store2 = GraphStore::open_memory().unwrap();
    run(
        &store2,
        "CREATE (a:A {prop: 1})-[:R]->(b:B {prop: 1}), (a)-[:R]->(:C {prop: 2}), \
         (a)-[:R]->(d:D {prop: 3}), (b)-[:R]->(d)",
    );
    let result = run(
        &store2,
        "MATCH (n) WHERE exists { \
             MATCH (n)-->(m) \
             WITH n, count(*) AS numConnections \
             WHERE numConnections = 3 \
             RETURN true \
         } RETURN n.prop",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 1);
}

#[test]
fn exists_full_subquery_form_rejects_a_mutating_clause() {
    // TCK ExistentialSubquery2 [3]: real Cypher's `InvalidClauseComposition`
    // -- an updating clause inside `exists {}` is a compile-time error,
    // checked regardless of whether any row would ever reach it.
    let store = GraphStore::open_memory().unwrap();
    let stmt =
        parse("MATCH (n) WHERE exists { MATCH (n)-->(m) SET m.prop = 'fail' } RETURN n").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_err());
}

/// A bare `RETURN <expr>` with no `MATCH`/`UNWIND`/`MERGE` at all is real
/// Cypher (`match_stmt`'s `clause*`, not `clause+`) -- needs no graph
/// access, just the one synthetic empty row `execute_match` already seeds
/// `current_rows` with by default.
#[test]
fn bare_return_needs_no_match_clause() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN 1 + 2 * 3 AS x");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 7),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn pattern_comprehension_introduces_new_node_and_rel_vars() {
    // TCK expressions/pattern Pattern2 [4]/[5]: a pattern comprehension
    // can introduce brand-new node/relationship variables, unlike a
    // pattern predicate.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a {name: 'a'}), (b {name: 'val'}), (c)");
    run(
        &store,
        "MATCH (a), (b) WHERE a.name = 'a' AND b.name = 'val' CREATE (a)-[:T]->(b)",
    );
    run(
        &store,
        "MATCH (b {name: 'val'}), (c) WHERE c.name IS NULL CREATE (b)-[:T]->(c)",
    );

    let result = run(&store, "MATCH (n) RETURN [(n)-[:T]->(b) | b.name] AS list");
    let mut lists: Vec<Vec<Option<String>>> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::List(items) => items
                .iter()
                .map(|v| match v {
                    Value::Property(marsdb_graph::PropertyValue::String(s)) => Some(s.clone()),
                    Value::Null => None,
                    other => panic!("unexpected value {other:?}"),
                })
                .collect(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    lists.sort();
    assert_eq!(
        lists,
        vec![vec![], vec![None], vec![Some("val".to_string())]]
    );
}

#[test]
fn variable_length_hop_cannot_reuse_an_earlier_fixed_hops_edge() {
    // TCK clauses/match Match5 [27]: real Cypher's edge-isomorphism rule
    // (no relationship repeated within one MATCH pattern) applies across a
    // whole pattern, not just within one variable-length hop's own BFS --
    // a var-length hop mustn't walk back over an edge an earlier fixed hop
    // in the *same* pattern already used.
    //
    // A -[:R]-> B <-[:R]- C (both edges point into B). `(a:A)-[:R]->(b)`
    // must use the A-B edge; a subsequent `<-[:R*1]->` from `b` can then
    // only reach `C` via the B-C edge -- walking back to `A` would replay
    // the already-used A-B edge, and (with only one real path to `C`)
    // reusing it is the *only* way to get a second row.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'A'})-[:R]->(:B {name: 'B'})");
    run(&store, "MATCH (b:B) CREATE (:C {name: 'C'})-[:R]->(b)");
    let result = run(&store, "MATCH (a:A)-[:R]->(b)<-[:R*1]->(c) RETURN c.name");
    let names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["C".to_string()]);
}

#[test]
fn undirected_pattern_matches_either_direction() {
    let store = GraphStore::open_memory().unwrap();
    // a->b (created via Right direction), so from b's perspective it's an
    // incoming edge — an undirected MATCH from b must still find a.
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );

    let from_a = run(
        &store,
        "MATCH (n:Person {name: 'Alice'})-[:KNOWS]-(friend) RETURN friend.name",
    );
    assert_eq!(from_a.rows.len(), 1);

    let from_b = run(
        &store,
        "MATCH (n:Person {name: 'Bob'})-[:KNOWS]-(friend) RETURN friend.name",
    );
    assert_eq!(from_b.rows.len(), 1);
    match &from_b.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Alice"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn undirected_pattern_dedupes_self_loop() {
    use std::collections::BTreeMap;

    // A true self-loop needs the same node bound as both src and dst, which
    // Cypher CREATE can't express in v1 (each pattern position always
    // creates a fresh node) — construct it directly via GraphStore instead.
    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store
        .create_edge("KNOWS", alice, alice, BTreeMap::new())
        .unwrap();

    let result = run(&store, "MATCH (n:Person)-[:KNOWS]-(friend) RETURN friend");
    assert_eq!(
        result.rows.len(),
        1,
        "a self-loop edge must be returned once via undirected dedup-by-edge_id, not twice"
    );
}

#[test]
fn hop_node_first_label_is_actually_filtered() {
    // Regression test: the planner used to unconditionally skip the FIRST
    // label when filtering a node reached via Expand/VarExpand (only
    // correct for the pattern's start node, where NodeByLabelScan already
    // handles the first label) -- so `(a)-[:R]->(b:Post)` would match ANY
    // labeled node at the far end of the hop, not just :Post ones.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Root {name: 'r'})-[:R]->(b:Post {name: 'post'})",
    );
    run(
        &store,
        "CREATE (a2:Root {name: 'r2'})-[:R]->(c:Comment {name: 'comment'})",
    );

    let result = run(&store, "MATCH (a:Root)-[:R]->(b:Post) RETURN b.name");
    assert_eq!(
        result.rows.len(),
        1,
        "hop node's label filter must exclude the :Comment target"
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "post"),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn variable_length_pattern_walks_reply_chain_to_root() {
    use std::collections::BTreeMap;

    // Mirrors IS6's shape: MATCH (m:Message {id})-[:REPLY_OF*0..]->(p:Post) ...
    // Chain c2 -[:REPLY_OF]-> c1 -[:REPLY_OF]-> p. Cypher CREATE always
    // makes fresh nodes per pattern position (no MATCH+CREATE combo in
    // v1), so build this directly via GraphStore to get real shared node
    // identity across the chain.
    let store = GraphStore::open_memory().unwrap();
    let mut props_p = BTreeMap::new();
    props_p.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let p = store.create_node(&["Post", "Message"], props_p).unwrap();
    let mut props_c1 = BTreeMap::new();
    props_c1.insert("id".to_string(), marsdb_graph::PropertyValue::Int(2));
    let c1 = store
        .create_node(&["Comment", "Message"], props_c1)
        .unwrap();
    let mut props_c2 = BTreeMap::new();
    props_c2.insert("id".to_string(), marsdb_graph::PropertyValue::Int(3));
    let c2 = store
        .create_node(&["Comment", "Message"], props_c2)
        .unwrap();
    store
        .create_edge("REPLY_OF", c1, p, BTreeMap::new())
        .unwrap();
    store
        .create_edge("REPLY_OF", c2, c1, BTreeMap::new())
        .unwrap();

    let result = run(
        &store,
        "MATCH (m:Message {id: 3})-[:REPLY_OF*0..]->(p:Post) RETURN p.id",
    );
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 1),
        other => panic!("unexpected value {other:?}"),
    }

    // min_hops = 0 also includes the start node itself if it happens to
    // match the target label — not exercised by IS6 (a Comment never
    // has :Post too) but worth confirming: starting FROM the post with
    // *0.. must return the post itself at hop 0.
    let from_post = run(
        &store,
        "MATCH (m:Message {id: 1})-[:REPLY_OF*0..]->(p:Post) RETURN p.id",
    );
    assert_eq!(from_post.rows.len(), 1);
}

#[test]
fn variable_length_bounded_range_respects_max_hops() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    for i in 0..5 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        ids.push(store.create_node(&["Item"], props).unwrap());
    }
    for i in 0..4 {
        store
            .create_edge("NEXT", ids[i], ids[i + 1], BTreeMap::new())
            .unwrap();
    }

    // From idx=0, *1..2 should reach idx=1 and idx=2 only (not 3, not 4;
    // not 0 itself since min_hops=1).
    let result = run(
        &store,
        "MATCH (n:Item {idx: 0})-[:NEXT*1..2]->(m:Item) RETURN m.idx",
    );
    let mut reached: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    reached.sort();
    assert_eq!(reached, vec![1, 2]);
}

/// A bare `[*]` (no explicit bounds) defaults to `min_hops = 1`, not 0 --
/// the destination is never the start node itself, same as `[*1..]`
/// already correctly behaved. A real bug found via the TCK: `parse_
/// rel_range` defaulted the *omitted* min (both the fully bare `*` case
/// and the `*..M` case with an empty min before `..`) to 0, incorrectly
/// including the zero-hop "reached myself" row. TCK's Match4 [2].
#[test]
fn variable_length_bare_star_defaults_to_min_hops_one() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    for i in 0..4 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        ids.push(store.create_node(&["Item"], props).unwrap());
    }
    for i in 0..3 {
        store
            .create_edge("NEXT", ids[i], ids[i + 1], BTreeMap::new())
            .unwrap();
    }
    for query in [
        "MATCH (n:Item {idx: 0})-[:NEXT*]->(m:Item) RETURN m.idx",
        "MATCH (n:Item {idx: 0})-[:NEXT*..3]->(m:Item) RETURN m.idx",
    ] {
        let result = run(&store, query);
        let mut reached: Vec<i64> = result
            .rows
            .iter()
            .map(|row| match &row[0] {
                Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
                other => panic!("unexpected value {other:?}"),
            })
            .collect();
        reached.sort();
        assert_eq!(reached, vec![1, 2, 3], "query: {query}");
    }

    // `*0..` (explicit zero lower bound) still legitimately includes the
    // start node itself -- only the *omitted*-min cases default to 1.
    let result = run(
        &store,
        "MATCH (n:Item {idx: 0})-[:NEXT*0..2]->(m:Item) RETURN m.idx",
    );
    let mut reached: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    reached.sort();
    assert_eq!(reached, vec![0, 1, 2]);
}

#[test]
fn variable_length_does_not_reuse_relationship_in_same_path() {
    use std::collections::BTreeMap;

    // An undirected traversal may cross the same edge in either direction,
    // but relationship uniqueness forbids using it to walk a -> b -> a.
    let store = GraphStore::open_memory().unwrap();
    let a = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    let b = store.create_node(&["Item"], BTreeMap::new()).unwrap();
    store.create_edge("LINK", a, b, BTreeMap::new()).unwrap();

    let result = run(&store, "MATCH (a:Item)-[:LINK*2]-(b:Item) RETURN b");
    assert!(result.rows.is_empty());
}

/// Real Cypher allows chained plain `MATCH` clauses with no `WITH`
/// between them (an implicit join on any shared variable, e.g. TCK's
/// Match5 `[1]`: `MATCH (a:A) MATCH (a)-[:LIKES*]->(c) RETURN c.name`) --
/// an earlier version of this parser wrongly required `WITH` there,
/// based on a mistaken assumption about real Cypher's own rule (only
/// OPTIONAL MATCH/UNWIND were exempted, when in fact plain MATCH needs no
/// exemption at all). The executor's `carried_vars` threading already
/// handled this correctly regardless -- the parser-level check was the
/// only thing blocking it.
#[test]
fn multiple_match_without_with_is_allowed() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'a'})-[:LIKES]->(:B {name: 'b'})");
    run(&store, "CREATE (:A {name: 'a2'})");

    let result = run(
        &store,
        "MATCH (a:A) MATCH (a)-[:LIKES]->(b) RETURN a.name, b.name",
    );
    assert_eq!(result.rows.len(), 1);
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Property(marsdb_graph::PropertyValue::String(a)),
            Value::Property(marsdb_graph::PropertyValue::String(b)),
        ) => {
            assert_eq!(a, "a");
            assert_eq!(b, "b");
        }
        other => panic!("unexpected value {other:?}"),
    }
}

/// Two paths are equal iff they visit the same nodes/relationships in the
/// same order -- `value_eq` had no `Value::Path` arm at all before this,
/// so any two paths were unconditionally unequal via `=` (fell through to
/// the catch-all `_ => false`). TCK's Comparison1 [14]: a self-loop
/// traversed forward vs backward is the same path (same single node,
/// same single relationship) either way.
#[test]
fn path_equality_compares_nodes_and_relationships_not_always_false() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A)-[:LOOP]->(n)");

    let result = run(
        &store,
        "MATCH p1 = (:A)-->() MATCH p2 = (:A)<--() RETURN p1 = p2",
    );
    assert!(bool_val(&result.rows[0][0]));

    run(&store, "CREATE (:B)-[:X]->(:C)");
    let result = run(
        &store,
        "MATCH p1 = (:A)-->() MATCH p2 = (:B)-->(:C) RETURN p1 = p2",
    );
    assert!(!bool_val(&result.rows[0][0]));
}

/// `MATCH (a:A), (b:B)` -- a genuine disjoint cross join, not a
/// continuation (`b` doesn't continue from `a`'s pattern). Real Cypher's
/// own implicit-join shape (TCK's Merge6/Merge7), previously rejected
/// outright since `group_into_linear_patterns` used to require every
/// comma-separated pattern to continue the previous one.
#[test]
fn comma_separated_match_patterns_cross_join() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A), (:A), (:B)");

    let result = run(&store, "MATCH (a:A), (b:B) RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 2);
}

#[test]
fn optional_match_with_var_eq_mirrors_is7_shape() {
    use std::collections::BTreeMap;

    // Mirrors IS7: MATCH (m)<-[:REPLY_OF]-(c)-[:HAS_CREATOR]->(p)
    //              OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a)-[r:KNOWS]-(p)
    //              RETURN ... CASE r WHEN null THEN false ELSE true END
    //
    // p is bound by the first MATCH (comment author) then reappears as the
    // endpoint of the OPTIONAL pattern -- must mean "KNOWS THIS p", not
    // "KNOWS anyone" (Expr::VarEq). c1's author knows the original
    // message's author; c2's author doesn't -- so the two rows must get
    // different CASE results, not both true/both false.
    let store = GraphStore::open_memory().unwrap();

    let mut m_props = BTreeMap::new();
    m_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(1));
    let m = store.create_node(&["Post", "Message"], m_props).unwrap();
    let original_author = store.create_node(&["Person"], BTreeMap::new()).unwrap();
    store
        .create_edge("HAS_CREATOR", m, original_author, BTreeMap::new())
        .unwrap();

    let mut p1_props = BTreeMap::new();
    p1_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("KnowsAuthor".into()),
    );
    let p1 = store.create_node(&["Person"], p1_props).unwrap();
    let mut c1_props = BTreeMap::new();
    c1_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(10));
    let c1 = store
        .create_node(&["Comment", "Message"], c1_props)
        .unwrap();
    store
        .create_edge("REPLY_OF", c1, m, BTreeMap::new())
        .unwrap();
    store
        .create_edge("HAS_CREATOR", c1, p1, BTreeMap::new())
        .unwrap();
    store
        .create_edge("KNOWS", p1, original_author, BTreeMap::new())
        .unwrap();

    let mut p2_props = BTreeMap::new();
    p2_props.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("StrangerAuthor".into()),
    );
    let p2 = store.create_node(&["Person"], p2_props).unwrap();
    let mut c2_props = BTreeMap::new();
    c2_props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(20));
    let c2 = store
        .create_node(&["Comment", "Message"], c2_props)
        .unwrap();
    store
        .create_edge("REPLY_OF", c2, m, BTreeMap::new())
        .unwrap();
    store
        .create_edge("HAS_CREATOR", c2, p2, BTreeMap::new())
        .unwrap();
    // No KNOWS edge between p2 and original_author.

    let result = run(
        &store,
        "MATCH (m:Message {id: 1})<-[:REPLY_OF]-(c:Comment)-[:HAS_CREATOR]->(p:Person) \
         OPTIONAL MATCH (m)-[:HAS_CREATOR]->(a:Person)-[r:KNOWS]-(p) \
         RETURN c.id AS commentId, p.name AS replyAuthorName, \
                CASE r WHEN null THEN false ELSE true END AS knowsFlag \
         ORDER BY commentId ASC",
    );

    assert_eq!(result.rows.len(), 2);

    let extract = |row: &Vec<Value>| -> (i64, String, bool) {
        let comment_id = match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected commentId {other:?}"),
        };
        let name = match &row[1] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected replyAuthorName {other:?}"),
        };
        let knows = match &row[2] {
            Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
            other => panic!("unexpected knowsFlag {other:?}"),
        };
        (comment_id, name, knows)
    };

    assert_eq!(
        extract(&result.rows[0]),
        (10, "KnowsAuthor".to_string(), true)
    );
    assert_eq!(
        extract(&result.rows[1]),
        (20, "StrangerAuthor".to_string(), false)
    );

    let _ = (m, c1, c2, p1, p2, original_author);
}

#[test]
fn optional_match_null_pads_when_nothing_matches() {
    use std::collections::BTreeMap;

    // No KNOWS edge exists at all -- the OPTIONAL MATCH must still return
    // one row per outer row (not zero), with `a`/`r` null-padded.
    let store = GraphStore::open_memory().unwrap();
    let alice = store.create_node(&["Person"], BTreeMap::new()).unwrap();

    let result = run(
        &store,
        "MATCH (p:Person) \
         OPTIONAL MATCH (p)-[r:KNOWS]-(friend) \
         RETURN CASE r WHEN null THEN false ELSE true END AS knowsFlag",
    );
    assert_eq!(
        result.rows.len(),
        1,
        "the outer MATCH row must survive even with zero optional matches"
    );
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Bool(b)) => assert!(!b),
        other => panic!("unexpected value {other:?}"),
    }
    let _ = alice;
}

#[test]
fn optional_match_without_with_shares_scope() {
    // MATCH ... OPTIONAL MATCH ... (no WITH between them) must be valid --
    // unlike two plain MATCHes, which require a WITH separator.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item {name: 'a'})");
    let result = run(
        &store,
        "MATCH (n:Item) OPTIONAL MATCH (n)-[:X]->(m) RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
}

/// `VariableAlreadyBound` is a compile-time/structural error, not a
/// data-dependent one -- it must fire even when the preceding `MATCH`
/// produces *zero* rows, since the check is about variable scope, not
/// runtime data. A real bug found and fixed: the check used to live only
/// in the per-row runtime path (`materialize_create`/`merge_one_row`),
/// which a zero-row MATCH skips entirely, silently no-op'ing instead of
/// erroring. Now duplicated at compile time in `semantic.rs`. TCK's
/// Create1 [13]/[14], Merge1 [15], Merge5 [26] ("any graph" -- MarsDB's
/// own harness exercises the empty-graph case).
#[test]
fn already_bound_rejection_fires_even_on_a_zero_row_match() {
    let store = GraphStore::open_memory().unwrap();
    for q in [
        "MATCH (a) CREATE (a)",
        "MATCH (a) CREATE (a {name: 'foo'}) RETURN a",
        "MATCH (a) MERGE (a)",
        "MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)",
    ] {
        let stmt = parse(q).unwrap();
        let result = Executor::new(&store).execute(&stmt);
        let err = match result {
            Ok(r) => panic!(
                "expected an already-bound error for {q:?}, got Ok: {:?}",
                r.rows
            ),
            Err(e) => e,
        };
        assert!(
            err.to_string().to_lowercase().contains("already bound"),
            "expected an already-bound error for {q:?}, got: {err}"
        );
    }
}

#[test]
fn with_chaining_disjoint_second_match_cross_joins_carried_var() {
    // `b`'s pattern doesn't chain from `a` at all -- before the scan/seed
    // cross-join fix, this silently dropped `a` instead of producing the
    // 2x2 cross join real Cypher semantics require here.
    let store = GraphStore::open_memory().unwrap();
    for name in ["Alice", "Bob"] {
        run(&store, &format!("CREATE (:Left {{name: '{name}'}})"));
    }
    for name in ["X", "Y"] {
        run(&store, &format!("CREATE (:Right {{name: '{name}'}})"));
    }

    let result = run(
        &store,
        "MATCH (a:Left) WITH a MATCH (b:Right) RETURN a.name AS leftName, b.name AS rightName \
         ORDER BY leftName, rightName",
    );
    assert_eq!(
        result.rows.len(),
        4,
        "2 Left x 2 Right must cross-join to 4 rows, not drop `a`"
    );
    let pairs: Vec<(String, String)> = result
        .rows
        .iter()
        .map(|r| (str_value(&r[0]), str_value(&r[1])))
        .collect();
    assert_eq!(
        pairs,
        vec![
            ("Alice".to_string(), "X".to_string()),
            ("Alice".to_string(), "Y".to_string()),
            ("Bob".to_string(), "X".to_string()),
            ("Bob".to_string(), "Y".to_string()),
        ]
    );
}

#[test]
fn optional_match_disjoint_pattern_does_not_panic() {
    // The OPTIONAL pattern doesn't chain from the outer `a` either -- same
    // root cause as the cross-join test above, but through
    // eval_optional_part's __seed_idx tagging instead of a plain MATCH.
    // Before the fix, scan() silently dropped that tag and
    // eval_optional_part's `unreachable!` fired.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Left {name: 'Alice'})");
    run(&store, "CREATE (:Right {name: 'X'})");

    let result = run(
        &store,
        "MATCH (a:Left) OPTIONAL MATCH (c:Right) RETURN a.name, c.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "X");
}

/// A `MATCH`/`MERGE` pattern's inline `{key: value}` can be any
/// expression, not just a literal -- a bound variable (TCK's Merge1 [8])
/// compiles to `Expr::GeneralCompare` (a generic post-scan filter,
/// evaluated per-row) instead of the index-seek-eligible
/// `Expr::Compare(_, _, Literal)` shape a real literal still gets.
#[test]
fn pattern_property_accepts_a_bound_variable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({var: 42}), ({var: 'not42'})");

    let result = run(&store, "WITH 42 AS x MATCH (n {var: x}) RETURN n.var");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 42);
}

/// Seed enumeration's direct-predicate route (leaf = simple
/// `var.prop <op> literal` filters over a scan, e.g. matrix's `title
/// CONTAINS`) must match the generic pipeline — including a predicate on
/// a property some nodes lack (absent -> filtered out, not an error).
#[test]
fn fast_path_contains_leaf_matches_generic() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (m1:Movie {title: 'The Matrix'}), (m2:Movie {title: 'The Matrix Reloaded'}), \
         (m3:Movie {title: 'Speed'}), (m4:Movie), (u:User {name: 'u'})",
    );
    for title in ["The Matrix", "The Matrix Reloaded", "Speed"] {
        run(
            &store,
            &format!(
                "MATCH (u:User {{name:'u'}}), (m:Movie {{title:'{title}'}}) \
                 CREATE (u)-[:RATED]->(m)"
            ),
        );
    }
    let fast = run(
        &store,
        "MATCH (m:Movie)<-[:RATED]-(u:User) WHERE m.title CONTAINS 'Matrix' \
         WITH m, count(*) AS reviews RETURN m.title, reviews ORDER BY reviews DESC LIMIT 5",
    );
    let generic = run(
        &store,
        "MATCH (m:Movie)<-[:RATED]-(u:User) WHERE m.title CONTAINS 'Matrix' AND u.name <> '\u{0}n' \
         WITH m, count(*) AS reviews RETURN m.title, reviews ORDER BY reviews DESC LIMIT 5",
    );
    assert_eq!(format!("{:?}", fast.rows), format!("{:?}", generic.rows));
    assert_eq!(fast.rows.len(), 2, "only the two Matrix titles qualify");
}

/// TCK's Pattern1 [10] outline -- a pattern predicate's endpoint that was
/// never bound anywhere else (`a` here) is a compile-time error, not a
/// pattern that silently matches anything.
#[test]
fn pattern_predicate_introducing_new_variable_is_rejected() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)");
    let stmt = parse("MATCH (n) WHERE (n)-[r]->(a) RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string()
            .to_lowercase()
            .contains("undefined variable"),
        "expected an undefined-variable error, got: {err}"
    );
}

/// `a:Label` as a general boolean expression, usable anywhere a
/// `return_expr` is (RETURN/WITH items, not just pattern-level WHERE) --
/// TCK's Graph5 "Node and edge label expressions". Reuses the existing
/// `ReturnExpr::HasLabel` (previously only reachable via the
/// parenthesized `(n:Foo)` general-expression form) through a new bare
/// grammar alternative.
#[test]
fn bare_label_predicate_as_return_expr() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:A:B:C), (:A:B), (:A:C), (:B:C), (:A), (:B), (:C), ()",
    );

    let result = run(&store, "MATCH (a) RETURN a:B AS result");
    let mut vals: Vec<bool> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::Bool(b)) => *b,
            other => panic!("expected a bool, got {other:?}"),
        })
        .collect();
    vals.sort();
    assert_eq!(
        vals,
        vec![false, false, false, false, true, true, true, true]
    );

    // A DELETE target must never be a label predicate (TCK's Delete1
    // [8]) -- a boolean can never be a node/relationship/path, and this
    // must be rejected at compile time regardless of whether any row
    // actually matches (an empty MATCH would otherwise let it through).
    let stmt = parse("MATCH (n) DELETE n:Person").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("delete target"),
        "expected a DELETE-target error, got: {err}"
    );
}

/// `[:A|B]`/`[:A|:B]` -- a relationship pattern matches if the edge's
/// type is any of the listed alternatives (TCK's Match2 [6]/Match3 [8],
/// Pattern1 [13]'s undirected pattern-predicate form). CREATE/MERGE
/// still require exactly one explicit type -- a brand new edge can't be
/// ambiguous about which type it gets.
#[test]
fn multi_type_relationship_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a {name: 'A'}), (b {name: 'B'}), (c {name: 'C'}), \
         (a)-[:KNOWS]->(b), (a)-[:HATES]->(c), (a)-[:WONDERS]->(c)",
    );

    let result = run(&store, "MATCH (n)-[r:KNOWS|HATES]->(x) RETURN r");
    assert_eq!(result.rows.len(), 2);

    // `:T|:T` -- the colon before subsequent alternatives is optional,
    // both forms mean the same thing.
    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE (a)-[:T]->(b)");
    let result = run(&store2, "MATCH (a)-[:T|:T]->(b) RETURN a, b");
    assert_eq!(result.rows.len(), 1);

    // Untyped (`[]`/`[r]`) is unaffected -- still matches any type.
    let store3 = GraphStore::open_memory().unwrap();
    run(&store3, "CREATE (a)-[:X]->(b)");
    let result = run(&store3, "MATCH (a)-[r]->(b) RETURN r");
    assert_eq!(result.rows.len(), 1);

    // CREATE/MERGE reject a multi-type target -- which type would the
    // new edge get?
    let store4 = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a)-[:A|B]->(b)").unwrap();
    assert!(Executor::new(&store4).execute(&stmt).is_err());
    let stmt = parse("MATCH (a), (b) MERGE (a)-[:A|B]->(b)").unwrap();
    assert!(Executor::new(&store4).execute(&stmt).is_err());
}

#[test]
fn named_path_capture_returns_alternating_node_edge_elements() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})-[:LIKES]->(:Person {name: 'Carol'})",
    );
    let result = run(
        &store,
        "MATCH p = (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person)-[:LIKES]->(c:Person) RETURN p",
    );
    let elems = path_elems(&result.rows[0][0]);
    assert_eq!(elems.len(), 5, "node,edge,node,edge,node");
    assert_eq!(node_name(&elems[0]), "Alice");
    assert!(matches!(&elems[1], PathElem::Edge(e) if e.label == "KNOWS"));
    assert_eq!(node_name(&elems[2]), "Bob");
    assert!(matches!(&elems[3], PathElem::Edge(e) if e.label == "LIKES"));
    assert_eq!(node_name(&elems[4]), "Carol");
}

#[test]
fn named_path_capture_with_anonymous_relationships() {
    // Neither hop's relationship is named -- name_pattern_for_path must
    // still track them internally (synthesized names), and they must not
    // leak into the output row (only `p` should be bound/returned here).
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
    );
    let result = run(
        &store,
        "MATCH p = (a:Person {name: 'Alice'})-[]->(b:Person) RETURN p",
    );
    let elems = path_elems(&result.rows[0][0]);
    assert_eq!(elems.len(), 3);
    assert_eq!(node_name(&elems[0]), "Alice");
    assert_eq!(node_name(&elems[2]), "Bob");
}

#[test]
fn shortest_path_finds_the_actual_shortest_not_just_a_path() {
    let store = GraphStore::open_memory().unwrap();
    // Direct 1-hop route.
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Dave'})",
    );
    // Longer 3-hop route between the *same* two people -- MATCH...CREATE,
    // not a chained plain CREATE, so the trailing (:Person{name:'Dave'})
    // token reuses the existing Dave instead of silently creating a 2nd
    // one (a chained CREATE never reuses an unbound token, even one that
    // matches an existing node by props -- exactly the gap MATCH...CREATE
    // exists to work around).
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (d:Person {name: 'Dave'}) \
         CREATE (a)-[:KNOWS]->(:Person {name: 'Bob'})-[:KNOWS]->(:Person {name: 'Carol'})-[:KNOWS]->(d)",
    );
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) OPTIONAL MATCH (d:Person {name: 'Dave'}) \
         OPTIONAL MATCH p = shortestPath((a)-[:KNOWS*]-(d)) RETURN length(p)",
    );
    assert_eq!(
        result.rows.len(),
        1,
        "only one Dave -- must not have duplicated it"
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "must pick the 1-hop route, not the 3-hop one"
    );
}

#[test]
fn shortest_path_returns_null_when_unreachable() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Zed'})");
    let result = run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) OPTIONAL MATCH (z:Person {name: 'Zed'}) \
         OPTIONAL MATCH p = shortestPath((a)-[:KNOWS*]-(z)) RETURN p",
    );
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn shortest_path_requires_both_endpoints_already_bound() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    let stmt =
        parse("MATCH p = shortestPath((a:Person {name: 'Alice'})-[:KNOWS*]-(z:Person)) RETURN p")
            .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("shortestpath"));
}

/// Named-path capture over a *single* variable-length hop -- TCK's
/// Quantifier1-4 [8]/[9], ReturnOrderBy2 [12], Pattern2 [9]. Assembles the
/// path from `expand_variable_row`'s own internally-traversed edge/node
/// sequence (deposited under that hop's own synthesized `rel.var`, see
/// `assemble_path`'s docs), not a plain fixed-hop token.
#[test]
fn named_path_over_a_single_variable_length_hop() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:N {n: 1})-[:KNOWS]->(b:N {n: 2})-[:KNOWS]->(c:N {n: 3})",
    );
    let result = run(
        &store,
        "MATCH (a:N {n: 1}) MATCH p = (a)-[:KNOWS*1..2]->(b) RETURN p, length(p) AS len",
    );
    assert_eq!(result.rows.len(), 2);
    for row in &result.rows {
        match &row[0] {
            Value::Path(elems) => {
                // A path of length L has 2L+1 elements (Node, Edge, Node,
                // ..., Node) -- `nodes.len() == edges.len() + 1`.
                let len = int(&row[1]) as usize;
                assert_eq!(elems.len(), 2 * len + 1);
                assert!(matches!(elems[0], PathElem::Node(_)));
                assert!(matches!(elems[elems.len() - 1], PathElem::Node(_)));
            }
            other => panic!("expected a Path, got {other:?}"),
        }
    }
}

/// Named-path capture over a pattern *mixing* variable-length hops with
/// other hops -- was rejected until a `LogicalPlan::VarExpand`
/// edge-isomorphism gap was fixed (TCK's Match4 `[7]`: an earlier
/// double-count bug in exactly this shape). Two `*0..1` hops
/// around a fixed middle hop -- each of `a`, `b`, `c` optionally coincide,
/// so this must not silently drop or duplicate rows.
#[test]
fn named_path_mixing_variable_length_and_fixed_hops() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:N {n: 1})-[:KNOWS]->(:N {n: 2})-[:LIKES]->(:N {n: 3})-[:KNOWS]->(:N {n: 4})",
    );
    let result = run(
        &store,
        "MATCH p = (a)-[:KNOWS*0..1]->(b)-[:LIKES]->(c)-[:KNOWS*0..1]->(d) RETURN p",
    );
    for row in &result.rows {
        match &row[0] {
            Value::Path(elems) => {
                assert_eq!(elems.len() % 2, 1, "alternating node/edge, odd length");
                assert!(matches!(elems[0], PathElem::Node(_)));
                assert!(matches!(elems[elems.len() - 1], PathElem::Node(_)));
            }
            other => panic!("expected a Path, got {other:?}"),
        }
    }
    assert!(!result.rows.is_empty());
}

#[test]
fn shortest_path_requires_a_variable_length_hop() {
    let err = parse("MATCH p = shortestPath((a)-[:KNOWS]->(b)) RETURN p").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}

#[test]
fn standalone_with_no_preceding_match() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN list");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
}

#[test]
fn type_mismatch_comparison_semantics_differ_by_operator() {
    // Regression: a single blanket "type mismatch -> false" was wrong for
    // three different operator families -- confirmed against real TCK
    // scenarios: `=` on mismatched types is false, `<>` is true (never
    // equal, so "not equal" holds), ordering is null (no defined order),
    // and STARTS WITH/ENDS WITH/CONTAINS on a non-string operand is also
    // null, not false.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN (1 = 'a') AS a, (1 <> 'a') AS b, ('1.0' < 1.0) AS c, ('abc' STARTS WITH true) AS d",
    );
    assert!(!bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
    assert!(matches!(result.rows[0][3], Value::Null));
}

#[test]
fn match_without_declared_index_still_works() {
    // Regression guard: a plain node-pattern-property match with no
    // index declared must still hit the ordinary Filter-over-scan path,
    // not error or silently return nothing just because `apply_index_seeks`
    // now runs over every plan.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com'})");
    let result = run(
        &store,
        "MATCH (n:Person {email: 'alice@x.com'}) RETURN n.email",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "alice@x.com");
}

#[test]
fn explain_shows_expand_between_two_scans() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'a'})-[:KNOWS]->(:Person {name: 'b'})",
    );

    let result = run(
        &store,
        "EXPLAIN MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a, b",
    );
    let lines = plan_lines(&result);
    assert!(lines.iter().any(|l| l.contains("Expand(a)-[:KNOWS]->(b)")));
    assert!(lines
        .iter()
        .any(|l| l.contains("NodeByLabelScan(a:Person)")));
}

#[test]
fn type_error_for_a_runtime_value_shape_mismatch() {
    // The query is syntactically and structurally fine (both operands
    // are "a scalar" as far as the pre-execution semantic pass can tell)
    // -- the mismatch only exists once the actual values are in hand and
    // one turns out to be a bool, not a number.
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 + true").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("type error:"));
}

#[test]
fn bracketless_relationship_arrows_match_the_bracketed_forms() {
    // `-->`/`<--`/`--` are real Cypher's shorthand for an anonymous,
    // untyped, propertyless relationship -- brackets are only needed at
    // all to carry a var/type/range/props. Must behave identically to
    // `-[]->`/`<-[]-`/`-[]-`, not just parse.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {n: 'a'})-[:X]->(:B {n: 'b'})");

    let out = run(&store, "MATCH (a)-->(b) RETURN a.n, b.n");
    assert_eq!(out.rows.len(), 1);
    assert_eq!(str_value(&out.rows[0][0]), "a");
    assert_eq!(str_value(&out.rows[0][1]), "b");

    let in_ = run(&store, "MATCH (b)<--(a) RETURN a.n, b.n");
    assert_eq!(in_.rows.len(), 1);
    assert_eq!(str_value(&in_.rows[0][0]), "a");
    assert_eq!(str_value(&in_.rows[0][1]), "b");

    let either = run(&store, "MATCH (x)--(y) RETURN x.n, y.n");
    assert_eq!(either.rows.len(), 2, "undirected must match both endpoints");
}

#[test]
fn builtin_keys_labels_properties_on_a_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:L1:L2 {a: 1, b: 'x'})");

    let keys = run(&store, "MATCH (n) RETURN keys(n)");
    assert_eq!(list_str_values(&keys.rows[0][0]), vec!["a", "b"]);

    let labels = run(&store, "MATCH (n) RETURN labels(n)");
    assert_eq!(list_str_values(&labels.rows[0][0]), vec!["L1", "L2"]);

    let props = run(&store, "MATCH (n) RETURN properties(n)");
    let Value::Map(m) = &props.rows[0][0] else {
        panic!("expected a map");
    };
    assert_eq!(m.len(), 2);
}

#[test]
fn builtin_type_on_a_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:KNOWS]->(:B)");
    let result = run(&store, "MATCH ()-[r]->() RETURN type(r)");
    assert_eq!(str_value(&result.rows[0][0]), "KNOWS");
}

#[test]
fn builtin_nodes_and_relationships_over_a_path() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:KNOWS]->(:B)");
    let result = run(
        &store,
        "MATCH p = (a:A)-[:KNOWS]->(b:B) RETURN nodes(p), relationships(p)",
    );
    let Value::List(nodes) = &result.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(nodes.len(), 2);
    let Value::List(rels) = &result.rows[0][1] else {
        panic!("expected a list");
    };
    assert_eq!(rels.len(), 1);
}

#[test]
fn label_check_expression_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    run(&store, "CREATE (:Foo)");
    let result = run(&store, "MATCH (n) RETURN (n:Foo)");
    assert_eq!(result.rows.len(), 2);
    assert!(!bool_value(&result.rows[0][0]));
    assert!(bool_value(&result.rows[1][0]));
}

#[test]
fn label_check_expression_requires_every_listed_label() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A:B)");
    let result = run(&store, "MATCH (n:A:B) RETURN (n:A:B), (n:A:C)");
    assert!(bool_value(&result.rows[0][0]));
    assert!(!bool_value(&result.rows[0][1]));
}

#[test]
fn label_check_expression_on_a_null_binding_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "OPTIONAL MATCH (n:DoesNotExist) RETURN (n:Foo)");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn label_check_expression_on_a_non_node_is_a_semantic_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("WITH 5 AS x RETURN (x:Foo)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn union_combines_two_match_clauses() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)");
    run(&store, "CREATE (:B)");
    let result = run(
        &store,
        "MATCH (a:A) RETURN a AS a UNION MATCH (b:B) RETURN b AS a",
    );
    assert_eq!(result.rows.len(), 2);
}

/// `r.name` where `r` is bound to a path -- real Cypher's
/// `InvalidArgumentType`, not a silent null (a path was never a valid
/// property-access target). TCK's MatchWhere1 [14].
#[test]
fn property_access_on_a_path_is_a_type_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH r = (n)-[*]->() WHERE r.name = 'apa' RETURN r").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("path"),
        "unexpected error: {err}"
    );
}

/// `size(p)` where `p` is a path -- real Cypher rejects this at compile
/// time, not just at runtime (a zero-row MATCH could otherwise silently
/// skip ever evaluating it). TCK's List6 [5].
#[test]
fn size_on_a_path_is_a_compile_time_error() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH p = (a)-[*]->(b) RETURN size(p)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("size"),
        "unexpected error: {err}"
    );
}

/// `MATCH (a)-[r*1..3]->(b) RETURN r` -- binding a variable to a
/// variable-length relationship pattern collects the traversed
/// relationships into a list, not a single edge. TCK's Match4 [1]/[6].
#[test]
fn variable_length_relationship_binds_a_list_of_edges() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T]->()");
    let result = run(&store, "MATCH (a)-[r*1..1]->(b) RETURN r");
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                Value::Edge(e) => assert_eq!(e.label, "T"),
                other => panic!("expected an Edge, got {other:?}"),
            }
        }
        other => panic!("expected a List, got {other:?}"),
    }

    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:A), (b), (c) CREATE (a)-[:X]->(b), (b)-[:Y]->(c)",
    );
    let result = run(&store, "MATCH (a:A) MATCH (a)-[r*2]->() RETURN r");
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            let labels: Vec<&str> = items
                .iter()
                .map(|v| match v {
                    Value::Edge(e) => e.label.as_str(),
                    other => panic!("expected an Edge, got {other:?}"),
                })
                .collect();
            assert_eq!(labels, vec!["X", "Y"]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    // Matching a variable-length pattern against an *already-bound* list
    // variable (TCK's Match4 [8]) means "match a path whose edges equal
    // this list" -- deterministic (the edges are already concrete, so
    // there's exactly one possible walk to check), not a fresh BFS search.
    let result = run(
        &store,
        "MATCH ()-[r1]->()-[r2]->() WITH [r1, r2] AS rs LIMIT 1 \
         MATCH (first)-[rs*]->(second) RETURN first, second",
    );
    assert_eq!(result.rows.len(), 1);
    match (&result.rows[0][0], &result.rows[0][1]) {
        (Value::Node(first), Value::Node(second)) => {
            assert!(first.labels.contains(&"A".to_string()));
            assert_eq!(second.labels, Vec::<String>::new());
        }
        other => panic!("expected two nodes, got {other:?}"),
    }
}

/// `WITH null AS a OPTIONAL MATCH p = (a)-[r]->()` -- reusing an
/// already-bound-to-`null` variable as a node/relationship pattern token
/// is legal (matches nothing, same as any other `OPTIONAL MATCH` miss),
/// unlike reusing a variable bound to a real, wrong-typed value (`WITH 1
/// AS x MATCH (x)-->()`, still a real compile-time type error). TCK's
/// Path1 [1], Path2 [3].
#[test]
fn null_bound_variable_reused_as_pattern_token_is_legal() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH null AS a \
         OPTIONAL MATCH p = (a)-[r]->() \
         RETURN type(r), nodes(p), relationships(p), length(p)",
    );
    for cell in &result.rows[0] {
        assert!(matches!(cell, Value::Null), "expected null, got {cell:?}");
    }

    // A real, non-null, wrong-typed reused variable must still error.
    let stmt = parse("WITH 1 AS x MATCH (x)-[:R]->(n) RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("node"));
}

/// `MATCH (a)-[:TYPE* {prop: 'x'}]->(b)` -- filters *every* hop of the
/// variable-length traversal by the same inline property map, not just
/// the final one. A 2-hop path where only the second hop matches doesn't
/// survive as a 1-hop match either -- the whole path from the first
/// non-matching hop onward is excluded. TCK's Match4 [5].
#[test]
fn inline_properties_on_a_variable_length_relationship_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Artist:A), (b:Artist:B), (c:Artist:C) \
         CREATE (a)-[:WORKED_WITH {year: 1987}]->(b), \
                (b)-[:WORKED_WITH {year: 1988}]->(c)",
    );
    let result = run(
        &store,
        "MATCH (a:Artist)-[:WORKED_WITH* {year: 1988}]->(b:Artist) RETURN a, b",
    );
    assert_eq!(result.rows.len(), 1);
    let a_labels = match &result.rows[0][0] {
        Value::Node(n) => &n.labels,
        other => panic!("expected a Node, got {other:?}"),
    };
    let b_labels = match &result.rows[0][1] {
        Value::Node(n) => &n.labels,
        other => panic!("expected a Node, got {other:?}"),
    };
    assert!(a_labels.contains(&"B".to_string()));
    assert!(b_labels.contains(&"C".to_string()));
}

// --- comma-separated MATCH patterns: cross joins, correlated parts, and
// clause-wide relationship uniqueness ------------------------------------

/// `MATCH (a:A), (b:B)` is a genuine cross join: every combination of the
/// two disjoint parts, exactly the Cartesian product. (Split into
/// separate `QueryPart`s by `group_into_linear_patterns`; found working
/// while porting a third-party Northwind benchmark whose loader depends
/// on this shape, previously listed as "not verified" in
/// CYPHER_COVERAGE.md.)
#[test]
fn comma_separated_disjoint_match_is_a_cross_join() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {x: 1}), (:A {x: 2})");
    run(&store, "CREATE (:B {y: 10}), (:B {y: 20}), (:B {y: 30})");
    let result = run(
        &store,
        "MATCH (a:A), (b:B) RETURN a.x, b.y ORDER BY a.x, b.y",
    );
    let rows: Vec<(i64, i64)> = result
        .rows
        .iter()
        .map(|r| (int(&r[0]), int(&r[1])))
        .collect();
    assert_eq!(
        rows,
        vec![(1, 10), (1, 20), (1, 30), (2, 10), (2, 20), (2, 30)]
    );
}

/// Real Cypher's relationship-uniqueness rule spans the *whole* MATCH
/// clause pattern, comma-separated parts included: two parts sharing a
/// start node may not bind the same relationship instance, so the
/// tag-co-occurrence shape yields only the (a,b)/(b,a) cross pairs, never
/// the (a,a)/(b,b) self-pairs that reusing one HAS_TAG edge for both hops
/// would produce. (The bug this guards against was masked in the
/// benchmark that surfaced it by a `WHERE t1.id < t2.id` filter.)
#[test]
fn comma_separated_parts_of_one_match_share_relationship_uniqueness() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (p:Post)-[:HAS_TAG]->(:Tag {name: 'a'}) CREATE (p)-[:HAS_TAG]->(:Tag {name: 'b'})",
    );
    let result = run(
        &store,
        "MATCH (p:Post)-[:HAS_TAG]->(t1:Tag), (p)-[:HAS_TAG]->(t2:Tag) \
         RETURN t1.name, t2.name ORDER BY t1.name, t2.name",
    );
    let rows: Vec<(String, String)> = result
        .rows
        .iter()
        .map(|r| (str_value(&r[0]), str_value(&r[1])))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string())
        ]
    );

    // A separate MATCH *clause* starts a fresh uniqueness scope -- it may
    // bind the very same relationship again (2 tags x 2 tags = 4 rows,
    // self-pairs included).
    let result = run(
        &store,
        "MATCH (p:Post)-[:HAS_TAG]->(t1:Tag) MATCH (p)-[:HAS_TAG]->(t2:Tag) \
         RETURN t1.name, t2.name",
    );
    assert_eq!(result.rows.len(), 4);
}

/// The uniqueness scope covers variable-length hops across parts too: on
/// a single-edge graph, a fixed hop in the second part can't re-bind the
/// one edge the first part's var-length traversal used.
#[test]
fn comma_separated_parts_exclude_a_var_length_hops_traversed_edges() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {n: 1})-[:R]->(:Y {n: 2})");
    let result = run(&store, "MATCH (a)-[:R*1..1]->(b), (c)-[r:R]->(d) RETURN r");
    assert_eq!(result.rows.len(), 0);

    // With a second edge available, each part takes a different one.
    run(&store, "MATCH (y:Y {n: 2}) CREATE (y)-[:R]->(:Z {n: 3})");
    let result = run(
        &store,
        "MATCH (a)-[:R*1..1]->(b), (c)-[r:R]->(d) RETURN a.n, c.n",
    );
    assert_eq!(result.rows.len(), 2);
}

/// Reusing one relationship-variable NAME across comma-separated parts of
/// one MATCH clause is a compile-time error (same rule as within one
/// pattern) -- while a later, separate MATCH clause reusing the name is
/// legal and means "verify this exact relationship again".
#[test]
fn relationship_variable_reuse_across_comma_parts_errors_across_clauses_verifies() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {n: 1})-[:R]->(:Y {n: 2})");
    let stmt = parse("MATCH (a)-[r]->(b), (c)-[r]->(d) RETURN r").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("relationship variable"),
        "expected the same-pattern reuse error, got: {err}"
    );

    let result = run(&store, "MATCH (a)-[r]->(b) MATCH (x)-[r]->(y) RETURN x.n");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// Comma-separated CREATE patterns: disjoint node groups and disjoint
/// relationship chains in one clause each create independently (also a
/// previously "not verified" CYPHER_COVERAGE.md shape).
#[test]
fn comma_separated_create_patterns() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "CREATE (a:A {x: 1}), (b:B {y: 2})");
    assert_eq!(result.stats.nodes_created, 2);
    let result = run(&store, "CREATE (:C)-[:R]->(:D), (:E)-[:S]->(:F)");
    assert_eq!(result.stats.nodes_created, 4);
    assert_eq!(result.stats.relationships_created, 2);
    let result = run(&store, "MATCH ()-[r]->() RETURN type(r) ORDER BY type(r)");
    let types: Vec<String> = result.rows.iter().map(|r| str_value(&r[0])).collect();
    assert_eq!(types, vec!["R".to_string(), "S".to_string()]);
}

// --- RETURN DISTINCT ... LIMIT early termination over var-length
// traversal (lazy VarExpandIter + collect_rows_until_distinct) ----------

/// The pipeline must stop pulling as soon as LIMIT-many distinct
/// projected rows exist — proven the same way the plain-LIMIT laziness
/// test in smoke_filtering.rs does: a relationship-expansion budget too
/// small for full enumeration but ample for the early stop. A 6-node
/// :K clique enumerates ~85 edge-distinct paths for `*1..3` from any
/// node, while the first handful of DFS steps already yield 2 distinct
/// endpoints.
#[test]
fn distinct_limit_terminates_var_length_traversal_early() {
    use marsdb_query::{ExecutionOptions, QueryError};
    let store = GraphStore::open_memory().unwrap();
    let mut ids = Vec::new();
    ids.push(
        store
            .create_node(&["Start"], std::collections::BTreeMap::new())
            .unwrap(),
    );
    for _ in 0..5 {
        ids.push(
            store
                .create_node(&["Mid"], std::collections::BTreeMap::new())
                .unwrap(),
        );
    }
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            store
                .create_edge("K", ids[i], ids[j], std::collections::BTreeMap::new())
                .unwrap();
        }
    }
    let executor = Executor::new(&store);
    let budget = ExecutionOptions {
        max_relationship_expansions: Some(25),
        ..Default::default()
    };

    // Early stop: 2 distinct endpoints found within the budget.
    let limited = parse("MATCH (s:Start)-[:K*1..3]-(m) RETURN DISTINCT m LIMIT 2").unwrap();
    let result = executor.execute_with_options(&limited, &budget).unwrap();
    assert_eq!(result.rows.len(), 2);

    // Without the LIMIT the same budget is exhausted by full enumeration
    // — the early stop above genuinely skipped that work rather than
    // doing it and discarding rows.
    let unlimited = parse("MATCH (s:Start)-[:K*1..3]-(m) RETURN DISTINCT m").unwrap();
    let err = executor
        .execute_with_options(&unlimited, &budget)
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)), "got: {err}");

    // ORDER BY disqualifies the early stop (it must see every row) —
    // same budget, same exhaustion.
    let ordered =
        parse("MATCH (s:Start)-[:K*1..3]-(m) RETURN DISTINCT m.x ORDER BY m.x LIMIT 2").unwrap();
    let err = executor
        .execute_with_options(&ordered, &budget)
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)), "got: {err}");
}

/// The early-terminated result is a correct DISTINCT result: exactly
/// min(LIMIT, |full DISTINCT set|) rows, internally duplicate-free, and
/// a subset of the un-LIMITed DISTINCT rows. SKIP composes on top (it
/// slices after dedup, so SKIP s LIMIT k needs s+k distinct collected).
#[test]
fn distinct_limit_rows_are_a_correct_distinct_subset() {
    let store = GraphStore::open_memory().unwrap();
    // 4 people, each knowing the others (6 undirected edges): `*1..2`
    // reaches every other person through many paths.
    let mut ids = Vec::new();
    for i in 0..4 {
        let mut props = std::collections::BTreeMap::new();
        props.insert("id".to_string(), marsdb_graph::PropertyValue::Int(i));
        ids.push(store.create_node(&["P"], props).unwrap());
    }
    for i in 0..4 {
        for j in (i + 1)..4 {
            store
                .create_edge("K", ids[i], ids[j], std::collections::BTreeMap::new())
                .unwrap();
        }
    }
    let full: std::collections::BTreeSet<i64> = run(
        &store,
        "MATCH (p:P {id: 0})-[:K*1..2]-(m:P) RETURN DISTINCT m.id",
    )
    .rows
    .iter()
    .map(|r| int(&r[0]))
    .collect();

    let limited = run(
        &store,
        "MATCH (p:P {id: 0})-[:K*1..2]-(m:P) RETURN DISTINCT m.id LIMIT 2",
    );
    let got: Vec<i64> = limited.rows.iter().map(|r| int(&r[0])).collect();
    assert_eq!(got.len(), 2);
    let got_set: std::collections::BTreeSet<i64> = got.iter().copied().collect();
    assert_eq!(got_set.len(), 2, "LIMITed DISTINCT rows must be distinct");
    assert!(got_set.is_subset(&full));

    // LIMIT larger than the distinct set: every distinct row, once.
    let all = run(
        &store,
        "MATCH (p:P {id: 0})-[:K*1..2]-(m:P) RETURN DISTINCT m.id LIMIT 100",
    );
    let all_set: std::collections::BTreeSet<i64> = all.rows.iter().map(|r| int(&r[0])).collect();
    assert_eq!(all_set, full);
    assert_eq!(all.rows.len(), full.len());

    // SKIP composes: 1 skipped + 2 returned, all distinct, all valid.
    let skipped = run(
        &store,
        "MATCH (p:P {id: 0})-[:K*1..2]-(m:P) RETURN DISTINCT m.id SKIP 1 LIMIT 2",
    );
    let sk: std::collections::BTreeSet<i64> = skipped.rows.iter().map(|r| int(&r[0])).collect();
    assert_eq!(sk.len(), 2);
    assert!(sk.is_subset(&full));
}
