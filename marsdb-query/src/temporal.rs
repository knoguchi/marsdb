//! Calendar math and ISO-8601 text conversion for `PropertyValue::Date`/
//! `PropertyValue::Duration`. Kept out of `marsdb-graph` (which stores the
//! value but doesn't know Cypher's construction/formatting rules) and out
//! of `executor.rs` (which dispatches to these, not the arithmetic
//! itself).
//!
//! `DATE` (calendar year/month/day, ISO week-date, and ordinal/quarter-date
//! construction forms), `DURATION`, `LOCAL TIME`, `TIME`, `LOCAL DATETIME`,
//! and `DATETIME` are all supported. `TIME`/`DATETIME` only accept a fixed
//! UTC offset (`'+01:00'`, `{timezone: '+01:00'}`), never a named timezone
//! (`'Europe/Stockholm'`) -- that needs a real IANA timezone database, out
//! of scope for this module. See the README's "Cypher coverage" section
//! for the exact gaps.

use chrono::{LocalResult, NaiveDateTime, Offset, TimeZone, Timelike};

/// A `DateTime`'s zone -- a plain mirror of `PropertyValue::DateTime`'s
/// `zone: marsdb_graph::model::TzId` field, independent of `marsdb_graph`
/// since this module doesn't depend on that crate; translated at the
/// `executor.rs` boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum TzId {
    Offset(i32),
    Named(String),
}

const SECONDS_PER_DAY: i64 = 86_400;

/// Average Gregorian month length in days (365.2425 / 12), Neo4j's
/// documented conversion factor for folding a fractional month (e.g. the
/// `0.75` in `duration({months: 0.75})`) down into days, since "0.75
/// months" has no exact length without a reference date. Only applied to
/// the fractional remainder of a month count -- a whole month stays a
/// whole month, applied to a `Date` via real calendar arithmetic in
/// `add_duration_to_date`.
const AVG_MONTH_DAYS: f64 = 365.2425 / 12.0;

const NANOS_PER_SEC: i128 = 1_000_000_000;

// ---------------------------------------------------------------------
// Proleptic-Gregorian civil-calendar core (Howard Hinnant's algorithms)
// ---------------------------------------------------------------------
// Pure i64 integer math, not chrono: chrono's `NaiveDate` caps years at
// ±262_143, far short of Cypher's ±999_999_999 (ISO 8601 expanded years).
// chrono remains only for `capture_now` and named-IANA-zone resolution,
// inherently bounded by chrono-tz's own range. Epoch-day origin is
// 1970-01-01, same as `std::time::UNIX_EPOCH`.

/// Cypher's documented year range (java.time's, which real Cypher
/// mirrors). Every constructor validates against it; epoch days for
/// this range (±365 billion) always fit i64 with room for nanosecond
/// totals in i128.
pub const MIN_YEAR: i64 = -999_999_999;
pub const MAX_YEAR: i64 = 999_999_999;

fn is_leap_year(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn last_day_of_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Epoch days for an already-validated civil y/m/d.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = y - (m <= 2) as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m as i64 + 9) % 12; // March=0 .. February=11
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Inverse of `days_from_civil`: `(year, month, day)` for an epoch day.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    (y + (m <= 2) as i64, m, d)
}

/// ISO weekday, 1=Monday..7=Sunday (1970-01-01 was a Thursday, 4).
fn iso_weekday_from_days(z: i64) -> i64 {
    (z + 3).rem_euclid(7) + 1
}

fn ordinal_day_of(y: i64, m: u32, d: u32) -> i64 {
    days_from_civil(y, m, d) - days_from_civil(y, 1, 1) + 1
}

fn days_in_year(y: i64) -> i64 {
    if is_leap_year(y) {
        366
    } else {
        365
    }
}

/// ISO 8601 week count for a week-numbering year: 53 iff Jan 1 falls on
/// a Thursday, or on a Wednesday of a leap year; else 52.
fn iso_weeks_in_year(y: i64) -> i64 {
    let jan1 = iso_weekday_from_days(days_from_civil(y, 1, 1));
    if jan1 == 4 || (is_leap_year(y) && jan1 == 3) {
        53
    } else {
        52
    }
}

/// `(iso_week_year, iso_week)` for an epoch day -- the week-numbering
/// year diverges from the calendar year near a year boundary.
fn iso_week_of(z: i64) -> (i64, i64) {
    let (y, m, d) = civil_from_days(z);
    let doy = ordinal_day_of(y, m, d);
    let wd = iso_weekday_from_days(z);
    let week = (doy - wd + 10) / 7;
    if week < 1 {
        (y - 1, iso_weeks_in_year(y - 1))
    } else if week > iso_weeks_in_year(y) {
        (y + 1, 1)
    } else {
        (y, week)
    }
}

/// The Monday of ISO week 1 of `week_year` -- January 4 is always in
/// week 1, so it anchors the calculation.
fn iso_week1_monday(week_year: i64) -> i64 {
    let jan4 = days_from_civil(week_year, 1, 4);
    jan4 - (iso_weekday_from_days(jan4) - 1)
}

