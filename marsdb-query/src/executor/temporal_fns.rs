//! Temporal builtins (`date()`, `datetime()`, `duration()`, truncation,
//! `duration.between`, component access) and temporal arithmetic.

use super::*;

pub(crate) fn as_date(v: &Value) -> Option<i64> {
    match v {
        Value::Property(PropertyValue::Date(d)) => Some(*d),
        _ => None,
    }
}

pub(crate) fn as_duration(v: &Value) -> Option<temporal::DurationParts> {
    match v {
        Value::Property(PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => Some((*months, *days, *seconds, *nanos)),
        _ => None,
    }
}

pub(crate) fn duration_value((months, days, seconds, nanos): temporal::DurationParts) -> Value {
    Value::Property(PropertyValue::Duration {
        months,
        days,
        seconds,
        nanos,
    })
}

pub(crate) fn as_local_time(v: &Value) -> Option<i64> {
    match v {
        Value::Property(PropertyValue::LocalTime(n)) => Some(*n),
        _ => None,
    }
}

pub(crate) fn as_time(v: &Value) -> Option<(i64, i32)> {
    match v {
        Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => Some((*nanos_of_day, *offset_seconds)),
        _ => None,
    }
}

pub(crate) fn as_local_date_time(v: &Value) -> Option<(i64, i32)> {
    match v {
        Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => Some((*epoch_seconds, *nanos)),
        _ => None,
    }
}

pub(crate) fn as_date_time(v: &Value) -> Option<(i64, i32, temporal::TzId)> {
    match v {
        Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => Some((*epoch_seconds, *nanos, tz_from_graph(zone))),
        _ => None,
    }
}

/// `marsdb_graph::TzId` <-> `temporal::TzId` -- two independent, same-
/// shaped types (`temporal.rs` deliberately doesn't depend on
/// `marsdb_graph`, see its own module doc comment), converted at this
/// storage/query-layer boundary.
pub(crate) fn tz_from_graph(zone: &GraphTzId) -> temporal::TzId {
    match zone {
        GraphTzId::Offset(o) => temporal::TzId::Offset(*o),
        GraphTzId::Named(name) => temporal::TzId::Named(name.clone()),
    }
}

pub(crate) fn tz_to_graph(zone: temporal::TzId) -> GraphTzId {
    match zone {
        temporal::TzId::Offset(o) => GraphTzId::Offset(o),
        temporal::TzId::Named(name) => GraphTzId::Named(name),
    }
}

/// The `Date`/`Duration`/`LocalTime`/`Time`/`LocalDateTime`/`DateTime`
/// cases of `+`/`-`/`*`/`/` -- tried before `apply_arith`'s generic
/// numeric path, since none of these are ever an `ArithNum`. Returns
/// `Ok(None)` (not an error) for any operand-type combination it doesn't
/// recognize, so `apply_arith` falls through to its own "not two
/// numbers" error with the *original* operands in the message, rather
/// than this function needing to duplicate that error text.
///
/// `<temporal> - <temporal>` (real Cypher's `duration.between(...)` is
/// the actual spelling for that, itself out of scope -- see the README)
/// is deliberately *not* handled for any of the 5 non-Duration types,
/// falling through to the same "not two numbers" error a truly
/// nonsensical subtraction would already get.
pub(crate) fn apply_temporal_arith(
    op: ArithOp,
    a: &Value,
    b: &Value,
) -> Result<Option<Value>, QueryError> {
    let date_plus_duration =
        |d: i64, dur: temporal::DurationParts, negate: bool| -> Result<Value, QueryError> {
            let (months, days, seconds, nanos) = dur;
            temporal::add_duration_to_date(d, months, days, seconds, nanos, negate)
                .map(|d| Value::Property(PropertyValue::Date(d)))
                .ok_or_else(|| {
                    QueryError::Type("date +/- duration produced an out-of-range date".into())
                })
        };
    let local_time_plus_duration = |t: i64, dur: temporal::DurationParts, negate: bool| -> Value {
        let (_, _, seconds, nanos) = dur;
        Value::Property(PropertyValue::LocalTime(temporal::add_duration_to_time(
            t, seconds, nanos, negate,
        )))
    };
    let time_plus_duration =
        |(t, offset): (i64, i32), dur: temporal::DurationParts, negate: bool| -> Value {
            let (_, _, seconds, nanos) = dur;
            Value::Property(PropertyValue::Time {
                nanos_of_day: temporal::add_duration_to_time(t, seconds, nanos, negate),
                offset_seconds: offset,
            })
        };
    let local_date_time_plus_duration = |(epoch_seconds, existing_nanos): (i64, i32),
                                         dur: temporal::DurationParts,
                                         negate: bool|
     -> Result<Value, QueryError> {
        let (months, days, seconds, nanos) = dur;
        temporal::add_duration_to_local_date_time(
            epoch_seconds,
            existing_nanos,
            months,
            days,
            seconds,
            nanos,
            negate,
        )
        .map(|(epoch_seconds, nanos)| {
            Value::Property(PropertyValue::LocalDateTime {
                epoch_seconds,
                nanos,
            })
        })
        .ok_or_else(|| {
            QueryError::Type("local date-time +/- duration produced an out-of-range value".into())
        })
    };
    // `Named` zone arithmetic is only ever a single fixed-offset op, not
    // a full DST-crossing re-resolution -- the offset is resolved once
    // (at the *pre*-arithmetic instant) via `resolve_offset` and carried
    // through unchanged, same as `Offset`'s own behavior; no TCK scenario
    // exercises arithmetic on a `Named`-zone `DateTime` at all, so this
    // is a real, deliberately narrow scope, not silently wrong for a
    // tested case.
    let date_time_plus_duration =
        |(epoch_seconds, existing_nanos, zone): (i64, i32, temporal::TzId),
         dur: temporal::DurationParts,
         negate: bool|
         -> Result<Value, QueryError> {
            let (months, days, seconds, nanos) = dur;
            let offset_seconds = temporal::resolve_offset(&zone, epoch_seconds);
            temporal::add_duration_to_local_date_time(
                epoch_seconds + offset_seconds as i64,
                existing_nanos,
                months,
                days,
                seconds,
                nanos,
                negate,
            )
            .map(|(local_epoch_seconds, nanos)| {
                Value::Property(PropertyValue::DateTime {
                    epoch_seconds: local_epoch_seconds - offset_seconds as i64,
                    nanos,
                    zone: tz_to_graph(zone),
                })
            })
            .ok_or_else(|| {
                QueryError::Type("date-time +/- duration produced an out-of-range value".into())
            })
        };
    Ok(match op {
        ArithOp::Add => {
            if let (Some(d), Some(dur)) = (as_date(a), as_duration(b)) {
                Some(date_plus_duration(d, dur, false)?)
            } else if let (Some(dur), Some(d)) = (as_duration(a), as_date(b)) {
                Some(date_plus_duration(d, dur, false)?)
            } else if let (Some(t), Some(dur)) = (as_local_time(a), as_duration(b)) {
                Some(local_time_plus_duration(t, dur, false))
            } else if let (Some(dur), Some(t)) = (as_duration(a), as_local_time(b)) {
                Some(local_time_plus_duration(t, dur, false))
            } else if let (Some(t), Some(dur)) = (as_time(a), as_duration(b)) {
                Some(time_plus_duration(t, dur, false))
            } else if let (Some(dur), Some(t)) = (as_duration(a), as_time(b)) {
                Some(time_plus_duration(t, dur, false))
            } else if let (Some(dt), Some(dur)) = (as_local_date_time(a), as_duration(b)) {
                Some(local_date_time_plus_duration(dt, dur, false)?)
            } else if let (Some(dur), Some(dt)) = (as_duration(a), as_local_date_time(b)) {
                Some(local_date_time_plus_duration(dt, dur, false)?)
            } else if let (Some(dt), Some(dur)) = (as_date_time(a), as_duration(b)) {
                Some(date_time_plus_duration(dt, dur, false)?)
            } else if let (Some(dur), Some(dt)) = (as_duration(a), as_date_time(b)) {
                Some(date_time_plus_duration(dt, dur, false)?)
            } else if let (Some(x), Some(y)) = (as_duration(a), as_duration(b)) {
                Some(duration_value(temporal::add_duration(x, y).ok_or_else(
                    || QueryError::Type("duration addition overflow".into()),
                )?))
            } else {
                None
            }
        }
        ArithOp::Sub => {
            if let (Some(d), Some(dur)) = (as_date(a), as_duration(b)) {
                Some(date_plus_duration(d, dur, true)?)
            } else if let (Some(t), Some(dur)) = (as_local_time(a), as_duration(b)) {
                Some(local_time_plus_duration(t, dur, true))
            } else if let (Some(t), Some(dur)) = (as_time(a), as_duration(b)) {
                Some(time_plus_duration(t, dur, true))
            } else if let (Some(dt), Some(dur)) = (as_local_date_time(a), as_duration(b)) {
                Some(local_date_time_plus_duration(dt, dur, true)?)
            } else if let (Some(dt), Some(dur)) = (as_date_time(a), as_duration(b)) {
                Some(date_time_plus_duration(dt, dur, true)?)
            } else if let (Some(x), Some(y)) = (as_duration(a), as_duration(b)) {
                Some(duration_value(temporal::sub_duration(x, y).ok_or_else(
                    || QueryError::Type("duration subtraction overflow".into()),
                )?))
            } else {
                None
            }
        }
        ArithOp::Mul => {
            if let (Some(dur), Some(f)) = (as_duration(a), value_as_f64(b)) {
                Some(duration_value(temporal::scale_duration(dur, f)))
            } else if let (Some(f), Some(dur)) = (value_as_f64(a), as_duration(b)) {
                Some(duration_value(temporal::scale_duration(dur, f)))
            } else {
                None
            }
        }
        ArithOp::Div => {
            if let (Some(dur), Some(f)) = (as_duration(a), value_as_f64(b)) {
                if f == 0.0 {
                    return Err(QueryError::Type("division by zero".into()));
                }
                Some(duration_value(temporal::scale_duration(dur, 1.0 / f)))
            } else {
                None
            }
        }
        ArithOp::Mod => None,
        // `^` is never meaningful for a date/duration/etc operand --
        // real Cypher has no temporal exponentiation, so this always
        // falls through to `apply_arith`'s own numeric-only rejection.
        ArithOp::Pow => None,
    })
}

/// `date()` — zero args (today, UTC, from the `Executor`-cached
/// `temporal::NowSnapshot` — see its docs for why every no-arg temporal
/// call within one query shares the same captured instant), a string
/// (`date('2015-07-21')`, the calendar forms `temporal::
/// parse_date` supports), a map (`date({year: 1984, month: 10, day:
/// 11})`, calendar construction only), or another `Date` (identity —
/// `date(d)` where `d` is already a `Date`, e.g. from `toString`
/// round-tripping through `date(toString(d))`). Deliberately does *not*
/// support the week-date/ordinal-date/quarter map or string construction
/// forms real Cypher also has (`date({year: 2015, week: 1})`,
/// `date('2015-W30-2')`, ...) — a real, documented gap (see the README),
/// not a silent wrong answer: both `parse_date` and `date_from_map`
/// return a clear error/`None` for those rather than guessing.
/// `date.transaction()`/`.statement()`/`.realtime()` and their siblings
/// for the other 4 temporal types conceptually take no argument (they
/// always return the current transaction/statement/realtime instant) --
/// but real Cypher still requires them to propagate a `null` argument
/// (TCK's Temporal4 [13] "Should propagate null"), same as every other
/// temporal constructor. Found via the TCK: pest's own grammar couldn't
/// parse these namespaced calls with an argument at all, so this always-
/// ignore-args behavior was untested until ANTLR's grammar (which does
/// support it) newly exposed it as a silent wrong answer instead of null.
pub(crate) fn now_or_null(args: &[Value], now_value: impl FnOnce() -> Value) -> Value {
    if matches!(args.first(), Some(Value::Null)) {
        Value::Null
    } else {
        now_value()
    }
}

pub(crate) fn date_builtin(
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Semantic(format!(
            "date() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::Date(now.epoch_day)));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::Date(d)) = arg {
        return Ok(Value::Property(PropertyValue::Date(*d)));
    }
    // `date(otherTemporal)` -- a bare `LocalDateTime`/`DateTime` argument
    // projects its own date part, same as `date({date: otherTemporal})`
    // (TCK's Temporal3 [1]).
    if matches!(
        arg,
        Value::Property(PropertyValue::LocalDateTime { .. } | PropertyValue::DateTime { .. })
    ) {
        let epoch_day = extract_date_base_epoch_day("date() argument", arg)?;
        return Ok(Value::Property(PropertyValue::Date(epoch_day)));
    }
    if let Some(s) = as_arith_str(arg) {
        let d = temporal::parse_date(s).ok_or_else(|| {
            QueryError::Type(format!(
                "'{s}' isn't a date string MarsDB can parse -- only the calendar forms YYYY-MM-DD/YYYYMMDD/\
                 YYYY-MM/YYYYMM/YYYY, week-date forms YYYY-Www[-D]/YYYYWww[D], and ordinal-date forms \
                 YYYY-DDD/YYYYDDD are supported"
            ))
        })?;
        return Ok(Value::Property(PropertyValue::Date(d)));
    }
    if let Value::Map(m) = arg {
        return Ok(Value::Property(PropertyValue::Date(date_from_map(m)?)));
    }
    Err(QueryError::Type(format!(
        "date() doesn't support this argument: {arg:?}"
    )))
}

/// Pulls the local (offset-adjusted for `DateTime`) epoch-day out of a
/// `Date`/`LocalDateTime`/`DateTime` value -- the "base" a `date`/
/// `datetime` map key projects its calendar fields from
/// (`date({date: other, day: 5})`, `localdatetime({date: other, hour:
/// 10, ...})`, ...). Returns the raw epoch-day (not a pre-split
/// `(year, month, day)`) so a caller can read *any* calendar component
/// off it (`weekYear`/`week`/`dayOfWeek`/`quarter`/`dayOfQuarter`/
/// `ordinalDay` via `date_component`), for defaulting the alternate
/// week/ordinal/quarter-date map-construction forms (see
/// `calendar_fields_from_map`).
pub(crate) fn extract_date_base_epoch_day(key: &str, v: &Value) -> Result<i64, QueryError> {
    match v {
        Value::Property(PropertyValue::Date(d)) => Ok(*d),
        Value::Property(PropertyValue::LocalDateTime { epoch_seconds, .. }) => {
            Ok(temporal::split_epoch_seconds(*epoch_seconds).0)
        }
        Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            zone,
            ..
        }) => {
            let offset_seconds = temporal::resolve_offset(&tz_from_graph(zone), *epoch_seconds);
            Ok(temporal::split_epoch_seconds(epoch_seconds + offset_seconds as i64).0)
        }
        other => Err(QueryError::Type(format!(
            "'{key}' must be a Date, LocalDateTime, or DateTime, got {other:?}"
        ))),
    }
}

