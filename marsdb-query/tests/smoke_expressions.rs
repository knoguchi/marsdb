//! Smoke tests: scalar/list/map expressions, builtins, CASE, params, error paths -- split from the original smoke.rs.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

/// Real Cypher's two comment forms (`//` line, `/* */` block) -- a real
/// grammar gap found via the openCypher TCK's own fixture text, which
/// pervasively annotates `CREATE` blocks this way.
#[test]
fn line_and_block_comments_are_ignored() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:A {num: 1}), //first node\n(:A {num: 2}) // second node",
    );
    let result = run(
        &store,
        "/* leading */ MATCH (a:A) // trailing\nRETURN a.num /* mid-expr */ + 0 ORDER BY a.num",
    );
    let nums: Vec<i64> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(nums, vec![1, 2]);
}

#[test]
fn coalesce_returns_first_non_null() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (a:Post {content: 'hello', imageFile: 'ignored.png'})",
    );
    run(&store, "CREATE (b:Post {imageFile: 'pic.png'})"); // no content prop

    let result = run(
        &store,
        "MATCH (n:Post) RETURN coalesce(n.content, n.imageFile) AS x",
    );
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["hello".to_string(), "pic.png".to_string()]);
}

#[test]
fn to_integer_parses_string_and_passes_through_int() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {code: '42'})");
    let result = run(&store, "MATCH (n:Person) RETURN toInteger(n.code) AS x");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 42),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn to_integer_parses_a_float_formatted_string_by_truncating() {
    // Regression: `toInteger('1.7')` used to fail straight to null since
    // the string-parse path only ever tried an i64 parse.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [2, 2.9, '1.7'] AS things RETURN [n IN things | toInteger(n)] AS x",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 2, 1]);
}

#[test]
fn to_integer_on_an_unparseable_string_is_null_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH ['2', '2.9', 'foo'] AS numbers RETURN [n IN numbers | toInteger(n)] AS x",
    );
    match &result.rows[0][0] {
        Value::List(items) => {
            assert_eq!(int(&items[0]), 2);
            assert_eq!(int(&items[1]), 2);
            assert!(matches!(items[2], Value::Null));
        }
        other => panic!("expected a List, got {other:?}"),
    }
}

#[test]
fn to_integer_on_a_list_errors_instead_of_silently_nulling() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN toInteger([1, 2])").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("tointeger"));
}

#[test]
fn case_when_then_else() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {age: 30})");
    run(&store, "CREATE (b:Person {age: 17})");
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE n.age WHEN 30 THEN 'thirty' ELSE 'other' END AS x",
    );
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["other".to_string(), "thirty".to_string()]);
}

#[test]
fn case_null_equals_null_is_true_not_standard_three_valued_logic() {
    // Documents the deliberate convention CASE relies on for IS7: a missing
    // property compared against `null` in a WHEN arm matches.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice'})"); // no `age` prop
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE n.age WHEN null THEN 'yes' ELSE 'no' END AS x",
    );
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "yes"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Real Cypher's "searched CASE" form -- no subject expression, each
/// `WHEN` carries its own full boolean condition (`CASE WHEN cond THEN
/// ... END`), distinct from the "simple CASE" form both other `case_*`
/// tests exercise (`CASE x WHEN v THEN ... END`). A bare `WHEN` right
/// after `CASE` used to get swallowed as a bare-identifier subject
/// expression (pest's `?` doesn't backtrack across a later parse
/// failure), rejecting every searched-CASE query.
#[test]
fn case_searched_form_has_no_subject_expression() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {age: 30})");
    run(&store, "CREATE (b:Person {age: 17})");
    let result = run(
        &store,
        "MATCH (n:Person) RETURN CASE WHEN n.age >= 18 THEN 'adult' ELSE 'minor' END AS x",
    );
    let mut values: Vec<String> = result
        .rows
        .iter()
        .map(|row| match &row[0] {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    values.sort();
    assert_eq!(values, vec!["adult".to_string(), "minor".to_string()]);
}

#[test]
fn integer_arithmetic_overflow_returns_errors_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    for cypher in [
        "RETURN 9223372036854775807 + 1",
        "RETURN -9223372036854775808 - 1",
        "RETURN 9223372036854775807 * 2",
        "RETURN -9223372036854775808 / -1",
        "RETURN -9223372036854775808 % -1",
    ] {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(err.to_string().contains("overflow"), "{cypher}: {err}");
    }
}

#[test]
fn execution_options_enforce_rows_expansions_cancellation_and_timeout() {
    use std::time::Duration;

    use marsdb_query::{CancellationToken, ExecutionOptions, QueryError};

    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Item {id: 1})-[:NEXT]->(:Item {id: 2})-[:NEXT]->(:Item {id: 3})",
    );
    let executor = Executor::new(&store);

    let scan = parse("MATCH (n:Item) RETURN n").unwrap();
    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                max_intermediate_rows: Some(2),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                max_intermediate_rows: Some(10),
                max_result_rows: Some(2),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let expand = parse("MATCH (n:Item)-[:NEXT]->(m:Item) RETURN m").unwrap();
    let err = executor
        .execute_with_options(
            &expand,
            &ExecutionOptions {
                max_relationship_expansions: Some(1),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::ResourceLimit(_)));

    let token = CancellationToken::new();
    token.cancel();
    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                cancellation_token: Some(token),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::Cancelled));

    let err = executor
        .execute_with_options(
            &scan,
            &ExecutionOptions {
                timeout: Some(Duration::ZERO),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, QueryError::Timeout));
}

#[test]
fn semantic_validation_rejects_invalid_names_and_structural_types() {
    let store = GraphStore::open_memory().unwrap();
    for (cypher, expected) in [
        ("RETURN missing", "undefined variable 'missing'"),
        (
            "WITH 1 AS x MATCH (x)-[:R]->(n) RETURN n",
            "node pattern requires a node",
        ),
        (
            "MATCH ()-[r:R]->() SET r:Label",
            "SET label target requires a node",
        ),
        (
            "MATCH (n) WITH n AS kept RETURN n",
            "undefined variable 'n'",
        ),
        (
            "MATCH (n) WITH n AS x, n AS x RETURN x",
            "duplicate variable 'x'",
        ),
    ] {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().contains("semantic error") && err.to_string().contains(expected),
            "{cypher}: expected {expected:?}, got {err}"
        );
    }
}

#[test]
fn min_max_on_non_orderable_errors() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n:Item {idx: 1})");
    let stmt = parse("MATCH (n:Item) RETURN min(n) AS m").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("comparable"),
        "expected a comparability error, got: {err}"
    );
}

/// `max()`/`min()` over a list argument -- real Cypher orders a list
/// element-by-element (reusing the same `list_cmp_asc` ORDER BY
/// already uses), found as a real bug: `comparable_ordering` had no
/// `List` arm at all, so any list argument unconditionally errored
/// "requires a comparable scalar argument". Aggregation2 [9]/[10].
#[test]
fn max_min_over_list_values() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [[1], [2], [2, 1]] AS x RETURN max(x), min(x)",
    );
    match (&result.rows[0][0], &result.rows[0][1]) {
        (Value::List(max), Value::List(min)) => {
            assert_eq!(max.len(), 2); // [2, 1]
            assert_eq!(min.len(), 1); // [1]
        }
        other => panic!("expected two lists, got {other:?}"),
    }
}

