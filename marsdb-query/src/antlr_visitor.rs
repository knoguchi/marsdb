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

use crate::ast::{
    is_aggregate_name, ArithOp, CompareOp, Literal, NodePattern, Pattern, PropAccess, QueryPart,
    RelDirection, RelPattern, ReturnExpr, ReturnItem, SortDir, Tail,
};
use crate::error::QueryError;
use crate::generated::cypherparser::{
    AddSubExpressionContext, AndExpressionContext, AndExpressionContextAttrs, AtomContext,
    AtomContextAttrs, AtomicExpressionContext, AtomicExpressionContextAll,
    AtomicExpressionContextAttrs, BoolLitContext, BoolLitContextAttrs, CharLitContext,
    CharLitContextAttrs, ComparisonExpressionContext, ComparisonExpressionContextAttrs,
    ComparisonSignsContextAll, ComparisonSignsContextAttrs, CountAllContext,
    ExpressionChainContextAttrs, ExpressionContext, ExpressionContextAttrs,
    FunctionInvocationContext, FunctionInvocationContextAttrs, InvocationNameContextAll,
    InvocationNameContextAttrs, LimitStContextAttrs, ListExpressionContextAll,
    ListExpressionContextAttrs, LiteralContext, LiteralContextAttrs, MatchStContext,
    MatchStContextAttrs, MultDivExpressionContext, NodeLabelsContextAttrs, NodePatternContext,
    NodePatternContextAttrs, NotExpressionContext, NotExpressionContextAttrs,
    NullExpressionContextAttrs, NumLitContext, NumLitContextAll, NumLitContextAttrs,
    OrderItemContextAttrs, OrderStContext, OrderStContextAttrs, ParameterContext,
    ParameterContextAttrs, ParenthesizedExpressionContext, ParenthesizedExpressionContextAttrs,
    PatternContextAttrs, PatternElemChainContextAttrs, PatternElemContext, PatternElemContextAttrs,
    PatternPartContextAttrs, PatternWhereContextAttrs, PowerExpressionContext,
    PowerExpressionContextAttrs, ProjectionBodyContext, ProjectionBodyContextAttrs,
    ProjectionItemContextAttrs, ProjectionItemsContextAttrs, PropertyExpressionContext,
    PropertyExpressionContextAttrs, PropertyOrLabelExpressionContext,
    PropertyOrLabelExpressionContextAttrs, RelationDetailContext, RelationDetailContextAttrs,
    RelationshipPatternContext, RelationshipPatternContextAttrs, RelationshipTypesContextAttrs,
    ReturnStContext, ReturnStContextAttrs, SkipStContextAttrs, StringExpPrefixContextAll,
    StringExpPrefixContextAttrs, StringExpressionContextAll, StringExpressionContextAttrs,
    StringLitContext, StringLitContextAttrs, SymbolContextAll, SymbolContextAttrs,
    UnaryAddSubExpressionContext, UnaryAddSubExpressionContextAttrs, XorExpressionContext,
    XorExpressionContextAttrs,
};
use crate::generated::cypherparservisitor::CypherParserVisitorCompat;
use crate::parser::{
    group_into_linear_patterns, parse_int_literal, parse_rel_range, unescape_string,
    validate_named_path_pattern,
};
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat, Tree};

#[derive(Debug, Default)]
pub(crate) enum AstNode {
    #[default]
    None,
    Literal(Literal),
    NodePattern(NodePattern),
    RelPattern(RelPattern),
    Pattern(Pattern),
    QueryParts(Vec<QueryPart>),
    ReturnExpr(ReturnExpr),
    ReturnClause(ParsedReturnClause),
    Err(QueryError),
}

/// `returnSt`'s/`withSt`'s shared `projectionBody` bundles the item list,
/// `DISTINCT`, `ORDER BY`, `SKIP`, and `LIMIT` together, but `Tail`
/// (items + distinct) and `order_by`/`skip`/`limit` live at different
/// levels of `ast::Statement::Match` (the latter three are statement-wide,
/// not per-`Tail`) -- this carries all four out of the visitor together
/// so the caller building `Statement::Match` can split them apart.
#[derive(Debug)]
pub(crate) struct ParsedReturnClause {
    pub tail: Tail,
    pub order_by: Option<Vec<(ReturnExpr, SortDir)>>,
    pub skip: Option<i64>,
    pub limit: Option<i64>,
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
    ast_node_into!(into_return_expr, ReturnExpr, ReturnExpr);
    ast_node_into!(into_return_clause, ReturnClause, ParsedReturnClause);
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
        match parse_num_lit_text(&text) {
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

    fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) -> Self::Return {
        let mut operands = ctx.xorExpression_all().into_iter();
        let mut lhs = match self
            .visit(
                &*operands
                    .next()
                    .expect("expression has at least one xorExpression"),
            )
            .into_return_expr()
        {
            Ok(e) => e,
            Err(e) => return AstNode::Err(e),
        };
        for rhs_ctx in operands {
            let rhs = match self.visit(&*rhs_ctx).into_return_expr() {
                Ok(e) => e,
                Err(e) => return AstNode::Err(e),
            };
            lhs = ReturnExpr::Or(Box::new(lhs), Box::new(rhs));
        }
        AstNode::ReturnExpr(lhs)
    }

