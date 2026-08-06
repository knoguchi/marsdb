// Not wired into a public entry point yet -- only this file's own tests
// use these functions until later Phase 2 increments call them from a
// higher grammar rule. Remove once that happens.
#![allow(dead_code)]

//! ANTLR-based AST builder (Phase 2, mars-nog) -- replaces `parser.rs`'s
//! pest-tree-walk once complete, built incrementally clause by clause.
//! Not wired into `parse`/`parse_many` yet.
//!
//! ANTLR4's Rust target only supports one shared `Return` type across an
//! entire `Visitor` implementation (`ParseTreeVisitorCompat::Return`),
//! which doesn't fit a heterogeneous AST (`Literal` vs `Expr` vs
//! `Statement` ...) without an enum wrapping every possible node type and
//! unwrapping it at every call site. Walking the generated `XContextAttrs`
//! accessors directly (this file's approach) mirrors `parser.rs`'s
//! existing one-fn-per-rule style and avoids that -- swap pest's
//! `Pair<Rule>` for ANTLR's generated `XContextAll`, keep the shape.

use crate::ast::Literal;
use crate::error::QueryError;
use crate::generated::cypherparser::{
    BoolLitContextAll, BoolLitContextAttrs, CharLitContextAll, CharLitContextAttrs,
    LiteralContextAll, LiteralContextAttrs, NumLitContextAll, NumLitContextAttrs,
    StringLitContextAll, StringLitContextAttrs,
};
use crate::parser::{parse_int_literal, unescape_string};
use antlr4rust::tree::ParseTree;

pub(crate) fn build_literal(ctx: &LiteralContextAll) -> Result<Literal, QueryError> {
    if let Some(b) = ctx.boolLit() {
        return Ok(build_bool_lit(&b));
    }
    if let Some(n) = ctx.numLit() {
        return build_num_lit(&n);
    }
    if ctx.NULL_W().is_some() {
        return Ok(Literal::Null);
    }
    if let Some(s) = ctx.stringLit() {
        return build_string_lit(&s);
    }
    if let Some(c) = ctx.charLit() {
        return build_char_lit(&c);
    }
    // listLit/mapLit are handled at the `Expr` level (`Expr::List`/
    // `Expr::Map`), not here -- `literal`'s other two alternatives.
    unreachable!("literal context has no boolLit/numLit/NULL_W/stringLit/charLit child")
}

fn build_bool_lit(ctx: &BoolLitContextAll) -> Literal {
    Literal::Bool(ctx.TRUE().is_some())
}

/// The lexer's `DIGIT` token covers hex/octal/decimal integers *and*
/// floats in one token (`DIGIT : HexDigits | OctalDigits | Digits |
/// FLOAT;`), unlike pest's grammar which split `int_literal`/
/// `float_literal` into separate rules -- so int-vs-float has to be
/// decided from the raw text here instead of from which sub-rule matched.
/// Hex/octal integers can't contain `.`/exponent/`f`/`d` at all (the
/// lexer wouldn't have matched `DIGIT` as those alternatives if they
/// did), so checking for those unconditionally is safe and doesn't
/// misfire on e.g. `0xE` (a hex digit `E`, not a float exponent).
fn build_num_lit(ctx: &NumLitContextAll) -> Result<Literal, QueryError> {
    let text = ctx
        .DIGIT()
        .expect("numLit context always has a DIGIT token")
        .get_text();
    let is_float = text.contains('.')
        || text.ends_with(['f', 'F', 'd', 'D'])
        || text
            .rfind(['e', 'E'])
            .is_some_and(|i| text[..i].chars().all(|c| c.is_ascii_digit() || c == '-'));
    if is_float {
        let f: f64 = text
            .parse()
            .map_err(|_| QueryError::Syntax(format!("invalid float literal '{text}'")))?;
        // `str::parse::<f64>()` silently returns `f64::INFINITY` for a
        // magnitude beyond f64's representable range instead of erroring
        // (`"1e999".parse::<f64>()` is `Ok(inf)`) -- real Cypher requires
        // this to be a compile-time error (TCK Literals5 [27],
        // `FloatingPointOverflow`).
        if f.is_infinite() {
            return Err(QueryError::Syntax(format!(
                "float literal '{text}' is too large to represent"
            )));
        }
        Ok(Literal::Float(f))
    } else {
        Ok(Literal::Int(parse_int_literal(&text)?))
    }
}