/// `max()`/`min()` over genuinely mixed types (numbers, strings, a
/// list) -- real Cypher's cross-type orderability ranks `List` *below*
/// every scalar (sorts first), the opposite of an earlier, unverified
/// version of `type_rank` that put it last. Aggregation2 [11]/[12].
#[test]
fn max_min_over_mixed_types_including_a_list() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [1, 'a', null, [1, 2], 0.2, 'b'] AS x RETURN max(x), min(x)",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(i)) => assert_eq!(*i, 1),
        other => panic!("expected max to be the int 1, got {other:?}"),
    }
    match &result.rows[0][1] {
        Value::List(items) => assert_eq!(items.len(), 2), // [1, 2]
        other => panic!("expected min to be the list [1, 2], got {other:?}"),
    }
}

#[test]
fn string_literal_escaped_quote_round_trips() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, r"CREATE (:Person {name: 'O\'Brien'})");
    let result = run(&store, "MATCH (n:Person) RETURN n.name");
    assert_eq!(str_value(&result.rows[0][0]), "O'Brien");
}

#[test]
fn string_literal_escaped_backslash_and_common_escapes() {
    // Cypher source (raw string below is literal, no extra doubling
    // needed): `\\` is one escaped backslash, `\t`/`\n` are tab/newline.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        r"CREATE (:Path {p: 'C:\\Users\\x', tab: 'a\tb', nl: 'a\nb'})",
    );
    let result = run(&store, "MATCH (n:Path) RETURN n.p, n.tab, n.nl");
    assert_eq!(str_value(&result.rows[0][0]), r"C:\Users\x");
    assert_eq!(str_value(&result.rows[0][1]), "a\tb");
    assert_eq!(str_value(&result.rows[0][2]), "a\nb");
}

#[test]
fn string_literal_unrecognized_escape_errors() {
    // `\q` isn't one of openCypher.bnf's own closed set of valid escape
    // sequences (backslash/quote/tab/etc/\uXXXX) -- a real syntax error,
    // not just a specific message's wording, which legitimately differs
    // by implementation (a lenient-lex-then-semantic-check parser can
    // give a precise "unrecognized escape" message; a parser whose lexer
    // itself only matches real escape sequences, arguably closer to spec,
    // fails to tokenize the string at all instead -- both are correct
    // rejections of the same invalid input).
    let err = parse(r"MATCH (n {x: 'a\qb'}) RETURN n").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("syntax error"));
}

/// `parse_many`'s `queries` grammar used to have `~ ";"? ~ EOI` at the
/// end -- with a genuinely-trailing `;`, `(";" ~ statement)*` greedily
/// consumed it as one more separator, needing a `statement` after it;
/// since `match_stmt` can match zero-width, an empty string satisfied
/// that, producing a spurious extra empty statement that then failed its
/// own "needs a tail" validation. Caught via the TCK's binary-tree named-
/// graph fixture, a real multi-statement file that ends with `;`.
#[test]
fn parse_many_tolerates_a_trailing_semicolon() {
    assert_eq!(marsdb_query::parse_many("CREATE (a);").unwrap().len(), 1);
    assert_eq!(marsdb_query::parse_many("CREATE (a)").unwrap().len(), 1);
    assert_eq!(
        marsdb_query::parse_many("CREATE (a); CREATE (b);")
            .unwrap()
            .len(),
        2
    );
    // A semicolon inside a string literal must not get mis-split either.
    assert_eq!(
        marsdb_query::parse_many("RETURN ';' AS x;").unwrap().len(),
        1
    );
}

/// `map['key']` -- real Cypher's dynamic map-field access, previously
/// rejected at compile time even though `apply_index`'s runtime already
/// fully supported it. `null['key']` must still be `null`, not an error
/// (a `null`-valued base types as `Kind::Scalar` in this codebase's
/// imprecise `Kind` system, deliberately tolerated here the same way
/// every other `Kind::Scalar` case is).
#[test]
fn map_index_access() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH {a: 1, b: 2} AS m RETURN m['a'] AS value");
    assert_eq!(int(&result.rows[0][0]), 1);

    let result = run(
        &store,
        "WITH null AS expr, 'x' AS idx RETURN expr[idx] AS value",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
}

/// `$1` -- real Cypher's legacy positional-parameter form (a plain
/// non-negative-integer name), not just a `$name` identifier.
#[test]
fn numeric_named_parameters() {
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();
    let mut stmt = parse("RETURN $1 AS x").unwrap();
    let mut params = HashMap::new();
    params.insert("1".to_string(), marsdb_graph::PropertyValue::Int(42));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int(&result.rows[0][0]), 42);
}

/// A list-valued (including nested-list) `$param` -- `Literal` has no
/// list variant (no list-literal *syntax* to substitute one into), so
/// `substitute_params` rewrites the whole `ReturnExpr::Lit(Literal::
/// Param(_))` node into a `ReturnExpr::ListLit` instead, recursively.
/// TCK's List1 [3]/[5], Null3 [4].
#[test]
fn list_valued_parameters_substitute_into_a_list_literal_expression() {
    use std::collections::HashMap;
    let store = GraphStore::open_memory().unwrap();

    let mut stmt = parse("RETURN $coll[1] AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::String("a".into()),
            marsdb_graph::PropertyValue::String("b".into()),
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "b"),
        other => panic!("unexpected value {other:?}"),
    }

    // Three-valued `IN`: a `null` element present, no definite match ->
    // "unknown" (null), not `false`.
    let mut stmt = parse("RETURN 2 IN $coll AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::Int(1),
            marsdb_graph::PropertyValue::Null,
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert!(matches!(&result.rows[0][0], Value::Null));

    // Nested list -- a param list can itself contain lists.
    let mut stmt = parse("RETURN $coll[1][0] AS x").unwrap();
    let mut params = HashMap::new();
    params.insert(
        "coll".to_string(),
        marsdb_graph::PropertyValue::List(vec![
            marsdb_graph::PropertyValue::Int(1),
            marsdb_graph::PropertyValue::List(vec![marsdb_graph::PropertyValue::Int(2)]),
        ]),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    assert_eq!(int(&result.rows[0][0]), 2);
}

/// The executor's per-statement node decode cache (mars-m79) must not
/// leak a stale record across statements when one `Executor` is reused
/// for several (`execute_batch`, group commit) -- a node read in an
/// earlier statement, then mutated in a later one, must show the new
/// value on the next read, not the first statement's cached copy.
#[test]
fn node_cache_does_not_leak_stale_records_across_statements() {
    let store = GraphStore::open_memory().unwrap();
    let executor = Executor::new(&store);

    let create = parse("CREATE (:Item {name: 'old'})").unwrap();
    executor.execute(&create).unwrap();

    // Populates the cache for this node under the first Executor use.
    let read1 = parse("MATCH (n:Item) RETURN n.name").unwrap();
    let result1 = executor.execute(&read1).unwrap();
    match &result1.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "old"),
        other => panic!("unexpected value {other:?}"),
    }

    let update = parse("MATCH (n:Item) SET n.name = 'new'").unwrap();
    executor.execute(&update).unwrap();

    // Same Executor, same node id -- must see the update, not stale
    // cached data from `read1`'s statement.
    let read2 = parse("MATCH (n:Item) RETURN n.name").unwrap();
    let result2 = executor.execute(&read2).unwrap();
    match &result2.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "new"),
        other => panic!("unexpected value {other:?}, cache leaked a stale record"),
    }
}

/// A write statement can intern a brand-new property name mid-statement
/// and then read it back in a later clause of that SAME statement -- the
/// per-property read path's name->id memo (`Executor::prop_id_memo`) must
/// not serve a stale "never interned" for it. This is the exact scenario
/// that forces the memo to be gated to read-only statements (see the
/// field's docs): an earlier clause probes `fresh` before any node has it
/// (memoizing `None` would be tempting), a middle clause creates the
/// first node carrying it, and a later clause filters on it and must
/// match.
#[test]
fn property_interned_mid_statement_is_visible_to_later_clauses_of_the_same_statement() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Seed {n: 1})");

    let result = run(
        &store,
        "MATCH (s:Seed) WHERE s.fresh IS NULL \
         CREATE (:Made {fresh: 42}) \
         WITH s MATCH (m:Made) WHERE m.fresh = 42 RETURN m.fresh",
    );
    assert_eq!(
        result.rows.len(),
        1,
        "the mid-statement write must be visible"
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 42),
        other => panic!("unexpected value {other:?}"),
    }
}

