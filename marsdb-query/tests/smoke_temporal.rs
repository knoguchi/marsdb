//! Smoke tests: date/time/datetime/duration construction, arithmetic, truncation -- split from the original smoke.rs.

/// ISO 8601 expanded years end-to-end -- the exact queries of TCK
/// Temporal10 [9]/[10], which the i64/civil-calendar date core exists
/// for (chrono's own range caps at ±262k years).
#[test]
fn expanded_year_dates_span_the_full_cypher_range() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN duration.between(date('-999999999-01-01'), date('+999999999-12-31')) AS duration",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P1999999998Y11M30D");

    let result = run(
        &store,
        "RETURN duration.inSeconds(localdatetime('-999999999-01-01'), \
         localdatetime('+999999999-12-31T23:59:59')) AS duration",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT17531639991215H59M59S");

    // Expanded-year date round-trips through storage and toString().
    run(&store, "CREATE (:Event {at: date('+999999999-12-31')})");
    let result = run(&store, "MATCH (e:Event) RETURN toString(e.at), e.at.year");
    assert_eq!(temporal_str(&result.rows[0][0]), "+999999999-12-31");
    assert_eq!(int_value(&result.rows[0][1]), 999_999_999);
}

mod common;
#[allow(unused_imports)]
use common::*;
use marsdb_graph::GraphStore;
use marsdb_query::{parse, Executor, Value};

#[test]
fn set_updates_property() {
    let store = GraphStore::open_memory().unwrap();
    run(&store, "CREATE (a:Person {name: 'Alice', age: 30})");
    run(&store, "MATCH (n:Person {name: 'Alice'}) SET n.age = 31");
    let result = run(&store, "MATCH (n:Person {name: 'Alice'}) RETURN n.age");
    match &result.rows[0][0] {
        Value::Property(marsdb_graph::PropertyValue::Int(v)) => assert_eq!(*v, 31),
        other => panic!("unexpected value {other:?}"),
    }
}

#[test]
fn variable_length_unbounded_depth_cap_errors_not_truncates() {
    use std::collections::BTreeMap;

    let store = GraphStore::open_memory().unwrap();
    let mut prev = {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(0));
        store.create_node(&["Item"], props).unwrap()
    };
    // 40 hops, past the 30-hop safety cap.
    for i in 1..40 {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), marsdb_graph::PropertyValue::Int(i));
        let next = store.create_node(&["Item"], props).unwrap();
        store
            .create_edge("NEXT", prev, next, BTreeMap::new())
            .unwrap();
        prev = next;
    }

    let stmt = parse("MATCH (n:Item {idx: 0})-[:NEXT*0..]->(m:Item) RETURN m.idx").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("depth cap"),
        "expected a depth-cap error, got: {err}"
    );
}

/// A temporal-valued `$param`, in ordinary expression position -- there's
/// no temporal *literal* syntax to substitute one into (same "no literal
/// syntax" gap `list_valued_parameters_...` documents for lists), so
/// `substitute_params` rewrites it into a call to the matching
/// constructor (`date(...)`, `duration(...)`, ...) over its formatted
/// string instead. All 6 temporal `PropertyValue` variants, plus nested
/// inside a list/map -- the shape a bulk-load script binding a `$rows`
/// param full of dated records actually needs.
#[test]
fn temporal_valued_parameters_substitute_into_a_constructor_call() {
    use marsdb_graph::PropertyValue;
    use std::collections::HashMap;

    let store = GraphStore::open_memory().unwrap();
    let cases: Vec<(&str, PropertyValue)> = vec![
        ("date('2020-06-15')", date(2020, 6, 15)),
        (
            "duration('P1Y2M3D')",
            PropertyValue::Duration {
                months: 14,
                days: 3,
                seconds: 0,
                nanos: 0,
            },
        ),
        (
            "localtime('12:34:56')",
            PropertyValue::LocalTime(45_296_000_000_000),
        ),
        (
            "time('12:34:56+02:00')",
            PropertyValue::Time {
                nanos_of_day: 45_296_000_000_000,
                offset_seconds: 7200,
            },
        ),
        (
            "localdatetime('2020-06-15T12:34:56')",
            PropertyValue::LocalDateTime {
                epoch_seconds: date_time_epoch(2020, 6, 15, 12, 34, 56),
                nanos: 0,
            },
        ),
        (
            "datetime('2020-06-15T12:34:56+02:00')",
            PropertyValue::DateTime {
                epoch_seconds: date_time_epoch(2020, 6, 15, 12, 34, 56) - 7200,
                nanos: 0,
                zone: marsdb_graph::TzId::Offset(7200),
            },
        ),
        // Named-zone `DateTime` -- offset and zone name can drift apart
        // (only the offset feeds `epoch_seconds`, `zone` is carried for
        // display/round-tripping), the hairier of the two DateTime shapes.
        (
            "datetime('2020-06-15T12:34:56+02:00[Europe/Stockholm]')",
            PropertyValue::DateTime {
                epoch_seconds: date_time_epoch(2020, 6, 15, 10, 34, 56),
                nanos: 0,
                zone: marsdb_graph::TzId::Named("Europe/Stockholm".to_string()),
            },
        ),
        // Mixed-sign `Duration` -- each component prints its own sign
        // (`P-6M-15D`), not one shared prefix (see `format_duration`'s
        // docs) -- the other hairy round-trip.
        (
            "duration('P-6M-15D')",
            PropertyValue::Duration {
                months: -6,
                days: -15,
                seconds: 0,
                nanos: 0,
            },
        ),
    ];

    for (expected_expr, param_value) in cases {
        let mut params = HashMap::new();
        params.insert("x".to_string(), param_value);
        let mut actual = parse("RETURN $x AS v").unwrap();
        marsdb_query::substitute_params(&mut actual, &params).unwrap();
        let actual_result = Executor::new(&store).execute(&actual).unwrap();

        let expected = parse(&format!("RETURN {expected_expr} AS v")).unwrap();
        let expected_result = Executor::new(&store).execute(&expected).unwrap();

        assert_eq!(
            format!("{:?}", actual_result.rows[0][0]),
            format!("{:?}", expected_result.rows[0][0]),
            "param substitution for {expected_expr} didn't match a literal call to it"
        );
    }

    // Nested inside a list-of-maps -- exactly the `UNWIND $rows AS row`
    // bulk-load shape this was fixed for.
    let mut row = std::collections::BTreeMap::new();
    row.insert("born".to_string(), date(1984, 10, 11));
    let rows = PropertyValue::List(vec![PropertyValue::Map(row)]);
    let mut params = HashMap::new();
    params.insert("rows".to_string(), rows);
    let mut stmt = parse("UNWIND $rows AS row RETURN row.born AS v").unwrap();
    marsdb_query::substitute_params(&mut stmt, &params).unwrap();
    let result = Executor::new(&store).execute(&stmt).unwrap();
    let expected = Executor::new(&store)
        .execute(&parse("RETURN date('1984-10-11') AS v").unwrap())
        .unwrap();
    assert_eq!(
        format!("{:?}", result.rows[0][0]),
        format!("{:?}", expected.rows[0][0])
    );
}