/// Calendar month shift with end-of-month clamping (Jan 31 + 1 month =
/// Feb 28/29, not an error and not Mar 3) -- the same rule
/// `checked_add_months` had when this was chrono-backed.
fn add_months_to_epoch_day(z: i64, months: i64) -> Option<i64> {
    let (y, m, d) = civil_from_days(z);
    let total = y
        .checked_mul(12)?
        .checked_add(m as i64 - 1)?
        .checked_add(months)?;
    let ny = total.div_euclid(12);
    let nm = total.rem_euclid(12) as u32 + 1;
    if !(MIN_YEAR..=MAX_YEAR).contains(&ny) {
        return None;
    }
    let nd = d.min(last_day_of_month(ny, nm));
    Some(days_from_civil(ny, nm, nd))
}

pub fn epoch_day_from_ymd(year: i64, month: u32, day: u32) -> Option<i64> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > last_day_of_month(year, month) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

/// A single captured instant, pre-derived into every shape a no-arg
/// `date()`/`localtime()`/`time()`/`localdatetime()`/`datetime()` call
/// needs. Cypher guarantees every such call within the same query returns
/// the same value (`duration.between(date(), date())` is always `PT0S`),
/// which holds only because one `chrono::Utc::now()` capture derives
/// every field, rather than one `now()` call per field.
#[derive(Clone, Copy)]
pub struct NowSnapshot {
    pub epoch_day: i64,
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

pub fn format_date(epoch_day: i64) -> String {
    let (y, m, d) = civil_from_days(epoch_day);
    if (0..=9999).contains(&y) {
        format!("{y:04}-{m:02}-{d:02}")
    } else {
        // ISO 8601 expanded year: explicit sign outside 0000..=9999
        // (`+999999999-12-31`, `-999999999-01-01`) -- round-trips
        // through `parse_date`'s own sign handling.
        format!("{y:+}-{m:02}-{d:02}")
    }
}

/// Parses every date string form MarsDB supports: the plain calendar
/// forms `YYYY-MM-DD`/`YYYYMMDD`/`YYYY-MM`/`YYYYMM`/`YYYY` (missing
/// month/day default to `1`), ISO week-date `YYYY-Www[-D]`/`YYYYWww[D]`
/// (missing day defaults to `1`), ordinal-date `YYYY-DDD`/`YYYYDDD` (see
/// `parse_week_or_ordinal_date`), and ISO 8601 expanded years -- an
/// explicit leading sign with up to 9 year digits (`'-999999999-01-01'`,
/// `'+999999999-12-31'`). The sign is stripped here and applied to
/// whichever year field the body then parses.
pub fn parse_date(s: &str) -> Option<i64> {
    let s = s.trim();
    // The compact forms below use byte offsets because their grammar is
    // ASCII-only. Reject non-ASCII input before slicing so malformed user
    // input can never put an offset in the middle of a UTF-8 code point.
    if !s.is_ascii() {
        return None;
    }
    let (year_sign, s) = match s.as_bytes().first()? {
        b'+' => (1i64, &s[1..]),
        b'-' => (-1i64, &s[1..]),
        _ => (1, s),
    };
    // ISO week-date (`YYYY-Www[-D]` / `YYYYWww[D]`) and ordinal-date
    // (`YYYY-DDD` / `YYYYDDD`) forms -- checked before the plain calendar
    // forms below since a `W` unambiguously marks a week-date, and a
    // 7-digit no-`-` run is ordinal (a plain compact calendar date is
    // either 4, 6, or 8 digits, never 7).
    if let Some(epoch_day) = parse_week_or_ordinal_date(s, year_sign) {
        return Some(epoch_day);
    }
    let (year, month, day) = if let Some((y, rest)) = s.split_once('-') {
        let year: i64 = y.parse().ok()?;
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
    epoch_day_from_ymd(year_sign * year, month, day)
}

/// ISO week-date (`YYYY-Www[-D]` / `YYYYWww[D]`, day defaults to `1` when
/// omitted) and ordinal-date (`YYYY-DDD` / `YYYYDDD`) string forms --
/// `None` for anything not matching, falling through to `parse_date`'s
/// plain calendar parsing.
fn parse_week_or_ordinal_date(s: &str, year_sign: i64) -> Option<i64> {
    if let Some((y, rest)) = s.split_once('-') {
        if let Some(w) = rest.strip_prefix('W') {
            let week_year: i64 = y.parse().ok()?;
            let (week, day) = match w.split_once('-') {
                Some((w, d)) => (w.parse().ok()?, d.parse().ok()?),
                None => (w.parse().ok()?, 1),
            };
            return epoch_day_from_week_fields(year_sign * week_year, week, day);
        }
        // `YYYY-DDD` -- an ordinal date, distinguished from the plain
        // `YYYY-MM` calendar form by `rest`'s length (3 digits, not 2).
        if rest.len() == 3 && rest.bytes().all(|b| b.is_ascii_digit()) {
            let year: i64 = y.parse().ok()?;
            let ordinal: u32 = rest.parse().ok()?;
            return epoch_day_from_ordinal_fields(year_sign * year, ordinal);
        }
        return None;
    }
    if s.len() >= 5 {
        if let Some(w) = s[4..].strip_prefix('W') {
            let week_year: i64 = s[0..4].parse().ok()?;
            let (week, day) = match w.len() {
                2 => (w.parse().ok()?, 1),
                3 => (w[0..2].parse().ok()?, w[2..3].parse().ok()?),
                _ => return None,
            };
            return epoch_day_from_week_fields(year_sign * week_year, week, day);
        }
    }
    if s.len() == 7 && s.bytes().all(|b| b.is_ascii_digit()) {
        let year: i64 = s[0..4].parse().ok()?;
        let ordinal: u32 = s[4..7].parse().ok()?;
        return epoch_day_from_ordinal_fields(year_sign * year, ordinal);
    }
    None
}

/// `d.<prop>` component access for a `Date` -- the "forward" (date ->
/// components) half of ISO week/quarter calendar math; the "backward"
/// half lives in `epoch_day_from_week_fields`/`epoch_day_from_ordinal_
/// fields`/`epoch_day_from_quarter_fields` below. `None` for an
/// unrecognized property name, treated as a missing property by the
/// caller.
pub fn date_component(epoch_day: i64, prop: &str) -> Option<i64> {
    let (y, m, d) = civil_from_days(epoch_day);
    Some(match prop {
        "year" => y,
        "month" => m as i64,
        "day" => d as i64,
        "quarter" => ((m - 1) / 3 + 1) as i64,
        "ordinalDay" => ordinal_day_of(y, m, d),
        "weekDay" | "dayOfWeek" => iso_weekday_from_days(epoch_day),
        "week" => iso_week_of(epoch_day).1,
        "weekYear" => iso_week_of(epoch_day).0,
        "dayOfQuarter" => {
            let quarter_start_month = (m - 1) / 3 * 3 + 1;
            epoch_day - days_from_civil(y, quarter_start_month, 1) + 1
        }
        _ => return None,
    })
}

/// Constructs an epoch-day from ISO week-date fields -- the inverse of
/// `date_component`'s `"weekYear"`/`"week"`/`"dayOfWeek"` accessors.
/// `week_year` is the ISO week-numbering year, not necessarily the
/// calendar year of the resulting date -- they diverge near a year
/// boundary (e.g. week-year 1817 week 1 day 2 is calendar date
/// 1816-12-31).
pub fn epoch_day_from_week_fields(week_year: i64, week: u32, day_of_week: i64) -> Option<i64> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&week_year)
        || !(1..=7).contains(&day_of_week)
        || week < 1
        || week as i64 > iso_weeks_in_year(week_year)
    {
        return None;
    }
    Some(iso_week1_monday(week_year) + (week as i64 - 1) * 7 + (day_of_week - 1))
}