/// `^` (exponentiation) and general unary minus -- TCK's Precedence2
/// "On numeric values" plus Return2 [1]. `^` always produces a `Float`
/// (even for two `Int`s), binds tighter than `*`/`/`/`%`/`+`/`-` but
/// looser than unary minus, and is LEFT-associative (`4^(3*2)^3` is
/// `(4^6)^3`, confirmed against the real TCK fixture -- general math
/// convention's right-associativity would have been wrong here).
#[test]
fn exponentiation_and_unary_minus_precedence() {
    let store = GraphStore::open_memory().unwrap();

    // `^` binds tighter than `*`.
    let result = run(&store, "RETURN 4 ^ 3 * 2 ^ 3 AS a, 4 ^ (3 * 2) ^ 3 AS c");
    assert!((as_float(&result.rows[0][0]) - 512.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - 68719476736.0).abs() < 1e-3);

    // `^` binds tighter than `+`.
    let result = run(&store, "RETURN 4 ^ 3 + 2 ^ 3 AS a, 4 ^ (3 + 2) ^ 3 AS c");
    assert!((as_float(&result.rows[0][0]) - 72.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - 1073741824.0).abs() < 1e-3);

    // Unary minus binds tighter than `^`: `-3^2` is `(-3)^2`, not `-(3^2)`.
    let result = run(&store, "RETURN -3 ^ 2 AS a, -(3 ^ 2) AS c");
    assert!((as_float(&result.rows[0][0]) - 9.0).abs() < 1e-9);
    assert!((as_float(&result.rows[0][1]) - (-9.0)).abs() < 1e-9);

    // A negative numeric literal is unaffected -- still a plain `Literal`,
    // not a `Neg`-wrapped computed value (preserves the planner's
    // index-seek fusion for `MATCH (n {x: -5})`-shaped patterns).
    let result = run(&store, "RETURN -3 AS x");
    assert_eq!(int(&result.rows[0][0]), -3);

    // General unary minus on a non-literal (a bound variable) -- this is
    // the actually-new grammar shape (`-3` alone always worked). Chained
    // unary minus (`--n`) isn't real openCypher -- per
    // openCypher.bnf's `<arithmetic unary> ::= [<sign>] <postfix
    // expression>`, the sign is a single optional, not repeatable.
    let result = run(&store, "WITH 3 AS n RETURN -n AS x");
    assert_eq!(int(&result.rows[0][0]), -3);
}

/// `RETURN 1 AS a, 2 AS a` -- reusing the same explicit alias for two
/// columns is a real error (`ColumnNameConflict`). An unaliased
/// expression repeated (`RETURN date(x), date(y)`) is *not* a
/// conflict, even though both currently fall back to the same generic
/// placeholder column name (`"date(...)"`, not argument-aware) --
/// only a genuinely meaningful name (an alias, or a bare variable/
/// property-access name) can actually collide.
#[test]
fn return_rejects_duplicate_explicit_column_names() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AS a, 2 AS a").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("duplicate"),
        "expected a duplicate-column error, got: {err}"
    );

    let result = run(&store, "RETURN date('2015-07-21'), date('2015-07-22')");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns, vec!["date(...)", "date(...)"]);
}

#[test]
fn list_index_by_position() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN list[0], list[2]",
    );
    assert_eq!(int(&result.rows[0][0]), 1);
    assert_eq!(int(&result.rows[0][1]), 3);
}

#[test]
fn list_index_out_of_bounds_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN list[10], list[-10]");
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
}

#[test]
fn list_slice_basic_and_open_ended() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN list[1..3], list[..2], list[2..]",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 3]);
    assert_eq!(list_ints(&result.rows[0][1]), vec![1, 2]);
    assert_eq!(list_ints(&result.rows[0][2]), vec![3, 4, 5]);
}

#[test]
fn list_comprehension_filter_and_project() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN [x IN list WHERE x % 2 = 0 | x * 10] AS y",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![20, 40]);
}

#[test]
fn list_comprehension_project_only() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3] AS list RETURN [x IN list | x * 2] AS y",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![2, 4, 6]);
}

#[test]
fn list_comprehension_filter_only() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3, 4, 5] AS list RETURN [x IN list WHERE x > 2] AS y",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![3, 4, 5]);
}

#[test]
fn list_comprehension_bare_identity() {
    // No WHERE, no projection -- a legal no-op comprehension.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH [1, 2, 3] AS list RETURN [x IN list] AS y");
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
}

#[test]
fn quantifier_all_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN all(x IN [1, 2, 3] WHERE x > 0) AS a, all(x IN [1, 2, 3] WHERE x > 1) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_any_true_and_false() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN any(x IN [1, 2, 3] WHERE x > 2) AS a, any(x IN [1, 2, 3] WHERE x > 5) AS b",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn quantifier_none_on_empty_list_is_true() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN none(x IN [] WHERE x > 0) AS a");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn quantifier_three_valued_null_propagation() {
    // Regression: a first version collapsed a null predicate straight to
    // false, which happened to pass every non-null-list scenario but was
    // wrong on lists containing nulls -- a definite true/false among the
    // elements still decides the answer even with nulls present; only
    // "no definite answer, but at least one unknown" is null.
    let store = GraphStore::open_memory().unwrap();

    let all = run(&store, "RETURN all(x IN [null] WHERE x = 2) AS a, all(x IN [0, null] WHERE x = 2) AS b, all(x IN [2, null] WHERE x = 2) AS c");
    assert!(matches!(all.rows[0][0], Value::Null));
    assert!(!bool_val(&all.rows[0][1]));
    assert!(matches!(all.rows[0][2], Value::Null));

    let any = run(
        &store,
        "RETURN any(x IN [null] WHERE x = 2) AS a, any(x IN [2, null] WHERE x = 2) AS b",
    );
    assert!(matches!(any.rows[0][0], Value::Null));
    assert!(bool_val(&any.rows[0][1]));

    let none = run(
        &store,
        "RETURN none(x IN [null] WHERE x = 2) AS a, none(x IN [2, null] WHERE x = 2) AS b",
    );
    assert!(matches!(none.rows[0][0], Value::Null));
    assert!(!bool_val(&none.rows[0][1]));

    let single = run(
        &store,
        "RETURN single(x IN [2, null] WHERE x = 2) AS a, single(x IN [34, 0, null, 5, 900] WHERE x < 10) AS b",
    );
    assert!(matches!(single.rows[0][0], Value::Null));
    assert!(!bool_val(&single.rows[0][1]));
}

#[test]
fn map_literal_property_access() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH {existing: 42, notMissing: null} AS m RETURN m.missing, m.notMissing, m.existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);
}

#[test]
fn map_literal_property_access_on_null_is_null() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH null AS m RETURN m.missing");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn boolean_expr_and_or_xor_not() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN true AND false AS a, true OR false AS b, true XOR true AS c, NOT true AS d",
    );
    assert!(!bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(!bool_val(&result.rows[0][2]));
    assert!(!bool_val(&result.rows[0][3]));
}

#[test]
fn boolean_expr_comparison_as_return_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN 1 = 1 AS a, 1 < 2 AS b, 2 > 3 AS c, 'ab' STARTS WITH 'a' AS d",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(!bool_val(&result.rows[0][2]));
    assert!(bool_val(&result.rows[0][3]));
}

