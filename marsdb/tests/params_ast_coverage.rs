//! `substitute_params` (`marsdb-query/src/params.rs`) walks the *entire*
//! parsed AST looking for `$name` placeholders, regardless of whether a
//! given query actually uses any -- so its per-expression-shape/per-
//! clause-shape branches only get covered by a query that contains that
//! shape and goes through `execute_with_params`/`execute_with_options`
//! (the only entry points that call it; `marsdb-query`'s own crate-level
//! smoke tests call `Executor::execute` directly and never touch this
//! module at all). These smoke-test the AST shapes params.rs handles
//! that no other `marsdb`-crate test happens to exercise via that path.

use std::collections::HashMap;

use marsdb::Database;

fn run(db: &Database, cypher: &str) -> marsdb::QueryResult {
    db.execute_with_params(cypher, &HashMap::new())
        .unwrap_or_else(|e| panic!("execute_with_params failed for {cypher:?}: {e}"))
}

/// A single `RETURN` touching most `ReturnExpr` shapes `substitute_params`
/// recurses into: slice, list comprehension, quantifier, map literal,
/// AND/OR/XOR/NOT, comparison, IS NULL, IN.
#[test]
fn substitute_params_walks_every_return_expr_shape() {
    let db = Database::in_memory().unwrap();
    run(
        &db,
        "RETURN [1, 2, 3][0..2] AS slice, \
                [x IN [1, 2, 3] WHERE x > 1 | x] AS comp, \
                any(x IN [1, 2, 3] WHERE x > 1) AS quant, \
                {a: 1, b: 2} AS map, \
                (true AND false) AS a, (true OR false) AS o, (true XOR false) AS x, \
                NOT true AS n, \
                (1 = 1) AS eq, (1 IS NULL) AS isnull, (1 IN [1, 2, 3]) AS in_list",
    );
}

/// `WITH ... WHERE` (`substitute_with_expr`'s own `WithExpr` shape,
/// distinct from `Expr` used by `MATCH`'s `WHERE`): AND/OR/NOT/compare/
/// IS NULL/bare-boolean, all in one chain.
#[test]
fn substitute_params_walks_with_where_expr_shapes() {
    let db = Database::in_memory().unwrap();
    run(
        &db,
        "MATCH (n) WITH n WHERE (n.age > 0 AND n.age < 200) OR NOT (n.age IS NULL) \
         RETURN n",
    );
    run(&db, "MATCH (n) WITH n WHERE n.active RETURN n");
}

/// Mid-query SET/DELETE/CREATE (`QueryClause::Set`/`Delete`/`Create`,
/// chained directly into another updating clause -- see
/// `marsdb-query/tests/smoke_explain.rs`'s grammar note on why a plain
/// `MATCH` can't follow).
#[test]
fn substitute_params_walks_mid_query_set_delete_create() {
    let db = Database::in_memory().unwrap();
    run(&db, "CREATE (:Person {name: 'a'}), (:Person {name: 'b'})");
    run(
        &db,
        "MATCH (n:Person {name: 'a'}) SET n.age = 1 MERGE (x:X) RETURN n, x",
    );
    run(
        &db,
        "MATCH (n:Person {name: 'b'}) DELETE n MERGE (x:X) RETURN x",
    );
    run(
        &db,
        "MATCH (n:Person) CREATE (m:Friend {of: n.name}) MERGE (x:X) RETURN n, m, x",
    );
}

/// `MERGE ... ON CREATE SET ... ON MATCH SET ...` -- `substitute_merge_clause`'s
/// own `on_create`/`on_match` item lists.
#[test]
fn substitute_params_walks_merge_on_create_on_match() {
    let db = Database::in_memory().unwrap();
    run(
        &db,
        "MERGE (n:Counter {id: 1}) ON CREATE SET n.hits = 1 ON MATCH SET n.hits = n.hits + 1 \
         RETURN n.hits",
    );
}

/// In-query `CALL proc(args) YIELD ...` -- `substitute_call_clause`.
#[test]
fn substitute_params_walks_in_query_call_args() {
    let db = Database::in_memory().unwrap();
    run(
        &db,
        "MATCH (n:Person) CALL db.labels() YIELD label RETURN n, label",
    );
}

/// A pattern comprehension in `RETURN` position --
/// `ReturnExpr::PatternComprehension`'s own pattern/where/projection
/// substitution.
#[test]
fn substitute_params_walks_pattern_comprehension() {
    let db = Database::in_memory().unwrap();
    run(
        &db,
        "CREATE (a:Person {name: 'a'})-[:KNOWS]->(b:Person {name: 'b'})",
    );
    let result = run(
        &db,
        "MATCH (n:Person {name: 'a'}) RETURN [(n)-[:KNOWS]->(m) WHERE m.name <> 'x' | m.name]",
    );
    assert_eq!(result.rows.len(), 1);
}
