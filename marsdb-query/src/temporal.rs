//! Calendar math and ISO-8601 text conversion for `PropertyValue::Date`/
//! `PropertyValue::Duration` -- kept out of `marsdb-graph` deliberately
//! (that crate stores the value, it doesn't know Cypher's construction/
//! formatting rules -- see `PropertyValue`'s own doc comment) and out of
//! `executor.rs` (which owns *dispatching* to these, not the arithmetic
//! itself, matching the split `apply_arith`/`compare` already have from
//! e.g. the planner).
//!
//! Scope, honestly: `DATE` (calendar year/month/day, ISO week-date, and
//! ordinal/quarter-date construction forms), `DURATION`, `LOCAL TIME`,
//! `TIME`, `LOCAL DATETIME`, and `DATETIME` are all supported -- but
//! `TIME`/`DATETIME` only accept a *fixed* UTC offset (`'+01:00'`,
//! `{timezone: '+01:00'}`), never a named timezone (`'Europe/Stockholm'`)
//! -- that needs a real IANA timezone database, deliberately out of
//! scope (no DST/zone-rule awareness anywhere in this module). See the
//! README's "Cypher coverage" section for the exact list of what that
//! leaves out of TCK's `expressions/temporal` suite.

use chrono::{
    Datelike, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};

/// A `DateTime`'s zone -- a plain, `marsdb_graph`-independent mirror of
/// `PropertyValue::DateTime`'s own `zone: marsdb_graph::model::TzId`
/// field (same reasoning as `DurationParts` below: this module doesn't
/// depend on `marsdb_graph`), translated at the `executor.rs` boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum TzId {
    Offset(i32),
    Named(String),
}

const SECONDS_PER_DAY: i64 = 86_400;

/// Average Gregorian month length in days (365.2425 / 12) -- Neo4j's own
/// documented conversion factor for folding a fractional month (e.g. the
/// `0.75` in `duration({months: 0.75})`) down into days, since "0.75
/// months" has no exact length in days without a reference date. Only
/// ever applied to the *fractional remainder* of a month count, never the
/// whole-number part (a whole month always stays a whole month in the
/// normalized representation, added/subtracted from a `Date` via real
/// calendar month arithmetic in `add_duration_to_date`, not this
/// average).
const AVG_MONTH_DAYS: f64 = 365.2425 / 12.0;

const NANOS_PER_SEC: i128 = 1_000_000_000;

/// `PropertyValue::Date`'s epoch-day origin -- 1970-01-01, matching the
/// same convention `std::time::UNIX_EPOCH`/most other systems use, so
/// nothing here needs to remember an unusual offset.
fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date")
}

pub fn epoch_day_from_ymd(year: i32, month: u32, day: u32) -> Option<i32> {
    let d = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(d.signed_duration_since(epoch()).num_days() as i32)
}

fn date_from_epoch_day(epoch_day: i32) -> NaiveDate {
    epoch() + chrono::Duration::days(epoch_day as i64)
}

/// A single captured instant, pre-derived into every shape a no-arg
/// `date()`/`localtime()`/`time()`/`localdatetime()`/`datetime()` call
/// needs -- real Cypher guarantees every such call *within the same
/// query* returns the same value (so `duration.between(date(), date())`
/// is always `PT0S`, never a few-microseconds-off nonzero duration from
/// two independent `now()` reads); capturing one `chrono::Utc::now()`
/// and deriving every field from it (not one `now()` call per field)
/// is what makes that guarantee hold even within a single construction.
#[derive(Clone, Copy)]
pub struct NowSnapshot {
    pub epoch_day: i32,
    pub nanos_of_day: i64,
    pub epoch_seconds: i64,
    pub nanos: i32,
}

pub fn capture_now() -> NowSnapshot {
    let now = chrono::Utc::now();
    let epoch_seconds = now.timestamp();
    let nanos = now.nanosecond() as i32;
    let (epoch_day, nanos_of_day) = split_epoch_seconds(epoch_seconds);
    NowSnapshot {
        epoch_day,
        nanos_of_day: nanos_of_day + nanos as i64,
        epoch_seconds,
        nanos,
    }
}