/// Constructs an epoch-day from a calendar year plus an ordinal day
/// (`1..=365`/`366`) -- the inverse of `date_component`'s `"ordinalDay"`.
pub fn epoch_day_from_ordinal_fields(year: i64, ordinal_day: u32) -> Option<i64> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&year)
        || ordinal_day < 1
        || ordinal_day as i64 > days_in_year(year)
    {
        return None;
    }
    Some(days_from_civil(year, 1, 1) + ordinal_day as i64 - 1)
}

/// Constructs an epoch-day from a calendar year, quarter (`1..=4`), and
/// day-of-quarter (`1`-based) -- the inverse of `date_component`'s
/// `"quarter"`/`"dayOfQuarter"`.
pub fn epoch_day_from_quarter_fields(year: i64, quarter: u32, day_of_quarter: i64) -> Option<i64> {
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) || !(1..=4).contains(&quarter) {
        return None;
    }
    let quarter_start_month = (quarter - 1) * 3 + 1;
    Some(days_from_civil(year, quarter_start_month, 1) + (day_of_quarter - 1))
}

/// Adds a `Duration` to a `Date` via real calendar month arithmetic
/// (clamping to the shorter month's last day -- e.g. Jan 31 + 1 month =
/// Feb 28/29, not an error and not Mar 3) followed by a plain day offset.
/// `negate`: `true` for `date - duration`.
///
/// `seconds`/`nanos` can't shift a `Date` by a fraction of a day, but
/// aren't simply dropped: any whole extra day they add still counts (e.g.
/// a duration normalizing to `days: 29` plus a ~34-hour remainder
/// contributes one more whole day on top of the 29). `seconds/86_400`
/// (truncated towards zero, so a negative duration's extra day is
/// subtracted, not added) is the whole-day count; anything finer is
/// discarded -- Date's precision floor is one day.
pub fn add_duration_to_date(
    epoch_day: i64,
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i32,
    negate: bool,
) -> Option<i64> {
    let total_ns: i128 = seconds as i128 * NANOS_PER_SEC + nanos as i128;
    let extra_days = (total_ns / (86_400 * NANOS_PER_SEC)) as i64;
    let days = days.checked_add(extra_days)?;
    let (months, days) = if negate {
        (months.checked_neg()?, days.checked_neg()?)
    } else {
        (months, days)
    };
    let result = add_months_to_epoch_day(epoch_day, months)?.checked_add(days)?;
    // Keep the result inside Cypher's year range -- the chrono-backed
    // version got this via NaiveDate's own (smaller) range failing.
    let (y, _, _) = civil_from_days(result);
    if !(MIN_YEAR..=MAX_YEAR).contains(&y) {
        return None;
    }
    Some(result)
}

/// The four independently-signed components of a normalized `Duration`,
/// matching `PropertyValue::Duration`'s own fields -- a plain tuple
/// alias rather than a re-export, since this module doesn't depend on
/// `marsdb_graph`.
pub type DurationParts = (i64, i64, i64, i32);