#[test]
fn date_construct_from_calendar_map() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN date({year: 1984, month: 10, day: 11}) AS d");
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11");
}

/// ISO week-date construction (`{year, week, dayOfWeek}`) -- TCK's
/// Temporal1 [1]. `year` here is the ISO week-numbering year, which can
/// diverge from the resulting date's calendar year near a year boundary.
#[test]
fn date_construct_from_week_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date({year: 1816, week: 1}), date({year: 1818, week: 53}), \
         date({dayOfWeek: 2, year: 1817, week: 1})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1816-01-01");
    assert_eq!(temporal_str(&row[1]), "1818-12-28");
    assert_eq!(temporal_str(&row[2]), "1816-12-31");

    // `week`/`dayOfWeek` default from a `date` base's own weekYear/week/
    // dayOfWeek, same as month/day already default from a base.
    let result = run(
        &store,
        "RETURN date({date: date('1816-12-30'), week: 2, dayOfWeek: 3}), \
         date({date: date('1816-12-31'), week: 2})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1817-01-08");
    assert_eq!(temporal_str(&row[1]), "1817-01-07");
}

/// Ordinal-date (`{year, ordinalDay}`) and quarter-date
/// (`{year, quarter, dayOfQuarter}`) construction -- TCK's Temporal1 [4].
#[test]
fn date_construct_from_ordinal_and_quarter_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date({year: 1984, ordinalDay: 202}), \
         date({year: 1984, quarter: 3, dayOfQuarter: 45}), \
         date({year: 1984, quarter: 3})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1984-07-20");
    assert_eq!(temporal_str(&row[1]), "1984-08-14");
    assert_eq!(temporal_str(&row[2]), "1984-07-01");
}

/// A bare positional temporal argument to `date()`/`localtime()`/
/// `time()`/`localdatetime()` projects the relevant part of a
/// *different* temporal type, same as the equivalent `{date: other}`/
/// `{time: other}`/`{datetime: other}` map form -- TCK's Temporal3
/// [1]/[2]/[3]/[7].
#[test]
fn temporal_constructors_accept_a_cross_type_positional_argument() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 11, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN date(other)",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");

    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(localtime(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:00");

    let result = run(
        &store,
        "WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS other \
         RETURN toString(time(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123Z");

    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(localdatetime(other))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11T12:00");
}

#[test]
fn date_construct_from_string_forms() {
    let store = GraphStore::open_memory().unwrap();
    // Temporal2 scenario [1] -- the plain calendar forms; week-date/
    // ordinal-date forms are covered separately, see
    // date_string_week_and_ordinal_date_forms_parse.
    let result = run(
        &store,
        "RETURN date('2015-07-21'), date('20150721'), date('2015-07'), date('201507'), date('2015')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "2015-07-21");
    assert_eq!(temporal_str(&row[1]), "2015-07-21");
    assert_eq!(temporal_str(&row[2]), "2015-07-01");
    assert_eq!(temporal_str(&row[3]), "2015-07-01");
    assert_eq!(temporal_str(&row[4]), "2015-01-01");
}

/// ISO week-date (`YYYY-Www[-D]`/`YYYYWww[D]`) and ordinal-date
/// (`YYYY-DDD`/`YYYYDDD`) string forms -- TCK's Date2/Date3, real Cypher
/// parses these the same as the equivalent `{week, dayOfWeek}`/
/// `{ordinalDay}` map construction (`temporal::parse_week_or_ordinal_date`).
#[test]
fn date_string_week_and_ordinal_date_forms_parse() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN date('2015W302'), date('2015-W30-2'), date('2015W30'), date('2015-W30'), \
         date('2015202'), date('2015-202')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "2015-07-21");
    assert_eq!(temporal_str(&row[1]), "2015-07-21");
    assert_eq!(temporal_str(&row[2]), "2015-07-20");
    assert_eq!(temporal_str(&row[3]), "2015-07-20");
    assert_eq!(temporal_str(&row[4]), "2015-07-21");
    assert_eq!(temporal_str(&row[5]), "2015-07-21");
}