pub fn format_date(epoch_day: i32) -> String {
    let d = date_from_epoch_day(epoch_day);
    // `{:04}` pads a positive year to at least 4 digits (real Cypher/ISO-
    // 8601's normal case); a negative or >9999 year prints with however
    // many digits it needs rather than a fixed width -- MarsDB doesn't
    // claim exact ISO-8601 extended-year formatting, just enough to
    // round-trip through `parse_date` for the realistic years the TCK
    // (and any real workload) actually exercises.
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// Parses every date string form MarsDB supports: the plain calendar
/// forms `YYYY-MM-DD`/`YYYYMMDD`/`YYYY-MM`/`YYYYMM`/`YYYY` (missing
/// month/day default to `1`), ISO week-date `YYYY-Www[-D]`/`YYYYWww[D]`
/// (missing day defaults to `1`), and ordinal-date `YYYY-DDD`/`YYYYDDD`
/// (see `parse_week_or_ordinal_date`).
pub fn parse_date(s: &str) -> Option<i32> {
    let s = s.trim();
    // The compact forms below use byte offsets because their grammar is
    // ASCII-only. Reject non-ASCII input before slicing so malformed user
    // input can never put an offset in the middle of a UTF-8 code point.
    if !s.is_ascii() {
        return None;
    }
    // ISO week-date (`YYYY-Www[-D]` / `YYYYWww[D]`) and ordinal-date
    // (`YYYY-DDD` / `YYYYDDD`) forms -- checked before the plain calendar
    // forms below since a `W` unambiguously marks a week-date, and a
    // 7-digit no-`-` run is ordinal (a plain compact calendar date is
    // either 4, 6, or 8 digits, never 7).
    if let Some(epoch_day) = parse_week_or_ordinal_date(s) {
        return Some(epoch_day);
    }
    let (year, month, day) = if let Some((y, rest)) = s.split_once('-') {
        let year: i32 = y.parse().ok()?;
        match rest.split_once('-') {
            Some((m, d)) => (year, m.parse().ok()?, d.parse().ok()?),
            None => (year, rest.parse().ok()?, 1),
        }
    } else {
        match s.len() {
            8 => (
                s[0..4].parse().ok()?,
                s[4..6].parse().ok()?,
                s[6..8].parse().ok()?,
            ),
            6 => (s[0..4].parse().ok()?, s[4..6].parse().ok()?, 1),
            4 => (s[0..4].parse().ok()?, 1, 1),
            _ => return None,
        }
    };
    epoch_day_from_ymd(year, month, day)
}

/// ISO week-date (`YYYY-Www[-D]` / `YYYYWww[D]`, day defaults to `1` when
/// omitted) and ordinal-date (`YYYY-DDD` / `YYYYDDD`) string forms --
/// `None` for anything not matching one of these two shapes (the plain
/// calendar forms fall through to `parse_date`'s own parsing).
fn parse_week_or_ordinal_date(s: &str) -> Option<i32> {
    if let Some((y, rest)) = s.split_once('-') {
        if let Some(w) = rest.strip_prefix('W') {
            let week_year: i32 = y.parse().ok()?;
            let (week, day) = match w.split_once('-') {
                Some((w, d)) => (w.parse().ok()?, d.parse().ok()?),
                None => (w.parse().ok()?, 1),
            };
            return epoch_day_from_week_fields(week_year, week, day);
        }
        // `YYYY-DDD` -- an ordinal date, distinguished from the plain
        // `YYYY-MM` calendar form by `rest`'s length (3 digits, not 2).
        if rest.len() == 3 && rest.bytes().all(|b| b.is_ascii_digit()) {
            let year: i32 = y.parse().ok()?;
            let ordinal: u32 = rest.parse().ok()?;
            return epoch_day_from_ordinal_fields(year, ordinal);
        }
        return None;
    }
    if s.len() >= 5 {
        if let Some(w) = s[4..].strip_prefix('W') {
            let week_year: i32 = s[0..4].parse().ok()?;
            let (week, day) = match w.len() {
                2 => (w.parse().ok()?, 1),
                3 => (w[0..2].parse().ok()?, w[2..3].parse().ok()?),
                _ => return None,
            };
            return epoch_day_from_week_fields(week_year, week, day);
        }
    }
    if s.len() == 7 && s.bytes().all(|b| b.is_ascii_digit()) {
        let year: i32 = s[0..4].parse().ok()?;
        let ordinal: u32 = s[4..7].parse().ok()?;
        return epoch_day_from_ordinal_fields(year, ordinal);
    }
    None
}

/// `d.<prop>` component access for a `Date` -- the "forward" (date ->
/// components) half of ISO week/quarter calendar math; the "backward"
/// half (`week`/`dayOfWeek`/`quarter`/`dayOfQuarter`/`ordinalDay` ->
/// date) lives in `epoch_day_from_week_fields`/`epoch_day_from_ordinal_
/// fields`/`epoch_day_from_quarter_fields` below. Returns `None` for any
/// property name this doesn't recognize (the caller treats that the same
/// as a missing property, matching every other `.prop` access in this
/// codebase).
pub fn date_component(epoch_day: i32, prop: &str) -> Option<i64> {
    let d = date_from_epoch_day(epoch_day);
    Some(match prop {
        "year" => d.year() as i64,
        "month" => d.month() as i64,
        "day" => d.day() as i64,
        "quarter" => ((d.month() - 1) / 3 + 1) as i64,
        "ordinalDay" => d.ordinal() as i64,
        "weekDay" | "dayOfWeek" => d.weekday().number_from_monday() as i64,
        "week" => d.iso_week().week() as i64,
        "weekYear" => d.iso_week().year() as i64,
        "dayOfQuarter" => {
            let quarter_start_month = (d.month() - 1) / 3 * 3 + 1;
            let quarter_start = NaiveDate::from_ymd_opt(d.year(), quarter_start_month, 1)?;
            d.signed_duration_since(quarter_start).num_days() + 1
        }
        _ => return None,
    })
}

/// ISO week-date's `1..=7` (Monday=1) -> chrono's `Weekday`, the inverse
/// of `date_component`'s `"dayOfWeek"` (`number_from_monday`).
fn weekday_from_iso_number(n: i64) -> Option<chrono::Weekday> {
    use chrono::Weekday::*;
    Some(match n {
        1 => Mon,
        2 => Tue,
        3 => Wed,
        4 => Thu,
        5 => Fri,
        6 => Sat,
        7 => Sun,
        _ => return None,
    })
}

/// Constructs an epoch-day from ISO week-date fields -- the inverse of
/// `date_component`'s `"weekYear"`/`"week"`/`"dayOfWeek"` accessors.
/// `week_year` is the ISO week-numbering year, not necessarily the
/// calendar year of the resulting date (they diverge near a year
/// boundary -- e.g. week-year 1817 week 1 day 2 is calendar date
/// 1816-12-31, TCK's Temporal1 [1]).
pub fn epoch_day_from_week_fields(week_year: i32, week: u32, day_of_week: i64) -> Option<i32> {
    let weekday = weekday_from_iso_number(day_of_week)?;
    let d = NaiveDate::from_isoywd_opt(week_year, week, weekday)?;
    Some(d.signed_duration_since(epoch()).num_days() as i32)
}

/// Constructs an epoch-day from a calendar year plus an ordinal day
/// (`1..=365`/`366`) -- the inverse of `date_component`'s `"ordinalDay"`.
pub fn epoch_day_from_ordinal_fields(year: i32, ordinal_day: u32) -> Option<i32> {
    let d = NaiveDate::from_yo_opt(year, ordinal_day)?;
    Some(d.signed_duration_since(epoch()).num_days() as i32)
}

/// Constructs an epoch-day from a calendar year, quarter (`1..=4`), and
/// day-of-quarter (`1`-based) -- the inverse of `date_component`'s
/// `"quarter"`/`"dayOfQuarter"`.
pub fn epoch_day_from_quarter_fields(year: i32, quarter: u32, day_of_quarter: i64) -> Option<i32> {
    if !(1..=4).contains(&quarter) {
        return None;
    }
    let quarter_start_month = (quarter - 1) * 3 + 1;
    let quarter_start = NaiveDate::from_ymd_opt(year, quarter_start_month, 1)?;
    let d = quarter_start.checked_add_signed(chrono::Duration::days(day_of_quarter - 1))?;
    Some(d.signed_duration_since(epoch()).num_days() as i32)
}

/// Adds a `Duration` to a `Date` via real calendar month arithmetic
/// (`checked_add_months`/`checked_sub_months`, which clamps to the
/// shorter month's last day -- e.g. Jan 31 + 1 month = Feb 28/29, not an
/// error and not Mar 3) followed by a plain day offset. `negate`: `true`
/// for `date - duration` (real Cypher's other overload), reusing the same
/// function rather than duplicating it with `-` in every arithmetic
/// expression.
///
/// `seconds`/`nanos` can't shift a `Date` by a fraction of a day (it has
/// no time-of-day to carry a remainder into), but they're *not* simply
/// dropped either -- any *whole* extra day they add still counts: e.g.
/// `duration({months: 0.5, days: 14.5, hours: 16.5, ...})` normalizes to
/// `days: 29` plus a `seconds`/`nanos` remainder equivalent to ~34 hours,
/// and that 34 hours contributes one more whole day (34h > 24h) on top of
/// the 29 -- verified against Temporal8's fractional-duration date-
/// arithmetic scenario, which is exactly the case that exposed this (an
/// earlier version of this function dropped `seconds`/`nanos` outright
/// and was a day off). `seconds/86_400` (truncated towards zero, so a
/// negative duration's extra day is subtracted, not added) is the whole-
/// day count; anything finer than that is genuinely discarded, matching
/// "adding a Duration to a value with less precision than the Duration
/// provides truncates to that lower precision" -- Date's precision floor
/// is one day.
pub fn add_duration_to_date(
    epoch_day: i32,
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i32,
    negate: bool,
) -> Option<i32> {
    let total_ns: i128 = seconds as i128 * NANOS_PER_SEC + nanos as i128;
    let extra_days = (total_ns / (86_400 * NANOS_PER_SEC)) as i64;
    let days = days.checked_add(extra_days)?;
    let (months, days) = if negate {
        (months.checked_neg()?, days.checked_neg()?)
    } else {
        (months, days)
    };
    let d = date_from_epoch_day(epoch_day);
    let with_months = if months >= 0 {
        d.checked_add_months(chrono::Months::new(months.try_into().ok()?))?
    } else {
        d.checked_sub_months(chrono::Months::new(months.checked_neg()?.try_into().ok()?))?
    };
    let result = with_months.checked_add_signed(chrono::Duration::try_days(days)?)?;
    Some(result.signed_duration_since(epoch()).num_days() as i32)
}

/// The four independently-signed components of a normalized `Duration`,
/// matching `PropertyValue::Duration`'s own fields exactly -- a plain
/// tuple alias, not a re-export of the `PropertyValue` variant itself,
/// since this module deliberately doesn't depend on `marsdb_graph` (see
/// this file's top-of-module doc comment on the crate split).
pub type DurationParts = (i64, i64, i64, i32);

/// Raw, not-yet-normalized inputs to `duration({...})`/`duration('...')`
/// construction -- one `f64` per Cypher map key (`0.0` when absent), kept
/// as a struct (not 10 positional `f64` args) so call sites read as
/// `years: 12.0, ..Default::default()` rather than an unlabeled tuple.
#[derive(Default, Clone, Copy)]
pub struct DurationFields {
    pub years: f64,
    pub months: f64,
    pub weeks: f64,
    pub days: f64,
    pub hours: f64,
    pub minutes: f64,
    pub seconds: f64,
    pub milliseconds: f64,
    pub microseconds: f64,
    pub nanoseconds: f64,
}

/// Folds raw (possibly fractional, possibly negative) field values into
/// `PropertyValue::Duration`'s normalized `(months, days, seconds,
/// nanos)` form. The cascade only ever flows one direction -- years into
/// months, a fractional month's remainder into days (via `AVG_MONTH_DAYS`
/// -- the only place that average is used), a fractional day's remainder
/// into seconds, sub-second fields into nanoseconds -- matching Neo4j's
/// own documented normalization, verified line-by-line against every
/// `duration(...)` example in the TCK's Temporal1/Temporal2 feature
/// files. Never the other direction (seconds never cascade *into* days --
/// `duration({hours: 40})` stays `PT40H`, not `P1DT16H`; a "day" isn't a
/// fixed number of hours once timezones/DST exist, so real Cypher never
/// makes that assumption even though MarsDB's own `Date` type is
/// timezone-naive).
pub fn normalize_duration(f: DurationFields) -> DurationParts {
    let months_f = f.years * 12.0 + f.months;
    let days_f = f.weeks * 7.0 + f.days;
    let seconds_f = f.hours * 3600.0 + f.minutes * 60.0 + f.seconds;
    // Sub-second fields are exact integer nanosecond counts in every real
    // scenario (`nanosecond: 789`, never a fractional nanosecond) --
    // `.trunc()`, not `.round()`, so a hypothetical fractional input
    // doesn't get a phantom extra nanosecond rounded in.
    let extra_nanos =
        (f.milliseconds * 1_000_000.0 + f.microseconds * 1_000.0 + f.nanoseconds).trunc() as i128;
    cascade(months_f, days_f, seconds_f, extra_nanos)
}

/// Shared cascade core for both `normalize_duration` (raw map/string
/// fields) and `scale_duration` (multiply/divide by a scalar) -- the only
/// difference between the two callers is what they pass as `seconds_f`/
/// `extra_nanos`, not the cascade logic itself.
fn cascade(months_f: f64, days_f: f64, seconds_f: f64, extra_nanos: i128) -> DurationParts {
    let whole_months = months_f.trunc();
    let frac_months = months_f - whole_months;
    let days_f2 = days_f + frac_months * AVG_MONTH_DAYS;
    let whole_days = days_f2.trunc();
    let frac_days = days_f2 - whole_days;
    let seconds_f2 = seconds_f + frac_days * 86_400.0;
    // `.round()` here (not `.trunc()`) -- `seconds_f2` is a continuous
    // quantity built from several multiplications/additions (e.g. the
    // `0.75` months -> `71509.5` seconds case), so it can land a
    // few-ULP hair off the exact value; rounding to the nearest whole
    // nanosecond recovers the exact intended value, whereas truncating
    // would occasionally drop a real nanosecond that FP noise pushed
    // just under the integer.
    let total_ns = (seconds_f2 * NANOS_PER_SEC as f64).round() as i128 + extra_nanos;
    let seconds = (total_ns / NANOS_PER_SEC) as i64;
    let nanos = (total_ns % NANOS_PER_SEC) as i32;
    (whole_months as i64, whole_days as i64, seconds, nanos)
}

/// Component-wise `a + b` -- *not* a re-cascade through `normalize_
/// duration` (months/days add directly, no re-derivation via
/// `AVG_MONTH_DAYS`), matching the TCK's "add two already-normalized
/// durations" examples, which sum months and days independently and only
/// ever carry between `seconds`/`nanos` (via the exact `i128` total,
/// avoiding the sign-mismatch bug a naive `a.nanos + b.nanos` would hit
/// when the two operands' `seconds` signs differ). Returns `None` if any
/// component would overflow its persisted integer representation.
pub fn add_duration(a: DurationParts, b: DurationParts) -> Option<DurationParts> {
    let months = a.0.checked_add(b.0)?;
    let days = a.1.checked_add(b.1)?;
    let total_ns =
        a.2 as i128 * NANOS_PER_SEC + a.3 as i128 + b.2 as i128 * NANOS_PER_SEC + b.3 as i128;
    Some((
        months,
        days,
        (total_ns / NANOS_PER_SEC).try_into().ok()?,
        (total_ns % NANOS_PER_SEC) as i32,
    ))
}

pub fn negate_duration(a: DurationParts) -> Option<DurationParts> {
    Some((
        a.0.checked_neg()?,
        a.1.checked_neg()?,
        a.2.checked_neg()?,
        a.3.checked_neg()?,
    ))
}

pub fn sub_duration(a: DurationParts, b: DurationParts) -> Option<DurationParts> {
    add_duration(a, negate_duration(b)?)
}

/// `duration * factor` / `duration / factor` (`factor` is `1.0 / n` for
/// division) -- re-cascades through the same `AVG_MONTH_DAYS`-based logic
/// `normalize_duration` uses (scaling a whole month by a non-integer
/// factor produces a fractional month again, e.g. `P1M / 2` needs to
/// become "15.2 days", not stay a fractional month), so this calls the
/// shared `cascade` directly with `months`/`days` pre-multiplied and the
/// exact `seconds`+`nanos` total pre-multiplied as one `i128` quantity
/// (truncated, same "no phantom sub-nanosecond digit" reasoning as
/// `normalize_duration`'s `extra_nanos`).
pub fn scale_duration(a: DurationParts, factor: f64) -> DurationParts {
    let months_f = a.0 as f64 * factor;
    let days_f = a.1 as f64 * factor;
    let total_ns_exact = a.2 as i128 * NANOS_PER_SEC + a.3 as i128;
    let extra_nanos = (total_ns_exact as f64 * factor).trunc() as i128;
    cascade(months_f, days_f, 0.0, extra_nanos)
}

/// `d.<prop>` component access for a `Duration` -- every field (`years`,
/// `quarters`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`,
/// `milliseconds`, `microseconds`, `nanoseconds`) is simply the *whole
/// duration re-expressed in that one unit alone*, truncated towards zero
/// -- not a calendar-style "the months-of-year part" breakdown. E.g. for
/// `duration({years: 1, months: 4, ...})` (16 total months), `d.years` is
/// `16 / 12 = 1` and `d.months` is `16` itself, not `4`. Verified against
/// every field in Temporal5's "accessors for duration" scenario. The
/// `*OfX` fields (`monthsOfYear`, `secondsOfMinute`, ...) are each the
/// same computation's *remainder* instead of its quotient -- literally
/// "what `d.<prop>` would be, mod the next unit up".
/// `seconds`/`nanos` are stored the same way real Cypher's own `Duration`
/// stores them (mirroring Java's `Duration`): `seconds` carries the whole
/// sign, `nanos` is always non-negative (0..999_999_999) -- see
/// `PropertyValue::Duration`'s own docs. Component accessors must read
/// off *these two raw fields directly*, not recombine them into one
/// signed total and re-split -- that would silently reintroduce a
/// negative `nanos` (`-23H-59M-59.9S`'s stored form is `seconds: -86400,
/// nanos: 100_000_000`; re-splitting `-86399.9s` via truncating division
/// gives the wrong `seconds: -86399, nanosecondsOfSecond: -900_000_000`
/// instead, TCK's Temporal10 `[1]`). `hours`/`minutes`/`seconds` (and
/// their `-OfHour`/`-OfMinute` cousins) only ever divide `seconds` itself
/// (never touch `nanos` -- a whole hour/minute can't hide inside a
/// sub-second remainder); `milliseconds`/`microseconds`/`nanoseconds`
/// (the fine-grained *totals*, not `-OfSecond` splits) are the one place
/// that legitimately combines both fields, since `nanos`' own
/// always-non-negative convention means simple addition (not `total_ns`
/// division-then-truncation) already gives the right signed result.
pub fn duration_component(
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i32,
    prop: &str,
) -> Option<i64> {
    let nanos = nanos as i64;
    Some(match prop {
        "years" => months / 12,
        "quarters" => months / 3,
        "months" => months,
        "weeks" => days / 7,
        "days" => days,
        "hours" => seconds / 3600,
        "minutes" => seconds / 60,
        "seconds" => seconds,
        "milliseconds" => seconds * 1000 + nanos / 1_000_000,
        "microseconds" => seconds * 1_000_000 + nanos / 1_000,
        "nanoseconds" => seconds * NANOS_PER_SEC as i64 + nanos,
        "quartersOfYear" => (months % 12) / 3,
        "monthsOfQuarter" => (months % 12) % 3,
        "monthsOfYear" => months % 12,
        "daysOfWeek" => days % 7,
        "minutesOfHour" => (seconds / 60) % 60,
        "secondsOfMinute" => seconds % 60,
        "millisecondsOfSecond" => nanos / 1_000_000,
        "microsecondsOfSecond" => nanos / 1_000,
        "nanosecondsOfSecond" => nanos,
        _ => return None,
    })
}

/// Renders `(months, days, seconds, nanos)` as MarsDB's canonical
/// ISO-8601 duration text -- always in `PnYnMnDTnHnMn.fS` order (never
/// `W`, even though `duration({weeks: 1})` accepts it as an *input*
/// unit -- weeks fold into `days` during normalization and never come
/// back out, matching every `toString(duration(...))` example in the
/// TCK). Each component is a straight divmod of the sign-independent
/// whole -- a negative `months`/`days`/`seconds` prints its own `-`
/// (`P-6M-15D...`), not one shared sign prefix, matching the TCK's mixed-
/// sign examples exactly (see Temporal8's duration-subtraction table).
pub fn format_duration(months: i64, days: i64, seconds: i64, nanos: i32) -> String {
    if months == 0 && days == 0 && seconds == 0 && nanos == 0 {
        return "PT0S".to_string();
    }
    let mut out = String::from("P");
    let years = months / 12;
    let rem_months = months % 12;
    if years != 0 {
        out.push_str(&format!("{years}Y"));
    }
    if rem_months != 0 {
        out.push_str(&format!("{rem_months}M"));
    }
    if days != 0 {
        out.push_str(&format!("{days}D"));
    }
    let total_time_ns = seconds as i128 * NANOS_PER_SEC + nanos as i128;
    if total_time_ns != 0 {
        out.push('T');
        if total_time_ns >= 0 {
            let hours = seconds / 3600;
            let rem = seconds % 3600;
            let minutes = rem / 60;
            let secs = rem % 60;
            if hours != 0 {
                out.push_str(&format!("{hours}H"));
            }
            if minutes != 0 {
                out.push_str(&format!("{minutes}M"));
            }
            if secs != 0 || nanos != 0 {
                out.push_str(&format_seconds_fraction(secs, nanos));
                out.push('S');
            }
        } else {
            let total_ns = total_time_ns;
            let hours = (total_ns / 3_600_000_000_000) as i64;
            let rem_h = total_ns % 3_600_000_000_000;
            let minutes = (rem_h / 60_000_000_000) as i64;
            let rem_m = rem_h % 60_000_000_000;
            let secs = (rem_m / 1_000_000_000) as i64;
            let sub_nanos = (rem_m % 1_000_000_000) as i32;
            if hours != 0 {
                out.push_str(&format!("{hours}H"));
            }
            if minutes != 0 {
                out.push_str(&format!("{minutes}M"));
            }
            if secs != 0 || sub_nanos != 0 {
                out.push_str(&format_seconds_fraction(secs, sub_nanos));
                out.push('S');
            }
        }
    }
    out
}

/// `secs` and `nanos` (same sign, or one of them zero -- the
/// `PropertyValue::Duration` invariant) rendered as one signed decimal,
/// e.g. `(1, 999_000_000)` -> `"1.999"`, `(0, -500_000_000)` -> `"-0.5"`.
/// Trailing zero digits (but not a bare trailing `.`) are trimmed -- real
/// Cypher's `toString` never prints `10.100000000S`.
fn format_seconds_fraction(secs: i64, nanos: i32) -> String {
    if nanos == 0 {
        return secs.to_string();
    }
    let negative = secs < 0 || nanos < 0;
    let mut frac = format!("{:09}", nanos.unsigned_abs());
    while frac.ends_with('0') {
        frac.pop();
    }
    format!(
        "{}{}.{}",
        if negative { "-" } else { "" },
        secs.unsigned_abs(),
        frac
    )
}

/// Parses an ISO-8601 duration string (`P[nY][nM][nW][nD][T[nH][nM][nS]]`,
/// each `n` an optional-sign decimal) into raw `DurationFields`, then
/// normalizes the same way `duration({...})` does -- construction from
/// text and from a map are the same operation once the units are pulled
/// apart, see `normalize_duration`'s docs.
///
/// Deliberately does *not* handle the alternative "combined date-time"
/// duration representation (`P2012-02-02T14:37:21.545`, ISO-8601's other
/// duration syntax) -- a real gap (see the README), not a silent
/// misparse: that string doesn't match `P` followed by number+letter
/// pairs, so this returns `None`, the same "reject, don't guess" outcome
/// `parse_date` gives an unsupported date string form.
pub fn parse_duration(s: &str) -> Option<DurationParts> {
    let s = s.trim();
    let s = s.strip_prefix('P')?;
    let (date_part, time_part) = match s.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };
    if let Some(fields) = parse_combined_date_time_duration(date_part, time_part) {
        return Some(normalize_duration(fields));
    }
    let date_pairs = scan_number_unit_pairs(date_part)?;
    let time_pairs = match time_part {
        Some(part) => scan_number_unit_pairs(part)?,
        None => Vec::new(),
    };
    if date_pairs.is_empty() && time_pairs.is_empty() {
        return None;
    }

    let mut f = DurationFields::default();
    for (value, unit) in date_pairs {
        match unit {
            'Y' => f.years = value,
            'M' => f.months = value,
            'W' => f.weeks = value,
            'D' => f.days = value,
            _ => return None,
        }
    }
    for (value, unit) in time_pairs {
        match unit {
            'H' => f.hours = value,
            'M' => f.minutes = value,
            'S' => f.seconds = value,
            _ => return None,
        }
    }
    Some(normalize_duration(f))
}