/// `(hour, minute, second, nanos, zone)` pulled out of a `LocalTime`/
/// `Time`/`LocalDateTime`/`DateTime` value -- the "base" a `time`/
/// `datetime` map key projects its clock fields from. `nanos` here is
/// just the nanosecond-of-second remainder (not the whole nanos-of-day),
/// matching the map constructors' own `nanosecond` field. `zone` is
/// `Some((original_zone, resolved_offset_seconds))` only for `Time`/
/// `DateTime` sources -- both are kept, not just the resolved number, so
/// a caller that projects this base *without* an explicit `timezone`
/// override (`{time: t}`, `{datetime: dt}`) can preserve the source's
/// own zone *identity* (a `Named` zone stays `Named`, TCK's Temporal3
/// [9]/[11] `{datetime: other}` rows), while a caller that only ever
/// needs a plain number (`time_builtin`'s cross-type conversion, `TIME`
/// structurally can't hold a name) uses the resolved half directly.
pub(crate) type ClockBase = (i64, i64, i64, i64, Option<(temporal::TzId, i32)>);

pub(crate) fn extract_time_base(key: &str, v: &Value) -> Result<ClockBase, QueryError> {
    let hms_nanos = |nanos_of_day: i64| {
        (
            temporal::local_time_component(nanos_of_day, "hour").unwrap(),
            temporal::local_time_component(nanos_of_day, "minute").unwrap(),
            temporal::local_time_component(nanos_of_day, "second").unwrap(),
            temporal::local_time_component(nanos_of_day, "nanosecond").unwrap(),
        )
    };
    match v {
        Value::Property(PropertyValue::LocalTime(n)) => {
            let (h, m, s, ns) = hms_nanos(*n);
            Ok((h, m, s, ns, None))
        }
        Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => {
            let (h, m, s, ns) = hms_nanos(*nanos_of_day);
            Ok((
                h,
                m,
                s,
                ns,
                Some((temporal::TzId::Offset(*offset_seconds), *offset_seconds)),
            ))
        }
        Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => {
            let (_, nanos_of_day) = temporal::split_epoch_seconds(*epoch_seconds);
            let (h, m, s, _) = hms_nanos(nanos_of_day);
            Ok((h, m, s, *nanos as i64, None))
        }
        Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => {
            let tz = tz_from_graph(zone);
            let offset_seconds = temporal::resolve_offset(&tz, *epoch_seconds);
            let local = epoch_seconds + offset_seconds as i64;
            let (_, nanos_of_day) = temporal::split_epoch_seconds(local);
            let (h, m, s, _) = hms_nanos(nanos_of_day);
            Ok((h, m, s, *nanos as i64, Some((tz, offset_seconds))))
        }
        other => Err(QueryError::Type(format!(
            "'{key}' must be a LocalTime, Time, LocalDateTime, or DateTime, got {other:?}"
        ))),
    }
}

pub(crate) const DATE_ALLOWED_KEYS: &[&str] = &[
    "year",
    "month",
    "day",
    "week",
    "dayOfWeek",
    "ordinalDay",
    "quarter",
    "dayOfQuarter",
    "date",
];

