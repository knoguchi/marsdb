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

use crate::ast::{Literal, NodePattern, Pattern, QueryPart, RelDirection, RelPattern};
use crate::error::QueryError;
use crate::generated::cypherparser::{
    BoolLitContext, BoolLitContextAttrs, CharLitContext, CharLitContextAttrs, LiteralContext,
    LiteralContextAttrs, MatchStContext, MatchStContextAttrs, NodeLabelsContextAttrs,
    NodePatternContext, NodePatternContextAttrs, NumLitContext, NumLitContextAttrs,
    PatternContextAttrs, PatternElemChainContextAttrs, PatternElemContext, PatternElemContextAttrs,
    PatternPartContextAttrs, PatternWhereContextAttrs, RelationDetailContext,
    RelationDetailContextAttrs, RelationshipPatternContext, RelationshipPatternContextAttrs,
    RelationshipTypesContextAttrs, StringLitContext, StringLitContextAttrs, SymbolContextAll,
    SymbolContextAttrs,
};
use crate::generated::cypherparservisitor::CypherParserVisitorCompat;
use crate::parser::{
    group_into_linear_patterns, parse_int_literal, parse_rel_range, unescape_string,
    validate_named_path_pattern,
};
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat};

#[derive(Debug, Default)]
pub(crate) enum AstNode {
    #[default]
    None,
    Literal(Literal),
    NodePattern(NodePattern),
    RelPattern(RelPattern),
    Pattern(Pattern),
    QueryParts(Vec<QueryPart>),
    Err(QueryError),
}

macro_rules! ast_node_into {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(self) -> Result<$ty, QueryError> {
            match self {
                AstNode::$variant(v) => Ok(v),
                AstNode::Err(e) => Err(e),
                other => unreachable!("expected AstNode::{}, got {other:?}", stringify!($variant)),
            }
        }
    };
}

impl AstNode {
    ast_node_into!(into_literal, Literal, Literal);
    ast_node_into!(into_node_pattern, NodePattern, NodePattern);
    ast_node_into!(into_rel_pattern, RelPattern, RelPattern);
    ast_node_into!(into_pattern, Pattern, Pattern);
    ast_node_into!(into_query_parts, QueryParts, Vec<QueryPart>);
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