/// Raw, not-yet-normalized inputs to `duration({...})`/`duration('...')`
/// construction -- one `f64` per Cypher map key (`0.0` when absent), kept
/// as a struct so call sites read as `years: 12.0, ..Default::default()`
/// rather than an unlabeled tuple.
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
/// nanos)` form. The cascade only flows one direction -- years into
/// months, a fractional month's remainder into days (via
/// `AVG_MONTH_DAYS`), a fractional day's remainder into seconds,
/// sub-second fields into nanoseconds -- matching Neo4j's documented
/// normalization. Never the other direction: seconds never cascade into
/// days (`duration({hours: 40})` stays `PT40H`, not `P1DT16H`), since a
/// "day" isn't a fixed number of hours once timezones/DST exist.
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
/// fields) and `scale_duration` (multiply/divide by a scalar) -- callers
/// differ only in what they pass as `seconds_f`/`extra_nanos`.
fn cascade(months_f: f64, days_f: f64, seconds_f: f64, extra_nanos: i128) -> DurationParts {
    let whole_months = months_f.trunc();
    let frac_months = months_f - whole_months;
    let days_f2 = days_f + frac_months * AVG_MONTH_DAYS;
    let whole_days = days_f2.trunc();
    let frac_days = days_f2 - whole_days;
    let seconds_f2 = seconds_f + frac_days * 86_400.0;
    // `.round()`, not `.trunc()`: `seconds_f2` is a continuous quantity
    // built from several multiplications/additions, so it can land a
    // few-ULP hair off the exact value; rounding recovers the intended
    // nanosecond that truncating could drop.
    let total_ns = (seconds_f2 * NANOS_PER_SEC as f64).round() as i128 + extra_nanos;
    let seconds = (total_ns / NANOS_PER_SEC) as i64;
    let nanos = (total_ns % NANOS_PER_SEC) as i32;
    (whole_months as i64, whole_days as i64, seconds, nanos)
}

/// Component-wise `a + b` -- not a re-cascade through `normalize_
/// duration`: months/days add directly (no re-derivation via
/// `AVG_MONTH_DAYS`), and only `seconds`/`nanos` carry between each other,
/// via the exact `i128` total, avoiding the sign-mismatch bug a naive
/// `a.nanos + b.nanos` would hit when the operands' `seconds` signs
/// differ. `None` if any component overflows its persisted representation.
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
/// `normalize_duration` uses, since scaling a whole month by a
/// non-integer factor produces a fractional month again (`P1M / 2` must
/// become "15.2 days", not stay a fractional month). Calls the shared
/// `cascade` directly with `months`/`days` and the exact `seconds`+`nanos`
/// total pre-multiplied as one `i128` quantity.
pub fn scale_duration(a: DurationParts, factor: f64) -> DurationParts {
    let months_f = a.0 as f64 * factor;
    let days_f = a.1 as f64 * factor;
    let total_ns_exact = a.2 as i128 * NANOS_PER_SEC + a.3 as i128;
    let extra_nanos = (total_ns_exact as f64 * factor).trunc() as i128;
    cascade(months_f, days_f, 0.0, extra_nanos)
}

/// `d.<prop>` component access for a `Duration` -- every field (`years`,
/// `quarters`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`,
/// `milliseconds`, `microseconds`, `nanoseconds`) is the whole duration
/// re-expressed in that one unit alone, truncated towards zero -- not a
/// calendar-style "months-of-year part" breakdown (16 total months gives
/// `d.years == 1` and `d.months == 16`, not `4`). The `*OfX` fields
/// (`monthsOfYear`, `secondsOfMinute`, ...) are each the same
/// computation's remainder instead of its quotient.
///
/// `seconds`/`nanos` are stored the way Java's `Duration` stores them:
/// `seconds` carries the whole sign, `nanos` is always non-negative
/// (0..999_999_999) -- see `PropertyValue::Duration`'s docs. Accessors
/// must read these two raw fields directly, not recombine into one
/// signed total and re-split, which would reintroduce a negative `nanos`
/// (`-23H-59M-59.9S` is stored as `seconds: -86400, nanos: 100_000_000`;
/// re-splitting via truncating division gives the wrong
/// `seconds: -86399, nanosecondsOfSecond: -900_000_000`).
/// `hours`/`minutes`/`seconds` only ever divide `seconds` (never touch
/// `nanos`); `milliseconds`/`microseconds`/`nanoseconds` are the one
/// place that combines both fields, since `nanos`' always-non-negative
/// convention makes simple addition already give the right signed result.
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
/// `W`: weeks fold into `days` during normalization and never come back
/// out). Each component is a straight divmod of the sign-independent
/// whole -- a negative `months`/`days`/`seconds` prints its own `-`
/// (`P-6M-15D...`), not one shared sign prefix.
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
/// apart.
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
/// (`P<date>T<time>`, e.g. `P2012-02-02T14:37:21.545`) -- date/time
/// formatted like a calendar date/time-of-day, but each field means
/// "this many years/months/days/hours/minutes/seconds", not an actual
/// calendar date: no day-of-month validity check, `P2012-13-40` is a
/// legal 12-year-13-month-40-day duration under this form. Only matches
/// when `date_part` has this shape (plain `N-N-N`, no unit letters) --
/// an ordinary `PnYnMnD` string never does, and a negative duration's
/// leading `-` makes the first split empty rather than a valid number.
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
/// optional `.digits`, then exactly one unit letter). The entire input
/// must match: returning a successfully parsed prefix would make
/// malformed text such as `P1Ygarbage` silently construct a one-year
/// duration.
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
/// digits/`:`/`.` only). Only called on the time half of a combined
/// date+time string (after splitting on `T`), never the date half, which
/// legitimately contains `-`.
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