#[test]
fn boolean_expr_precedence_and_binds_tighter_than_or() {
    let store = GraphStore::open_memory().unwrap();
    // AND binds tighter than OR: false AND false = false, then true OR
    // false = true -- if OR bound tighter this would instead need to
    // evaluate (true OR false) AND false = false.
    let result = run(&store, "RETURN true OR false AND false AS x");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn boolean_expr_not_binds_looser_than_comparison() {
    let store = GraphStore::open_memory().unwrap();
    // NOT (1 = 2), not (NOT 1) = 2 -- comparison binds tighter.
    let result = run(&store, "RETURN NOT 1 = 2 AS x");
    assert!(bool_val(&result.rows[0][0]));
}

#[test]
fn boolean_expr_three_valued_null_propagation() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN null AND false AS a, null AND true AS b, null OR true AS c, null OR false AS d",
    );
    assert!(!bool_val(&result.rows[0][0])); // false wins over unknown
    assert!(matches!(result.rows[0][1], Value::Null));
    assert!(bool_val(&result.rows[0][2])); // true wins over unknown
    assert!(matches!(result.rows[0][3], Value::Null));
}

#[test]
fn boolean_expr_non_bool_operand_errors() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN 1 AND true").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("boolean"));
}

#[test]
fn list_equality_is_structural_not_null() {
    // Regression: compare_values used to reduce List/Map operands
    // through value_to_property_value, which collapses both to
    // PropertyValue::Null -- every list/map `=`/`<>` comparison silently
    // became `null` regardless of actual content.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN [1, 2] = [1, 2] AS a, [1, 2] = [1, 3] AS b, [null] = [1] AS c",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
}

#[test]
fn list_ordering_is_lexicographic() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN [1, 0] >= [1] AS a, [1, null] >= [1] AS b, [1, 2] >= [1, null] AS c",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(bool_val(&result.rows[0][1]));
    assert!(matches!(result.rows[0][2], Value::Null));
}

#[test]
fn boolean_ordering_false_less_than_true() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN false <= true AS x, false > true AS y");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

#[test]
fn is_null_return_expr() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN null IS NULL AS a, 1 IS NULL AS b, 1 IS NOT NULL AS c",
    );
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
    assert!(bool_val(&result.rows[0][2]));
}

#[test]
fn chained_comparisons_fold_into_and() {
    // `1 < x < 10` means `(1 < x) AND (x < 10)`, not `(1 < x) < 10`.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "WITH 5 AS x RETURN 1 < x < 10 AS a, 1 < x < 3 AS b");
    assert!(bool_val(&result.rows[0][0]));
    assert!(!bool_val(&result.rows[0][1]));
}

/// `IS [NOT] NULL` binds *tighter* than a surrounding comparison -- `false
/// = true IS NULL` is `false = (true IS NULL)`, not `(false = true) IS
/// NULL`. Real Cypher's own precedence rule (TCK's Precedence1 [8]/[23]).
#[test]
fn is_null_binds_tighter_than_comparison() {
    let store = GraphStore::open_memory().unwrap();
    // true IS NULL == false, so false = false == true.
    let result = run(&store, "RETURN false = true IS NULL AS a");
    assert!(bool_val(&result.rows[0][0]));

    // Both sides describe the same precedence via different groupings --
    // must agree regardless of operator or null-ness.
    let result = run(
        &store,
        "WITH 1 AS a, null AS b \
         RETURN (a = b IS NULL) = (a = (b IS NULL)) AS eq",
    );
    assert!(bool_val(&result.rows[0][0]));
}

/// `x IN list` -- real Cypher's list membership test, previously
/// unsupported as a general expression (only existed inside a list
/// comprehension/quantifier's own `filter_expr`). Three-valued like `=`,
/// and binds *tighter* than a surrounding comparison, same precedence
/// tier as `IS NULL` (TCK's Precedence3 [6]: `[1,2] = [3,4] IN
/// [[3,4],false]` is `[1,2] = ([3,4] IN [[3,4],false])`).
#[test]
fn in_operator_list_membership_and_precedence() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "RETURN 3 IN [1, 2, 3] AS r");
    assert!(bool_val(&result.rows[0][0]));

    let result = run(&store, "RETURN 3 IN [1, 2, 3][0..2] AS r");
    assert!(!bool_val(&result.rows[0][0]));

    // Binds tighter than `=`: `[3,4] IN [...]` is `true`, so the whole
    // thing is `[1,2] = true`, which is `false` (never `[1,2] = [3,4]`
    // first, which would make the IN operand a bool, nonsensical).
    let result = run(&store, "RETURN [1, 2] = [3, 4] IN [[3, 4], false] AS a");
    assert!(!bool_val(&result.rows[0][0]));

    // null propagation: an empty list is always definite false regardless
    // of the needle's nullness; a null element only makes the result
    // unknown when no earlier element definitely matched.
    assert!(matches!(
        run(&store, "RETURN null IN [1, 2] AS r").rows[0][0],
        Value::Null
    ));
    assert!(!bool_val(&run(&store, "RETURN 1 IN [] AS r").rows[0][0]));
    assert!(matches!(
        run(&store, "RETURN null IN [] AS r").rows[0][0],
        Value::Literal(marsdb_query::Literal::Bool(false))
    ));
    assert!(bool_val(
        &run(&store, "RETURN 1 IN [null, 1] AS r").rows[0][0]
    ));
    assert!(matches!(
        run(&store, "RETURN 1 IN [null, 2] AS r").rows[0][0],
        Value::Null
    ));
}

/// `+` is also real Cypher's list concatenation/append/prepend operator
/// (`[1,2] + [3]` concatenates, `[1,2] + 3`/`3 + [1,2]` appends/prepends
/// the scalar) -- `apply_arith`/`ReturnExpr::Arith`'s semantic check both
/// only ever handled numbers/strings before this, unconditionally
/// rejecting any list operand.
#[test]
fn plus_concatenates_and_appends_lists() {
    let store = GraphStore::open_memory().unwrap();

    let result = run(&store, "RETURN [1, 10, 100] + [4, 5] AS foo");
    match &result.rows[0][0] {
        Value::List(items) => {
            let ints: Vec<i64> = items.iter().map(int).collect();
            assert_eq!(ints, vec![1, 10, 100, 4, 5]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    let result = run(&store, "RETURN [false, true] + false AS foo");
    match &result.rows[0][0] {
        Value::List(items) => assert_eq!(items.len(), 3),
        other => panic!("expected a List, got {other:?}"),
    }

    let result = run(&store, "RETURN 0 + [1, 2] AS foo");
    match &result.rows[0][0] {
        Value::List(items) => {
            let ints: Vec<i64> = items.iter().map(int).collect();
            assert_eq!(ints, vec![0, 1, 2]);
        }
        other => panic!("expected a List, got {other:?}"),
    }

    // Non-`+` operators must still reject a list at compile time.
    let stmt = parse("RETURN [1, 2] - 1").unwrap();
    let err = Executor::new(&store)
        .execute(&stmt)
        .expect_err("subtracting from a list must be rejected");
    assert!(format!("{err}").contains("cannot use a list"));
}

/// Real Cypher's integer literal grammar has hex (`0x...`) and octal
/// (`0o...`) forms beyond plain decimal, on both a positive and negative
/// literal. Also exercises `i64::MIN`'s magnitude (`2^63`), which doesn't
/// fit in a *positive* `i64` at all -- only `-0x8000000000000000` (the
/// negated form) is representable, needing the two's-complement special
/// case `parse_int_literal` has.
#[test]
fn int_literal_accepts_hex_and_octal_forms() {
    let store = GraphStore::open_memory().unwrap();
    let cases: &[(&str, i64)] = &[
        ("0x1", 1),
        ("0x7FFFFFFFFFFFFFFF", i64::MAX),
        ("-0x1", -1),
        ("-0x8000000000000000", i64::MIN),
        ("0x1a2b3", 0x1a2b3),
        ("0x1A2B3", 0x1a2b3),
        ("0o1", 1),
        ("0o777777777777777777777", i64::MAX),
        ("-0o1", -1),
        ("-0o1000000000000000000000", i64::MIN),
    ];
    for (text, expected) in cases {
        let result = run(&store, &format!("RETURN {text} AS x"));
        assert_eq!(int(&result.rows[0][0]), *expected, "for {text}");
    }
    // A plain decimal literal must stay unaffected.
    assert_eq!(int(&run(&store, "RETURN 42 AS x").rows[0][0]), 42);
}

/// Real Cypher accepts either quote style for a string literal, not just
/// `'...'` -- and `\uXXXX` (exactly 4 hex digits, a BMP code point) as a
/// string escape, previously unrecognized.
#[test]
fn double_quoted_strings_and_unicode_escapes() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN \"\" AS a, \"hello\" AS b");
    match (&result.rows[0][0], &result.rows[0][1]) {
        (
            Value::Literal(marsdb_query::Literal::String(a)),
            Value::Literal(marsdb_query::Literal::String(b)),
        ) => {
            assert_eq!(a, "");
            assert_eq!(b, "hello");
        }
        other => panic!("unexpected value {other:?}"),
    }

    let result = run(&store, "RETURN '\\u01FF' AS a");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "\u{1FF}"),
        other => panic!("unexpected value {other:?}"),
    }
}