#[test]
fn temporal_constructors_reject_malformed_inputs_and_wrong_arity() {
    let store = GraphStore::open_memory().unwrap();
    for query in [
        "RETURN date('123é4')",
        "RETURN duration('Pgarbage')",
        "RETURN duration('P1Ygarbage')",
        "RETURN date('2020', '2021')",
        "RETURN duration('P1D', 'P2D')",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }
}

#[test]
fn date_map_requires_in_range_integer_fields() {
    let store = GraphStore::open_memory().unwrap();
    for query in [
        "RETURN date({year: 2020.9, month: 1, day: 2})",
        "RETURN date({year: 4294969280, month: 1, day: 1})",
        "RETURN date({year: 2020, month: 4294967297, day: 1})",
        "RETURN date({year: 2020, month: 1, day: 4294967297})",
    ] {
        let stmt = parse(query).unwrap();
        assert!(
            Executor::new(&store).execute(&stmt).is_err(),
            "{query} must fail"
        );
    }
}

#[test]
fn date_comparison() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1980, month: 12, day: 24}) AS x, date({year: 1984, month: 10, day: 11}) AS d \
         RETURN x > d, x < d, x >= d, x <= d, x = d",
    );
    let row = &result.rows[0];
    assert!(!boolean(&row[0]));
    assert!(boolean(&row[1]));
    assert!(!boolean(&row[2]));
    assert!(boolean(&row[3]));
    assert!(!boolean(&row[4]));
}

#[test]
fn date_component_access_via_stored_property() {
    // Temporal5 scenario [1]'s exact shape: construct via CREATE (so the
    // Date round-trips through storage), then access components off a
    // WITH-projected scalar.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Val {date: date({year: 1984, month: 10, day: 11})})",
    );
    let result = run(
        &store,
        "MATCH (v:Val) WITH v.date AS d \
         RETURN d.year, d.quarter, d.month, d.week, d.weekYear, d.day, d.ordinalDay, d.weekDay, d.dayOfQuarter",
    );
    let row = &result.rows[0];
    assert_eq!(int(&row[0]), 1984);
    assert_eq!(int(&row[1]), 4);
    assert_eq!(int(&row[2]), 10);
    assert_eq!(int(&row[3]), 41);
    assert_eq!(int(&row[4]), 1984);
    assert_eq!(int(&row[5]), 11);
    assert_eq!(int(&row[6]), 285);
    assert_eq!(int(&row[7]), 4);
    assert_eq!(int(&row[8]), 11);
}

#[test]
fn duration_construct_from_map_normalizes_and_formats() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN duration({days: 14, hours: 16, minutes: 12}), \
                duration({months: 5, days: 1.5}), \
                duration({months: 0.75}), \
                duration({weeks: 2.5}), \
                duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70})",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P14DT16H12M");
    assert_eq!(temporal_str(&row[1]), "P5M1DT12H");
    assert_eq!(temporal_str(&row[2]), "P22DT19H51M49.5S");
    assert_eq!(temporal_str(&row[3]), "P17DT12H");
    assert_eq!(temporal_str(&row[4]), "P12Y5M14DT16H13M10S");
}

#[test]
fn duration_construct_from_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN duration('P14DT16H12M'), duration('P0.75M'), duration('P2.5W')",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P14DT16H12M");
    assert_eq!(temporal_str(&row[1]), "P22DT19H51M49.5S");
    assert_eq!(temporal_str(&row[2]), "P17DT12H");
}

#[test]
fn duration_equality_is_component_wise_not_calendar_aware() {
    // Temporal7 scenario [6] -- two durations with the same total months/
    // days/seconds/nanos are equal even if their *inputs* differed
    // (60s + 13m == 70s + 12m), but a different `days` component makes
    // them unequal even when hours "look like" they'd make up the gap.
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70}) AS x \
         RETURN x = duration({years: 12, months: 5, days: 14, hours: 16, minutes: 13, seconds: 10}), \
                x = duration({years: 12, months: 5, days: 13, hours: 40, minutes: 13, seconds: 10})",
    );
    let row = &result.rows[0];
    assert!(boolean(&row[0]));
    assert!(!boolean(&row[1]));
}

#[test]
fn date_plus_and_minus_duration() {
    // Temporal8 scenario [1] row 1.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Duration {dur: duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, \
         seconds: 70, nanoseconds: 2})})",
    );
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS x \
         MATCH (d:Duration) RETURN x + d.dur AS sum, x - d.dur AS diff",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1997-03-25");
    assert_eq!(temporal_str(&row[1]), "1972-04-27");
}