/// ISO-8601's alternate "combined date-time" duration representation
/// (`P<date>T<time>`, e.g. `P2012-02-02T14:37:21.545` -- date/time
/// formatted exactly like a calendar date/time-of-day, but each field
/// means "this many years/months/days/hours/minutes/seconds", not an
/// actual calendar date -- no day-of-month validity check, `P2012-13-40`
/// is a legal 12-year-13-month-40-day duration under this form. TCK's
/// Temporal2 `[7]`. Only matches when `date_part` genuinely has this
/// shape (plain `N-N-N`, no unit letters) -- an ordinary `PnYnMnD`
/// string never does, and a negative duration's leading `-` makes the
/// first split empty rather than a valid number, so neither can be
/// mistaken for this form.
fn parse_combined_date_time_duration(
    date_part: &str,
    time_part: Option<&str>,
) -> Option<DurationFields> {
    let mut date_fields = date_part.splitn(3, '-');
    let years: f64 = date_fields.next()?.parse().ok()?;
    let months: f64 = date_fields.next()?.parse().ok()?;
    let days: f64 = date_fields.next()?.parse().ok()?;
    if date_fields.next().is_some() {
        return None;
    }
    let mut f = DurationFields {
        years,
        months,
        days,
        ..Default::default()
    };
    if let Some(time_part) = time_part {
        let mut time_fields = time_part.splitn(3, ':');
        let hours: f64 = time_fields.next()?.parse().ok()?;
        let minutes: f64 = time_fields.next()?.parse().ok()?;
        let seconds: f64 = time_fields.next()?.parse().ok()?;
        if time_fields.next().is_some() {
            return None;
        }
        f.hours = hours;
        f.minutes = minutes;
        f.seconds = seconds;
    }
    Some(f)
}

