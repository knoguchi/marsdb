//! Smoke tests: CREATE/MERGE/SET/REMOVE/DELETE write-path behavior.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

#[test]
fn create_match_return() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice', age: 30})-[:KNOWS]->(b:Person {name: 'Bob', age: 25})",
    );

    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(result.columns, vec!["n.name"]);
    let mut names: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    names.sort();
    assert_eq!(names, vec!["Alice", "Bob"]);
}

/// Chaining multiple `CREATE` clauses in one statement shares bindings
/// across all of them -- a repeated `CREATE` keyword is just another
/// pattern separator, same as `,`. TCK fixtures commonly split this as
/// `CREATE (a {..}), (b {..})\nCREATE (a)-[:T]->(b)`; running each line
/// as an independent statement would lose the shared `a`/`b` bindings.
#[test]
fn chained_create_clauses_share_scope() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a {name: 'a'}), (b {name: 'b'})\nCREATE (a)-[:T]->(b)",
    );
    let result = run(&store, "MATCH p = ({name: 'a'})-->({name: 'b'}) RETURN p");
    assert_eq!(result.rows.len(), 1);

    // Three-deep chaining, not just two.
    let store2 = GraphStore::open_memory().unwrap();
    run(
        &store2,
        "CREATE (a {n: 1})\nCREATE (b {n: 2})\nCREATE (a)-[:T]->(b)",
    );
    let result2 = run(&store2, "MATCH ({n: 1})-->({n: 2}) RETURN 1");
    assert_eq!(result2.rows.len(), 1);
}

#[test]
fn skip_past_the_end_of_the_result_set_returns_empty_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (n:Item {{idx: {i}}})"));
    }
    let result = run(
        &store,
        "MATCH (n:Item) RETURN n.idx AS v ORDER BY v SKIP 100",
    );
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn detach_delete() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(&store, "MATCH (n:Person {name: 'Alice'}) DETACH DELETE n");
    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn multi_label_create_and_match() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (p:Post:Message {id: 1})");
    run(&store, "CREATE (c:Comment:Message {id: 2})");
    run(&store, "CREATE (p2:Post {id: 3})"); // single-label, must NOT match :Message

    let as_message = run(&store, "MATCH (m:Message) RETURN m.id");
    let mut ids: Vec<i64> = as_message
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);

    let as_post = run(&store, "MATCH (p:Post) RETURN p.id");
    let mut post_ids: Vec<i64> = as_post
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(v)) => *v,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    post_ids.sort();
    assert_eq!(post_ids, vec![1, 3]);

    // multi-label pattern match: AND semantics
    let both = run(&store, "MATCH (n:Post:Message) RETURN n.id");
    assert_eq!(both.rows.len(), 1);
    match &both.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 1),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn match_create_binds_created_relationship_for_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Root {id: 1})");
    let result = run(
        &store,
        "MATCH (a:Root {id: 1}) CREATE (a)-[r:LINK]->(b:Leaf) RETURN r",
    );
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(&result.rows[0][0], Value::Edge(edge) if edge.label == "LINK"));
}

#[test]
fn create_rejects_undirected_pattern_at_execute_time() {
    // CREATE and MATCH share the same pattern rule, so an undirected
    // `-[...]-` parses fine; execute_create rejects it since CREATE always
    // needs a direction to know which node is src and which is dst.
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Person)-[:KNOWS]-(b:Person)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("direct"));
}

#[test]
fn create_rejects_variable_length_pattern() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Item)-[:NEXT*1..3]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}

/// A comma-separated cross join followed directly by `MERGE`: the
/// disjoint groups become separate `QueryClause::Match` entries, and
/// `MERGE` still sees both `a`/`b` bound (TCK's Merge6 [1]).
#[test]
fn comma_separated_match_cross_join_feeds_a_following_merge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A), (:B)");

    run(
        &store,
        "MATCH (a:A), (b:B) MERGE (a)-[:KNOWS]->(b) ON CREATE SET b.created = 1",
    );
    let result = run(&store, "MATCH (b:B) RETURN b.created");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// MERGE may need to *create* its relationship on no-match, so an
/// untyped edge pattern is rejected same as CREATE's (TCK's Merge5 [24]).
#[test]
fn merge_requires_an_explicit_relationship_type() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (a), (b) MERGE (a)-[NO_COLON]->(b)").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("an untyped MERGE relationship pattern must be rejected");
    let msg = format!("{err}");
    assert!(
        msg.contains("explicit relationship type"),
        "unexpected error: {msg}"
    );
}

#[test]
fn match_create_connects_two_already_existing_nodes() {
    // Standalone CREATE always makes fresh nodes; WITH-chaining is what
    // lets two independently matched *existing* nodes both stay bound in
    // the same row for the CREATE tail.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    run(&store, "CREATE (b:Person {name: 'Bob'})");

    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) CREATE (a)-[:KNOWS]->(b)",
    );

    // No new Person nodes were created -- still exactly 2.
    let people = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(people.rows.len(), 2);

    let result = run(
        &store,
        "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Bob");
}

#[test]
fn match_create_adds_new_node_to_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");

    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:OWNS]->(i:Item {name: 'Widget'})",
    );

    let result = run(
        &store,
        "MATCH (a:Person)-[:OWNS]->(i:Item) RETURN a.name, i.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
    assert_eq!(str_value(&result.rows[0][1]), "Widget");
}

#[test]
fn match_create_runs_once_per_matched_row() {
    let store = GraphStore::open_memory().unwrap();
    for i in 0..3 {
        run(&store, &format!("CREATE (p:Person {{idx: {i}}})"));
    }

    run(&store, "MATCH (p:Person) CREATE (p)-[:HAS_LOG]->(l:Log)");

    let result = run(
        &store,
        "MATCH (p:Person)-[:HAS_LOG]->(l:Log) RETURN count(*)",
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        3,
        "one new Log node per matched Person row"
    );
}