#[test]
fn date_plus_duration_with_fractional_components_carries_extra_day() {
    // Regression guard for the bug an earlier version of
    // `add_duration_to_date` had: dropping a duration's `seconds`/`nanos`
    // remainder outright instead of folding any *whole* extra day out of
    // it. Temporal8 scenario [1] row 3.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Duration {dur: duration({years: 12.5, months: 5.5, days: 14.5, hours: 16.5, \
         minutes: 12.5, seconds: 70.5, nanoseconds: 3})})",
    );
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS x \
         MATCH (d:Duration) RETURN x + d.dur AS sum, x - d.dur AS diff",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "1997-10-11");
    assert_eq!(temporal_str(&row[1]), "1971-10-12");
}

#[test]
fn duration_plus_minus_scale() {
    // Temporal8 scenarios [6]/[7].
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration({years: 12, months: 5, days: 14, hours: 16, minutes: 12, seconds: 70, nanoseconds: 1}) AS x \
         RETURN x + x, x - x, x * 2, x / 2",
    );
    let row = &result.rows[0];
    assert_eq!(temporal_str(&row[0]), "P24Y10M28DT32H26M20.000000002S");
    assert_eq!(temporal_str(&row[1]), "PT0S");
    assert_eq!(temporal_str(&row[2]), "P24Y10M28DT32H26M20.000000002S");
    assert_eq!(temporal_str(&row[3]), "P6Y2M22DT13H21M8S");
}

#[test]
fn temporal_arithmetic_overflow_returns_errors_instead_of_panicking() {
    let store = GraphStore::open_memory().unwrap();
    let cases = [
        "RETURN duration({months: 9223372036854775807}) + duration({months: 1})",
        "RETURN duration({months: 9223372036854775807}) - duration({months: -1})",
        "RETURN date('9999-12-31') + duration({days: 9223372036854775807})",
        "RETURN date('9999-12-31') - duration({days: -9223372036854775808})",
    ];

    for cypher in cases {
        let stmt = parse(cypher).unwrap();
        let err = Executor::new(&store).execute(&stmt).unwrap_err();
        assert!(
            err.to_string().contains("overflow") || err.to_string().contains("out-of-range"),
            "unexpected error for {cypher:?}: {err}"
        );
    }
}

#[test]
fn duration_component_access() {
    // Temporal5 scenario [7].
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE (:Val {date: duration({years: 1, months: 4, days: 10, hours: 1, minutes: 1, seconds: 1, \
         nanoseconds: 111111111})})",
    );
    let result = run(
        &store,
        "MATCH (v:Val) WITH v.date AS d \
         RETURN d.years, d.quarters, d.months, d.weeks, d.days, d.hours, d.minutes, d.seconds, \
                d.milliseconds, d.microseconds, d.nanoseconds",
    );
    let row = &result.rows[0];
    assert_eq!(int(&row[0]), 1);
    assert_eq!(int(&row[1]), 5);
    assert_eq!(int(&row[2]), 16);
    assert_eq!(int(&row[3]), 1);
    assert_eq!(int(&row[4]), 10);
    assert_eq!(int(&row[5]), 1);
    assert_eq!(int(&row[6]), 61);
    assert_eq!(int(&row[7]), 3661);
    assert_eq!(int(&row[8]), 3_661_111);
    assert_eq!(int(&row[9]), 3_661_111_111);
    assert_eq!(int(&row[10]), 3_661_111_111_111);
}

#[test]
fn stored_date_survives_the_storage_round_trip() {
    // Temporal4 scenario [1] -- a Date stored as a node property comes
    // back as the same Date (not degraded to a plain Int/String), the
    // real reason PropertyValue got a first-class Date variant instead of
    // reusing Int/String -- see PropertyValue's own doc comment.
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE ({created: date({year: 1984, month: 10, day: 11})})",
    );
    let result = run(&store, "MATCH (n) RETURN n.created");
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11");
}

// -- Temporal: LocalTime/Time/LocalDateTime/DateTime -------------------
//
// Real shapes pulled directly from the TCK's Temporal1/2/5/7/8 feature
// files. Scope: fixed UTC offsets only (`'+01:00'`) -- named timezones
// (`'Europe/Stockholm'`) are a documented gap, covered by
// `datetime_named_timezone_is_rejected_not_silently_wrong` below.

#[test]
fn local_date_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localdatetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-11T12:31:14");

    let result = run(
        &store,
        "RETURN toString(localdatetime('2015-07-21T21:40:32.142')) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "2015-07-21T21:40:32.142");
}

#[test]
fn date_time_construct_from_map_and_string() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'})) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14+01:00"
    );

    let result = run(
        &store,
        "RETURN toString(datetime('2015-07-21T21:40:32.142+0100')) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "2015-07-21T21:40:32.142+01:00"
    );
}