/// Hand-scans `"12Y5M1.5D"`-style text into `(value, unit_letter)` pairs
/// -- no regex dependency for a grammar this small (a sign, digits, an
/// optional `.digits`, then exactly one unit letter), matching this
/// codebase's other hand-rolled small parsers (e.g. `marsdb-tck`'s
/// `CellParser`). The entire input must match: returning a successfully
/// parsed prefix would make malformed text such as `P1Ygarbage` silently
/// construct a one-year duration.
fn scan_number_unit_pairs(s: &str) -> Option<Vec<(f64, char)>> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        if chars[i] == '-' || chars[i] == '+' {
            i += 1;
        }
        let digits_start = i;
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
            i += 1;
        }
        if i == digits_start {
            return None;
        }
        let &unit = chars.get(i)?;
        let value = chars[start..i]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .ok()?;
        out.push((value, unit));
        i += 1;
    }
    Some(out)
}

// ---------------------------------------------------------------------
// LocalTime / Time
// ---------------------------------------------------------------------

/// Builds a `LocalTime`'s nanos-of-day from calendar-style fields
/// (`localtime({hour, minute, second, nanosecond})`'s already-summed
/// sub-second `nanos`) -- range-checked the same way `date_from_map`
/// checks year/month/day, `None` for anything out of range.
pub fn local_time_nanos_from_fields(
    hour: i64,
    minute: i64,
    second: i64,
    nanos: i64,
) -> Option<i64> {
    if !(0..24).contains(&hour)
        || !(0..60).contains(&minute)
        || !(0..60).contains(&second)
        || !(0..1_000_000_000).contains(&nanos)
    {
        return None;
    }
    Some(hour * 3_600_000_000_000 + minute * 60_000_000_000 + second * 1_000_000_000 + nanos)
}

/// Parses `HH[:MM[:SS[.fraction]]]` or the compact `HHMM[SS[.fraction]]`/
/// `HH` forms into nanoseconds since midnight -- the same colon-vs-
/// compact dispatch `parse_date` uses for the calendar forms.
fn parse_time_of_day(s: &str) -> Option<i64> {
    if !s.is_ascii() || s.is_empty() {
        return None;
    }
    let (hour, minute, second, nanos) = if s.contains(':') {
        let mut parts = s.splitn(3, ':');
        let h: u32 = parts.next()?.parse().ok()?;
        let m: u32 = match parts.next() {
            Some(p) => p.parse().ok()?,
            None => 0,
        };
        let (sec, nanos) = match parts.next() {
            Some(p) => parse_seconds_fraction(p)?,
            None => (0, 0),
        };
        (h, m, sec, nanos)
    } else {
        match s.len() {
            2 => (s.parse().ok()?, 0, 0, 0),
            4 => (s[0..2].parse().ok()?, s[2..4].parse().ok()?, 0, 0),
            n if n > 4 => {
                let (sec, nanos) = parse_seconds_fraction(&s[4..])?;
                (s[0..2].parse().ok()?, s[2..4].parse().ok()?, sec, nanos)
            }
            _ => return None,
        }
    };
    local_time_nanos_from_fields(hour as i64, minute as i64, second as i64, nanos as i64)
}

/// `"32.142"` / `"32"` -> `(seconds, nanos)`. The whole-number part must
/// be exactly 2 digits when called from the compact (no-`:`) form's
/// tail, but this function itself doesn't enforce that -- `parse_time_of_day`
/// slices the fixed-width prefix before calling it.
fn parse_seconds_fraction(s: &str) -> Option<(u32, u32)> {
    let (sec_str, frac_str) = match s.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let sec: u32 = sec_str.parse().ok()?;
    if sec >= 60 {
        return None;
    }
    let nanos = match frac_str {
        None => 0,
        Some(f) => {
            if f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let mut digits = f.to_string();
            digits.truncate(9);
            while digits.len() < 9 {
                digits.push('0');
            }
            digits.parse().ok()?
        }
    };
    Some((sec, nanos))
}

/// Splits a time-of-day-with-offset string into `(time_part,
/// offset_part)` -- the offset marker is a trailing `Z` or the first
/// `+`/`-` at index >= 1 (a bare time-of-day's own components are
/// digits/`:`/`.` only, so that's always the offset sign, never
/// something inside the time itself). Only ever called on the *time*
/// half of a combined date+time string (after splitting on `T`), never
/// the date half, which legitimately contains `-`.
fn split_time_offset(s: &str) -> (&str, Option<&str>) {
    if let Some(stripped) = s.strip_suffix('Z') {
        return (stripped, Some("Z"));
    }
    let bytes = s.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == b'+' || bytes[i] == b'-' {
            return (&s[..i], Some(&s[i..]));
        }
    }
    (s, None)
}

/// `Z` or `[+-]HH[:MM[:SS]]` / compact `[+-]HHMM[SS]` -> whole seconds
/// east of UTC.
pub fn parse_offset_seconds(s: &str) -> Option<i32> {
    if s == "Z" {
        return Some(0);
    }
    let bytes = s.as_bytes();
    let sign: i32 = match bytes.first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &s[1..];
    let (h, m, sec): (i32, i32, i32) = if rest.contains(':') {
        let mut parts = rest.splitn(3, ':');
        let h = parts.next()?.parse().ok()?;
        let m = match parts.next() {
            Some(p) => p.parse().ok()?,
            None => 0,
        };
        let sec = match parts.next() {
            Some(p) => p.parse().ok()?,
            None => 0,
        };
        (h, m, sec)
    } else {
        match rest.len() {
            2 => (rest.parse().ok()?, 0, 0),
            4 => (rest[0..2].parse().ok()?, rest[2..4].parse().ok()?, 0),
            6 => (
                rest[0..2].parse().ok()?,
                rest[2..4].parse().ok()?,
                rest[4..6].parse().ok()?,
            ),
            _ => return None,
        }
    };
    if !(0..24).contains(&h) || !(0..60).contains(&m) || !(0..60).contains(&sec) {
        return None;
    }
    Some(sign * (h * 3600 + m * 60 + sec))
}

/// `localtime('21:40:32.142')` -- a bare time-of-day, no offset allowed
/// (a trailing `Z`/`+HH:MM` makes the whole string fail the strict
/// digit/`:`/`.`-only parse above and correctly return `None`, the same
/// "reject, don't guess" stance as every other malformed-input case in
/// this module).
pub fn parse_local_time(s: &str) -> Option<i64> {
    parse_time_of_day(s.trim())
}

/// `time('21:40:32.142+01:00')` -- a time-of-day *with* a required
/// offset. Returns `None` if the string has no offset at all, or if it
/// carries a bracketed named-zone suffix (`[Europe/Stockholm]`) -- the
/// caller (`Executor::call_builtin`'s `"time"` arm) checks for `[`
/// itself first and raises a specific "named zones aren't supported"
/// error rather than this generic parse failure, but this function
/// still refuses to silently ignore/misparse the bracket if called
/// directly.
pub fn parse_time(s: &str) -> Option<(i64, i32)> {
    let s = s.trim();
    if s.contains('[') {
        return None;
    }
    let (time_part, offset_part) = split_time_offset(s);
    // A missing offset defaults to UTC (`+00:00`) -- real Cypher's
    // `time()` falls back to the statement's default time zone rather
    // than rejecting the string outright (TCK's Temporal10: `time('14:30')`
    // is a valid, offset-less argument).
    let offset_seconds = match offset_part {
        Some(part) => parse_offset_seconds(part)?,
        None => 0,
    };
    Some((parse_time_of_day(time_part)?, offset_seconds))
}

/// `d.<prop>` component access shared by `LocalTime` and (for its own
/// wall-clock time-of-day fields) `Time`/`LocalDateTime`/`DateTime`.
pub fn local_time_component(nanos_of_day: i64, prop: &str) -> Option<i64> {
    Some(match prop {
        "hour" => nanos_of_day / 3_600_000_000_000,
        "minute" => (nanos_of_day / 60_000_000_000) % 60,
        "second" => (nanos_of_day / 1_000_000_000) % 60,
        "millisecond" => (nanos_of_day / 1_000_000) % 1000,
        "microsecond" => (nanos_of_day / 1_000) % 1_000_000,
        "nanosecond" => nanos_of_day % 1_000_000_000,
        _ => return None,
    })
}