/// Real Cypher's float literal grammar has three shapes beyond plain
/// `digits.digits`: a leading-dot form with no integer part (`.1`), and
/// exponent notation on either form or on a bare integer (`1e9`, `.1e-5`).
/// `float_literal`'s old grammar only accepted `digits.digits`.
#[test]
fn float_literal_accepts_leading_dot_and_exponent_forms() {
    let store = GraphStore::open_memory().unwrap();
    let cases: &[(&str, f64)] = &[
        (".1", 0.1),
        (".0", 0.0),
        ("1e9", 1e9),
        ("1E9", 1e9),
        (".1e9", 0.1e9),
        ("1e-5", 1e-5),
        (".1e-5", 0.1e-5),
    ];
    for (text, expected) in cases {
        let result = run(&store, &format!("RETURN {text} AS x"));
        match &result.rows[0][0] {
            Value::Literal(marsdb_query::Literal::Float(f)) => {
                assert!(
                    (*f - expected).abs() < 1e-15,
                    "{text}: got {f}, expected {expected}"
                );
            }
            other => panic!("{text}: expected a float literal, got {other:?}"),
        }
    }
    // A plain integer must stay an Int, not get swept into the widened
    // float grammar.
    let result = run(&store, "RETURN 42 AS x");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Int(n)) => assert_eq!(*n, 42),
        other => panic!("expected an int literal, got {other:?}"),
    }
}

/// `str::parse::<f64>()` silently returns `f64::INFINITY` for a magnitude
/// beyond f64's representable range instead of erroring -- real Cypher
/// requires this to be a compile-time error, not a silently-produced
/// `inf` literal.
#[test]
fn float_literal_overflow_is_a_syntax_error_not_infinity() {
    let err = marsdb_query::parse("RETURN 1.34E999")
        .expect_err("a float literal beyond f64's range must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("too large"), "unexpected error: {msg}");

    // Within range -- must still parse fine.
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN 1.23456789e308 AS x");
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::Float(f)) => assert!(f.is_finite()),
        other => panic!("expected a finite float literal, got {other:?}"),
    }
}

#[test]
fn list_slice_out_of_range_bounds_clamp_instead_of_null() {
    // Regression guard: unlike single-element indexing, out-of-range slice
    // bounds clamp to [0, len] rather than producing null.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH [1, 2, 3] AS list RETURN list[-100..100], list[5..10]",
    );
    assert_eq!(list_ints(&result.rows[0][0]), vec![1, 2, 3]);
    assert_eq!(list_ints(&result.rows[0][1]), Vec::<i64>::new());
}

#[test]
fn to_string_and_round_trip() {
    // Temporal6 scenarios [1]/[6].
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS d \
         RETURN toString(d), date(toString(d)) = d",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1984-10-11");
    assert!(boolean(&row[1]));

    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: -14, hours: 16}) AS d \
         RETURN toString(d), duration(toString(d)) = d",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P12Y5M-14DT16H");
    assert!(boolean(&row[1]));
}

#[test]
fn to_string_rejects_invalid_types() {
    // TypeConversion4 scenario [10]'s five examples: list, map, node,
    // relationship, and path values are runtime type errors, not null.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (n)-[:T]->(m)");
    for query in [
        "RETURN toString([])",
        "RETURN toString({})",
        "MATCH (n) RETURN toString(n)",
        "MATCH ()-[r:T]->() RETURN toString(r)",
        "MATCH p = ()-[:T]->() RETURN toString(p)",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }

    let result = run(&store, "RETURN toString(null)");
    assert!(matches!(result.rows[0][0], Value::Null));
}

#[test]
fn local_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123");

    let result = run(&store, "RETURN toString(localtime('21:40:32.142')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32.142");

    // No seconds/fraction given -> none printed (real Cypher's rule).
    let result = run(&store, "RETURN toString(localtime('21:40')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40");
}

#[test]
fn time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(time({hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123+01:00");

    let result = run(&store, "RETURN toString(time('21:40:32.142+0100')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40:32.142+01:00");

    // Zero offset prints as `Z`, not `+00:00` -- Temporal2 [3].
    let result = run(&store, "RETURN toString(time('2140-00:00')) AS r");
    assert_eq!(temporal_str(&result.rows[0][0]), "21:40Z");
}

/// `time({time: namedZoneDateTime})` -- no *explicit* `timezone` key, the
/// zone was just carried through from the projected base -- silently
/// degrades to the resolved offset instead of erroring, unlike an
/// explicit named-zone request. TCK's Temporal3 [3] row 125.
#[test]
fn time_projected_from_a_named_zone_base_degrades_to_plain_offset() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS other \
         RETURN toString(time({time: other})), toString(time(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:00+01:00");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:00+01:00");
}

#[test]
fn explain_shows_index_seek_and_residual_filter() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com', age: 30})");
    run(&store, "CREATE INDEX ON :Person(email)");

    let result = run(
        &store,
        "EXPLAIN MATCH (n:Person) WHERE n.email = 'alice@x.com' AND n.age > 20 RETURN n",
    );
    let lines = plan_lines(&result);
    assert!(lines.iter().any(|l| l.contains("IndexSeek(n:Person")
        && l.contains("email")
        && l.contains("alice@x.com")));
    assert!(lines.iter().any(|l| l.contains("Filter n.age > 20")));
}

/// A var-free function-call equality (`n.joined = date('2020-01-10')`,
/// the shape a `$param`-substituted temporal lookup takes) must use a
/// declared index, not fall back to a per-row label-scan filter -- and
/// must return the same rows either way.
#[test]
fn explain_shows_index_seek_for_a_literal_arg_call_equality() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Event(joined)");
    run(
        &store,
        "CREATE (:Event {name: 'a', joined: date('2020-01-10')}), \
                (:Event {name: 'b', joined: date('2021-03-04')})",
    );

    let explained = run(
        &store,
        "EXPLAIN MATCH (n:Event) WHERE n.joined = date('2020-01-10') RETURN n.name",
    );
    let lines = plan_lines(&explained);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("IndexSeek(n:Event") && l.contains("joined")),
        "expected an IndexSeek, got plan: {lines:?}"
    );

    let result = run(
        &store,
        "MATCH (n:Event) WHERE n.joined = date('2020-01-10') RETURN n.name",
    );
    assert_eq!(result.rows.len(), 1);
    assert_eq!(str_value(&result.rows[0][0]), "a");
}