#[test]
fn match_create_rejects_relabeling_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    let stmt =
        parse("MATCH (a:Person {name: 'Alice'}) CREATE (a:Employee)-[:X]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MATCH (a) CREATE (a)` -- a bare already-bound node with no
/// relationship creates and connects nothing, so it's rejected outright
/// rather than silently no-op'ing.
#[test]
fn match_create_rejects_bare_already_bound_node_with_no_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// TCK Create1 [19]: `(n {})` -- an explicit but empty inline map --
/// still counts as imposing a new predicate on an already-bound node,
/// same as a non-empty one. `NodePattern.props` alone can't tell
/// `(n {})` apart from `(n)` (both give an empty `Vec`), hence
/// `has_explicit_props` to preserve the syntactic distinction.
#[test]
fn create_rejects_empty_explicit_prop_map_on_already_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (n:Foo) CREATE (n {})-[:OWNS]->(:Dog)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// TCK Merge5 [22]: MERGE's node endpoints share CREATE's already-bound
/// check -- an already-bound variable can't carry a new label/property
/// predicate even as a different endpoint of the same relationship.
#[test]
fn merge_rejects_new_label_predicate_on_already_bound_node() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (a:Foo) MERGE (a)-[r:KNOWS]->(a:Bar)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

#[test]
fn match_create_rejects_variable_length_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)-[:NEXT*1..3]->(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("variable-length"));
}

#[test]
fn match_create_rejects_undirected_pattern() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Item)");
    let stmt = parse("MATCH (a:Item) CREATE (a)-[:X]-(b:Item)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("undirected"));
}

#[test]
fn merge_single_node_creates_then_reuses() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "MERGE (n:Person {name: 'Alice'})");
    run(&store, "MERGE (n:Person {name: 'Alice'})");
    let result = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "second MERGE must reuse, not create a duplicate"
    );
}

#[test]
fn merge_one_hop_both_endpoints_bound_reuses_existing_edge() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'})",
    );
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) MERGE (a)-[:KNOWS]->(b)",
    );
    let result = run(
        &store,
        "MATCH (:Person)-[r:KNOWS]->(:Person) RETURN count(*)",
    );
    assert_eq!(
        int_value(&result.rows[0][0]),
        1,
        "must reuse the existing edge, not create a 2nd one"
    );
}

#[test]
fn merge_one_hop_both_endpoints_bound_creates_missing_edge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Bob'})");
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) WITH a MATCH (b:Person {name: 'Bob'}) MERGE (a)-[:KNOWS]->(b)",
    );
    let result = run(
        &store,
        "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)",
    );
    assert_eq!(int_value(&result.rows[0][0]), 1);
    // No new nodes -- both endpoints already existed, only the edge is new.
    let nodes = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(int_value(&nodes.rows[0][0]), 2);
}

#[test]
fn merge_one_fresh_endpoint_does_not_reuse_an_unconnected_matching_node() {
    // A per-token scan would find this pre-existing, unconnected Bob and
    // wrongly reuse it; the composite search must come up empty and
    // create a brand-new, properly-connected Bob instead.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'Alice'})");
    run(&store, "CREATE (:Person {name: 'Bob'})"); // unconnected to Alice
    run(
        &store,
        "MATCH (a:Person {name: 'Alice'}) MERGE (a)-[:KNOWS]->(b:Person {name: 'Bob'})",
    );

    let bobs = run(&store, "MATCH (n:Person {name: 'Bob'}) RETURN count(*)");
    assert_eq!(
        int_value(&bobs.rows[0][0]),
        2,
        "must create a 2nd Bob, not reuse the unconnected one"
    );
    let connected = run(
        &store,
        "MATCH (:Person {name: 'Alice'})-[:KNOWS]->(:Person {name: 'Bob'}) RETURN count(*)",
    );
    assert_eq!(int_value(&connected.rows[0][0]), 1);
}

#[test]
fn merge_standalone_both_endpoints_fresh() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    run(
        &store,
        "MERGE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    let nodes = run(&store, "MATCH (n:Person) RETURN count(*)");
    assert_eq!(
        int_value(&nodes.rows[0][0]),
        2,
        "2nd MERGE must reuse both nodes and the edge, not duplicate"
    );
    let edges = run(
        &store,
        "MATCH (:Person)-[:KNOWS]->(:Person) RETURN count(*)",
    );
    assert_eq!(int_value(&edges.rows[0][0]), 1);
}

#[test]
fn merge_on_create_and_on_match_fire_on_the_right_rows() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2",
    );
    let after_create = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_create.rows[0][0]), 1);

    run(
        &store,
        "MERGE (n:Person {name: 'Alice'}) ON CREATE SET n.seen = 1 ON MATCH SET n.seen = 2",
    );
    let after_match = run(&store, "MATCH (n:Person) RETURN n.seen");
    assert_eq!(int_value(&after_match.rows[0][0]), 2);
}

/// `MERGE (a)` -- a completely unconstrained, unbound node pattern (no
/// label/property) is valid Cypher (TCK's Merge1 [1]): it searches
/// for/creates any node with no constraints at all.
#[test]
fn merge_unconstrained_node_pattern_creates_then_matches() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "MERGE (a) RETURN count(*) AS n");
    assert_eq!(int(&result.rows[0][0]), 1);
    assert_eq!(
        int(&run(&store, "MATCH (n) RETURN count(*) AS c").rows[0][0]),
        1
    );

    // A second MERGE (a) must match the existing node, not create a
    // second one.
    run(&store, "MERGE (a)");
    assert_eq!(
        int(&run(&store, "MATCH (n) RETURN count(*) AS c").rows[0][0]),
        1
    );
}

