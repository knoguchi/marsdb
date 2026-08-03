use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::QueryError;

#[derive(Parser)]
#[grammar = "cypher.pest"]
struct CypherParser;

pub fn parse(input: &str) -> Result<Statement, QueryError> {
    let mut pairs = CypherParser::parse(Rule::query, input)
        .map_err(|e| QueryError::Parse(e.to_string()))?;
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
    let mut pairs = CypherParser::parse(Rule::queries, input)
        .map_err(|e| QueryError::Parse(e.to_string()))?;
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
        Rule::create_stmt => parse_create_stmt(inner),
        Rule::match_stmt => parse_match_stmt(inner),
        r => unreachable!("unexpected statement child rule {r:?}"),
    }
}

fn parse_create_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    Ok(Statement::Create(parse_create_patterns(pair)?))
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
        return Err(QueryError::Parse(
            "chaining past one WITH boundary in a single MATCH isn't supported yet".into(),
        ));
    }
    for i in 0..clauses.len() {
        let (QueryClause::Match(part), Some(QueryClause::Match(next))) = (&clauses[i], clauses.get(i + 1)) else {
            continue;
        };
        if part.with.is_none() && !next.optional {
            return Err(QueryError::Parse(
                "multiple MATCH clauses must be separated by WITH".into(),
            ));
        }
    }

    Ok(Statement::Match {
        clauses,
        tail: tail.ok_or_else(|| QueryError::Parse("MATCH requires RETURN/DELETE/SET".into()))?,
        order_by,
        limit,
    })
}

fn clause_with(clause: &QueryClause) -> Option<&WithClause> {
    match clause {
        QueryClause::Match(part) => part.with.as_ref(),
        QueryClause::Unwind(u) => u.with.as_ref(),
    }
}

fn parse_clause(pair: Pair<Rule>) -> Result<QueryClause, QueryError> {
    let inner = pair.into_inner().next().expect("clause has one child");
    match inner.as_rule() {
        Rule::match_part => Ok(QueryClause::Match(parse_match_part(inner)?)),
        Rule::unwind_clause => Ok(QueryClause::Unwind(parse_unwind_clause(inner)?)),
        r => unreachable!("unexpected clause child rule {r:?}"),
    }
}

fn parse_unwind_clause(pair: Pair<Rule>) -> Result<UnwindClause, QueryError> {
    let mut inner = pair.into_inner();
    let source = parse_unwind_source(inner.next().expect("unwind_clause has an unwind_source"))?;
    let var = inner.next().expect("unwind_clause has an AS identifier").as_str().to_string();
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
    let inner = pair.into_inner().next().expect("unwind_source has one child");
    match inner.as_rule() {
        Rule::list_literal => Ok(UnwindSource::List(
            inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::literal)
                .map(parse_literal)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::identifier => Ok(UnwindSource::Var(inner.as_str().to_string())),
        r => unreachable!("unexpected unwind_source child rule {r:?}"),
    }
}

fn parse_match_part(pair: Pair<Rule>) -> Result<QueryPart, QueryError> {
    let mut optional = false;
    let mut patterns = Vec::new();
    let mut where_clause = None;
    let mut with = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::match_keyword => {
                optional = p.as_str().to_ascii_uppercase().starts_with("OPTIONAL");
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
    Ok(QueryPart {
        optional,
        pattern,
        where_clause,
        with,
    })
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
    parse_with_or_expr(pair.into_inner().next().expect("with_expr has a with_or_expr"))
}

fn parse_with_or_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_with_and_expr(parts.next().expect("with_or_expr has at least one with_and_expr"))?;
    for rest in parts {
        acc = WithExpr::Or(Box::new(acc), Box::new(parse_with_and_expr(rest)?));
    }
    Ok(acc)
}

fn parse_with_and_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut parts = pair.into_inner();
    let mut acc = parse_with_unary_expr(parts.next().expect("with_and_expr has at least one with_unary_expr"))?;
    for rest in parts {
        acc = WithExpr::And(Box::new(acc), Box::new(parse_with_unary_expr(rest)?));
    }
    Ok(acc)
}