#[test]
fn datetime_named_timezone_construction_and_parsing() {
    let store = GraphStore::open_memory().unwrap();
    // Map construction, string parsing (with and without an explicit
    // offset alongside the bracket), and DST-aware resolution (October
    // = standard time, ordinalDay 202 = July = summer time) -- TCK's
    // Temporal1 [10] / Temporal2 [6].
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, \
         second: 14, nanosecond: 645876123, timezone: 'Europe/Stockholm'})), \
         toString(datetime({year: 1984, ordinalDay: 202, hour: 12, minute: 31, second: 14, \
         nanosecond: 645876123, timezone: 'Europe/Stockholm'})), \
         toString(datetime('2015-07-21T21:40:32.142+02:00[Europe/Stockholm]')), \
         toString(datetime('2015-07-21T21:40:32.142[Europe/London]'))",
    );
    let row = &result.rows[0];
    assert_eq!(
        temporal_str(&row[0]),
        "1984-10-11T12:31:14.645876123+01:00[Europe/Stockholm]"
    );
    assert_eq!(
        temporal_str(&row[1]),
        "1984-07-20T12:31:14.645876123+02:00[Europe/Stockholm]"
    );
    assert_eq!(
        temporal_str(&row[2]),
        "2015-07-21T21:40:32.142+02:00[Europe/Stockholm]"
    );
    // No explicit offset at all -- derived purely from the zone (BST).
    assert_eq!(
        temporal_str(&row[3]),
        "2015-07-21T21:40:32.142+01:00[Europe/London]"
    );

    // `.timezone` is the zone name; `.offset` is the resolved offset --
    // the two diverge only for a `Named` zone. TCK's Temporal5 [6].
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 11, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS d \
         RETURN d.timezone, d.offset",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "Europe/Stockholm");
    assert_eq!(temporal_str(&result.rows[0][1]), "+01:00");
}

/// `TIME` has no calendar date, so a named zone's DST-dependent offset
/// has nothing to resolve against -- unlike `DATETIME`, it still only
/// accepts a fixed UTC offset. A real, deliberately narrow scope line,
/// not a silent wrong answer.
#[test]
fn time_named_timezone_is_rejected_not_silently_wrong() {
    let store = GraphStore::open_memory().unwrap();
    let stmt = parse("RETURN time('21:40:32.142[Europe/Stockholm]')").unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("named timezone"),
        "expected a named-timezone error, got: {err}"
    );

    let stmt2 = parse("RETURN time({hour: 12, timezone: 'Europe/Stockholm'})").unwrap();
    let err2 = Executor::new(&store).execute(&stmt2).unwrap_err();
    assert!(
        err2.to_string().contains("named timezone"),
        "expected a named-timezone error, got: {err2}"
    );
}

/// Projecting a `Named`-zone base with an *explicit* `timezone` override
/// shifts the wall-clock to preserve the same instant -- the target
/// offset is resolved *for the actual target date* (which the `day`
/// override can move to a different DST period than the base's own
/// date), not the base's original instant. A real bug found and fixed
/// this session: an earlier version resolved both the "from" and "to"
/// offsets against stale/inconsistent dates, producing wrong instants.
/// TCK's Temporal3 [9]/[10] (a representative sample of the row shapes).
#[test]
fn datetime_shift_into_named_zone_resolves_offsets_for_the_target_date() {
    let store = GraphStore::open_memory().unwrap();
    // Time-with-offset base, fresh year/month/day, explicit shift.
    let result = run(
        &store,
        "WITH time({hour: 12, minute: 31, second: 14, microsecond: 645876, timezone: '+01:00'}) AS other \
         RETURN toString(datetime({year: 1984, month: 10, day: 11, time: other, second: 42, \
         timezone: 'Pacific/Honolulu'}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T01:31:42.645876-10:00[Pacific/Honolulu]"
    );
    // Named-zone-base shifted into a *different* named zone, where the
    // `day` override moves the result across a DST boundary for the
    // *base's own* zone too (Stockholm: standard time in October, but
    // summer time by the overridden March 28) -- the "from" offset used
    // for the shift must reflect the *target* date, not the base's
    // original (October) instant.
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS otherDate, \
         datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS otherTime \
         RETURN toString(datetime({date: otherDate, time: otherTime, day: 28, second: 42, \
         timezone: 'Pacific/Honolulu'}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-03-28T00:00:42-10:00[Pacific/Honolulu]"
    );
}

/// With *no* explicit `timezone` override, a `Named`-zone base's zone
/// identity is preserved as-is and the wall-clock is *not* shifted, even
/// if a `day` override moves the result across a DST boundary for that
/// same zone -- the displayed offset is simply re-resolved for the new
/// date, the local time itself never changes. A real bug found and fixed
/// this session: an earlier version always re-resolved and shifted
/// whenever the zone's real offset differed for the new date, even
/// without an explicit override. TCK's Temporal3 [10] rows 336/337.
#[test]
fn datetime_no_override_preserves_zone_identity_without_shifting() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS otherDate, \
         datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: 'Europe/Stockholm'}) AS otherTime \
         RETURN toString(datetime({date: otherDate, time: otherTime, day: 28, second: 42}))",
    );
    // Same 12:00 wall-clock as the base, just re-displayed with the
    // zone's real (now summer-time) offset for the new date -- not
    // shifted to a different hour.
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-03-28T12:00:42+02:00[Europe/Stockholm]"
    );
}

/// `datetime(otherLocalDateTime)` -- a bare `LocalDateTime` argument has
/// no zone of its own, defaults to UTC -- TCK's Temporal3 [11].
#[test]
fn datetime_construct_from_bare_local_date_time_defaults_to_utc() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, \
         millisecond: 645}) AS other \
         RETURN toString(datetime(other)), toString(datetime({datetime: other}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-03-07T12:31:14.645Z");
    assert_eq!(temporal_str(&result.rows[0][1]), "1984-03-07T12:31:14.645Z");
}

/// `Time`'s comparison is by the UTC-equivalent instant-of-day, not the
/// raw wall-clock reading -- Temporal7 [3].
#[test]
fn time_comparison_is_instant_based_not_wall_clock() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH time({hour: 10, minute: 0, timezone: '+01:00'}) AS x, \
              time({hour: 9, minute: 35, second: 14, nanosecond: 645876123, timezone: '+00:00'}) AS d \
         RETURN x > d, x < d, x >= d, x <= d, x = d",
    );
    let bools: Vec<bool> = result.rows[0].iter().map(boolean).collect();
    assert_eq!(bools, vec![false, true, false, true, false]);
}