/// A never-interned property name takes the per-property path's shortcut
/// (no record decode), but the two "missing" kinds must stay distinct:
/// absent-on-a-live-node is null, while any property access on a node
/// deleted earlier in the statement is an error, even a never-interned one.
#[test]
fn never_interned_property_is_null_on_live_nodes_but_still_errors_on_deleted_ones() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item {present: 1})");

    let result = run(&store, "MATCH (n:Item) RETURN n.no_such_prop_ever");
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], Value::Null));

    let executor = Executor::new(&store);
    let stmt = parse("MATCH (n:Item) DELETE n RETURN n.no_such_prop_ever").unwrap();
    let err = executor.execute(&stmt).unwrap_err();
    assert!(
        matches!(err, marsdb_query::QueryError::UnboundVariable(_)),
        "deleted-entity access must error, not read as null: {err}"
    );
}

#[test]
fn node_cache_resets_across_the_write_transaction_entry_point_too() {
    let store = GraphStore::open_memory().unwrap();
    let executor = Executor::new(&store);

    executor
        .execute(&parse("CREATE (:Item {name: 'v1'})").unwrap())
        .unwrap();
    // Populates the node cache via `execute`; `node_cache_enabled` stays
    // set until the *next* entry-point call resets it.
    executor
        .execute(&parse("MATCH (n:Item) RETURN n.name").unwrap())
        .unwrap();

    // Same node via the *other* entry point: this statement mutates it
    // then reads it back, and must see its own fresh write, not the
    // previous statement's cached 'v1'.
    let write_txn = store.begin_write().unwrap();
    let result = executor
        .execute_in_write_transaction(
            &parse("MATCH (n:Item) SET n.name = 'v2' WITH n RETURN n.name AS after").unwrap(),
            &write_txn,
        )
        .unwrap();
    write_txn.commit().unwrap();

    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "v2"),
        other => {
            panic!("unexpected value {other:?}, cache leaked across the write-txn entry point")
        }
    }
}

/// `SET ... WITH ...` continues the query past the mutation instead of
/// only allowing a trailing `RETURN` (TCK's Set6 [5]). `QueryClause::Set`
/// doesn't change row bindings, only the underlying graph -- the SET's
/// side effect (all 5 nodes incremented) is independent of the later
/// WHERE filter only letting 3 through to the final RETURN.
#[test]
fn set_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) SET n.num = n.num + 1 WITH n WHERE n.num % 2 = 0 RETURN n.num AS num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4, 6]);

    // The SET's own side effect applies to every matched row, not just
    // the ones that survive the later WHERE filter.
    let all = run(&store, "MATCH (n:N) RETURN n.num");
    let mut all_vals: Vec<i64> = all.rows.iter().map(|row| int(&row[0])).collect();
    all_vals.sort();
    assert_eq!(all_vals, vec![2, 3, 4, 5, 6]);

    // `SET ... RETURN` (no WITH) must still work: `set_as_clause`'s
    // positive lookahead only fires when a WITH is definitely next.
    let result = run(&store, "MATCH (n:N {num: 2}) SET n.num = 100 RETURN n.num");
    assert_eq!(int(&result.rows[0][0]), 100);

    // A bare terminal SET with nothing after must still work too.
    run(&store, "MATCH (n:N {num: 3}) SET n.num = 200");
    let result = run(&store, "MATCH (n:N {num: 200}) RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `DELETE ... WITH ...` -- same continuation as `SET ... WITH ...` above,
/// applied to `QueryClause::Delete` (TCK's Delete6 [5]/[6]/[7]). `num` is
/// carried into a `WITH`-projected scalar before the DELETE, so the later
/// WHERE filter/aggregation never touches the now-gone node.
#[test]
fn delete_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) WITH n, n.num AS num DELETE n WITH num WHERE num % 2 = 0 RETURN num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4]);

    // Every matched node was actually deleted, not just filtered out of
    // the result set.
    let left = run(&store, "MATCH (n:N) RETURN count(n) AS c");
    assert_eq!(int(&left.rows[0][0]), 0);

    // The pre-existing `DELETE ... RETURN` (no WITH) shape must still work.
    run(&store, "CREATE (:M {num: 1})");
    let result = run(&store, "MATCH (n:M) DELETE n RETURN count(*) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `REMOVE ... WITH ...` -- same continuation, applied to
/// `QueryClause::Remove` (TCK's Remove3).
#[test]
fn remove_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();
    for i in 1..=5 {
        run(&store, &format!("CREATE (:N {{num: {i}, tag: 'x'}})"));
    }

    let result = run(
        &store,
        "MATCH (n:N) REMOVE n.tag WITH n WHERE n.num % 2 = 0 RETURN n.num AS num",
    );
    let mut vals: Vec<i64> = result.rows.iter().map(|row| int(&row[0])).collect();
    vals.sort();
    assert_eq!(vals, vec![2, 4]);

    // The REMOVE's own side effect applies to every matched row, not just
    // the ones that survive the later WHERE filter.
    let tagged = run(
        &store,
        "MATCH (n:N) WHERE n.tag IS NOT NULL RETURN count(n) AS c",
    );
    assert_eq!(int(&tagged.rows[0][0]), 0);

    // The pre-existing `REMOVE ... RETURN` (no WITH) shape must still work.
    let result = run(&store, "MATCH (n:N {num: 2}) REMOVE n:N RETURN labels(n)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
}

/// A standalone (no preceding `MATCH`) `CREATE ... RETURN ...`/`EXPLAIN
/// CREATE ... RETURN ...` -- TCK's Graph3 "Node labels" and similarly-
/// shaped Create1 scenarios. A grammar `|` alternative is never revisited
/// once matched, so `create_stmt_only` needs a `!(return_clause |
/// with_clause)` lookahead to fall through to `match_stmt`'s
/// `mutating_tail`, which supports the trailing RETURN.
#[test]
fn standalone_create_followed_by_return() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());

    let result = run(
        &store,
        "CREATE (node:Foo:Bar {name: 'Mattias'}) RETURN labels(node)",
    );
    let mut labels = list_str_values(&result.rows[0][0]);
    labels.sort();
    assert_eq!(labels, vec!["Bar".to_string(), "Foo".to_string()]);

    // The plain, nothing-after shape (Statement::Create) must still work.
    run(&store, "CREATE (:X), (:Y)");
    let result = run(&store, "MATCH (n) RETURN count(n) AS c");
    assert_eq!(int(&result.rows[0][0]), 4);

    // EXPLAIN has the same `create_stmt`-in-ordered-choice trap.
    let stmt = parse("EXPLAIN CREATE (:Q)").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_ok());
    let stmt = parse("EXPLAIN CREATE (n:Q) RETURN n").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_ok());
}