pub(crate) fn date_from_map(m: &BTreeMap<String, Value>) -> Result<i64, QueryError> {
    let (year, month, day) = calendar_fields_from_map("date", m, DATE_ALLOWED_KEYS)?;
    temporal::epoch_day_from_ymd(year, month, day).ok_or_else(|| {
        QueryError::Type(format!(
            "{year:04}-{month:02}-{day:02} isn't a valid calendar date"
        ))
    })
}

/// Computes `(year, month, day)` from a map that specifies one of four
/// mutually exclusive ways to pin a calendar day -- the plain calendar
/// form (`year`/`month`/`day`, each optionally defaulted from a `date`/
/// `datetime` base's own value), ISO week-date (`week`/`dayOfWeek`,
/// defaulted from the base's `weekYear`/`week`/`dayOfWeek`), ordinal-date
/// (`ordinalDay`, year defaulted from the base's `year`), or quarter-date
/// (`quarter`/`dayOfQuarter`, defaulted from the base's `quarter`/
/// `dayOfQuarter`) -- real Cypher's four alternate ways to construct a
/// date, all reducible to the same `(year, month, day)` triple
/// `epoch_day_from_ymd` needs. Shared by `date()`'s own map form and
/// `localdatetime()`/`datetime()`'s map forms (`allowed` differs only in
/// whether clock/timezone keys are also permitted in the same map -- this
/// function only ever looks at the date-shaped keys).
pub(crate) fn calendar_fields_from_map(
    caller: &str,
    m: &BTreeMap<String, Value>,
    allowed: &[&str],
) -> Result<(i64, u32, u32), QueryError> {
    if let Some(bad) = m.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(QueryError::Type(format!(
            "{caller}({{...}}) key '{bad}' isn't a recognized field"
        )));
    }
    let int_field = |key: &str, value: &Value| {
        value_as_i64(value).ok_or_else(|| {
            QueryError::Type(format!("{caller}({{...}})'s '{key}' must be an integer"))
        })
    };
    let base_epoch_day = m
        .get("date")
        .map(|v| ("date", v))
        .or_else(|| m.get("datetime").map(|v| ("datetime", v)))
        .map(|(k, v)| extract_date_base_epoch_day(k, v))
        .transpose()?;
    let epoch_day_from_component =
        |prop: &str| base_epoch_day.map(|ed| temporal::date_component(ed, prop).unwrap());

    if m.contains_key("week") || m.contains_key("dayOfWeek") {
        let week_year = match m.get("year") {
            Some(v) => int_field("year", v)?,
            None => epoch_day_from_component("weekYear").ok_or_else(|| {
                QueryError::Type(format!("{caller}({{...}}) requires a 'year' key"))
            })?,
        };
        let week = match m.get("week") {
            Some(v) => u32::try_from(int_field("week", v)?).map_err(|_| {
                QueryError::Type(format!("{caller}({{...}})'s 'week' is out of range"))
            })?,
            None => u32::try_from(epoch_day_from_component("week").ok_or_else(|| {
                QueryError::Type(format!("{caller}({{...}}) requires a 'week' key"))
            })?)
            .unwrap(),
        };
        let day_of_week = match m.get("dayOfWeek") {
            Some(v) => int_field("dayOfWeek", v)?,
            None => epoch_day_from_component("dayOfWeek").unwrap_or(1),
        };
        let epoch_day = temporal::epoch_day_from_week_fields(week_year, week, day_of_week)
            .ok_or_else(|| {
                QueryError::Type(format!(
                    "{caller}({{...}}) has an out-of-range week-date field"
                ))
            })?;
        return Ok((
            temporal::date_component(epoch_day, "year").unwrap(),
            temporal::date_component(epoch_day, "month").unwrap() as u32,
            temporal::date_component(epoch_day, "day").unwrap() as u32,
        ));
    }

    if m.contains_key("ordinalDay") {
        let year = match m.get("year") {
            Some(v) => int_field("year", v)?,
            None => epoch_day_from_component("year").ok_or_else(|| {
                QueryError::Type(format!("{caller}({{...}}) requires a 'year' key"))
            })?,
        };
        let ordinal_raw = int_field("ordinalDay", m.get("ordinalDay").unwrap())?;
        let ordinal_day = u32::try_from(ordinal_raw).map_err(|_| {
            QueryError::Type(format!("{caller}({{...}})'s 'ordinalDay' is out of range"))
        })?;
        let epoch_day =
            temporal::epoch_day_from_ordinal_fields(year, ordinal_day).ok_or_else(|| {
                QueryError::Type(format!(
                    "{caller}({{...}}) has an out-of-range ordinalDay field"
                ))
            })?;
        return Ok((
            year,
            temporal::date_component(epoch_day, "month").unwrap() as u32,
            temporal::date_component(epoch_day, "day").unwrap() as u32,
        ));
    }

    if m.contains_key("quarter") || m.contains_key("dayOfQuarter") {
        let year = match m.get("year") {
            Some(v) => int_field("year", v)?,
            None => epoch_day_from_component("year").ok_or_else(|| {
                QueryError::Type(format!("{caller}({{...}}) requires a 'year' key"))
            })?,
        };
        let quarter = match m.get("quarter") {
            Some(v) => u32::try_from(int_field("quarter", v)?).map_err(|_| {
                QueryError::Type(format!("{caller}({{...}})'s 'quarter' is out of range"))
            })?,
            None => u32::try_from(epoch_day_from_component("quarter").ok_or_else(|| {
                QueryError::Type(format!("{caller}({{...}}) requires a 'quarter' key"))
            })?)
            .unwrap(),
        };
        let day_of_quarter = match m.get("dayOfQuarter") {
            Some(v) => int_field("dayOfQuarter", v)?,
            None => epoch_day_from_component("dayOfQuarter").unwrap_or(1),
        };
        let epoch_day = temporal::epoch_day_from_quarter_fields(year, quarter, day_of_quarter)
            .ok_or_else(|| {
                QueryError::Type(format!(
                    "{caller}({{...}}) has an out-of-range quarter-date field"
                ))
            })?;
        return Ok((
            year,
            temporal::date_component(epoch_day, "month").unwrap() as u32,
            temporal::date_component(epoch_day, "day").unwrap() as u32,
        ));
    }

    let year_raw = match m.get("year") {
        Some(v) => int_field("year", v)?,
        None => epoch_day_from_component("year")
            .ok_or_else(|| QueryError::Type(format!("{caller}({{...}}) requires a 'year' key")))?,
    };
    let year = year_raw;
    let month_raw = match m.get("month") {
        Some(v) => int_field("month", v)?,
        None => epoch_day_from_component("month").unwrap_or(1),
    };
    let month = u32::try_from(month_raw).map_err(|_| {
        QueryError::Type(format!(
            "{caller}({{...}})'s 'month' is out of range: {month_raw}"
        ))
    })?;
    let day_raw = match m.get("day") {
        Some(v) => int_field("day", v)?,
        None => epoch_day_from_component("day").unwrap_or(1),
    };
    let day = u32::try_from(day_raw).map_err(|_| {
        QueryError::Type(format!(
            "{caller}({{...}})'s 'day' is out of range: {day_raw}"
        ))
    })?;
    Ok((year, month, day))
}

/// `duration(...)` — a string (ISO-8601 `'P...'` text, `temporal::
/// parse_duration`) or a map (`duration({days: 14, hours: 16})`,
/// `temporal::normalize_duration`). No zero-arg form (real Cypher has
/// none either — a duration has no "current" value the way a date/time
/// does).
pub(crate) fn duration_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if args.len() != 1 {
        return Err(QueryError::Semantic(format!(
            "duration() expects exactly one argument, got {}",
            args.len()
        )));
    }
    let arg = &args[0];
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    let (months, days, seconds, nanos) = if let Some(s) = as_arith_str(arg) {
        temporal::parse_duration(s).ok_or_else(|| {
            QueryError::Type(format!(
                "'{s}' isn't a duration string MarsDB can parse -- only ISO-8601 'PnYnMnWnDTnHnMnS' text is \
                 supported, not the alternate combined date-time duration syntax"
            ))
        })?
    } else if let Value::Map(m) = arg {
        temporal::normalize_duration(duration_fields_from_map(m)?)
    } else {
        return Err(QueryError::Type(format!(
            "duration() doesn't support this argument: {arg:?}"
        )));
    };
    Ok(Value::Property(PropertyValue::Duration {
        months,
        days,
        seconds,
        nanos,
    }))
}

