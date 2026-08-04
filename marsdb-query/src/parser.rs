use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::QueryError;

#[derive(Parser)]
#[grammar = "cypher.pest"]
struct CypherParser;

pub fn parse(input: &str) -> Result<Statement, QueryError> {
    let mut pairs =
        CypherParser::parse(Rule::query, input).map_err(|e| QueryError::Syntax(e.to_string()))?;
    let query_pair = pairs.next().expect("query rule always produces one pair");
    let statement_pair = query_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::statement)
        .expect("query grammar guarantees a statement");
    parse_statement(statement_pair)
}

/// Parses a `;`-separated batch of one or more statements (e.g.
/// `"CREATE (a); CREATE (b); MATCH (n) RETURN n"`). A `;` inside a string
/// literal doesn't split anything — see `queries`' grammar comment.
pub fn parse_many(input: &str) -> Result<Vec<Statement>, QueryError> {
    let mut pairs =
        CypherParser::parse(Rule::queries, input).map_err(|e| QueryError::Syntax(e.to_string()))?;
    let queries_pair = pairs.next().expect("queries rule always produces one pair");
    queries_pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::statement)
        .map(parse_statement)
        .collect()
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let inner = pair.into_inner().next().expect("statement has one child");
    match inner.as_rule() {
        Rule::explain_stmt => parse_explain_stmt(inner),
        Rule::create_index_stmt => parse_create_index_stmt(inner),
        Rule::create_stmt => parse_create_stmt(inner),
        Rule::match_stmt => parse_match_stmt(inner),
        r => unreachable!("unexpected statement child rule {r:?}"),
    }
}

/// `explain_stmt = { ^"EXPLAIN" ~ (create_index_stmt | create_stmt |
/// match_stmt) }` -- one child, the wrapped statement, dispatched through
/// the same per-rule parsers `parse_statement` itself uses (not a second
/// copy of `parse_statement`, since `explain_stmt` can't recurse into
/// another `explain_stmt`).
fn parse_explain_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let inner = pair
        .into_inner()
        .next()
        .expect("explain_stmt wraps exactly one statement");
    let wrapped = match inner.as_rule() {
        Rule::create_index_stmt => parse_create_index_stmt(inner),
        Rule::create_stmt => parse_create_stmt(inner),
        Rule::match_stmt => parse_match_stmt(inner),
        r => unreachable!("unexpected explain_stmt child rule {r:?}"),
    }?;
    Ok(Statement::Explain(Box::new(wrapped)))
}

fn parse_create_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    Ok(Statement::Create(parse_create_patterns(pair)?))
}

/// `create_index_stmt = { ^"CREATE" ~ ^"INDEX" ~ ^"ON" ~ ":" ~ identifier
/// ~ "(" ~ identifier ~ ")" ~ unique_kw? }` -- the two `identifier`
/// children are label then prop (in that order); `unique_kw`'s presence
/// (a real `Pair`, not an inline literal) is what distinguishes `UNIQUE`.
fn parse_create_index_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let mut inner = pair.into_inner();
    let label = inner
        .next()
        .expect("create_index_stmt has a label identifier")
        .as_str()
        .to_string();
    let prop = inner
        .next()
        .expect("create_index_stmt has a prop identifier")
        .as_str()
        .to_string();
    let unique = inner.next().is_some_and(|p| p.as_rule() == Rule::unique_kw);
    Ok(Statement::CreateIndex {
        label,
        prop,
        unique,
    })
}

/// Shared by standalone `CREATE` (`parse_create_stmt`) and a `MATCH ...
/// CREATE` tail (`parse_tail_clause`'s `create_stmt` arm) — both reuse the
/// `create_stmt` grammar rule (`^"CREATE" ~ pattern ~ ("," ~ pattern)*`),
/// only what the executor does with the resulting patterns differs.
fn parse_create_patterns(pair: Pair<Rule>) -> Result<Vec<Pattern>, QueryError> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::pattern)
        .map(parse_pattern)
        .collect()
}

fn parse_match_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let mut clauses = Vec::new();
    let mut tail = None;
    let mut order_by = None;
    let mut limit = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::clause => clauses.push(parse_clause(p)?),
            Rule::tail_clause => tail = Some(parse_tail_clause(p)?),
            Rule::order_by_clause => order_by = Some(parse_order_by_clause(p)?),
            Rule::limit_clause => limit = Some(parse_limit_clause(p)?),
            r => unreachable!("unexpected match_stmt child rule {r:?}"),
        }
    }

    // Mirrors real Cypher's rule that multiple reading clauses need a WITH
    // between them, and additionally caps chaining at one WITH boundary
    // total — nothing IS1-7 needs requires more, and a hand-rolled parser
    // is safer erroring on untested shapes than silently mishandling them.
    // OPTIONAL MATCH and UNWIND are both exempt from the WITH-separation
    // requirement (matching real Cypher: `MATCH (a) OPTIONAL MATCH (b) ...`
    // and `MATCH (a) UNWIND [1,2] AS x ...` are both valid without a WITH
    // between them — they continue in the same scope rather than starting
    // a fresh reading context). The one-WITH-total cap still counts every
    // clause kind's `with` uniformly.
    let with_count = clauses.iter().filter(|c| clause_with(c).is_some()).count();
    if with_count > 1 {
        return Err(QueryError::Syntax(
            "chaining past one WITH boundary in a single MATCH isn't supported yet".into(),
        ));
    }
    for i in 0..clauses.len() {
        let (QueryClause::Match(part), Some(QueryClause::Match(next))) =
            (&clauses[i], clauses.get(i + 1))
        else {
            continue;
        };
        if part.with.is_none() && !next.optional {
            return Err(QueryError::Syntax(
                "multiple MATCH clauses must be separated by WITH".into(),
            ));
        }
    }

    // A missing tail is only valid when a MERGE clause is present (a bare
    // `MERGE (n:Label)`, a pure write with nothing to return — same as
    // standalone CREATE). Otherwise a missing tail is almost certainly a
    // mistake (`MATCH (n)` alone does nothing at all), so it's still
    // rejected.
    if tail.is_none() && !clauses.iter().any(|c| matches!(c, QueryClause::Merge(_))) {
        return Err(QueryError::Syntax(
            "a query needs a RETURN/DELETE/SET tail, unless it has a MERGE clause with nothing after it".into(),
        ));
    }

    Ok(Statement::Match {
        clauses,
        tail,
        order_by,
        limit,
    })
}