/// `localtime('21:40:32.142')` -- a bare time-of-day, no offset allowed.
/// A trailing `Z`/`+HH:MM` fails the strict digit/`:`/`.`-only parse
/// above and correctly returns `None`.
pub fn parse_local_time(s: &str) -> Option<i64> {
    parse_time_of_day(s.trim())
}

/// `time('21:40:32.142+01:00')` -- a time-of-day with a required offset.
/// Returns `None` if the string has no offset, or carries a bracketed
/// named-zone suffix (`[Europe/Stockholm]`) -- the caller
/// (`Executor::call_builtin`'s `"time"` arm) checks for `[` itself first
/// and raises a specific "named zones aren't supported" error, but this
/// function still refuses to silently misparse the bracket if called
/// directly.
pub fn parse_time(s: &str) -> Option<(i64, i32)> {
    let s = s.trim();
    if s.contains('[') {
        return None;
    }
    let (time_part, offset_part) = split_time_offset(s);
    // A missing offset defaults to UTC (`+00:00`) -- `time()` falls back
    // to the statement's default time zone rather than rejecting the
    // string outright.
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
/// seconds component).
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
/// only if nanos is non-zero (trailing zeros trimmed).
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
pub fn split_epoch_seconds(epoch_seconds: i64) -> (i64, i64) {
    let epoch_day = epoch_seconds.div_euclid(SECONDS_PER_DAY);
    let secs_of_day = epoch_seconds.rem_euclid(SECONDS_PER_DAY);
    (epoch_day, secs_of_day * 1_000_000_000)
}

pub fn combine_epoch_day_and_nanos_of_day(epoch_day: i64, nanos_of_day: i64) -> i64 {
    epoch_day * SECONDS_PER_DAY + nanos_of_day / 1_000_000_000
}

/// Combines an `(epoch_day, nanos_of_day)` pair into `LocalDateTime`'s
/// `(epoch_seconds, nanos)` storage shape -- shared by `<type>.
/// truncate()`'s date+time recombination step.
pub fn combine_date_and_time(epoch_day: i64, nanos_of_day: i64) -> (i64, i32) {
    (
        combine_epoch_day_and_nanos_of_day(epoch_day, nanos_of_day),
        (nanos_of_day % 1_000_000_000) as i32,
    )
}

/// Calendar + time-of-day fields for `localdatetime({...})`/
/// `datetime({...})`'s map constructors -- bundled into one struct (not
/// 7 positional args) to stay under clippy's argument-count cap.
pub struct CalendarDateTime {
    pub year: i64,
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
/// in the given zone -- for a fixed `Offset`, subtracts it to get the UTC
/// instant `DateTime` actually stores; for a `Named` zone, resolves the
/// real DST-aware offset for this specific local date-time via
/// `chrono-tz`, since the same zone can mean a different offset on a
/// different date.
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
            let naive = chrono_naive_from(epoch_day, nanos_of_day)?;
            let (epoch_seconds, _offset) = utc_from_local_and_named_zone(naive, tz)?;
            Some((epoch_seconds, (nanos_of_day % 1_000_000_000) as i32))
        }
    }
}

