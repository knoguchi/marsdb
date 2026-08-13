//! Smoke tests for `EXPLAIN`'s one-line-per-clause formatting
//! (`explain.rs`) -- the clause kinds that never compile to a
//! `LogicalPlan` (CALL, SET, DELETE, REMOVE, UNWIND, MERGE, WITH *) and
//! so are otherwise only exercised indirectly through plan-shape tests
//! in smoke_expressions.rs/smoke_matching.rs.
//!
//! Grammar note: `singlePartQ` is `readingStatement* (returnSt |
//! updatingStatement+ returnSt?)` -- reading clauses (MATCH/UNWIND/
//! in-query CALL) must all come *before* any updating clause (CREATE/
//! MERGE/DELETE/SET/REMOVE), and updating clauses must be contiguous.
//! So `MATCH ... SET ... MATCH ... RETURN` doesn't parse; chaining a
//! second updating clause (as the "mutating clauses chain directly"
//! tests elsewhere in this crate do) means following with MERGE, not
//! another MATCH.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::parse;

/// The in-query `CALL ... YIELD` form (`QueryClause::Call`, a
/// `readingStatement` -- distinct from `Statement::StandaloneCall`,
/// which `explainSt`'s grammar never wraps, see below). Its `YIELD`
/// items must extend `carried_vars` so a later clause referencing them
/// plans correctly, and both a bare and an aliased yield item are
/// exercised.
#[test]
fn explain_in_query_call_with_args_extends_carried_vars() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:N) CALL test.proc(n.x) YIELD a, b AS c RETURN n.x, a, c",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("CALL test.proc(") && l.contains("YIELD a, b AS c")));
    assert!(lines.iter().any(|l| l.starts_with("MATCH")));
    assert!(lines.iter().any(|l| l.starts_with("RETURN")));
}

/// `EXPLAIN CALL ...` (a standalone call, `Statement::StandaloneCall`)
/// is rejected at the grammar level -- `explainSt` only ever wraps
/// `createIndexSt | regularQuery`, never `standaloneCall`. So
/// `explain_statement`'s own `Statement::StandaloneCall(call) => ...`
/// arm (which reuses the same `explain_call_clause` helper as the
/// in-query form above) is unreachable through the public parser.
#[test]
fn explain_standalone_call_is_rejected_at_parse_time() {
    let err = parse("EXPLAIN CALL db.labels()").unwrap_err();
    assert!(err.to_string().starts_with("syntax error:"));
}

/// SET chained directly into another updating clause
/// (`QueryClause::Set`, not the final-tail `Tail::Set`) -- all three
/// `SetItem` shapes in one clause: property assignment, label
/// assignment, and a map assign (both plain and merging `+=`).
#[test]
fn explain_mid_query_set_covers_every_set_item_shape() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) SET n.age = 1, n:Employee, n = {a: 1}, n += {b: 2} MERGE (x:X) RETURN n, x",
    ));
    assert!(lines.iter().any(|l| l.starts_with("SET")
        && l.contains("n.age")
        && l.contains("n:Employee")
        && l.contains("n +=")));
}

/// DELETE / DETACH DELETE chained directly into another updating clause
/// (`QueryClause::Delete`).
#[test]
fn explain_mid_query_delete_and_detach_delete() {
    let store = GraphStore::open_memory().unwrap();

    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (a:A) DELETE a MERGE (a2:A) RETURN a2",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("DELETE") && l.contains('a')));

    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (a:A) DETACH DELETE a MERGE (a2:A) RETURN a2",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("DETACH DELETE") && l.contains('a')));
}

/// REMOVE chained directly into another updating clause
/// (`QueryClause::Remove`) -- both `RemoveItem` shapes: a property and
/// a label.
#[test]
fn explain_mid_query_remove_covers_both_item_shapes() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) REMOVE n.age, n:Employee MERGE (x:X) RETURN n, x",
    ));
    assert!(lines.iter().any(|l| l == "REMOVE n.age, n:Employee"));
}

/// CREATE chained directly into another updating clause
/// (`QueryClause::Create`, not the final-tail `Tail::Create`).
#[test]
fn explain_mid_query_create() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) CREATE (m:Friend) MERGE (m)-[:X]->(n) RETURN n, m",
    ));
    assert!(lines.iter().any(|l| l == "CREATE (1 pattern)"));
}

/// Every mutating `Tail` shape as the statement's *final* clause (no
/// RETURN at all) -- a distinct formatting path (`explain_tail`) from
/// the mid-query `QueryClause::*` cases above.
#[test]
fn explain_tail_mutations_with_no_return() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'a'})");

    assert!(
        plan_lines(&run(&store, "EXPLAIN MATCH (n:Person) DELETE n"))
            .iter()
            .any(|l| l.starts_with("DELETE"))
    );
    assert!(
        plan_lines(&run(&store, "EXPLAIN MATCH (n:Person) DETACH DELETE n"))
            .iter()
            .any(|l| l.starts_with("DETACH DELETE"))
    );
    assert!(
        plan_lines(&run(&store, "EXPLAIN MATCH (n:Person) SET n.age = 1"))
            .iter()
            .any(|l| l.starts_with("SET"))
    );
    assert!(plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) REMOVE n.age, n:Employee"
    ))
    .iter()
    .any(|l| l == "REMOVE n.age, n:Employee"));
    assert!(
        plan_lines(&run(&store, "EXPLAIN MATCH (n:Person) CREATE (m:Friend)"))
            .iter()
            .any(|l| l == "CREATE (1 pattern)")
    );
}