fn clause_with(clause: &QueryClause) -> Option<&WithClause> {
    match clause {
        QueryClause::Match(part) => part.with.as_ref(),
        QueryClause::Unwind(u) => u.with.as_ref(),
        QueryClause::Merge(m) => m.with.as_ref(),
        // A standalone leading WITH *is* a WITH boundary itself, not a
        // trailing suffix of one -- still counts toward the "at most one
        // WITH boundary per statement" cap the same way.
        QueryClause::With(with) => Some(with),
    }
}

fn parse_clause(pair: Pair<Rule>) -> Result<QueryClause, QueryError> {
    let inner = pair.into_inner().next().expect("clause has one child");
    match inner.as_rule() {
        Rule::match_part => Ok(QueryClause::Match(parse_match_part(inner)?)),
        Rule::unwind_clause => Ok(QueryClause::Unwind(parse_unwind_clause(inner)?)),
        Rule::merge_clause => Ok(QueryClause::Merge(parse_merge_clause(inner)?)),
        Rule::with_clause => Ok(QueryClause::With(parse_with_clause(inner)?)),
        r => unreachable!("unexpected clause child rule {r:?}"),
    }
}

/// `pattern.hops.len() > 1` is rejected here, not left to the executor —
/// whole-pattern atomicity across multiple simultaneously-unbound hops
/// isn't attempted in v1 (see `executor::eval_merge`'s docs), so a clear
/// parse-time error is better than a confusing runtime one.
fn parse_merge_clause(pair: Pair<Rule>) -> Result<MergeClause, QueryError> {
    let mut inner = pair.into_inner();
    let pattern = parse_pattern(inner.next().expect("merge_clause has a pattern"))?;
    if pattern.hops.len() > 1 {
        return Err(QueryError::Syntax(
            "MERGE with more than one relationship hop isn't supported yet — split it into a MATCH \
             for the already-known part and a MERGE for one new hop"
                .into(),
        ));
    }
    let mut on_create = Vec::new();
    let mut on_match = Vec::new();
    let mut with = None;
    for p in inner {
        match p.as_rule() {
            Rule::on_create_clause => {
                on_create = p
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::set_item)
                    .map(parse_set_item)
                    .collect::<Result<_, _>>()?;
            }
            Rule::on_match_clause => {
                on_match = p
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::set_item)
                    .map(parse_set_item)
                    .collect::<Result<_, _>>()?;
            }
            Rule::with_clause => with = Some(parse_with_clause(p)?),
            r => unreachable!("unexpected merge_clause child rule {r:?}"),
        }
    }
    Ok(MergeClause {
        pattern,
        on_create,
        on_match,
        with,
    })
}

fn parse_unwind_clause(pair: Pair<Rule>) -> Result<UnwindClause, QueryError> {
    let mut inner = pair.into_inner();
    let source = parse_unwind_source(inner.next().expect("unwind_clause has an unwind_source"))?;
    let var = inner
        .next()
        .expect("unwind_clause has an AS identifier")
        .as_str()
        .to_string();
    let mut where_clause = None;
    let mut with = None;
    for p in inner {
        match p.as_rule() {
            Rule::with_where_clause => {
                let expr_pair = p.into_inner().next().expect("WHERE has a with_expr");
                where_clause = Some(parse_with_expr(expr_pair)?);
            }
            Rule::with_clause => with = Some(parse_with_clause(p)?),
            r => unreachable!("unexpected unwind_clause child rule {r:?}"),
        }
    }
    Ok(UnwindClause {
        source,
        var,
        where_clause,
        with,
    })
}

fn parse_unwind_source(pair: Pair<Rule>) -> Result<UnwindSource, QueryError> {
    let inner = pair
        .into_inner()
        .next()
        .expect("unwind_source has one child");
    match inner.as_rule() {
        Rule::list_literal => Ok(UnwindSource::List(
            inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::literal)
                .map(parse_literal)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        // `UNWIND null AS x` is real Cypher -- unwinding null behaves like
        // unwinding an empty list (zero rows), not a bound variable lookup.
        Rule::null_literal => Ok(UnwindSource::List(vec![])),
        Rule::identifier => Ok(UnwindSource::Var(inner.as_str().to_string())),
        r => unreachable!("unexpected unwind_source child rule {r:?}"),
    }
}

fn parse_match_part(pair: Pair<Rule>) -> Result<QueryPart, QueryError> {
    let mut optional = false;
    let mut path_var = None;
    let mut shortest_path = false;
    let mut patterns = Vec::new();
    let mut where_clause = None;
    let mut with = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::match_keyword => {
                optional = p.as_str().to_ascii_uppercase().starts_with("OPTIONAL");
            }
            Rule::path_pattern => {
                let (var, is_shortest, pattern) = parse_path_pattern(p)?;
                path_var = var;
                shortest_path = is_shortest;
                patterns.push(pattern);
            }
            Rule::pattern => patterns.push(parse_pattern(p)?),
            Rule::where_clause => {
                let expr_pair = p.into_inner().next().expect("WHERE has an expr");
                where_clause = Some(parse_expr(expr_pair)?);
            }
            Rule::with_clause => with = Some(parse_with_clause(p)?),
            r => unreachable!("unexpected match_part child rule {r:?}"),
        }
    }
    let pattern = splice_patterns(patterns)?;
    if shortest_path {
        validate_shortest_path_pattern(&pattern)?;
    } else if path_var.is_some() {
        validate_named_path_pattern(&pattern)?;
    }
    Ok(QueryPart {
        optional,
        path_var,
        shortest_path,
        pattern,
        where_clause,
        with,
    })
}

