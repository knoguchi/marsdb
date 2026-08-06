// Not wired into a public entry point yet -- only this file's own tests
// exercise `Visitor::visit` until later Phase 2 increments need more of
// `AstNode`. Remove once that happens.
#![allow(dead_code)]

//! ANTLR-based AST builder (Phase 2, mars-nog) -- replaces `parser.rs`'s
//! pest-tree-walk once complete, built incrementally clause by clause.
//! Not wired into `parse`/`parse_many` yet.
//!
//! Implements the generated `CypherParserVisitorCompat` trait rather than
//! manually walking context accessors: ANTLR's own `accept()`/`visit()`
//! double-dispatch already routes to the right `visit_X` method for
//! whichever grammar alternative is actually present, so alternation
//! (`literal : boolLit | numLit | NULL_W | stringLit | charLit | listLit
//! | mapLit`) doesn't need a hand-written `if let Some(x) = ctx.boolLit()
//! ... else if ...` chain -- only `visit_literal` needs a one-line manual
//! check, for the bare `NULL_W` terminal alternative specifically (a
//! terminal has no grammar-rule `visit_X` hook of its own to override).
//!
//! `Return` (via [`AstNode`]) is one shared enum across the whole
//! visitor -- required by `ParseTreeVisitorCompat`, which supports
//! exactly one `Return` type for the entire tree walk, not a per-rule
//! type. Grows a variant per AST node kind as later increments need it.

use crate::ast::Literal;
use crate::error::QueryError;
use crate::generated::cypherparser::{
    BoolLitContext, BoolLitContextAttrs, CharLitContext, CharLitContextAttrs, LiteralContext,
    LiteralContextAttrs, NumLitContext, NumLitContextAttrs, StringLitContext,
    StringLitContextAttrs,
};
use crate::generated::cypherparservisitor::CypherParserVisitorCompat;
use crate::parser::{parse_int_literal, unescape_string};
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat};

#[derive(Debug, Default)]
pub(crate) enum AstNode {
    #[default]
    None,
    Literal(Literal),
    Err(QueryError),
}

impl AstNode {
    fn into_literal(self) -> Result<Literal, QueryError> {
        match self {
            AstNode::Literal(l) => Ok(l),
            AstNode::Err(e) => Err(e),
            other => unreachable!("expected AstNode::Literal, got {other:?}"),
        }
    }
}

pub(crate) struct AstBuilder {
    result: AstNode,
}

impl AstBuilder {
    pub(crate) fn new() -> Self {
        AstBuilder {
            result: AstNode::default(),
        }
    }
}

impl<'input> ParseTreeVisitorCompat<'input> for AstBuilder {
    type Node = crate::generated::cypherparser::CypherParserContextType;
    type Return = AstNode;

    fn temp_result(&mut self) -> &mut Self::Return {
        &mut self.result
    }
}

impl<'input> CypherParserVisitorCompat<'input> for AstBuilder {
    fn visit_literal(&mut self, ctx: &LiteralContext<'input>) -> Self::Return {
        // The only alternative that's a bare terminal, not a sub-rule --
        // no `visit_X` grammar-rule hook exists to override for it, so it
        // needs the one manual check `visit_children`'s default dispatch
        // can't cover on its own.
        if ctx.NULL_W().is_some() {
            return AstNode::Literal(Literal::Null);
        }
        self.visit_children(ctx)
    }

    fn visit_boolLit(&mut self, ctx: &BoolLitContext<'input>) -> Self::Return {
        AstNode::Literal(Literal::Bool(ctx.TRUE().is_some()))
    }

    /// The lexer's `DIGIT` token covers hex/octal/decimal integers *and*
    /// floats in one token (`DIGIT : HexDigits | OctalDigits | Digits |
    /// FLOAT;`), unlike pest's grammar which split `int_literal`/
    /// `float_literal` into separate rules -- so int-vs-float is decided
    /// from the raw text here instead of from which sub-rule matched.
    /// Hex/octal integers can't contain `.`/exponent/`f`/`d` at all (the
    /// lexer wouldn't have matched `DIGIT` as those alternatives if they
    /// did), so checking for those unconditionally is safe and doesn't
    /// misfire on e.g. `0xE` (a hex digit `E`, not a float exponent).
    fn visit_numLit(&mut self, ctx: &NumLitContext<'input>) -> Self::Return {
        let text = ctx
            .DIGIT()
            .expect("numLit context always has a DIGIT token")
            .get_text();
        let is_float = text.contains('.')
            || text.ends_with(['f', 'F', 'd', 'D'])
            || text
                .rfind(['e', 'E'])
                .is_some_and(|i| text[..i].chars().all(|c| c.is_ascii_digit() || c == '-'));
        let result = if is_float {
            text.parse::<f64>()
                .map_err(|_| QueryError::Syntax(format!("invalid float literal '{text}'")))
                .and_then(|f| {
                    // `str::parse::<f64>()` silently returns `f64::INFINITY`
                    // for a magnitude beyond f64's representable range
                    // instead of erroring (`"1e999".parse::<f64>()` is
                    // `Ok(inf)`) -- real Cypher requires this to be a
                    // compile-time error (TCK Literals5 [27],
                    // `FloatingPointOverflow`).
                    if f.is_infinite() {
                        Err(QueryError::Syntax(format!(
                            "float literal '{text}' is too large to represent"
                        )))
                    } else {
                        Ok(Literal::Float(f))
                    }
                })
        } else {
            parse_int_literal(&text).map(Literal::Int)
        };
        match result {
            Ok(lit) => AstNode::Literal(lit),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_stringLit(&mut self, ctx: &StringLitContext<'input>) -> Self::Return {
        let text = ctx
            .STRING_LITERAL()
            .expect("stringLit context always has a STRING_LITERAL token")
            .get_text();
        match unescape_string(&text[1..text.len() - 1]) {
            Ok(s) => AstNode::Literal(Literal::String(s)),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_charLit(&mut self, ctx: &CharLitContext<'input>) -> Self::Return {
        let text = ctx
            .CHAR_LITERAL()
            .expect("charLit context always has a CHAR_LITERAL token")
            .get_text();
        match unescape_string(&text[1..text.len() - 1]) {
            Ok(s) => AstNode::Literal(Literal::String(s)),
            Err(e) => AstNode::Err(e),
        }
    }
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
        AstBuilder::new().visit(&*ctx).into_literal()
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