/// rand() must never be hoisted into an IndexSeek value -- it has to
/// evaluate per candidate row (fresh number each call), so the equality
/// stays a Filter over the scan even with an index declared.
#[test]
fn explain_keeps_a_rand_equality_as_a_filter_despite_an_index() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE INDEX ON :Event(score)");
    run(&store, "CREATE (:Event {score: 0.5})");

    let explained = run(
        &store,
        "EXPLAIN MATCH (n:Event) WHERE n.score = rand() RETURN n",
    );
    let lines = plan_lines(&explained);
    assert!(
        !lines.iter().any(|l| l.contains("IndexSeek")),
        "rand() must not become an IndexSeek, got plan: {lines:?}"
    );
}

#[test]
fn explain_falls_back_to_scan_when_no_index_declared() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {email: 'alice@x.com'})");

    let result = run(
        &store,
        "EXPLAIN MATCH (n:Person {email: 'alice@x.com'}) RETURN n",
    );
    let lines = plan_lines(&result);
    assert!(lines
        .iter()
        .any(|l| l.contains("NodeByLabelScan(n:Person)")));
    assert!(!lines.iter().any(|l| l.contains("IndexSeek")));
}

#[test]
fn explain_never_mutates_even_a_write_statement() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:Person {name: 'a'})");

    let explained = run(&store, "EXPLAIN CREATE (:Person {name: 'b'})");
    assert_eq!(plan_lines(&explained).len(), 1);
    assert!(plan_lines(&explained)[0].contains("no query plan"));

    let count = run(&store, "MATCH (n:Person) RETURN count(n)");
    assert_eq!(int_value(&count.rows[0][0]), 1);
}

#[test]
fn syntax_error_for_malformed_query_text() {
    // Never reaches planning/execution at all -- pest itself rejects it.
    let err = parse("MATCH (n RETURN n").unwrap_err();
    assert!(err.to_string().starts_with("syntax error:"));
}

#[test]
fn semantic_error_for_structurally_invalid_but_parseable_query() {
    // Parses fine, but references a name never bound anywhere -- caught
    // by the pre-execution semantic pass, not a grammar failure.
    let stmt = parse("RETURN missing").unwrap();
    let store = GraphStore::open_memory().unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

#[test]
fn builtin_size_list_and_string() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        int_value(&run(&store, "RETURN size([1,2,3])").rows[0][0]),
        3
    );
    assert_eq!(
        int_value(&run(&store, "RETURN size('hello')").rows[0][0]),
        5
    );
}

#[test]
fn builtin_range_inclusive_both_ends_and_negative_step() {
    let store = GraphStore::open_memory().unwrap();
    let up = run(&store, "RETURN range(1, 5)");
    let Value::List(items) = &up.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(
        items.iter().map(int_value).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );

    let down = run(&store, "RETURN range(10, 0, -2)");
    let Value::List(items) = &down.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(
        items.iter().map(int_value).collect::<Vec<_>>(),
        vec![10, 8, 6, 4, 2, 0]
    );

    let stmt = parse("RETURN range(1, 5, 0)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().contains("step"));
}

#[test]
fn builtin_head_tail_last_on_a_list() {
    // List literal elements are `Value::Literal`, not `Value::Property`
    // (they're never round-tripped through storage) -- a local extractor,
    // not the shared `int_value` (which is deliberately strict about
    // that distinction for its other callers).
    fn any_int(v: &Value) -> i64 {
        match v {
            Value::Literal(marsdb_query::Literal::Int(i)) => *i,
            other => panic!("expected an int, got {other:?}"),
        }
    }

    let store = GraphStore::open_memory().unwrap();
    assert_eq!(any_int(&run(&store, "RETURN head([1,2,3])").rows[0][0]), 1);
    assert_eq!(any_int(&run(&store, "RETURN last([1,2,3])").rows[0][0]), 3);
    let tail = run(&store, "RETURN tail([1,2,3])");
    let Value::List(items) = &tail.rows[0][0] else {
        panic!("expected a list");
    };
    assert_eq!(items.iter().map(any_int).collect::<Vec<_>>(), vec![2, 3]);

    // Empty list is null, not an error -- same out-of-bounds convention
    // as list indexing elsewhere in this codebase.
    let empty_head = run(&store, "RETURN head([])");
    assert!(matches!(empty_head.rows[0][0], Value::Null));
}

#[test]
fn builtin_string_functions() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        str_value(&run(&store, "RETURN toUpper('hi')").rows[0][0]),
        "HI"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN toLower('HI')").rows[0][0]),
        "hi"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN trim('  hi  ')").rows[0][0]),
        "hi"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN reverse('abc')").rows[0][0]),
        "cba"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN replace('hello world', 'world', 'there')").rows[0][0]),
        "hello there"
    );
    let split = run(&store, "RETURN split('a,b,c', ',')");
    assert_eq!(list_str_values(&split.rows[0][0]), vec!["a", "b", "c"]);
    assert_eq!(
        str_value(&run(&store, "RETURN substring('hello', 1, 3)").rows[0][0]),
        "ell"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN left('hello', 3)").rows[0][0]),
        "hel"
    );
    assert_eq!(
        str_value(&run(&store, "RETURN right('hello', 3)").rows[0][0]),
        "llo"
    );
}

#[test]
fn builtin_math_functions() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(int_value(&run(&store, "RETURN abs(-5)").rows[0][0]), 5);
    assert_eq!(
        float_value(&run(&store, "RETURN abs(-5.5)").rows[0][0]),
        5.5
    );
    assert_eq!(
        float_value(&run(&store, "RETURN ceil(4.1)").rows[0][0]),
        5.0
    );
    assert_eq!(
        float_value(&run(&store, "RETURN floor(4.9)").rows[0][0]),
        4.0
    );
    assert_eq!(
        float_value(&run(&store, "RETURN sqrt(16.0)").rows[0][0]),
        4.0
    );
    assert_eq!(int_value(&run(&store, "RETURN sign(-7)").rows[0][0]), -1);
    assert_eq!(int_value(&run(&store, "RETURN sign(0)").rows[0][0]), 0);
}

#[test]
fn builtin_to_float_and_to_boolean() {
    let store = GraphStore::open_memory().unwrap();
    assert_eq!(
        float_value(&run(&store, "RETURN toFloat('12.5')").rows[0][0]),
        12.5
    );
    assert!(bool_value(
        &run(&store, "RETURN toBoolean('true')").rows[0][0]
    ));
    assert!(!bool_value(
        &run(&store, "RETURN toBoolean('false')").rows[0][0]
    ));
}

/// `toFloat()` on a `Bool` is a real type error, not `null` -- unlike an
/// unparseable *string*, which real Cypher does treat as `null` (a
/// string always at least plausibly could be numeric text, a boolean
/// never could be). TCK's TypeConversion3 [6].
#[test]
fn to_float_on_a_bool_is_a_type_error_not_null() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN toFloat(true)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("tofloat"));
    // sanity: an unparseable string still degrades to null, not an error
    assert!(matches!(
        run(&store, "RETURN toFloat('nope')").rows[0][0],
        Value::Null
    ));
}