/// Two `DateTime`s at the same instant but different offsets are equal
/// -- real Cypher's rule (see `PropertyValue::DateTime`'s doc comment),
/// not the derived structural equality every other `PropertyValue`
/// variant gets.
#[test]
fn date_time_equality_is_instant_based_not_structural() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 2020, month: 1, day: 1, hour: 1, minute: 0, second: 0, timezone: '+01:00'}) AS x, \
              datetime({year: 2020, month: 1, day: 1, hour: 0, minute: 0, second: 0, timezone: '+00:00'}) AS d \
         RETURN x = d",
    );
    assert!(boolean(&result.rows[0][0]));
}

/// `DateTime`'s calendar/clock component access (`.hour`, `.day`, ...)
/// reflects the *local* (offset-adjusted) wall-clock reading that was
/// written, not the underlying UTC instant -- `epochSeconds`/
/// `epochMillis` are the one exception, always UTC. Temporal5 [4].
#[test]
fn date_time_component_access_uses_local_reading_except_epoch_fields() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}) AS d \
         RETURN d.year, d.month, d.day, d.hour, d.minute, d.second, d.timezone, d.offset, d.offsetSeconds, d.offsetMinutes",
    );
    let ints: Vec<i64> = [0usize, 1, 2, 3, 4, 5]
        .iter()
        .map(|&i| match &result.rows[0][i] {
            Value::Property(marsdb_graph::PropertyValue::Int(n)) => *n,
            other => panic!("unexpected value {other:?}"),
        })
        .collect();
    assert_eq!(ints, vec![1984, 10, 11, 12, 31, 14]);
    assert_eq!(temporal_str(&result.rows[0][6]), "+01:00");
    assert_eq!(temporal_str(&result.rows[0][7]), "+01:00");
}

/// `date({date: other, ...overrides})` -- projects year/month/day from
/// another temporal value, individual keys override on top. Temporal3
/// [1].
#[test]
fn date_projects_from_another_temporal_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 11, day: 11}) AS other \
         RETURN toString(date({date: other})), toString(date({date: other, year: 28})), \
                toString(date({date: other, day: 28}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");
    assert_eq!(temporal_str(&result.rows[0][1]), "0028-11-11");
    assert_eq!(temporal_str(&result.rows[0][2]), "1984-11-28");

    // Projects from LocalDateTime/DateTime too, not just Date.
    let result = run(
        &store,
        "WITH localdatetime({year: 1984, month: 11, day: 11, hour: 12}) AS other RETURN toString(date({date: other}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-11-11");
}

/// `localtime({time: other, ...overrides})` -- Temporal3 [2].
#[test]
fn local_time_projects_from_another_temporal_value() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS other \
         RETURN toString(localtime({time: other})), toString(localtime({time: other, second: 42}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876123");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:31:42.645876123");
}

/// `time({time: other, timezone: ...})` -- when the override timezone
/// differs from the base's own offset, the wall-clock shifts to
/// preserve the same instant (real Cypher's rule, not just relabeling
/// the offset) -- Temporal3 [3].
#[test]
fn time_projection_with_different_timezone_shifts_wall_clock_to_preserve_instant() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH time({hour: 12, minute: 31, second: 14, microsecond: 645876, timezone: '+01:00'}) AS other \
         RETURN toString(time({time: other})), toString(time({time: other, timezone: '+05:00'}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "12:31:14.645876+01:00");
    assert_eq!(temporal_str(&result.rows[0][1]), "16:31:14.645876+05:00");

    // An explicit field override applies *after* the zone shift, not
    // before -- Temporal3 [3]'s own compound example.
    let result = run(
        &store,
        "WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other \
         RETURN toString(time({time: other, second: 42, timezone: '+05:00'}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "16:00:42+05:00");
}

/// `localdatetime({date: ..., time: ..., ...overrides})` -- combining a
/// date projected from one value and a time from another (or literal
/// fields), Temporal3 [4]/[5]/[6].
#[test]
fn local_date_time_projects_date_and_time_independently() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH date({year: 1984, month: 10, day: 11}) AS d, \
              localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS t \
         RETURN toString(localdatetime({date: d, time: t})), \
                toString(localdatetime({date: d, time: t, day: 28, second: 42})), \
                toString(localdatetime({date: d, hour: 10, minute: 10, second: 10}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14.645876123"
    );
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-28T12:31:42.645876123"
    );
    assert_eq!(temporal_str(&result.rows[0][2]), "1984-10-11T10:10:10");
}

/// `datetime(...) + duration(...)` -- real calendar month arithmetic on
/// the *local* reading, seconds/nanos carrying across day boundaries
/// (unlike `Date`, which has no time-of-day to carry into). Temporal8.
#[test]
fn date_time_plus_duration() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}) \
                          + duration({months: 1, days: 5, hours: 2})) AS r",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-11-16T14:31:14+01:00"
    );
}