pub(crate) fn duration_fields_from_map(
    m: &BTreeMap<String, Value>,
) -> Result<temporal::DurationFields, QueryError> {
    const ALLOWED: &[&str] = &[
        "years",
        "months",
        "weeks",
        "days",
        "hours",
        "minutes",
        "seconds",
        "milliseconds",
        "microseconds",
        "nanoseconds",
    ];
    if let Some(bad) = m.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
        return Err(QueryError::Type(format!(
            "duration({{...}}) key '{bad}' isn't a recognized duration unit"
        )));
    }
    let field = |key: &str| -> Result<f64, QueryError> {
        match m.get(key) {
            None => Ok(0.0),
            Some(v) => value_as_f64(v).ok_or_else(|| {
                QueryError::Type(format!("duration({{...}})'s '{key}' must be a number"))
            }),
        }
    };
    Ok(temporal::DurationFields {
        years: field("years")?,
        months: field("months")?,
        weeks: field("weeks")?,
        days: field("days")?,
        hours: field("hours")?,
        minutes: field("minutes")?,
        seconds: field("seconds")?,
        milliseconds: field("milliseconds")?,
        microseconds: field("microseconds")?,
        nanoseconds: field("nanoseconds")?,
    })
}

/// Sums the 3 sub-second map keys (`millisecond`/`microsecond`/
/// `nanosecond`) shared by every one-of-day-or-later temporal map
/// constructor into one nanosecond count -- each key independently
/// *additive* (matching real Cypher's own construction semantics,
/// e.g. `{millisecond: 645, nanosecond: 123}` is `645ms + 123ns`, not
/// "645ms, ignore the usual nanosecond digit position"), separate from
/// `duration`'s own `nanoseconds` field of the same name.
///
/// `base_fraction_ns` (`0..1_000_000_000`) is the fractional-second
/// part of whatever this map is *overriding* (a `time`/`datetime`
/// projection key, or a `.truncate()` call's already-truncated value)
/// -- `0` for plain from-scratch construction, where there's no base to
/// inherit from. Any of the 3 keys the map doesn't set defaults to that
/// *digit group* of the base (millisecond/microsecond/nanosecond each
/// their own `0..999` slice), not to `0` outright -- found as a real
/// bug: `{nanosecond: 2}` alone on a base with a real millisecond value
/// was silently dropping that millisecond instead of keeping it, only
/// the nanosecond digit was meant to change.
pub(crate) fn sub_second_nanos_from_map(
    base_fraction_ns: i64,
    m: &BTreeMap<String, Value>,
) -> Result<i64, QueryError> {
    let base_ms = base_fraction_ns / 1_000_000;
    let base_us = (base_fraction_ns / 1_000) % 1000;
    let base_ns = base_fraction_ns % 1000;
    let ms = int_field(m, "millisecond", base_ms)?;
    let us = int_field(m, "microsecond", base_us)?;
    let ns = int_field(m, "nanosecond", base_ns)?;
    Ok(ms * 1_000_000 + us * 1_000 + ns)
}

pub(crate) fn int_field(
    m: &BTreeMap<String, Value>,
    key: &str,
    default: i64,
) -> Result<i64, QueryError> {
    match m.get(key) {
        None => Ok(default),
        Some(v) => {
            value_as_i64(v).ok_or_else(|| QueryError::Type(format!("'{key}' must be an integer")))
        }
    }
}

/// Computes `(hour, minute, second, nanos, offset_seconds)` for a
/// `localtime`/`time`/`localdatetime`/`datetime` map constructor -- a
/// `time`/`datetime` key (if present) projects its clock fields as the
/// default, explicit `hour`/`minute`/`second`/`millisecond`/
/// `microsecond`/`nanosecond` keys override individual fields on top of
/// that (`{time: other, second: 42}` keeps everything from `other`
/// except `second`). No base key falls back to all-zero defaults,
/// matching the plain (non-projecting) map form.
///
/// If the base carries an offset (`Time`/`DateTime`) and an explicit
/// `timezone` key names a *different* one, the wall-clock is shifted
/// first to preserve the same instant (`{time: other, timezone:
/// '+05:00'}` on a `+01:00` base advances the hour by 4) -- real
/// Cypher's rule, confirmed against Temporal3's own examples -- and
/// only *then* do explicit hour/minute/second overrides apply, on top
/// of the shifted result, not the original.
/// `epoch_day` is the calendar date the resulting clock fields will be
/// combined with -- only needed to resolve a *shift into a named zone*
/// (its real, DST-aware offset depends on the date, TCK's Temporal3 [9]
/// row: `{time: t+01:00, second: 42, timezone: 'Pacific/Honolulu'}`),
/// `None` for callers with no date at all (`time()`'s own map form,
/// which can't shift into a named zone regardless -- its caller rejects
/// that case itself) or that don't care about the resolved zone
/// (`localdatetime()`'s map form, which discards it).
/// The 5th element is `Some((effective_zone, effective_offset))` --
/// `effective_zone` preserves a `Named` base's identity when no
/// explicit `timezone` override is given (needed by `DATETIME`, which
/// can hold one); `effective_offset` is always a plain resolved number,
/// usable directly by a caller that structurally can't hold a zone name
/// (`TIME`) regardless of which case produced it.
pub(crate) fn clock_fields_from_map(
    m: &BTreeMap<String, Value>,
    epoch_day: Option<i64>,
) -> Result<ClockBase, QueryError> {
    let (base_h, base_m, base_s, base_ns, base_zone) = if let Some(v) = m.get("time") {
        extract_time_base("time", v)?
    } else if let Some(v) = m.get("datetime") {
        extract_time_base("datetime", v)?
    } else {
        (0, 0, 0, 0, None)
    };
    let has_explicit_timezone = m.contains_key("timezone");
    let effective_zone = match m.get("timezone") {
        Some(v) => Some(timezone_value_to_tzid(v)?),
        // No explicit override -- preserve the base's own zone
        // *identity* (a `Named` zone stays `Named`), not just its
        // resolved offset (TCK's Temporal3 [9]/[11] `{datetime: other}`
        // rows, where `other` is itself a named-zone value).
        None => base_zone.as_ref().map(|(tz, _)| tz.clone()),
    };
    // The wall-clock is only ever *shifted* by an *explicit* `timezone`
    // override that actually changes the zone -- with no override, the
    // literal local time passes straight through unchanged even if the
    // base's own zone's real offset differs for the (possibly
    // day-overridden) new date, e.g. a DST boundary crossed by a `day`
    // override (TCK's Temporal3 [10]: a `Named` base carried through
    // with no `timezone` key keeps its `12:00` wall-clock as `12:00`,
    // just re-displayed with whatever offset that zone now resolves to
    // -- it does *not* shift to a different wall-clock hour).
    let base_nanos_of_day =
        base_h * 3_600_000_000_000 + base_m * 60_000_000_000 + base_s * 1_000_000_000 + base_ns;
    let (base_h, base_m, base_s, base_ns, effective_offset) = if has_explicit_timezone {
        // The base's own offset, re-resolved against the *new* date --
        // not its own original instant's offset (`extract_time_base`'s
        // `Named` resolution, which used the *source* value's own
        // epoch_seconds/date, not necessarily this one -- a `day`
        // override can move the result to a different date than the
        // base's, potentially across a DST boundary for the *same*
        // zone, TCK's Temporal3 [10] row 337).
        let from_offset = match base_zone.as_ref() {
            Some((temporal::TzId::Offset(o), _)) => Some(*o),
            Some((zone @ temporal::TzId::Named(_), resolved)) => Some(match epoch_day {
                Some(ed) => temporal::resolve_offset(
                    zone,
                    temporal::combine_epoch_day_and_nanos_of_day(ed, base_nanos_of_day),
                ),
                None => *resolved,
            }),
            None => None,
        };
        let to_offset = match (from_offset, effective_zone.as_ref(), epoch_day) {
            (Some(_), Some(temporal::TzId::Offset(to)), _) => Some(*to),
            (Some(from), Some(zone @ temporal::TzId::Named(_)), Some(ed)) => {
                let approx_epoch_seconds =
                    temporal::combine_epoch_day_and_nanos_of_day(ed, base_nanos_of_day)
                        - from as i64;
                Some(temporal::resolve_offset(zone, approx_epoch_seconds))
            }
            _ => None,
        };
        match (from_offset, to_offset) {
            (Some(from), Some(to)) if from != to => {
                let shifted = (base_nanos_of_day + (to - from) as i64 * 1_000_000_000)
                    .rem_euclid(86_400_000_000_000);
                (
                    shifted / 3_600_000_000_000,
                    (shifted / 60_000_000_000) % 60,
                    (shifted / 1_000_000_000) % 60,
                    shifted % 1_000_000_000,
                    to_offset.unwrap_or(0),
                )
            }
            _ => (base_h, base_m, base_s, base_ns, to_offset.unwrap_or(0)),
        }
    } else {
        // No override -- the resolved offset is just the base's own
        // (unchanged, no re-resolution -- a caller that can't hold a
        // zone name, `TIME`, degrades a `Named` base to this number
        // silently, TCK's Temporal3 [3] row 125: `{time: t}` where `t`
        // is a named-zone `DateTime` -> the plain offset, no error).
        (
            base_h,
            base_m,
            base_s,
            base_ns,
            base_zone.as_ref().map_or(0, |(_, o)| *o),
        )
    };
    Ok((
        int_field(m, "hour", base_h)?,
        int_field(m, "minute", base_m)?,
        int_field(m, "second", base_s)?,
        sub_second_nanos_from_map(base_ns, m)?,
        effective_zone.map(|z| (z, effective_offset)),
    ))
}

