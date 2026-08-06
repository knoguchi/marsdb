//! Pure, grammar-agnostic parsing helpers shared by `antlr_visitor.rs` --
//! extracted from the old pest-based `parser.rs` at Phase 3 cutover (see
//! `grammar/README.md`/beads `mars-cuk`), which reused these as-is via
//! `pub(crate)` visibility throughout Phase 2 rather than re-deriving
//! them. None of these ever touched `pest::Pair`/`Rule` directly.

use crate::ast::*;
use crate::error::QueryError;

/// `shortestPath()`'s inner pattern must be exactly the shape it's built
/// for: one variable-length hop between two nodes — not fixed-hop (nothing
/// to search shortest-among), not multi-hop (which hop would even be the
/// variable-length one is ambiguous), not hopless (no relationship to
/// traverse at all).
pub(crate) fn validate_shortest_path_pattern(pattern: &Pattern) -> Result<(), QueryError> {
    if pattern.hops.len() != 1 || pattern.hops[0].0.hop_range.is_none() {
        return Err(QueryError::Syntax(
            "shortestPath() requires exactly one variable-length relationship pattern (e.g. (a)-[:TYPE*..5]-(b))"
                .into(),
        ));
    }
    Ok(())
}

/// General named-path capture (`p = (a)-->(b)`, no `shortestPath()`).
/// Fixed-hop patterns of any length are fine; a *single* variable-length
/// hop (`p = (a)-[*1..3]->(b)`, TCK's Quantifier1-4 `[8]`/`[9]`) is also
/// supported (`executor::name_pattern_for_path`/`assemble_path` know how
/// to capture its own internally-traversed edge/node sequence) -- but a
/// pattern *mixing* a variable-length hop with any other hop isn't (not
/// exercised by the TCK, and which hop's own path-building strategy would
/// even apply gets genuinely ambiguous once there's more than one).
pub(crate) fn validate_named_path_pattern(pattern: &Pattern) -> Result<(), QueryError> {
    let var_len_hops = pattern
        .hops
        .iter()
        .filter(|(rel, _)| rel.hop_range.is_some())
        .count();
    if var_len_hops > 0 && (var_len_hops > 1 || pattern.hops.len() > 1) {
        return Err(QueryError::Syntax(
            "named-path capture (`p = ...`) over a pattern mixing a variable-length relationship \
             with another hop isn't supported yet — use shortestPath() instead, or drop the path \
             variable"
                .into(),
        ));
    }
    Ok(())
}

/// Groups comma-separated patterns within one `MATCH` into linear
/// `Pattern` chains -- when a later pattern's start variable is exactly
/// the previous one's last-introduced variable (e.g. IS2's `MATCH
/// (message)-[...]->(post:Post), (post)-[...]->(person)`, where `post` is
/// both the first pattern's end and the second's start), it's spliced
/// into the same chain (any labels/props it restates on that shared
/// variable merge in as additional filters); otherwise it starts a new
/// group -- a genuine disjoint cross join (`MATCH (a:A), (b:B)`), which
/// becomes its own separate `QueryPart`/`QueryClause::Match`. A later
/// group referencing an even-earlier group's variable (not the
/// immediately-preceding one) doesn't need special-casing here either --
/// it just starts its own new group, and the executor's existing
/// already-bound-variable handling (used for chained MATCH clauses
/// generally) resolves the reference correctly once both clauses run in
/// order.
pub(crate) fn group_into_linear_patterns(
    mut patterns: Vec<Pattern>,
) -> Result<Vec<Pattern>, QueryError> {
    if patterns.is_empty() {
        return Err(QueryError::Syntax("MATCH requires a pattern".into()));
    }
    let mut groups = vec![patterns.remove(0)];
    for next in patterns {
        let current = groups.last_mut().expect("groups is never empty");
        let last_var = current
            .hops
            .last()
            .map(|(_, n)| n.var.clone())
            .unwrap_or_else(|| current.start.var.clone());
        if next.start.var.is_some() && last_var == next.start.var {
            let target = match current.hops.last_mut() {
                Some((_, node)) => node,
                None => &mut current.start,
            };
            target.labels.extend(next.start.labels);
            target.props.extend(next.start.props);
            current.hops.extend(next.hops);
        } else {
            groups.push(next);
        }
    }
    Ok(groups)
}