/// `Time`/`LocalTime` + `Duration` wraps at the 24h boundary -- there's
/// no calendar to carry an extra day into.
#[test]
fn time_plus_duration_wraps_at_midnight() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(time({hour: 23, minute: 0, timezone: 'Z'}) + duration({hours: 2})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "01:00Z");
}

/// `duration.between(a, b)` -- real calendar month arithmetic plus a
/// day/second/nanos remainder, mixing every pair of the 5 non-Duration
/// temporal types. Temporal10 [1]/[2].
#[test]
fn duration_between_mixed_types() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.between(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P30Y8M13D");

    // Either side lacking a date degrades to a plain time-of-day
    // difference -- the date side's real calendar date never enters
    // the calculation at all.
    let result = run(
        &store,
        "RETURN toString(duration.between(date('1984-10-11'), localtime('16:30'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT16H30M");

    let result = run(
        &store,
        "RETURN toString(duration.between(localdatetime('2015-07-21T21:40:32.142'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P-27DT-21H-40M-32.142S");
}

/// Two `DateTime`s at *different* offsets -- the month/day/second
/// breakdown must account for the real offset delta, not just the raw
/// local wall-clock digits (found as a real bug: naive local-to-local
/// subtraction here gave `P11M29DT23H59M55.999S` instead of the
/// correct `P1YT59M55.999S`, off by exactly the 1h offset difference).
/// Temporal10 [2].
#[test]
fn duration_between_two_datetimes_with_different_offsets_accounts_for_the_offset_delta() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.between(datetime('2014-07-21T21:40:36.143+0200'), \
                                           datetime('2015-07-21T21:40:32.142+0100'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P1YT59M55.999S");
}

/// The same offset-reconciliation rule applies even in the time-only
/// "degrade" mode (one side has no date) when *both* operands still
/// carry a real offset (`Time`/`DateTime`) -- found as a second real
/// bug alongside the one above.
#[test]
fn duration_between_time_only_mode_still_accounts_for_offset_when_both_sides_have_one() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(datetime('2014-07-21T21:40:36.143+0200'), \
                                             time('16:30+0100'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-4H-10M-36.143S");
}

/// `.inMonths`/`.inDays`/`.inSeconds` collapse the same underlying
/// computation into a single bucket -- `.inMonths` keeps just the
/// calendar month count, `.inDays`/`.inSeconds` discard the month
/// optimization entirely and use the *raw* total elapsed time (so
/// `.inDays` on a date+time target truncates away any sub-day
/// remainder rather than carrying it as leftover seconds). Temporal10
/// [3]/[4]/[5].
#[test]
fn duration_in_months_days_seconds_collapse_to_a_single_bucket() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inMonths(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P30Y8M");

    let result = run(
        &store,
        "RETURN toString(duration.inDays(date('1984-10-11'), localdatetime('2016-07-21T21:45:22.142'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "P11606D");

    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(date('1984-10-11'), date('2015-06-24'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT269112H");
}

/// `duration.between`'s own remainder-decomposition edge case: a
/// negative sub-second-only difference must still round-trip through
/// `toString` correctly (a real pre-existing invariant --
/// `format_seconds_fraction`'s `(0, -500_000_000) -> "-0.5"` case --
/// exercised here via the actual `duration.between` code path, not a
/// hand-built `Duration`). Temporal10 [6].
#[test]
fn duration_in_seconds_negative_sub_second_only_difference() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(duration.inSeconds(localdatetime('2014-07-21T21:40:36.143'), \
                                             localdatetime('2014-07-21T21:40:36.142'))) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-0.001S");
}

/// `date.truncate(unit, value, map)` -- calendar-unit truncation
/// (`millennium`/`century`/`decade`/`year`/`quarter`/`month`/`week`/
/// `weekYear`/`day`), plus optional field overrides applied after
/// truncation. Temporal9 [1].
#[test]
fn date_truncate_calendar_units() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('millennium', date({year: 2017, month: 10, day: 11}), {day: 2})), \
                toString(date.truncate('century', date({year: 1984, month: 10, day: 11}), {})), \
                toString(date.truncate('decade', date({year: 1984, month: 10, day: 11}), {})), \
                toString(date.truncate('quarter', date({year: 1984, month: 11, day: 11}), {})), \
                toString(date.truncate('week', date({year: 1984, month: 10, day: 11}), {}))",
    );
    let strs: Vec<String> = (0..5).map(|i| temporal_str(&result.rows[0][i])).collect();
    assert_eq!(
        strs,
        vec![
            "2000-01-02",
            "1900-01-01",
            "1980-01-01",
            "1984-10-01",
            "1984-10-08"
        ]
    );
}

/// `weekYear` truncation crosses a real ISO week-year boundary (Jan 1
/// 1984 belongs to ISO week-year 1983) -- Temporal9 [1].
#[test]
fn date_truncate_week_year_crosses_iso_boundary() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('weekYear', datetime({year: 1984, month: 1, day: 1, hour: 12, timezone: '+01:00'}), {})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1983-01-03");
}

/// `datetime.truncate`/`localdatetime.truncate` -- a calendar-scale
/// unit truncates the date *and* resets the time to midnight; a
/// clock-scale unit leaves the date untouched. Temporal9 [2]/[3].
#[test]
fn date_time_truncate_calendar_vs_clock_units() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(datetime.truncate('millennium', datetime({year: 2017, month: 10, day: 11, hour: 12, minute: 31, second: 14, timezone: '+01:00'}), {day: 2})), \
                toString(localdatetime.truncate('hour', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "2000-01-02T00:00+01:00");
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-11T12:00:00.000000002"
    );
}

