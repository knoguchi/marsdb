//! `Database::prepare`/`execute_prepared_plan` — the plan-cache-backed
//! prepared-statement API. Correctness is what matters here: repeat
//! calls on one `PreparedPlan` must return the same results a fresh
//! `execute_with_params` call would, regardless of whether a given call
//! took the validation-skipping fast path or not (which of the two ran
//! isn't observable from the public API in this version, by design —
//! only the results are).

use std::collections::HashMap;

use marsdb::{Database, PropertyValue};

fn ground_truth(
    db: &Database,
    cypher: &str,
    params: &HashMap<String, PropertyValue>,
) -> marsdb::QueryResult {
    db.execute_with_params(cypher, params)
        .unwrap_or_else(|e| panic!("execute_with_params failed for {cypher:?}: {e}"))
}

/// `Value` has no `PartialEq` (see `marsdb-query/src/value.rs`), so
/// tests compare via `Debug` formatting -- the same pattern other
/// `marsdb`/`marsdb-query` tests use for whole-row comparisons.
fn rows_debug(result: &marsdb::QueryResult) -> String {
    format!("{:?}", result.rows)
}

fn int_at(result: &marsdb::QueryResult, row: usize, col: usize) -> i64 {
    match &result.rows[row][col] {
        marsdb::Value::Property(PropertyValue::Int(i)) => *i,
        other => panic!("expected Int at [{row}][{col}], got {other:?}"),
    }
}

fn string_at(result: &marsdb::QueryResult, row: usize, col: usize) -> String {
    match &result.rows[row][col] {
        marsdb::Value::Property(PropertyValue::String(s)) => s.clone(),
        other => panic!("expected String at [{row}][{col}], got {other:?}"),
    }
}

/// Repeat calls with ordinary scalar params (the common case) return
/// correct, identical-shaped results, and the plan is genuinely reused
/// (parsed once via `prepare`, not re-parsed per call).
#[test]
fn repeat_calls_with_scalar_params_stay_correct() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:Person {name: 'Alice', age: 30})")
        .unwrap();
    db.execute("CREATE (:Person {name: 'Bob', age: 25})")
        .unwrap();

    let prepared = db
        .prepare("MATCH (p:Person {name: $name}) RETURN p.age")
        .unwrap();

    for (name, expected_age) in [("Alice", 30), ("Bob", 25), ("Alice", 30)] {
        let mut params = HashMap::new();
        params.insert("name".to_string(), PropertyValue::String(name.to_string()));
        let result = db
            .execute_prepared_plan(&prepared, &params, &marsdb::ExecutionOptions::default())
            .unwrap();
        assert_eq!(
            result.rows.len(),
            1,
            "expected exactly one match for {name}"
        );
        assert_eq!(int_at(&result, 0, 0), expected_age, "wrong age for {name}");
    }
}

/// A parameter's coarse category (null/scalar/list/map) changing across
/// calls on the *same* `PreparedPlan` must still validate and execute
/// correctly every time -- this is exactly the case
/// `PreparedPlan::can_skip_validation` has to get right (see its doc
/// comment): `UNWIND $x` behaves differently depending on whether `$x`
/// is a list, a scalar, null, or (invalidly) a map.
#[test]
fn param_category_changes_across_calls_stay_correct() {
    let db = Database::in_memory().unwrap();
    let prepared = db.prepare("UNWIND $x AS item RETURN item").unwrap();
    let opts = marsdb::ExecutionOptions::default();

    // List: unwinds to 3 rows.
    let mut p = HashMap::new();
    p.insert(
        "x".to_string(),
        PropertyValue::List(vec![
            PropertyValue::Int(1),
            PropertyValue::Int(2),
            PropertyValue::Int(3),
        ]),
    );
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 3);
    assert_eq!(
        rows_debug(&result),
        rows_debug(&ground_truth(&db, "UNWIND $x AS item RETURN item", &p))
    );

    // Scalar: `Kind::Scalar` defers to runtime (validation passes), but
    // execution itself rejects a non-list, non-null UNWIND source. Must
    // still be caught as an error here, matching ground truth exactly --
    // not silently misexecuted because a previous call's plan is reused.
    let mut p = HashMap::new();
    p.insert("x".to_string(), PropertyValue::Int(42));
    let err = db.execute_prepared_plan(&prepared, &p, &opts).unwrap_err();
    let ground_truth_err = db
        .execute_with_params("UNWIND $x AS item RETURN item", &p)
        .unwrap_err();
    assert_eq!(err.to_string(), ground_truth_err.to_string());

    // Null: unwinds to 0 rows.
    let mut p = HashMap::new();
    p.insert("x".to_string(), PropertyValue::Null);
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 0);
    assert_eq!(
        rows_debug(&result),
        rows_debug(&ground_truth(&db, "UNWIND $x AS item RETURN item", &p))
    );

    // Back to list, after having validated scalar and null in between --
    // must not have latched onto a stale fingerprint.
    let mut p = HashMap::new();
    p.insert(
        "x".to_string(),
        PropertyValue::List(vec![PropertyValue::Int(9)]),
    );
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 1);

    // Map: `bind_unwind` rejects a map UNWIND source outright -- this
    // must still be caught as an error, not silently accepted because a
    // previous scalar/list/null call cached a "validation passed" state.
    let mut p = HashMap::new();
    let mut m = std::collections::BTreeMap::new();
    m.insert("a".to_string(), PropertyValue::Int(1));
    p.insert("x".to_string(), PropertyValue::Map(m));
    let err = db.execute_prepared_plan(&prepared, &p, &opts).unwrap_err();
    let ground_truth_err = db
        .execute_with_params("UNWIND $x AS item RETURN item", &p)
        .unwrap_err();
    assert_eq!(err.to_string(), ground_truth_err.to_string());
}