fn parse_path_pattern(pair: Pair<Rule>) -> Result<(Option<String>, bool, Pattern), QueryError> {
    let mut var = None;
    let mut shortest_path = false;
    let mut pattern = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::identifier => var = Some(p.as_str().to_string()),
            Rule::shortest_path_wrapper => {
                shortest_path = true;
                let inner_pattern = p
                    .into_inner()
                    .next()
                    .expect("shortest_path_wrapper has a pattern");
                pattern = Some(parse_pattern(inner_pattern)?);
            }
            Rule::pattern => pattern = Some(parse_pattern(p)?),
            r => unreachable!("unexpected path_pattern child rule {r:?}"),
        }
    }
    Ok((
        var,
        shortest_path,
        pattern.expect("path_pattern always has a pattern or shortest_path_wrapper"),
    ))
}

/// `shortestPath()`'s inner pattern must be exactly the shape it's built
/// for: one variable-length hop between two nodes — not fixed-hop (nothing
/// to search shortest-among), not multi-hop (which hop would even be the
/// variable-length one is ambiguous), not hopless (no relationship to
/// traverse at all).
fn validate_shortest_path_pattern(pattern: &Pattern) -> Result<(), QueryError> {
    if pattern.hops.len() != 1 || pattern.hops[0].0.hop_range.is_none() {
        return Err(QueryError::Syntax(
            "shortestPath() requires exactly one variable-length relationship pattern (e.g. (a)-[:TYPE*..5]-(b))"
                .into(),
        ));
    }
    Ok(())
}

/// General named-path capture (`p = (a)-->(b)`, no `shortestPath()`) is
/// limited to fixed-hop patterns — see `QueryPart::path_var`'s docs for
/// why a variable-length hop isn't supported there.
fn validate_named_path_pattern(pattern: &Pattern) -> Result<(), QueryError> {
    if pattern.hops.iter().any(|(rel, _)| rel.hop_range.is_some()) {
        return Err(QueryError::Syntax(
            "named-path capture (`p = ...`) over a variable-length relationship pattern isn't supported yet \
             — use shortestPath() instead, or drop the path variable"
                .into(),
        ));
    }
    Ok(())
}

fn parse_with_clause(pair: Pair<Rule>) -> Result<WithClause, QueryError> {
    let mut items = Vec::new();
    let mut where_clause = None;
    let mut order_by = None;
    let mut limit = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::return_item => items.push(parse_return_item(p)?),
            Rule::with_where_clause => {
                let expr_pair = p.into_inner().next().expect("WITH...WHERE has a with_expr");
                where_clause = Some(parse_with_expr(expr_pair)?);
            }
            Rule::order_by_clause => order_by = Some(parse_order_by_clause(p)?),
            Rule::limit_clause => limit = Some(parse_limit_clause(p)?),
            r => unreachable!("unexpected with_clause child rule {r:?}"),
        }
    }
    Ok(WithClause {
        items,
        where_clause,
        order_by,
        limit,
    })
}

fn parse_with_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    // with_expr = { with_or_expr }
    parse_with_or_expr(
        pair.into_inner()
            .next()
            .expect("with_expr has a with_or_expr"),
    )
}

fn parse_with_or_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_with_and_expr(
        parts
            .next()
            .expect("with_or_expr has at least one with_and_expr"),
    )?;
    for rest in parts {
        acc = WithExpr::Or(Box::new(acc), Box::new(parse_with_and_expr(rest)?));
    }
    Ok(acc)
}

fn parse_with_and_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_with_unary_expr(
        parts
            .next()
            .expect("with_and_expr has at least one with_unary_expr"),
    )?;
    for rest in parts {
        acc = WithExpr::And(Box::new(acc), Box::new(parse_with_unary_expr(rest)?));
    }
    Ok(acc)
}

fn parse_with_unary_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let inner = pair
        .into_inner()
        .next()
        .expect("with_unary_expr has one child");
    match inner.as_rule() {
        Rule::with_unary_expr => Ok(WithExpr::Not(Box::new(parse_with_unary_expr(inner)?))),
        Rule::with_comparison => parse_with_comparison(inner),
        Rule::with_expr => parse_with_expr(inner),
        r => unreachable!("unexpected with_unary_expr child rule {r:?}"),
    }
}

fn parse_with_comparison(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut inner = pair.into_inner();
    let lhs = parse_add_expr(inner.next().expect("with_comparison has an add_expr"))?;
    let op_pair = inner.next().expect("with_comparison has a compare_op");
    let op = parse_compare_op(op_pair);
    let literal = parse_literal(inner.next().expect("with_comparison has a literal"))?;
    Ok(WithExpr::Compare(lhs, op, literal))
}

fn parse_order_by_clause(pair: Pair<Rule>) -> Result<Vec<(ReturnExpr, SortDir)>, QueryError> {
    pair.into_inner()
        .filter(|c| c.as_rule() == Rule::sort_item)
        .map(parse_sort_item)
        .collect()
}

fn parse_limit_clause(pair: Pair<Rule>) -> Result<i64, QueryError> {
    let n_pair = pair.into_inner().next().expect("LIMIT has an int_literal");
    let n = n_pair
        .as_str()
        .parse::<i64>()
        .map_err(|_| QueryError::Syntax("invalid LIMIT value".into()))?;
    if n < 0 {
        return Err(QueryError::Syntax("LIMIT can't be negative".into()));
    }
    Ok(n)
}