/// `CREATE ... WITH ...` continues the query past the mutation instead
/// of only being the last (optionally RETURN-followed) thing in a
/// statement (TCK's Create3/Match4/Match5/Match6 fixtures). Unlike
/// `SET`/`DELETE`/`REMOVE`-as-clause, CREATE changes row bindings: the
/// second CREATE sees the first CREATE's `a` binding, carried forward
/// through an intervening `WITH *`.
#[test]
fn create_followed_by_with_continues_the_query() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(
        &store,
        "CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)",
    );
    assert!(result.rows.is_empty());

    let nodes = run(&store, "MATCH (n) RETURN count(n) AS c");
    assert_eq!(int(&nodes.rows[0][0]), 2);
    let rels = run(&store, "MATCH ()-[r:T]->() RETURN count(r) AS c");
    assert_eq!(int(&rels.rows[0][0]), 1);

    // CREATE, WITH *, UNWIND, then another CREATE: each fanned-out node
    // must still see the first CREATE's binding.
    let store2 = GraphStore::open_memory().unwrap();
    let result = run(
        &store2,
        "CREATE (a {var: 'start'})
         WITH *
         UNWIND range(1, 5) AS i
         CREATE (n {var: i})-[:T]->(a)
         RETURN count(n) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 5);

    // Standalone `CREATE ... RETURN ...`/bare-CREATE shapes must still work.
    let result = run(&store2, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
    run(&store2, "CREATE (:X), (:Y)");
    let result = run(&store2, "MATCH (n:X) RETURN count(n) AS c");
    assert_eq!(int(&result.rows[0][0]), 1);
}

/// `MATCH (a) MERGE (a)` -- a bare already-bound node with no
/// relationship doesn't search for or create anything, so it's rejected,
/// the same rule `materialize_create` applies to standalone `CREATE (a)`.
#[test]
fn merge_rejects_bare_already_bound_node_with_no_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Item)");
    let stmt = parse("MATCH (a:Item) MERGE (a)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)` -- reusing an already-bound
/// relationship as MERGE's own pattern token is always an error; unlike
/// a node endpoint, MERGE has no "search using this specific existing
/// edge" mode.
#[test]
fn merge_rejects_reusing_an_already_bound_relationship() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:R]->(:B)");
    let stmt = parse("MATCH (a)-[r]->(b) MERGE (a)-[r]->(b)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("already bound"),
        "expected an already-bound error, got: {err}"
    );
}

/// `MERGE ({num: null})` -- a null-valued property in MERGE's own
/// pattern can never be searched-or-created consistently (null never
/// matches on search, but storing null is the same as not storing the
/// property at all).
#[test]
fn merge_rejects_a_null_valued_pattern_property() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MERGE ({num: null})").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("null"),
        "expected a null-property error, got: {err}"
    );
}

#[test]
fn merge_two_hop_pattern_errors_at_parse_time() {
    let err = parse("MERGE (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)").unwrap_err();
    assert!(err
        .to_string()
        .to_lowercase()
        .contains("one relationship hop"));
}

#[test]
fn merge_can_be_followed_by_return() {
    // The whole reason MERGE is a QueryClause, not a Tail -- unlike
    // MATCH...CREATE, MATCH...MERGE...RETURN must work.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "MERGE (n:Person {name: 'Alice'}) RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "Alice");
}

/// `CREATE (n {prop: null})` never stores `prop`, same as `SET n.prop =
/// null` removing a property. Observable via `keys()` (TCK's Graph8
/// [8]): a stored `PropertyValue::Null` would still show up as a key.
#[test]
fn create_inline_null_property_is_never_stored() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({exists: 42, missing: null})");

    let result = run(
        &store,
        "MATCH (n) RETURN 'exists' IN keys(n) AS a, 'missing' IN keys(n) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

/// A `time()` string argument with no offset defaults to UTC (`Z`),
/// the statement default time zone, rather than erroring -- TCK's
/// Temporal10, `time('14:30')`.
#[test]
fn time_string_with_no_offset_defaults_to_utc() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN toString(time('21:40:32'))");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32Z");
}

#[test]
fn create_with_list_property_round_trips_as_a_real_list() {
    // A list-valued property (`PropertyValue::List`) reads back as a
    // genuine `Value::List`, not an opaque `Value::Property(..)`, so list
    // operations (indexing, `size()`, `IN`, `UNWIND`) work transparently.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n {tags: [1, 2, 3]})");
    let result = run(&store, "MATCH (n) RETURN n.tags");
    let Value::List(items) = &result.rows[0][0] else {
        panic!("expected a list, got {:?}", result.rows[0][0]);
    };
    let ints: Vec<i64> = items.iter().map(int).collect();
    assert_eq!(ints, vec![1, 2, 3]);

    let indexed = run(&store, "MATCH (n) RETURN n.tags[1], size(n.tags)");
    assert_eq!(int(&indexed.rows[0][0]), 2);
    assert_eq!(int(&indexed.rows[0][1]), 3);

    let contains = run(&store, "MATCH (n) WHERE 2 IN n.tags RETURN n");
    assert_eq!(contains.rows.len(), 1);
}

/// An unstorable property shape (map, node, edge, path) must error
/// clearly -- widening `value_to_storable_property` to accept
/// `Value::List` must not have widened it to accept everything.
#[test]
fn create_with_unsupported_map_property_errors_clearly_not_silently_nulls() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (n {tags: {a: 1}})").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("property"),
        "expected a clear error, got: {err}"
    );
}