/// `localtime(...)` -- zero args (now, UTC), a string (`temporal::
/// parse_local_time`), a map (`localtime({hour: 21, minute: 40, ...})`,
/// optionally projected from another temporal value via a `time` key),
/// or another `LocalTime` (identity, e.g. round-tripping through
/// `toString`).
pub(crate) fn local_time_builtin(
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Semantic(format!(
            "localtime() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::LocalTime(now.nanos_of_day)));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::LocalTime(t)) = arg {
        return Ok(Value::Property(PropertyValue::LocalTime(*t)));
    }
    // `localtime(otherTemporal)` -- a bare `Time`/`LocalDateTime`/
    // `DateTime` argument projects its own time-of-day part (offset
    // dropped, same as `{time: otherTemporal}`), TCK's Temporal3 [2].
    if matches!(
        arg,
        Value::Property(
            PropertyValue::Time { .. }
                | PropertyValue::LocalDateTime { .. }
                | PropertyValue::DateTime { .. }
        )
    ) {
        let (hour, minute, second, nanos, _) = extract_time_base("localtime() argument", arg)?;
        let t = temporal::local_time_nanos_from_fields(hour, minute, second, nanos).ok_or_else(
            || QueryError::Type("localtime() argument has an out-of-range field".into()),
        )?;
        return Ok(Value::Property(PropertyValue::LocalTime(t)));
    }
    if let Some(s) = as_arith_str(arg) {
        let t = temporal::parse_local_time(s).ok_or_else(|| {
            QueryError::Type(format!("'{s}' isn't a local time string MarsDB can parse"))
        })?;
        return Ok(Value::Property(PropertyValue::LocalTime(t)));
    }
    if let Value::Map(m) = arg {
        const ALLOWED: &[&str] = &[
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
            "time",
        ];
        if let Some(bad) = m.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
            return Err(QueryError::Type(format!(
                "localtime({{...}}) key '{bad}' isn't a recognized field"
            )));
        }
        let (hour, minute, second, nanos, _) = clock_fields_from_map(m, None)?;
        let t = temporal::local_time_nanos_from_fields(hour, minute, second, nanos)
            .ok_or_else(|| QueryError::Type("localtime({...}) has an out-of-range field".into()))?;
        return Ok(Value::Property(PropertyValue::LocalTime(t)));
    }
    Err(QueryError::Type(format!(
        "localtime() doesn't support this argument: {arg:?}"
    )))
}

/// `time(...)` -- same shapes as `localtime(...)`, but every form
/// (except identity) requires a `timezone` map key / string offset
/// suffix. A bracketed named-zone suffix (`[Europe/Stockholm]`) gets a
/// specific "not supported" error rather than the generic parse-failure
/// message, since that's a real (if out of scope) Cypher form, not
/// malformed input.
pub(crate) fn time_builtin(
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Semantic(format!(
            "time() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::Time {
            nanos_of_day: now.nanos_of_day,
            offset_seconds: 0,
        }));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::Time {
        nanos_of_day,
        offset_seconds,
    }) = arg
    {
        return Ok(Value::Property(PropertyValue::Time {
            nanos_of_day: *nanos_of_day,
            offset_seconds: *offset_seconds,
        }));
    }
    // `time(otherTemporal)` -- a bare `LocalTime`/`LocalDateTime`/
    // `DateTime` argument projects its own time part, defaulting the
    // offset to UTC when the source has none (`LocalTime`/
    // `LocalDateTime`), same as `{time: otherTemporal}` (TCK's
    // Temporal3 [3]).
    if matches!(
        arg,
        Value::Property(
            PropertyValue::LocalTime(_)
                | PropertyValue::LocalDateTime { .. }
                | PropertyValue::DateTime { .. }
        )
    ) {
        let (hour, minute, second, nanos, zone) = extract_time_base("time() argument", arg)?;
        let nanos_of_day = temporal::local_time_nanos_from_fields(hour, minute, second, nanos)
            .ok_or_else(|| QueryError::Type("time() argument has an out-of-range field".into()))?;
        return Ok(Value::Property(PropertyValue::Time {
            nanos_of_day,
            // `TIME` structurally can't carry a zone name -- degrades a
            // `Named` source to its resolved numeric offset (TCK's
            // Temporal3 [3] `datetime({..., timezone: 'Europe/
            // Stockholm'})` -> `time(other)` = `'12:00+01:00'`, the
            // offset alone, no bracket).
            offset_seconds: zone.map_or(0, |(_, o)| o),
        }));
    }
    if let Some(s) = as_arith_str(arg) {
        if s.contains('[') {
            return Err(QueryError::Type(
                "time('...'): named timezones (e.g. '[Europe/Stockholm]') aren't supported, only a fixed UTC \
                 offset like '+01:00'"
                    .into(),
            ));
        }
        let (nanos_of_day, offset_seconds) = temporal::parse_time(s).ok_or_else(|| {
            QueryError::Type(format!("'{s}' isn't a time string MarsDB can parse"))
        })?;
        return Ok(Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }));
    }
    if let Value::Map(m) = arg {
        const ALLOWED: &[&str] = &[
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
            "timezone",
            "time",
        ];
        if let Some(bad) = m.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
            return Err(QueryError::Type(format!(
                "time({{...}}) key '{bad}' isn't a recognized field"
            )));
        }
        let (hour, minute, second, nanos, zone) = clock_fields_from_map(m, None)?;
        let offset_seconds = match zone {
            None => 0,
            // A `Named` zone reaching here with no *explicit* `timezone`
            // key was just carried through from a projected `time`/
            // `datetime` base (`{time: namedZoneDateTime}`) -- `TIME`
            // can't hold a name, so it silently degrades to the base's
            // own resolved offset, same as the cross-type positional
            // form already does (TCK's Temporal3 [3] row 125). An
            // *explicit* named-zone request, though, is a real error --
            // there's no calendar date here to resolve it against.
            Some((_, o)) if !m.contains_key("timezone") => o,
            Some((temporal::TzId::Offset(o), _)) => o,
            Some((temporal::TzId::Named(name), _)) => {
                return Err(QueryError::Type(format!(
                    "'timezone': '{name}' looks like a named timezone (e.g. 'Europe/Stockholm') -- TIME has \
                     no calendar date to resolve a named zone's DST-dependent offset against, only a fixed \
                     UTC offset like '+01:00' is supported"
                )));
            }
        };
        let nanos_of_day = temporal::local_time_nanos_from_fields(hour, minute, second, nanos)
            .ok_or_else(|| QueryError::Type("time({...}) has an out-of-range field".into()))?;
        return Ok(Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }));
    }
    Err(QueryError::Type(format!(
        "time() doesn't support this argument: {arg:?}"
    )))
}

/// `{timezone: '+01:00'}`'s value -- a fixed UTC offset, or an IANA zone
/// name (`'Europe/Stockholm'`). Both forms are always syntactically
/// disjoint (an offset always starts with `+`/`-`/`Z`, a zone name never
/// does), so there's no ambiguity to resolve between them. A caller that
/// can't accept a `Named` zone (`time_builtin`'s map form -- `TIME` has
/// no calendar date to resolve a named zone's DST-dependent offset
/// against) rejects it itself, after this succeeds.
pub(crate) fn timezone_value_to_tzid(v: &Value) -> Result<temporal::TzId, QueryError> {
    let s = as_arith_str(v).ok_or_else(|| {
        QueryError::Type(
            "'timezone' must be a string offset or IANA zone name, e.g. '+01:00' or \
             'Europe/Stockholm'"
                .into(),
        )
    })?;
    if let Some(offset) = temporal::parse_offset_seconds(s) {
        return Ok(temporal::TzId::Offset(offset));
    }
    if temporal::parse_timezone_name(s).is_some() {
        return Ok(temporal::TzId::Named(s.to_string()));
    }
    Err(QueryError::Type(format!(
        "'timezone': '{s}' isn't a valid UTC offset or a recognized IANA zone name"
    )))
}