/// Merges comma-separated patterns within one `MATCH` into a single linear
/// `Pattern`. Not a general cross-join — each subsequent pattern's start
/// variable must be exactly the previous pattern's last-introduced
/// variable (e.g. IS2's `MATCH (message)-[...]->(post:Post), (post)-[...]->
/// (person)`, where `post` is both the first pattern's end and the second's
/// start). Any labels/props the continuing pattern restates on that shared
/// variable are merged in as additional filters. Non-linear/branching
/// comma patterns (sharing a variable that isn't this exact splice point)
/// are rejected rather than silently mishandled.
fn splice_patterns(mut patterns: Vec<Pattern>) -> Result<Pattern, QueryError> {
    if patterns.is_empty() {
        return Err(QueryError::Syntax("MATCH requires a pattern".into()));
    }
    let mut combined = patterns.remove(0);
    for next in patterns {
        let Some(start_var) = next.start.var.clone() else {
            return Err(QueryError::Syntax(
                "a comma-separated MATCH pattern must start from a named variable".into(),
            ));
        };
        let last_var = combined
            .hops
            .last()
            .map(|(_, n)| n.var.clone())
            .unwrap_or_else(|| combined.start.var.clone());
        if last_var.as_deref() != Some(start_var.as_str()) {
            return Err(QueryError::Syntax(format!(
                "comma-separated MATCH pattern must continue from the previous pattern's last \
                 variable ('{}'), not '{start_var}' — general cross-joins aren't supported",
                last_var.unwrap_or_default()
            )));
        }
        let target = match combined.hops.last_mut() {
            Some((_, node)) => node,
            None => &mut combined.start,
        };
        target.labels.extend(next.start.labels);
        target.props.extend(next.start.props);
        combined.hops.extend(next.hops);
    }
    Ok(combined)
}

fn parse_sort_item(pair: Pair<Rule>) -> Result<(ReturnExpr, SortDir), QueryError> {
    let mut inner = pair.into_inner();
    let expr = parse_return_expr(inner.next().expect("sort_item has a return_expr"))?;
    let dir = match inner.next() {
        Some(d) if d.as_str().eq_ignore_ascii_case("desc") => SortDir::Desc,
        _ => SortDir::Asc,
    };
    Ok((expr, dir))
}

fn parse_tail_clause(pair: Pair<Rule>) -> Result<Tail, QueryError> {
    let inner = pair.into_inner().next().expect("tail_clause has one child");
    match inner.as_rule() {
        Rule::return_clause => {
            let (items, distinct) = parse_return_clause(inner)?;
            Ok(Tail::Return(items, distinct))
        }
        Rule::mutating_tail => parse_mutating_tail(inner),
        r => unreachable!("unexpected tail_clause child rule {r:?}"),
    }
}

/// `return_clause`'s children -- `(items, distinct)` -- shared by
/// `Tail::Return` itself (`parse_tail_clause`) and a trailing `ReturnTail`
/// after a mutating clause (`parse_mutating_tail`), same grammar rule
/// either way.
fn parse_return_clause(pair: Pair<Rule>) -> Result<(Vec<ReturnItem>, bool), QueryError> {
    let children: Vec<_> = pair.into_inner().collect();
    let distinct = children.iter().any(|p| p.as_rule() == Rule::distinct_kw);
    let items = children
        .into_iter()
        .filter(|p| p.as_rule() == Rule::return_item)
        .map(parse_return_item)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((items, distinct))
}

/// A mutating clause (`DETACH DELETE`/`DELETE`/`REMOVE`/`SET`/`CREATE`)
/// optionally followed by one trailing `RETURN` — see `mutating_tail`'s
/// grammar comment and `Tail::Delete`'s docs for the exact shape supported.
fn parse_mutating_tail(pair: Pair<Rule>) -> Result<Tail, QueryError> {
    let mut inner = pair.into_inner();
    let clause = inner.next().expect("mutating_tail has a mutating clause");
    let ret = inner
        .next()
        .map(|p| -> Result<ReturnTail, QueryError> {
            let (items, distinct) = parse_return_clause(p)?;
            Ok(ReturnTail { items, distinct })
        })
        .transpose()?;
    match clause.as_rule() {
        Rule::detach_delete_clause => {
            let vars = clause
                .into_inner()
                .filter(|p| p.as_rule() == Rule::identifier)
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(Tail::DetachDelete(vars, ret))
        }
        Rule::delete_clause => {
            let vars = clause
                .into_inner()
                .filter(|p| p.as_rule() == Rule::identifier)
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(Tail::Delete(vars, ret))
        }
        Rule::set_clause => {
            let items = clause
                .into_inner()
                .filter(|p| p.as_rule() == Rule::set_item)
                .map(parse_set_item)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Tail::Set(items, ret))
        }
        Rule::remove_clause => {
            let items = clause
                .into_inner()
                .filter(|p| p.as_rule() == Rule::remove_item)
                .map(parse_remove_item)
                .collect();
            Ok(Tail::Remove(items, ret))
        }
        Rule::create_stmt => Ok(Tail::Create(parse_create_patterns(clause)?, ret)),
        r => unreachable!("unexpected mutating_tail child rule {r:?}"),
    }
}

fn parse_set_item(pair: Pair<Rule>) -> Result<SetItem, QueryError> {
    let mut inner = pair.into_inner();
    let first = inner.next().expect("set_item has at least one child");
    match first.as_rule() {
        Rule::prop_access => {
            let literal_pair = inner
                .next()
                .expect("set_item's prop_access form has a literal");
            Ok(SetItem::Prop(
                parse_prop_access(first),
                parse_literal(literal_pair)?,
            ))
        }
        Rule::set_label_item => {
            let (var, labels) = parse_set_label_item(first);
            Ok(SetItem::Labels(var, labels))
        }
        r => unreachable!("unexpected set_item child rule {r:?}"),
    }
}

fn parse_set_label_item(pair: Pair<Rule>) -> (String, Vec<String>) {
    let mut inner = pair.into_inner();
    let var = inner
        .next()
        .expect("set_label_item has a var identifier")
        .as_str()
        .to_string();
    let labels = inner.map(|p| p.as_str().to_string()).collect();
    (var, labels)
}

fn parse_remove_item(pair: Pair<Rule>) -> RemoveItem {
    let inner = pair.into_inner().next().expect("remove_item has one child");
    match inner.as_rule() {
        Rule::prop_access => RemoveItem::Prop(parse_prop_access(inner)),
        Rule::set_label_item => {
            let (var, labels) = parse_set_label_item(inner);
            RemoveItem::Labels(var, labels)
        }
        r => unreachable!("unexpected remove_item child rule {r:?}"),
    }
}