/// Only a node, relationship, map, or temporal value has any `.prop` to
/// access at all -- a plain scalar or list is a real type error, not a
/// silent `null`. TCK's Graph6 [9] / Map1 [6].
#[test]
fn property_access_on_a_non_graph_scalar_or_list_is_a_type_error() {
    let store = GraphStore::open_memory().unwrap();
    for exp in ["123", "42.45", "true", "false", "'string'", "[123, true]"] {
        let stmt = parse(&format!(
            "WITH {exp} AS nonGraphElement RETURN nonGraphElement.num"
        ))
        .unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("propert"),
            "expected a property-access type error for {exp:?}, got: {err}"
        );
    }
    // sanity: null, maps, nodes, and temporal values still work normally
    assert!(matches!(
        run(&store, "WITH null AS x RETURN x.num").rows[0][0],
        Value::Null
    ));
    match &run(&store, "WITH {name: 'foo'} AS m RETURN m.name").rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "foo"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `type()` only ever accepts a relationship -- `MATCH (r) RETURN
/// type(r)` (`r` a *node*, from the pattern itself) is a compile-time
/// error even when the `MATCH` matches zero rows, not only a runtime one
/// a zero-row match would silently skip. TCK's Graph4 [7].
#[test]
fn type_on_a_node_is_a_compile_time_error_even_on_zero_rows() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("MATCH (r) RETURN type(r)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("relationship"));
}

#[test]
fn property_presence_check_via_is_not_null() {
    // `exists(n.num)` (bare function-call form) isn't real openCypher --
    // grep against openCypher.bnf/the TCK corpus finds no such function,
    // only the unrelated `EXISTS { <pattern> }` subquery form. `IS NOT
    // NULL` is the real, spec-correct way to check property presence.
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N {num: 42})");
    assert!(bool_value(
        &run(&store, "MATCH (n) RETURN n.num IS NOT NULL").rows[0][0]
    ));
    assert!(!bool_value(
        &run(&store, "MATCH (n) RETURN n.missing IS NOT NULL").rows[0][0]
    ));
}

#[test]
fn builtin_id_returns_an_integer() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (:N)");
    let result = run(&store, "MATCH (n) RETURN id(n)");
    // Just needs to be a real, non-negative integer -- the exact value is
    // an internal id, not something callers should depend on.
    assert!(int_value(&result.rows[0][0]) >= 0);
}

#[test]
fn unknown_function_name_is_a_semantic_error_not_a_panic() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN totallyMadeUpFunction(1)").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(err.to_string().starts_with("semantic error:"));
}

/// `(list[1]).prop` -- property access on a node/map produced by indexing
/// into a list, not a bare variable. TCK's Map1 [3], Graph6 [4]/[8].
/// Missing properties read back as `null`, same as `var.missing` already
/// does for a bound variable.
#[test]
fn property_access_on_an_indexed_list_element() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({existing: 42, missing: null})");
    let result = run(
        &store,
        "MATCH (n) WITH [123, n] AS list \
         RETURN (list[1]).missing, (list[1]).missingToo, (list[1]).existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);

    let result = run(
        &store,
        "WITH [123, {existing: 42, notMissing: null}] AS list \
         RETURN (list[1]).missing, (list[1]).notMissing, (list[1]).existing",
    );
    assert!(matches!(result.rows[0][0], Value::Null));
    assert!(matches!(result.rows[0][1], Value::Null));
    assert_eq!(int(&result.rows[0][2]), 42);
}

/// `nestedMap.name.name2` -- a chain of two `.prop` suffixes. TCK's
/// With2 [2].
#[test]
fn chained_property_access_on_a_nested_map_literal() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH {name: {name2: 'baz'}} AS nestedMap RETURN nestedMap.name.name2",
    );
    match &result.rows[0][0] {
        Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "baz"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `n['name']` -- dynamic property access on a node/relationship, with a
/// computed key expression, not just a literal string. TCK's Graph7
/// [1]-[3].
#[test]
fn dynamic_property_access_on_a_node() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE ({name: 'Apa'})");
    let result = run(
        &store,
        "MATCH (n {name: 'Apa'}) RETURN n['nam' + 'e'] AS value",
    );
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s)) => assert_eq!(s, "Apa"),
        other => panic!("expected a string, got {other:?}"),
    }
}

/// `type(null)` -- a null-valued argument to a graph-object builtin
/// (`type`/`nodes`/`relationships`/`length`) is `null`, not a compile-time
/// type error. TCK's Graph4 [3], Path1 [1], Path2 [3].
#[test]
fn graph_builtins_on_null_argument_are_null_not_an_error() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN type(null), nodes(null), relationships(null), length(null)",
    );
    for cell in &result.rows[0] {
        assert!(matches!(cell, Value::Null), "expected null, got {cell:?}");
    }
}

/// A map-valued `$param` (`{name: 'Apa'}`) -- substitutes into a
/// `ReturnExpr::MapLit`, same recursive "no literal syntax, rewrite the
/// whole Lit node" treatment list-valued params already get. TCK's
/// Map2/Map3.
#[test]
fn map_valued_parameters_substitute_into_a_map_literal_expression() {
    use std::collections::{BTreeMap, HashMap};
    let store = GraphStore::open_memory().unwrap();
    let mut stmt = parse("WITH $expr AS expr, $idx AS idx RETURN expr[idx]").unwrap();
    let mut map = BTreeMap::new();
    map.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Apa".to_string()),
    );
    let mut params = HashMap::new();
    params.insert("expr".to_string(), marsdb_graph::PropertyValue::Map(map));
    params.insert(
        "idx".to_string(),
        marsdb_graph::PropertyValue::String("name".to_string()),
    );
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::String(s))
        | Value::Literal(marsdb_query::Literal::String(s)) => assert_eq!(s, "Apa"),
        other => panic!("expected a string, got {other:?}"),
    }

    // keys($param) on a (possibly nested) map-valued parameter.
    let mut stmt = parse("RETURN keys($param) AS k").unwrap();
    let mut inner = BTreeMap::new();
    inner.insert(
        "city".to_string(),
        marsdb_graph::PropertyValue::String("London".to_string()),
    );
    let mut outer = BTreeMap::new();
    outer.insert(
        "name".to_string(),
        marsdb_graph::PropertyValue::String("Alice".to_string()),
    );
    outer.insert(
        "address".to_string(),
        marsdb_graph::PropertyValue::Map(inner),
    );
    let mut params = HashMap::new();
    params.insert("param".to_string(), marsdb_graph::PropertyValue::Map(outer));
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    let mut keys: Vec<String> = match &result.rows[0][0] {
        Value::List(items) => items
            .iter()
            .map(|v| match v {
                Value::Property(marsdb_graph::PropertyValue::String(s)) => s.clone(),
                other => panic!("expected a string, got {other:?}"),
            })
            .collect(),
        other => panic!("expected a List, got {other:?}"),
    };
    keys.sort();
    assert_eq!(keys, vec!["address".to_string(), "name".to_string()]);
}

mod call_procedures {
    use std::collections::HashMap;

    use marsdb_graph::GraphStore;
    use marsdb_query::{
        parse, ExecutionOptions, Executor, ProcedureProvider, ProcedureSignature, Procedures,
        QueryError, Value,
    };

    use super::run;

    /// A minimal test-only `ProcedureProvider` -- ignores `args`
    /// entirely and always returns the same fixed rows, since these
    /// tests exercise MarsDB's own CALL/YIELD mechanics (arity, renaming,
    /// WHERE, WITH-chaining, standalone auto-yield), not a real
    /// table-lookup mock protocol (that's `marsdb-tck`'s own concern, see
    /// its `TckProcedureProvider`).
    /// `(input names, output names, fixed output rows)`.
    type ProcFixture = (Vec<&'static str>, Vec<&'static str>, Vec<Vec<Value>>);