/// `localdatetime(...)` -- zero args (now, UTC), a string, a map
/// (`localdatetime({year, month, day, hour, minute, second, ...})`), or
/// another `LocalDateTime` (identity).
pub(crate) fn local_date_time_builtin(
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Semantic(format!(
            "localdatetime() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds: now.epoch_seconds,
            nanos: now.nanos,
        }));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::LocalDateTime {
        epoch_seconds,
        nanos,
    }) = arg
    {
        return Ok(Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds: *epoch_seconds,
            nanos: *nanos,
        }));
    }
    // `localdatetime(otherTemporal)` -- a bare `DateTime` argument drops
    // its offset and keeps its local date+time, same as
    // `{datetime: otherTemporal}` (TCK's Temporal3 [7]).
    if matches!(arg, Value::Property(PropertyValue::DateTime { .. })) {
        let epoch_day = extract_date_base_epoch_day("localdatetime() argument", arg)?;
        let year = temporal::date_component(epoch_day, "year").unwrap();
        let month = temporal::date_component(epoch_day, "month").unwrap() as u32;
        let day = temporal::date_component(epoch_day, "day").unwrap() as u32;
        let (hour, minute, second, nanos, _) = extract_time_base("localdatetime() argument", arg)?;
        let (epoch_seconds, nanos) =
            temporal::local_date_time_from_fields(temporal::CalendarDateTime {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanos,
            })
            .ok_or_else(|| {
                QueryError::Type("localdatetime() argument has an out-of-range field".into())
            })?;
        return Ok(Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }));
    }
    if let Some(s) = as_arith_str(arg) {
        let (epoch_seconds, nanos) = temporal::parse_local_date_time(s).ok_or_else(|| {
            QueryError::Type(format!(
                "'{s}' isn't a local date-time string MarsDB can parse"
            ))
        })?;
        return Ok(Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }));
    }
    if let Value::Map(m) = arg {
        let (year, month, day) =
            calendar_fields_from_map("localdatetime", m, DATE_TIME_ALLOWED_KEYS)?;
        let (hour, minute, second, nanos, _) = clock_fields_from_map(m, None)?;
        let (epoch_seconds, nanos) =
            temporal::local_date_time_from_fields(temporal::CalendarDateTime {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanos,
            })
            .ok_or_else(|| {
                QueryError::Type("localdatetime({...}) has an out-of-range field".into())
            })?;
        return Ok(Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }));
    }
    Err(QueryError::Type(format!(
        "localdatetime() doesn't support this argument: {arg:?}"
    )))
}

pub(crate) const DATE_TIME_ALLOWED_KEYS: &[&str] = &[
    "year",
    "month",
    "day",
    "week",
    "dayOfWeek",
    "ordinalDay",
    "quarter",
    "dayOfQuarter",
    "hour",
    "minute",
    "second",
    "millisecond",
    "microsecond",
    "nanosecond",
    "timezone",
    "date",
    "time",
    "datetime",
];

/// `datetime(...)` -- zero args (now, UTC), a string, a map
/// (`datetime({year, ..., timezone: '+01:00'})` or `{..., timezone:
/// 'Europe/Stockholm'}`), or another `DateTime` (identity). Requires a
/// `timezone` for every constructed form except identity (defaults to
/// UTC, `TzId::Offset(0)`, if the map omits it -- matches `date()`'s own
/// "no timezone info -> UTC" convention).
pub(crate) fn date_time_builtin(
    args: &[Value],
    now: temporal::NowSnapshot,
) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Semantic(format!(
            "datetime() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::DateTime {
            epoch_seconds: now.epoch_seconds,
            nanos: now.nanos,
            zone: GraphTzId::Offset(0),
        }));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::DateTime {
        epoch_seconds,
        nanos,
        zone,
    }) = arg
    {
        return Ok(Value::Property(PropertyValue::DateTime {
            epoch_seconds: *epoch_seconds,
            nanos: *nanos,
            zone: zone.clone(),
        }));
    }
    // `datetime(otherLocalDateTime)` -- a bare `LocalDateTime` argument
    // has no zone of its own, defaults to UTC, same as `{datetime:
    // otherLocalDateTime}` (TCK's Temporal3 [11]).
    if let Value::Property(PropertyValue::LocalDateTime {
        epoch_seconds,
        nanos,
    }) = arg
    {
        return Ok(Value::Property(PropertyValue::DateTime {
            epoch_seconds: *epoch_seconds,
            nanos: *nanos,
            zone: GraphTzId::Offset(0),
        }));
    }
    if let Some(s) = as_arith_str(arg) {
        let (epoch_seconds, nanos, zone) = temporal::parse_date_time(s).ok_or_else(|| {
            QueryError::Type(format!("'{s}' isn't a date-time string MarsDB can parse"))
        })?;
        return Ok(Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone: tz_to_graph(zone),
        }));
    }
    if let Value::Map(m) = arg {
        let (year, month, day) = calendar_fields_from_map("datetime", m, DATE_TIME_ALLOWED_KEYS)?;
        let epoch_day = temporal::epoch_day_from_ymd(year, month, day);
        let (hour, minute, second, nanos, zone) = clock_fields_from_map(m, epoch_day)?;
        let zone = zone.map_or(temporal::TzId::Offset(0), |(z, _)| z);
        let (epoch_seconds, nanos) = temporal::date_time_from_fields(
            temporal::CalendarDateTime {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanos,
            },
            &zone,
        )
        .ok_or_else(|| QueryError::Type("datetime({...}) has an out-of-range field".into()))?;
        return Ok(Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone: tz_to_graph(zone),
        }));
    }
    Err(QueryError::Type(format!(
        "datetime() doesn't support this argument: {arg:?}"
    )))
}

/// Reduces any of the 5 non-`Duration` temporal types to `(epoch_day,
/// nanos_of_day, offset_seconds)`, each independently `None` when that
/// value has no such component -- e.g. `LocalTime` is `(None, Some(_),
/// None)`, bare `Date` is `(Some(_), None, None)`. `DateTime`'s
/// date/time components use its *local* (offset-adjusted) reading,
/// matching every other `DateTime` component access (see
/// `date_time_component`'s docs); its real offset is *also* returned
/// (not disregarded) since `duration.between`'s own instant-aware
/// reconciliation needs it when both operands carry one -- see
/// `temporal::between_components`'s docs for exactly when it applies.
pub(crate) fn between_operand(name: &str, v: &Value) -> Result<BetweenOperand, QueryError> {
    match v {
        Value::Property(PropertyValue::Date(d)) => Ok((Some(*d), None, None)),
        Value::Property(PropertyValue::LocalTime(n)) => Ok((None, Some(*n), None)),
        Value::Property(PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        }) => Ok((
            None,
            Some(*nanos_of_day),
            Some(temporal::TzId::Offset(*offset_seconds)),
        )),
        Value::Property(PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        }) => {
            let (d, n) = temporal::split_epoch_seconds(*epoch_seconds);
            Ok((Some(d), Some(n + *nanos as i64), None))
        }
        Value::Property(PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        }) => {
            let tz = tz_from_graph(zone);
            let offset_seconds = temporal::resolve_offset(&tz, *epoch_seconds);
            let local = epoch_seconds + offset_seconds as i64;
            let (d, n) = temporal::split_epoch_seconds(local);
            Ok((Some(d), Some(n + *nanos as i64), Some(tz)))
        }
        other => Err(QueryError::Type(format!(
            "{name}() needs a Date, LocalTime, Time, LocalDateTime, or DateTime, got {other:?}"
        ))),
    }
}

/// `(epoch_day, nanos_of_day, zone)`, see `between_operand`'s docs.
pub(crate) type BetweenOperand = (Option<i64>, Option<i64>, Option<temporal::TzId>);

/// `(a_epoch_day, a_nanos_of_day, a_zone, b_epoch_day,
/// b_nanos_of_day, b_zone) -> DurationParts` -- the shape
/// every `temporal::duration_between`/`duration_in_months`/
/// `duration_in_days`/`duration_in_seconds` function shares.
pub(crate) type BetweenFn = fn(
    Option<i64>,
    Option<i64>,
    Option<&temporal::TzId>,
    Option<i64>,
    Option<i64>,
    Option<&temporal::TzId>,
) -> temporal::DurationParts;

/// Shared dispatch for `duration.between`/`.inMonths`/`.inDays`/
/// `.inSeconds` -- all 4 take exactly 2 temporal args and differ only
/// in which `temporal.rs` decomposition function turns the pair into a
/// `Duration`.
pub(crate) fn duration_between_builtin(
    name: &str,
    args: &[Value],
    f: BetweenFn,
) -> Result<Value, QueryError> {
    if args.len() != 2 {
        return Err(QueryError::Semantic(format!(
            "{name}() expects exactly two arguments, got {}",
            args.len()
        )));
    }
    if matches!(args[0], Value::Null) || matches!(args[1], Value::Null) {
        return Ok(Value::Null);
    }
    let (a_date, a_time, a_zone) = between_operand(name, &args[0])?;
    let (b_date, b_time, b_zone) = between_operand(name, &args[1])?;
    Ok(duration_value(f(
        a_date,
        a_time,
        a_zone.as_ref(),
        b_date,
        b_time,
        b_zone.as_ref(),
    )))
}

/// `<type>.truncate(unit, value, map?)`'s first two/three args -- `unit`
/// is a string literal, `value` the source temporal value, and the
/// trailing map (if present and non-null) carries field overrides
/// applied *after* truncation.
pub(crate) type TruncateArgs<'a> = (&'a str, &'a Value, Option<&'a BTreeMap<String, Value>>);

pub(crate) fn parse_truncate_args<'a>(
    name: &str,
    args: &'a [Value],
) -> Result<TruncateArgs<'a>, QueryError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(QueryError::Semantic(format!(
            "{name}() expects 2 or 3 arguments, got {}",
            args.len()
        )));
    }
    let unit = as_arith_str(&args[0]).ok_or_else(|| {
        QueryError::Type(format!("{name}()'s first argument must be a unit string"))
    })?;
    let map = match args.get(2) {
        None | Some(Value::Null) => None,
        Some(Value::Map(m)) => Some(m),
        Some(other) => {
            return Err(QueryError::Type(format!(
                "{name}()'s third argument must be a map, got {other:?}"
            )))
        }
    };
    Ok((unit, &args[1], map))
}