    fn visit_xorExpression(&mut self, ctx: &XorExpressionContext<'input>) -> Self::Return {
        let mut operands = ctx.andExpression_all().into_iter();
        let mut lhs = match self
            .visit(
                &*operands
                    .next()
                    .expect("xorExpression has at least one andExpression"),
            )
            .into_return_expr()
        {
            Ok(e) => e,
            Err(e) => return AstNode::Err(e),
        };
        for rhs_ctx in operands {
            let rhs = match self.visit(&*rhs_ctx).into_return_expr() {
                Ok(e) => e,
                Err(e) => return AstNode::Err(e),
            };
            lhs = ReturnExpr::Xor(Box::new(lhs), Box::new(rhs));
        }
        AstNode::ReturnExpr(lhs)
    }

    fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) -> Self::Return {
        let mut operands = ctx.notExpression_all().into_iter();
        let mut lhs = match self
            .visit(
                &*operands
                    .next()
                    .expect("andExpression has at least one notExpression"),
            )
            .into_return_expr()
        {
            Ok(e) => e,
            Err(e) => return AstNode::Err(e),
        };
        for rhs_ctx in operands {
            let rhs = match self.visit(&*rhs_ctx).into_return_expr() {
                Ok(e) => e,
                Err(e) => return AstNode::Err(e),
            };
            lhs = ReturnExpr::And(Box::new(lhs), Box::new(rhs));
        }
        AstNode::ReturnExpr(lhs)
    }

    fn visit_notExpression(&mut self, ctx: &NotExpressionContext<'input>) -> Self::Return {
        let inner = ctx
            .comparisonExpression()
            .expect("notExpression always has a comparisonExpression");
        match self.visit(&*inner).into_return_expr() {
            Ok(mut expr) => {
                for _ in ctx.NOT_all() {
                    expr = ReturnExpr::Not(Box::new(expr));
                }
                AstNode::ReturnExpr(expr)
            }
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_comparisonExpression(
        &mut self,
        ctx: &ComparisonExpressionContext<'input>,
    ) -> Self::Return {
        match self.build_comparison_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_addSubExpression(&mut self, ctx: &AddSubExpressionContext<'input>) -> Self::Return {
        match self.build_add_sub_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_multDivExpression(&mut self, ctx: &MultDivExpressionContext<'input>) -> Self::Return {
        match self.build_mult_div_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    /// Left-associative (`4 ^ 3 ^ 2` is `(4 ^ 3) ^ 2`), same as every
    /// other binary chain here -- matches `parser.rs`'s `parse_pow_expr`,
    /// confirmed against the real TCK fixture (see that function's docs).
    fn visit_powerExpression(&mut self, ctx: &PowerExpressionContext<'input>) -> Self::Return {
        let mut operands = ctx.unaryAddSubExpression_all().into_iter();
        let mut lhs = match self
            .visit(
                &*operands
                    .next()
                    .expect("powerExpression has at least one unaryAddSubExpression"),
            )
            .into_return_expr()
        {
            Ok(e) => e,
            Err(e) => return AstNode::Err(e),
        };
        for rhs_ctx in operands {
            let rhs = match self.visit(&*rhs_ctx).into_return_expr() {
                Ok(e) => e,
                Err(e) => return AstNode::Err(e),
            };
            lhs = ReturnExpr::Arith(Box::new(lhs), ArithOp::Pow, Box::new(rhs));
        }
        AstNode::ReturnExpr(lhs)
    }

    fn visit_unaryAddSubExpression(
        &mut self,
        ctx: &UnaryAddSubExpressionContext<'input>,
    ) -> Self::Return {
        match self.build_unary_add_sub_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_atomicExpression(&mut self, ctx: &AtomicExpressionContext<'input>) -> Self::Return {
        match self.build_atomic_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_propertyOrLabelExpression(
        &mut self,
        ctx: &PropertyOrLabelExpressionContext<'input>,
    ) -> Self::Return {
        match self.build_property_or_label_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_propertyExpression(
        &mut self,
        ctx: &PropertyExpressionContext<'input>,
    ) -> Self::Return {
        match self.build_property_expression(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_atom(&mut self, ctx: &AtomContext<'input>) -> Self::Return {
        match self.build_atom(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_parenthesizedExpression(
        &mut self,
        ctx: &ParenthesizedExpressionContext<'input>,
    ) -> Self::Return {
        let inner = ctx
            .expression()
            .expect("parenthesizedExpression always has an expression");
        self.visit(&*inner)
    }

    fn visit_functionInvocation(
        &mut self,
        ctx: &FunctionInvocationContext<'input>,
    ) -> Self::Return {
        match self.build_function_invocation(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_parameter(&mut self, ctx: &ParameterContext<'input>) -> Self::Return {
        match self.build_parameter(ctx) {
            Ok(expr) => AstNode::ReturnExpr(expr),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_countAll(&mut self, _ctx: &CountAllContext<'input>) -> Self::Return {
        AstNode::ReturnExpr(ReturnExpr::CountStar)
    }

    fn visit_returnSt(&mut self, ctx: &ReturnStContext<'input>) -> Self::Return {
        let body_ctx = ctx
            .projectionBody()
            .expect("returnSt always has a projectionBody");
        match self.build_projection_body(&body_ctx) {
            Ok(c) => AstNode::ReturnClause(c),
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

/// Shared by `visit_numLit` (unsigned) and `build_unary_add_sub_expr`'s
/// sign-folding special case (`text` prefixed with `-`) -- see that
/// function's docs for why a leading sign has to be handled there instead
/// of in `DIGIT` itself.
fn parse_num_lit_text(text: &str) -> Result<Literal, QueryError> {
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
            Err(QueryError::Syntax(format!(
                "float literal '{text}' is too large to represent"
            )))
        } else {
            Ok(Literal::Float(f))
        }
    } else {
        parse_int_literal(text).map(Literal::Int)
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

fn compare_sign(ctx: &ComparisonSignsContextAll) -> CompareOp {
    if ctx.LE().is_some() {
        CompareOp::Le
    } else if ctx.GE().is_some() {
        CompareOp::Ge
    } else if ctx.GT().is_some() {
        CompareOp::Gt
    } else if ctx.LT().is_some() {
        CompareOp::Lt
    } else if ctx.NOT_EQUAL().is_some() {
        CompareOp::Ne
    } else {
        // Only ASSIGN ('=') left -- comparisonSigns' six alternatives are
        // exhaustive.
        CompareOp::Eq
    }
}

fn string_exp_op(ctx: &StringExpPrefixContextAll) -> CompareOp {
    if ctx.STARTS().is_some() {
        CompareOp::StartsWith
    } else if ctx.ENDS().is_some() {
        CompareOp::EndsWith
    } else {
        CompareOp::Contains
    }
}

fn invocation_name_text(ctx: &InvocationNameContextAll) -> String {
    ctx.symbol_all()
        .iter()
        .map(|s| symbol_text(s))
        .collect::<Vec<_>>()
        .join(".")
}

/// Whether `atomicExpression` reduces to exactly a bare numeric literal --
/// no property/label/postfix suffixes at any level between it and the
/// `numLit` itself. Used by `build_unary_add_sub_expression` to fold a
/// leading `-` directly into the literal (see that function's docs).
fn bare_num_lit<'i>(
    ctx: &AtomicExpressionContextAll<'i>,
) -> Option<std::rc::Rc<NumLitContextAll<'i>>> {
    if !ctx.stringExpression_all().is_empty()
        || !ctx.listExpression_all().is_empty()
        || !ctx.nullExpression_all().is_empty()
    {
        return None;
    }
    let prop_or_label = ctx.propertyOrLabelExpression()?;
    if prop_or_label.nodeLabels().is_some() {
        return None;
    }
    let prop_expr = prop_or_label.propertyExpression()?;
    if !prop_expr.name_all().is_empty() {
        return None;
    }
    prop_expr.atom()?.literal()?.numLit()
}

/// For a single-bound `list[..N]`/`list[N..]` slice, `expression_all()`
/// alone can't say which side of `RANGE` the one present bound is on --
/// walk the raw children between `LBRACK` and `RBRACK` to find out, same
/// "read raw children in source order" approach `build_add_sub_expression`
/// uses. `[`/`]`/`..` are the only fixed-text children possible here; the
/// one remaining child is the expression itself.
fn list_expr_bound_is_before_range(ctx: &ListExpressionContextAll) -> bool {
    let mut seen_range = false;
    for child in ctx.get_children() {
        match child.get_text().as_str() {
            "[" | "]" => continue,
            ".." => seen_range = true,
            _ => return !seen_range,
        }
    }
    unreachable!("listExpression slice form always has exactly one expression child")
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

    /// Mirrors `parser.rs`'s `parse_compare_expr` -- a chain folds into
    /// nested `And`s of each *adjacent* pair (`a op0 b op1 c` -> `(a op0
    /// b) AND (b op1 c)`, real Cypher's own chained-comparison semantics),
    /// not a separate AST shape.
    fn build_comparison_expression(
        &mut self,
        ctx: &ComparisonExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let mut operands = Vec::new();
        for operand_ctx in ctx.addSubExpression_all() {
            operands.push(self.visit(&*operand_ctx).into_return_expr()?);
        }
        let mut ops = Vec::new();
        for sign_ctx in ctx.comparisonSigns_all() {
            ops.push(compare_sign(&sign_ctx));
        }
        if ops.is_empty() {
            return Ok(operands
                .into_iter()
                .next()
                .expect("comparisonExpression has at least one addSubExpression"));
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

    /// `SUB_all()`/`PLUS_all()` each only return same-type tokens, losing
    /// which operator occupies which position among possibly-mixed `+`/`-`
    /// -- walking the raw children directly instead recovers real source
    /// order for free, and lets ANTLR's own dispatch (`self.visit` on a
    /// generic child) route each operand to `visit_multDivExpression`
    /// rather than needing the typed `multDivExpression_all()` accessor at
    /// all. The grammar shape (`multDivExpression ((PLUS | SUB)
    /// multDivExpression)*`) guarantees strict operand/operator
    /// alternation, so no type check is needed to tell them apart.
    fn build_add_sub_expression(
        &mut self,
        ctx: &AddSubExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let mut children = ctx.get_children();
        let mut lhs = self
            .visit(
                &*children
                    .next()
                    .expect("addSubExpression has at least one multDivExpression"),
            )
            .into_return_expr()?;
        while let Some(op_node) = children.next() {
            let op = match op_node.get_text().as_str() {
                "+" => ArithOp::Add,
                "-" => ArithOp::Sub,
                other => unreachable!("unexpected addSubExpression operator {other:?}"),
            };
            let rhs_node = children
                .next()
                .expect("addSubExpression operator has a following multDivExpression");
            let rhs = self.visit(&*rhs_node).into_return_expr()?;
            lhs = ReturnExpr::Arith(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn build_mult_div_expression(
        &mut self,
        ctx: &MultDivExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let mut children = ctx.get_children();
        let mut lhs = self
            .visit(
                &*children
                    .next()
                    .expect("multDivExpression has at least one powerExpression"),
            )
            .into_return_expr()?;
        while let Some(op_node) = children.next() {
            let op = match op_node.get_text().as_str() {
                "*" => ArithOp::Mul,
                "/" => ArithOp::Div,
                "%" => ArithOp::Mod,
                other => unreachable!("unexpected multDivExpression operator {other:?}"),
            };
            let rhs_node = children
                .next()
                .expect("multDivExpression operator has a following powerExpression");
            let rhs = self.visit(&*rhs_node).into_return_expr()?;
            lhs = ReturnExpr::Arith(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// The parser already has correct unary-minus handling at this
    /// precedence level (`(PLUS | SUB)? atomicExpression`), but for
    /// `i64::MIN` (`-9223372036854775808`) to round-trip, the sign has to
    /// fold directly into the literal's own parse rather than building
    /// `Neg(Lit(Int(9223372036854775808)))` -- `9223372036854775808`
    /// itself doesn't fit in a positive `i64` at all (only `i64::MIN`'s
    /// magnitude does, via `parse_int_literal`'s two's-complement special
    /// case, which needs the sign in its input string up front). Pest's
    /// grammar sidestepped this by including an optional leading `-` in
    /// `int_literal`/`float_literal` themselves; this grammar's `DIGIT`
    /// deliberately doesn't (see the binary-minus fix), so the fold has to
    /// happen here instead, for the one case where the operand is exactly
    /// a bare numeric literal with no other operators/suffixes.
    fn build_unary_add_sub_expression(
        &mut self,
        ctx: &UnaryAddSubExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let atomic_ctx = ctx
            .atomicExpression()
            .expect("unaryAddSubExpression always has an atomicExpression");
        if ctx.SUB().is_some() {
            if let Some(numlit_ctx) = bare_num_lit(&atomic_ctx) {
                let text = numlit_ctx
                    .DIGIT()
                    .expect("numLit context always has a DIGIT token")
                    .get_text();
                return parse_num_lit_text(&format!("-{text}")).map(ReturnExpr::Lit);
            }
            let operand = self.visit(&*atomic_ctx).into_return_expr()?;
            return Ok(ReturnExpr::Neg(Box::new(operand)));
        }
        // A leading `+` is always a no-op in Cypher (`+x` is just `x`).
        self.visit(&*atomic_ctx).into_return_expr()
    }

    fn build_atomic_expression(
        &mut self,
        ctx: &AtomicExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let base_ctx = ctx
            .propertyOrLabelExpression()
            .expect("atomicExpression always has a propertyOrLabelExpression");
        let base = self.visit(&*base_ctx).into_return_expr()?;
        let string_suffixes = ctx.stringExpression_all();
        let list_suffixes = ctx.listExpression_all();
        let null_suffixes = ctx.nullExpression_all();
        let total = string_suffixes.len() + list_suffixes.len() + null_suffixes.len();
        if total == 0 {
            return Ok(base);
        }
        if total > 1 {
            return Err(QueryError::Syntax(
                "chaining multiple STARTS WITH/ENDS WITH/CONTAINS/IN/[]/IS NULL suffixes on one expression isn't supported yet".into(),
            ));
        }
        if let Some(s) = string_suffixes.into_iter().next() {
            return self.build_string_expression(&s, base);
        }
        if let Some(l) = list_suffixes.into_iter().next() {
            return self.build_list_expression(&l, base);
        }
        let n = null_suffixes
            .into_iter()
            .next()
            .expect("total == 1 and string/list suffixes are empty");
        Ok(if n.NOT().is_some() {
            ReturnExpr::Not(Box::new(ReturnExpr::IsNull(Box::new(base))))
        } else {
            ReturnExpr::IsNull(Box::new(base))
        })
    }

    fn build_string_expression(
        &mut self,
        ctx: &StringExpressionContextAll,
        base: ReturnExpr,
    ) -> Result<ReturnExpr, QueryError> {
        let prefix_ctx = ctx
            .stringExpPrefix()
            .expect("stringExpression always has a stringExpPrefix");
        let op = string_exp_op(&prefix_ctx);
        let rhs_ctx = ctx
            .propertyOrLabelExpression()
            .expect("stringExpression always has a propertyOrLabelExpression");
        let rhs = self.visit(&*rhs_ctx).into_return_expr()?;
        Ok(ReturnExpr::Compare(Box::new(base), op, Box::new(rhs)))
    }

    fn build_list_expression(
        &mut self,
        ctx: &ListExpressionContextAll,
        base: ReturnExpr,
    ) -> Result<ReturnExpr, QueryError> {
        if ctx.IN().is_some() {
            let haystack_ctx = ctx
                .propertyOrLabelExpression()
                .expect("`IN` listExpression always has a propertyOrLabelExpression");
            let haystack = self.visit(&*haystack_ctx).into_return_expr()?;
            return Ok(ReturnExpr::In(Box::new(base), Box::new(haystack)));
        }
        let exprs = ctx.expression_all();
        if ctx.RANGE().is_some() {
            // `list[start..end]` -- either bound can be omitted.
            // `expression_all()` in source order: 0, 1, or 2 present.
            let (start, end) = match exprs.len() {
                0 => (None, None),
                1 => {
                    // One bound present -- is it before or after `RANGE`?
                    // Same alternating-children approach as
                    // `build_add_sub_expression`: walk raw children past
                    // `LBRACK` and see whether the expression comes before
                    // or after the `..` token.
                    let before_range = list_expr_bound_is_before_range(ctx);
                    let e = self.visit(&*exprs[0].clone()).into_return_expr()?;
                    if before_range {
                        (Some(Box::new(e)), None)
                    } else {
                        (None, Some(Box::new(e)))
                    }
                }
                2 => {
                    let start = self.visit(&*exprs[0].clone()).into_return_expr()?;
                    let end = self.visit(&*exprs[1].clone()).into_return_expr()?;
                    (Some(Box::new(start)), Some(Box::new(end)))
                }
                n => unreachable!("listExpression slice form has {n} expressions, expected 0-2"),
            };
            return Ok(ReturnExpr::Slice(Box::new(base), start, end));
        }
        let index_ctx = exprs
            .into_iter()
            .next()
            .expect("non-slice, non-IN listExpression always has exactly one expression");
        let index = self.visit(&*index_ctx).into_return_expr()?;
        Ok(ReturnExpr::Index(Box::new(base), Box::new(index)))
    }

    fn build_property_or_label_expression(
        &mut self,
        ctx: &PropertyOrLabelExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let prop_ctx = ctx
            .propertyExpression()
            .expect("propertyOrLabelExpression always has a propertyExpression");
        let base = self.visit(&*prop_ctx).into_return_expr()?;
        let Some(labels_ctx) = ctx.nodeLabels() else {
            return Ok(base);
        };
        let ReturnExpr::Var(var) = base else {
            return Err(QueryError::Syntax(
                "a label check (`x:Label`) only applies to a bare variable".into(),
            ));
        };
        let labels = labels_ctx.name_all().iter().map(|n| n.get_text()).collect();
        Ok(ReturnExpr::HasLabel(var, labels))
    }

    /// `propertyExpression : atom (DOT name)*`. Mars's `ReturnExpr::Prop`
    /// is a flat `{var, prop}` pair, not a recursive/chainable node --
    /// matching pest's own `prop_access` rule, which is likewise a
    /// dedicated single-level `identifier DOT identifier` production, not
    /// a generic postfix chain over any atom. So a bare atom (no `.name`
    /// suffix) passes through unchanged, exactly one suffix on a bare
    /// variable becomes `Prop`, and anything wider (a chain, or a suffix
    /// on a non-variable base like a function call's result) errors
    /// clearly rather than silently mishandling -- neither pest nor this
    /// parser can represent `duration.between(a, b).days` directly today
    /// (real queries route it through a bound variable first instead,
    /// confirmed against the TCK's own Temporal10 fixtures).
    fn build_property_expression(
        &mut self,
        ctx: &PropertyExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let atom_ctx = ctx.atom().expect("propertyExpression always has an atom");
        let base = self.visit(&*atom_ctx).into_return_expr()?;
        let names = ctx.name_all();
        match names.len() {
            0 => Ok(base),
            1 => {
                let ReturnExpr::Var(var) = base else {
                    return Err(QueryError::Syntax(
                        "property access (`x.prop`) is only supported on a bare variable, not a computed expression"
                            .into(),
                    ));
                };
                Ok(ReturnExpr::Prop(PropAccess {
                    var,
                    prop: names[0].get_text(),
                }))
            }
            _ => Err(QueryError::Syntax(
                "chained property access (`a.b.c`) isn't supported yet".into(),
            )),
        }
    }

    fn build_atom(&mut self, ctx: &AtomContext) -> Result<ReturnExpr, QueryError> {
        if let Some(lit_ctx) = ctx.literal() {
            return self.visit(&*lit_ctx).into_literal().map(ReturnExpr::Lit);
        }
        if let Some(param_ctx) = ctx.parameter() {
            return self.build_parameter(&param_ctx);
        }
        if let Some(paren_ctx) = ctx.parenthesizedExpression() {
            return self.visit(&*paren_ctx).into_return_expr();
        }
        if let Some(func_ctx) = ctx.functionInvocation() {
            return self.build_function_invocation(&func_ctx);
        }
        if let Some(count_ctx) = ctx.countAll() {
            let _ = self.visit(&*count_ctx);
            return Ok(ReturnExpr::CountStar);
        }
        if let Some(sym_ctx) = ctx.symbol() {
            return Ok(ReturnExpr::Var(symbol_text(&sym_ctx)));
        }
        Err(QueryError::Syntax(
            "this expression form (CASE/list comprehension/pattern comprehension/filter/path-as-expression/EXISTS subquery) isn't supported by the ANTLR parser yet".into(),
        ))
    }

    fn build_function_invocation(
        &mut self,
        ctx: &FunctionInvocationContext,
    ) -> Result<ReturnExpr, QueryError> {
        let name_ctx = ctx
            .invocationName()
            .expect("functionInvocation always has an invocationName");
        let name = invocation_name_text(&name_ctx);
        let distinct = ctx.DISTINCT().is_some();
        let mut args = Vec::new();
        if let Some(chain_ctx) = ctx.expressionChain() {
            for arg_ctx in chain_ctx.expression_all() {
                args.push(self.visit(&*arg_ctx).into_return_expr()?);
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

    fn build_parameter(&mut self, ctx: &ParameterContext) -> Result<ReturnExpr, QueryError> {
        let name = if let Some(sym_ctx) = ctx.symbol() {
            symbol_text(&sym_ctx)
        } else if let Some(num_ctx) = ctx.numLit() {
            num_ctx
                .DIGIT()
                .expect("numLit context always has a DIGIT token")
                .get_text()
        } else {
            unreachable!("parameter always has a symbol or numLit")
        };
        Ok(ReturnExpr::Lit(Literal::Param(name)))
    }

    fn build_projection_body(
        &mut self,
        ctx: &ProjectionBodyContext,
    ) -> Result<ParsedReturnClause, QueryError> {
        let distinct = ctx.DISTINCT().is_some();
        let items_ctx = ctx
            .projectionItems()
            .expect("projectionBody always has projectionItems");
        let tail = if items_ctx.MULT().is_some() {
            Tail::ReturnStar(distinct)
        } else {
            let mut items = Vec::new();
            for item_ctx in items_ctx.projectionItem_all() {
                let expr_ctx = item_ctx
                    .expression()
                    .expect("projectionItem always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                let alias = item_ctx.symbol().map(|s| symbol_text(&s));
                items.push(ReturnItem { expr, alias });
            }
            Tail::Return(items, distinct)
        };

        let order_by = match ctx.orderSt() {
            Some(order_ctx) => Some(self.build_order_by(&order_ctx)?),
            None => None,
        };
        let skip = match ctx.skipSt() {
            Some(skip_ctx) => {
                let expr_ctx = skip_ctx
                    .expression()
                    .expect("skipSt always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(literal_non_negative_int(expr, "SKIP")?)
            }
            None => None,
        };
        let limit = match ctx.limitSt() {
            Some(limit_ctx) => {
                let expr_ctx = limit_ctx
                    .expression()
                    .expect("limitSt always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(literal_non_negative_int(expr, "LIMIT")?)
            }
            None => None,
        };

        Ok(ParsedReturnClause {
            tail,
            order_by,
            skip,
            limit,
        })
    }

    fn build_order_by(
        &mut self,
        ctx: &OrderStContext,
    ) -> Result<Vec<(ReturnExpr, SortDir)>, QueryError> {
        let mut items = Vec::new();
        for item_ctx in ctx.orderItem_all() {
            let expr_ctx = item_ctx
                .expression()
                .expect("orderItem always has an expression");
            let expr = self.visit(&*expr_ctx).into_return_expr()?;
            let dir = if item_ctx.DESC().is_some() || item_ctx.DESCENDING().is_some() {
                SortDir::Desc
            } else {
                SortDir::Asc
            };
            items.push((expr, dir));
        }
        Ok(items)
    }
}

/// `skipSt`/`limitSt` grammar-allow any `expression` (`SKIP_W expression`),
/// wider than pest's grammar, which structurally requires a bare
/// `int_literal` there. Matching pest's existing behavior rather than
/// silently accepting something it can't (`SKIP $n`, `LIMIT 1 + 1`) --
/// restrict to a literal non-negative integer here too.
fn literal_non_negative_int(expr: ReturnExpr, clause: &str) -> Result<i64, QueryError> {
    let ReturnExpr::Lit(Literal::Int(n)) = expr else {
        return Err(QueryError::Syntax(format!(
            "{clause} must be a literal non-negative integer"
        )));
    };
    if n < 0 {
        return Err(QueryError::Syntax(format!("{clause} can't be negative")));
    }
    Ok(n)
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

    fn parse_expr(input: &str) -> Result<ReturnExpr, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .expression()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `expression`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_return_expr()
    }

    fn parse_return(input: &str) -> Result<ParsedReturnClause, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .returnSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `returnSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_return_clause()
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

    #[test]
    fn arithmetic_precedence() {
        // 1 + 2 * 3 = 7, not 9 -- * binds tighter than +.
        assert_eq!(
            parse_expr("1 + 2 * 3").unwrap(),
            ReturnExpr::Arith(
                Box::new(ReturnExpr::Lit(Literal::Int(1))),
                ArithOp::Add,
                Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Lit(Literal::Int(2))),
                    ArithOp::Mul,
                    Box::new(ReturnExpr::Lit(Literal::Int(3))),
                )),
            )
        );
    }

    #[test]
    fn arithmetic_left_associative() {
        // 10 - 2 - 3 = (10 - 2) - 3 = 5, not 10 - (2 - 3) = 11.
        assert_eq!(
            parse_expr("10 - 2 - 3").unwrap(),
            ReturnExpr::Arith(
                Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Lit(Literal::Int(10))),
                    ArithOp::Sub,
                    Box::new(ReturnExpr::Lit(Literal::Int(2))),
                )),
                ArithOp::Sub,
                Box::new(ReturnExpr::Lit(Literal::Int(3))),
            )
        );
    }

    #[test]
    fn power_left_associative() {
        assert_eq!(
            parse_expr("4 ^ 3 ^ 2").unwrap(),
            ReturnExpr::Arith(
                Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Lit(Literal::Int(4))),
                    ArithOp::Pow,
                    Box::new(ReturnExpr::Lit(Literal::Int(3))),
                )),
                ArithOp::Pow,
                Box::new(ReturnExpr::Lit(Literal::Int(2))),
            )
        );
    }

    #[test]
    fn binary_minus_no_whitespace() {
        // Exercises the DIGIT-sign-removal grammar fix end to end: `5-1`
        // used to tokenize as two adjacent DIGIT tokens with no operator.
        assert_eq!(
            parse_expr("5-1").unwrap(),
            ReturnExpr::Arith(
                Box::new(ReturnExpr::Lit(Literal::Int(5))),
                ArithOp::Sub,
                Box::new(ReturnExpr::Lit(Literal::Int(1))),
            )
        );
    }

    #[test]
    fn unary_minus_on_variable() {
        assert_eq!(
            parse_expr("-x").unwrap(),
            ReturnExpr::Neg(Box::new(ReturnExpr::Var("x".to_string())))
        );
    }

    #[test]
    fn unary_minus_folds_into_literal() {
        assert_eq!(parse_expr("-5").unwrap(), ReturnExpr::Lit(Literal::Int(-5)));
        assert_eq!(
            parse_expr("-5.5").unwrap(),
            ReturnExpr::Lit(Literal::Float(-5.5))
        );
    }

    #[test]
    fn unary_minus_int_min_two_complement_edge_case() {
        // 9223372036854775808 (2^63) doesn't fit in a positive i64 at all
        // -- only i64::MIN's magnitude does. Folding the sign directly
        // into the literal (rather than building Neg(Lit(Int(...)))) is
        // what makes this representable.
        assert_eq!(
            parse_expr("-9223372036854775808").unwrap(),
            ReturnExpr::Lit(Literal::Int(i64::MIN))
        );
    }

    #[test]
    fn comparison_chain_folds_into_nested_and() {
        // 1 < x < 3 -> (1 < x) AND (x < 3), real Cypher's chained-
        // comparison semantics, not a separate AST shape.
        assert_eq!(
            parse_expr("1 < x < 3").unwrap(),
            ReturnExpr::And(
                Box::new(ReturnExpr::Compare(
                    Box::new(ReturnExpr::Lit(Literal::Int(1))),
                    CompareOp::Lt,
                    Box::new(ReturnExpr::Var("x".to_string())),
                )),
                Box::new(ReturnExpr::Compare(
                    Box::new(ReturnExpr::Var("x".to_string())),
                    CompareOp::Lt,
                    Box::new(ReturnExpr::Lit(Literal::Int(3))),
                )),
            )
        );
    }

    #[test]
    fn boolean_operators() {
        assert_eq!(
            parse_expr("true AND false").unwrap(),
            ReturnExpr::And(
                Box::new(ReturnExpr::Lit(Literal::Bool(true))),
                Box::new(ReturnExpr::Lit(Literal::Bool(false))),
            )
        );
        assert_eq!(
            parse_expr("true OR false").unwrap(),
            ReturnExpr::Or(
                Box::new(ReturnExpr::Lit(Literal::Bool(true))),
                Box::new(ReturnExpr::Lit(Literal::Bool(false))),
            )
        );
        assert_eq!(
            parse_expr("true XOR false").unwrap(),
            ReturnExpr::Xor(
                Box::new(ReturnExpr::Lit(Literal::Bool(true))),
                Box::new(ReturnExpr::Lit(Literal::Bool(false))),
            )
        );
    }

    #[test]
    fn double_negation() {
        // Exercises the notExpression NOT* grammar fix end to end.
        assert_eq!(
            parse_expr("NOT NOT true").unwrap(),
            ReturnExpr::Not(Box::new(ReturnExpr::Not(Box::new(ReturnExpr::Lit(
                Literal::Bool(true)
            )))))
        );
    }

    #[test]
    fn is_null() {
        assert_eq!(
            parse_expr("x IS NULL").unwrap(),
            ReturnExpr::IsNull(Box::new(ReturnExpr::Var("x".to_string())))
        );
        assert_eq!(
            parse_expr("x IS NOT NULL").unwrap(),
            ReturnExpr::Not(Box::new(ReturnExpr::IsNull(Box::new(ReturnExpr::Var(
                "x".to_string()
            )))))
        );
    }

    #[test]
    fn in_operator() {
        assert_eq!(
            parse_expr("x IN y").unwrap(),
            ReturnExpr::In(
                Box::new(ReturnExpr::Var("x".to_string())),
                Box::new(ReturnExpr::Var("y".to_string())),
            )
        );
    }

    #[test]
    fn string_predicates() {
        assert_eq!(
            parse_expr("x STARTS WITH y").unwrap(),
            ReturnExpr::Compare(
                Box::new(ReturnExpr::Var("x".to_string())),
                CompareOp::StartsWith,
                Box::new(ReturnExpr::Var("y".to_string())),
            )
        );
        assert_eq!(
            parse_expr("x ENDS WITH y").unwrap(),
            ReturnExpr::Compare(
                Box::new(ReturnExpr::Var("x".to_string())),
                CompareOp::EndsWith,
                Box::new(ReturnExpr::Var("y".to_string())),
            )
        );
        assert_eq!(
            parse_expr("x CONTAINS y").unwrap(),
            ReturnExpr::Compare(
                Box::new(ReturnExpr::Var("x".to_string())),
                CompareOp::Contains,
                Box::new(ReturnExpr::Var("y".to_string())),
            )
        );
    }

    #[test]
    fn index_and_slice() {
        assert_eq!(
            parse_expr("list[0]").unwrap(),
            ReturnExpr::Index(
                Box::new(ReturnExpr::Var("list".to_string())),
                Box::new(ReturnExpr::Lit(Literal::Int(0))),
            )
        );
        assert_eq!(
            parse_expr("list[1..3]").unwrap(),
            ReturnExpr::Slice(
                Box::new(ReturnExpr::Var("list".to_string())),
                Some(Box::new(ReturnExpr::Lit(Literal::Int(1)))),
                Some(Box::new(ReturnExpr::Lit(Literal::Int(3)))),
            )
        );
        assert_eq!(
            parse_expr("list[..3]").unwrap(),
            ReturnExpr::Slice(
                Box::new(ReturnExpr::Var("list".to_string())),
                None,
                Some(Box::new(ReturnExpr::Lit(Literal::Int(3)))),
            )
        );
        assert_eq!(
            parse_expr("list[1..]").unwrap(),
            ReturnExpr::Slice(
                Box::new(ReturnExpr::Var("list".to_string())),
                Some(Box::new(ReturnExpr::Lit(Literal::Int(1)))),
                None,
            )
        );
    }

    #[test]
    fn property_access() {
        assert_eq!(
            parse_expr("n.name").unwrap(),
            ReturnExpr::Prop(PropAccess {
                var: "n".to_string(),
                prop: "name".to_string(),
            })
        );
    }

    #[test]
    fn chained_property_access_not_yet_supported() {
        assert!(parse_expr("n.a.b").is_err());
    }

    #[test]
    fn property_access_on_computed_expr_not_supported() {
        assert!(parse_expr("duration.between(a, b).days").is_err());
    }

    #[test]
    fn has_label() {
        assert_eq!(
            parse_expr("n:Person").unwrap(),
            ReturnExpr::HasLabel("n".to_string(), vec!["Person".to_string()])
        );
    }

    #[test]
    fn function_call() {
        assert_eq!(
            parse_expr("size(list)").unwrap(),
            ReturnExpr::Call {
                name: "size".to_string(),
                args: vec![ReturnExpr::Var("list".to_string())],
                distinct: false,
            }
        );
    }

    #[test]
    fn namespaced_function_call() {
        assert_eq!(
            parse_expr("duration.between(a, b)").unwrap(),
            ReturnExpr::Call {
                name: "duration.between".to_string(),
                args: vec![
                    ReturnExpr::Var("a".to_string()),
                    ReturnExpr::Var("b".to_string())
                ],
                distinct: false,
            }
        );
    }

    #[test]
    fn count_star() {
        assert_eq!(parse_expr("count(*)").unwrap(), ReturnExpr::CountStar);
    }

    #[test]
    fn aggregate_distinct() {
        assert_eq!(
            parse_expr("count(DISTINCT x)").unwrap(),
            ReturnExpr::Call {
                name: "count".to_string(),
                args: vec![ReturnExpr::Var("x".to_string())],
                distinct: true,
            }
        );
    }

    #[test]
    fn distinct_on_non_aggregate_errors() {
        assert!(parse_expr("size(DISTINCT x)").is_err());
    }

    #[test]
    fn distinct_on_namespaced_call_errors() {
        assert!(parse_expr("duration.between(DISTINCT a, b)").is_err());
    }

    #[test]
    fn parameter_by_name() {
        assert_eq!(
            parse_expr("$name").unwrap(),
            ReturnExpr::Lit(Literal::Param("name".to_string()))
        );
    }

    #[test]
    fn parameter_by_position() {
        assert_eq!(
            parse_expr("$0").unwrap(),
            ReturnExpr::Lit(Literal::Param("0".to_string()))
        );
    }

    #[test]
    fn parenthesized_expression() {
        assert_eq!(
            parse_expr("(1 + 2) * 3").unwrap(),
            ReturnExpr::Arith(
                Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Lit(Literal::Int(1))),
                    ArithOp::Add,
                    Box::new(ReturnExpr::Lit(Literal::Int(2))),
                )),
                ArithOp::Mul,
                Box::new(ReturnExpr::Lit(Literal::Int(3))),
            )
        );
    }

    #[test]
    fn return_simple_items() {
        let c = parse_return("RETURN a, b.name AS name").unwrap();
        let Tail::Return(items, distinct) = c.tail else {
            panic!("expected Tail::Return");
        };
        assert!(!distinct);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].expr, ReturnExpr::Var("a".to_string()));
        assert_eq!(items[0].alias, None);
        assert_eq!(
            items[1].expr,
            ReturnExpr::Prop(PropAccess {
                var: "b".to_string(),
                prop: "name".to_string(),
            })
        );
        assert_eq!(items[1].alias.as_deref(), Some("name"));
    }

    #[test]
    fn return_distinct() {
        let c = parse_return("RETURN DISTINCT a").unwrap();
        let Tail::Return(_, distinct) = c.tail else {
            panic!("expected Tail::Return");
        };
        assert!(distinct);
    }

    #[test]
    fn return_star() {
        let c = parse_return("RETURN *").unwrap();
        assert!(matches!(c.tail, Tail::ReturnStar(false)));
    }

    #[test]
    fn return_order_by_skip_limit() {
        let c = parse_return("RETURN a ORDER BY a DESC SKIP 5 LIMIT 10").unwrap();
        let order_by = c.order_by.unwrap();
        assert_eq!(order_by.len(), 1);
        assert_eq!(order_by[0].0, ReturnExpr::Var("a".to_string()));
        assert_eq!(order_by[0].1, SortDir::Desc);
        assert_eq!(c.skip, Some(5));
        assert_eq!(c.limit, Some(10));
    }

    #[test]
    fn order_by_default_ascending() {
        let c = parse_return("RETURN a ORDER BY a").unwrap();
        assert_eq!(c.order_by.unwrap()[0].1, SortDir::Asc);
    }

    #[test]
    fn limit_negative_errors() {
        // skipSt/limitSt grammar-allow any expression; restricted to a
        // literal non-negative integer to match pest's existing behavior.
        assert!(parse_return("RETURN a LIMIT -1").is_err());
    }

    #[test]
    fn limit_non_literal_errors() {
        assert!(parse_return("RETURN a LIMIT 1 + 1").is_err());
    }
}