    fn visit_nodePattern(&mut self, ctx: &NodePatternContext<'input>) -> Self::Return {
        match self.build_node_pattern(ctx) {
            Ok(n) => AstNode::NodePattern(n),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_relationDetail(&mut self, ctx: &RelationDetailContext<'input>) -> Self::Return {
        match self.build_rel_detail(ctx) {
            Ok(r) => AstNode::RelPattern(r),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_relationshipPattern(
        &mut self,
        ctx: &RelationshipPatternContext<'input>,
    ) -> Self::Return {
        match self.build_relationship_pattern(ctx) {
            Ok(r) => AstNode::RelPattern(r),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_patternElem(&mut self, ctx: &PatternElemContext<'input>) -> Self::Return {
        match self.build_pattern_elem(ctx) {
            Ok(p) => AstNode::Pattern(p),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_matchSt(&mut self, ctx: &MatchStContext<'input>) -> Self::Return {
        match self.build_match_st(ctx) {
            Ok(parts) => AstNode::QueryParts(parts),
            Err(e) => AstNode::Err(e),
        }
    }
}

fn symbol_text(ctx: &SymbolContextAll) -> String {
    match ctx.ESC_LITERAL() {
        // `` `a weird name` `` -- strip the surrounding backticks.
        Some(t) => {
            let text = t.get_text();
            text[1..text.len() - 1].to_string()
        }
        None => ctx.get_text(),
    }
}

fn no_properties_yet<T>(properties_present: bool) -> Result<Vec<(String, T)>, QueryError> {
    if properties_present {
        // Property values are arbitrary expressions (`{x: 1 + 2}`), not
        // just literals -- deferred until the expression chain lands
        // later in Phase 2, rather than silently dropping them.
        Err(QueryError::Syntax(
            "pattern properties aren't supported by the ANTLR parser yet".into(),
        ))
    } else {
        Ok(Vec::new())
    }
}

impl AstBuilder {
    fn build_node_pattern(&mut self, ctx: &NodePatternContext) -> Result<NodePattern, QueryError> {
        let var = ctx.symbol().map(|s| symbol_text(&s));
        let labels = ctx
            .nodeLabels()
            .map(|nl| nl.name_all().iter().map(|n| n.get_text()).collect())
            .unwrap_or_default();
        let props = no_properties_yet(ctx.properties().is_some())?;
        Ok(NodePattern { var, labels, props })
    }

    fn build_rel_detail(&mut self, ctx: &RelationDetailContext) -> Result<RelPattern, QueryError> {
        let var = ctx.symbol().map(|s| symbol_text(&s));
        let rel_types = ctx
            .relationshipTypes()
            .map(|rt| rt.name_all().iter().map(|n| n.get_text()).collect())
            .unwrap_or_default();
        let props = no_properties_yet(ctx.properties().is_some())?;
        let hop_range = ctx
            .rangeLit()
            .map(|r| parse_rel_range(&r.get_text()))
            .transpose()?;
        Ok(RelPattern {
            var,
            rel_types,
            props,
            // Overwritten by `build_relationship_pattern`, the only
            // caller -- `relationDetail` itself (`[...]`) carries no
            // directionality, that's `<`/`>` on the surrounding
            // `relationshipPattern`.
            direction: RelDirection::Either,
            hop_range,
        })
    }

    fn build_relationship_pattern(
        &mut self,
        ctx: &RelationshipPatternContext,
    ) -> Result<RelPattern, QueryError> {
        let mut rel = match ctx.relationDetail() {
            Some(rd) => self.visit(&*rd).into_rel_pattern()?,
            None => RelPattern {
                var: None,
                rel_types: Vec::new(),
                props: Vec::new(),
                direction: RelDirection::Either,
                hop_range: None,
            },
        };
        rel.direction = if ctx.LT().is_some() {
            RelDirection::Left
        } else if ctx.GT().is_some() {
            RelDirection::Right
        } else {
            RelDirection::Either
        };
        Ok(rel)
    }

    fn build_pattern_elem(&mut self, ctx: &PatternElemContext) -> Result<Pattern, QueryError> {
        if ctx.LPAREN().is_some() || !ctx.qppElemChain_all().is_empty() {
            return Err(QueryError::Syntax(
                "quantified path patterns aren't supported yet".into(),
            ));
        }
        let node_ctx = ctx
            .nodePattern()
            .expect("patternElem always starts with a nodePattern in the non-QPP alternative");
        let start = self.visit(&*node_ctx).into_node_pattern()?;
        let mut hops = Vec::new();
        for chain in ctx.patternElemChain_all() {
            let rel_ctx = chain
                .relationshipPattern()
                .expect("patternElemChain always has a relationshipPattern");
            let node_ctx = chain
                .nodePattern()
                .expect("patternElemChain always has a nodePattern");
            let rel = self.visit(&*rel_ctx).into_rel_pattern()?;
            let node = self.visit(&*node_ctx).into_node_pattern()?;
            hops.push((rel, node));
        }
        Ok(Pattern { start, hops })
    }

    /// Mirrors `parser.rs`'s `parse_match_part` -- comma-separated pattern
    /// parts splice into linear chains (shared-node merging) or split into
    /// separate `QueryPart`s (disjoint cross join) via
    /// `group_into_linear_patterns`, reused as-is.
    fn build_match_st(&mut self, ctx: &MatchStContext) -> Result<Vec<QueryPart>, QueryError> {
        let optional = ctx.OPTIONAL().is_some();
        let pw = ctx
            .patternWhere()
            .expect("matchSt always has a patternWhere");
        if pw.where_().is_some() {
            return Err(QueryError::Syntax(
                "WHERE isn't supported by the ANTLR parser yet".into(),
            ));
        }
        let pattern_ctx = pw.pattern().expect("patternWhere always has a pattern");

        let mut path_var = None;
        let mut patterns = Vec::new();
        for part in pattern_ctx.patternPart_all() {
            let elem_ctx = part
                .patternElem()
                .expect("patternPart always has a patternElem");
            patterns.push(self.visit(&*elem_ctx).into_pattern()?);
            if part.ASSIGN().is_some() {
                if path_var.is_some() {
                    return Err(QueryError::Syntax(
                        "at most one comma-separated pattern part can have a named-path variable"
                            .into(),
                    ));
                }
                let symbol_ctx = part
                    .symbol()
                    .expect("patternPart with ASSIGN always has a symbol");
                path_var = Some(symbol_text(&symbol_ctx));
            }
        }

        let groups = group_into_linear_patterns(patterns)?;
        if groups.len() > 1 && path_var.is_some() {
            return Err(QueryError::Syntax(
                "a named path can't span a comma-separated cross join".into(),
            ));
        }
        if path_var.is_some() {
            validate_named_path_pattern(&groups[0])?;
        }

        // `where_clause`/`with` are unconditionally `None` here -- WHERE
        // already errors out above (before `groups` even exists), and
        // this grammar's `matchSt` has no trailing WITH of its own (that's
        // a separate clause in the statement's clause list). Once WHERE
        // support lands, only the *last* group should carry it, same as
        // `parser.rs`'s `parse_match_part`.
        Ok(groups
            .into_iter()
            .enumerate()
            .map(|(i, pattern)| QueryPart {
                optional,
                path_var: if i == 0 { path_var.clone() } else { None },
                // shortestPath()/allShortestPaths() aren't implemented by
                // the vendored grammar (antlr/grammars-v4/cypher) at all
                // -- a real gap, not deferred-for-now like WHERE/props.
                shortest_path: false,
                pattern,
                where_clause: None,
                with: None,
            })
            .collect())
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

    fn parse_pattern(input: &str) -> Result<Pattern, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .patternElem()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `patternElem`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_pattern()
    }

    fn parse_match(input: &str) -> Result<Vec<QueryPart>, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .matchSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `matchSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_query_parts()
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

    #[test]
    fn single_node() {
        let p = parse_pattern("(a:Person)").unwrap();
        assert_eq!(p.start.var.as_deref(), Some("a"));
        assert_eq!(p.start.labels, vec!["Person".to_string()]);
        assert!(p.hops.is_empty());
    }

    #[test]
    fn anonymous_node() {
        let p = parse_pattern("()").unwrap();
        assert_eq!(p.start.var, None);
        assert!(p.start.labels.is_empty());
    }

    #[test]
    fn multiple_labels() {
        let p = parse_pattern("(a:Person:Employee)").unwrap();
        assert_eq!(
            p.start.labels,
            vec!["Person".to_string(), "Employee".to_string()]
        );
    }

    #[test]
    fn escaped_identifier() {
        let p = parse_pattern("(`weird name`)").unwrap();
        assert_eq!(p.start.var.as_deref(), Some("weird name"));
    }

    #[test]
    fn directions() {
        assert_eq!(
            parse_pattern("(a)-->(b)").unwrap().hops[0].0.direction,
            RelDirection::Right
        );
        assert_eq!(
            parse_pattern("(a)<--(b)").unwrap().hops[0].0.direction,
            RelDirection::Left
        );
        assert_eq!(
            parse_pattern("(a)--(b)").unwrap().hops[0].0.direction,
            RelDirection::Either
        );
    }

    #[test]
    fn rel_type_and_var() {
        let p = parse_pattern("(a)-[r:KNOWS]->(b)").unwrap();
        let (rel, node) = &p.hops[0];
        assert_eq!(rel.var.as_deref(), Some("r"));
        assert_eq!(rel.rel_types, vec!["KNOWS".to_string()]);
        assert_eq!(node.var.as_deref(), Some("b"));
        assert_eq!(rel.hop_range, None);
    }

    #[test]
    fn multiple_rel_types() {
        let p = parse_pattern("(a)-[:KNOWS|LIKES]->(b)").unwrap();
        assert_eq!(
            p.hops[0].0.rel_types,
            vec!["KNOWS".to_string(), "LIKES".to_string()]
        );
    }

    #[test]
    fn var_length_bounds() {
        // Exercises the DIGIT/ID lexer fixes end to end -- `*0`/`*2` used
        // to hard-fail before those were fixed upstream.
        assert_eq!(
            parse_pattern("(a)-[*0]->(b)").unwrap().hops[0].0.hop_range,
            Some((0, Some(0)))
        );
        assert_eq!(
            parse_pattern("(a)-[*2]->(b)").unwrap().hops[0].0.hop_range,
            Some((2, Some(2)))
        );
        assert_eq!(
            parse_pattern("(a)-[*1..3]->(b)").unwrap().hops[0]
                .0
                .hop_range,
            Some((1, Some(3)))
        );
        assert_eq!(
            parse_pattern("(a)-[*]->(b)").unwrap().hops[0].0.hop_range,
            Some((1, None))
        );
    }

    #[test]
    fn multi_hop_chain() {
        let p = parse_pattern("(a)-[:KNOWS]->(b)<-[:LIKES]-(c)").unwrap();
        assert_eq!(p.hops.len(), 2);
        assert_eq!(p.hops[0].0.direction, RelDirection::Right);
        assert_eq!(p.hops[1].0.direction, RelDirection::Left);
    }

    #[test]
    fn properties_not_yet_supported() {
        assert!(parse_pattern("(a {name: 'x'})").is_err());
    }

    #[test]
    fn simple_match() {
        let parts = parse_match("MATCH (a:Person)-[:KNOWS]->(b)").unwrap();
        assert_eq!(parts.len(), 1);
        assert!(!parts[0].optional);
        assert_eq!(parts[0].path_var, None);
        assert_eq!(parts[0].pattern.start.var.as_deref(), Some("a"));
        assert_eq!(parts[0].pattern.hops.len(), 1);
    }

    #[test]
    fn optional_match() {
        let parts = parse_match("OPTIONAL MATCH (a)").unwrap();
        assert!(parts[0].optional);
    }

    #[test]
    fn named_path() {
        let parts = parse_match("MATCH p = (a)-->(b)").unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].path_var.as_deref(), Some("p"));
    }

    #[test]
    fn comma_pattern_shared_node_merges_into_one_linear_chain() {
        // `(a)-->(b), (b)-->(c)` shares `b` -- one QueryPart, three-node
        // chain, not two disjoint ones. Exercises group_into_linear_patterns.
        let parts = parse_match("MATCH (a)-->(b), (b)-->(c)").unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].pattern.hops.len(), 2);
    }

    #[test]
    fn comma_pattern_disjoint_becomes_multiple_query_parts() {
        let parts = parse_match("MATCH (a), (b)").unwrap();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn named_path_over_disjoint_cross_join_errors() {
        assert!(parse_match("MATCH p = (a), (b)").is_err());
    }

    #[test]
    fn named_path_over_variable_length_errors() {
        assert!(parse_match("MATCH p = (a)-[*1..3]->(b)").is_err());
    }

    #[test]
    fn where_not_yet_supported() {
        assert!(parse_match("MATCH (a) WHERE a.x = 1").is_err());
    }
}