/// `WITH *` mid-query -- exercises `explain_with_projection`'s star
/// expansion (every carried + newly bound var, deduped and sorted).
#[test]
fn explain_mid_query_with_star() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) WITH * MATCH (m:Person) RETURN n, m",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("WITH") && l.contains('n')));
}

/// A plain (non-star) `WITH` mid-query, with a computed alias --
/// `explain_with_projection`'s non-star path plus `with_columns`.
#[test]
fn explain_mid_query_with_plain_projection() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) WITH n AS x, count(*) AS c MATCH (m:Person) RETURN x, c, m",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("WITH") && l.contains('x') && l.contains('c')));
}

#[test]
fn explain_unwind_clause() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(&store, "EXPLAIN UNWIND [1, 2, 3] AS x RETURN x"));
    assert!(lines.iter().any(|l| l == "UNWIND ... AS x"));
}

#[test]
fn explain_merge_clause() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (a:Person) MERGE (a)-[:KNOWS]->(b:Person) RETURN a, b",
    ));
    assert!(lines
        .iter()
        .any(|l| l.starts_with("MERGE (match-or-create")));
}

#[test]
fn explain_create_index_unique() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN CREATE INDEX ON :Person(email) UNIQUE",
    ));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("UNIQUE"));
}

/// `EXPLAIN EXPLAIN ...` is rejected at the grammar level (`explainSt`
/// never wraps another `explainSt`) -- `explain_statement`'s own
/// `Statement::Explain(_) => Err(...)` arm is unreachable through the
/// public parser and exists only as a defensive match-exhaustiveness
/// guard, so this only asserts the (syntax-level) rejection, not that
/// runtime arm.
#[test]
fn explain_explain_is_rejected_at_parse_time() {
    let err = parse("EXPLAIN EXPLAIN MATCH (n) RETURN n").unwrap_err();
    assert!(err.to_string().starts_with("syntax error:"));
}

#[test]
fn explain_in_query_call_without_yield() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:N) CALL test.proc() RETURN n",
    ));
    assert!(lines.iter().any(|l| l == "CALL test.proc()"));
}

/// A `WITH` that starts the whole clause list (nothing precedes it to
/// attach onto) goes through `QueryClause::With` directly, unlike a
/// `WITH` immediately following a `MATCH` pattern (which attaches to
/// that match part's own trailing `with` field instead -- see
/// `explain_mid_query_with_star`/`_plain_projection` above).
#[test]
fn explain_leading_with_clause() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN WITH 1 AS x MATCH (n:Person) RETURN x, n",
    ));
    assert!(lines.iter().any(|l| l == "WITH x"));
}

/// Residual-`Filter` expression formatting: a WHERE predicate too
/// varied to become an `IndexSeek` exercises `format_expr`'s AND/OR/NOT/
/// IS NULL/comparison-operator arms all in one plan.
#[test]
fn explain_filter_formats_boolean_and_comparison_operators() {
    let store = GraphStore::open_memory().unwrap();
    let explained = run(
        &store,
        "EXPLAIN MATCH (n:Person) WHERE n.age <> 1 AND n.age < 2 AND n.age <= 3 \
         AND n.age >= 4 AND n.name STARTS WITH 'a' AND n.name ENDS WITH 'b' \
         AND n.name CONTAINS 'c' AND NOT n.flag AND n.other IS NULL RETURN n",
    );
    let lines = plan_lines(&explained);
    let filter = lines
        .iter()
        .find(|l| l.trim_start().starts_with("Filter"))
        .unwrap_or_else(|| panic!("expected a residual Filter line, got: {lines:?}"));
    for needle in [
        "AND",
        "<>",
        "<",
        "<=",
        ">=",
        "STARTS WITH",
        "ENDS WITH",
        "CONTAINS",
        "NOT",
        "IS NULL",
    ] {
        assert!(
            filter.contains(needle),
            "expected Filter line to contain {needle:?}, got: {filter}"
        );
    }

    let or_lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (n:Person) WHERE n.age > 1 OR n.age < 0 RETURN n",
    ));
    assert!(or_lines.iter().any(|l| l.contains("OR")));
}

#[test]
fn explain_var_length_expand_without_bound_rel_list() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (a:Person)-[:KNOWS*1..3]->(b:Person) RETURN a, b",
    ));
    assert!(lines.iter().any(|l| l.contains("VarExpand")));
}

/// `MatchRelList` (a deterministic "verify the chain" plan, distinct
/// from `VarExpand`'s fresh BFS) only fires when the relationship-list
/// variable is *already* bound before this hop -- a fresh `[r*1..3]`
/// capture always plans as `VarExpand` (see the test above); re-matching
/// `r` through a second variable-length hop is what triggers it.
#[test]
fn explain_var_length_expand_with_bound_rel_list() {
    let store = GraphStore::open_memory().unwrap();
    let lines = plan_lines(&run(
        &store,
        "EXPLAIN MATCH (a:Person)-[r:KNOWS*1..3]->(b:Person) WITH r \
         MATCH (x:Person)-[r*1..3]->(y:Person) RETURN r",
    ));
    assert!(
        lines.iter().any(|l| l.contains("MatchRelList")),
        "expected a MatchRelList line, got: {lines:?}"
    );
}