/// `SET n = {...}` replaces every existing property (removing any key
/// not in the map); `SET n += {...}` merges instead (only the map's keys
/// change, `null` removes that key, the rest stays untouched) -- TCK's
/// Set4/Set5. Both also accept a bound node/relationship as the RHS
/// (`SET r = a` copies `a`'s properties -- TCK's Merge6 [6]/Merge7 [4]).
#[test]
fn set_map_assign_replace_and_merge() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X {name: 'A', name2: 'B'})");

    // `=` replaces everything -- `name2` is gone, `baz` is new.
    let result = run(
        &store,
        "MATCH (n:X {name: 'A'}) SET n = {name: 'B', baz: 'C'} RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert_eq!(node.props.len(), 2);
    assert_eq!(
        node.props.get("name"),
        Some(&marsdb_graph::PropertyValue::String("B".to_string()))
    );
    assert_eq!(
        node.props.get("baz"),
        Some(&marsdb_graph::PropertyValue::String("C".to_string()))
    );
    assert!(!node.props.contains_key("name2"));

    // `+=` merges -- existing untouched keys survive.
    run(&store, "CREATE (:Y {name: 'A'})");
    let result = run(
        &store,
        "MATCH (n:Y {name: 'A'}) SET n += {name2: 'B'} RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert_eq!(node.props.len(), 2);

    // Copying properties from a bound node, not just a map literal.
    run(&store, "CREATE (:P {name: 'A'}), (:Q)");
    run(&store, "MATCH (p:P), (q:Q) SET q = p");
    let result = run(&store, "MATCH (q:Q) RETURN q.name");
    assert_eq!(str_value(&result.rows[0][0]), "A");
}

/// `SET (n).name = 'x'` -- a parenthesized-variable target, same meaning
/// as bare `n.name` (TCK's Set1 [3]/[4]).
#[test]
fn set_parenthesized_target() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)");
    let result = run(&store, "MATCH (n:A) SET (n).name = 'neo4j' RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "neo4j");
}

/// `CREATE (a {...}) SET a.prop = ...` -- CREATE followed directly by
/// another mutating clause, no `WITH` in between (TCK's Set1 [6]/[7]).
/// Also covers a property-sourced list surviving a concat
/// (`a.numbers + [4, 5]`) and a list comprehension over a property
/// access (`[i IN n.numbers | ...]`, TCK's Set1 [5]).
#[test]
fn create_followed_directly_by_set_no_with() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "CREATE (a {numbers: [1, 2, 3]}) SET a.numbers = a.numbers + [4, 5] RETURN a.numbers",
    );
    let ints: Vec<i64> = match &result.rows[0][0] {
        Value::List(items) => items.iter().map(int).collect(),
        other => panic!("expected a list, got {other:?}"),
    };
    assert_eq!(ints, vec![1, 2, 3, 4, 5]);

    run(&store, "CREATE (:N)");
    let result = run(
        &store,
        "MATCH (n:N) SET n.numbers = [1, 2, 3] RETURN [i IN n.numbers | i / 2.0] AS x",
    );
    let floats: Vec<f64> = match &result.rows[0][0] {
        Value::List(items) => items.iter().map(as_float).collect(),
        other => panic!("expected a list, got {other:?}"),
    };
    assert_eq!(floats, vec![0.5, 1.0, 1.5]);

    // `CREATE ... WITH ...` and standalone `CREATE ... RETURN ...`/bare-
    // `CREATE` shapes must still work -- create_as_clause's lookahead
    // grew wider, not narrower.
    let result = run(
        &store,
        "CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)",
    );
    assert!(result.rows.is_empty());
    let result = run(&store, "CREATE (node) RETURN labels(node)");
    assert_eq!(list_str_values(&result.rows[0][0]), Vec::<String>::new());
}

/// `ON MATCH`/`ON CREATE` are allowed in either order, not just `ON
/// CREATE` before `ON MATCH` -- a repeated one of the same kind is still
/// an error (TCK's Merge4).
#[test]
fn merge_on_match_on_create_either_order() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    run(
        &store,
        "MATCH () MERGE (a:L) ON MATCH SET a:M1 ON CREATE SET a:M2",
    );
    let result = run(&store, "MATCH (a:L) RETURN labels(a)");
    let mut labels = list_str_values(&result.rows[0][0]);
    labels.sort();
    assert_eq!(labels, vec!["L".to_string(), "M2".to_string()]);

    // A repeated ON CREATE is rejected at parse time, not execution.
    assert!(parse("MERGE (a:L) ON CREATE SET a:M1 ON CREATE SET a:M2").is_err());
}

// --- `<mutating-clause> RETURN ...` (SET/DELETE/DETACH DELETE/REMOVE/
// MATCH...CREATE followed directly by a RETURN in the same statement) ---

#[test]
fn set_then_return_sees_the_just_set_value() {
    // materialize_set applies the mutation before materialize_return runs,
    // so RETURN sees the updated property, not the pre-SET one (TCK's
    // Set2.feature scenario [1]).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {property1: 'orig'})");
    let result = run(
        &store,
        "MATCH (n:A) SET n.property1 = 'updated' RETURN n.property1",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "updated");
}

#[test]
fn set_label_then_return_labels() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:X)");
    let result = run(&store, "MATCH (n:X) SET n:Foo RETURN n");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Node(node) => {
            let mut labels = node.labels.clone();
            labels.sort();
            assert_eq!(labels, vec!["Foo".to_string(), "X".to_string()]);
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn delete_then_return_computed_value_not_the_deleted_var() {
    // TCK DELETE+RETURN scenarios (Delete1/Delete4/Delete6) never RETURN
    // the deleted variable's live properties -- only a computed value (a
    // literal, count(*), or a WITH-projected scalar captured before the
    // delete). Expected count is 2, not 1: the undirected pattern matches
    // both directions (Delete4 [1], "Undirected expand ... delete and count").
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A)-[:R]->(b:B)");
    let result = run(
        &store,
        "MATCH (a)-[r]-(b) DELETE r, a, b RETURN count(*) AS c",
    );
    assert_eq!(int_value(&result.rows[0][0]), 2);
    // The delete itself really happened -- nothing left to match.
    let remaining = run(&store, "MATCH (n) RETURN n");
    assert_eq!(remaining.rows.len(), 0);
}