fn parse_with_unary_expr(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let inner = pair.into_inner().next().expect("with_unary_expr has one child");
    match inner.as_rule() {
        Rule::with_unary_expr => Ok(WithExpr::Not(Box::new(parse_with_unary_expr(inner)?))),
        Rule::with_comparison => parse_with_comparison(inner),
        Rule::with_expr => parse_with_expr(inner),
        r => unreachable!("unexpected with_unary_expr child rule {r:?}"),
    }
}

fn parse_with_comparison(pair: Pair<Rule>) -> Result<WithExpr, QueryError> {
    let mut inner = pair.into_inner();
    let lhs = parse_return_expr(inner.next().expect("with_comparison has a return_expr"))?;
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
    n_pair
        .as_str()
        .parse::<i64>()
        .map_err(|_| QueryError::Parse("invalid LIMIT value".into()))
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
        return Err(QueryError::Parse("MATCH requires a pattern".into()));
    }
    let mut combined = patterns.remove(0);
    for next in patterns {
        let Some(start_var) = next.start.var.clone() else {
            return Err(QueryError::Parse(
                "a comma-separated MATCH pattern must start from a named variable".into(),
            ));
        };
        let last_var = combined
            .hops
            .last()
            .map(|(_, n)| n.var.clone())
            .unwrap_or_else(|| combined.start.var.clone());
        if last_var.as_deref() != Some(start_var.as_str()) {
            return Err(QueryError::Parse(format!(
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
            let items = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::return_item)
                .map(parse_return_item)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Tail::Return(items))
        }
        Rule::detach_delete_clause => {
            let vars = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::identifier)
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(Tail::DetachDelete(vars))
        }
        Rule::delete_clause => {
            let vars = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::identifier)
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(Tail::Delete(vars))
        }
        Rule::set_clause => {
            let items = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::set_item)
                .map(parse_set_item)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Tail::Set(items))
        }
        Rule::create_stmt => Ok(Tail::Create(parse_create_patterns(inner)?)),
        r => unreachable!("unexpected tail_clause child rule {r:?}"),
    }
}

fn parse_set_item(pair: Pair<Rule>) -> Result<(PropAccess, Literal), QueryError> {
    let mut inner = pair.into_inner();
    let prop_access_pair = inner.next().expect("set_item has a prop_access");
    let literal_pair = inner.next().expect("set_item has a literal");
    Ok((parse_prop_access(prop_access_pair), parse_literal(literal_pair)?))
}

fn parse_return_item(pair: Pair<Rule>) -> Result<ReturnItem, QueryError> {
    let mut inner = pair.into_inner();
    let expr_pair = inner.next().expect("return_item has a return_expr");
    let expr = parse_return_expr(expr_pair)?;
    let alias = inner.next().map(|p| p.as_str().to_string());
    Ok(ReturnItem { expr, alias })
}

