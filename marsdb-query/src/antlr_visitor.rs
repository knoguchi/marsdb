// `AstBuilder`/`AstNode` internals stay unused-by-external-code from
// rustc's perspective even though `parse_antlr` (this file's real public
// entry point, re-exported as `lib.rs`'s `parse`/`parse_many`) exercises
// them at runtime -- the visitor trait's `visit_X` overrides are only
// ever called through dynamic dispatch (`accept()`), which rustc's
// dead-code analysis can't see through.
#![allow(dead_code)]

//! ANTLR-based AST builder (`mars-nog`/`mars-cuk`) -- replaced the old
//! pest-tree-walk (`parser.rs`/`cypher.pest`, deleted at cutover) as this
//! crate's real Cypher parser. `parse_antlr`/`parse_antlr_many` are
//! re-exported by `lib.rs` as `parse`/`parse_many`.
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
    is_aggregate_name, ArithOp, CompareOp, Expr, Literal, MergeClause, NodePattern, Pattern,
    PropAccess, QuantifierKind, QueryClause, QueryPart, RelDirection, RelPattern, RemoveItem,
    ReturnExpr, ReturnItem, ReturnTail, SetItem, SortDir, Statement, Tail, UnwindClause,
    UnwindSource, WithClause, WithExpr,
};
use crate::error::QueryError;
use crate::generated::cypherparser::{
    AddSubExpressionContext, AndExpressionContext, AndExpressionContextAttrs, AtomContext,
    AtomContextAttrs, AtomicExpressionContext, AtomicExpressionContextAll,
    AtomicExpressionContextAttrs, BoolLitContext, BoolLitContextAttrs, CaseExpressionContext,
    CharLitContext, CharLitContextAttrs, ComparisonExpressionContext,
    ComparisonExpressionContextAttrs, ComparisonSignsContextAll, ComparisonSignsContextAttrs,
    CountAllContext, CreateIndexStContext, CreateIndexStContextAttrs, CreateStContext,
    CreateStContextAttrs, DeleteStContext, DeleteStContextAttrs, ExplainStContext,
    ExplainStContextAttrs, ExpressionChainContextAttrs, ExpressionContext, ExpressionContextAttrs,
    FilterExpressionContext, FilterExpressionContextAttrs, FilterWithContext,
    FilterWithContextAttrs, FunctionInvocationContext, FunctionInvocationContextAttrs,
    InExpressionContextAttrs, InvocationNameContextAll, InvocationNameContextAttrs,
    LhsContextAttrs, LimitStContextAttrs, ListComprehensionContext, ListComprehensionContextAttrs,
    ListExpressionContextAll, ListExpressionContextAttrs, ListLitContext, ListLitContextAttrs,
    LiteralContext, LiteralContextAttrs, MapLitContext, MapLitContextAttrs, MapPairContextAttrs,
    MatchStContext, MatchStContextAttrs, MergeActionContextAll, MergeActionContextAttrs,
    MergeStContext, MergeStContextAttrs, MultDivExpressionContext, MultiPartQContext,
    MultiPartQContextAttrs, NameContextAll, NameContextAttrs, NodeLabelsContextAttrs,
    NodePatternContext, NodePatternContextAttrs, NotExpressionContext, NotExpressionContextAttrs,
    NullExpressionContextAttrs, NumLitContext, NumLitContextAll, NumLitContextAttrs,
    OrderItemContextAttrs, OrderStContext, OrderStContextAttrs, ParameterContext,
    ParameterContextAttrs, ParenthesizedExpressionContext, ParenthesizedExpressionContextAttrs,
    PatternComprehensionContext, PatternComprehensionContextAttrs, PatternContextAttrs,
    PatternElemChainContextAttrs, PatternElemContext, PatternElemContextAttrs,
    PatternPartContextAttrs, PatternWhereContextAttrs, PowerExpressionContext,
    PowerExpressionContextAttrs, ProjectionBodyContext, ProjectionBodyContextAttrs,
    ProjectionItemContextAttrs, ProjectionItemsContextAttrs, PropertiesContextAll,
    PropertiesContextAttrs, PropertyExpressionContext, PropertyExpressionContextAttrs,
    PropertyOrLabelExpressionContext, PropertyOrLabelExpressionContextAttrs,
    ReadingStatementContextAll, ReadingStatementContextAttrs, RegularQueryContext,
    RegularQueryContextAttrs, RelationDetailContext, RelationDetailContextAttrs,
    RelationshipPatternContext, RelationshipPatternContextAttrs, RelationshipTypesContextAttrs,
    RelationshipsChainPatternContext, RelationshipsChainPatternContextAttrs, RemoveItemContextAll,
    RemoveItemContextAttrs, RemoveStContext, RemoveStContextAttrs, ReturnStContext,
    ReturnStContextAttrs, SetItemContextAll, SetItemContextAttrs, SetStContext, SetStContextAttrs,
    ShortestPathWrapperContextAttrs, SinglePartQContext, SinglePartQContextAttrs,
    SkipStContextAttrs, StandaloneCallContext, StringExpPrefixContextAll,
    StringExpPrefixContextAttrs, StringExpressionContextAll, StringExpressionContextAttrs,
    StringListNullExpressionContext, StringListNullExpressionContextAttrs, StringLitContext,
    StringLitContextAttrs, SubqueryExistContext, SubqueryExistContextAttrs, SymbolContextAll,
    SymbolContextAttrs, UnaryAddSubExpressionContext, UnaryAddSubExpressionContextAttrs,
    UnionStContextAttrs, UnwindStContext, UnwindStContextAttrs, UpdatingStatementContextAll,
    UpdatingStatementContextAttrs, WhereContextAttrs, WithStContext, WithStContextAttrs,
    XorExpressionContext, XorExpressionContextAttrs,
};
use crate::generated::cypherparservisitor::CypherParserVisitorCompat;
use crate::parse_helpers::{
    group_into_linear_patterns, parse_int_literal, parse_rel_range, unescape_string,
    validate_named_path_pattern, validate_shortest_path_pattern,
};
use antlr4rust::parser_rule_context::ParserRuleContext;
use antlr4rust::token::Token;
use antlr4rust::tree::{ParseTree, ParseTreeVisitorCompat, Tree};
use std::rc::Rc;

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
    WithClause(WithClause),
    UnwindClause(UnwindClause),
    SetItems(Vec<SetItem>),
    DeleteItems(ParsedDelete),
    RemoveItems(Vec<RemoveItem>),
    CreatePatterns(Vec<Pattern>),
    MergeClause(MergeClause),
    Statement(Statement),
    Err(QueryError),
}

#[derive(Debug)]
pub(crate) struct ParsedDelete {
    pub items: Vec<ReturnExpr>,
    pub detach: bool,
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

    /// `ast::Literal` has no List/Map variant -- `listLit`/`mapLit` build
    /// `ReturnExpr::ListLit`/`MapLit` directly instead (see
    /// `visit_listLit`/`visit_mapLit`), so a `literal` context (reached
    /// via `atom`, which can't tell in advance which of its 7 alternatives
    /// it'll get) may resolve to either an `AstNode::Literal` (bool/num/
    /// string/char/null) or an `AstNode::ReturnExpr` (list/map). This
    /// accepts either, wrapping a bare `Literal` in `ReturnExpr::Lit`.
    fn into_return_expr_lenient(self) -> Result<ReturnExpr, QueryError> {
        match self {
            AstNode::Literal(l) => Ok(ReturnExpr::Lit(l)),
            AstNode::ReturnExpr(e) => Ok(e),
            AstNode::Err(e) => Err(e),
            other => {
                unreachable!("expected AstNode::Literal or AstNode::ReturnExpr, got {other:?}")
            }
        }
    }
    ast_node_into!(into_return_clause, ReturnClause, ParsedReturnClause);
    ast_node_into!(into_with_clause, WithClause, WithClause);
    ast_node_into!(into_unwind_clause, UnwindClause, UnwindClause);
    ast_node_into!(into_set_items, SetItems, Vec<SetItem>);
    ast_node_into!(into_delete_items, DeleteItems, ParsedDelete);
    ast_node_into!(into_remove_items, RemoveItems, Vec<RemoveItem>);
    ast_node_into!(into_create_patterns, CreatePatterns, Vec<Pattern>);
    ast_node_into!(into_merge_clause, MergeClause, MergeClause);
    ast_node_into!(into_statement, Statement, Statement);
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