/// Formats an offset as Cypher's canonical text: `Z` for UTC, else
/// `[+-]HH:MM` (extended with `:SS` only when the offset has a non-zero
/// seconds component -- real offsets are almost always whole minutes,
/// but the TCK's timezone grep found at least one `-02:05:07` example).
pub fn format_offset(offset_seconds: i32) -> String {
    if offset_seconds == 0 {
        return "Z".to_string();
    }
    let sign = if offset_seconds < 0 { "-" } else { "+" };
    let abs = offset_seconds.unsigned_abs();
    let h = abs / 3600;
    let m = (abs / 60) % 60;
    let sec = abs % 60;
    if sec != 0 {
        format!("{sign}{h:02}:{m:02}:{sec:02}")
    } else {
        format!("{sign}{h:02}:{m:02}")
    }
}

/// `HH:MM` always; `:SS` only if seconds/nanos are non-zero; `.fraction`
/// only if nanos is non-zero (trailing zeros trimmed) -- matches every
/// `toString(localtime(...))`/`toString(time(...))` example in the TCK,
/// where `'21:40'` (no seconds given) prints without `:00`, but
/// `'21:40:32'` (seconds given, even if it were `:00`... though no TCK
/// example actually exercises that edge) prints with it.
fn format_time_of_day(nanos_of_day: i64) -> String {
    let hour = nanos_of_day / 3_600_000_000_000;
    let minute = (nanos_of_day / 60_000_000_000) % 60;
    let second = (nanos_of_day / 1_000_000_000) % 60;
    let nanos = (nanos_of_day % 1_000_000_000) as u32;
    let mut out = format!("{hour:02}:{minute:02}");
    if second != 0 || nanos != 0 {
        out.push_str(&format!(":{second:02}"));
        if nanos != 0 {
            let mut frac = format!("{nanos:09}");
            while frac.ends_with('0') {
                frac.pop();
            }
            out.push('.');
            out.push_str(&frac);
        }
    }
    out
}

pub fn format_local_time(nanos_of_day: i64) -> String {
    format_time_of_day(nanos_of_day)
}

pub fn format_time(nanos_of_day: i64, offset_seconds: i32) -> String {
    format!(
        "{}{}",
        format_time_of_day(nanos_of_day),
        format_offset(offset_seconds)
    )
}

// ---------------------------------------------------------------------
// LocalDateTime / DateTime
// ---------------------------------------------------------------------

/// Decomposes total (possibly negative) `epoch_seconds` into an
/// `(epoch_day, nanos_of_day)` pair -- `div_euclid`/`rem_euclid`, not
/// plain `/`/`%`, so a pre-1970 instant (negative `epoch_seconds`)
/// still gets a `nanos_of_day` in `0..NANOS_PER_DAY` (Rust's `%` on a
/// negative dividend returns a negative remainder, which would put the
/// "same calendar day" one day off).
pub fn split_epoch_seconds(epoch_seconds: i64) -> (i32, i64) {
    let epoch_day = epoch_seconds.div_euclid(SECONDS_PER_DAY) as i32;
    let secs_of_day = epoch_seconds.rem_euclid(SECONDS_PER_DAY);
    (epoch_day, secs_of_day * 1_000_000_000)
}

pub fn combine_epoch_day_and_nanos_of_day(epoch_day: i32, nanos_of_day: i64) -> i64 {
    epoch_day as i64 * SECONDS_PER_DAY + nanos_of_day / 1_000_000_000
}

/// Combines an `(epoch_day, nanos_of_day)` pair into `LocalDateTime`'s
/// own `(epoch_seconds, nanos)` storage shape -- shared by `<type>.
/// truncate()`'s date+time recombination step.
pub fn combine_date_and_time(epoch_day: i32, nanos_of_day: i64) -> (i64, i32) {
    (
        combine_epoch_day_and_nanos_of_day(epoch_day, nanos_of_day),
        (nanos_of_day % 1_000_000_000) as i32,
    )
}

/// Calendar + time-of-day fields for `localdatetime({...})`/
/// `datetime({...})`'s map constructors -- bundled into one struct (not
/// 7 positional args) purely to stay under clippy's argument-count cap,
/// matching this codebase's established convention for that lint (see
/// e.g. `executor.rs`'s `VarExpandSpec`/`IndexSeekSpec`).
pub struct CalendarDateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
    pub nanos: i64,
}

/// Builds a naive (zone-less) `(epoch_seconds, nanos)` instant from
/// calendar + time-of-day fields -- shared by `localdatetime({...})`'s
/// map form and (before the UTC offset adjustment) `datetime({...})`'s.
pub fn local_date_time_from_fields(f: CalendarDateTime) -> Option<(i64, i32)> {
    let epoch_day = epoch_day_from_ymd(f.year, f.month, f.day)?;
    let nanos_of_day = local_time_nanos_from_fields(f.hour, f.minute, f.second, f.nanos)?;
    Some((
        combine_epoch_day_and_nanos_of_day(epoch_day, nanos_of_day),
        (nanos_of_day % 1_000_000_000) as i32,
    ))
}

/// Same as `local_date_time_from_fields`, but the wall-clock reading is
/// in the given zone -- for a fixed `Offset`, subtracts it to get the
/// UTC instant `DateTime` actually stores (see its doc comment); for a
/// `Named` zone, resolves the real, DST-aware offset for *this specific*
/// local date-time via `chrono-tz` (the same zone can mean a different
/// offset on a different date, which is why this needs the full
/// calendar context `resolve_offset` alone doesn't have).
pub fn date_time_from_fields(f: CalendarDateTime, zone: &TzId) -> Option<(i64, i32)> {
    match zone {
        TzId::Offset(offset_seconds) => {
            let (local_epoch_seconds, nanos) = local_date_time_from_fields(f)?;
            Some((local_epoch_seconds - *offset_seconds as i64, nanos))
        }
        TzId::Named(name) => {
            let tz = parse_timezone_name(name)?;
            let epoch_day = epoch_day_from_ymd(f.year, f.month, f.day)?;
            let nanos_of_day = local_time_nanos_from_fields(f.hour, f.minute, f.second, f.nanos)?;
            let naive = naive_datetime_from(epoch_day, nanos_of_day);
            let (epoch_seconds, _offset) = utc_from_local_and_named_zone(naive, tz)?;
            Some((epoch_seconds, (nanos_of_day % 1_000_000_000) as i32))
        }
    }
}

/// Parses `YYYY-MM-DDTHH:MM:SS.fff` (and the compact/date-only-precision
/// variants `parse_date` already supports for the date half) into a
/// naive `(epoch_seconds, nanos)` instant.
pub fn parse_local_date_time(s: &str) -> Option<(i64, i32)> {
    let s = s.trim();
    let (date_part, time_part) = s.split_once('T')?;
    let epoch_day = parse_date(date_part)?;
    let nanos_of_day = parse_time_of_day(time_part)?;
    Some((
        combine_epoch_day_and_nanos_of_day(epoch_day, nanos_of_day),
        (nanos_of_day % 1_000_000_000) as i32,
    ))
}

/// Same date+time parse as `parse_local_date_time`, plus a required
/// zone on the time half -- either a fixed offset (`+01:00`), a
/// bracketed named zone with no explicit offset (`[Europe/London]`, the
/// true offset derived from the zone for *this* local date-time, TCK's
/// Temporal2 [6]), or both together (`+02:00[Europe/Stockholm]`, the
/// explicit offset is trusted for the instant and the bracket is kept
/// only for `TzId::Named`'s round-trip display).
pub fn parse_date_time(s: &str) -> Option<(i64, i32, TzId)> {
    let s = s.trim();
    let (date_part, time_part) = s.split_once('T')?;
    let epoch_day = parse_date(date_part)?;
    let (time_part, zone_name) = match time_part.split_once('[') {
        Some((t, rest)) => (t, Some(rest.strip_suffix(']')?)),
        None => (time_part, None),
    };
    let (time_only, offset_part) = split_time_offset(time_part);
    let nanos_of_day = parse_time_of_day(time_only)?;
    match (offset_part, zone_name) {
        (Some(offset_str), zone_name) => {
            let offset_seconds = parse_offset_seconds(offset_str)?;
            let local_epoch_seconds = combine_epoch_day_and_nanos_of_day(epoch_day, nanos_of_day);
            let zone = match zone_name {
                Some(zone_str) => {
                    parse_timezone_name(zone_str)?;
                    TzId::Named(zone_str.to_string())
                }
                None => TzId::Offset(offset_seconds),
            };
            Some((
                local_epoch_seconds - offset_seconds as i64,
                (nanos_of_day % 1_000_000_000) as i32,
                zone,
            ))
        }
        (None, Some(zone_str)) => {
            let tz = parse_timezone_name(zone_str)?;
            let naive = naive_datetime_from(epoch_day, nanos_of_day);
            let (epoch_seconds, _offset) = utc_from_local_and_named_zone(naive, tz)?;
            Some((
                epoch_seconds,
                (nanos_of_day % 1_000_000_000) as i32,
                TzId::Named(zone_str.to_string()),
            ))
        }
        (None, None) => None,
    }
}

/// `d.<prop>` component access for `LocalDateTime`/`DateTime`'s
/// *calendar* fields (`year`, `month`, ..., `dayOfQuarter`) -- delegates
/// straight to `date_component` on the instant's calendar day, since
/// the calendar math is identical to `Date`'s.
pub fn date_time_calendar_component(epoch_seconds: i64, prop: &str) -> Option<i64> {
    let (epoch_day, _) = split_epoch_seconds(epoch_seconds);
    date_component(epoch_day, prop)
}

/// `d.<prop>` component access for `LocalDateTime`/`DateTime`'s
/// *time-of-day* fields (`hour`, ..., `nanosecond`) -- delegates to
/// `local_time_component` on the instant's nanos-of-day, folding in the
/// caller-supplied sub-second `nanos` remainder (`epoch_seconds` alone
/// only has whole-second precision).
pub fn date_time_clock_component(epoch_seconds: i64, nanos: i32, prop: &str) -> Option<i64> {
    let (_, nanos_of_day) = split_epoch_seconds(epoch_seconds);
    local_time_component(nanos_of_day + nanos as i64, prop)
}