#[test]
fn delete_then_return_the_deleted_var_itself_errors_not_panics() {
    // No TCK scenario tests a bare `RETURN n` here directly, but Return2
    // [15]/[17] tests the property-access cousin (`RETURN n.num` must
    // raise `DeletedEntityAccess`). `binding_to_value`/`lookup_prop` (via
    // `deleted_entity_access`) must turn a gone record into a
    // `QueryError`, not a panic, when the whole node is returned too.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p: 1})");
    let stmt = parse("MATCH (n:A) DELETE n RETURN n").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err}"
    );
    // A failed statement rolls back its whole write transaction
    // (`Executor::execute`'s abort-on-error path).
    let remaining = run(&store, "MATCH (n:A) RETURN n");
    assert_eq!(
        remaining.rows.len(),
        1,
        "a failed statement must roll back, not partially apply its delete"
    );
}

#[test]
fn delete_then_return_a_property_of_the_deleted_var_errors() {
    // TCK Return2 scenarios [15]/[17]: accessing a property of a just-
    // deleted node/relationship must raise DeletedEntityAccess, not
    // silently succeed with the pre-delete value.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n {num: 0})");
    let stmt = parse("MATCH (n) DELETE n RETURN n.num").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err}"
    );

    let store2 = GraphStore::open_memory().unwrap();
    run(&store2, "CREATE ()-[:T {num: 0}]->()");
    let stmt2 = parse("MATCH ()-[r]->() DELETE r RETURN r.num").unwrap();
    let err2 = Executor::new(&store2).execute(&stmt2).unwrap_err();
    assert!(
        err2.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err2}"
    );
}

#[test]
fn count_of_a_deleted_variable_works_property_access_still_errors() {
    // `count(p)` needs only the entity's identity, never its record, so
    // `CREATE (p) WITH p DELETE p RETURN count(p)` answers 1 rather than
    // raising DeletedEntityAccess. Property access on the deleted
    // variable must keep erroring (TCK Return2 [15]/[17], tests above) --
    // only the identity-only count path bypasses `deleted_entity_access`.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "CREATE (p:Product {productID: 999}) WITH p DELETE p RETURN count(p) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 1);

    // Same shape for a relationship variable.
    let result = run(
        &store,
        "CREATE (:S)-[r:T]->(:P) WITH r DELETE r RETURN count(r) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 1);

    // count(DISTINCT <deleted var>) dedups by graph identity, same as it
    // would for live entities.
    run(&store, "CREATE (:D), (:D)");
    let result = run(&store, "MATCH (m:D) DELETE m RETURN count(DISTINCT m) AS c");
    assert_eq!(int(&result.rows[0][0]), 2);

    // The identity-only fast path must not break count()'s null-skipping:
    // an unmatched OPTIONAL MATCH variable still contributes nothing.
    run(&store, "CREATE (:Lonely)");
    let result = run(
        &store,
        "MATCH (n:Lonely) OPTIONAL MATCH (n)-[:NOPE]->(x) RETURN count(x) AS c",
    );
    assert_eq!(int(&result.rows[0][0]), 0);

    // count(p.prop) on a deleted variable touches the record and must
    // still error, same as the bare property access above.
    run(&store, "CREATE (:E {num: 1})");
    let stmt = parse("MATCH (n:E) DELETE n RETURN count(n.num)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("no longer exists"),
        "expected a deleted-entity error, got: {err}"
    );
}

#[test]
fn detach_delete_then_return() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person {name: 'Bob'})",
    );
    let result = run(
        &store,
        "MATCH (n:Person {name: 'Alice'}) DETACH DELETE n RETURN 42 AS num",
    );
    // `42` is a bare literal, not a node/edge property -- eval_return_expr
    // yields Value::Literal here, not Value::Property.
    assert!(matches!(
        &result.rows[0][0],
        Value::Literal(marsdb_query::Literal::Int(42))
    ));
    let remaining = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(remaining.rows.len(), 1);
    assert_eq!(str_value(&remaining.rows[0][0]), "Bob");
}

#[test]
fn optional_match_delete_null_return_null() {
    // TCK Delete1 scenario [5]: "Ignore null when deleting node" -- an
    // OPTIONAL MATCH that finds nothing pads with a null binding, DELETE on
    // null is a documented no-op, and the trailing RETURN of that same
    // (null) variable must round-trip as null, not error.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Real)");
    let result = run(&store, "OPTIONAL MATCH (a:DoesNotExist) DELETE a RETURN a");
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn remove_then_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p1: 1, p2: 2})");
    let result = run(&store, "MATCH (n:A) REMOVE n.p1 RETURN n");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        Value::Node(node) => {
            assert!(!node.props.contains_key("p1"));
            assert_eq!(
                int_value(&Value::Property(node.props.get("p2").unwrap().clone())),
                2
            );
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn match_create_then_return_sees_the_newly_created_binding() {
    // materialize_create threads each row's updated bindings forward, so
    // the trailing RETURN sees `i`, the node CREATE just made.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})");
    let result = run(&store, "MATCH (a:Person {name: 'Alice'}) CREATE (a)-[:OWNS]->(i:Item {name: 'Widget'}) RETURN i.name");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "Widget");
}

#[test]
fn set_then_return_distinct_dedups() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:A)-[:R]->(x:X {tag: 'same'})");
    run(&store, "CREATE (b:A)-[:R]->(y:X {tag: 'same'})");
    let result = run(
        &store,
        "MATCH (a:A)-[:R]->(x:X) SET a.touched = true RETURN DISTINCT x.tag",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "same");
}