/// `Int` vs `String` never changes `validate_statement`'s outcome (both
/// are `Kind::Scalar`), so this should be the common, fully-cached-hit
/// case -- still needs to just work.
#[test]
fn scalar_type_varies_within_the_scalar_category() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:Item {tag: 'a'})").unwrap();
    db.execute("CREATE (:Item {tag: '5'})").unwrap();

    let prepared = db
        .prepare("MATCH (n:Item) WHERE n.tag = $t RETURN n.tag")
        .unwrap();
    let opts = marsdb::ExecutionOptions::default();

    let mut p = HashMap::new();
    p.insert("t".to_string(), PropertyValue::String("a".to_string()));
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 1);

    // Same prepared plan, an Int-valued param this time -- still a
    // Kind::Scalar, must still validate/execute correctly (no match,
    // not an error).
    let mut p = HashMap::new();
    p.insert("t".to_string(), PropertyValue::Int(5));
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 0);
}

/// A second `CREATE INDEX` after a `PreparedPlan` has already cached a
/// validated fingerprint must not leave the cache silently stale --
/// `GraphStore::schema_generation()` is the invalidation signal. This
/// doesn't assert anything about *which* plan gets used (out of scope
/// for this version, which doesn't cache planning at all yet), only
/// that results stay correct across the index creation.
#[test]
fn schema_change_after_prepare_stays_correct() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE INDEX ON :Person(name)").unwrap();
    db.execute("CREATE (:Person {name: 'Alice', city: 'Boston'})")
        .unwrap();
    db.execute("CREATE (:Person {name: 'Bob', city: 'Boston'})")
        .unwrap();

    let prepared = db
        .prepare("MATCH (p:Person {name: $name}) RETURN p.city")
        .unwrap();
    let opts = marsdb::ExecutionOptions::default();

    let mut p = HashMap::new();
    p.insert(
        "name".to_string(),
        PropertyValue::String("Alice".to_string()),
    );
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 1);

    // New index mid-sequence, bumping schema_generation.
    db.execute("CREATE INDEX ON :Person(city)").unwrap();

    let mut p = HashMap::new();
    p.insert("name".to_string(), PropertyValue::String("Bob".to_string()));
    let result = db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(string_at(&result, 0, 0), "Boston");
}

/// `MERGE` must keep working correctly through a `PreparedPlan` --
/// its own plan is already per-row today (`RowExpr`-based seeks,
/// unrelated to this feature), so this just guards against a
/// regression, not a specific caching behavior.
#[test]
fn merge_through_prepared_plan_stays_correct() {
    let db = Database::in_memory().unwrap();
    let prepared = db
        .prepare("MERGE (p:Person {name: $name}) ON CREATE SET p.seen = 1 ON MATCH SET p.seen = 2")
        .unwrap();
    let opts = marsdb::ExecutionOptions::default();

    let mut p = HashMap::new();
    p.insert(
        "name".to_string(),
        PropertyValue::String("Alice".to_string()),
    );
    db.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    db.execute_prepared_plan(&prepared, &p, &opts).unwrap();

    let result = db
        .execute("MATCH (p:Person {name: 'Alice'}) RETURN p.seen")
        .unwrap();
    assert_eq!(
        result.rows.len(),
        1,
        "MERGE must not have created a duplicate node"
    );
    assert_eq!(
        int_at(&result, 0, 0),
        2,
        "second call should have taken the ON MATCH branch"
    );
}

/// `Transaction::execute_prepared_plan` -- the explicit-transaction twin.
#[test]
fn execute_prepared_plan_inside_an_explicit_transaction() {
    let db = Database::in_memory().unwrap();
    let prepared = db.prepare("CREATE (:Counter {n: $n})").unwrap();
    let opts = marsdb::ExecutionOptions::default();

    let mut tx = db.begin_transaction().unwrap();
    for n in 0..3 {
        let mut p = HashMap::new();
        p.insert("n".to_string(), PropertyValue::Int(n));
        tx.execute_prepared_plan(&prepared, &p, &opts).unwrap();
    }
    tx.commit().unwrap();

    let result = db.execute("MATCH (c:Counter) RETURN count(*)").unwrap();
    assert_eq!(int_at(&result, 0, 0), 3);
}