fn parse_return_item(pair: Pair<Rule>) -> Result<ReturnItem, QueryError> {
    let mut inner = pair.into_inner();
    let expr_pair = inner.next().expect("return_item has a return_expr");
    let expr = parse_return_expr(expr_pair)?;
    let alias = inner.next().map(|p| p.as_str().to_string());
    Ok(ReturnItem { expr, alias })
}

fn parse_return_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let inner = pair
        .into_inner()
        .next()
        .expect("return_expr has one child (bool_or_expr)");
    parse_bool_or_expr(inner)
}

/// `kw_or`/`kw_xor`/`kw_and`/`kw_not` are atomic (see their grammar
/// comments -- word-boundary lookahead needs no implicit whitespace
/// inserted), so unlike the pre-existing inline-literal operators
/// elsewhere in this grammar, they DO produce their own `Pair` and show
/// up in `.into_inner()`. Filtered out here rather than relied on for
/// structure -- only the operand rule (`bool_xor_expr`/etc) matters for
/// building the left-fold.
fn parse_bool_or_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::bool_xor_expr);
    let mut lhs = parse_bool_xor_expr(
        inner
            .next()
            .expect("bool_or_expr has at least one bool_xor_expr"),
    )?;
    for rhs_pair in inner {
        lhs = ReturnExpr::Or(Box::new(lhs), Box::new(parse_bool_xor_expr(rhs_pair)?));
    }
    Ok(lhs)
}

fn parse_bool_xor_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::bool_and_expr);
    let mut lhs = parse_bool_and_expr(
        inner
            .next()
            .expect("bool_xor_expr has at least one bool_and_expr"),
    )?;
    for rhs_pair in inner {
        lhs = ReturnExpr::Xor(Box::new(lhs), Box::new(parse_bool_and_expr(rhs_pair)?));
    }
    Ok(lhs)
}

fn parse_bool_and_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::bool_not_expr);
    let mut lhs = parse_bool_not_expr(
        inner
            .next()
            .expect("bool_and_expr has at least one bool_not_expr"),
    )?;
    for rhs_pair in inner {
        lhs = ReturnExpr::And(Box::new(lhs), Box::new(parse_bool_not_expr(rhs_pair)?));
    }
    Ok(lhs)
}

/// `bool_not_expr = { (kw_not ~ bool_not_expr) | compare_expr }` -- right-
/// recursive, so (ignoring the atomic `kw_not` token when present) this
/// either has one `bool_not_expr` child (peel one `NOT`, recurse) or one
/// `compare_expr` child (base case).
fn parse_bool_not_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let inner = pair
        .into_inner()
        .find(|p| p.as_rule() != Rule::kw_not)
        .expect("bool_not_expr has a bool_not_expr or compare_expr child");
    match inner.as_rule() {
        Rule::bool_not_expr => Ok(ReturnExpr::Not(Box::new(parse_bool_not_expr(inner)?))),
        Rule::compare_expr => parse_compare_expr(inner),
        r => unreachable!("unexpected bool_not_expr child rule {r:?}"),
    }
}

/// `compare_expr = { add_expr ~ ((compare_op ~ add_expr)+ | is_null_suffix)? }`
/// -- either a chain of 1+ `compare_op ~ add_expr` pairs, a single
/// `is_null_suffix`, or neither. A chain folds into nested `And`s of
/// each *adjacent* pair (`a op0 b op1 c` -> `(a op0 b) AND (b op1 c)`,
/// real Cypher's own chained-comparison semantics) -- note this means a
/// middle operand like `b` is evaluated twice, harmless since no
/// `ReturnExpr` form has side effects.
fn parse_compare_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let first = parse_add_expr(
        inner
            .next()
            .expect("compare_expr has at least one add_expr"),
    )?;
    let Some(next) = inner.next() else {
        return Ok(first);
    };
    if next.as_rule() == Rule::is_null_suffix {
        return Ok(parse_is_null_suffix(next, first));
    }
    // A chain: `next` is the first compare_op, and the remaining pairs
    // alternate compare_op, add_expr, compare_op, add_expr, ...
    let mut operands = vec![first];
    let mut ops = vec![parse_compare_op(next)];
    loop {
        operands.push(parse_add_expr(
            inner.next().expect("compare_op has a following add_expr"),
        )?);
        match inner.next() {
            Some(op_pair) => ops.push(parse_compare_op(op_pair)),
            None => break,
        }
    }
    let mut pairs = operands.windows(2).zip(&ops).map(|(pair, op)| {
        ReturnExpr::Compare(Box::new(pair[0].clone()), *op, Box::new(pair[1].clone()))
    });
    let mut acc = pairs
        .next()
        .expect("a comparison chain has at least one pair");
    for next in pairs {
        acc = ReturnExpr::And(Box::new(acc), Box::new(next));
    }
    Ok(acc)
}

/// `is_null_suffix = { kw_is ~ kw_not? ~ kw_null }` -- `kw_not`'s
/// presence (as its own `Pair`, since it's atomic) distinguishes `IS
/// NULL` from `IS NOT NULL`.
fn parse_is_null_suffix(pair: Pair<Rule>, operand: ReturnExpr) -> ReturnExpr {
    let is_not = pair.into_inner().any(|p| p.as_rule() == Rule::kw_not);
    let is_null = ReturnExpr::IsNull(Box::new(operand));
    if is_not {
        ReturnExpr::Not(Box::new(is_null))
    } else {
        is_null
    }
}

