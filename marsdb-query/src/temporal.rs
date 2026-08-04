//! Calendar math and ISO-8601 text conversion for `PropertyValue::Date`/
//! `PropertyValue::Duration` -- kept out of `marsdb-graph` deliberately
//! (that crate stores the value, it doesn't know Cypher's construction/
//! formatting rules -- see `PropertyValue`'s own doc comment) and out of
//! `executor.rs` (which owns *dispatching* to these, not the arithmetic
//! itself, matching the split `apply_arith`/`compare` already have from
//! e.g. the planner).
//!
//! Scope, honestly: only `DATE` (calendar year/month/day, no week-date/
//! ordinal-date/quarter construction forms) and `DURATION` are supported.
//! `LOCAL TIME`/`TIME`/`LOCAL DATETIME`/`DATETIME` (anything with a
//! time-of-day or timezone) don't exist at all -- see the README's
//! "Cypher coverage" section for the exact list of what that leaves out
//! of TCK's `expressions/temporal` suite.

use chrono::{Datelike, NaiveDate};

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

/// `date()` with no arguments -- today's date in UTC. There's no timezone
/// support at all (see this module's top-of-file docs), so "today" can
/// only mean one fixed instant's date rather than "today where the caller
/// is" -- UTC, the same zero-configuration choice every timezone-naive
/// part of this module already makes.
pub fn today_epoch_day() -> i32 {
    let today = chrono::Utc::now().date_naive();
    epoch_day_from_ymd(today.year(), today.month(), today.day()).expect("today is always a valid date")
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

/// Parses the calendar-date string forms MarsDB supports: `YYYY-MM-DD`,
/// `YYYYMMDD`, `YYYY-MM`, `YYYYMM`, `YYYY` (missing month/day default to
/// `1`). Deliberately does *not* handle the ISO week-date (`2015-W30-2`)
/// or ordinal-date (`2015-202`) string forms real Cypher also accepts --
/// a real gap (see the README), not silently misparsed: a week-date
/// string's `W` fails the plain-integer month/day parse below and
/// correctly returns `None` rather than a wrong date.
pub fn parse_date(s: &str) -> Option<i32> {
    let s = s.trim();
    // The compact forms below use byte offsets because their grammar is
    // ASCII-only. Reject non-ASCII input before slicing so malformed user
    // input can never put an offset in the middle of a UTF-8 code point.
    if !s.is_ascii() {
        return None;
    }
    let (year, month, day) = if let Some((y, rest)) = s.split_once('-') {
        let year: i32 = y.parse().ok()?;
        match rest.split_once('-') {
            Some((m, d)) => (year, m.parse().ok()?, d.parse().ok()?),
            None => (year, rest.parse().ok()?, 1),
        }
    } else {
        match s.len() {
            8 => (s[0..4].parse().ok()?, s[4..6].parse().ok()?, s[6..8].parse().ok()?),
            6 => (s[0..4].parse().ok()?, s[4..6].parse().ok()?, 1),
            4 => (s[0..4].parse().ok()?, 1, 1),
            _ => return None,
        }
    };
    epoch_day_from_ymd(year, month, day)
}

/// `d.<prop>` component access for a `Date` -- the "forward" (date ->
/// components) half of ISO week/quarter calendar math, which is much
/// simpler than the "backward" half (`date({week: ..., dayOfWeek: ...})`
/// construction, deliberately not supported -- see `Executor::
/// call_builtin`'s `"date"` arm, which only builds a `Date` from
/// `year`/`month`/`day` map keys) since it's pure arithmetic on an
/// already-valid date, no ambiguity to resolve. Returns `None` for any
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
pub fn add_duration_to_date(epoch_day: i32, months: i64, days: i64, seconds: i64, nanos: i32, negate: bool) -> Option<i32> {
    let total_ns: i128 = seconds as i128 * NANOS_PER_SEC + nanos as i128;
    let extra_days = (total_ns / (86_400 * NANOS_PER_SEC)) as i64;
    let days = days + extra_days;
    let (months, days) = if negate { (-months, -days) } else { (months, days) };
    let d = date_from_epoch_day(epoch_day);
    let with_months = if months >= 0 {
        d.checked_add_months(chrono::Months::new(months as u32))?
    } else {
        d.checked_sub_months(chrono::Months::new((-months) as u32))?
    };
    let result = with_months.checked_add_signed(chrono::Duration::days(days))?;
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
    let extra_nanos = (f.milliseconds * 1_000_000.0 + f.microseconds * 1_000.0 + f.nanoseconds).trunc() as i128;
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
/// when the two operands' `seconds` signs differ).
pub fn add_duration(a: DurationParts, b: DurationParts) -> DurationParts {
    let months = a.0 + b.0;
    let days = a.1 + b.1;
    let total_ns = a.2 as i128 * NANOS_PER_SEC + a.3 as i128 + b.2 as i128 * NANOS_PER_SEC + b.3 as i128;
    (months, days, (total_ns / NANOS_PER_SEC) as i64, (total_ns % NANOS_PER_SEC) as i32)
}

pub fn negate_duration(a: DurationParts) -> DurationParts {
    (-a.0, -a.1, -a.2, -a.3)
}

pub fn sub_duration(a: DurationParts, b: DurationParts) -> DurationParts {
    add_duration(a, negate_duration(b))
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
pub fn duration_component(months: i64, days: i64, seconds: i64, nanos: i32, prop: &str) -> Option<i64> {
    let total_ns: i128 = seconds as i128 * NANOS_PER_SEC + nanos as i128;
    Some(match prop {
        "years" => months / 12,
        "quarters" => months / 3,
        "months" => months,
        "weeks" => days / 7,
        "days" => days,
        "hours" => (total_ns / 3_600_000_000_000) as i64,
        "minutes" => (total_ns / 60_000_000_000) as i64,
        "seconds" => (total_ns / NANOS_PER_SEC) as i64,
        "milliseconds" => (total_ns / 1_000_000) as i64,
        "microseconds" => (total_ns / 1_000) as i64,
        "nanoseconds" => total_ns as i64,
        "quartersOfYear" => (months % 12) / 3,
        "monthsOfQuarter" => (months % 12) % 3,
        "monthsOfYear" => months % 12,
        "daysOfWeek" => days % 7,
        "minutesOfHour" => (total_ns / 60_000_000_000) as i64 % 60,
        "secondsOfMinute" => (total_ns / NANOS_PER_SEC) as i64 % 60,
        "millisecondsOfSecond" => (total_ns / 1_000_000) as i64 % 1000,
        "microsecondsOfSecond" => (total_ns / 1_000) as i64 % 1_000_000,
        "nanosecondsOfSecond" => (total_ns % NANOS_PER_SEC) as i64,
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
    if seconds != 0 || nanos != 0 {
        out.push('T');
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
    format!("{}{}.{}", if negative { "-" } else { "" }, secs.unsigned_abs(), frac)
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
        let value = chars[start..i].iter().collect::<String>().parse::<f64>().ok()?;
        out.push((value, unit));
        i += 1;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn du(months: f64, days: f64, hours: f64, minutes: f64, seconds: f64) -> DurationParts {
        normalize_duration(DurationFields { months, days, hours, minutes, seconds, ..Default::default() })
    }

    #[test]
    fn construct_basic() {
        assert_eq!(format_duration_parts(du(0.0, 14.0, 16.0, 12.0, 0.0)), "P14DT16H12M");
    }

    #[test]
    fn construct_fractional_months() {
        let d = normalize_duration(DurationFields { months: 0.75, ..Default::default() });
        assert_eq!(format_duration_parts(d), "P22DT19H51M49.5S");
    }

    #[test]
    fn construct_fractional_weeks() {
        let d = normalize_duration(DurationFields { weeks: 2.5, ..Default::default() });
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
        let d = normalize_duration(DurationFields { days: 14.0, seconds: 70.0, milliseconds: 1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "P14DT1M10.001S");
    }

    #[test]
    fn construct_minutes_fraction() {
        let d = normalize_duration(DurationFields { minutes: 1.5, seconds: 1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "PT1M31S");
    }

    #[test]
    fn parse_string_p14dt16h12m() {
        assert_eq!(format_duration_parts(parse_duration("P14DT16H12M").unwrap()), "P14DT16H12M");
    }

    #[test]
    fn parse_string_p0_75m() {
        assert_eq!(format_duration_parts(parse_duration("P0.75M").unwrap()), "P22DT19H51M49.5S");
    }

    #[test]
    fn parse_string_pt0_75m() {
        assert_eq!(format_duration_parts(parse_duration("PT0.75M").unwrap()), "PT45S");
    }

    #[test]
    fn malformed_temporal_strings_are_rejected_without_panicking() {
        assert_eq!(parse_date("123é4"), None);
        for malformed in ["P", "PT", "Pgarbage", "P1Ygarbage", "P1Y2", "P1.2.3Y"] {
            assert_eq!(parse_duration(malformed), None, "{malformed} must be rejected");
        }
    }

    #[test]
    fn add_durations() {
        let a = du(149.0, 14.0, 16.0, 12.0, 70.0);
        let a = (a.0, a.1, a.2, 1);
        let sum = add_duration(a, a);
        assert_eq!(format_duration_parts(sum), "P24Y10M28DT32H26M20.000000002S");
    }

    #[test]
    fn scale_duration_by_half() {
        let base = (149, 14, 58390, 1);
        assert_eq!(format_duration_parts(scale_duration(base, 0.5)), "P6Y2M22DT13H21M8S");
        assert_eq!(format_duration_parts(scale_duration(base, 2.0)), "P24Y10M28DT32H26M20.000000002S");
    }

    #[test]
    fn negative_seconds_fraction() {
        let d = normalize_duration(DurationFields { seconds: 2.0, milliseconds: -1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "PT1.999S");
        let d = normalize_duration(DurationFields { seconds: -2.0, milliseconds: 1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "PT-1.999S");
        let d = normalize_duration(DurationFields { seconds: -2.0, milliseconds: -1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "PT-2.001S");
        let d = normalize_duration(DurationFields { seconds: 60.0, milliseconds: -1.0, ..Default::default() });
        assert_eq!(format_duration_parts(d), "PT59.999S");
        let d = normalize_duration(DurationFields { minutes: 12.0, seconds: -60.0, ..Default::default() });
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