/// `localtime.truncate`/`time.truncate` -- clock-only truncation, `time.
/// truncate` inherits the source's offset unless overridden. Temporal9
/// [4]/[5].
#[test]
fn local_time_and_time_truncate() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localtime.truncate('day', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2})), \
                toString(time.truncate('hour', time({hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {}))",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "00:00:00.000000002");
    assert_eq!(temporal_str(&result.rows[0][1]), "12:00+01:00");
}

/// `date.truncate('week', d, {dayOfWeek: N})` -- the `dayOfWeek`
/// override moves within the truncated result's own ISO week (found
/// as a real bug: `apply_date_overrides` didn't recognize `dayOfWeek`
/// at all and silently ignored it instead of applying it or erroring).
/// Temporal9 [1].
#[test]
fn date_truncate_day_of_week_override() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(date.truncate('week', date({year: 1984, month: 10, day: 11}), {dayOfWeek: 2})) AS r",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "1984-10-09");
}

/// A `.truncate()` map with a field the target type has no slot for
/// (`hour` on a `date.truncate` result, which is a bare `Date`) is a
/// real error, not silently ignored.
#[test]
fn date_truncate_rejects_a_time_only_override_field() {
    let store = GraphStore::open_memory().unwrap();
    let stmt =
        parse("RETURN date.truncate('year', date({year: 1984, month: 10, day: 11}), {hour: 5})")
            .unwrap();
    let err = Executor::new(&store).execute(&stmt).unwrap_err();
    assert!(
        err.to_string().contains("unrecognized field"),
        "expected an unrecognized-field error, got: {err}"
    );
}

/// A `.truncate()` map overriding *only* `nanosecond` must keep the
/// truncated base's own millisecond/microsecond digits, not silently
/// reset them to zero -- found as a real bug (`{nanosecond: 2}` alone
/// was dropping the base's `.645` millisecond value entirely instead
/// of producing `.645000002`). Temporal9 [2]-[5].
#[test]
fn truncate_sub_second_override_keeps_the_bases_other_digits() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN toString(localdatetime.truncate('millisecond', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2})), \
                toString(localdatetime.truncate('microsecond', datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 645876123, timezone: '+01:00'}), {nanosecond: 2}))",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1984-10-11T12:31:14.645000002"
    );
    assert_eq!(
        temporal_str(&result.rows[0][1]),
        "1984-10-11T12:31:14.645876002"
    );
}

#[test]
fn stored_time_and_date_time_survive_the_storage_round_trip() {
    let store = GraphStore::open_memory().unwrap();
    run(
        &store,
        "CREATE ({t: time({hour: 9, minute: 0, timezone: '+02:00'})})",
    );
    let result = run(&store, "MATCH (n) RETURN n.t");
    assert_eq!(temporal_str(&result.rows[0][0]), "09:00+02:00");
}

/// `duration.between(...)`'s own component accessors must read the
/// stored `seconds`/`nanos` fields directly, not recombine them into one
/// signed total and re-split (which would silently reintroduce a
/// negative `nanos`, breaking the "nanos always non-negative, sign
/// lives in seconds" storage invariant). TCK's Temporal10 [1].
#[test]
fn duration_component_accessors_read_raw_fields_not_a_resplit_total() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "WITH duration.between(localdatetime('2018-01-02T10:00:00.1'), \
         localdatetime('2018-01-01T10:00:00.2')) AS dur \
         RETURN dur, dur.days, dur.seconds, dur.nanosecondsOfSecond",
    );
    assert_eq!(temporal_str(&result.rows[0][0]), "PT-23H-59M-59.9S");
    assert_eq!(int(&result.rows[0][1]), 0);
    assert_eq!(int(&result.rows[0][2]), -86400);
    assert_eq!(int(&result.rows[0][3]), 100_000_000);
}

/// `duration('P2012-02-02T14:37:21.545')` -- ISO-8601's alternate
/// "combined date-time" duration representation (date/time formatted
/// like a calendar date/time-of-day, each field meaning "this many
/// years/months/days/hours/minutes/seconds"), not the more common
/// `PnYnMnD` form. TCK's Temporal2 [7].
#[test]
fn duration_parses_the_combined_date_time_alternate_form() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(&store, "RETURN duration('P2012-02-02T14:37:21.545') AS d");
    assert_eq!(temporal_str(&result.rows[0][0]), "P2012Y2M2DT14H37M21.545S");
}

/// `datetime.fromepoch(seconds, nanos)`/`datetime.fromepochmillis(millis)`.
/// TCK's Temporal1 [11].
#[test]
fn datetime_from_epoch_and_epoch_millis() {
    let store = GraphStore::open_memory().unwrap();
    let result = run(
        &store,
        "RETURN datetime.fromepoch(416779, 999999999) AS d1, \
         datetime.fromepochmillis(237821673987) AS d2",
    );
    assert_eq!(
        temporal_str(&result.rows[0][0]),
        "1970-01-05T19:46:19.999999999Z"
    );
    assert_eq!(temporal_str(&result.rows[0][1]), "1977-07-15T13:34:33.987Z");
}