#[test]
fn set_then_return_with_param_substitution() {
    // `params::substitute_tail`/`substitute_return_tail`: a `$param`
    // inside the trailing RETURN of a mutating tail must resolve just
    // like one inside the mutating clause itself.
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {p: 1})");
    let mut params = HashMap::new();
    params.insert("newp".to_string(), marsdb_graph::PropertyValue::Int(99));
    let mut stmt = marsdb_query::parse("MATCH (n:A) SET n.p = 2 RETURN $newp AS x").unwrap();
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert!(matches!(
        &result.rows[0][0],
        Value::Literal(marsdb_query::Literal::Int(99))
    ));
    let after = run(&store, "MATCH (n:A) RETURN n.p");
    assert_eq!(int_value(&after.rows[0][0]), 2);
}

#[test]
fn set_property_to_null_removes_it_not_stores_a_null_value() {
    // `SET n.prop = null` must *remove* the property (TCK's Set2 "Set a
    // Property to Null" scenarios), not store a literal
    // `PropertyValue::Null` under that key -- a stored null still shows
    // up when a node's props are enumerated.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:A {property1: 45, property2: 46})");
    let result = run(&store, "MATCH (n:A) SET n.property1 = null RETURN n");
    match &result.rows[0][0] {
        Value::Node(node) => {
            assert!(
                !node.props.contains_key("property1"),
                "property1 must be gone, not null: {:?}",
                node.props
            );
            assert_eq!(
                int_value(&Value::Property(
                    node.props.get("property2").unwrap().clone()
                )),
                46
            );
        }
        other => panic!("expected a node, got {other:?}"),
    }
}

#[test]
fn set_and_remove_on_a_null_binding_are_silent_no_ops() {
    // An OPTIONAL MATCH miss pads its variable with a null binding;
    // SET/REMOVE (property and label forms) on that null must be silent
    // no-ops, same as DELETE already does. TCK's Set1/Set3/Remove1/
    // Remove2 "Ignore null when setting/removing property/label" scenarios.
    let store = GraphStore::open_memory().unwrap();
    let prop_set = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) SET a.num = 42 RETURN a",
    );
    assert!(matches!(prop_set.rows[0][0], Value::Null));
    let label_set = run(&store, "OPTIONAL MATCH (a:DoesNotExist) SET a:L RETURN a");
    assert!(matches!(label_set.rows[0][0], Value::Null));
    let prop_remove = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) REMOVE a.num RETURN a",
    );
    assert!(matches!(prop_remove.rows[0][0], Value::Null));
    let label_remove = run(
        &store,
        "OPTIONAL MATCH (a:DoesNotExist) REMOVE a:L RETURN a",
    );
    assert!(matches!(label_remove.rows[0][0], Value::Null));
}

#[test]
fn create_index_then_lookup_via_index_seek() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE (:Person {email: 'bob@x.com', age: 25})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "MATCH (n:Person {email: 'alice@x.com'}) RETURN n.age",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(int_value(&result.rows[0][0]), 30);
}

#[test]
fn create_index_unique_rejects_duplicate() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'same@x.com'})");
    run(&store, "CREATE (:Person {email: 'same@x.com'})");

    let stmt = parse("CREATE INDEX ON :Person(email) UNIQUE").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("unique"));
}

#[test]
fn create_with_bracketless_arrow_requires_a_relationship_type() {
    // Unlike MATCH (where an untyped hop means "any relationship"),
    // CREATE always makes exactly one new relationship and requires an
    // explicit `:TYPE`, true for `-->` same as `-[]->` (TCK's Create2
    // [18], "Fail when creating a relationship without a type").
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("CREATE (:A)-->(:B)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn create_with_a_typed_relationship_still_works() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A)-[:REL]->(:B)");
    let out = run(&store, "MATCH ()-[:REL]->() RETURN count(*)");
    assert_eq!(out.rows.len(), 1);
    assert_eq!(int_value(&out.rows[0][0]), 1);
}

#[test]
fn set_property_to_a_computed_expression() {
    // `SetItem::Prop`'s RHS accepts arithmetic/property-read/function-call
    // expressions, not just literals.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:A {name: 'Andres'})");
    let result = run(
        &store,
        "MATCH (n:A) SET n.name = n.name + ' was here' RETURN n.name",
    );
    assert_eq!(str_value(&result.rows[0][0]), "Andres was here");

    run(&store, "CREATE (:B {x: 10})");
    let result2 = run(&store, "MATCH (n:B) SET n.y = n.x * 2 RETURN n.y");
    assert_eq!(int_value(&result2.rows[0][0]), 20);
}

#[test]
fn set_property_to_a_computed_null_removes_it() {
    // Null-removes-property is a runtime fact about the evaluated value,
    // not a check against the `Literal::Null` AST token -- `SET n.prop =
    // coalesce(null, null)` must remove the property too.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:C {keep: 1, drop: 5})");
    let result = run(
        &store,
        "MATCH (n:C) SET n.drop = coalesce(null, null) RETURN n",
    );
    let Value::Node(node) = &result.rows[0][0] else {
        panic!("expected a node");
    };
    assert!(!node.props.contains_key("drop"));
    assert_eq!(
        node.props.get("keep"),
        Some(&marsdb_graph::PropertyValue::Int(1))
    );
}

#[test]
fn set_property_to_a_param_still_works() {
    // Widening `SetItem::Prop`'s RHS to a general `ReturnExpr` must not
    // break `$param` substitution, the more common SET shape.
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:D {n: 1})");
    let stmt = parse("MATCH (n:D) SET n.n = $v RETURN n.n").unwrap();
    let mut stmt = stmt;
    let mut params = HashMap::new();
    params.insert("v".to_string(), marsdb_graph::PropertyValue::Int(99));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int_value(&result.rows[0][0]), 99);
}