    fn visit_stringListNullExpression(
        &mut self,
        ctx: &StringListNullExpressionContext<'input>,
    ) -> Self::Return {
        match self.build_string_list_null_expression(ctx) {
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

    fn visit_withSt(&mut self, ctx: &WithStContext<'input>) -> Self::Return {
        match self.build_with_clause(ctx) {
            Ok(c) => AstNode::WithClause(c),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_unwindSt(&mut self, ctx: &UnwindStContext<'input>) -> Self::Return {
        match self.build_unwind_st(ctx) {
            Ok(c) => AstNode::UnwindClause(c),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_setSt(&mut self, ctx: &SetStContext<'input>) -> Self::Return {
        match self.build_set_st(ctx) {
            Ok(items) => AstNode::SetItems(items),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_deleteSt(&mut self, ctx: &DeleteStContext<'input>) -> Self::Return {
        match self.build_delete_st(ctx) {
            Ok(d) => AstNode::DeleteItems(d),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_removeSt(&mut self, ctx: &RemoveStContext<'input>) -> Self::Return {
        match self.build_remove_st(ctx) {
            Ok(items) => AstNode::RemoveItems(items),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_createSt(&mut self, ctx: &CreateStContext<'input>) -> Self::Return {
        match self.build_create_st(ctx) {
            Ok(patterns) => AstNode::CreatePatterns(patterns),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_mergeSt(&mut self, ctx: &MergeStContext<'input>) -> Self::Return {
        match self.build_merge_st(ctx) {
            Ok(c) => AstNode::MergeClause(c),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_singlePartQ(&mut self, ctx: &SinglePartQContext<'input>) -> Self::Return {
        match self.build_single_part_q(ctx) {
            Ok(s) => AstNode::Statement(s),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_multiPartQ(&mut self, ctx: &MultiPartQContext<'input>) -> Self::Return {
        match self.build_multi_part_q(ctx) {
            Ok(s) => AstNode::Statement(s),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_regularQuery(&mut self, ctx: &RegularQueryContext<'input>) -> Self::Return {
        match self.build_regular_query(ctx) {
            Ok(s) => AstNode::Statement(s),
            Err(e) => AstNode::Err(e),
        }
    }

    // `query : explainSt | regularQuery | standaloneCall | createIndexSt`
    // -- `regularQuery`/`explainSt`/`createIndexSt` need no override of
    // their own *here* (default `visit_children` dispatch already routes
    // to each rule's own override below), but `standaloneCall` (a bare
    // `CALL proc(...) YIELD ...` with no MATCH at all) has no `Statement`
    // representation yet (same CALL gap as `queryCallSt`, tracked
    // separately as mars-82w) -- overridden so reaching it errors cleanly
    // instead of default-recursing into its inner symbol/expression nodes
    // and silently producing a wrong-shaped `AstNode`.
    fn visit_standaloneCall(&mut self, _ctx: &StandaloneCallContext<'input>) -> Self::Return {
        AstNode::Err(QueryError::Syntax(
            "CALL isn't supported by the ANTLR parser yet".into(),
        ))
    }

    fn visit_explainSt(&mut self, ctx: &ExplainStContext<'input>) -> Self::Return {
        match self.build_explain_st(ctx) {
            Ok(s) => AstNode::Statement(s),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_createIndexSt(&mut self, ctx: &CreateIndexStContext<'input>) -> Self::Return {
        match self.build_create_index_st(ctx) {
            Ok(s) => AstNode::Statement(s),
            Err(e) => AstNode::Err(e),
        }
    }

    fn visit_listLit(&mut self, ctx: &ListLitContext<'input>) -> Self::Return {
        let mut items = Vec::new();
        if let Some(chain_ctx) = ctx.expressionChain() {
            for expr_ctx in chain_ctx.expression_all() {
                match self.visit(&*expr_ctx).into_return_expr() {
                    Ok(e) => items.push(e),
                    Err(e) => return AstNode::Err(e),
                }
            }
        }
        AstNode::ReturnExpr(ReturnExpr::ListLit(items))
    }

    fn visit_mapLit(&mut self, ctx: &MapLitContext<'input>) -> Self::Return {
        let mut items = Vec::new();
        for pair_ctx in ctx.mapPair_all() {
            let name_ctx = pair_ctx.name().expect("mapPair always has a name");
            let expr_ctx = pair_ctx
                .expression()
                .expect("mapPair always has an expression");
            let value = match self.visit(&*expr_ctx).into_return_expr() {
                Ok(v) => v,
                Err(e) => return AstNode::Err(e),
            };
            items.push((name_text(&name_ctx), value));
        }
        AstNode::ReturnExpr(ReturnExpr::MapLit(items))
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

/// `name : symbol | reservedWord`, used for label/property/map-key names
/// (unlike `symbol`, usable for bound-variable names too). Delegates to
/// `symbol_text` (backtick-stripping) when the alternative taken is
/// `symbol` -- a bare `.get_text()` here would keep the backticks
/// themselves as part of the name (e.g. `` map.`name` `` would look up
/// the map key `` `name` `` instead of `name`, always missing -- a real
/// bug found via the TCK, not just deferred coverage). `reservedWord` has
/// no escaping to strip either way.
fn name_text(ctx: &NameContextAll) -> String {
    match ctx.symbol() {
        Some(s) => symbol_text(&s),
        None => ctx.get_text(),
    }
}

/// Shared by `visit_numLit` (unsigned) and `build_unary_add_sub_expr`'s
/// sign-folding special case (`text` prefixed with `-`) -- see that
/// function's docs for why a leading sign has to be handled there instead
/// of in `DIGIT` itself.
fn parse_num_lit_text(text: &str) -> Result<Literal, QueryError> {
    // A hex/octal literal's own digits can end in `f`/`F`/`d`/`D` (real hex
    // digits, e.g. `0x7FFFFFFFFFFFFFFF`) or contain `e`/`E` (also a real
    // hex digit) -- neither is real openCypher's `<approximate number
    // suffix>` (`F`/`D`/`f`), which per spec only ever follows a decimal
    // literal already in scientific or common (has a `.`) notation. Found
    // via a Phase 3 dry-run behavioral test failure
    // (`int_literal_accepts_hex_and_octal_forms`): a hex literal ending in
    // a suffix-shaped digit was misdetected as float and failed to parse.
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let is_hex_or_octal = unsigned
        .as_bytes()
        .get(1)
        .is_some_and(|b| matches!(b, b'x' | b'X' | b'o' | b'O'))
        && unsigned.starts_with('0');
    let is_float = !is_hex_or_octal
        && (text.contains('.')
            || text.ends_with(['f', 'F', 'd', 'D'])
            || text
                .rfind(['e', 'E'])
                .is_some_and(|i| text[..i].chars().all(|c| c.is_ascii_digit() || c == '-')));
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
/// `stringExpression`/`nullExpression`/`inExpression` no longer live at
/// this level at all (moved up to `stringListNullExpression`, see its own
/// docs) -- only `listExpression` (postfix index/slice) can still appear
/// here, so that's the only check left.
fn bare_num_lit<'i>(
    ctx: &AtomicExpressionContextAll<'i>,
) -> Option<std::rc::Rc<NumLitContextAll<'i>>> {
    if !ctx.listExpression_all().is_empty() {
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
    /// `properties : mapLit | parameter`. Only the `mapLit` alternative
    /// has a real `NodePattern`/`RelPattern::props` representation --
    /// `Vec<(String, ReturnExpr)>` has no "the whole map comes from one
    /// parameter" shape, and pest doesn't support that on a pattern's
    /// inline properties either (only `map_expr`), so rejecting it here
    /// isn't a regression, just parity.
    fn build_properties(
        &mut self,
        ctx: Option<Rc<PropertiesContextAll>>,
    ) -> Result<Vec<(String, ReturnExpr)>, QueryError> {
        let Some(ctx) = ctx else {
            return Ok(Vec::new());
        };
        let Some(map_ctx) = ctx.mapLit() else {
            return Err(QueryError::Syntax(
                "a parameter can't be used as a pattern's whole properties map".into(),
            ));
        };
        let expr = self.visit(&*map_ctx).into_return_expr()?;
        let ReturnExpr::MapLit(items) = expr else {
            unreachable!("mapLit always builds a ReturnExpr::MapLit");
        };
        Ok(items)
    }

    fn build_node_pattern(&mut self, ctx: &NodePatternContext) -> Result<NodePattern, QueryError> {
        let var = ctx.symbol().map(|s| symbol_text(&s));
        let labels = ctx
            .nodeLabels()
            .map(|nl| nl.name_all().iter().map(|n| name_text(n)).collect())
            .unwrap_or_default();
        let has_explicit_props = ctx.properties().is_some();
        let props = self.build_properties(ctx.properties())?;
        Ok(NodePattern {
            var,
            labels,
            props,
            has_explicit_props,
        })
    }

    fn build_rel_detail(&mut self, ctx: &RelationDetailContext) -> Result<RelPattern, QueryError> {
        let var = ctx.symbol().map(|s| symbol_text(&s));
        let rel_types = ctx
            .relationshipTypes()
            .map(|rt| rt.name_all().iter().map(|n| name_text(n)).collect())
            .unwrap_or_default();
        let props = self.build_properties(ctx.properties())?;
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
        // Both LT and GT present (`<-[...]->`) is *not* "left wins" --
        // it's the same undirected/either shape as neither being present
        // (`-[...]-`), and CREATE/MERGE already reject `Either` outright
        // (`RequiresDirectedRelationship`, executor.rs). Found via the
        // TCK: the old `if LT ... else if GT ...` order silently treated
        // `<-[:FOO]->` as plain `Left`, both letting CREATE wrongly
        // succeed (Create2 [20]) and giving MATCH's own undirected
        // multi-hop patterns the wrong direction entirely (mars-w37,
        // Match5 [27]/Match6 [12]'s wrong row counts).
        rel.direction = match (ctx.LT().is_some(), ctx.GT().is_some()) {
            (true, false) => RelDirection::Left,
            (false, true) => RelDirection::Right,
            (true, true) | (false, false) => RelDirection::Either,
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

    /// `relationshipsChainPattern : nodePattern patternElemChain+` -- an
    /// `atom` alternative (`(n)-->()` used directly as a boolean
    /// expression, TCK's Pattern1/2 "Pattern predicate"), same node+chain
    /// shape `build_pattern_elem` already builds for real match patterns,
    /// just requiring at least one hop (no bare-node pattern predicate,
    /// matching the grammar's own `+` here vs `patternElem`'s `*`).
    fn build_relationships_chain_pattern(
        &mut self,
        ctx: &RelationshipsChainPatternContext,
    ) -> Result<Pattern, QueryError> {
        let node_ctx = ctx
            .nodePattern()
            .expect("relationshipsChainPattern always has a nodePattern");
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
        let where_clause = match pw.where_() {
            Some(where_ctx) => {
                let expr_ctx = where_ctx
                    .expression()
                    .expect("where always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(return_expr_to_expr(expr)?)
            }
            None => None,
        };
        let pattern_ctx = pw.pattern().expect("patternWhere always has a pattern");

        let mut path_var = None;
        let mut shortest_path = false;
        let mut patterns = Vec::new();
        for (i, part) in pattern_ctx.patternPart_all().into_iter().enumerate() {
            // `shortestPathWrapper` is grammar-permissive (any
            // comma-separated position) -- restricted here to the first
            // position only, same as `parser.rs`'s `parse_path_pattern`
            // (real Cypher: naming/shortestPath only make sense on a
            // single linear pattern, never a cross join).
            let pattern = match part.shortestPathWrapper() {
                Some(sp_ctx) => {
                    if i != 0 {
                        return Err(QueryError::Syntax(
                            "shortestPath() must be the first (and only) comma-separated pattern"
                                .into(),
                        ));
                    }
                    shortest_path = true;
                    let elem_ctx = sp_ctx
                        .patternElem()
                        .expect("shortestPathWrapper always has a patternElem");
                    self.visit(&*elem_ctx).into_pattern()?
                }
                None => {
                    let elem_ctx = part.patternElem().expect(
                        "patternPart always has a patternElem when shortestPathWrapper is absent",
                    );
                    self.visit(&*elem_ctx).into_pattern()?
                }
            };
            patterns.push(pattern);
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
        if groups.len() > 1 && (shortest_path || path_var.is_some()) {
            return Err(QueryError::Syntax(
                "a named path/shortestPath() can't span a comma-separated cross join".into(),
            ));
        }
        if shortest_path {
            validate_shortest_path_pattern(&groups[0])?;
        } else if path_var.is_some() {
            validate_named_path_pattern(&groups[0])?;
        }

        // `where_clause` attaches to the *last* group only, same as
        // `parser.rs`'s `parse_match_part` -- a comma-separated cross join
        // sees every group's bindings by the time WHERE runs. `with` stays
        // unconditionally `None` here: this grammar's `matchSt` has no
        // trailing WITH of its own (that's a separate clause in the
        // statement's clause list, attached by whichever caller builds
        // that list, not here).
        let last = groups.len() - 1;
        Ok(groups
            .into_iter()
            .enumerate()
            .map(|(i, pattern)| QueryPart {
                optional,
                path_var: if i == 0 { path_var.clone() } else { None },
                shortest_path: i == 0 && shortest_path,
                pattern,
                where_clause: if i == last {
                    where_clause.clone()
                } else {
                    None
                },
                with: None,
            })
            .collect())
    }

    /// Mirrors `parser.rs`'s `parse_compare_expr` -- a chain folds into
    /// nested `And`s of each *adjacent* pair (`a op0 b op1 c` -> `(a op0
    /// b) AND (b op1 c)`, real Cypher's own chained-comparison semantics),
    /// not a separate AST shape. Operand type is `stringListNullExpression`
    /// (not `addSubExpression` directly) since the precedence fix moved
    /// `IN`/`STARTS WITH`/etc up to sit between this level and arithmetic
    /// -- see `build_string_list_null_expression`'s docs.
    fn build_comparison_expression(
        &mut self,
        ctx: &ComparisonExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let mut operands = Vec::new();
        for operand_ctx in ctx.stringListNullExpression_all() {
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
                .expect("comparisonExpression has at least one stringListNullExpression"));
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

    /// `atomicExpression : propertyOrLabelExpression (listExpression)*`
    /// -- only postfix index/slice suffixes live here now (`IN`/
    /// `stringExpression`/`nullExpression` moved up to
    /// `stringListNullExpression`, see its own docs); genuinely
    /// left-to-right chainable (`list[0][1]`, real postfix repetition per
    /// openCypher.bnf's `<postfix expression> ::= ... | <postfix
    /// expression> <postfix operator>`), so no "at most one" restriction
    /// is needed here at all anymore.
    fn build_atomic_expression(
        &mut self,
        ctx: &AtomicExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let base_ctx = ctx
            .propertyOrLabelExpression()
            .expect("atomicExpression always has a propertyOrLabelExpression");
        let mut base = self.visit(&*base_ctx).into_return_expr()?;
        for l in ctx.listExpression_all() {
            base = self.build_list_expression(&l, base)?;
        }
        Ok(base)
    }

    /// `stringListNullExpression : addSubExpression (stringExpression |
    /// inExpression | nullExpression)?` -- fixes a real precedence bug in
    /// the vendored grammar (found via a Phase 3 behavioral dry-run, not
    /// the TCK): `IN`/`STARTS WITH`/`ENDS WITH`/`CONTAINS`/`IS NULL` used
    /// to attach at `atomicExpression`'s level (tighter than `+`/`-`/`*`/
    /// `/`/`^`), so `n.val + 0 IS NULL` parsed as `n.val + (0 IS NULL)`.
    /// Per openCypher.bnf's `<comparison predicate>` chain, these operate
    /// on a full `<arithmetic value expression>` (this file's
    /// `addSubExpression`), sitting above arithmetic and below `=`/`<>`/
    /// `<`/`>`/`<=`/`>=` (`comparisonExpression`, one level up) --
    /// see `grammar/README.md` for the upstream PR this was also sent to.
    fn build_string_list_null_expression(
        &mut self,
        ctx: &StringListNullExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let base_ctx = ctx
            .addSubExpression()
            .expect("stringListNullExpression always has an addSubExpression");
        let base = self.visit(&*base_ctx).into_return_expr()?;
        if let Some(s) = ctx.stringExpression() {
            return self.build_string_expression(&s, base);
        }
        if let Some(i) = ctx.inExpression() {
            let rhs_ctx = i
                .addSubExpression()
                .expect("inExpression always has an addSubExpression");
            let rhs = self.visit(&*rhs_ctx).into_return_expr()?;
            return Ok(ReturnExpr::In(Box::new(base), Box::new(rhs)));
        }
        let Some(n) = ctx.nullExpression() else {
            return Ok(base);
        };
        Ok(if n.NOT().is_some() {
            ReturnExpr::Not(Box::new(ReturnExpr::IsNull(Box::new(base))))
        } else {
            ReturnExpr::IsNull(Box::new(base))
        })
    }

    /// Operand widened from `propertyOrLabelExpression` to
    /// `addSubExpression` (moved up alongside `stringListNullExpression`,
    /// see its own docs) -- `x STARTS WITH y + z` is now real, matching
    /// spec's `<advanced comparison predicand> ::= <arithmetic value
    /// expression>`.
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
            .addSubExpression()
            .expect("stringExpression always has an addSubExpression");
        let rhs = self.visit(&*rhs_ctx).into_return_expr()?;
        Ok(ReturnExpr::Compare(Box::new(base), op, Box::new(rhs)))
    }

    /// `listExpression` no longer has an `IN` alternative at all (moved to
    /// the new `inExpression` rule, built directly in
    /// `build_string_list_null_expression`) -- only the postfix index/
    /// slice forms remain.
    fn build_list_expression(
        &mut self,
        ctx: &ListExpressionContextAll,
        base: ReturnExpr,
    ) -> Result<ReturnExpr, QueryError> {
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
            .expect("non-slice listExpression always has exactly one expression");
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
        let labels = labels_ctx.name_all().iter().map(|n| name_text(n)).collect();
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
                    prop: name_text(&names[0]),
                }))
            }
            _ => Err(QueryError::Syntax(
                "chained property access (`a.b.c`) isn't supported yet".into(),
            )),
        }
    }

    fn build_atom(&mut self, ctx: &AtomContext) -> Result<ReturnExpr, QueryError> {
        if let Some(lit_ctx) = ctx.literal() {
            return self.visit(&*lit_ctx).into_return_expr_lenient();
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
        if let Some(filter_ctx) = ctx.filterWith() {
            return self.build_filter_with(&filter_ctx);
        }
        if let Some(lc_ctx) = ctx.listComprehension() {
            return self.build_list_comprehension(&lc_ctx);
        }
        if let Some(case_ctx) = ctx.caseExpression() {
            return self.build_case_expression(&case_ctx);
        }
        if let Some(pc_ctx) = ctx.patternComprehension() {
            return self.build_pattern_comprehension(&pc_ctx);
        }
        if let Some(rcp_ctx) = ctx.relationshipsChainPattern() {
            return Ok(ReturnExpr::PatternPredicate(
                self.build_relationships_chain_pattern(&rcp_ctx)?,
            ));
        }
        if let Some(se_ctx) = ctx.subqueryExist() {
            return self.build_subquery_exist(&se_ctx);
        }
        Err(QueryError::Syntax(
            "this expression form (path-as-expression) isn't supported by the ANTLR parser yet"
                .into(),
        ))
    }

    /// `patternComprehension : LBRACK lhs? relationshipsChainPattern where?
    /// STICK expression RBRACK` -- `lhs` (`symbol ASSIGN`) is the optional
    /// named-path capture (`p = (n)-->()`), reusing
    /// `build_relationships_chain_pattern` for the pattern itself (same
    /// node+chain shape a pattern predicate already builds, just here it's
    /// enumerated rather than existence-checked) and the same `where?`
    /// production `build_match_st` uses for an ordinary `MATCH`'s own
    /// pattern-level `WHERE` (not `ListComp`'s post-projection
    /// `ReturnExpr`-shaped filter -- `patternComprehension` shares its
    /// grammar rule with `MATCH`, not with `listComprehension`).
    fn build_pattern_comprehension(
        &mut self,
        ctx: &PatternComprehensionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let path_var = ctx
            .lhs()
            .and_then(|lhs| lhs.symbol())
            .map(|s| symbol_text(&s));
        let rcp_ctx = ctx
            .relationshipsChainPattern()
            .expect("patternComprehension always has a relationshipsChainPattern");
        let pattern = self.build_relationships_chain_pattern(&rcp_ctx)?;
        let where_clause = match ctx.where_() {
            Some(where_ctx) => {
                let expr_ctx = where_ctx
                    .expression()
                    .expect("where always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(Box::new(return_expr_to_expr(expr)?))
            }
            None => None,
        };
        let proj_ctx = ctx
            .expression()
            .expect("patternComprehension always has a projection expression");
        let projection = self.visit(&*proj_ctx).into_return_expr()?;
        Ok(ReturnExpr::PatternComprehension {
            path_var,
            pattern: Box::new(pattern),
            where_clause,
            projection: Box::new(projection),
        })
    }

    /// `subqueryExist : EXISTS LBRACE (regularQuery | patternWhere)
    /// RBRACE` -- only the `patternWhere` alternative (TCK's
    /// ExistentialSubquery1, the "simple" form: a pattern with an
    /// optional inline `WHERE`, same grammar rule `MATCH` itself uses)
    /// is supported. The `regularQuery` alternative (TCK's
    /// ExistentialSubquery2, a full nested `MATCH ... RETURN ...`
    /// subquery, arbitrarily many clauses) needs running an arbitrary
    /// nested `Statement` correlated against the current row -- a bigger
    /// change, not attempted here.
    fn build_subquery_exist(
        &mut self,
        ctx: &SubqueryExistContext,
    ) -> Result<ReturnExpr, QueryError> {
        let Some(pw_ctx) = ctx.patternWhere() else {
            return Err(QueryError::Syntax(
                "exists { MATCH ... RETURN ... } (a full nested subquery, as opposed to \
                 exists { (pattern) WHERE ... }) isn't supported yet"
                    .into(),
            ));
        };
        let pattern_ctx = pw_ctx.pattern().expect("patternWhere always has a pattern");
        let mut parts = pattern_ctx.patternPart_all().into_iter();
        let part = parts
            .next()
            .expect("pattern always has at least one patternPart");
        if parts.next().is_some() {
            return Err(QueryError::Syntax(
                "exists {} with more than one comma-separated pattern isn't supported yet".into(),
            ));
        }
        if part.ASSIGN().is_some() || part.shortestPathWrapper().is_some() {
            return Err(QueryError::Syntax(
                "exists {} doesn't support a named path or shortestPath()".into(),
            ));
        }
        let elem_ctx = part
            .patternElem()
            .expect("a patternPart without ASSIGN/shortestPathWrapper always has a patternElem");
        let pattern = self.visit(&*elem_ctx).into_pattern()?;
        let where_clause = match pw_ctx.where_() {
            Some(where_ctx) => {
                let expr_ctx = where_ctx
                    .expression()
                    .expect("where always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(Box::new(return_expr_to_expr(expr)?))
            }
            None => None,
        };
        Ok(ReturnExpr::ExistsPattern {
            pattern: Box::new(pattern),
            where_clause,
        })
    }

    /// `caseExpression : CASE expression? (WHEN expression THEN
    /// expression)+ (ELSE expression)? END`. No typed per-`WHEN`/`THEN`
    /// accessor exists (`expression_all()` flattens every branch's exprs
    /// together, `WHEN()`/`THEN()`/`ELSE()` only ever return the *first*
    /// occurrence) -- walked via raw children instead, same "read raw
    /// children in source order" approach `build_add_sub_expression` uses,
    /// tracking position via each keyword *terminal*'s own text. Matched
    /// case-insensitively (the lexer's `caseInsensitive = true` means
    /// `get_text()` returns the source's own casing, e.g. `case`/`CASE`
    /// both valid) -- safe against a same-named real expression, since
    /// CASE/WHEN/THEN/ELSE/END are all in `reservedWord`, so none can
    /// appear as a bare variable at this position.
    fn build_case_expression(
        &mut self,
        ctx: &CaseExpressionContext,
    ) -> Result<ReturnExpr, QueryError> {
        #[derive(PartialEq)]
        enum Pos {
            BeforeFirstWhen,
            AfterWhen,
            AfterThen,
            AfterElse,
        }
        let mut pos = Pos::BeforeFirstWhen;
        let mut test = None;
        let mut whens: Vec<(ReturnExpr, ReturnExpr)> = Vec::new();
        let mut pending_when: Option<ReturnExpr> = None;
        let mut else_ = None;
        for child in ctx.get_children() {
            match child.get_text().to_ascii_uppercase().as_str() {
                "CASE" | "END" => continue,
                "WHEN" => pos = Pos::AfterWhen,
                "THEN" => pos = Pos::AfterThen,
                "ELSE" => pos = Pos::AfterElse,
                _ => {
                    let expr = self.visit(&*child).into_return_expr()?;
                    match pos {
                        Pos::BeforeFirstWhen => test = Some(Box::new(expr)),
                        Pos::AfterWhen => pending_when = Some(expr),
                        Pos::AfterThen => {
                            let w = pending_when
                                .take()
                                .expect("a THEN expression always follows a WHEN expression");
                            whens.push((w, expr));
                        }
                        Pos::AfterElse => else_ = Some(Box::new(expr)),
                    }
                }
            }
        }
        Ok(ReturnExpr::Case { test, whens, else_ })
    }

    /// `filterExpression : symbol IN expression where?` -- shared by
    /// `filterWith` (ALL/ANY/NONE/SINGLE quantifiers) and
    /// `listComprehension`, both of which bind one variable over a source
    /// list, optionally filtered.
    fn build_filter_expression(
        &mut self,
        ctx: &FilterExpressionContext,
    ) -> Result<(String, ReturnExpr, Option<Box<ReturnExpr>>), QueryError> {
        let var_ctx = ctx.symbol().expect("filterExpression always has a symbol");
        let var = symbol_text(&var_ctx);
        let source_ctx = ctx
            .expression()
            .expect("filterExpression always has an expression");
        let source = self.visit(&*source_ctx).into_return_expr()?;
        let where_clause = match ctx.where_() {
            Some(where_ctx) => {
                let expr_ctx = where_ctx
                    .expression()
                    .expect("where always has an expression");
                Some(Box::new(self.visit(&*expr_ctx).into_return_expr()?))
            }
            None => None,
        };
        Ok((var, source, where_clause))
    }

    /// `filterWith : (ALL | ANY | NONE | SINGLE) LPAREN filterExpression
    /// RPAREN` -- `ReturnExpr::Quantifier`, always evaluates to a bool
    /// (`where_clause` absent means "every element's own truthiness", same
    /// convention `Quantifier::where_clause`'s own docs describe).
    fn build_filter_with(&mut self, ctx: &FilterWithContext) -> Result<ReturnExpr, QueryError> {
        let kind = if ctx.ALL().is_some() {
            QuantifierKind::All
        } else if ctx.ANY().is_some() {
            QuantifierKind::Any
        } else if ctx.NONE().is_some() {
            QuantifierKind::None
        } else {
            ctx.SINGLE()
                .expect("filterWith always has one of ALL/ANY/NONE/SINGLE");
            QuantifierKind::Single
        };
        let fe_ctx = ctx
            .filterExpression()
            .expect("filterWith always has a filterExpression");
        let (var, source, where_clause) = self.build_filter_expression(&fe_ctx)?;
        Ok(ReturnExpr::Quantifier {
            kind,
            var,
            source: Box::new(source),
            where_clause,
        })
    }

    /// `listComprehension : LBRACK filterExpression (STICK expression)?
    /// RBRACK` -- `ctx.expression()` here is `listComprehension`'s own
    /// direct child (the `STICK`-following projection), not
    /// `filterExpression`'s nested one (a different context type, no
    /// ambiguity).
    fn build_list_comprehension(
        &mut self,
        ctx: &ListComprehensionContext,
    ) -> Result<ReturnExpr, QueryError> {
        let fe_ctx = ctx
            .filterExpression()
            .expect("listComprehension always has a filterExpression");
        let (var, source, where_clause) = self.build_filter_expression(&fe_ctx)?;
        let project = match ctx.expression() {
            Some(expr_ctx) => Some(Box::new(self.visit(&*expr_ctx).into_return_expr()?)),
            None => None,
        };
        Ok(ReturnExpr::ListComp {
            var,
            source: Box::new(source),
            where_clause,
            project,
        })
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
            // `projectionItems : (MULT | projectionItem) (COMMA
            // projectionItem)*` syntactically allows `RETURN *, x AS y`
            // (MULT first, then a COMMA'd projectionItem) -- but
            // `Tail::ReturnStar` has no field for extra items alongside
            // the star (unlike `WithClause`, which has both `star` and
            // `items`), so silently taking the star-only path here would
            // drop `x AS y` on the floor. Error instead.
            if !items_ctx.projectionItem_all().is_empty() {
                return Err(QueryError::Syntax(
                    "RETURN * can't be combined with additional items".into(),
                ));
            }
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

        let (order_by, skip, limit) = self.build_order_skip_limit(ctx)?;

        Ok(ParsedReturnClause {
            tail,
            order_by,
            skip,
            limit,
        })
    }

    /// Shared by `build_projection_body` (RETURN) and `build_with_clause`
    /// (WITH) -- both grammar rules bundle `orderSt`/`skipSt`/`limitSt`
    /// into the same `projectionBody`.
    #[allow(clippy::type_complexity)]
    fn build_order_skip_limit(
        &mut self,
        ctx: &ProjectionBodyContext,
    ) -> Result<(Option<Vec<(ReturnExpr, SortDir)>>, Option<i64>, Option<i64>), QueryError> {
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
        Ok((order_by, skip, limit))
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

    fn build_with_clause(&mut self, ctx: &WithStContext) -> Result<WithClause, QueryError> {
        let body_ctx = ctx
            .projectionBody()
            .expect("withSt always has a projectionBody");
        let distinct = body_ctx.DISTINCT().is_some();
        let items_ctx = body_ctx
            .projectionItems()
            .expect("projectionBody always has projectionItems");
        let star = items_ctx.MULT().is_some();
        let mut items = Vec::new();
        for item_ctx in items_ctx.projectionItem_all() {
            let expr_ctx = item_ctx
                .expression()
                .expect("projectionItem always has an expression");
            let expr = self.visit(&*expr_ctx).into_return_expr()?;
            let alias = item_ctx.symbol().map(|s| symbol_text(&s));
            items.push(ReturnItem { expr, alias });
        }
        let (order_by, skip, limit) = self.build_order_skip_limit(&body_ctx)?;
        let where_clause = match ctx.where_() {
            Some(where_ctx) => {
                let expr_ctx = where_ctx
                    .expression()
                    .expect("where always has an expression");
                let expr = self.visit(&*expr_ctx).into_return_expr()?;
                Some(return_expr_to_with_expr(expr))
            }
            None => None,
        };
        Ok(WithClause {
            items,
            star,
            distinct,
            where_clause,
            order_by,
            skip,
            limit,
        })
    }

    /// `UnwindClause::where_clause`/`::with` are populated wherever mars's
    /// own AST assembly attaches a following `WHERE`/`WITH` -- neither is
    /// part of `unwindSt`'s own grammar (`UNWIND expression AS symbol`,
    /// no trailing clauses at all), unlike pest's grammar, which does let
    /// UNWIND carry an inline WHERE directly (a mars-specific extension
    /// beyond real openCypher syntax, per `UnwindClause::where_clause`'s
    /// own docs). Always `None` here; a real capability gap versus pest
    /// for this specific extension, not a deferred-for-now stub.
    fn build_unwind_st(&mut self, ctx: &UnwindStContext) -> Result<UnwindClause, QueryError> {
        let expr_ctx = ctx.expression().expect("unwindSt always has an expression");
        let source = UnwindSource(self.visit(&*expr_ctx).into_return_expr()?);
        let var_ctx = ctx.symbol().expect("unwindSt always has a symbol");
        Ok(UnwindClause {
            source,
            var: symbol_text(&var_ctx),
            where_clause: None,
            with: None,
        })
    }

    fn build_set_st(&mut self, ctx: &SetStContext) -> Result<Vec<SetItem>, QueryError> {
        ctx.setItem_all()
            .into_iter()
            .map(|item_ctx| self.build_set_item(&item_ctx))
            .collect()
    }

    fn build_set_item(&mut self, ctx: &SetItemContextAll) -> Result<SetItem, QueryError> {
        // `setItem`'s first alternative is `propertyExpression ASSIGN
        // expression`, and `propertyExpression`'s own zero-`.name`-suffix
        // form degenerates to a bare variable -- so `n = {...}` (no dots
        // at all) parses through *this* alternative too, not the
        // `symbol ASSIGN expression` one below (which ANTLR only reaches
        // for `+=`, since alternative one has no ADD_ASSIGN option at
        // all). `build_property_expression`'s result tells them apart:
        // `Prop` is real `x.prop` access; `Var` is the degenerate case,
        // meaning `SetItem::MapAssign` (never `merge: true` here --
        // that's only reachable via `+=`, which can't take this branch).
        if let Some(prop_ctx) = ctx.propertyExpression() {
            let expr_ctx = ctx
                .expression()
                .expect("setItem's propertyExpression form always has an expression");
            return match self.build_property_expression(&prop_ctx)? {
                ReturnExpr::Prop(prop) => {
                    let value = self.visit(&*expr_ctx).into_return_expr()?;
                    Ok(SetItem::Prop(prop, value))
                }
                ReturnExpr::Var(var) => {
                    let value = self.visit(&*expr_ctx).into_return_expr()?;
                    Ok(SetItem::MapAssign {
                        var,
                        value,
                        merge: false,
                    })
                }
                _ => Err(QueryError::Syntax(
                    "expected a property access (x.prop) or variable on the left of SET's `=`"
                        .into(),
                )),
            };
        }
        let sym_ctx = ctx
            .symbol()
            .expect("setItem always has a propertyExpression or symbol");
        let var = symbol_text(&sym_ctx);
        if let Some(labels_ctx) = ctx.nodeLabels() {
            let labels = labels_ctx.name_all().iter().map(|n| name_text(n)).collect();
            return Ok(SetItem::Labels(var, labels));
        }
        let expr_ctx = ctx
            .expression()
            .expect("setItem's symbol-assign form always has an expression");
        let value = self.visit(&*expr_ctx).into_return_expr()?;
        Ok(SetItem::MapAssign {
            var,
            value,
            merge: ctx.ADD_ASSIGN().is_some(),
        })
    }

    fn build_delete_st(&mut self, ctx: &DeleteStContext) -> Result<ParsedDelete, QueryError> {
        let chain_ctx = ctx
            .expressionChain()
            .expect("deleteSt always has an expressionChain");
        let mut items = Vec::new();
        for expr_ctx in chain_ctx.expression_all() {
            items.push(self.visit(&*expr_ctx).into_return_expr()?);
        }
        Ok(ParsedDelete {
            items,
            detach: ctx.DETACH().is_some(),
        })
    }

    fn build_remove_st(&mut self, ctx: &RemoveStContext) -> Result<Vec<RemoveItem>, QueryError> {
        ctx.removeItem_all()
            .into_iter()
            .map(|item_ctx| self.build_remove_item(&item_ctx))
            .collect()
    }

    fn build_remove_item(&mut self, ctx: &RemoveItemContextAll) -> Result<RemoveItem, QueryError> {
        if let Some(prop_ctx) = ctx.propertyExpression() {
            return Ok(RemoveItem::Prop(self.build_prop_access(&prop_ctx)?));
        }
        let sym_ctx = ctx
            .symbol()
            .expect("removeItem always has a symbol+nodeLabels or a propertyExpression");
        let labels_ctx = ctx
            .nodeLabels()
            .expect("removeItem's symbol form always has nodeLabels");
        let labels = labels_ctx.name_all().iter().map(|n| name_text(n)).collect();
        Ok(RemoveItem::Labels(symbol_text(&sym_ctx), labels))
    }

    /// `propertyExpression`'s own grammar rule is reused by `setItem`/
    /// `removeItem` for their `x.prop` alternative -- `build_property_
    /// expression` already builds exactly `ReturnExpr::Prop` for that
    /// shape (or errors for anything wider, chained access etc), so this
    /// just unwraps the one variant these two callers can ever legally
    /// see here (the grammar alternative they're on doesn't admit a bare
    /// `symbol` or anything else propertyExpression could otherwise
    /// produce).
    fn build_prop_access(
        &mut self,
        ctx: &PropertyExpressionContext,
    ) -> Result<PropAccess, QueryError> {
        match self.build_property_expression(ctx)? {
            ReturnExpr::Prop(p) => Ok(p),
            _ => Err(QueryError::Syntax(
                "expected a property access (x.prop)".into(),
            )),
        }
    }

    /// `Statement::Create`'s `Vec<Pattern>` has no named-path-capture slot
    /// at all (unlike `QueryPart::path_var`), and unlike `MATCH`, CREATE's
    /// comma-separated patterns are never spliced into linear chains --
    /// each becomes its own independent `Pattern` directly (matches
    /// `parser.rs`'s `parse_create_patterns`, which does the same, no
    /// `group_into_linear_patterns` call).
    fn build_create_st(&mut self, ctx: &CreateStContext) -> Result<Vec<Pattern>, QueryError> {
        let pattern_ctx = ctx.pattern().expect("createSt always has a pattern");
        pattern_ctx
            .patternPart_all()
            .into_iter()
            .map(|part_ctx| {
                if part_ctx.ASSIGN().is_some() {
                    return Err(QueryError::Syntax(
                        "named-path capture (`p = ...`) isn't supported on CREATE".into(),
                    ));
                }
                if part_ctx.shortestPathWrapper().is_some() {
                    return Err(QueryError::Syntax(
                        "shortestPath() isn't valid in CREATE".into(),
                    ));
                }
                let elem_ctx = part_ctx.patternElem().expect(
                    "patternPart always has a patternElem when shortestPathWrapper is absent",
                );
                self.visit(&*elem_ctx).into_pattern()
            })
            .collect()
    }

    /// Mirrors `parser.rs`'s `parse_merge_clause`: `MergeClause::pattern`
    /// caps at one relationship hop (checked here, not the grammar, which
    /// permissively allows any hop count via the same `patternElem` every
    /// other pattern context uses), and real Cypher rejects more than one
    /// `ON CREATE`/`ON MATCH` on the same MERGE (also grammar-permissive,
    /// `mergeAction*` allows any order/count) -- same "grammar permissive,
    /// builder enforces the exact constraint" split used there. No
    /// named-path capture either, same reasoning as `build_create_st`.
    fn build_merge_st(&mut self, ctx: &MergeStContext) -> Result<MergeClause, QueryError> {
        let part_ctx = ctx.patternPart().expect("mergeSt always has a patternPart");
        if part_ctx.ASSIGN().is_some() {
            return Err(QueryError::Syntax(
                "named-path capture (`p = ...`) isn't supported on MERGE".into(),
            ));
        }
        if part_ctx.shortestPathWrapper().is_some() {
            return Err(QueryError::Syntax(
                "shortestPath() isn't valid in MERGE".into(),
            ));
        }
        let elem_ctx = part_ctx
            .patternElem()
            .expect("patternPart always has a patternElem when shortestPathWrapper is absent");
        let pattern = self.visit(&*elem_ctx).into_pattern()?;
        if pattern.hops.len() > 1 {
            return Err(QueryError::Syntax(
                "MERGE with more than one relationship hop isn't supported yet — split it into a MATCH \
                 for the already-known part and a MERGE for one new hop"
                    .into(),
            ));
        }

        let mut on_create = Vec::new();
        let mut on_match = Vec::new();
        for action_ctx in ctx.mergeAction_all() {
            let set_items = self.build_merge_action(&action_ctx)?;
            if action_ctx.MATCH().is_some() {
                if !on_match.is_empty() {
                    return Err(QueryError::Syntax(
                        "MERGE can have at most one ON MATCH SET clause".into(),
                    ));
                }
                on_match = set_items;
            } else {
                if !on_create.is_empty() {
                    return Err(QueryError::Syntax(
                        "MERGE can have at most one ON CREATE SET clause".into(),
                    ));
                }
                on_create = set_items;
            }
        }

        Ok(MergeClause {
            pattern,
            on_create,
            on_match,
            with: None,
        })
    }

    fn build_merge_action(
        &mut self,
        ctx: &MergeActionContextAll,
    ) -> Result<Vec<SetItem>, QueryError> {
        let set_ctx = ctx.setSt().expect("mergeAction always has a setSt");
        self.build_set_st(&set_ctx)
    }

    /// `readingStatement : matchSt | unwindSt | queryCallSt`. `matchSt` can
    /// expand to more than one `QueryClause::Match` (comma-separated
    /// disjoint patterns splice into separate `QueryPart`s -- see
    /// `build_match_st`'s docs), so this appends rather than returning a
    /// single clause. `queryCallSt` (`CALL proc(...) YIELD ...` used as a
    /// reading clause) has no `QueryClause` variant to build at all yet --
    /// CALL/YIELD support is a separate, tracked gap (beads mars-82w), not
    /// part of this pass.
    fn append_reading_statement(
        &mut self,
        ctx: &ReadingStatementContextAll,
        clauses: &mut Vec<QueryClause>,
    ) -> Result<(), QueryError> {
        if let Some(match_ctx) = ctx.matchSt() {
            let parts = self.visit(&*match_ctx).into_query_parts()?;
            clauses.extend(parts.into_iter().map(QueryClause::Match));
            return Ok(());
        }
        if let Some(unwind_ctx) = ctx.unwindSt() {
            let clause = self.visit(&*unwind_ctx).into_unwind_clause()?;
            clauses.push(QueryClause::Unwind(clause));
            return Ok(());
        }
        Err(QueryError::Syntax(
            "CALL isn't supported by the ANTLR parser yet".into(),
        ))
    }

    /// `updatingStatement : createSt | mergeSt | deleteSt | setSt |
    /// removeSt`, used where it's just another clause in the sequence (not
    /// the statement's final tail -- see `build_mutating_tail` for that
    /// position instead).
    fn build_updating_statement_as_clause(
        &mut self,
        ctx: &UpdatingStatementContextAll,
    ) -> Result<QueryClause, QueryError> {
        if let Some(create_ctx) = ctx.createSt() {
            return Ok(QueryClause::Create(
                self.visit(&*create_ctx).into_create_patterns()?,
            ));
        }
        if let Some(merge_ctx) = ctx.mergeSt() {
            return Ok(QueryClause::Merge(
                self.visit(&*merge_ctx).into_merge_clause()?,
            ));
        }
        if let Some(delete_ctx) = ctx.deleteSt() {
            let d = self.visit(&*delete_ctx).into_delete_items()?;
            return Ok(QueryClause::Delete {
                items: d.items,
                detach: d.detach,
            });
        }
        if let Some(set_ctx) = ctx.setSt() {
            return Ok(QueryClause::Set(self.visit(&*set_ctx).into_set_items()?));
        }
        let remove_ctx = ctx
            .removeSt()
            .expect("updatingStatement always has one of its 5 alternatives");
        Ok(QueryClause::Remove(
            self.visit(&*remove_ctx).into_remove_items()?,
        ))
    }

    /// The statement's final mutating clause (`createSt`/`deleteSt`/
    /// `setSt`/`removeSt` -- never `mergeSt`, which has no `Tail` variant
    /// at all and always becomes a `QueryClause::Merge` entry even when
    /// it's last, per `Statement::Match`'s own "missing tail is only valid
    /// with a MERGE clause" rule) folds into a `Tail::X(_, Option
    /// <ReturnTail>)`, consuming an optional trailing `returnSt` as a
    /// narrower `ReturnTail` (items + distinct only, matching pest's
    /// `ReturnTail`, which has no other fields either). `RETURN *` isn't
    /// supported in this position (`ReturnTail` has no star-resolution
    /// site -- mirrors `parser.rs`'s `parse_mutating_tail`, same
    /// real restriction there too, confirmed via the TCK). ORDER BY/SKIP/
    /// LIMIT, though, are NOT restricted here (an earlier version of this
    /// function wrongly rejected them, found via a full TCK parse-parity
    /// run -- `MATCH (n) DELETE n RETURN 42 LIMIT 0` is real, TCK-tested
    /// Cypher) -- returned to the caller instead, which places them on the
    /// *statement's* own `order_by`/`skip`/`limit` fields, same as pest:
    /// its `mutating_tail` rule has no order/skip/limit slot of its own at
    /// all, they're siblings of `tail_clause` at `match_stmt`'s own level
    /// (`clause* ~ tail_clause? ~ order_by_clause? ~ skip_clause? ~
    /// limit_clause?`), applying regardless of which `Tail` variant is
    /// active. This grammar just nests them inside `returnSt`'s own
    /// `projectionBody` structurally instead of keeping them as separate
    /// statement-level siblings -- same semantics, different grammar shape.
    #[allow(clippy::type_complexity)]
    fn build_mutating_tail(
        &mut self,
        ctx: &UpdatingStatementContextAll,
        return_ctx: Option<&ReturnStContext>,
    ) -> Result<
        (
            Tail,
            Option<Vec<(ReturnExpr, SortDir)>>,
            Option<i64>,
            Option<i64>,
        ),
        QueryError,
    > {
        let mut order_by = None;
        let mut skip = None;
        let mut limit = None;
        let ret = match return_ctx {
            Some(return_ctx) => {
                let c = self.visit(return_ctx).into_return_clause()?;
                order_by = c.order_by;
                skip = c.skip;
                limit = c.limit;
                let Tail::Return(items, distinct) = c.tail else {
                    return Err(QueryError::Syntax(
                        "RETURN * isn't supported as a mutating clause's own trailing RETURN"
                            .into(),
                    ));
                };
                Some(ReturnTail { items, distinct })
            }
            None => None,
        };
        let tail = if let Some(create_ctx) = ctx.createSt() {
            Tail::Create(self.visit(&*create_ctx).into_create_patterns()?, ret)
        } else if let Some(delete_ctx) = ctx.deleteSt() {
            let d = self.visit(&*delete_ctx).into_delete_items()?;
            if d.detach {
                Tail::DetachDelete(d.items, ret)
            } else {
                Tail::Delete(d.items, ret)
            }
        } else if let Some(set_ctx) = ctx.setSt() {
            Tail::Set(self.visit(&*set_ctx).into_set_items()?, ret)
        } else {
            let remove_ctx = ctx
                .removeSt()
                .expect("build_mutating_tail's caller already excluded mergeSt");
            Tail::Remove(self.visit(&*remove_ctx).into_remove_items()?, ret)
        };
        Ok((tail, order_by, skip, limit))
    }

    /// `singlePartQ : readingStatement* (returnSt | updatingStatement+
    /// returnSt?)`. No WITH chaining at this level at all (that's
    /// `multiPartQ`'s job, not yet wired up -- see this file's module
    /// doc). Mirrors `parser.rs`'s `parse_match_stmt` for the no-WITH
    /// case: leading reading statements become `QueryClause`s; either a
    /// bare `returnSt` becomes the statement's `Tail::Return`/`ReturnStar`
    /// (with ORDER BY/SKIP/LIMIT at the statement level, where they
    /// belong for this form), or the *last* updating statement becomes the
    /// tail (see `build_mutating_tail`) with every earlier one just
    /// another `QueryClause`, unless that last one is `mergeSt` (never a
    /// tail -- see that function's docs), in which case a trailing
    /// `returnSt`, if present, becomes the statement's own `Tail::Return`
    /// instead.
    fn build_single_part_q(&mut self, ctx: &SinglePartQContext) -> Result<Statement, QueryError> {
        let mut clauses = Vec::new();
        for rs_ctx in ctx.readingStatement_all() {
            self.append_reading_statement(&rs_ctx, &mut clauses)?;
        }

        let updating = ctx.updatingStatement_all();
        let return_ctx = ctx.returnSt();

        // Bare `CREATE (...)` with nothing else at all (no leading MATCH/
        // UNWIND, no trailing RETURN, no other updating clause) -- mirrors
        // pest's `create_stmt_only` (`create_stmt ~ !(return_clause |
        // chainable_clause_follows)`), producing a real `Statement::Create`
        // directly instead of wrapping in `Statement::Match` with a
        // `Tail::Create`. Found via a Phase 3 dry-run behavioral test
        // failure (`explain_never_mutates_even_a_write_statement`):
        // `explain.rs`'s "no query plan" output depends on this exact
        // shape distinction, not just equivalent semantics.
        if clauses.is_empty() && return_ctx.is_none() && updating.len() == 1 {
            if let Some(create_ctx) = updating[0].createSt() {
                let patterns = self.visit(&*create_ctx).into_create_patterns()?;
                return Ok(Statement::Create(patterns));
            }
        }

        let mut tail = None;
        let mut order_by = None;
        let mut skip = None;
        let mut limit = None;
        let mut consumed_return = false;

        if let Some((last, earlier)) = updating.split_last() {
            for us_ctx in earlier {
                clauses.push(self.build_updating_statement_as_clause(us_ctx)?);
            }
            if last.mergeSt().is_some() {
                clauses.push(self.build_updating_statement_as_clause(last)?);
            } else {
                let (t, ob, sk, lim) = self.build_mutating_tail(last, return_ctx.as_deref())?;
                tail = Some(t);
                order_by = ob;
                skip = sk;
                limit = lim;
                consumed_return = return_ctx.is_some();
            }
        }

        if !consumed_return {
            if let Some(return_ctx) = return_ctx {
                let c = self.visit(&*return_ctx).into_return_clause()?;
                tail = Some(c.tail);
                order_by = c.order_by;
                skip = c.skip;
                limit = c.limit;
            }
        }

        if tail.is_none() && !clauses.iter().any(|c| matches!(c, QueryClause::Merge(_))) {
            return Err(QueryError::Syntax(
                "a query needs a RETURN/DELETE/SET tail, unless it has a MERGE clause with nothing after it".into(),
            ));
        }

        Ok(Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        })
    }

    /// `multiPartQ : readingStatement* ((readingStatement | updatingStatement)*
    /// withSt)+ singlePartQ` -- one or more WITH boundaries, each preceded by
    /// zero or more reading/updating statements, followed by a final
    /// `singlePartQ` (itself another `readingStatement*` run plus the
    /// statement's real tail). The grammar's typed accessors
    /// (`readingStatement_all`/`updatingStatement_all`/`withSt_all`) each
    /// flatten across every group, losing which items came before which
    /// `withSt` -- recovered by sorting all three by source position
    /// (`start().get_token_index()`) instead of walking raw children (which
    /// would need runtime downcasting to tell a `readingStatement` from an
    /// `updatingStatement` from a `withSt`).
    ///
    /// A `withSt` attaches to the immediately preceding MATCH/UNWIND/MERGE
    /// clause's own `with` field (only the *last* one, for a comma
    /// cross-join `matchSt`) -- same as `parser.rs`'s `parse_match_part`/
    /// `parse_merge_clause`/`parse_unwind_clause`. If nothing attachable
    /// immediately precedes it (statement-leading, or right after a
    /// SET/DELETE/REMOVE/CREATE -- none of which have a `with` field on
    /// their `QueryClause` variant -- or right after another `withSt`), it
    /// becomes its own standalone `QueryClause::With` entry, mirroring
    /// pest's `clause = { ... | with_clause | ... }` alternative.
    fn build_multi_part_q(&mut self, ctx: &MultiPartQContext) -> Result<Statement, QueryError> {
        enum Item<'i> {
            Reading(Rc<ReadingStatementContextAll<'i>>),
            Updating(Rc<UpdatingStatementContextAll<'i>>),
            With(Rc<WithStContext<'i>>),
        }
        let mut items: Vec<(isize, Item)> = Vec::new();
        for rs in ctx.readingStatement_all() {
            let idx = rs.start().get_token_index();
            items.push((idx, Item::Reading(rs)));
        }
        for us in ctx.updatingStatement_all() {
            let idx = us.start().get_token_index();
            items.push((idx, Item::Updating(us)));
        }
        for w in ctx.withSt_all() {
            let idx = w.start().get_token_index();
            items.push((idx, Item::With(w)));
        }
        items.sort_by_key(|(idx, _)| *idx);

        let mut clauses: Vec<QueryClause> = Vec::new();
        let mut attach_target: Option<usize> = None;
        for (_, item) in items {
            match item {
                Item::Reading(rs) => {
                    self.append_reading_statement(&rs, &mut clauses)?;
                    attach_target = Some(clauses.len() - 1);
                }
                Item::Updating(us) => {
                    let clause = self.build_updating_statement_as_clause(&us)?;
                    let can_attach = matches!(clause, QueryClause::Merge(_));
                    clauses.push(clause);
                    attach_target = can_attach.then_some(clauses.len() - 1);
                }
                Item::With(w) => {
                    let with = self.visit(&*w).into_with_clause()?;
                    match attach_target.take() {
                        Some(i) => match &mut clauses[i] {
                            QueryClause::Match(part) => part.with = Some(with),
                            QueryClause::Unwind(u) => u.with = Some(with),
                            QueryClause::Merge(m) => m.with = Some(with),
                            _ => unreachable!(
                                "attach_target is only ever set right after pushing a Match/Unwind/Merge clause"
                            ),
                        },
                        None => clauses.push(QueryClause::With(with)),
                    }
                }
            }
        }

        let sp_ctx = ctx
            .singlePartQ()
            .expect("multiPartQ always ends in a singlePartQ");
        // `build_single_part_q` can also return a bare `Statement::Create`
        // directly (its own "CREATE with nothing else at all" special
        // case, mirroring pest's `create_stmt_only`) -- but nested inside
        // a `multiPartQ` (past at least one `WITH` boundary already),
        // that's still just this statement's final `Tail::Create`, same
        // as an ordinary trailing `CREATE` would be. Only a genuinely
        // top-level, whole-statement bare CREATE gets the dedicated
        // `Statement::Create` shape (`explain.rs`'s "no query plan" case).
        let (tail_clauses, tail, order_by, skip, limit) =
            match self.build_single_part_q(&sp_ctx)? {
                Statement::Match {
                    clauses,
                    tail,
                    order_by,
                    skip,
                    limit,
                } => (clauses, tail, order_by, skip, limit),
                Statement::Create(patterns) => {
                    (Vec::new(), Some(Tail::Create(patterns, None)), None, None, None)
                }
                other => unreachable!(
                    "build_single_part_q only ever returns Statement::Match or Statement::Create, got {other:?}"
                ),
            };
        clauses.extend(tail_clauses);
        Ok(Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        })
    }

    /// `explainSt : EXPLAIN (createIndexSt | regularQuery)` -- mars-specific
    /// grammar extension (this file's own local addition, not from
    /// upstream `antlr/grammars-v4/cypher`; see `grammar/README.md`), no
    /// real openCypher equivalent. Mirrors `parser.rs`'s `parse_explain_stmt`.
    fn build_explain_st(&mut self, ctx: &ExplainStContext) -> Result<Statement, QueryError> {
        let inner = match ctx.createIndexSt() {
            Some(ci_ctx) => self.build_create_index_st(&ci_ctx)?,
            None => {
                let rq_ctx = ctx
                    .regularQuery()
                    .expect("explainSt always has a createIndexSt or regularQuery");
                self.visit(&*rq_ctx).into_statement()?
            }
        };
        Ok(Statement::Explain(Box::new(inner)))
    }

    /// `createIndexSt : CREATE INDEX ON COLON name LPAREN name RPAREN
    /// UNIQUE?` -- same mars-specific-extension caveat as `build_explain_st`
    /// above. Mirrors `parser.rs`'s `parse_create_index_stmt`; `name_all()`
    /// returns the label then the property name in source order (the only
    /// two `name` children this rule ever has).
    fn build_create_index_st(
        &mut self,
        ctx: &CreateIndexStContext,
    ) -> Result<Statement, QueryError> {
        let names = ctx.name_all();
        let label = name_text(
            names
                .first()
                .expect("createIndexSt always has a label name"),
        );
        let prop = name_text(
            names
                .get(1)
                .expect("createIndexSt always has a property name"),
        );
        Ok(Statement::CreateIndex {
            label,
            prop,
            unique: ctx.UNIQUE().is_some(),
        })
    }

    /// `regularQuery : singleQuery unionSt*`. No `unionSt` at all just
    /// passes the single `Statement` straight through -- `singleQuery`
    /// itself (`singlePartQ | multiPartQ`) needs no override, default
    /// dispatch already routes to whichever of those two produced the
    /// `Statement`. Otherwise mirrors `parser.rs`'s `parse_union_stmt`:
    /// every `unionSt`'s `ALL` presence must agree (real Cypher rejects
    /// mixing bare `UNION` and `UNION ALL` in one statement), checked here
    /// rather than in the grammar since it's only knowable once every
    /// occurrence is in hand.
    fn build_regular_query(&mut self, ctx: &RegularQueryContext) -> Result<Statement, QueryError> {
        let sq_ctx = ctx
            .singleQuery()
            .expect("regularQuery always has a singleQuery");
        let first = self.visit(&*sq_ctx).into_statement()?;
        let unions = ctx.unionSt_all();
        if unions.is_empty() {
            return Ok(first);
        }
        let mut parts = vec![first];
        let mut all: Option<bool> = None;
        for u_ctx in unions {
            let this_all = u_ctx.ALL().is_some();
            match all {
                None => all = Some(this_all),
                Some(prev) if prev != this_all => {
                    return Err(QueryError::Syntax(
                        "can't mix UNION and UNION ALL in the same statement".into(),
                    ));
                }
                Some(_) => {}
            }
            let part_sq = u_ctx
                .singleQuery()
                .expect("unionSt always has a singleQuery");
            parts.push(self.visit(&*part_sq).into_statement()?);
        }
        Ok(Statement::Union {
            parts,
            all: all.unwrap_or(false),
        })
    }
}

/// The real implementation behind `lib.rs`'s public `parse` (Phase 3
/// cutover, mars-cuk/mars-nog) -- the pest-based `parser.rs`/`cypher.pest`
/// this replaced are gone (see `grammar/README.md`). CALL is the one
/// remaining gap versus the old parser, and it's not a regression: pest
/// never supported it either (tracked separately as mars-82w).
pub fn parse_antlr(input: &str) -> Result<Statement, QueryError> {
    use crate::generated::cypherlexer::CypherLexer;
    use crate::generated::cypherparser::{CypherParser, ScriptContextAttrs};
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::error_listener::ErrorListener;
    use antlr4rust::recognizer::Recognizer;
    use antlr4rust::token_factory::TokenFactory;
    use antlr4rust::InputStream;
    use antlr4rust::Parser as _;
    use std::cell::RefCell;

    struct CollectErrors(Rc<RefCell<Vec<String>>>);
    impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for CollectErrors {
        fn syntax_error(
            &self,
            _recognizer: &T,
            _offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
            line: isize,
            column: isize,
            msg: &str,
            _e: Option<&antlr4rust::errors::ANTLRError>,
        ) {
            self.0
                .borrow_mut()
                .push(format!("line {line}:{column} {msg}"));
        }
    }

    let errors = Rc::new(RefCell::new(Vec::new()));
    let stream = InputStream::new(input);
    let mut lexer = CypherLexer::new(stream);
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(CollectErrors(errors.clone())));
    let tokens = CommonTokenStream::new(lexer);
    let mut parser = CypherParser::new(tokens);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(CollectErrors(errors.clone())));
    let ctx = parser
        .script()
        .map_err(|e| QueryError::Syntax(e.to_string()))?;
    if let Some(msg) = errors.borrow().first() {
        return Err(QueryError::Syntax(format!("syntax error: {msg}")));
    }
    // `script : query SEMI? EOF` -- visiting the whole tree would run
    // straight into the default `aggregate_results`' unconditional
    // "last child wins" rule (not "last *non-default*", despite this
    // file's other alternation rules getting away with relying on that
    // distinction -- see this function's own docs): the trailing `EOF`
    // terminal has no `visit_X` hook of its own, so it'd overwrite
    // `query`'s real result with `AstNode::None`. Visiting `query`
    // directly sidesteps it -- `script`'s own job (rejecting trailing
    // garbage after a valid query) is already done by the `parser.script()`
    // call above succeeding.
    let query_ctx = ctx.query().expect("script always has a query");
    AstBuilder::new().visit(&*query_ctx).into_statement()
}

/// The real implementation behind `lib.rs`'s public `parse_many` (Phase 3
/// cutover) -- parses a `;`-separated batch of one or more statements
/// (`"CREATE (a); CREATE (b); MATCH (n) RETURN n"`). `queries : query
/// (SEMI query)* EOF` is a mars-specific grammar extension (see
/// `grammar/README.md`), including stripping a single genuinely-trailing
/// `;` in Rust before parsing -- the grammar rule itself has no trailing
/// `SEMI?`, to avoid a `queries`/`script` prefix ambiguity.
pub fn parse_antlr_many(input: &str) -> Result<Vec<Statement>, QueryError> {
    use crate::generated::cypherlexer::CypherLexer;
    use crate::generated::cypherparser::{CypherParser, QueriesContextAttrs};
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::error_listener::ErrorListener;
    use antlr4rust::recognizer::Recognizer;
    use antlr4rust::token_factory::TokenFactory;
    use antlr4rust::InputStream;
    use antlr4rust::Parser as _;
    use std::cell::RefCell;

    struct CollectErrors(Rc<RefCell<Vec<String>>>);
    impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for CollectErrors {
        fn syntax_error(
            &self,
            _recognizer: &T,
            _offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
            line: isize,
            column: isize,
            msg: &str,
            _e: Option<&antlr4rust::errors::ANTLRError>,
        ) {
            self.0
                .borrow_mut()
                .push(format!("line {line}:{column} {msg}"));
        }
    }

    let trimmed = input.trim_end();
    let trimmed = trimmed.strip_suffix(';').unwrap_or(trimmed);

    let errors = Rc::new(RefCell::new(Vec::new()));
    let stream = InputStream::new(trimmed);
    let mut lexer = CypherLexer::new(stream);
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(CollectErrors(errors.clone())));
    let tokens = CommonTokenStream::new(lexer);
    let mut parser = CypherParser::new(tokens);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(CollectErrors(errors.clone())));
    let ctx = parser
        .queries()
        .map_err(|e| QueryError::Syntax(e.to_string()))?;
    if let Some(msg) = errors.borrow().first() {
        return Err(QueryError::Syntax(format!("syntax error: {msg}")));
    }
    ctx.query_all()
        .into_iter()
        .map(|q| AstBuilder::new().visit(&*q).into_statement())
        .collect()
}

/// `where`'s grammar reuses the same `expression` rule as everywhere else
/// (unlike pest, which has a separate, narrower `with_expr` grammar chain
/// building `WithExpr` directly) -- so a full `ReturnExpr` has to be built
/// first and then folded down into `WithExpr` here. Only the variants with
/// an exact `WithExpr` counterpart (`And`/`Or`/`Not`/`Compare`/`IsNull`)
/// unwrap recursively; everything else (including `Xor`, which `WithExpr`
/// has no variant for at all) becomes `Bare` -- `WithExpr::Bare`'s own
/// docs already cover "any boolean-valued expression used directly as a
/// predicate", which this falls under regardless of its exact shape.
fn return_expr_to_with_expr(expr: ReturnExpr) -> WithExpr {
    match expr {
        ReturnExpr::And(l, r) => WithExpr::And(
            Box::new(return_expr_to_with_expr(*l)),
            Box::new(return_expr_to_with_expr(*r)),
        ),
        ReturnExpr::Or(l, r) => WithExpr::Or(
            Box::new(return_expr_to_with_expr(*l)),
            Box::new(return_expr_to_with_expr(*r)),
        ),
        ReturnExpr::Not(inner) => WithExpr::Not(Box::new(return_expr_to_with_expr(*inner))),
        ReturnExpr::Compare(l, op, r) => WithExpr::Compare(*l, op, *r),
        ReturnExpr::IsNull(inner) => WithExpr::IsNull(*inner),
        other => WithExpr::Bare(other),
    }
}

/// `matchSt`'s `where`, like `withSt`'s, reuses the same generic
/// `expression` rule as everywhere else (unlike pest, which has dedicated
/// narrower grammar rules -- `comparison`/`general_comparison`/
/// `label_predicate`/`var_compare` -- picking the right `Expr` variant
/// directly at parse time). So the same fold-down-after-the-fact approach
/// as `return_expr_to_with_expr` applies here too, just against `Expr`'s
/// wider shape: a `Compare` between two bare `Prop`s becomes `PropCompare`,
/// a `Prop` compared to a `Lit` keeps the planner-fusable `Compare` variant
/// pest's `comparison` rule reserves for exactly that shape, two bare
/// `Var`s becomes identity comparison (`VarEq`/`Not(VarEq)`, matching
/// pest's `var_compare`'s restriction to `=`/`<>` — anything else is a real
/// error, not a silent `GeneralCompare` fallback, since no ordering exists
/// between two nodes/relationships), anything else falls back to
/// `GeneralCompare`. Similarly `IsNull` on a bare `Prop` keeps the narrow
/// variant, anything else becomes `GeneralIsNull`. `HasLabel` folds
/// multiple labels into a `HasLabel` `And` chain exactly like pest's
/// `parse_label_predicate`. Everything else becomes `GeneralBare`.
fn return_expr_to_expr(expr: ReturnExpr) -> Result<Expr, QueryError> {
    Ok(match expr {
        ReturnExpr::And(l, r) => Expr::And(
            Box::new(return_expr_to_expr(*l)?),
            Box::new(return_expr_to_expr(*r)?),
        ),
        ReturnExpr::Or(l, r) => Expr::Or(
            Box::new(return_expr_to_expr(*l)?),
            Box::new(return_expr_to_expr(*r)?),
        ),
        ReturnExpr::Not(inner) => Expr::Not(Box::new(return_expr_to_expr(*inner)?)),
        ReturnExpr::Compare(l, op, r) => match (*l, *r) {
            (ReturnExpr::Prop(pa), ReturnExpr::Lit(lit)) => Expr::Compare(pa, op, lit),
            (ReturnExpr::Prop(pa1), ReturnExpr::Prop(pa2)) => Expr::PropCompare(pa1, op, pa2),
            (ReturnExpr::Var(a), ReturnExpr::Var(b)) => match op {
                CompareOp::Eq => Expr::VarEq(a, b),
                CompareOp::Ne => Expr::Not(Box::new(Expr::VarEq(a, b))),
                _ => {
                    return Err(QueryError::Syntax(format!(
                        "{a} {op:?} {b}: only = and <> are meaningful for comparing two \
                         nodes/relationships by identity (no ordering exists between them)"
                    )))
                }
            },
            (l, r) => Expr::GeneralCompare(l, op, r),
        },
        ReturnExpr::IsNull(inner) => match *inner {
            ReturnExpr::Prop(pa) => Expr::IsNull(pa),
            other => Expr::GeneralIsNull(other),
        },
        ReturnExpr::HasLabel(var, labels) => {
            let mut labels = labels.into_iter();
            let first = labels
                .next()
                .expect("HasLabel always carries at least one label");
            labels.fold(Expr::HasLabel(var.clone(), first), |acc, label| {
                Expr::And(Box::new(acc), Box::new(Expr::HasLabel(var.clone(), label)))
            })
        }
        ReturnExpr::PatternPredicate(pattern) => Expr::Pattern(pattern),
        ReturnExpr::ExistsPattern {
            pattern,
            where_clause,
        } => Expr::Exists {
            pattern,
            where_clause,
        },
        other => Expr::GeneralBare(other),
    })
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

    fn parse_with(input: &str) -> Result<WithClause, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .withSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `withSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_with_clause()
    }

    fn parse_unwind(input: &str) -> Result<UnwindClause, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .unwindSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `unwindSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_unwind_clause()
    }

    fn parse_set(input: &str) -> Result<Vec<SetItem>, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .setSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `setSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_set_items()
    }

    fn parse_delete(input: &str) -> Result<ParsedDelete, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .deleteSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `deleteSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_delete_items()
    }

    fn parse_remove(input: &str) -> Result<Vec<RemoveItem>, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .removeSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `removeSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_remove_items()
    }

    fn parse_create(input: &str) -> Result<Vec<Pattern>, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .createSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `createSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_create_patterns()
    }

    fn parse_merge(input: &str) -> Result<MergeClause, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .mergeSt()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `mergeSt`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_merge_clause()
    }

    fn parse_statement(input: &str) -> Result<Statement, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .singlePartQ()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `singlePartQ`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_statement()
    }

    fn parse_multi_part_statement(input: &str) -> Result<Statement, QueryError> {
        let stream = InputStream::new(input);
        let lexer = CypherLexer::new(stream);
        let tokens = CommonTokenStream::new(lexer);
        let mut parser = CypherParser::new(tokens);
        let ctx = parser
            .multiPartQ()
            .unwrap_or_else(|e| panic!("failed to parse {input:?} as `multiPartQ`: {e:?}"));
        AstBuilder::new().visit(&*ctx).into_statement()
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
        // Both arrowheads (`<-...->`) is the same undirected/either shape
        // as neither -- regression found via the TCK (Match6 [12]/
        // Create2 [20]/mars-w37): used to silently resolve to Left,
        // checking LT before GT and never noticing GT was also present.
        assert_eq!(
            parse_pattern("(a)<-->(b)").unwrap().hops[0].0.direction,
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
    fn node_pattern_properties() {
        let pattern = parse_pattern("(a {name: 'x', age: 1 + 1})").unwrap();
        assert_eq!(
            pattern.start.props,
            vec![
                (
                    "name".to_string(),
                    ReturnExpr::Lit(Literal::String("x".to_string()))
                ),
                (
                    "age".to_string(),
                    ReturnExpr::Arith(
                        Box::new(ReturnExpr::Lit(Literal::Int(1))),
                        ArithOp::Add,
                        Box::new(ReturnExpr::Lit(Literal::Int(1))),
                    )
                ),
            ]
        );
    }

    #[test]
    fn rel_pattern_properties() {
        let pattern = parse_pattern("(a)-[:T {weight: 5}]->(b)").unwrap();
        assert_eq!(
            pattern.hops[0].0.props,
            vec![("weight".to_string(), ReturnExpr::Lit(Literal::Int(5)))]
        );
    }

    #[test]
    fn pattern_properties_parameter_not_supported() {
        assert!(parse_pattern("(a $props)").is_err());
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
    fn shortest_path() {
        let parts = parse_match("MATCH shortestPath((a)-[*1..3]->(b))").unwrap();
        assert_eq!(parts.len(), 1);
        assert!(parts[0].shortest_path);
        assert_eq!(parts[0].pattern.hops.len(), 1);
    }

    #[test]
    fn shortest_path_with_named_path_capture() {
        let parts = parse_match("MATCH p = shortestPath((a)-[*1..3]->(b))").unwrap();
        assert_eq!(parts[0].path_var.as_deref(), Some("p"));
        assert!(parts[0].shortest_path);
    }

    #[test]
    fn shortest_path_requires_variable_length_hop() {
        assert!(parse_match("MATCH shortestPath((a)-->(b))").is_err());
    }

    #[test]
    fn shortest_path_not_first_in_cross_join_errors() {
        assert!(parse_match("MATCH (c), shortestPath((a)-[*1..3]->(b))").is_err());
    }

    #[test]
    fn shortest_path_over_disjoint_cross_join_errors() {
        assert!(parse_match("MATCH shortestPath((a)-[*1..3]->(b)), (c)").is_err());
    }

    #[test]
    fn shortest_path_not_valid_in_create() {
        assert!(parse_statement("CREATE shortestPath((a)-[*1..3]->(b))").is_err());
    }

    #[test]
    fn shortest_path_not_valid_in_merge() {
        assert!(parse_merge("MERGE shortestPath((a)-[*1..3]->(b))").is_err());
    }

    #[test]
    fn named_path_over_variable_length_errors() {
        assert!(parse_match("MATCH p = (a)-[*1..3]->(b)").is_err());
    }

    #[test]
    fn match_where() {
        let parts = parse_match("MATCH (a) WHERE a.x = 1").unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(
            parts[0].where_clause,
            Some(Expr::Compare(
                PropAccess { .. },
                CompareOp::Eq,
                Literal::Int(1),
            ))
        ));
    }

    #[test]
    fn match_where_var_eq() {
        let parts = parse_match("MATCH (a), (b) WHERE a = b").unwrap();
        assert!(matches!(parts[1].where_clause, Some(Expr::VarEq(_, _))));
    }

    #[test]
    fn match_where_label_predicate() {
        let parts = parse_match("MATCH (a) WHERE a:A:B").unwrap();
        assert!(matches!(parts[0].where_clause, Some(Expr::And(_, _))));
    }

    #[test]
    fn match_where_pattern_predicate() {
        let parts = parse_match("MATCH (n) WHERE (n)-[]->() RETURN n")
            .unwrap_or_else(|e| panic!("expected pattern predicate to parse, got {e:?}"));
        let Some(Expr::Pattern(pattern)) = &parts[0].where_clause else {
            panic!("expected Expr::Pattern");
        };
        assert_eq!(pattern.hops.len(), 1);
    }

    #[test]
    fn match_where_pattern_predicate_combined_with_and() {
        let parts = parse_match("MATCH (n) WHERE (n)-->() AND n.x = 1").unwrap();
        let Some(Expr::And(l, r)) = &parts[0].where_clause else {
            panic!("expected Expr::And");
        };
        assert!(matches!(**l, Expr::Pattern(_)));
        assert!(matches!(**r, Expr::Compare(..)));
    }

    #[test]
    fn pattern_predicate_outside_where_still_parses() {
        // Grammatically legal anywhere an expression is (real Cypher
        // restricts it to WHERE) -- parses fine as a ReturnExpr;
        // semantic::infer_expr is what rejects it outside a WHERE-folded
        // position, at compile time (see that function's own docs).
        let expr = parse_expr("(n)-->()").unwrap();
        assert!(matches!(expr, ReturnExpr::PatternPredicate(_)));
    }

    #[test]
    fn match_where_on_last_group_of_cross_join() {
        let parts = parse_match("MATCH (a), (b) WHERE b.x = 1").unwrap();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].where_clause.is_none());
        assert!(parts[1].where_clause.is_some());
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
    fn is_null_binds_looser_than_arithmetic() {
        // Precedence bug found via a Phase 3 behavioral dry-run: `IS
        // NULL`/`IN`/`STARTS WITH` etc must bind above `+`/`-`/`*`/`/`/`^`
        // (openCypher.bnf's <comparison predicate> chain), so `x + 0 IS
        // NULL` is `(x + 0) IS NULL`, not `x + (0 IS NULL)`.
        assert_eq!(
            parse_expr("x + 0 IS NULL").unwrap(),
            ReturnExpr::IsNull(Box::new(ReturnExpr::Arith(
                Box::new(ReturnExpr::Var("x".to_string())),
                ArithOp::Add,
                Box::new(ReturnExpr::Lit(Literal::Int(0))),
            )))
        );
    }

    #[test]
    fn in_binds_looser_than_arithmetic_and_operand_can_be_sliced() {
        assert_eq!(
            parse_expr("3 IN [1, 2, 3][0..2]").unwrap(),
            ReturnExpr::In(
                Box::new(ReturnExpr::Lit(Literal::Int(3))),
                Box::new(ReturnExpr::Slice(
                    Box::new(ReturnExpr::ListLit(vec![
                        ReturnExpr::Lit(Literal::Int(1)),
                        ReturnExpr::Lit(Literal::Int(2)),
                        ReturnExpr::Lit(Literal::Int(3)),
                    ])),
                    Some(Box::new(ReturnExpr::Lit(Literal::Int(0)))),
                    Some(Box::new(ReturnExpr::Lit(Literal::Int(2)))),
                ))
            )
        );
    }

    #[test]
    fn starts_with_operand_can_be_an_arithmetic_expression() {
        assert_eq!(
            parse_expr("x STARTS WITH y + z").unwrap(),
            ReturnExpr::Compare(
                Box::new(ReturnExpr::Var("x".to_string())),
                CompareOp::StartsWith,
                Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Var("y".to_string())),
                    ArithOp::Add,
                    Box::new(ReturnExpr::Var("z".to_string())),
                )),
            )
        );
    }

    #[test]
    fn chained_index_postfix_still_works() {
        assert_eq!(
            parse_expr("[[1, 2], [3, 4]][0][1]").unwrap(),
            ReturnExpr::Index(
                Box::new(ReturnExpr::Index(
                    Box::new(ReturnExpr::ListLit(vec![
                        ReturnExpr::ListLit(vec![
                            ReturnExpr::Lit(Literal::Int(1)),
                            ReturnExpr::Lit(Literal::Int(2)),
                        ]),
                        ReturnExpr::ListLit(vec![
                            ReturnExpr::Lit(Literal::Int(3)),
                            ReturnExpr::Lit(Literal::Int(4)),
                        ]),
                    ])),
                    Box::new(ReturnExpr::Lit(Literal::Int(0))),
                )),
                Box::new(ReturnExpr::Lit(Literal::Int(1))),
            )
        );
    }

    #[test]
    fn case_searched_form() {
        assert_eq!(
            parse_expr("CASE WHEN x > 1 THEN 'big' WHEN x > 0 THEN 'small' ELSE 'none' END")
                .unwrap(),
            ReturnExpr::Case {
                test: None,
                whens: vec![
                    (
                        ReturnExpr::Compare(
                            Box::new(ReturnExpr::Var("x".to_string())),
                            CompareOp::Gt,
                            Box::new(ReturnExpr::Lit(Literal::Int(1))),
                        ),
                        ReturnExpr::Lit(Literal::String("big".to_string())),
                    ),
                    (
                        ReturnExpr::Compare(
                            Box::new(ReturnExpr::Var("x".to_string())),
                            CompareOp::Gt,
                            Box::new(ReturnExpr::Lit(Literal::Int(0))),
                        ),
                        ReturnExpr::Lit(Literal::String("small".to_string())),
                    ),
                ],
                else_: Some(Box::new(ReturnExpr::Lit(Literal::String(
                    "none".to_string()
                )))),
            }
        );
    }

    #[test]
    fn case_simple_form_with_test_no_else() {
        assert_eq!(
            parse_expr("CASE x WHEN 1 THEN 'one' WHEN 2 THEN 'two' END").unwrap(),
            ReturnExpr::Case {
                test: Some(Box::new(ReturnExpr::Var("x".to_string()))),
                whens: vec![
                    (
                        ReturnExpr::Lit(Literal::Int(1)),
                        ReturnExpr::Lit(Literal::String("one".to_string())),
                    ),
                    (
                        ReturnExpr::Lit(Literal::Int(2)),
                        ReturnExpr::Lit(Literal::String("two".to_string())),
                    ),
                ],
                else_: None,
            }
        );
    }

    #[test]
    fn quantifier_none() {
        assert_eq!(
            parse_expr("none(x IN [1,2] WHERE x > 1)").unwrap(),
            ReturnExpr::Quantifier {
                kind: QuantifierKind::None,
                var: "x".to_string(),
                source: Box::new(ReturnExpr::ListLit(vec![
                    ReturnExpr::Lit(Literal::Int(1)),
                    ReturnExpr::Lit(Literal::Int(2)),
                ])),
                where_clause: Some(Box::new(ReturnExpr::Compare(
                    Box::new(ReturnExpr::Var("x".to_string())),
                    CompareOp::Gt,
                    Box::new(ReturnExpr::Lit(Literal::Int(1))),
                ))),
            }
        );
    }

    #[test]
    fn quantifier_all_any_single_no_where() {
        assert!(matches!(
            parse_expr("all(x IN [1]) ").unwrap(),
            ReturnExpr::Quantifier {
                kind: QuantifierKind::All,
                where_clause: None,
                ..
            }
        ));
        assert!(matches!(
            parse_expr("any(x IN [1])").unwrap(),
            ReturnExpr::Quantifier {
                kind: QuantifierKind::Any,
                ..
            }
        ));
        assert!(matches!(
            parse_expr("single(x IN [1])").unwrap(),
            ReturnExpr::Quantifier {
                kind: QuantifierKind::Single,
                ..
            }
        ));
    }

    #[test]
    fn list_comprehension_with_projection() {
        assert_eq!(
            parse_expr("[x IN [1,2] WHERE x > 1 | x * 2]").unwrap(),
            ReturnExpr::ListComp {
                var: "x".to_string(),
                source: Box::new(ReturnExpr::ListLit(vec![
                    ReturnExpr::Lit(Literal::Int(1)),
                    ReturnExpr::Lit(Literal::Int(2)),
                ])),
                where_clause: Some(Box::new(ReturnExpr::Compare(
                    Box::new(ReturnExpr::Var("x".to_string())),
                    CompareOp::Gt,
                    Box::new(ReturnExpr::Lit(Literal::Int(1))),
                ))),
                project: Some(Box::new(ReturnExpr::Arith(
                    Box::new(ReturnExpr::Var("x".to_string())),
                    ArithOp::Mul,
                    Box::new(ReturnExpr::Lit(Literal::Int(2))),
                ))),
            }
        );
    }

    #[test]
    fn list_comprehension_with_where_no_project() {
        assert_eq!(
            parse_expr("[x IN [1,2] WHERE x > 1]").unwrap(),
            ReturnExpr::ListComp {
                var: "x".to_string(),
                source: Box::new(ReturnExpr::ListLit(vec![
                    ReturnExpr::Lit(Literal::Int(1)),
                    ReturnExpr::Lit(Literal::Int(2)),
                ])),
                where_clause: Some(Box::new(ReturnExpr::Compare(
                    Box::new(ReturnExpr::Var("x".to_string())),
                    CompareOp::Gt,
                    Box::new(ReturnExpr::Lit(Literal::Int(1))),
                ))),
                project: None,
            }
        );
    }

    #[test]
    fn list_comprehension_bare_identity_no_where_no_project() {
        // `[x IN list]` (neither WHERE nor `| project`) is genuinely
        // ambiguous with a one-element `listLit` containing the boolean
        // `x IN list` membership test -- `atom`'s alternatives are
        // ordered so `listComprehension` wins (real, spec-valid Cypher on
        // its own per openCypher.bnf's `<list comprehension>`, whose
        // filter/projection half is optional; found wrong via a Phase 3
        // behavioral dry-run, not the TCK).
        assert_eq!(
            parse_expr("[x IN [1, 2, 3]]").unwrap(),
            ReturnExpr::ListComp {
                var: "x".to_string(),
                source: Box::new(ReturnExpr::ListLit(vec![
                    ReturnExpr::Lit(Literal::Int(1)),
                    ReturnExpr::Lit(Literal::Int(2)),
                    ReturnExpr::Lit(Literal::Int(3)),
                ])),
                where_clause: None,
                project: None,
            }
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
    fn property_access_with_backtick_escaped_name() {
        // Regression (found via the TCK, Map1 [5]): `.get_text()` on the
        // `name` context kept the surrounding backticks as part of the
        // property name (`` `name` `` instead of `name`), so this always
        // looked up the wrong key. `name_text` strips them, same as
        // `symbol_text` already does for backtick-escaped variable names.
        assert_eq!(
            parse_expr("n.`weird name`").unwrap(),
            ReturnExpr::Prop(PropAccess {
                var: "n".to_string(),
                prop: "weird name".to_string(),
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

    #[test]
    fn return_star_with_extra_items_errors() {
        // projectionItems syntactically allows `* , x` (MULT then a
        // COMMA'd projectionItem), but Tail::ReturnStar has no field to
        // carry the extra item -- must error, not silently drop it.
        assert!(parse_return("RETURN *, x AS y").is_err());
    }

    #[test]
    fn with_items() {
        let c = parse_with("WITH a, b.name AS name").unwrap();
        assert!(!c.star);
        assert!(!c.distinct);
        assert_eq!(c.items.len(), 2);
        assert_eq!(c.items[0].expr, ReturnExpr::Var("a".to_string()));
        assert_eq!(c.items[1].alias.as_deref(), Some("name"));
    }

    #[test]
    fn with_star() {
        let c = parse_with("WITH *").unwrap();
        assert!(c.star);
        assert!(c.items.is_empty());
    }

    #[test]
    fn with_star_and_items() {
        // Unlike RETURN *, WithClause has both `star` and `items` fields
        // -- real Cypher's `WITH *, x AS y` is fully representable.
        let c = parse_with("WITH *, x AS y").unwrap();
        assert!(c.star);
        assert_eq!(c.items.len(), 1);
        assert_eq!(c.items[0].alias.as_deref(), Some("y"));
    }

    #[test]
    fn with_distinct_order_skip_limit() {
        let c = parse_with("WITH DISTINCT a ORDER BY a SKIP 1 LIMIT 2").unwrap();
        assert!(c.distinct);
        assert!(c.order_by.is_some());
        assert_eq!(c.skip, Some(1));
        assert_eq!(c.limit, Some(2));
    }

    #[test]
    fn with_where_compare() {
        let c = parse_with("WITH a WHERE a.x = 1").unwrap();
        let WithExpr::Compare(lhs, op, rhs) = c.where_clause.unwrap() else {
            panic!("expected WithExpr::Compare");
        };
        assert_eq!(
            lhs,
            ReturnExpr::Prop(PropAccess {
                var: "a".to_string(),
                prop: "x".to_string()
            })
        );
        assert_eq!(op, CompareOp::Eq);
        assert_eq!(rhs, ReturnExpr::Lit(Literal::Int(1)));
    }

    #[test]
    fn with_where_and_or_not() {
        let c = parse_with("WITH a WHERE NOT (a.x = 1 AND a.y = 2)").unwrap();
        assert!(matches!(c.where_clause.unwrap(), WithExpr::Not(_)));

        let c = parse_with("WITH a WHERE a.x = 1 OR a.y = 2").unwrap();
        assert!(matches!(c.where_clause.unwrap(), WithExpr::Or(_, _)));
    }

    #[test]
    fn with_where_is_null() {
        let c = parse_with("WITH a WHERE a IS NULL").unwrap();
        assert!(matches!(c.where_clause.unwrap(), WithExpr::IsNull(_)));
    }

    #[test]
    fn with_where_bare_expression() {
        // A boolean-valued expression with no comparison operator at all
        // (here: a HasLabel check) -- no exact WithExpr variant, so it
        // falls back to Bare rather than erroring.
        let c = parse_with("WITH n WHERE n:Person").unwrap();
        assert!(matches!(c.where_clause.unwrap(), WithExpr::Bare(_)));
    }

    #[test]
    fn with_where_xor_becomes_bare() {
        // WithExpr has no Xor variant at all -- confirmed falls back to
        // Bare rather than silently dropping the XOR semantics.
        let c = parse_with("WITH a WHERE a.x XOR a.y").unwrap();
        assert!(matches!(c.where_clause.unwrap(), WithExpr::Bare(_)));
    }

    #[test]
    fn unwind_basic() {
        let c = parse_unwind("UNWIND [1, 2, 3] AS x").unwrap();
        assert_eq!(c.var, "x");
        assert_eq!(
            c.source.0,
            ReturnExpr::ListLit(vec![
                ReturnExpr::Lit(Literal::Int(1)),
                ReturnExpr::Lit(Literal::Int(2)),
                ReturnExpr::Lit(Literal::Int(3)),
            ])
        );
        assert!(c.where_clause.is_none());
        assert!(c.with.is_none());
    }

    #[test]
    fn set_prop() {
        let items = parse_set("SET n.name = 'x'").unwrap();
        assert_eq!(items.len(), 1);
        let SetItem::Prop(prop, value) = &items[0] else {
            panic!("expected SetItem::Prop");
        };
        assert_eq!(prop.var, "n");
        assert_eq!(prop.prop, "name");
        assert_eq!(*value, ReturnExpr::Lit(Literal::String("x".to_string())));
    }

    #[test]
    fn set_labels() {
        let items = parse_set("SET n:A:B").unwrap();
        let SetItem::Labels(var, labels) = &items[0] else {
            panic!("expected SetItem::Labels");
        };
        assert_eq!(var, "n");
        assert_eq!(labels, &vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn set_map_assign() {
        let items = parse_set("SET n = {a: 1}").unwrap();
        let SetItem::MapAssign { var, merge, .. } = &items[0] else {
            panic!("expected SetItem::MapAssign");
        };
        assert_eq!(var, "n");
        assert!(!merge);

        let items = parse_set("SET n += {a: 1}").unwrap();
        let SetItem::MapAssign { merge, .. } = &items[0] else {
            panic!("expected SetItem::MapAssign");
        };
        assert!(merge);
    }

    #[test]
    fn set_multiple_items() {
        assert_eq!(parse_set("SET n.a = 1, n.b = 2").unwrap().len(), 2);
    }

    #[test]
    fn delete_items() {
        let d = parse_delete("DELETE n, r").unwrap();
        assert!(!d.detach);
        assert_eq!(d.items.len(), 2);
    }

    #[test]
    fn detach_delete() {
        let d = parse_delete("DETACH DELETE n").unwrap();
        assert!(d.detach);
    }

    #[test]
    fn remove_prop() {
        let items = parse_remove("REMOVE n.name").unwrap();
        let RemoveItem::Prop(prop) = &items[0] else {
            panic!("expected RemoveItem::Prop");
        };
        assert_eq!(prop.var, "n");
        assert_eq!(prop.prop, "name");
    }

    #[test]
    fn remove_labels() {
        let items = parse_remove("REMOVE n:A:B").unwrap();
        let RemoveItem::Labels(var, labels) = &items[0] else {
            panic!("expected RemoveItem::Labels");
        };
        assert_eq!(var, "n");
        assert_eq!(labels, &vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn create_single_pattern() {
        let patterns = parse_create("CREATE (a:Person)").unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].start.var.as_deref(), Some("a"));
    }

    #[test]
    fn create_comma_patterns_stay_separate() {
        // Unlike MATCH, CREATE never splices shared-node comma patterns
        // into one linear chain -- each stays its own Pattern.
        let patterns = parse_create("CREATE (a), (a)-->(b)").unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn create_named_path_errors() {
        assert!(parse_create("CREATE p = (a)-->(b)").is_err());
    }

    #[test]
    fn merge_single_hop() {
        let m = parse_merge("MERGE (a)-[:KNOWS]->(b)").unwrap();
        assert_eq!(m.pattern.hops.len(), 1);
        assert!(m.on_create.is_empty());
        assert!(m.on_match.is_empty());
    }

    #[test]
    fn merge_multi_hop_errors() {
        assert!(parse_merge("MERGE (a)-->(b)-->(c)").is_err());
    }

    #[test]
    fn merge_named_path_errors() {
        assert!(parse_merge("MERGE p = (a)-->(b)").is_err());
    }

    #[test]
    fn merge_on_create_on_match() {
        let m = parse_merge("MERGE (a) ON CREATE SET a.created = true ON MATCH SET a.seen = true")
            .unwrap();
        assert_eq!(m.on_create.len(), 1);
        assert_eq!(m.on_match.len(), 1);
    }

    #[test]
    fn merge_duplicate_on_create_errors() {
        assert!(parse_merge("MERGE (a) ON CREATE SET a.x = 1 ON CREATE SET a.y = 2").is_err());
    }

    #[test]
    fn merge_duplicate_on_match_errors() {
        assert!(parse_merge("MERGE (a) ON MATCH SET a.x = 1 ON MATCH SET a.y = 2").is_err());
    }

    #[test]
    fn statement_match_return() {
        let s = parse_statement("MATCH (a) RETURN a").unwrap();
        let Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        } = s
        else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 1);
        assert!(matches!(clauses[0], QueryClause::Match(_)));
        assert!(matches!(tail, Some(Tail::Return(_, false))));
        assert!(order_by.is_none());
        assert!(skip.is_none());
        assert!(limit.is_none());
    }

    #[test]
    fn statement_return_star() {
        let s = parse_statement("MATCH (a) RETURN *").unwrap();
        let Statement::Match { tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::ReturnStar(false))));
    }

    #[test]
    fn statement_order_by_skip_limit_on_bare_return() {
        let s = parse_statement("MATCH (a) RETURN a ORDER BY a SKIP 1 LIMIT 2").unwrap();
        let Statement::Match {
            order_by,
            skip,
            limit,
            ..
        } = s
        else {
            panic!("expected Statement::Match");
        };
        assert!(order_by.is_some());
        assert_eq!(skip, Some(1));
        assert_eq!(limit, Some(2));
    }

    #[test]
    fn statement_multiple_reading_clauses() {
        let s = parse_statement("MATCH (a) UNWIND [1,2] AS x RETURN a, x").unwrap();
        let Statement::Match { clauses, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 2);
        assert!(matches!(clauses[0], QueryClause::Match(_)));
        assert!(matches!(clauses[1], QueryClause::Unwind(_)));
    }

    #[test]
    fn statement_set_becomes_tail_with_return_tail() {
        let s = parse_statement("MATCH (n) SET n.x = 1 RETURN n").unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 1);
        let Some(Tail::Set(items, Some(ret))) = tail else {
            panic!("expected Tail::Set with a ReturnTail");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(ret.items.len(), 1);
    }

    #[test]
    fn statement_set_without_trailing_return() {
        let s = parse_statement("MATCH (n) SET n.x = 1").unwrap();
        let Statement::Match { tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::Set(_, None))));
    }

    #[test]
    fn statement_detach_delete_tail() {
        let s = parse_statement("MATCH (n) DETACH DELETE n").unwrap();
        let Statement::Match { tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::DetachDelete(_, None))));
    }

    #[test]
    fn statement_two_updating_clauses_last_becomes_tail() {
        // SET is just another QueryClause; DELETE (last) becomes the Tail.
        let s = parse_statement("MATCH (n) SET n.x = 1 DELETE n RETURN count(n)").unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 2);
        assert!(matches!(clauses[1], QueryClause::Set(_)));
        assert!(matches!(tail, Some(Tail::Delete(_, Some(_)))));
    }

    #[test]
    fn statement_bare_merge_no_tail() {
        // MERGE alone (no RETURN) is the one case a missing Tail is valid
        // -- MERGE never becomes the Tail itself (no Tail::Merge variant).
        let s = parse_statement("MERGE (a)").unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(clauses[0], QueryClause::Merge(_)));
        assert!(tail.is_none());
    }

    #[test]
    fn statement_merge_with_trailing_return() {
        // MERGE followed by RETURN: MERGE is a QueryClause, RETURN becomes
        // the statement's own full Tail::Return (order/skip/limit-capable),
        // not a narrower embedded ReturnTail the way SET/DELETE/REMOVE/
        // CREATE consume their own trailing RETURN.
        let s = parse_statement("MERGE (a) RETURN a ORDER BY a").unwrap();
        let Statement::Match {
            clauses,
            tail,
            order_by,
            ..
        } = s
        else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(clauses[0], QueryClause::Merge(_)));
        assert!(matches!(tail, Some(Tail::Return(_, false))));
        assert!(order_by.is_some());
    }

    #[test]
    fn statement_bare_match_without_tail_errors() {
        // Unlike MERGE, a bare MATCH with nothing after it is almost
        // certainly a mistake, not a deliberate no-op.
        assert!(parse_statement("MATCH (n)").is_err());
    }

    #[test]
    fn statement_mutating_tail_order_by_skip_limit_apply_at_statement_level() {
        // ReturnTail itself (SET/DELETE/REMOVE/CREATE's own trailing
        // RETURN) has no room for ORDER BY/SKIP/LIMIT -- but real Cypher
        // still allows them here (TCK's Delete6/Remove3 "Persistence of
        // .../remove clause side effects"), applying to the *statement*,
        // same as pest's own grammar keeps them as siblings of tail_clause
        // rather than nested inside the RETURN.
        let s =
            parse_statement("MATCH (n) SET n.x = 1 RETURN n ORDER BY n.x SKIP 1 LIMIT 2").unwrap();
        let Statement::Match {
            tail,
            order_by,
            skip,
            limit,
            ..
        } = s
        else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::Set(_, Some(_)))));
        assert!(order_by.is_some());
        assert_eq!(skip, Some(1));
        assert_eq!(limit, Some(2));
    }

    #[test]
    fn statement_mutating_tail_return_star_errors() {
        assert!(parse_statement("MATCH (n) SET n.x = 1 RETURN *").is_err());
    }

    #[test]
    fn statement_create_tail() {
        let s = parse_statement("CREATE (a) RETURN a").unwrap();
        let Statement::Match { tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::Create(_, Some(_)))));
    }

    #[test]
    fn statement_bare_create_is_not_wrapped_in_match() {
        // `CREATE (...)` with nothing else at all mirrors pest's
        // `create_stmt_only` -- a real `Statement::Create` directly, not
        // `Statement::Match{tail: Some(Tail::Create(...))}`. Found via a
        // Phase 3 dry-run: `explain.rs`'s "no query plan" output depends
        // on this exact shape distinction.
        let s = parse_antlr("CREATE (a);").unwrap();
        assert!(matches!(s, Statement::Create(_)));
    }

    #[test]
    fn statement_remove_tail() {
        let s = parse_statement("MATCH (n) REMOVE n.x").unwrap();
        let Statement::Match { tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert!(matches!(tail, Some(Tail::Remove(_, None))));
    }

    #[test]
    fn multi_part_with_attaches_to_preceding_match() {
        let s = parse_multi_part_statement("MATCH (a:A) WITH a MATCH (b:B) RETURN a, b").unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 2);
        let QueryClause::Match(first) = &clauses[0] else {
            panic!("expected first clause to be Match");
        };
        assert!(first.with.is_some());
        assert!(matches!(clauses[1], QueryClause::Match(_)));
        assert!(matches!(tail, Some(Tail::Return(_, false))));
    }

    #[test]
    fn multi_part_chained_with_second_one_standalone() {
        // TCK's chained `WITH x AS y WITH y % 3 AS y` shape: the first WITH
        // attaches to the preceding MATCH, the second has nothing
        // attachable immediately before it (another WITH, not a fresh
        // clause) so it becomes its own standalone `QueryClause::With`.
        let s = parse_multi_part_statement("MATCH (a:A) WITH a.num AS x WITH x % 3 AS x RETURN x")
            .unwrap();
        let Statement::Match { clauses, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 2);
        let QueryClause::Match(first) = &clauses[0] else {
            panic!("expected first clause to be Match");
        };
        assert!(first.with.is_some());
        assert!(matches!(clauses[1], QueryClause::With(_)));
    }

    #[test]
    fn multi_part_set_then_with_stays_separate_entries() {
        // SET has no `with` field on its `QueryClause` variant -- a
        // following WITH always becomes its own standalone entry, never
        // folded into the SET.
        let s = parse_multi_part_statement(
            "MATCH (n:N) WITH n, n.num AS num DELETE n WITH num WHERE num % 2 = 0 RETURN num",
        )
        .unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 3);
        assert!(matches!(clauses[0], QueryClause::Match(_)));
        assert!(matches!(clauses[1], QueryClause::Delete { .. }));
        assert!(matches!(clauses[2], QueryClause::With(_)));
        assert!(matches!(tail, Some(Tail::Return(_, false))));
    }

    #[test]
    fn multi_part_create_with_star_create_create_tail() {
        let s =
            parse_multi_part_statement("CREATE (a) WITH a WITH * CREATE (b) CREATE (a)<-[:T]-(b)")
                .unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        // Create(a), With(a) folded away into... no: Create has no `with`
        // field, so the first WITH is standalone; the second WITH (WITH *)
        // is likewise standalone (nothing attachable precedes it either).
        assert_eq!(clauses.len(), 4);
        assert!(matches!(clauses[0], QueryClause::Create(_)));
        assert!(matches!(clauses[1], QueryClause::With(_)));
        assert!(matches!(clauses[2], QueryClause::With(_)));
        assert!(matches!(clauses[3], QueryClause::Create(_)));
        assert!(matches!(tail, Some(Tail::Create(_, None))));
    }

    #[test]
    fn multi_part_merge_with_attaches() {
        let s = parse_multi_part_statement("MERGE (a:A) WITH a MATCH (b:B) RETURN a, b").unwrap();
        let Statement::Match { clauses, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 2);
        let QueryClause::Merge(m) = &clauses[0] else {
            panic!("expected first clause to be Merge");
        };
        assert!(m.with.is_some());
    }

    #[test]
    fn multi_part_trailing_bare_create_becomes_tail_not_top_level_statement() {
        // Regression: build_single_part_q's "bare CREATE with nothing
        // else" special case (-> Statement::Create directly) must NOT
        // leak out of multiPartQ's own trailing singlePartQ -- past at
        // least one WITH boundary, a trailing CREATE is still just this
        // statement's Tail::Create, same as any other trailing CREATE.
        // Previously panicked (found via a full TCK execution run).
        let s = parse_multi_part_statement("MATCH (a) WITH a CREATE (b)").unwrap();
        let Statement::Match { clauses, tail, .. } = s else {
            panic!("expected Statement::Match");
        };
        assert_eq!(clauses.len(), 1);
        assert!(matches!(clauses[0], QueryClause::Match(_)));
        assert!(matches!(tail, Some(Tail::Create(_, None))));
    }

    #[test]
    fn parse_antlr_no_union_passes_through() {
        let s = parse_antlr("MATCH (a) RETURN a;").unwrap();
        assert!(matches!(s, Statement::Match { .. }));
    }

    #[test]
    fn parse_antlr_union() {
        let s = parse_antlr("MATCH (a) RETURN a UNION MATCH (b) RETURN b;").unwrap();
        let Statement::Union { parts, all } = s else {
            panic!("expected Statement::Union");
        };
        assert_eq!(parts.len(), 2);
        assert!(!all);
    }

    #[test]
    fn parse_antlr_union_all() {
        let s = parse_antlr("MATCH (a) RETURN a UNION ALL MATCH (b) RETURN b;").unwrap();
        let Statement::Union { parts, all } = s else {
            panic!("expected Statement::Union");
        };
        assert_eq!(parts.len(), 2);
        assert!(all);
    }

    #[test]
    fn parse_antlr_union_three_parts() {
        let s =
            parse_antlr("MATCH (a) RETURN a UNION MATCH (b) RETURN b UNION MATCH (c) RETURN c;")
                .unwrap();
        let Statement::Union { parts, .. } = s else {
            panic!("expected Statement::Union");
        };
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn parse_antlr_mixed_union_and_union_all_errors() {
        let err = parse_antlr(
            "MATCH (a) RETURN a UNION MATCH (b) RETURN b UNION ALL MATCH (c) RETURN c;",
        )
        .unwrap_err();
        assert!(matches!(err, QueryError::Syntax(_)));
    }

    #[test]
    fn parse_antlr_standalone_call_not_supported() {
        assert!(parse_antlr("CALL db.labels() YIELD label;").is_err());
    }

    #[test]
    fn parse_antlr_syntax_error() {
        assert!(parse_antlr("MATCH (a RETURN a;").is_err());
    }

    #[test]
    fn parse_antlr_many_basic() {
        let stmts = parse_antlr_many("CREATE (a); CREATE (b); MATCH (n) RETURN n").unwrap();
        assert_eq!(stmts.len(), 3);
        // Bare `CREATE (...)` with nothing else is `Statement::Create`
        // directly, not `Statement::Match` -- see `build_single_part_q`'s
        // own docs.
        assert!(matches!(stmts[0], Statement::Create(_)));
        assert!(matches!(stmts[2], Statement::Match { .. }));
    }

    #[test]
    fn parse_antlr_many_single_statement() {
        let stmts = parse_antlr_many("RETURN 1").unwrap();
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn parse_antlr_many_strips_single_trailing_semicolon() {
        let stmts = parse_antlr_many("CREATE (a);").unwrap();
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn parse_antlr_many_semicolon_inside_string_literal_not_a_separator() {
        let stmts = parse_antlr_many("RETURN ';'").unwrap();
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn parse_antlr_create_index() {
        let s = parse_antlr("CREATE INDEX ON :Person(name);").unwrap();
        let Statement::CreateIndex {
            label,
            prop,
            unique,
        } = s
        else {
            panic!("expected Statement::CreateIndex");
        };
        assert_eq!(label, "Person");
        assert_eq!(prop, "name");
        assert!(!unique);
    }

    #[test]
    fn parse_antlr_create_index_unique() {
        let s = parse_antlr("CREATE INDEX ON :Person(name) UNIQUE;").unwrap();
        let Statement::CreateIndex { unique, .. } = s else {
            panic!("expected Statement::CreateIndex");
        };
        assert!(unique);
    }

    #[test]
    fn parse_antlr_explain_match() {
        let s = parse_antlr("EXPLAIN MATCH (a) RETURN a;").unwrap();
        let Statement::Explain(inner) = s else {
            panic!("expected Statement::Explain");
        };
        assert!(matches!(*inner, Statement::Match { .. }));
    }

    #[test]
    fn parse_antlr_explain_create_index() {
        let s = parse_antlr("EXPLAIN CREATE INDEX ON :Person(name);").unwrap();
        let Statement::Explain(inner) = s else {
            panic!("expected Statement::Explain");
        };
        assert!(matches!(*inner, Statement::CreateIndex { .. }));
    }

    #[test]
    fn parse_antlr_index_still_usable_as_property_name() {
        // `INDEX`/`EXPLAIN` becoming real keyword tokens (needed for
        // `createIndexSt`/`explainSt`) must not break their use as
        // ordinary property/label names elsewhere -- `name : symbol |
        // reservedWord` still absorbs them there.
        let s = parse_antlr("MATCH (a) RETURN a.index;").unwrap();
        assert!(matches!(s, Statement::Match { .. }));
    }
}
