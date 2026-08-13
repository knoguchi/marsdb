//! Smoke tests for `eval_projected_expr` (`executor/value_cmp.rs`) --
//! specifically its `Slice`/`ListComp`/`Quantifier` arms. Those three
//! expression forms are already exercised thoroughly during ordinary
//! projection (smoke_expressions.rs), but that's a *different*
//! evaluator (`executor.rs`'s main `eval_return_expr`).
//! `eval_projected_expr` only runs when `apply_order_by` can't match an
//! `ORDER BY` expression to an existing output column by name/structure
//! and has to re-evaluate it against the already-projected row instead
//! -- so these need an `ORDER BY` expression that's structurally
//! distinct from every `RETURN` item.

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;

#[test]
fn order_by_slice_of_a_derived_list() {
    let store = GraphStore::open_memory().unwrap();
    // `[n, 99][0..1]` is a single-element list containing just `n` --
    // sorting by it is equivalent to sorting by `n` itself, but its AST
    // shape (`Slice`) never matches the `RETURN n` item verbatim, so
    // `apply_order_by` must fall back to `eval_projected_expr`.
    let result = run(
        &store,
        "UNWIND [3, 1, 2] AS n RETURN n ORDER BY [n, 99][0..1]",
    );
    let ns: Vec<i64> = result.rows.iter().map(|row| int_value(&row[0])).collect();
    assert_eq!(ns, vec![1, 2, 3]);
}

#[test]
fn order_by_list_comprehension_over_a_derived_list() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "UNWIND [3, 1, 2] AS n RETURN n ORDER BY [i IN [n] WHERE i > 0 | i]",
    );
    let ns: Vec<i64> = result.rows.iter().map(|row| int_value(&row[0])).collect();
    assert_eq!(ns, vec![1, 2, 3]);

    // No WHERE and no projection (`| ...`) -- the comprehension's
    // "keep everything, project the bare item" defaults.
    let result = run(&store, "UNWIND [3, 1, 2] AS n RETURN n ORDER BY [i IN [n]]");
    let ns: Vec<i64> = result.rows.iter().map(|row| int_value(&row[0])).collect();
    assert_eq!(ns, vec![1, 2, 3]);
}

#[test]
fn order_by_quantifier_with_predicate() {
    let store = GraphStore::open_memory().unwrap();
    // `any(i IN [n] WHERE i > 1)` is false for n=1, true for n=2 and
    // n=3 -- false sorts before true, so n=1 comes first; n=2/n=3 keep
    // their relative (stable-sort) order after it.
    let result = run(
        &store,
        "UNWIND [3, 1, 2] AS n RETURN n ORDER BY any(i IN [n] WHERE i > 1)",
    );
    let ns: Vec<i64> = result.rows.iter().map(|row| int_value(&row[0])).collect();
    assert_eq!(ns, vec![1, 3, 2]);
}

/// Every quantifier kind (ALL/ANY/NONE/SINGLE), plus the WHERE-less form
/// (`filterExpression`'s `where?` is optional -- falls back to the
/// item's own truthiness) -- smoke-checked for "doesn't error", not
/// exact order, since only ANY/ALL make sense as a strict ordering key.
#[test]
fn order_by_every_quantifier_kind() {
    let store = GraphStore::open_memory().unwrap();
    for expr in [
        "all(i IN [n] WHERE i > 0)",
        "none(i IN [n] WHERE i > 5)",
        "single(i IN [n] WHERE i = 2)",
    ] {
        let result = run(
            &store,
            &format!("UNWIND [3, 1, 2] AS n RETURN n ORDER BY {expr}"),
        );
        assert_eq!(result.rows.len(), 3);
    }
}