#[test]
fn delete_target_can_be_a_map_value_access() {
    // DELETE targets accept any expression evaluating to a node/
    // relationship/path, not just bare identifiers.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:User)");
    run(&store, "CREATE (:User)");
    run(
        &store,
        "MATCH (u:User) WITH {key: u} AS nodes DELETE nodes.key",
    );
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_target_can_be_a_list_index() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:User)-[:FRIEND]->(:A)");
    run(&store, "CREATE (:User)-[:FRIEND]->(:B)");
    run(
        &store,
        "MATCH (:User)-[:FRIEND]->(n) WITH collect(n) AS friends DETACH DELETE friends[0]",
    );
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 3);
}

#[test]
fn detach_delete_whole_named_path() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:X)-[:R]->()-[:R]->()-[:R]->()");
    run(&store, "MATCH p = (:X)-->()-->()-->() DETACH DELETE p");
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_arithmetic_expression_is_a_semantic_error() {
    // Structurally can never be a node/relationship/path (unlike
    // `map.key`, below), so it's rejected at semantic-validation time,
    // independent of whether MATCH finds any rows (TCK's Delete5 [9]).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    let stmt = parse("MATCH (n) DELETE 1 + 1").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn delete_a_non_graph_map_value_is_a_runtime_type_error() {
    // A map/property access types as Kind::Scalar structurally (it might
    // hold a node at runtime, e.g. `nodes.key` elsewhere in this file) so
    // it passes semantic validation; only delete_value, once the actual
    // value is in hand, catches this one being a plain Int.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()");
    let stmt = parse("MATCH (n) WITH {key: 5} AS m DELETE m.key").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("type error:"));
}

#[test]
fn delete_multiple_bare_variables_across_two_rows_of_the_same_edge() {
    // A bare-variable DELETE target must use the raw row Binding (no
    // existence check), not full expression evaluation -- otherwise the
    // second of two rows referencing an already-deleted-by-the-first-row
    // entity would error instead of silently deduping (TCK's Delete4 [1]).
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:R]->()");
    let result = run(
        &store,
        "MATCH (a)-[r]-(b) DELETE r, a, b RETURN count(*) AS c",
    );
    assert_eq!(int_value(&result.rows[0][0]), 2);
    let count = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 0);
}

#[test]
fn delete_multiple_path_targets_deletes_all_edges_before_any_node() {
    // TCK Delete5 [7]: two DELETE targets in one (non-DETACH) clause can
    // each hold one of a shared node's two edges -- deleting the first
    // target's node inline (before the second target's edge is gone)
    // would wrongly fail with "node has incident edges".
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:User), (b:User) CREATE (a)-[:R]->(b), (b)-[:R]->(a)",
    );
    run(
        &store,
        "MATCH p = (:User)-[r]->(:User) \
         WITH {key: collect(p)} AS pathColls \
         DELETE pathColls.key[0], pathColls.key[1]",
    );
    let nodes = run(&store, "MATCH (n) RETURN count(n)");
    assert_eq!(int_value(&nodes.rows[0][0]), 0);
}

/// `MERGE p = ...` -- named-path capture on MERGE, both when the pattern
/// is created fresh and when it's found by an ordinary match. TCK's
/// Merge1 [13], Merge5 [10].
#[test]
fn merge_named_path_capture_on_create_and_on_match() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "MERGE p = (a {num: 1}) RETURN p");
    match &result.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 1),
        other => panic!("expected a Path, got {other:?}"),
    }

    run(&store, "MERGE (a {num: 1}) MERGE (b {num: 2})");
    let result = run(
        &store,
        "MATCH (a {num: 1}), (b {num: 2}) MERGE p = (a)-[:R]->(b) RETURN p",
    );
    match &result.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 3),
        other => panic!("expected a Path, got {other:?}"),
    }
    // Re-running the same MERGE finds the just-created relationship
    // instead of creating a second one -- the found branch must also
    // capture the path, not just the create branch.
    let again = run(
        &store,
        "MATCH (a {num: 1}), (b {num: 2}) MERGE p = (a)-[:R]->(b) RETURN p",
    );
    match &again.rows[0][0] {
        Value::Path(elems) => assert_eq!(elems.len(), 3),
        other => panic!("expected a Path, got {other:?}"),
    }
    let count = run(&store, "MATCH ()-[r:R]->() RETURN count(r)");
    assert_eq!(int(&count.rows[0][0]), 1);
}

/// `MATCH ()-[r]->() DELETE r RETURN type(r)` -- a relationship's type
/// never changes, so reading it after the relationship is deleted in the
/// same statement is allowed, unlike labels()/property access
/// (DeletedEntityAccess). TCK's Return2 [14].
#[test]
fn type_of_a_deleted_relationship_still_works() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T]->()");
    let result = run(&store, "MATCH ()-[r]->() DELETE r RETURN type(r)");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "T"),
        other => panic!("expected a string, got {other:?}"),
    }

    // Properties of a deleted relationship still correctly error.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:T {num: 0}]->()");
    let stmt = parse("MATCH ()-[r]->() DELETE r RETURN r.num").unwrap();
    assert!(Executor::new(&store).execute(&stmt).is_err());
}

/// `SET r.prop = x` / `REMOVE r.prop` on a *relationship* variable --
/// every other SET/REMOVE test targets a node, leaving
/// `GraphStore::set_edge_prop_in_txn`/`remove_edge_prop_in_txn` untested.
#[test]
fn set_and_remove_relationship_properties() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ()-[:KNOWS]->()");

    let result = run(
        &store,
        "MATCH ()-[r:KNOWS]->() SET r.since = 2020 RETURN r.since",
    );
    assert_eq!(int_value(&result.rows[0][0]), 2020);

    let result = run(
        &store,
        "MATCH ()-[r:KNOWS]->() REMOVE r.since RETURN r.since",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
}