/// `year`/`month`/`day`/`dayOfWeek` overrides shared by every
/// `.truncate()` builtin's optional trailing map -- any key the map
/// doesn't set keeps the truncated base's own value (`date.truncate(
/// 'month', d, {day: 5})` keeps the truncated year/month, only `day`
/// is overridden). `dayOfWeek` applies *after* year/month/day (moving
/// within the resulting date's own ISO week, see `set_iso_weekday`'s
/// docs) -- other week/quarter/ordinal-day override keys stay
/// unsupported, the same pre-existing construction gap as `date_from_map`.
pub(crate) fn apply_date_overrides(
    base_epoch_day: i64,
    map: Option<&BTreeMap<String, Value>>,
) -> Result<i64, QueryError> {
    let base_y = temporal::date_component(base_epoch_day, "year").unwrap();
    let base_m = temporal::date_component(base_epoch_day, "month").unwrap();
    let base_d = temporal::date_component(base_epoch_day, "day").unwrap();
    let Some(m) = map else {
        return Ok(base_epoch_day);
    };
    let year = int_field(m, "year", base_y)?;
    let month_raw = int_field(m, "month", base_m)?;
    let month = u32::try_from(month_raw)
        .map_err(|_| QueryError::Type(format!("'month' is out of range: {month_raw}")))?;
    let day_raw = int_field(m, "day", base_d)?;
    let day = u32::try_from(day_raw)
        .map_err(|_| QueryError::Type(format!("'day' is out of range: {day_raw}")))?;
    let result = temporal::epoch_day_from_ymd(year, month, day).ok_or_else(|| {
        QueryError::Type(format!(
            "{year:04}-{month:02}-{day:02} isn't a valid calendar date"
        ))
    })?;
    match m.get("dayOfWeek") {
        None => Ok(result),
        Some(v) => {
            let dow = value_as_i64(v)
                .ok_or_else(|| QueryError::Type("'dayOfWeek' must be an integer".into()))?;
            temporal::set_iso_weekday(result, dow).ok_or_else(|| {
                QueryError::Type(format!(
                    "'dayOfWeek' must be 1..7 (Monday..Sunday), got {dow}"
                ))
            })
        }
    }
}

/// `hour`/`minute`/`second`/`millisecond`/`microsecond`/`nanosecond`
/// overrides shared by every `.truncate()` builtin's optional trailing
/// map -- same "unset key keeps the truncated base's value" rule as
/// `apply_date_overrides`.
pub(crate) fn apply_time_overrides(
    base_nanos_of_day: i64,
    map: Option<&BTreeMap<String, Value>>,
) -> Result<i64, QueryError> {
    let base_h = temporal::local_time_component(base_nanos_of_day, "hour").unwrap();
    let base_min = temporal::local_time_component(base_nanos_of_day, "minute").unwrap();
    let base_s = temporal::local_time_component(base_nanos_of_day, "second").unwrap();
    let base_ns = temporal::local_time_component(base_nanos_of_day, "nanosecond").unwrap();
    let Some(m) = map else {
        return Ok(base_nanos_of_day);
    };
    let nanos = sub_second_nanos_from_map(base_ns, m)?;
    let hour = int_field(m, "hour", base_h)?;
    let minute = int_field(m, "minute", base_min)?;
    let second = int_field(m, "second", base_s)?;
    temporal::local_time_nanos_from_fields(hour, minute, second, nanos)
        .ok_or_else(|| QueryError::Type("truncate(...)'s map has an out-of-range field".into()))
}

/// Rejects a `.truncate()` map key that this specific target type has
/// no field for (e.g. `hour` on `date.truncate`'s result, which is a
/// bare `Date`) -- each of the 5 truncate builtins passes its own real
/// field list, since `apply_date_overrides`/`apply_time_overrides`
/// themselves are shared and don't know which caller's result shape
/// makes a given key meaningful.
pub(crate) fn validate_truncate_map_keys(
    name: &str,
    map: Option<&BTreeMap<String, Value>>,
    allowed: &[&str],
) -> Result<(), QueryError> {
    let Some(m) = map else { return Ok(()) };
    if let Some(bad) = m.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(QueryError::Type(format!(
            "{name}(...)'s map has an unrecognized field '{bad}'"
        )));
    }
    Ok(())
}

pub(crate) fn date_truncate_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let (unit, other, map) = parse_truncate_args("date.truncate", args)?;
    validate_truncate_map_keys("date.truncate", map, &["year", "month", "day", "dayOfWeek"])?;
    if matches!(other, Value::Null) {
        return Ok(Value::Null);
    }
    let (base_date, _, _) = between_operand("date.truncate", other)?;
    let base_date = base_date.ok_or_else(|| {
        QueryError::Type(
            "date.truncate() needs a value with a calendar date (Date, LocalDateTime, or DateTime)"
                .into(),
        )
    })?;
    let truncated = temporal::truncate_date_unit(base_date, unit).ok_or_else(|| {
        QueryError::Type(format!(
            "date.truncate(): '{unit}' isn't a recognized date unit"
        ))
    })?;
    Ok(Value::Property(PropertyValue::Date(apply_date_overrides(
        truncated, map,
    )?)))
}

pub(crate) const TIME_TRUNCATE_MAP_KEYS: &[&str] = &[
    "hour",
    "minute",
    "second",
    "millisecond",
    "microsecond",
    "nanosecond",
];

pub(crate) fn local_time_truncate_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let (unit, other, map) = parse_truncate_args("localtime.truncate", args)?;
    validate_truncate_map_keys("localtime.truncate", map, TIME_TRUNCATE_MAP_KEYS)?;
    if matches!(other, Value::Null) {
        return Ok(Value::Null);
    }
    let (_, base_time, _) = between_operand("localtime.truncate", other)?;
    let base_time = base_time.ok_or_else(|| {
        QueryError::Type(
            "localtime.truncate() needs a value with a time-of-day (LocalTime, Time, \
             LocalDateTime, or DateTime)"
                .into(),
        )
    })?;
    let truncated = temporal::truncate_time_unit(base_time, unit).ok_or_else(|| {
        QueryError::Type(format!(
            "localtime.truncate(): '{unit}' isn't a recognized time unit"
        ))
    })?;
    Ok(Value::Property(PropertyValue::LocalTime(
        apply_time_overrides(truncated, map)?,
    )))
}

pub(crate) fn time_truncate_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let (unit, other, map) = parse_truncate_args("time.truncate", args)?;
    validate_truncate_map_keys(
        "time.truncate",
        map,
        &[
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
            "timezone",
        ],
    )?;
    if matches!(other, Value::Null) {
        return Ok(Value::Null);
    }
    let (_, base_time, base_offset) = between_operand("time.truncate", other)?;
    let base_time = base_time.ok_or_else(|| {
        QueryError::Type(
            "time.truncate() needs a value with a time-of-day (LocalTime, Time, LocalDateTime, \
             or DateTime)"
                .into(),
        )
    })?;
    let truncated = temporal::truncate_time_unit(base_time, unit).ok_or_else(|| {
        QueryError::Type(format!(
            "time.truncate(): '{unit}' isn't a recognized time unit"
        ))
    })?;
    let nanos_of_day = apply_time_overrides(truncated, map)?;
    let offset_seconds = match map.and_then(|m| m.get("timezone")) {
        Some(v) => match timezone_value_to_tzid(v)? {
            temporal::TzId::Offset(o) => o,
            temporal::TzId::Named(name) => {
                return Err(QueryError::Type(format!(
                    "'timezone': '{name}' looks like a named timezone (e.g. 'Europe/Stockholm') -- TIME has \
                     no calendar date to resolve a named zone's DST-dependent offset against, only a fixed \
                     UTC offset like '+01:00' is supported"
                )));
            }
        },
        None => match base_offset {
            Some(temporal::TzId::Offset(o)) => o,
            _ => 0,
        },
    };
    Ok(Value::Property(PropertyValue::Time {
        nanos_of_day,
        offset_seconds,
    }))
}

/// Shared by `localdatetime.truncate`/`datetime.truncate`: a calendar-
/// scale `unit` (`year`, `month`, ...) truncates the date and resets
/// the time-of-day to midnight; a clock-scale `unit` (`hour`,
/// `minute`, ...) leaves the date untouched and truncates just the
/// time. `day` is both at once (`truncate_date_unit`'s own `day` arm
/// already returns the date unchanged), so trying the date-unit path
/// first handles it correctly without a separate case.
pub(crate) fn truncate_date_time(base_date: i64, base_time: i64, unit: &str) -> Option<(i64, i64)> {
    if let Some(d) = temporal::truncate_date_unit(base_date, unit) {
        Some((d, 0))
    } else {
        temporal::truncate_time_unit(base_time, unit).map(|t| (base_date, t))
    }
}