/// Parses the raw `rel_range` text (`*`, `*N`, `*N..`, `*N..M`, `*..M`)
/// directly rather than via sub-rules, since the `..` literal produces no
/// child node to structurally distinguish "*N" (exact) from "*N.." (N or
/// more).
pub(crate) fn parse_rel_range(text: &str) -> Result<(u32, Option<u32>), QueryError> {
    let rest = &text[1..]; // strip leading '*'
                           // Real Cypher's default minimum is 1, not 0 -- a variable-length
                           // pattern always requires at least one real relationship unless a
                           // zero-length lower bound is written explicitly (`*0..`); `x` in
                           // `(a)-[*]->(x)` is never `a` itself.
    if rest.is_empty() {
        return Ok((1, None));
    }
    if let Some(idx) = rest.find("..") {
        let min_str = &rest[..idx];
        let max_str = &rest[idx + 2..];
        let min = if min_str.is_empty() {
            1
        } else {
            min_str
                .parse()
                .map_err(|_| QueryError::Syntax("invalid variable-length min hop count".into()))?
        };
        let max =
            if max_str.is_empty() {
                None
            } else {
                Some(max_str.parse().map_err(|_| {
                    QueryError::Syntax("invalid variable-length max hop count".into())
                })?)
            };
        Ok((min, max))
    } else {
        let n: u32 = rest
            .parse()
            .map_err(|_| QueryError::Syntax("invalid variable-length hop count".into()))?;
        Ok((n, Some(n)))
    }
}

/// Resolves `\`-escapes in a string literal's already-quote-stripped
/// inner text. Only a fixed recognized set actually means something -- an
/// unrecognized escape (e.g. `\q`) errors here rather than silently
/// dropping the backslash or passing it through, matching this codebase's
/// stance elsewhere (error on an untested shape, don't guess). `\uXXXX`
/// is exactly 4 hex digits (a BMP code point, real Cypher's own escape
/// width -- not the 8-digit `\UXXXXXXXX` some other languages have).
pub(crate) fn unescape_string(s: &str) -> Result<String, QueryError> {
    if !s.contains('\\') {
        return Ok(s.to_string());
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('\'') => out.push('\''),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('u') => {
                let digits: String = (&mut chars).take(4).collect();
                if digits.len() != 4 {
                    return Err(QueryError::Syntax(
                        "\\u escape needs exactly 4 hex digits".into(),
                    ));
                }
                let code = u32::from_str_radix(&digits, 16).map_err(|_| {
                    QueryError::Syntax(format!("\\u{digits} isn't 4 valid hex digits"))
                })?;
                let ch = char::from_u32(code).ok_or_else(|| {
                    QueryError::Syntax(format!("\\u{digits} isn't a valid Unicode code point"))
                })?;
                out.push(ch);
            }
            Some(other) => {
                return Err(QueryError::Syntax(format!(
                    "unrecognized string escape '\\{other}'"
                )))
            }
            None => {
                return Err(QueryError::Syntax(
                    "string ends with a trailing '\\'".into(),
                ))
            }
        }
    }
    Ok(out)
}

/// Parses a (possibly `-`-prefixed, possibly `0x`/`0o`-prefixed) integer
/// literal's text, magnitude first regardless of base, then applies the
/// sign, rather than handing the whole string straight to
/// `str::parse::<i64>()` (which only understands plain decimal).
/// Magnitude-first also correctly handles `i64::MIN`
/// (`-9223372036854775808`/`-0x8000000000000000`): its magnitude, `2^63`,
/// doesn't fit in a *positive* `i64` at all, only in `u64`, and
/// `i64::MIN`'s own negation would itself overflow (`i64`'s range is
/// asymmetric) -- special-cased via the two's-complement identity instead
/// of negating.
pub(crate) fn parse_int_literal(s: &str) -> Result<i64, QueryError> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let magnitude: u64 = if let Some(hex) = rest.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else if let Some(oct) = rest.strip_prefix("0o") {
        u64::from_str_radix(oct, 8)
    } else {
        rest.parse::<u64>()
    }
    .map_err(|_| QueryError::Syntax("invalid integer literal".into()))?;
    let out_of_range = || QueryError::Syntax("integer literal out of range".into());
    if neg {
        if magnitude == 1u64 << 63 {
            Ok(i64::MIN)
        } else {
            i64::try_from(magnitude)
                .ok()
                .and_then(i64::checked_neg)
                .ok_or_else(out_of_range)
        }
    } else {
        i64::try_from(magnitude).map_err(|_| out_of_range())
    }
}