    struct TestProvider {
        procs: HashMap<&'static str, ProcFixture>,
    }

    impl ProcedureProvider for TestProvider {
        fn signature(&self, name: &str) -> Option<ProcedureSignature> {
            let (inputs, outputs, _) = self.procs.get(name)?;
            Some(ProcedureSignature {
                inputs: inputs.iter().map(|s| s.to_string()).collect(),
                // Unrecognized -- `value_matches_declared_type` accepts
                // anything for a type name it doesn't know, so these
                // tests (which exercise CALL's own mechanics, not
                // argument-type checking) aren't accidentally blocked by
                // whatever shape a test happens to pass.
                input_types: inputs.iter().map(|_| "ANY".to_string()).collect(),
                outputs: outputs.iter().map(|s| s.to_string()).collect(),
            })
        }

        fn call(&self, name: &str, _args: &[Value]) -> Result<Vec<Vec<Value>>, QueryError> {
            Ok(self
                .procs
                .get(name)
                .expect("checked by signature")
                .2
                .clone())
        }
    }

    fn options(procs: TestProvider) -> ExecutionOptions {
        ExecutionOptions {
            procedures: Some(Procedures(std::sync::Arc::new(procs))),
            ..Default::default()
        }
    }

    fn int(v: &Value) -> i64 {
        match v {
            Value::Literal(marsdb_query::Literal::Int(i)) => *i,
            Value::Property(marsdb_graph::PropertyValue::Int(i)) => *i,
            other => panic!("expected an Int, got {other:?}"),
        }
    }

    fn str_val(v: &Value) -> String {
        match v {
            Value::Literal(marsdb_query::Literal::String(s)) => s.clone(),
            other => panic!("expected a String, got {other:?}"),
        }
    }

    #[test]
    fn standalone_call_auto_yields_every_output_with_no_yield_written() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.labels",
                (
                    vec![],
                    vec!["label"],
                    vec![
                        vec![Value::Literal(marsdb_query::Literal::String("A".into()))],
                        vec![Value::Literal(marsdb_query::Literal::String("B".into()))],
                    ],
                ),
            )]),
        };
        let stmt = parse("CALL test.labels()").unwrap();
        let result = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap();
        assert_eq!(result.columns, vec!["label"]);
        let labels: Vec<String> = result.rows.iter().map(|r| str_val(&r[0])).collect();
        assert_eq!(labels, vec!["A", "B"]);
    }

    #[test]
    fn in_query_call_with_yield_fans_out_and_renames() {
        let store = GraphStore::open_memory().unwrap();
        run(&store, "CREATE (:N {x: 1}), (:N {x: 2})");
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.proc",
                (
                    vec!["in"],
                    vec!["a", "b"],
                    vec![vec![
                        Value::Literal(marsdb_query::Literal::Int(1)),
                        Value::Literal(marsdb_query::Literal::Int(2)),
                    ]],
                ),
            )]),
        };
        let stmt =
            parse("MATCH (n:N) CALL test.proc(n.x) YIELD a, b AS c RETURN n.x, a, c").unwrap();
        let result = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap();
        assert_eq!(result.columns, vec!["n.x", "a", "c"]);
        // Fans out once per input row -- 2 nodes x 1 proc row each = 2.
        assert_eq!(result.rows.len(), 2);
        for row in &result.rows {
            assert_eq!(int(&row[1]), 1);
            assert_eq!(int(&row[2]), 2);
        }
    }

    #[test]
    fn in_query_call_without_yield_discards_outputs_but_keeps_rows() {
        let store = GraphStore::open_memory().unwrap();
        run(&store, "CREATE (:A), (:B), (:C)");
        let procs = TestProvider {
            procs: HashMap::from([("test.doNothing", (vec![], vec![], vec![vec![]]))]),
        };
        let stmt = parse("MATCH (n) CALL test.doNothing() RETURN count(n) AS c").unwrap();
        let result = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap();
        assert_eq!(int(&result.rows[0][0]), 3);
    }

    #[test]
    fn call_without_yield_then_referencing_output_is_undefined_variable() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.proc",
                (vec!["in"], vec!["out"], vec![vec![Value::Null]]),
            )]),
        };
        let stmt = parse("CALL test.proc(1) RETURN out").unwrap();
        let err = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap_err();
        assert!(matches!(err, QueryError::Semantic(_)));
    }

    #[test]
    fn wrong_arity_is_a_compile_time_error() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([("test.proc", (vec!["a", "b"], vec!["out"], vec![]))]),
        };
        let stmt = parse("CALL test.proc(1)").unwrap();
        let err = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap_err();
        assert!(matches!(err, QueryError::Semantic(_)));
    }

    #[test]
    fn unknown_procedure_errors() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::new(),
        };
        let stmt = parse("CALL test.nope()").unwrap();
        let err = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap_err();
        assert!(matches!(err, QueryError::Semantic(_)));
    }

    #[test]
    fn yield_shadowing_an_already_bound_variable_is_a_compile_time_error() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.labels",
                (
                    vec![],
                    vec!["label"],
                    vec![vec![Value::Literal(marsdb_query::Literal::String(
                        "A".into(),
                    ))]],
                ),
            )]),
        };
        let stmt = parse("WITH 'Hi' AS label CALL test.labels() YIELD label RETURN *").unwrap();
        let err = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap_err();
        assert!(matches!(err, QueryError::Semantic(_)));
    }

    #[test]
    fn implicit_arguments_resolve_from_same_named_params() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.proc",
                (
                    vec!["name"],
                    vec!["out"],
                    vec![vec![Value::Literal(marsdb_query::Literal::String(
                        "found".into(),
                    ))]],
                ),
            )]),
        };
        let stmt = parse("CALL test.proc").unwrap();
        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            marsdb_graph::PropertyValue::String("Stefan".to_string()),
        );
        // `substitute_params` has nothing to do here (no `$param` literal
        // position exists for the implicit-argument form) -- the raw
        // params map itself has to flow through `ExecutionOptions`
        // instead, same as `marsdb::Database` already wires up.
        let mut opts = options(procs);
        opts.params = params;
        let result = Executor::new(&store)
            .execute_with_options(&stmt, &opts)
            .unwrap();
        assert_eq!(str_val(&result.rows[0][0]), "found");
    }

    #[test]
    fn implicit_arguments_missing_param_errors() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([("test.proc", (vec!["name"], vec!["out"], vec![]))]),
        };
        let stmt = parse("CALL test.proc").unwrap();
        let err = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap_err();
        assert!(matches!(err, QueryError::MissingParam(_)));
    }

    #[test]
    fn call_then_with_then_call_again_chains_correctly() {
        let store = GraphStore::open_memory().unwrap();
        let procs = TestProvider {
            procs: HashMap::from([(
                "test.labels",
                (
                    vec![],
                    vec!["label"],
                    vec![
                        vec![Value::Literal(marsdb_query::Literal::String("A".into()))],
                        vec![Value::Literal(marsdb_query::Literal::String("B".into()))],
                        vec![Value::Literal(marsdb_query::Literal::String("C".into()))],
                    ],
                ),
            )]),
        };
        let stmt = parse(
            "CALL test.labels() YIELD label \
             WITH count(*) AS c \
             CALL test.labels() YIELD label \
             RETURN *",
        )
        .unwrap();
        let result = Executor::new(&store)
            .execute_with_options(&stmt, &options(procs))
            .unwrap();
        assert_eq!(result.rows.len(), 3);
        for row in &result.rows {
            assert_eq!(int(&row[0]), 3);
        }
    }
}