fn parse_add_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let mut lhs = parse_mul_expr(inner.next().expect("add_expr has at least one mul_expr"))?;
    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => ArithOp::Add,
            "-" => ArithOp::Sub,
            other => unreachable!("unexpected add_op {other:?}"),
        };
        let rhs = parse_mul_expr(inner.next().expect("add_op has a following mul_expr"))?;
        lhs = ReturnExpr::Arith(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

fn parse_mul_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let mut lhs = parse_postfix_expr(
        inner
            .next()
            .expect("mul_expr has at least one postfix_expr"),
    )?;
    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "*" => ArithOp::Mul,
            "/" => ArithOp::Div,
            "%" => ArithOp::Mod,
            other => unreachable!("unexpected mul_op {other:?}"),
        };
        let rhs = parse_postfix_expr(inner.next().expect("mul_op has a following postfix_expr"))?;
        lhs = ReturnExpr::Arith(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

fn parse_postfix_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let mut base = parse_atom_expr(inner.next().expect("postfix_expr has one atom_expr"))?;
    for postfix in inner {
        // index_or_slice = { "[" ~ (slice_range | return_expr) ~ "]" }
        let child = postfix
            .into_inner()
            .next()
            .expect("index_or_slice has one child");
        base = match child.as_rule() {
            Rule::slice_range => {
                let (start, end) = parse_slice_bounds(child)?;
                ReturnExpr::Slice(Box::new(base), start, end)
            }
            Rule::return_expr => {
                ReturnExpr::Index(Box::new(base), Box::new(parse_return_expr(child)?))
            }
            r => unreachable!("unexpected index_or_slice child rule {r:?}"),
        };
    }
    Ok(base)
}

/// `slice_range = { return_expr? ~ ".." ~ return_expr? }` -- pest omits
/// the literal `..` (not a named rule), so its parsed children alone
/// don't say whether a single `return_expr` child is the start or the end
/// half of `start..`/`..end`. The pair's own source text (captured before
/// `.into_inner()` consumes it) does: split on the first `..`, and
/// whichever side is non-blank is present.
type OptionalReturnExpr = Option<Box<ReturnExpr>>;
type SliceBounds = (OptionalReturnExpr, OptionalReturnExpr);

fn parse_slice_bounds(slice_range_pair: Pair<Rule>) -> Result<SliceBounds, QueryError> {
    let raw = slice_range_pair.as_str().to_string();
    let dotdot_at = raw.find("..").expect("slice_range always contains ..");
    let has_start = !raw[..dotdot_at].trim().is_empty();
    let has_end = !raw[dotdot_at + 2..].trim().is_empty();
    let mut bounds = slice_range_pair.into_inner();
    let start = if has_start {
        Some(Box::new(parse_return_expr(
            bounds.next().expect("slice has a start return_expr"),
        )?))
    } else {
        None
    };
    let end = if has_end {
        Some(Box::new(parse_return_expr(
            bounds.next().expect("slice has an end return_expr"),
        )?))
    } else {
        None
    };
    Ok((start, end))
}

fn parse_atom_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let inner = pair.into_inner().next().expect("atom_expr has one child");
    match inner.as_rule() {
        Rule::case_expr => parse_case_expr(inner),
        Rule::quantifier_expr => parse_quantifier_expr(inner),
        Rule::function_call => parse_function_call(inner),
        Rule::list_expr => {
            let mut items = inner.into_inner().peekable();
            match items.peek().map(|p| p.as_rule()) {
                Some(Rule::list_comprehension) => parse_list_comprehension(items.next().unwrap()),
                _ => Ok(ReturnExpr::ListLit(
                    items
                        .map(parse_return_expr)
                        .collect::<Result<Vec<_>, _>>()?,
                )),
            }
        }
        // `parse_map_expr` (below `parse_node_pattern`/`parse_rel_pattern`
        // in this file) is shared with a `CREATE`/`MERGE` pattern's own
        // `{...}` prop map -- both are exactly the same grammar
        // production, `cypher.pest`'s `map_expr`.
        Rule::map_expr => parse_map_expr(inner),
        Rule::prop_access => Ok(ReturnExpr::Prop(parse_prop_access(inner))),
        Rule::literal => Ok(ReturnExpr::Lit(parse_literal(inner)?)),
        Rule::identifier => Ok(ReturnExpr::Var(inner.as_str().to_string())),
        // Parenthesized grouping -- `atom_expr`'s own `"(" ~ return_expr ~
        // ")"` alternative isn't a named sub-rule, so `return_expr` shows
        // up as a direct child here.
        Rule::return_expr => parse_return_expr(inner),
        r => unreachable!("unexpected atom_expr child rule {r:?}"),
    }
}

/// `list_comprehension = { identifier ~ ^"IN" ~ return_expr ~ (^"WHERE" ~
/// with_expr)? ~ ("|" ~ return_expr)? }` -- the `WHERE`/`|` clauses are
/// each independently optional, so the remaining children (after the
/// mandatory bound-variable identifier and source `return_expr`) are
/// distinguished by rule, not position.
/// `filter_expr = { identifier ~ ^"IN" ~ return_expr ~ (^"WHERE" ~
/// with_expr)? }` -- shared by `list_comprehension` and `quantifier_expr`,
/// so this returns the three parsed pieces rather than an `Expr` directly;
/// each caller wraps them into its own `ReturnExpr` variant.
type ParsedFilterExpr = (String, Box<ReturnExpr>, OptionalReturnExpr);

fn parse_filter_expr(pair: Pair<Rule>) -> Result<ParsedFilterExpr, QueryError> {
    let mut inner = pair.into_inner();
    let var = inner
        .next()
        .expect("filter_expr has a bound variable")
        .as_str()
        .to_string();
    let source = parse_return_expr(inner.next().expect("filter_expr has a source return_expr"))?;
    let where_clause = inner
        .next()
        .map(|w| parse_return_expr(w).map(Box::new))
        .transpose()?;
    Ok((var, Box::new(source), where_clause))
}

fn parse_list_comprehension(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let (var, source, where_clause) =
        parse_filter_expr(inner.next().expect("list_comprehension has a filter_expr"))?;
    let project = inner
        .next()
        .map(parse_return_expr)
        .transpose()?
        .map(Box::new);
    Ok(ReturnExpr::ListComp {
        var,
        source,
        where_clause,
        project,
    })
}

fn parse_quantifier_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let kind = match inner
        .next()
        .expect("quantifier_expr has a quantifier_kw")
        .as_str()
        .to_ascii_uppercase()
        .as_str()
    {
        "ALL" => QuantifierKind::All,
        "ANY" => QuantifierKind::Any,
        "NONE" => QuantifierKind::None,
        "SINGLE" => QuantifierKind::Single,
        other => unreachable!("unexpected quantifier_kw {other:?}"),
    };
    let (var, source, where_clause) =
        parse_filter_expr(inner.next().expect("quantifier_expr has a filter_expr"))?;
    Ok(ReturnExpr::Quantifier {
        kind,
        var,
        source,
        where_clause,
    })
}