pub(crate) fn local_date_time_truncate_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let (unit, other, map) = parse_truncate_args("localdatetime.truncate", args)?;
    validate_truncate_map_keys(
        "localdatetime.truncate",
        map,
        &[
            "year",
            "month",
            "day",
            "dayOfWeek",
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
        ],
    )?;
    if matches!(other, Value::Null) {
        return Ok(Value::Null);
    }
    let (base_date, base_time, _) = between_operand("localdatetime.truncate", other)?;
    let base_date = base_date.ok_or_else(|| {
        QueryError::Type(
            "localdatetime.truncate() needs a value with a calendar date (Date, LocalDateTime, \
             or DateTime)"
                .into(),
        )
    })?;
    let (trunc_date, trunc_time) = truncate_date_time(base_date, base_time.unwrap_or(0), unit)
        .ok_or_else(|| {
            QueryError::Type(format!(
                "localdatetime.truncate(): '{unit}' isn't a recognized unit"
            ))
        })?;
    let final_date = apply_date_overrides(trunc_date, map)?;
    let final_time = apply_time_overrides(trunc_time, map)?;
    let (epoch_seconds, nanos) = temporal::combine_date_and_time(final_date, final_time);
    Ok(Value::Property(PropertyValue::LocalDateTime {
        epoch_seconds,
        nanos,
    }))
}

pub(crate) fn date_time_truncate_builtin(args: &[Value]) -> Result<Value, QueryError> {
    let (unit, other, map) = parse_truncate_args("datetime.truncate", args)?;
    validate_truncate_map_keys(
        "datetime.truncate",
        map,
        &[
            "year",
            "month",
            "day",
            "dayOfWeek",
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
            "timezone",
        ],
    )?;
    if matches!(other, Value::Null) {
        return Ok(Value::Null);
    }
    let (base_date, base_time, base_offset) = between_operand("datetime.truncate", other)?;
    let base_date = base_date.ok_or_else(|| {
        QueryError::Type(
            "datetime.truncate() needs a value with a calendar date (Date, LocalDateTime, or \
             DateTime)"
                .into(),
        )
    })?;
    let (trunc_date, trunc_time) = truncate_date_time(base_date, base_time.unwrap_or(0), unit)
        .ok_or_else(|| {
            QueryError::Type(format!(
                "datetime.truncate(): '{unit}' isn't a recognized unit"
            ))
        })?;
    let final_date = apply_date_overrides(trunc_date, map)?;
    let final_time = apply_time_overrides(trunc_time, map)?;
    let zone = match map.and_then(|m| m.get("timezone")) {
        Some(v) => timezone_value_to_tzid(v)?,
        None => base_offset.unwrap_or(temporal::TzId::Offset(0)),
    };
    let calendar = temporal::CalendarDateTime {
        year: temporal::date_component(final_date, "year").unwrap(),
        month: temporal::date_component(final_date, "month").unwrap() as u32,
        day: temporal::date_component(final_date, "day").unwrap() as u32,
        hour: temporal::local_time_component(final_time, "hour").unwrap(),
        minute: temporal::local_time_component(final_time, "minute").unwrap(),
        second: temporal::local_time_component(final_time, "second").unwrap(),
        nanos: temporal::local_time_component(final_time, "nanosecond").unwrap(),
    };
    let (epoch_seconds, nanos) =
        temporal::date_time_from_fields(calendar, &zone).ok_or_else(|| {
            QueryError::Type("datetime.truncate() produced an out-of-range value".into())
        })?;
    Ok(Value::Property(PropertyValue::DateTime {
        epoch_seconds,
        nanos,
        zone: tz_to_graph(zone),
    }))
}

/// Shared `Date`/`Duration` component access for `d.<prop>` — used by
/// both `lookup_prop` (a bound row variable, e.g. `WITH v.date AS d ...
/// d.year`) and `eval_projected_expr`'s `Prop` arm (the post-projection/
/// ORDER BY path). Returns `None` for any property name that isn't a
/// recognized component (or a non-temporal `PropertyValue`), the same
/// "treat as absent, not an error" convention every other `.prop` access
/// already follows for an unknown property.
/// True for the 6 `PropertyValue` variants that have a real `.prop`
/// component-access interface (`temporal_component`) -- distinguishes
/// "a temporal value with an *unrecognized* property name" (still `null`,
/// same as a node/edge's own missing-property rule) from "a plain scalar
/// with *no* `.prop` interface at all" (a real type error, see
/// `lookup_prop_value`'s docs) -- `temporal_component` alone can't tell
/// these apart, since it returns `None` for both.
pub(crate) fn is_temporal_property_value(pv: &PropertyValue) -> bool {
    matches!(
        pv,
        PropertyValue::Date(_)
            | PropertyValue::Duration { .. }
            | PropertyValue::LocalTime(_)
            | PropertyValue::Time { .. }
            | PropertyValue::LocalDateTime { .. }
            | PropertyValue::DateTime { .. }
    )
}

pub(crate) fn temporal_component(pv: &PropertyValue, prop: &str) -> Option<PropertyValue> {
    match pv {
        PropertyValue::Date(d) => temporal::date_component(*d, prop).map(PropertyValue::Int),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => temporal::duration_component(*months, *days, *seconds, *nanos, prop)
            .map(PropertyValue::Int),
        PropertyValue::LocalTime(nanos_of_day) => {
            temporal::local_time_component(*nanos_of_day, prop).map(PropertyValue::Int)
        }
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => time_component(*nanos_of_day, *offset_seconds, prop),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => date_time_component(*epoch_seconds, *nanos, None, prop),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => date_time_component(*epoch_seconds, *nanos, Some(&tz_from_graph(zone)), prop),
        _ => None,
    }
}

/// `Time`'s own component set: `LocalTime`'s fields plus the offset
/// ones (`timezone`/`offset` as text, `offsetSeconds`/`offsetMinutes`
/// as integers).
pub(crate) fn time_component(
    nanos_of_day: i64,
    offset_seconds: i32,
    prop: &str,
) -> Option<PropertyValue> {
    match prop {
        "timezone" | "offset" => Some(PropertyValue::String(temporal::format_offset(
            offset_seconds,
        ))),
        "offsetSeconds" => Some(PropertyValue::Int(offset_seconds as i64)),
        "offsetMinutes" => Some(PropertyValue::Int(offset_seconds as i64 / 60)),
        _ => temporal::local_time_component(nanos_of_day, prop).map(PropertyValue::Int),
    }
}

/// `LocalDateTime`/`DateTime`'s shared component set: every `Date`
/// component, every `LocalTime` component, and (only when
/// `offset_seconds` is `Some`, i.e. a real `DateTime`) the same offset/
/// epoch fields `Time`/this-function's own `epochSeconds`/`epochMillis`
/// add on top.
///
/// Calendar/clock components (`year`..`nanosecond`) are computed against
/// the *local* (offset-adjusted) wall-clock reading, not the stored UTC
/// instant -- `datetime({..., hour: 12, timezone: '+01:00'}).hour` must
/// answer `12` (what was written/displayed), not `11` (the UTC hour) --
/// same "display the local reading" rule `format_date_time` already
/// follows. `epochSeconds`/`epochMillis` are the one exception,
/// deliberately using the raw (UTC) `epoch_seconds` -- "epoch" always
/// means the UTC instant, regardless of offset.
pub(crate) fn date_time_component(
    epoch_seconds: i64,
    nanos: i32,
    zone: Option<&temporal::TzId>,
    prop: &str,
) -> Option<PropertyValue> {
    if let Some(zone) = zone {
        let offset_seconds = temporal::resolve_offset(zone, epoch_seconds);
        match prop {
            // `.timezone` is the zone *identifier* as written -- the
            // zone name for a `Named` zone, or the offset text itself
            // for a fixed `Offset` (there's no separate name); `.offset`
            // is always the *resolved* offset text, so the two only
            // diverge for a `Named` zone (TCK's Temporal5's `d.timezone`
            // = `'Europe/Stockholm'` vs `d.offset` = `'+01:00'`).
            "timezone" => {
                let text = match zone {
                    temporal::TzId::Named(name) => name.clone(),
                    temporal::TzId::Offset(_) => temporal::format_offset(offset_seconds),
                };
                return Some(PropertyValue::String(text));
            }
            "offset" => {
                return Some(PropertyValue::String(temporal::format_offset(
                    offset_seconds,
                )))
            }
            "offsetSeconds" => return Some(PropertyValue::Int(offset_seconds as i64)),
            "offsetMinutes" => return Some(PropertyValue::Int(offset_seconds as i64 / 60)),
            "epochSeconds" => return Some(PropertyValue::Int(epoch_seconds)),
            "epochMillis" => {
                return Some(PropertyValue::Int(
                    temporal::epoch_seconds_and_millis(epoch_seconds, nanos).1,
                ))
            }
            _ => {}
        }
    }
    let offset_seconds = zone.map_or(0, |z| temporal::resolve_offset(z, epoch_seconds));
    let local_epoch_seconds = epoch_seconds + offset_seconds as i64;
    temporal::date_time_calendar_component(local_epoch_seconds, prop)
        .or_else(|| temporal::date_time_clock_component(local_epoch_seconds, nanos, prop))
        .map(PropertyValue::Int)
}