pub fn epoch_seconds_and_millis(epoch_seconds: i64, nanos: i32) -> (i64, i64) {
    (
        epoch_seconds,
        epoch_seconds * 1000 + (nanos as i64) / 1_000_000,
    )
}

/// `YYYY-MM-DDTHH:MM[:SS[.fraction]]` -- date half via `format_date`,
/// time half via the same `format_time_of_day` rule `LocalTime`/`Time`
/// use (seconds/fraction only shown when non-zero).
pub fn format_local_date_time(epoch_seconds: i64, nanos: i32) -> String {
    let (epoch_day, nanos_of_day) = split_epoch_seconds(epoch_seconds);
    format!(
        "{}T{}",
        format_date(epoch_day),
        format_time_of_day(nanos_of_day + nanos as i64)
    )
}

/// `Time`/`LocalTime` + `Duration` -- wraps at the 24h boundary (`Time`/
/// `LocalTime` have no calendar, so there's no "next day" to carry
/// into). Real Cypher truncates a Duration's calendar components
/// (`months`/`days`) when adding it to a time-only value -- only
/// `seconds`/`nanos` apply -- rather than erroring, so this never fails
/// (`Option` elsewhere in this module means "can overflow"; wrapping
/// never can).
pub fn add_duration_to_time(nanos_of_day: i64, seconds: i64, nanos: i32, negate: bool) -> i64 {
    let (seconds, nanos) = if negate {
        (-seconds, -nanos)
    } else {
        (seconds, nanos)
    };
    let total: i128 = nanos_of_day as i128 + seconds as i128 * NANOS_PER_SEC + nanos as i128;
    total.rem_euclid(NANOS_PER_DAY as i128) as i64
}

const NANOS_PER_DAY: i64 = SECONDS_PER_DAY * 1_000_000_000;

/// `LocalDateTime`/`DateTime` + `Duration` -- real calendar month
/// arithmetic on the date part (same `checked_add_months`/
/// `checked_sub_months` clamping as `add_duration_to_date`), then
/// `days`/`seconds`/`nanos` added as one exact nanosecond count that
/// carries across day boundaries (unlike `Date`, which has no time-of-
/// day to carry *into* -- a `LocalDateTime`/`DateTime` does, so nothing
/// here gets truncated the way `add_duration_to_date`'s `seconds`/
/// `nanos` do). Operates on the *local* wall-clock reading -- `DateTime`
/// callers pass `epoch_seconds + offset_seconds` in and subtract
/// `offset_seconds` back out of the result, so month/day arithmetic
/// happens against the calendar the user actually wrote, not the UTC
/// instant (matches real Cypher: `datetime({..., timezone: '+05:00'})
/// + duration({months: 1})` advances the *local* month).
pub fn add_duration_to_local_date_time(
    epoch_seconds: i64,
    existing_nanos: i32,
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i32,
    negate: bool,
) -> Option<(i64, i32)> {
    let (months, days, seconds, nanos) = if negate {
        (
            months.checked_neg()?,
            days.checked_neg()?,
            seconds.checked_neg()?,
            nanos.checked_neg()?,
        )
    } else {
        (months, days, seconds, nanos)
    };
    let (epoch_day, nanos_of_day) = split_epoch_seconds(epoch_seconds);
    let d = date_from_epoch_day(epoch_day);
    let with_months = if months >= 0 {
        d.checked_add_months(chrono::Months::new(months.try_into().ok()?))?
    } else {
        d.checked_sub_months(chrono::Months::new(months.checked_neg()?.try_into().ok()?))?
    };
    let new_epoch_day = with_months.signed_duration_since(epoch()).num_days();

    let total_ns: i128 = nanos_of_day as i128
        + existing_nanos as i128
        + days as i128 * NANOS_PER_DAY as i128
        + seconds as i128 * NANOS_PER_SEC
        + nanos as i128;
    let day_ns = NANOS_PER_DAY as i128;
    let extra_days = total_ns.div_euclid(day_ns) as i64;
    let final_nanos_of_day = total_ns.rem_euclid(day_ns) as i64;

    let final_epoch_day = new_epoch_day.checked_add(extra_days)?;
    let final_epoch_seconds = final_epoch_day
        .checked_mul(SECONDS_PER_DAY)?
        .checked_add(final_nanos_of_day / 1_000_000_000)?;
    Some((
        final_epoch_seconds,
        (final_nanos_of_day % 1_000_000_000) as i32,
    ))
}

pub fn format_date_time(epoch_seconds: i64, nanos: i32, zone: &TzId) -> String {
    // The *displayed* wall-clock reading is the local (offset-adjusted)
    // one, not the stored UTC instant -- `DateTime` round-trips through
    // `toString`/reparse showing the original offset's time-of-day, per
    // the TCK's own examples (e.g. `datetime({..., timezone: '+01:00'})`
    // prints that same `+01:00` wall-clock hour back, not the UTC one).
    let offset_seconds = resolve_offset(zone, epoch_seconds);
    let local_epoch_seconds = epoch_seconds + offset_seconds as i64;
    let zone_suffix = match zone {
        TzId::Offset(_) => String::new(),
        // Real Cypher's `toString()` round-trips the zone name alongside
        // its resolved offset (`+02:00[Europe/Stockholm]`), not just the
        // offset alone -- TCK's Temporal1 [10].
        TzId::Named(name) => format!("[{name}]"),
    };
    format!(
        "{}{}{}",
        format_local_date_time(local_epoch_seconds, nanos),
        format_offset(offset_seconds),
        zone_suffix
    )
}

/// Resolves a `TzId`'s real UTC offset (seconds east of UTC) at a given
/// UTC instant -- `Offset`'s value directly, or a `Named` zone's real,
/// DST-aware offset via `chrono-tz`'s embedded IANA database (the same
/// zone name resolves to a *different* offset depending on which instant
/// this is called with -- there's no single fixed "the" offset for a
/// named zone, e.g. TCK's Temporal1 [10] resolves `Europe/Stockholm` to
/// `+01:00` in October and `+02:00` in July). Falls back to UTC (`0`)
/// for a zone name that fails to parse -- should never happen for a
/// value MarsDB itself constructed (every `Named` zone is validated via
/// `parse_timezone_name` before being stored), but this function can't
/// return an error, so degrade gracefully rather than panic on a
/// hypothetical corrupt/foreign-written value.
pub fn resolve_offset(zone: &TzId, epoch_seconds: i64) -> i32 {
    match zone {
        TzId::Offset(o) => *o,
        TzId::Named(name) => {
            let tz = parse_timezone_name(name).unwrap_or(chrono_tz::Tz::UTC);
            let utc = chrono::DateTime::<chrono::Utc>::from_timestamp(epoch_seconds, 0)
                .unwrap_or_default();
            utc.with_timezone(&tz).offset().fix().local_minus_utc()
        }
    }
}

/// Parses an IANA timezone name (`'Europe/Stockholm'`) -- `None` if `s`
/// isn't a zone `chrono-tz`'s embedded database recognizes.
pub fn parse_timezone_name(s: &str) -> Option<chrono_tz::Tz> {
    s.parse().ok()
}

/// Given a *local* (wall-clock) naive date-time and a named zone,
/// resolves the true UTC `(epoch_seconds, offset_seconds)` -- the
/// overwhelming common case is `LocalResult::Single`; a DST fall-back
/// repeated hour (`Ambiguous`) takes the earlier instant, a DST
/// spring-forward gap (`None`, the local time never occurred) has no
/// valid mapping and fails -- real Cypher doesn't define a specific
/// tie-break for either, and no TCK scenario lands in one.
fn utc_from_local_and_named_zone(naive: NaiveDateTime, tz: chrono_tz::Tz) -> Option<(i64, i32)> {
    let dt = match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(earlier, _later) => earlier,
        LocalResult::None => return None,
    };
    let offset = dt.offset().fix().local_minus_utc();
    Some((dt.timestamp(), offset))
}

// ---------------------------------------------------------------------
// duration.between / .inMonths / .inDays / .inSeconds
// ---------------------------------------------------------------------

const NANOS_PER_DAY_I64: i64 = SECONDS_PER_DAY * 1_000_000_000;

fn naive_datetime_from(epoch_day: i32, nanos_of_day: i64) -> NaiveDateTime {
    let secs = (nanos_of_day / 1_000_000_000) as u32;
    let nanos = (nanos_of_day % 1_000_000_000) as u32;
    NaiveDateTime::new(
        date_from_epoch_day(epoch_day),
        NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .expect("nanos_of_day is always in 0..NANOS_PER_DAY by construction"),
    )
}

/// `java.time`'s `LocalDate`-difference-in-whole-months primitive
/// (`ChronoUnit.MONTHS.between`, which Neo4j's own `duration.between`
/// mirrors exactly): pack each date into a single sortable
/// `proleptic_month * 32 + day_of_month` value (32 safely exceeds any
/// month's real day count) so one integer division gives the exact
/// whole-month count, day-of-month aware, without a real calendar walk.
fn proleptic_month(d: NaiveDate) -> i64 {
    d.year() as i64 * 12 + d.month() as i64 - 1
}

fn months_between_dates(a: NaiveDate, b: NaiveDate) -> i64 {
    let packed_a = proleptic_month(a) * 32 + a.day() as i64;
    let packed_b = proleptic_month(b) * 32 + b.day() as i64;
    (packed_b - packed_a) / 32
}

/// Adds `months` to `dt`'s *date* only (real calendar month arithmetic,
/// clamping to the shorter month's last day, same as
/// `add_duration_to_date`), keeping the time-of-day unchanged.
fn shift_months(dt: NaiveDateTime, months: i64) -> NaiveDateTime {
    let d = dt.date();
    let shifted = if months >= 0 {
        d.checked_add_months(chrono::Months::new(months as u32))
    } else {
        d.checked_sub_months(chrono::Months::new((-months) as u32))
    }
    .expect("TCK-scale month shifts stay well within NaiveDate's range");
    NaiveDateTime::new(shifted, dt.time())
}