fn parse_case_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let test = parse_return_expr(inner.next().expect("case_expr has a test expr"))?;
    let mut whens = Vec::new();
    let mut else_ = None;
    for p in inner {
        match p.as_rule() {
            Rule::case_when => {
                let mut when_inner = p.into_inner();
                let when =
                    parse_return_expr(when_inner.next().expect("case_when has a WHEN expr"))?;
                let then =
                    parse_return_expr(when_inner.next().expect("case_when has a THEN expr"))?;
                whens.push((when, then));
            }
            // The only other possible child is the trailing ELSE return_expr.
            _ => else_ = Some(Box::new(parse_return_expr(p)?)),
        }
    }
    Ok(ReturnExpr::Case {
        test: Some(Box::new(test)),
        whens,
        else_,
    })
}

fn parse_function_call(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .expect("function_call has a name")
        .as_str()
        .to_string();
    let call_args = inner.next().expect("function_call has call_args");
    let is_star = call_args.as_str().trim() == "*";
    if is_star {
        if !name.eq_ignore_ascii_case("count") {
            return Err(QueryError::Syntax(format!(
                "'{name}(*)' isn't valid — '*' is only meaningful for count(*)"
            )));
        }
        return Ok(ReturnExpr::CountStar);
    }
    let mut distinct = false;
    let mut args = Vec::new();
    for p in call_args.into_inner() {
        match p.as_rule() {
            Rule::distinct_kw => distinct = true,
            _ => args.push(parse_return_expr(p)?),
        }
    }
    if distinct && !is_aggregate_name(&name) {
        return Err(QueryError::Syntax(format!(
            "'{name}(DISTINCT ...)' isn't valid — DISTINCT is only meaningful inside an aggregate function"
        )));
    }
    Ok(ReturnExpr::Call {
        name,
        args,
        distinct,
    })
}

fn parse_prop_access(pair: Pair<Rule>) -> PropAccess {
    let mut inner = pair.into_inner();
    let var = inner
        .next()
        .expect("prop_access has a var")
        .as_str()
        .to_string();
    let prop = inner
        .next()
        .expect("prop_access has a prop")
        .as_str()
        .to_string();
    PropAccess { var, prop }
}

fn parse_pattern(pair: Pair<Rule>) -> Result<Pattern, QueryError> {
    let mut inner = pair.into_inner();
    let start = parse_node_pattern(inner.next().expect("pattern has a start node"))?;
    let mut hops = Vec::new();
    while let Some(rel_pair) = inner.next() {
        let node_pair = inner
            .next()
            .ok_or_else(|| QueryError::Syntax("dangling relationship in pattern".into()))?;
        hops.push((parse_rel_pattern(rel_pair)?, parse_node_pattern(node_pair)?));
    }
    Ok(Pattern { start, hops })
}

fn parse_node_pattern(pair: Pair<Rule>) -> Result<NodePattern, QueryError> {
    let mut var = None;
    let mut labels = Vec::new();
    let mut props = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::node_var => var = Some(p.as_str().to_string()),
            Rule::node_label => labels.push(
                p.into_inner()
                    .next()
                    .expect("node_label has an identifier")
                    .as_str()
                    .to_string(),
            ),
            Rule::map_expr => props = parse_map_expr_as_props(p)?,
            r => unreachable!("unexpected node_pattern child rule {r:?}"),
        }
    }
    Ok(NodePattern { var, labels, props })
}

fn parse_rel_pattern(pair: Pair<Rule>) -> Result<RelPattern, QueryError> {
    let inner = pair.into_inner().next().expect("rel_pattern has one child");
    let direction = match inner.as_rule() {
        Rule::rel_right | Rule::rel_right_bare => RelDirection::Right,
        Rule::rel_left | Rule::rel_left_bare => RelDirection::Left,
        Rule::rel_either | Rule::rel_either_bare => RelDirection::Either,
        r => unreachable!("unexpected rel_pattern child rule {r:?}"),
    };
    let mut var = None;
    let mut rel_type = None;
    let mut props = Vec::new();
    let mut hop_range = None;
    for p in inner.into_inner() {
        match p.as_rule() {
            Rule::rel_var => var = Some(p.as_str().to_string()),
            Rule::rel_type => {
                rel_type = Some(
                    p.into_inner()
                        .next()
                        .expect("rel_type has an identifier")
                        .as_str()
                        .to_string(),
                )
            }
            Rule::rel_range => hop_range = Some(parse_rel_range(p.as_str())?),
            Rule::map_expr => props = parse_map_expr_as_props(p)?,
            r => unreachable!("unexpected rel_right/rel_left/rel_either child rule {r:?}"),
        }
    }
    Ok(RelPattern {
        var,
        rel_type,
        props,
        direction,
        hop_range,
    })
}

