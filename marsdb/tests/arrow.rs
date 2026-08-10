#![cfg(feature = "arrow")]
//! Arrow export — strict inference rules, exactness, batching. See
//! marsdb/src/arrow.rs's module docs for the rule table under test.

use std::collections::HashMap;

use arrow_array::{
    Array, BooleanArray, Date32Array, Float64Array, Int64Array, ListArray, RecordBatch, StringArray,
};
use arrow_schema::DataType;
use marsdb::{Database, ExecutionOptions};

fn arrow_all(db: &Database, cypher: &str, batch_rows: usize) -> Vec<RecordBatch> {
    db.query_arrow(
        cypher,
        &HashMap::new(),
        &ExecutionOptions::default(),
        batch_rows,
    )
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap()
}

fn arrow_err(db: &Database, cypher: &str) -> String {
    db.query_arrow(cypher, &HashMap::new(), &ExecutionOptions::default(), 1024)
        .err()
        .expect("expected an Arrow export error")
        .to_string()
}

#[test]
fn scalar_types_map_and_round_trip_exactly() {
    let db = Database::in_memory().unwrap();
    db.execute(
        "CREATE (:N {i: 9223372036854775807, f: 1.5, s: 'héllo', b: true, \
         d: date('1984-10-11'), dur: duration('P1M2DT3H')})",
    )
    .unwrap();
    let batches = arrow_all(
        &db,
        "MATCH (n:N) RETURN n.i AS i, n.f AS f, n.s AS s, n.b AS b, n.d AS d, n.dur AS dur",
        1024,
    );
    assert_eq!(batches.len(), 1);
    let b = &batches[0];
    assert_eq!(b.num_rows(), 1);
    let schema = b.schema();
    assert_eq!(schema.field(0).data_type(), &DataType::Int64);
    assert_eq!(schema.field(1).data_type(), &DataType::Float64);
    assert_eq!(schema.field(2).data_type(), &DataType::Utf8);
    assert_eq!(schema.field(3).data_type(), &DataType::Boolean);
    assert_eq!(schema.field(4).data_type(), &DataType::Date32);
    assert!(matches!(schema.field(5).data_type(), DataType::Interval(_)));

    let ints = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(ints.value(0), i64::MAX);
    let floats = b.column(1).as_any().downcast_ref::<Float64Array>().unwrap();
    assert_eq!(floats.value(0), 1.5);
    let strings = b.column(2).as_any().downcast_ref::<StringArray>().unwrap();
    assert_eq!(strings.value(0), "héllo");
    let bools = b.column(3).as_any().downcast_ref::<BooleanArray>().unwrap();
    assert!(bools.value(0));
    let dates = b.column(4).as_any().downcast_ref::<Date32Array>().unwrap();
    // 1984-10-11 = 5397 days since epoch.
    assert_eq!(dates.value(0), 5397);
}

#[test]
fn nulls_become_validity_not_type_changes() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:N {i: 1}), (:N), (:N {i: 3})").unwrap();
    let batches = arrow_all(&db, "MATCH (n:N) RETURN n.i AS i ORDER BY n.i", 1024);
    let col = batches[0].column(0);
    let ints = col.as_any().downcast_ref::<Int64Array>().unwrap();
    assert_eq!(ints.len(), 3);
    assert_eq!(ints.null_count(), 1);
}

#[test]
fn all_null_column_is_null_type_and_empty_result_works() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:N), (:N)").unwrap();
    let batches = arrow_all(&db, "MATCH (n:N) RETURN n.missing AS m", 1024);
    assert_eq!(batches[0].schema().field(0).data_type(), &DataType::Null);
    assert_eq!(batches[0].num_rows(), 2);

    let empty = arrow_all(&db, "MATCH (n:Nothing) RETURN n.x AS x", 1024);
    assert!(empty.is_empty());
}

#[test]
fn batching_splits_at_batch_rows() {
    let db = Database::in_memory().unwrap();
    for i in 0..10 {
        db.execute_with_params(
            "CREATE (:N {i: $i})",
            &HashMap::from([("i".to_string(), marsdb::PropertyValue::Int(i))]),
        )
        .unwrap();
    }
    let batches = arrow_all(&db, "MATCH (n:N) RETURN n.i AS i", 3);
    let sizes: Vec<usize> = batches.iter().map(|b| b.num_rows()).collect();
    assert_eq!(sizes, vec![3, 3, 3, 1]);
}

#[test]
fn homogeneous_lists_export_as_list_arrays() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:N {tags: [1, 2, 3]}), (:N {tags: [4]})")
        .unwrap();
    let batches = arrow_all(&db, "MATCH (n:N) RETURN n.tags AS tags", 1024);
    let col = batches[0].column(0);
    let lists = col.as_any().downcast_ref::<ListArray>().unwrap();
    assert_eq!(lists.len(), 2);
    let total: usize = (0..lists.len()).map(|i| lists.value(i).len()).sum();
    assert_eq!(total, 4);
}

#[test]
fn strict_errors_name_the_column() {
    let db = Database::in_memory().unwrap();
    db.execute("CREATE (a:N {x: 1})-[:R]->(b:N {x: 2.5})")
        .unwrap();
    // Mixed Int/Float in one column.
    let e = arrow_err(&db, "MATCH (n:N) RETURN n.x AS mixed");
    assert!(e.contains("'mixed'") && e.contains("Int and Float"), "{e}");
    // Whole entities.
    let e = arrow_err(&db, "MATCH (n:N) RETURN n");
    assert!(e.contains("project scalar properties"), "{e}");
    // Mixed list elements.
    let e = arrow_err(&db, "RETURN [1, 'a'] AS bad");
    assert!(e.contains("mixed list element types"), "{e}");
}

#[test]
fn stats_ride_along_and_options_apply() {
    let db = Database::in_memory().unwrap();
    let reader = db
        .query_arrow(
            "CREATE (:N {i: 1})",
            &HashMap::new(),
            &ExecutionOptions::default(),
            1024,
        )
        .unwrap();
    assert_eq!(reader.stats.nodes_created, 1);

    for _ in 0..5 {
        db.execute("CREATE (:N {i: 2})").unwrap();
    }
    let options = ExecutionOptions {
        max_result_rows: Some(2),
        ..Default::default()
    };
    assert!(db
        .query_arrow("MATCH (n:N) RETURN n.i", &HashMap::new(), &options, 1024)
        .is_err());
}

#[test]
fn ffi_stream_round_trips() {
    // Export through the C Data Interface stream and import it back --
    // the exact contract marsdb-capi and the PyCapsule protocol expose.
    use arrow_array::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};

    let db = Database::in_memory().unwrap();
    db.execute("CREATE (:N {i: 9223372036854775807}), (:N {i: 2})")
        .unwrap();
    let reader = db
        .query_arrow(
            "MATCH (n:N) RETURN n.i AS i",
            &HashMap::new(),
            &ExecutionOptions::default(),
            1,
        )
        .unwrap();
    let mut ffi = FFI_ArrowArrayStream::new(Box::new(reader));
    let imported = unsafe { ArrowArrayStreamReader::from_raw(&mut ffi) }.unwrap();
    let batches: Vec<RecordBatch> = imported.collect::<Result<_, _>>().unwrap();
    assert_eq!(batches.len(), 2);
    let all: Vec<i64> = batches
        .iter()
        .flat_map(|b| {
            let a = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
            (0..a.len()).map(|i| a.value(i)).collect::<Vec<_>>()
        })
        .collect();
    assert!(all.contains(&i64::MAX));
}