/// Shared core of `duration.between`/`.inMonths`/`.inDays`/
/// `.inSeconds`: `(months, shifted_remaining_ns, raw_total_ns)`.
///
/// If *either* operand has no calendar date (`a_date`/`b_date` is
/// `None` -- a bare `LocalTime`/`Time`), both operands' dates are
/// disregarded entirely (not even treated as a shared reference day --
/// verified against the TCK's own `date(...)` vs `localtime(...)`
/// examples, which produce a plain small time-of-day difference, never
/// a huge multi-year value derived from the date side's real calendar
/// date) -- `months` is always `0` in that case, and both the "raw" and
/// "month-shifted" totals collapse to the same plain time-of-day delta.
///
/// Otherwise: `months` is the real calendar month count between the two
/// full date-times (`months_between_datetimes_offset_aware`); `shifted_remaining_ns`
/// is the exact elapsed time between `from` *shifted forward by that
/// many months* and `to` (what `duration.between` bucket-splits into
/// days/seconds/nanos on top of `months` -- verified against the TCK to
/// NOT be a further calendar-date subtraction, just total elapsed time
/// re-divided by a day's worth of nanoseconds); `raw_total_ns` is the
/// plain, unshifted elapsed time between the two original instants
/// (what `.inDays`/`.inSeconds` use instead, discarding the month
/// optimization entirely -- confirmed by the TCK: `.inDays` on a
/// date+time target still reports a bare whole-day count with the
/// sub-day remainder silently truncated away, not carried as a
/// remaining `T...` component).
fn to_utc_instant_tz(dt: NaiveDateTime, zone: &TzId) -> NaiveDateTime {
    match zone {
        TzId::Offset(o) => dt - chrono::Duration::seconds(*o as i64),
        TzId::Named(name) => {
            if let Some(tz) = parse_timezone_name(name) {
                if let Some((epoch_seconds, _)) = utc_from_local_and_named_zone(dt, tz) {
                    return chrono::DateTime::<chrono::Utc>::from_timestamp(epoch_seconds, 0)
                        .map(|utc| utc.naive_utc())
                        .unwrap_or(dt);
                }
            }
            dt
        }
    }
}

fn elapsed_ns(
    from: NaiveDateTime,
    from_zone: Option<&TzId>,
    to: NaiveDateTime,
    to_zone: Option<&TzId>,
) -> i64 {
    let delta = match (from_zone, to_zone) {
        (Some(fz), Some(tz)) => to_utc_instant_tz(to, tz) - to_utc_instant_tz(from, fz),
        (Some(fz), None) => to_utc_instant_tz(to, fz) - to_utc_instant_tz(from, fz),
        (None, Some(tz)) => to_utc_instant_tz(to, tz) - to_utc_instant_tz(from, tz),
        (None, None) => to - from,
    };
    delta
        .num_nanoseconds()
        .expect("TCK-scale gaps stay well within i64 nanoseconds")
}

fn months_between_datetimes_offset_aware(
    from: NaiveDateTime,
    from_zone: Option<&TzId>,
    to: NaiveDateTime,
    to_zone: Option<&TzId>,
) -> i64 {
    let mut months = months_between_dates(from.date(), to.date());
    let shifted = shift_months(from, months);
    let overshot = match (from_zone, to_zone) {
        (Some(fz), Some(tz)) => to_utc_instant_tz(shifted, fz) > to_utc_instant_tz(to, tz),
        (Some(fz), None) => to_utc_instant_tz(shifted, fz) > to_utc_instant_tz(to, fz),
        (None, Some(tz)) => to_utc_instant_tz(shifted, tz) > to_utc_instant_tz(to, tz),
        (None, None) => shifted > to,
    };
    let undershot = match (from_zone, to_zone) {
        (Some(fz), Some(tz)) => to_utc_instant_tz(shifted, fz) < to_utc_instant_tz(to, tz),
        (Some(fz), None) => to_utc_instant_tz(shifted, fz) < to_utc_instant_tz(to, fz),
        (None, Some(tz)) => to_utc_instant_tz(shifted, tz) < to_utc_instant_tz(to, tz),
        (None, None) => shifted < to,
    };
    if months > 0 && overshot {
        months -= 1;
    } else if months < 0 && undershot {
        months += 1;
    }
    months
}

fn time_to_utc_nanos(nanos_of_day: i64, zone: &TzId, ref_date: Option<i32>) -> i64 {
    let epoch_day = ref_date.unwrap_or(0);
    let dt = naive_datetime_from(epoch_day, nanos_of_day);
    let utc = to_utc_instant_tz(dt, zone);
    (utc.signed_duration_since(epoch().and_hms_opt(0, 0, 0).unwrap()))
        .num_nanoseconds()
        .unwrap_or(0)
}