/// Parses the raw `rel_range` text (`*`, `*N`, `*N..`, `*N..M`, `*..M`)
/// directly rather than via sub-rules, since the `..` literal produces no
/// child `Pair` to structurally distinguish "*N" (exact) from "*N.." (N or
/// more).
fn parse_rel_range(text: &str) -> Result<(u32, Option<u32>), QueryError> {
    let rest = &text[1..]; // strip leading '*'
    if rest.is_empty() {
        return Ok((0, None));
    }
    if let Some(idx) = rest.find("..") {
        let min_str = &rest[..idx];
        let max_str = &rest[idx + 2..];
        let min = if min_str.is_empty() {
            0
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

/// A node/relationship pattern's `{...}` prop map -- same `map_expr`
/// grammar rule as a general map-literal expression (see `parse_map_expr`
/// below), just re-shaped into the `Vec<(String, ReturnExpr)>` form
/// `NodePattern`/`RelPattern` carry rather than a `ReturnExpr::MapLit`.
fn parse_map_expr_as_props(pair: Pair<Rule>) -> Result<Vec<(String, ReturnExpr)>, QueryError> {
    let ReturnExpr::MapLit(entries) = parse_map_expr(pair)? else {
        unreachable!("parse_map_expr always returns MapLit")
    };
    Ok(entries)
}

/// `{key: <expr>, ...}` -- shared by `atom_expr`'s `map_expr` alternative
/// (a map used as an ordinary expression) and node/relationship pattern
/// prop maps (via `parse_map_expr_as_props` above), since both are
/// exactly the same grammar production (`cypher.pest`'s `map_expr`).
fn parse_map_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let entries = pair
        .into_inner()
        .map(|p| {
            let mut inner = p.into_inner();
            let key = inner.next().expect("map_kv has a key").as_str().to_string();
            let value = parse_return_expr(inner.next().expect("map_kv has a value"))?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, QueryError>>()?;
    Ok(ReturnExpr::MapLit(entries))
}

/// Resolves `\`-escapes in a `string_literal`'s already-quote-stripped
/// inner text. The grammar accepts any `\`-prefixed char (see
/// `cypher.pest`'s comment); only a fixed recognized set actually means
/// something -- an unrecognized escape (e.g. `\q`) errors here rather
/// than silently dropping the backslash or passing it through, matching
/// this codebase's stance elsewhere (error on an untested shape, don't
/// guess). No `\uXXXX` unicode escapes -- not needed yet, noted as a gap
/// in the README alongside the other documented Cypher-coverage gaps.
fn unescape_string(s: &str) -> Result<String, QueryError> {
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

fn parse_literal(pair: Pair<Rule>) -> Result<Literal, QueryError> {
    let inner = pair.into_inner().next().expect("literal has one child");
    Ok(match inner.as_rule() {
        Rule::int_literal => Literal::Int(
            inner
                .as_str()
                .parse()
                .map_err(|_| QueryError::Syntax("invalid integer literal".into()))?,
        ),
        Rule::float_literal => Literal::Float(
            inner
                .as_str()
                .parse()
                .map_err(|_| QueryError::Syntax("invalid float literal".into()))?,
        ),
        Rule::string_literal => {
            let s = inner.as_str();
            Literal::String(unescape_string(&s[1..s.len() - 1])?)
        }
        Rule::bool_literal => Literal::Bool(inner.as_str().eq_ignore_ascii_case("true")),
        Rule::null_literal => Literal::Null,
        Rule::param => {
            let name = inner
                .into_inner()
                .next()
                .expect("param has an identifier")
                .as_str()
                .to_string();
            Literal::Param(name)
        }
        r => unreachable!("unexpected literal child rule {r:?}"),
    })
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, QueryError> {
    // expr = { or_expr }
    parse_or_expr(pair.into_inner().next().expect("expr has an or_expr"))
}

fn parse_or_expr(pair: Pair<Rule>) -> Result<Expr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_and_expr(parts.next().expect("or_expr has at least one and_expr"))?;
    for rest in parts {
        acc = Expr::Or(Box::new(acc), Box::new(parse_and_expr(rest)?));
    }
    Ok(acc)
}

fn parse_and_expr(pair: Pair<Rule>) -> Result<Expr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_unary_expr(parts.next().expect("and_expr has at least one unary_expr"))?;
    for rest in parts {
        acc = Expr::And(Box::new(acc), Box::new(parse_unary_expr(rest)?));
    }
    Ok(acc)
}

fn parse_unary_expr(pair: Pair<Rule>) -> Result<Expr, QueryError> {
    let inner = pair.into_inner().next().expect("unary_expr has one child");
    match inner.as_rule() {
        Rule::unary_expr => Ok(Expr::Not(Box::new(parse_unary_expr(inner)?))),
        Rule::is_null_expr => Ok(parse_is_null_expr(inner)),
        Rule::comparison => parse_comparison(inner),
        Rule::expr => parse_expr(inner),
        r => unreachable!("unexpected unary_expr child rule {r:?}"),
    }
}

fn parse_comparison(pair: Pair<Rule>) -> Result<Expr, QueryError> {
    let mut inner = pair.into_inner();
    let prop_access = parse_prop_access(inner.next().expect("comparison has a prop_access"));
    let op = parse_compare_op(inner.next().expect("comparison has a compare_op"));
    let literal = parse_literal(inner.next().expect("comparison has a literal"))?;
    Ok(Expr::Compare(prop_access, op, literal))
}

/// `is_null_expr = { prop_access ~ is_null_suffix }` -- `is_null_suffix`'s
/// `kw_not` presence (as its own `Pair`, since it's atomic) distinguishes
/// `IS NULL` from `IS NOT NULL`, same as the `ReturnExpr` counterpart.
fn parse_is_null_expr(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let prop_access = parse_prop_access(inner.next().expect("is_null_expr has a prop_access"));
    let suffix = inner.next().expect("is_null_expr has an is_null_suffix");
    let is_not = suffix.into_inner().any(|p| p.as_rule() == Rule::kw_not);
    let is_null = Expr::IsNull(prop_access);
    if is_not {
        Expr::Not(Box::new(is_null))
    } else {
        is_null
    }
}

fn parse_compare_op(pair: Pair<Rule>) -> CompareOp {
    // `STARTS WITH`/`ENDS WITH` are two separate keyword tokens in the
    // grammar (so any amount of whitespace between them matches, same as
    // `DETACH DELETE`) -- normalize before matching so the exact source
    // spacing/casing doesn't matter.
    let normalized = pair
        .as_str()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_uppercase();
    match normalized.as_str() {
        "=" => CompareOp::Eq,
        "<>" => CompareOp::Ne,
        "<" => CompareOp::Lt,
        "<=" => CompareOp::Le,
        ">" => CompareOp::Gt,
        ">=" => CompareOp::Ge,
        "STARTS WITH" => CompareOp::StartsWith,
        "ENDS WITH" => CompareOp::EndsWith,
        "CONTAINS" => CompareOp::Contains,
        other => unreachable!("unexpected compare_op {other:?}"),
    }
}