fn build_string_lit(ctx: &StringLitContextAll) -> Result<Literal, QueryError> {
    let text = ctx
        .STRING_LITERAL()
        .expect("stringLit context always has a STRING_LITERAL token")
        .get_text();
    Ok(Literal::String(unescape_string(&text[1..text.len() - 1])?))
}

fn build_char_lit(ctx: &CharLitContextAll) -> Result<Literal, QueryError> {
    let text = ctx
        .CHAR_LITERAL()
        .expect("charLit context always has a CHAR_LITERAL token")
        .get_text();
    Ok(Literal::String(unescape_string(&text[1..text.len() - 1])?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::cypherlexer::CypherLexer;
    use crate::generated::cypherparser::CypherParser;
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::InputStream;

    fn parse_literal_expr(input: &str) -> Result<Literal, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .literal()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `literal`: {e:?}"));
        build_literal(&ctx)
    }

    #[test]
    fn bool_literals() {
        assert_eq!(parse_literal_expr("true").unwrap(), Literal::Bool(true));
        assert_eq!(parse_literal_expr("FALSE").unwrap(), Literal::Bool(false));
    }

    #[test]
    fn null_literal() {
        assert_eq!(parse_literal_expr("null").unwrap(), Literal::Null);
    }

    #[test]
    fn decimal_int() {
        assert_eq!(parse_literal_expr("42").unwrap(), Literal::Int(42));
        assert_eq!(parse_literal_expr("007").unwrap(), Literal::Int(7));
    }

    #[test]
    fn hex_and_octal_int() {
        assert_eq!(parse_literal_expr("0x1A").unwrap(), Literal::Int(26));
        assert_eq!(parse_literal_expr("0o17").unwrap(), Literal::Int(15));
    }

    // `i64::MIN`'s two's-complement edge case (`-9223372036854775808`) is
    // exercised once sign-folding lands at the `unaryAddSubExpression`
    // level (see this file's module doc) -- unlike pest's `int_literal`,
    // which included an optional leading `-` in the literal token itself,
    // this grammar's `literal`/`numLit` never carries a sign at all; `-`
    // is strictly `unaryAddSubExpression`'s prefix operator, one level up.
    // `parse_int_literal` (reused from `parser.rs`) already handles the
    // two's-complement case correctly given a leading `-` in its input --
    // that part's covered; only the fold-sign-into-literal-vs-build-a-Neg-
    // node decision at the expression level remains.

    #[test]
    fn float_literals() {
        assert_eq!(parse_literal_expr("2.5").unwrap(), Literal::Float(2.5));
        assert_eq!(parse_literal_expr("1e10").unwrap(), Literal::Float(1e10));
        assert_eq!(parse_literal_expr(".5").unwrap(), Literal::Float(0.5));
    }

    #[test]
    fn float_overflow_errors() {
        assert!(parse_literal_expr("1e999").is_err());
    }

    #[test]
    fn string_and_char_literals() {
        assert_eq!(
            parse_literal_expr("\"hello\"").unwrap(),
            Literal::String("hello".to_string())
        );
        assert_eq!(
            parse_literal_expr("'a string with spaces and a hyphen-in-it'").unwrap(),
            Literal::String("a string with spaces and a hyphen-in-it".to_string())
        );
    }

    #[test]
    fn string_escapes() {
        assert_eq!(
            parse_literal_expr(r#"'line1\nline2'"#).unwrap(),
            Literal::String("line1\nline2".to_string())
        );
        assert_eq!(
            parse_literal_expr(r#"'é'"#).unwrap(),
            Literal::String("é".to_string())
        );
    }
}