fn parse_return_expr(pair: Pair<Rule>) -> Result<ReturnExpr, QueryError> {
    let inner = pair.into_inner().next().expect("return_expr has one child");
    match inner.as_rule() {
        Rule::case_expr => parse_case_expr(inner),
        Rule::function_call => parse_function_call(inner),
        Rule::prop_access => Ok(ReturnExpr::Prop(parse_prop_access(inner))),
        Rule::literal => Ok(ReturnExpr::Lit(parse_literal(inner)?)),
        Rule::identifier => Ok(ReturnExpr::Var(inner.as_str().to_string())),
        r => unreachable!("unexpected return_expr child rule {r:?}"),
    }
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
                let when = parse_return_expr(when_inner.next().expect("case_when has a WHEN expr"))?;
                let then = parse_return_expr(when_inner.next().expect("case_when has a THEN expr"))?;
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
    let name = inner.next().expect("function_call has a name").as_str().to_string();
    let call_args = inner.next().expect("function_call has call_args");
    let is_star = call_args.as_str().trim() == "*";
    if is_star {
        if !name.eq_ignore_ascii_case("count") {
            return Err(QueryError::Parse(format!(
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
        return Err(QueryError::Parse(format!(
            "'{name}(DISTINCT ...)' isn't valid — DISTINCT is only meaningful inside an aggregate function"
        )));
    }
    Ok(ReturnExpr::Call { name, args, distinct })
}

fn parse_prop_access(pair: Pair<Rule>) -> PropAccess {
    let mut inner = pair.into_inner();
    let var = inner.next().expect("prop_access has a var").as_str().to_string();
    let prop = inner.next().expect("prop_access has a prop").as_str().to_string();
    PropAccess { var, prop }
}

fn parse_pattern(pair: Pair<Rule>) -> Result<Pattern, QueryError> {
    let mut inner = pair.into_inner();
    let start = parse_node_pattern(inner.next().expect("pattern has a start node"))?;
    let mut hops = Vec::new();
    loop {
        let Some(rel_pair) = inner.next() else { break };
        let node_pair = inner
            .next()
            .ok_or_else(|| QueryError::Parse("dangling relationship in pattern".into()))?;
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
            Rule::node_label => {
                labels.push(p.into_inner().next().expect("node_label has an identifier").as_str().to_string())
            }
            Rule::prop_map => props = parse_prop_map(p)?,
            r => unreachable!("unexpected node_pattern child rule {r:?}"),
        }
    }
    Ok(NodePattern { var, labels, props })
}

fn parse_rel_pattern(pair: Pair<Rule>) -> Result<RelPattern, QueryError> {
    let inner = pair.into_inner().next().expect("rel_pattern has one child");
    let direction = match inner.as_rule() {
        Rule::rel_right => RelDirection::Right,
        Rule::rel_left => RelDirection::Left,
        Rule::rel_either => RelDirection::Either,
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
                rel_type = Some(p.into_inner().next().expect("rel_type has an identifier").as_str().to_string())
            }
            Rule::rel_range => hop_range = Some(parse_rel_range(p.as_str())?),
            Rule::prop_map => props = parse_prop_map(p)?,
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
                .map_err(|_| QueryError::Parse("invalid variable-length min hop count".into()))?
        };
        let max = if max_str.is_empty() {
            None
        } else {
            Some(
                max_str
                    .parse()
                    .map_err(|_| QueryError::Parse("invalid variable-length max hop count".into()))?,
            )
        };
        Ok((min, max))
    } else {
        let n: u32 = rest
            .parse()
            .map_err(|_| QueryError::Parse("invalid variable-length hop count".into()))?;
        Ok((n, Some(n)))
    }
}

fn parse_prop_map(pair: Pair<Rule>) -> Result<Vec<(String, Literal)>, QueryError> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::prop_kv)
        .map(|p| {
            let mut inner = p.into_inner();
            let key = inner.next().expect("prop_kv has a key").as_str().to_string();
            let value = parse_literal(inner.next().expect("prop_kv has a value"))?;
            Ok((key, value))
        })
        .collect()
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
                return Err(QueryError::Parse(format!("unrecognized string escape '\\{other}'")))
            }
            None => return Err(QueryError::Parse("string ends with a trailing '\\'".into())),
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
                .map_err(|_| QueryError::Parse("invalid integer literal".into()))?,
        ),
        Rule::float_literal => Literal::Float(
            inner
                .as_str()
                .parse()
                .map_err(|_| QueryError::Parse("invalid float literal".into()))?,
        ),
        Rule::string_literal => {
            let s = inner.as_str();
            Literal::String(unescape_string(&s[1..s.len() - 1])?)
        }
        Rule::bool_literal => Literal::Bool(inner.as_str().eq_ignore_ascii_case("true")),
        Rule::null_literal => Literal::Null,
        Rule::param => {
            let name = inner.into_inner().next().expect("param has an identifier").as_str().to_string();
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

fn parse_compare_op(pair: Pair<Rule>) -> CompareOp {
    match pair.as_str() {
        "=" => CompareOp::Eq,
        "<>" => CompareOp::Ne,
        "<" => CompareOp::Lt,
        "<=" => CompareOp::Le,
        ">" => CompareOp::Gt,
        ">=" => CompareOp::Ge,
        other => unreachable!("unexpected compare_op {other:?}"),
    }
}