/// Parses `YYYY-MM-DDTHH:MM:SS.fff` (and the compact/date-only-precision
/// variants `parse_date` already supports for the date half) into a
/// naive `(epoch_seconds, nanos)` instant. A date-only string (no `T`)
/// is also accepted, reading as midnight.
pub fn parse_local_date_time(s: &str) -> Option<(i64, i32)> {
    let s = s.trim();
    let (date_part, time_part) = match s.split_once('T') {
        Some(parts) => parts,
        None => (s, ""),
    };
    let epoch_day = parse_date(date_part)?;
    let nanos_of_day = if time_part.is_empty() {
        0
    } else {
        parse_time_of_day(time_part)?
    };
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
            let naive = chrono_naive_from(epoch_day, nanos_of_day)?;
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
/// into). Truncates a Duration's calendar components (`months`/`days`)
/// when adding to a time-only value -- only `seconds`/`nanos` apply --
/// rather than erroring, so this never fails.
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
/// arithmetic on the date part (same clamping as `add_duration_to_date`),
/// then `days`/`seconds`/`nanos` added as one exact nanosecond count that
/// carries across day boundaries. Unlike `Date`, a `LocalDateTime`/
/// `DateTime` has a time-of-day to carry into, so nothing here gets
/// truncated. Operates on the local wall-clock reading -- `DateTime`
/// callers pass `epoch_seconds + offset_seconds` in and subtract
/// `offset_seconds` back out of the result, so month/day arithmetic
/// happens against the calendar the user wrote, not the UTC instant.
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
    let new_epoch_day = add_months_to_epoch_day(epoch_day, months)?;

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
    // The displayed wall-clock reading is the local (offset-adjusted)
    // one, not the stored UTC instant -- `DateTime` round-trips through
    // `toString`/reparse showing the original offset's time-of-day.
    let offset_seconds = resolve_offset(zone, epoch_seconds);
    let local_epoch_seconds = epoch_seconds + offset_seconds as i64;
    let zone_suffix = match zone {
        TzId::Offset(_) => String::new(),
        // `toString()` round-trips the zone name alongside its resolved
        // offset (`+02:00[Europe/Stockholm]`), not just the offset alone.
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
/// zone name resolves to a different offset depending on which instant
/// this is called with, e.g. `Europe/Stockholm` is `+01:00` in October
/// and `+02:00` in July). Falls back to UTC (`0`) for a zone name that
/// fails to parse -- should never happen for a value MarsDB itself
/// constructed, but this function can't return an error, so it degrades
/// gracefully rather than panicking on a hypothetical corrupt value.
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

/// Given a local (wall-clock) naive date-time and a named zone, resolves
/// the true UTC `(epoch_seconds, offset_seconds)`. The common case is
/// `LocalResult::Single`; a DST fall-back repeated hour (`Ambiguous`)
/// takes the earlier instant; a DST spring-forward gap (`None`, the
/// local time never occurred) has no valid mapping and fails.
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

/// A civil (zone-less) date-time as this module's own pair -- replaces
/// chrono's `NaiveDateTime` in the `duration.between` core so the full
/// ±999_999_999-year range works (see the civil-core section's docs).
#[derive(Clone, Copy)]
struct CivilDateTime {
    epoch_day: i64,
    nanos_of_day: i64,
}

/// Total nanoseconds since the epoch -- i128 because the full year
/// range's span (~6.3e25 ns) far exceeds i64.
fn civil_total_ns(dt: CivilDateTime) -> i128 {
    dt.epoch_day as i128 * NANOS_PER_DAY_I64 as i128 + dt.nanos_of_day as i128
}

/// `NaiveDateTime` for the chrono-backed named-zone paths only -- `None`
/// outside chrono's own ±262k-year range (a named IANA zone has no
/// meaningful data at such years; fixed offsets never come through
/// here).
fn chrono_naive_from(epoch_day: i64, nanos_of_day: i64) -> Option<NaiveDateTime> {
    let secs = epoch_day.checked_mul(SECONDS_PER_DAY)? + nanos_of_day / 1_000_000_000;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, (nanos_of_day % 1_000_000_000) as u32)
        .map(|utc| utc.naive_utc())
}

/// `java.time`'s `LocalDate`-difference-in-whole-months primitive
/// (`ChronoUnit.MONTHS.between`, which Neo4j's own `duration.between`
/// mirrors exactly): pack each date into a single sortable
/// `proleptic_month * 32 + day_of_month` value (32 safely exceeds any
/// month's real day count) so one integer division gives the exact
/// whole-month count, day-of-month aware, without a real calendar walk.
fn packed_proleptic(epoch_day: i64) -> i64 {
    let (y, m, d) = civil_from_days(epoch_day);
    (y * 12 + m as i64 - 1) * 32 + d as i64
}

fn months_between_days(a: i64, b: i64) -> i64 {
    (packed_proleptic(b) - packed_proleptic(a)) / 32
}

/// Adds `months` to `dt`'s *date* only (real calendar month arithmetic,
/// clamping to the shorter month's last day, same as
/// `add_duration_to_date`), keeping the time-of-day unchanged.
fn shift_months(dt: CivilDateTime, months: i64) -> CivilDateTime {
    let shifted = add_months_to_epoch_day(dt.epoch_day, months)
        .expect("months_between_days never shifts past the endpoint it was computed from");
    CivilDateTime {
        epoch_day: shifted,
        nanos_of_day: dt.nanos_of_day,
    }
}

/// Shared core of `duration.between`/`.inMonths`/`.inDays`/
/// `.inSeconds`: `(months, shifted_remaining_ns, raw_total_ns)`.
///
/// If either operand has no calendar date (`a_date`/`b_date` is `None` --
/// a bare `LocalTime`/`Time`), both operands' dates are disregarded
/// entirely, not even treated as a shared reference day: `months` is
/// always `0`, and both the "raw" and "month-shifted" totals collapse to
/// the same plain time-of-day delta.
///
/// Otherwise: `months` is the real calendar month count between the two
/// full date-times (`months_between_datetimes_offset_aware`);
/// `shifted_remaining_ns` is the exact elapsed time between `from`
/// shifted forward by that many months and `to` (what `duration.between`
/// bucket-splits into days/seconds/nanos on top of `months`, not a
/// further calendar-date subtraction); `raw_total_ns` is the plain,
/// unshifted elapsed time between the two original instants, what
/// `.inDays`/`.inSeconds` use instead, discarding the month optimization
/// entirely.
fn to_utc_instant_ns(dt: CivilDateTime, zone: &TzId) -> i128 {
    match zone {
        TzId::Offset(o) => civil_total_ns(dt) - *o as i128 * NANOS_PER_SEC,
        TzId::Named(name) => {
            if let (Some(tz), Some(naive)) = (
                parse_timezone_name(name),
                chrono_naive_from(dt.epoch_day, dt.nanos_of_day),
            ) {
                if let Some((epoch_seconds, _)) = utc_from_local_and_named_zone(naive, tz) {
                    return epoch_seconds as i128 * NANOS_PER_SEC
                        + (dt.nanos_of_day % 1_000_000_000) as i128;
                }
            }
            civil_total_ns(dt)
        }
    }
}

fn elapsed_ns(
    from: CivilDateTime,
    from_zone: Option<&TzId>,
    to: CivilDateTime,
    to_zone: Option<&TzId>,
) -> i128 {
    match (from_zone, to_zone) {
        (Some(fz), Some(tz)) => to_utc_instant_ns(to, tz) - to_utc_instant_ns(from, fz),
        (Some(fz), None) => to_utc_instant_ns(to, fz) - to_utc_instant_ns(from, fz),
        (None, Some(tz)) => to_utc_instant_ns(to, tz) - to_utc_instant_ns(from, tz),
        (None, None) => civil_total_ns(to) - civil_total_ns(from),
    }
}

fn months_between_datetimes_offset_aware(
    from: CivilDateTime,
    from_zone: Option<&TzId>,
    to: CivilDateTime,
    to_zone: Option<&TzId>,
) -> i64 {
    let mut months = months_between_days(from.epoch_day, to.epoch_day);
    let shifted = shift_months(from, months);
    let delta = elapsed_ns(shifted, from_zone, to, to_zone);
    if months > 0 && delta < 0 {
        months -= 1;
    } else if months < 0 && delta > 0 {
        months += 1;
    }
    months
}

fn time_to_utc_nanos(nanos_of_day: i64, zone: &TzId, ref_date: Option<i64>) -> i128 {
    let dt = CivilDateTime {
        epoch_day: ref_date.unwrap_or(0),
        nanos_of_day,
    };
    to_utc_instant_ns(dt, zone)
}

fn between_components(
    a_date: Option<i64>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i64>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> (i64, i128, i128) {
    match (a_date, b_date) {
        (Some(ad), Some(bd)) => {
            let from = CivilDateTime {
                epoch_day: ad,
                nanos_of_day: a_time.unwrap_or(0),
            };
            let to = CivilDateTime {
                epoch_day: bd,
                nanos_of_day: b_time.unwrap_or(0),
            };
            let months = months_between_datetimes_offset_aware(from, a_zone, to, b_zone);
            let shifted = shift_months(from, months);
            let shifted_remaining_ns = elapsed_ns(shifted, a_zone, to, b_zone);
            let raw_total_ns = elapsed_ns(from, a_zone, to, b_zone);
            (months, shifted_remaining_ns, raw_total_ns)
        }
        _ => {
            let diff = match (a_zone, b_zone) {
                (Some(az), Some(bz)) => {
                    // Both sides resolve against the same reference date:
                    // "time-only mode" disregards the date each operand
                    // happens to carry (see this function's doc comment),
                    // so `a`/`b` must not each pull in their own,
                    // potentially different, real date -- that only
                    // cancels out in `bt - at` when identical on both
                    // sides. Only matters for a `Named` zone's
                    // DST-dependent offset -- a fixed `Offset` doesn't
                    // care what date it's given.
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
                (None, None) => (b_time.unwrap_or(0) - a_time.unwrap_or(0)) as i128,
            };
            (0, diff, diff)
        }
    }
}

pub fn duration_between(
    a_date: Option<i64>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i64>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (months, shifted_ns, _) =
        between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    // After the month shift the remainder spans at most one month of
    // calendar distance -- days always fit i64; the sub-day remainder
    // always fits i64 seconds.
    let days = (shifted_ns / NANOS_PER_DAY_I64 as i128) as i64;
    let rem = shifted_ns % NANOS_PER_DAY_I64 as i128;
    let seconds = rem.div_euclid(NANOS_PER_SEC) as i64;
    let nanos = rem.rem_euclid(NANOS_PER_SEC) as i32;
    (months, days, seconds, nanos)
}

pub fn duration_in_months(
    a_date: Option<i64>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i64>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (months, _, _) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (months, 0, 0, 0)
}

pub fn duration_in_days(
    a_date: Option<i64>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i64>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (_, _, raw) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (0, (raw / NANOS_PER_DAY_I64 as i128) as i64, 0, 0)
}

pub fn duration_in_seconds(
    a_date: Option<i64>,
    a_time: Option<i64>,
    a_zone: Option<&TzId>,
    b_date: Option<i64>,
    b_time: Option<i64>,
    b_zone: Option<&TzId>,
) -> DurationParts {
    let (_, _, raw) = between_components(a_date, a_time, a_zone, b_date, b_time, b_zone);
    (
        0,
        0,
        // The full-year-range span (~6.3e16 s) fits i64 seconds even
        // though its nanosecond total doesn't.
        raw.div_euclid(NANOS_PER_SEC) as i64,
        raw.rem_euclid(NANOS_PER_SEC) as i32,
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
/// ISO week-date math as `.week`/`.weekYear` component access
/// (`date_component`) -- the Monday of that ISO week/week-year.
pub fn truncate_date_unit(epoch_day: i64, unit: &str) -> Option<i64> {
    let (y, m, _) = civil_from_days(epoch_day);
    match unit {
        "millennium" => epoch_day_from_ymd(y - y.rem_euclid(1000), 1, 1),
        "century" => epoch_day_from_ymd(y - y.rem_euclid(100), 1, 1),
        "decade" => epoch_day_from_ymd(y - y.rem_euclid(10), 1, 1),
        "year" => epoch_day_from_ymd(y, 1, 1),
        "quarter" => epoch_day_from_ymd(y, (m - 1) / 3 * 3 + 1, 1),
        "month" => epoch_day_from_ymd(y, m, 1),
        "week" => {
            let (week_year, week) = iso_week_of(epoch_day);
            Some(iso_week1_monday(week_year) + (week - 1) * 7)
        }
        "weekYear" => {
            let (week_year, _) = iso_week_of(epoch_day);
            Some(iso_week1_monday(week_year))
        }
        "day" => Some(epoch_day),
        _ => None,
    }
}

/// Moves `epoch_day` to the given ISO weekday (`1`=Monday..`7`=Sunday)
/// within its own ISO week -- the `dayOfWeek` override key on a
/// `.truncate('week', ...)` result (`date.truncate('week', d,
/// {dayOfWeek: 2})` is "the Tuesday of `d`'s week"), not general
/// week-date construction from a `{year, week, dayOfWeek}` triple with
/// no existing anchor date (that's `epoch_day_from_week_fields`).
/// `None` for an out-of-range `day_of_week`.
pub fn set_iso_weekday(epoch_day: i64, day_of_week: i64) -> Option<i64> {
    if !(1..=7).contains(&day_of_week) {
        return None;
    }
    let (week_year, week) = iso_week_of(epoch_day);
    let monday = iso_week1_monday(week_year) + (week - 1) * 7;
    Some(monday + (day_of_week - 1))
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

    /// A fractional-duration case where whole extra days from
    /// `seconds`/`nanos` must be folded into the date -- see
    /// `add_duration_to_date`'s doc comment.
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

    /// The hand-rolled civil core must agree with chrono everywhere
    /// chrono can go -- sweeps ±~2.7 millennia of epoch days at a prime
    /// stride and cross-checks every derived component.
    #[test]
    fn civil_core_matches_chrono_across_its_range() {
        use chrono::Datelike;
        for epoch_day in (-1_000_000..1_000_000i64).step_by(9973) {
            let d = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                + chrono::Duration::days(epoch_day);
            let (y, m, day) = civil_from_days(epoch_day);
            assert_eq!((y, m, day), (d.year() as i64, d.month(), d.day()));
            assert_eq!(days_from_civil(y, m, day), epoch_day);
            assert_eq!(
                iso_weekday_from_days(epoch_day),
                d.weekday().number_from_monday() as i64
            );
            let iso = d.iso_week();
            assert_eq!(
                iso_week_of(epoch_day),
                (iso.year() as i64, iso.week() as i64)
            );
            assert_eq!(ordinal_day_of(y, m, day), d.ordinal() as i64);
        }
    }

    #[test]
    fn expanded_year_dates_parse_and_round_trip() {
        let min = parse_date("-999999999-01-01").unwrap();
        let max = parse_date("+999999999-12-31").unwrap();
        assert_eq!(format_date(min), "-999999999-01-01");
        assert_eq!(format_date(max), "+999999999-12-31");
        assert_eq!(date_component(min, "year"), Some(-999_999_999));
        assert_eq!(date_component(max, "year"), Some(999_999_999));
        // Normal-range years stay unsigned/4-digit-padded.
        assert_eq!(format_date(parse_date("2020-01-10").unwrap()), "2020-01-10");
        assert_eq!(format_date(parse_date("0033-06-01").unwrap()), "0033-06-01");
    }

    #[test]
    fn year_range_and_calendar_validity_are_enforced() {
        assert!(parse_date("+1000000000-01-01").is_none());
        assert!(parse_date("-1000000000-01-01").is_none());
        assert!(epoch_day_from_ymd(2020, 13, 1).is_none());
        assert!(epoch_day_from_ymd(2020, 2, 30).is_none());
        // Century leap rules: 1900 isn't a leap year, 2000 is.
        assert!(epoch_day_from_ymd(1900, 2, 29).is_none());
        assert!(epoch_day_from_ymd(2000, 2, 29).is_some());
    }

    /// The full-range duration.between.
    #[test]
    fn duration_between_spans_the_full_year_range() {
        let a = parse_date("-999999999-01-01").unwrap();
        let b = parse_date("+999999999-12-31").unwrap();
        let parts = duration_between(Some(a), None, None, Some(b), None, None);
        assert_eq!(format_duration_parts(parts), "P1999999998Y11M30D");
    }

    /// The full-range duration.inSeconds, whose nanosecond total
    /// overflows i64 -- a regression test for the i128 core.
    #[test]
    fn duration_in_seconds_spans_the_full_year_range() {
        let (a_secs, a_nanos) = parse_local_date_time("-999999999-01-01").unwrap();
        let (b_secs, b_nanos) = parse_local_date_time("+999999999-12-31T23:59:59").unwrap();
        let (a_day, a_nod) = split_epoch_seconds(a_secs);
        let (b_day, b_nod) = split_epoch_seconds(b_secs);
        let parts = duration_in_seconds(
            Some(a_day),
            Some(a_nod + a_nanos as i64),
            None,
            Some(b_day),
            Some(b_nod + b_nanos as i64),
            None,
        );
        assert_eq!(format_duration_parts(parts), "PT17531639991215H59M59S");
    }
}