fn between_components(
    a_date: Option<i32>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i32>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> (i64, i64, i64) {
    match (a_date, b_date) {
        (Some(ad), Some(bd)) => {
            let from = naive_datetime_from(ad, a_time.unwrap_or(0));
            let to = naive_datetime_from(bd, b_time.unwrap_or(0));
            let months = months_between_datetimes_offset_aware(from, a_zone, to, b_zone);
            let shifted = shift_months(from, months);
            let shifted_remaining_ns = elapsed_ns(shifted, a_zone, to, b_zone);
            let raw_total_ns = elapsed_ns(from, a_zone, to, b_zone);
            (months, shifted_remaining_ns, raw_total_ns)
        }
        _ => {
            let diff = match (a_zone, b_zone) {
                (Some(az), Some(bz)) => {
                    // Both sides resolved against the *same* reference
                    // date -- "time-only mode" means the date each
                    // operand happens to carry is disregarded (see this
                    // function's module docs), so `a`/`b` must not each
                    // pull in their own, potentially wildly different,
                    // real date (that only cancels out in `bt - at` when
                    // it's identical on both sides; a real, previously-
                    // caught regression when this used `a_date`/`b_date`
                    // independently). Only matters for resolving a
                    // `Named` zone's DST-dependent offset -- a fixed
                    // `Offset` doesn't care what date it's given at all.
                    let ref_date = a_date.or(b_date);
                    let at = time_to_utc_nanos(a_time.unwrap_or(0), az, ref_date);
                    let bt = time_to_utc_nanos(b_time.unwrap_or(0), bz, ref_date);
                    bt - at
                }
                (Some(az), None) => {
                    let at = time_to_utc_nanos(a_time.unwrap_or(0), az, a_date);
                    let bt = time_to_utc_nanos(b_time.unwrap_or(0), az, a_date);
                    bt - at
                }
                (None, Some(bz)) => {
                    let at = time_to_utc_nanos(a_time.unwrap_or(0), bz, b_date);
                    let bt = time_to_utc_nanos(b_time.unwrap_or(0), bz, b_date);
                    bt - at
                }
                (None, None) => b_time.unwrap_or(0) - a_time.unwrap_or(0),
            };
            (0, diff, diff)
        }
    }
}

pub fn duration_between(
    a_date: Option<i32>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i32>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (months, shifted_ns, _) =
        between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    let days = shifted_ns / NANOS_PER_DAY_I64;
    let rem = shifted_ns % NANOS_PER_DAY_I64;
    let seconds = rem.div_euclid(NANOS_PER_SEC as i64);
    let nanos = rem.rem_euclid(NANOS_PER_SEC as i64) as i32;
    (months, days, seconds, nanos)
}

pub fn duration_in_months(
    a_date: Option<i32>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i32>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (months, _, _) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (months, 0, 0, 0)
}

pub fn duration_in_days(
    a_date: Option<i32>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i32>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (_, _, raw) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (0, raw / NANOS_PER_DAY_I64, 0, 0)
}

pub fn duration_in_seconds(
    a_date: Option<i32>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i32>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (_, _, raw) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (
        0,
        0,
        raw.div_euclid(NANOS_PER_SEC as i64),
        raw.rem_euclid(NANOS_PER_SEC as i64) as i32,
    )
}

// ---------------------------------------------------------------------
// <type>.truncate(unit, value, map)
// ---------------------------------------------------------------------

/// Truncates a calendar date down to the start of `unit` -- `None` for
/// any unit that isn't a calendar-scale one (`hour`/`minute`/... apply
/// to the *time* half, see `truncate_time_unit`). `millennium`/
/// `century`/`decade` floor the year to the nearest boundary below it
/// (`2017 -> 2000`, `1984 -> 1900`/`1980`) -- plain `year -
/// year.rem_euclid(N)`, correct for negative years too since
/// `rem_euclid` is always non-negative. `week`/`weekYear` use the same
/// ISO week-date `chrono` already computes for `.week`/`.weekYear`
/// component access (`date_component`) -- the Monday of that ISO week/
/// week-year.
pub fn truncate_date_unit(epoch_day: i32, unit: &str) -> Option<i32> {
    let d = date_from_epoch_day(epoch_day);
    let y = d.year();
    let to_epoch_day = |d: NaiveDate| d.signed_duration_since(epoch()).num_days() as i32;
    match unit {
        "millennium" => epoch_day_from_ymd(y - y.rem_euclid(1000), 1, 1),
        "century" => epoch_day_from_ymd(y - y.rem_euclid(100), 1, 1),
        "decade" => epoch_day_from_ymd(y - y.rem_euclid(10), 1, 1),
        "year" => epoch_day_from_ymd(y, 1, 1),
        "quarter" => epoch_day_from_ymd(y, (d.month() - 1) / 3 * 3 + 1, 1),
        "month" => epoch_day_from_ymd(y, d.month(), 1),
        "week" => {
            let iso = d.iso_week();
            NaiveDate::from_isoywd_opt(iso.year(), iso.week(), chrono::Weekday::Mon)
                .map(to_epoch_day)
        }
        "weekYear" => {
            let iso = d.iso_week();
            NaiveDate::from_isoywd_opt(iso.year(), 1, chrono::Weekday::Mon).map(to_epoch_day)
        }
        "day" => Some(epoch_day),
        _ => None,
    }
}

/// Moves `epoch_day` to the given ISO weekday (`1`=Monday..`7`=Sunday)
/// *within its own ISO week* -- the `dayOfWeek` override key on a
/// `.truncate('week', ...)` result (`date.truncate('week', d,
/// {dayOfWeek: 2})` is "the Tuesday of `d`'s week"), not general
/// week-date construction from a `{year, week, dayOfWeek}` triple with
/// no existing anchor date (that's `epoch_day_from_week_fields`).
/// `None` for an out-of-range `day_of_week`.
pub fn set_iso_weekday(epoch_day: i32, day_of_week: i64) -> Option<i32> {
    if !(1..=7).contains(&day_of_week) {
        return None;
    }
    let d = date_from_epoch_day(epoch_day);
    let iso = d.iso_week();
    let monday = NaiveDate::from_isoywd_opt(iso.year(), iso.week(), chrono::Weekday::Mon)?;
    let result = monday + chrono::Duration::days(day_of_week - 1);
    Some(result.signed_duration_since(epoch()).num_days() as i32)
}

/// Truncates a time-of-day down to the start of `unit` -- `None` for
/// any unit that isn't a clock-scale one. `day` truncates to midnight
/// (`0`), the shared boundary between the date and time halves.
pub fn truncate_time_unit(nanos_of_day: i64, unit: &str) -> Option<i64> {
    let floor = |n: i64| (nanos_of_day / n) * n;
    match unit {
        "hour" => Some(floor(3_600_000_000_000)),
        "minute" => Some(floor(60_000_000_000)),
        "second" => Some(floor(1_000_000_000)),
        "millisecond" => Some(floor(1_000_000)),
        "microsecond" => Some(floor(1_000)),
        "day" => Some(0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn du(months: f64, days: f64, hours: f64, minutes: f64, seconds: f64) -> DurationParts {
        normalize_duration(DurationFields {
            months,
            days,
            hours,
            minutes,
            seconds,
            ..Default::default()
        })
    }

    #[test]
    fn construct_basic() {
        assert_eq!(
            format_duration_parts(du(0.0, 14.0, 16.0, 12.0, 0.0)),
            "P14DT16H12M"
        );
    }

    #[test]
    fn construct_fractional_months() {
        let d = normalize_duration(DurationFields {
            months: 0.75,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "P22DT19H51M49.5S");
    }

    #[test]
    fn construct_fractional_weeks() {
        let d = normalize_duration(DurationFields {
            weeks: 2.5,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "P17DT12H");
    }

    #[test]
    fn construct_years_months_days_seconds_overflow() {
        let d = normalize_duration(DurationFields {
            years: 12.0,
            months: 5.0,
            days: 14.0,
            hours: 16.0,
            minutes: 12.0,
            seconds: 70.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "P12Y5M14DT16H13M10S");
    }

    #[test]
    fn construct_sub_second() {
        let d = normalize_duration(DurationFields {
            days: 14.0,
            seconds: 70.0,
            milliseconds: 1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "P14DT1M10.001S");
    }

    #[test]
    fn construct_minutes_fraction() {
        let d = normalize_duration(DurationFields {
            minutes: 1.5,
            seconds: 1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT1M31S");
    }

    #[test]
    fn parse_string_p14dt16h12m() {
        assert_eq!(
            format_duration_parts(parse_duration("P14DT16H12M").unwrap()),
            "P14DT16H12M"
        );
    }

    #[test]
    fn parse_string_p0_75m() {
        assert_eq!(
            format_duration_parts(parse_duration("P0.75M").unwrap()),
            "P22DT19H51M49.5S"
        );
    }

    #[test]
    fn parse_string_pt0_75m() {
        assert_eq!(
            format_duration_parts(parse_duration("PT0.75M").unwrap()),
            "PT45S"
        );
    }

    #[test]
    fn malformed_temporal_strings_are_rejected_without_panicking() {
        assert_eq!(parse_date("123é4"), None);
        for malformed in ["P", "PT", "Pgarbage", "P1Ygarbage", "P1Y2", "P1.2.3Y"] {
            assert_eq!(
                parse_duration(malformed),
                None,
                "{malformed} must be rejected"
            );
        }
    }

    #[test]
    fn add_durations() {
        let a = du(149.0, 14.0, 16.0, 12.0, 70.0);
        let a = (a.0, a.1, a.2, 1);
        let sum = add_duration(a, a).unwrap();
        assert_eq!(format_duration_parts(sum), "P24Y10M28DT32H26M20.000000002S");
    }

    #[test]
    fn scale_duration_by_half() {
        let base = (149, 14, 58390, 1);
        assert_eq!(
            format_duration_parts(scale_duration(base, 0.5)),
            "P6Y2M22DT13H21M8S"
        );
        assert_eq!(
            format_duration_parts(scale_duration(base, 2.0)),
            "P24Y10M28DT32H26M20.000000002S"
        );
    }

    #[test]
    fn negative_seconds_fraction() {
        let d = normalize_duration(DurationFields {
            seconds: 2.0,
            milliseconds: -1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT1.999S");
        let d = normalize_duration(DurationFields {
            seconds: -2.0,
            milliseconds: 1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT-1.999S");
        let d = normalize_duration(DurationFields {
            seconds: -2.0,
            milliseconds: -1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT-2.001S");
        let d = normalize_duration(DurationFields {
            seconds: 60.0,
            milliseconds: -1.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT59.999S");
        let d = normalize_duration(DurationFields {
            minutes: 12.0,
            seconds: -60.0,
            ..Default::default()
        });
        assert_eq!(format_duration_parts(d), "PT11M");
    }

    #[test]
    fn date_roundtrip() {
        let d = epoch_day_from_ymd(1984, 10, 11).unwrap();
        assert_eq!(format_date(d), "1984-10-11");
        assert_eq!(parse_date("1984-10-11"), Some(d));
        assert_eq!(parse_date("19841011"), Some(d));
    }

    #[test]
    fn date_components() {
        let d = epoch_day_from_ymd(1984, 10, 11).unwrap();
        assert_eq!(date_component(d, "year"), Some(1984));
        assert_eq!(date_component(d, "quarter"), Some(4));
        assert_eq!(date_component(d, "month"), Some(10));
        assert_eq!(date_component(d, "week"), Some(41));
        assert_eq!(date_component(d, "weekYear"), Some(1984));
        assert_eq!(date_component(d, "day"), Some(11));
        assert_eq!(date_component(d, "ordinalDay"), Some(285));
        assert_eq!(date_component(d, "weekDay"), Some(4));
        assert_eq!(date_component(d, "dayOfQuarter"), Some(11));
    }

    #[test]
    fn date_plus_duration() {
        let x = epoch_day_from_ymd(1984, 10, 11).unwrap();
        let d = du(149.0, 14.0, 16.0, 12.0, 70.0);
        let sum = add_duration_to_date(x, d.0, d.1, d.2, d.3, false).unwrap();
        assert_eq!(format_date(sum), "1997-03-25");
        let diff = add_duration_to_date(x, d.0, d.1, d.2, d.3, true).unwrap();
        assert_eq!(format_date(diff), "1972-04-27");
    }

    /// The fractional-duration case that exposed `add_duration_to_date`
    /// dropping `seconds`/`nanos` outright instead of folding whole extra
    /// days out of them -- see that function's doc comment.
    #[test]
    fn date_plus_fractional_duration_carries_extra_day_from_seconds() {
        let x = epoch_day_from_ymd(1984, 10, 11).unwrap();
        let d = normalize_duration(DurationFields {
            years: 12.5,
            months: 5.5,
            days: 14.5,
            hours: 16.5,
            minutes: 12.5,
            seconds: 70.5,
            nanoseconds: 3.0,
            ..Default::default()
        });
        let sum = add_duration_to_date(x, d.0, d.1, d.2, d.3, false).unwrap();
        assert_eq!(format_date(sum), "1997-10-11");
        let diff = add_duration_to_date(x, d.0, d.1, d.2, d.3, true).unwrap();
        assert_eq!(format_date(diff), "1971-10-12");
    }

    #[test]
    fn duration_accessors() {
        let d = normalize_duration(DurationFields {
            years: 1.0,
            months: 4.0,
            days: 10.0,
            hours: 1.0,
            minutes: 1.0,
            seconds: 1.0,
            nanoseconds: 111_111_111.0,
            ..Default::default()
        });
        let get = |prop: &str| duration_component(d.0, d.1, d.2, d.3, prop).unwrap();
        assert_eq!(get("years"), 1);
        assert_eq!(get("quarters"), 5);
        assert_eq!(get("months"), 16);
        assert_eq!(get("weeks"), 1);
        assert_eq!(get("days"), 10);
        assert_eq!(get("hours"), 1);
        assert_eq!(get("minutes"), 61);
        assert_eq!(get("seconds"), 3661);
        assert_eq!(get("milliseconds"), 3_661_111);
        assert_eq!(get("microseconds"), 3_661_111_111);
        assert_eq!(get("nanoseconds"), 3_661_111_111_111);
        assert_eq!(get("quartersOfYear"), 1);
        assert_eq!(get("monthsOfQuarter"), 1);
        assert_eq!(get("monthsOfYear"), 4);
        assert_eq!(get("daysOfWeek"), 3);
        assert_eq!(get("minutesOfHour"), 1);
        assert_eq!(get("secondsOfMinute"), 1);
        assert_eq!(get("millisecondsOfSecond"), 111);
        assert_eq!(get("microsecondsOfSecond"), 111_111);
        assert_eq!(get("nanosecondsOfSecond"), 111_111_111);
    }

    fn format_duration_parts(p: DurationParts) -> String {
        format_duration(p.0, p.1, p.2, p.3)
    }
}
