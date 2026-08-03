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

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let inner = pair.into_inner().next().expect("statement has one child");
    match inner.as_rule() {
        Rule::create_stmt => parse_create_stmt(inner),
        Rule::match_stmt => parse_match_stmt(inner),
        r => unreachable!("unexpected statement child rule {r:?}"),
    }
}

fn parse_create_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let patterns = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::pattern)
        .map(parse_pattern)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Statement::Create(patterns))
}

fn parse_match_stmt(pair: Pair<Rule>) -> Result<Statement, QueryError> {
    let mut pattern = None;
    let mut where_clause = None;
    let mut tail = None;
    let mut limit = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::pattern => pattern = Some(parse_pattern(p)?),
            Rule::where_clause => {
                let expr_pair = p.into_inner().next().expect("WHERE has an expr");
                where_clause = Some(parse_expr(expr_pair)?);
            }
            Rule::tail_clause => tail = Some(parse_tail_clause(p)?),
            Rule::limit_clause => {
                let n_pair = p.into_inner().next().expect("LIMIT has an int_literal");
                let n = n_pair
                    .as_str()
                    .parse::<i64>()
                    .map_err(|_| QueryError::Parse("invalid LIMIT value".into()))?;
                limit = Some(n);
            }
            r => unreachable!("unexpected match_stmt child rule {r:?}"),
        }
    }
    Ok(Statement::Match {
        pattern: pattern.ok_or_else(|| QueryError::Parse("MATCH requires a pattern".into()))?,
        where_clause,
        tail: tail
            .ok_or_else(|| QueryError::Parse("MATCH requires RETURN/DELETE/SET".into()))?,
        limit,
    })
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
        Rule::prop_access => Ok(ReturnExpr::Prop(parse_prop_access(inner))),
        Rule::identifier => Ok(ReturnExpr::Var(inner.as_str().to_string())),
        Rule::literal => Ok(ReturnExpr::Lit(parse_literal(inner)?)),
        r => unreachable!("unexpected return_expr child rule {r:?}"),
    }
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
    let mut label = None;
    let mut props = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::node_var => var = Some(p.as_str().to_string()),
            Rule::node_label => {
                label = Some(p.into_inner().next().expect("node_label has an identifier").as_str().to_string())
            }
            Rule::prop_map => props = parse_prop_map(p)?,
            r => unreachable!("unexpected node_pattern child rule {r:?}"),
        }
    }
    Ok(NodePattern { var, label, props })
}

fn parse_rel_pattern(pair: Pair<Rule>) -> Result<RelPattern, QueryError> {
    let inner = pair.into_inner().next().expect("rel_pattern has one child");
    let direction = match inner.as_rule() {
        Rule::rel_right => RelDirection::Right,
        Rule::rel_left => RelDirection::Left,
        r => unreachable!("unexpected rel_pattern child rule {r:?}"),
    };
    let mut var = None;
    let mut rel_type = None;
    let mut props = Vec::new();
    for p in inner.into_inner() {
        match p.as_rule() {
            Rule::rel_var => var = Some(p.as_str().to_string()),
            Rule::rel_type => {
                rel_type = Some(p.into_inner().next().expect("rel_type has an identifier").as_str().to_string())
            }
            Rule::prop_map => props = parse_prop_map(p)?,
            r => unreachable!("unexpected rel_right/rel_left child rule {r:?}"),
        }
    }
    Ok(RelPattern {
        var,
        rel_type,
        props,
        direction,
    })
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
            Literal::String(s[1..s.len() - 1].to_string())
        }
        Rule::bool_literal => Literal::Bool(inner.as_str().eq_ignore_ascii_case("true")),
        Rule::null_literal => Literal::Null,
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
    let op_pair = inner.next().expect("comparison has a compare_op");
    let op = match op_pair.as_str() {
        "=" => CompareOp::Eq,
        "<>" => CompareOp::Ne,
        "<" => CompareOp::Lt,
        "<=" => CompareOp::Le,
        ">" => CompareOp::Gt,
        ">=" => CompareOp::Ge,
        other => unreachable!("unexpected compare_op {other:?}"),
    };
    let literal = parse_literal(inner.next().expect("comparison has a literal"))?;
    Ok(Expr::Compare(prop_access, op, literal))
}
