#![allow(nonstandard_style)]
// Generated from CypherParser.g4 by ANTLR 4.13.2
use super::cypherparser::*;
use antlr4rust::tree::{ParseTreeVisitor, ParseTreeVisitorCompat};

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link CypherParser}.
 */
pub trait CypherParserVisitor<'input>: ParseTreeVisitor<'input, CypherParserContextType> {
    /**
     * Visit a parse tree produced by {@link CypherParser#script}.
     * @param ctx the parse tree
     */
    fn visit_script(&mut self, ctx: &ScriptContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#queries}.
     * @param ctx the parse tree
     */
    fn visit_queries(&mut self, ctx: &QueriesContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#query}.
     * @param ctx the parse tree
     */
    fn visit_query(&mut self, ctx: &QueryContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#explainSt}.
     * @param ctx the parse tree
     */
    fn visit_explainSt(&mut self, ctx: &ExplainStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#createIndexSt}.
     * @param ctx the parse tree
     */
    fn visit_createIndexSt(&mut self, ctx: &CreateIndexStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#regularQuery}.
     * @param ctx the parse tree
     */
    fn visit_regularQuery(&mut self, ctx: &RegularQueryContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#singleQuery}.
     * @param ctx the parse tree
     */
    fn visit_singleQuery(&mut self, ctx: &SingleQueryContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#standaloneCall}.
     * @param ctx the parse tree
     */
    fn visit_standaloneCall(&mut self, ctx: &StandaloneCallContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#returnSt}.
     * @param ctx the parse tree
     */
    fn visit_returnSt(&mut self, ctx: &ReturnStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#withSt}.
     * @param ctx the parse tree
     */
    fn visit_withSt(&mut self, ctx: &WithStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#skipSt}.
     * @param ctx the parse tree
     */
    fn visit_skipSt(&mut self, ctx: &SkipStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#limitSt}.
     * @param ctx the parse tree
     */
    fn visit_limitSt(&mut self, ctx: &LimitStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionBody}.
     * @param ctx the parse tree
     */
    fn visit_projectionBody(&mut self, ctx: &ProjectionBodyContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionItems}.
     * @param ctx the parse tree
     */
    fn visit_projectionItems(&mut self, ctx: &ProjectionItemsContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionItem}.
     * @param ctx the parse tree
     */
    fn visit_projectionItem(&mut self, ctx: &ProjectionItemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#orderItem}.
     * @param ctx the parse tree
     */
    fn visit_orderItem(&mut self, ctx: &OrderItemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#orderSt}.
     * @param ctx the parse tree
     */
    fn visit_orderSt(&mut self, ctx: &OrderStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#singlePartQ}.
     * @param ctx the parse tree
     */
    fn visit_singlePartQ(&mut self, ctx: &SinglePartQContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#multiPartQ}.
     * @param ctx the parse tree
     */
    fn visit_multiPartQ(&mut self, ctx: &MultiPartQContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#matchSt}.
     * @param ctx the parse tree
     */
    fn visit_matchSt(&mut self, ctx: &MatchStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unwindSt}.
     * @param ctx the parse tree
     */
    fn visit_unwindSt(&mut self, ctx: &UnwindStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#readingStatement}.
     * @param ctx the parse tree
     */
    fn visit_readingStatement(&mut self, ctx: &ReadingStatementContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#updatingStatement}.
     * @param ctx the parse tree
     */
    fn visit_updatingStatement(&mut self, ctx: &UpdatingStatementContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#deleteSt}.
     * @param ctx the parse tree
     */
    fn visit_deleteSt(&mut self, ctx: &DeleteStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#removeSt}.
     * @param ctx the parse tree
     */
    fn visit_removeSt(&mut self, ctx: &RemoveStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#removeItem}.
     * @param ctx the parse tree
     */
    fn visit_removeItem(&mut self, ctx: &RemoveItemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#queryCallSt}.
     * @param ctx the parse tree
     */
    fn visit_queryCallSt(&mut self, ctx: &QueryCallStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parenExpressionChain}.
     * @param ctx the parse tree
     */
    fn visit_parenExpressionChain(&mut self, ctx: &ParenExpressionChainContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#yieldItems}.
     * @param ctx the parse tree
     */
    fn visit_yieldItems(&mut self, ctx: &YieldItemsContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#yieldItem}.
     * @param ctx the parse tree
     */
    fn visit_yieldItem(&mut self, ctx: &YieldItemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mergeSt}.
     * @param ctx the parse tree
     */
    fn visit_mergeSt(&mut self, ctx: &MergeStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mergeAction}.
     * @param ctx the parse tree
     */
    fn visit_mergeAction(&mut self, ctx: &MergeActionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#setSt}.
     * @param ctx the parse tree
     */
    fn visit_setSt(&mut self, ctx: &SetStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#setItem}.
     * @param ctx the parse tree
     */
    fn visit_setItem(&mut self, ctx: &SetItemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nodeLabels}.
     * @param ctx the parse tree
     */
    fn visit_nodeLabels(&mut self, ctx: &NodeLabelsContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#createSt}.
     * @param ctx the parse tree
     */
    fn visit_createSt(&mut self, ctx: &CreateStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternWhere}.
     * @param ctx the parse tree
     */
    fn visit_patternWhere(&mut self, ctx: &PatternWhereContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#where}.
     * @param ctx the parse tree
     */
    fn visit_where(&mut self, ctx: &WhereContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#pattern}.
     * @param ctx the parse tree
     */
    fn visit_pattern(&mut self, ctx: &PatternContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#expression}.
     * @param ctx the parse tree
     */
    fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#xorExpression}.
     * @param ctx the parse tree
     */
    fn visit_xorExpression(&mut self, ctx: &XorExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#andExpression}.
     * @param ctx the parse tree
     */
    fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#notExpression}.
     * @param ctx the parse tree
     */
    fn visit_notExpression(&mut self, ctx: &NotExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#comparisonExpression}.
     * @param ctx the parse tree
     */
    fn visit_comparisonExpression(&mut self, ctx: &ComparisonExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringListNullExpression}.
     * @param ctx the parse tree
     */
    fn visit_stringListNullExpression(&mut self, ctx: &StringListNullExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#inExpression}.
     * @param ctx the parse tree
     */
    fn visit_inExpression(&mut self, ctx: &InExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#comparisonSigns}.
     * @param ctx the parse tree
     */
    fn visit_comparisonSigns(&mut self, ctx: &ComparisonSignsContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#addSubExpression}.
     * @param ctx the parse tree
     */
    fn visit_addSubExpression(&mut self, ctx: &AddSubExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#multDivExpression}.
     * @param ctx the parse tree
     */
    fn visit_multDivExpression(&mut self, ctx: &MultDivExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#powerExpression}.
     * @param ctx the parse tree
     */
    fn visit_powerExpression(&mut self, ctx: &PowerExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unaryAddSubExpression}.
     * @param ctx the parse tree
     */
    fn visit_unaryAddSubExpression(&mut self, ctx: &UnaryAddSubExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#atomicExpression}.
     * @param ctx the parse tree
     */
    fn visit_atomicExpression(&mut self, ctx: &AtomicExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listExpression}.
     * @param ctx the parse tree
     */
    fn visit_listExpression(&mut self, ctx: &ListExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringExpression}.
     * @param ctx the parse tree
     */
    fn visit_stringExpression(&mut self, ctx: &StringExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringExpPrefix}.
     * @param ctx the parse tree
     */
    fn visit_stringExpPrefix(&mut self, ctx: &StringExpPrefixContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nullExpression}.
     * @param ctx the parse tree
     */
    fn visit_nullExpression(&mut self, ctx: &NullExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#propertyOrLabelExpression}.
     * @param ctx the parse tree
     */
    fn visit_propertyOrLabelExpression(&mut self, ctx: &PropertyOrLabelExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#propertyExpression}.
     * @param ctx the parse tree
     */
    fn visit_propertyExpression(&mut self, ctx: &PropertyExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternPart}.
     * @param ctx the parse tree
     */
    fn visit_patternPart(&mut self, ctx: &PatternPartContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#shortestPathWrapper}.
     * @param ctx the parse tree
     */
    fn visit_shortestPathWrapper(&mut self, ctx: &ShortestPathWrapperContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternElem}.
     * @param ctx the parse tree
     */
    fn visit_patternElem(&mut self, ctx: &PatternElemContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternElemChain}.
     * @param ctx the parse tree
     */
    fn visit_patternElemChain(&mut self, ctx: &PatternElemChainContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppElemChain}.
     * @param ctx the parse tree
     */
    fn visit_qppElemChain(&mut self, ctx: &QppElemChainContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppQuantifier}.
     * @param ctx the parse tree
     */
    fn visit_qppQuantifier(&mut self, ctx: &QppQuantifierContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppInt}.
     * @param ctx the parse tree
     */
    fn visit_qppInt(&mut self, ctx: &QppIntContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#properties}.
     * @param ctx the parse tree
     */
    fn visit_properties(&mut self, ctx: &PropertiesContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nodePattern}.
     * @param ctx the parse tree
     */
    fn visit_nodePattern(&mut self, ctx: &NodePatternContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#atom}.
     * @param ctx the parse tree
     */
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#lhs}.
     * @param ctx the parse tree
     */
    fn visit_lhs(&mut self, ctx: &LhsContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipPattern}.
     * @param ctx the parse tree
     */
    fn visit_relationshipPattern(&mut self, ctx: &RelationshipPatternContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationDetail}.
     * @param ctx the parse tree
     */
    fn visit_relationDetail(&mut self, ctx: &RelationDetailContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipTypes}.
     * @param ctx the parse tree
     */
    fn visit_relationshipTypes(&mut self, ctx: &RelationshipTypesContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unionSt}.
     * @param ctx the parse tree
     */
    fn visit_unionSt(&mut self, ctx: &UnionStContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#subqueryExist}.
     * @param ctx the parse tree
     */
    fn visit_subqueryExist(&mut self, ctx: &SubqueryExistContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#invocationName}.
     * @param ctx the parse tree
     */
    fn visit_invocationName(&mut self, ctx: &InvocationNameContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#functionInvocation}.
     * @param ctx the parse tree
     */
    fn visit_functionInvocation(&mut self, ctx: &FunctionInvocationContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parenthesizedExpression}.
     * @param ctx the parse tree
     */
    fn visit_parenthesizedExpression(&mut self, ctx: &ParenthesizedExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#filterWith}.
     * @param ctx the parse tree
     */
    fn visit_filterWith(&mut self, ctx: &FilterWithContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternComprehension}.
     * @param ctx the parse tree
     */
    fn visit_patternComprehension(&mut self, ctx: &PatternComprehensionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipsChainPattern}.
     * @param ctx the parse tree
     */
    fn visit_relationshipsChainPattern(&mut self, ctx: &RelationshipsChainPatternContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listComprehension}.
     * @param ctx the parse tree
     */
    fn visit_listComprehension(&mut self, ctx: &ListComprehensionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#filterExpression}.
     * @param ctx the parse tree
     */
    fn visit_filterExpression(&mut self, ctx: &FilterExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#countAll}.
     * @param ctx the parse tree
     */
    fn visit_countAll(&mut self, ctx: &CountAllContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#expressionChain}.
     * @param ctx the parse tree
     */
    fn visit_expressionChain(&mut self, ctx: &ExpressionChainContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#caseExpression}.
     * @param ctx the parse tree
     */
    fn visit_caseExpression(&mut self, ctx: &CaseExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parameter}.
     * @param ctx the parse tree
     */
    fn visit_parameter(&mut self, ctx: &ParameterContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#literal}.
     * @param ctx the parse tree
     */
    fn visit_literal(&mut self, ctx: &LiteralContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#rangeLit}.
     * @param ctx the parse tree
     */
    fn visit_rangeLit(&mut self, ctx: &RangeLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#boolLit}.
     * @param ctx the parse tree
     */
    fn visit_boolLit(&mut self, ctx: &BoolLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#numLit}.
     * @param ctx the parse tree
     */
    fn visit_numLit(&mut self, ctx: &NumLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringLit}.
     * @param ctx the parse tree
     */
    fn visit_stringLit(&mut self, ctx: &StringLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#charLit}.
     * @param ctx the parse tree
     */
    fn visit_charLit(&mut self, ctx: &CharLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listLit}.
     * @param ctx the parse tree
     */
    fn visit_listLit(&mut self, ctx: &ListLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mapLit}.
     * @param ctx the parse tree
     */
    fn visit_mapLit(&mut self, ctx: &MapLitContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mapPair}.
     * @param ctx the parse tree
     */
    fn visit_mapPair(&mut self, ctx: &MapPairContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#name}.
     * @param ctx the parse tree
     */
    fn visit_name(&mut self, ctx: &NameContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#symbol}.
     * @param ctx the parse tree
     */
    fn visit_symbol(&mut self, ctx: &SymbolContext<'input>) {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#reservedWord}.
     * @param ctx the parse tree
     */
    fn visit_reservedWord(&mut self, ctx: &ReservedWordContext<'input>) {
        self.visit_children(ctx)
    }
}

pub trait CypherParserVisitorCompat<'input>:
    ParseTreeVisitorCompat<'input, Node = CypherParserContextType>
{
    /**
     * Visit a parse tree produced by {@link CypherParser#script}.
     * @param ctx the parse tree
     */
    fn visit_script(&mut self, ctx: &ScriptContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#queries}.
     * @param ctx the parse tree
     */
    fn visit_queries(&mut self, ctx: &QueriesContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#query}.
     * @param ctx the parse tree
     */
    fn visit_query(&mut self, ctx: &QueryContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#explainSt}.
     * @param ctx the parse tree
     */
    fn visit_explainSt(&mut self, ctx: &ExplainStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#createIndexSt}.
     * @param ctx the parse tree
     */
    fn visit_createIndexSt(&mut self, ctx: &CreateIndexStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#regularQuery}.
     * @param ctx the parse tree
     */
    fn visit_regularQuery(&mut self, ctx: &RegularQueryContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#singleQuery}.
     * @param ctx the parse tree
     */
    fn visit_singleQuery(&mut self, ctx: &SingleQueryContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#standaloneCall}.
     * @param ctx the parse tree
     */
    fn visit_standaloneCall(&mut self, ctx: &StandaloneCallContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#returnSt}.
     * @param ctx the parse tree
     */
    fn visit_returnSt(&mut self, ctx: &ReturnStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#withSt}.
     * @param ctx the parse tree
     */
    fn visit_withSt(&mut self, ctx: &WithStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#skipSt}.
     * @param ctx the parse tree
     */
    fn visit_skipSt(&mut self, ctx: &SkipStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#limitSt}.
     * @param ctx the parse tree
     */
    fn visit_limitSt(&mut self, ctx: &LimitStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionBody}.
     * @param ctx the parse tree
     */
    fn visit_projectionBody(&mut self, ctx: &ProjectionBodyContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionItems}.
     * @param ctx the parse tree
     */
    fn visit_projectionItems(&mut self, ctx: &ProjectionItemsContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#projectionItem}.
     * @param ctx the parse tree
     */
    fn visit_projectionItem(&mut self, ctx: &ProjectionItemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#orderItem}.
     * @param ctx the parse tree
     */
    fn visit_orderItem(&mut self, ctx: &OrderItemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#orderSt}.
     * @param ctx the parse tree
     */
    fn visit_orderSt(&mut self, ctx: &OrderStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#singlePartQ}.
     * @param ctx the parse tree
     */
    fn visit_singlePartQ(&mut self, ctx: &SinglePartQContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#multiPartQ}.
     * @param ctx the parse tree
     */
    fn visit_multiPartQ(&mut self, ctx: &MultiPartQContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#matchSt}.
     * @param ctx the parse tree
     */
    fn visit_matchSt(&mut self, ctx: &MatchStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unwindSt}.
     * @param ctx the parse tree
     */
    fn visit_unwindSt(&mut self, ctx: &UnwindStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#readingStatement}.
     * @param ctx the parse tree
     */
    fn visit_readingStatement(&mut self, ctx: &ReadingStatementContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#updatingStatement}.
     * @param ctx the parse tree
     */
    fn visit_updatingStatement(&mut self, ctx: &UpdatingStatementContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#deleteSt}.
     * @param ctx the parse tree
     */
    fn visit_deleteSt(&mut self, ctx: &DeleteStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#removeSt}.
     * @param ctx the parse tree
     */
    fn visit_removeSt(&mut self, ctx: &RemoveStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#removeItem}.
     * @param ctx the parse tree
     */
    fn visit_removeItem(&mut self, ctx: &RemoveItemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#queryCallSt}.
     * @param ctx the parse tree
     */
    fn visit_queryCallSt(&mut self, ctx: &QueryCallStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parenExpressionChain}.
     * @param ctx the parse tree
     */
    fn visit_parenExpressionChain(
        &mut self,
        ctx: &ParenExpressionChainContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#yieldItems}.
     * @param ctx the parse tree
     */
    fn visit_yieldItems(&mut self, ctx: &YieldItemsContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#yieldItem}.
     * @param ctx the parse tree
     */
    fn visit_yieldItem(&mut self, ctx: &YieldItemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mergeSt}.
     * @param ctx the parse tree
     */
    fn visit_mergeSt(&mut self, ctx: &MergeStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mergeAction}.
     * @param ctx the parse tree
     */
    fn visit_mergeAction(&mut self, ctx: &MergeActionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#setSt}.
     * @param ctx the parse tree
     */
    fn visit_setSt(&mut self, ctx: &SetStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#setItem}.
     * @param ctx the parse tree
     */
    fn visit_setItem(&mut self, ctx: &SetItemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nodeLabels}.
     * @param ctx the parse tree
     */
    fn visit_nodeLabels(&mut self, ctx: &NodeLabelsContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#createSt}.
     * @param ctx the parse tree
     */
    fn visit_createSt(&mut self, ctx: &CreateStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternWhere}.
     * @param ctx the parse tree
     */
    fn visit_patternWhere(&mut self, ctx: &PatternWhereContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#where}.
     * @param ctx the parse tree
     */
    fn visit_where(&mut self, ctx: &WhereContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#pattern}.
     * @param ctx the parse tree
     */
    fn visit_pattern(&mut self, ctx: &PatternContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#expression}.
     * @param ctx the parse tree
     */
    fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#xorExpression}.
     * @param ctx the parse tree
     */
    fn visit_xorExpression(&mut self, ctx: &XorExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#andExpression}.
     * @param ctx the parse tree
     */
    fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#notExpression}.
     * @param ctx the parse tree
     */
    fn visit_notExpression(&mut self, ctx: &NotExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#comparisonExpression}.
     * @param ctx the parse tree
     */
    fn visit_comparisonExpression(
        &mut self,
        ctx: &ComparisonExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringListNullExpression}.
     * @param ctx the parse tree
     */
    fn visit_stringListNullExpression(
        &mut self,
        ctx: &StringListNullExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#inExpression}.
     * @param ctx the parse tree
     */
    fn visit_inExpression(&mut self, ctx: &InExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#comparisonSigns}.
     * @param ctx the parse tree
     */
    fn visit_comparisonSigns(&mut self, ctx: &ComparisonSignsContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#addSubExpression}.
     * @param ctx the parse tree
     */
    fn visit_addSubExpression(&mut self, ctx: &AddSubExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#multDivExpression}.
     * @param ctx the parse tree
     */
    fn visit_multDivExpression(&mut self, ctx: &MultDivExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#powerExpression}.
     * @param ctx the parse tree
     */
    fn visit_powerExpression(&mut self, ctx: &PowerExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unaryAddSubExpression}.
     * @param ctx the parse tree
     */
    fn visit_unaryAddSubExpression(
        &mut self,
        ctx: &UnaryAddSubExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#atomicExpression}.
     * @param ctx the parse tree
     */
    fn visit_atomicExpression(&mut self, ctx: &AtomicExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listExpression}.
     * @param ctx the parse tree
     */
    fn visit_listExpression(&mut self, ctx: &ListExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringExpression}.
     * @param ctx the parse tree
     */
    fn visit_stringExpression(&mut self, ctx: &StringExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringExpPrefix}.
     * @param ctx the parse tree
     */
    fn visit_stringExpPrefix(&mut self, ctx: &StringExpPrefixContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nullExpression}.
     * @param ctx the parse tree
     */
    fn visit_nullExpression(&mut self, ctx: &NullExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#propertyOrLabelExpression}.
     * @param ctx the parse tree
     */
    fn visit_propertyOrLabelExpression(
        &mut self,
        ctx: &PropertyOrLabelExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#propertyExpression}.
     * @param ctx the parse tree
     */
    fn visit_propertyExpression(
        &mut self,
        ctx: &PropertyExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternPart}.
     * @param ctx the parse tree
     */
    fn visit_patternPart(&mut self, ctx: &PatternPartContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#shortestPathWrapper}.
     * @param ctx the parse tree
     */
    fn visit_shortestPathWrapper(
        &mut self,
        ctx: &ShortestPathWrapperContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternElem}.
     * @param ctx the parse tree
     */
    fn visit_patternElem(&mut self, ctx: &PatternElemContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternElemChain}.
     * @param ctx the parse tree
     */
    fn visit_patternElemChain(&mut self, ctx: &PatternElemChainContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppElemChain}.
     * @param ctx the parse tree
     */
    fn visit_qppElemChain(&mut self, ctx: &QppElemChainContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppQuantifier}.
     * @param ctx the parse tree
     */
    fn visit_qppQuantifier(&mut self, ctx: &QppQuantifierContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#qppInt}.
     * @param ctx the parse tree
     */
    fn visit_qppInt(&mut self, ctx: &QppIntContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#properties}.
     * @param ctx the parse tree
     */
    fn visit_properties(&mut self, ctx: &PropertiesContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#nodePattern}.
     * @param ctx the parse tree
     */
    fn visit_nodePattern(&mut self, ctx: &NodePatternContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#atom}.
     * @param ctx the parse tree
     */
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#lhs}.
     * @param ctx the parse tree
     */
    fn visit_lhs(&mut self, ctx: &LhsContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipPattern}.
     * @param ctx the parse tree
     */
    fn visit_relationshipPattern(
        &mut self,
        ctx: &RelationshipPatternContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationDetail}.
     * @param ctx the parse tree
     */
    fn visit_relationDetail(&mut self, ctx: &RelationDetailContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipTypes}.
     * @param ctx the parse tree
     */
    fn visit_relationshipTypes(&mut self, ctx: &RelationshipTypesContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#unionSt}.
     * @param ctx the parse tree
     */
    fn visit_unionSt(&mut self, ctx: &UnionStContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#subqueryExist}.
     * @param ctx the parse tree
     */
    fn visit_subqueryExist(&mut self, ctx: &SubqueryExistContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#invocationName}.
     * @param ctx the parse tree
     */
    fn visit_invocationName(&mut self, ctx: &InvocationNameContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#functionInvocation}.
     * @param ctx the parse tree
     */
    fn visit_functionInvocation(
        &mut self,
        ctx: &FunctionInvocationContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parenthesizedExpression}.
     * @param ctx the parse tree
     */
    fn visit_parenthesizedExpression(
        &mut self,
        ctx: &ParenthesizedExpressionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#filterWith}.
     * @param ctx the parse tree
     */
    fn visit_filterWith(&mut self, ctx: &FilterWithContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#patternComprehension}.
     * @param ctx the parse tree
     */
    fn visit_patternComprehension(
        &mut self,
        ctx: &PatternComprehensionContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#relationshipsChainPattern}.
     * @param ctx the parse tree
     */
    fn visit_relationshipsChainPattern(
        &mut self,
        ctx: &RelationshipsChainPatternContext<'input>,
    ) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listComprehension}.
     * @param ctx the parse tree
     */
    fn visit_listComprehension(&mut self, ctx: &ListComprehensionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#filterExpression}.
     * @param ctx the parse tree
     */
    fn visit_filterExpression(&mut self, ctx: &FilterExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#countAll}.
     * @param ctx the parse tree
     */
    fn visit_countAll(&mut self, ctx: &CountAllContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#expressionChain}.
     * @param ctx the parse tree
     */
    fn visit_expressionChain(&mut self, ctx: &ExpressionChainContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#caseExpression}.
     * @param ctx the parse tree
     */
    fn visit_caseExpression(&mut self, ctx: &CaseExpressionContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#parameter}.
     * @param ctx the parse tree
     */
    fn visit_parameter(&mut self, ctx: &ParameterContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#literal}.
     * @param ctx the parse tree
     */
    fn visit_literal(&mut self, ctx: &LiteralContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#rangeLit}.
     * @param ctx the parse tree
     */
    fn visit_rangeLit(&mut self, ctx: &RangeLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#boolLit}.
     * @param ctx the parse tree
     */
    fn visit_boolLit(&mut self, ctx: &BoolLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#numLit}.
     * @param ctx the parse tree
     */
    fn visit_numLit(&mut self, ctx: &NumLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#stringLit}.
     * @param ctx the parse tree
     */
    fn visit_stringLit(&mut self, ctx: &StringLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#charLit}.
     * @param ctx the parse tree
     */
    fn visit_charLit(&mut self, ctx: &CharLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#listLit}.
     * @param ctx the parse tree
     */
    fn visit_listLit(&mut self, ctx: &ListLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mapLit}.
     * @param ctx the parse tree
     */
    fn visit_mapLit(&mut self, ctx: &MapLitContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#mapPair}.
     * @param ctx the parse tree
     */
    fn visit_mapPair(&mut self, ctx: &MapPairContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#name}.
     * @param ctx the parse tree
     */
    fn visit_name(&mut self, ctx: &NameContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#symbol}.
     * @param ctx the parse tree
     */
    fn visit_symbol(&mut self, ctx: &SymbolContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }

    /**
     * Visit a parse tree produced by {@link CypherParser#reservedWord}.
     * @param ctx the parse tree
     */
    fn visit_reservedWord(&mut self, ctx: &ReservedWordContext<'input>) -> Self::Return {
        self.visit_children(ctx)
    }
}

impl<'input, T> CypherParserVisitor<'input> for T
where
    T: CypherParserVisitorCompat<'input>,
{
    fn visit_script(&mut self, ctx: &ScriptContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_script(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_queries(&mut self, ctx: &QueriesContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_queries(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_query(&mut self, ctx: &QueryContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_query(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_explainSt(&mut self, ctx: &ExplainStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_explainSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_createIndexSt(&mut self, ctx: &CreateIndexStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_createIndexSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_regularQuery(&mut self, ctx: &RegularQueryContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_regularQuery(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_singleQuery(&mut self, ctx: &SingleQueryContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_singleQuery(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_standaloneCall(&mut self, ctx: &StandaloneCallContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_standaloneCall(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_returnSt(&mut self, ctx: &ReturnStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_returnSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_withSt(&mut self, ctx: &WithStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_withSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_skipSt(&mut self, ctx: &SkipStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_skipSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_limitSt(&mut self, ctx: &LimitStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_limitSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_projectionBody(&mut self, ctx: &ProjectionBodyContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_projectionBody(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_projectionItems(&mut self, ctx: &ProjectionItemsContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_projectionItems(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_projectionItem(&mut self, ctx: &ProjectionItemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_projectionItem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_orderItem(&mut self, ctx: &OrderItemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_orderItem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_orderSt(&mut self, ctx: &OrderStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_orderSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_singlePartQ(&mut self, ctx: &SinglePartQContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_singlePartQ(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_multiPartQ(&mut self, ctx: &MultiPartQContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_multiPartQ(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_matchSt(&mut self, ctx: &MatchStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_matchSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_unwindSt(&mut self, ctx: &UnwindStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_unwindSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_readingStatement(&mut self, ctx: &ReadingStatementContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_readingStatement(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_updatingStatement(&mut self, ctx: &UpdatingStatementContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_updatingStatement(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_deleteSt(&mut self, ctx: &DeleteStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_deleteSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_removeSt(&mut self, ctx: &RemoveStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_removeSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_removeItem(&mut self, ctx: &RemoveItemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_removeItem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_queryCallSt(&mut self, ctx: &QueryCallStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_queryCallSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_parenExpressionChain(&mut self, ctx: &ParenExpressionChainContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_parenExpressionChain(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_yieldItems(&mut self, ctx: &YieldItemsContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_yieldItems(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_yieldItem(&mut self, ctx: &YieldItemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_yieldItem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_mergeSt(&mut self, ctx: &MergeStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_mergeSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_mergeAction(&mut self, ctx: &MergeActionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_mergeAction(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_setSt(&mut self, ctx: &SetStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_setSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_setItem(&mut self, ctx: &SetItemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_setItem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_nodeLabels(&mut self, ctx: &NodeLabelsContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_nodeLabels(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_createSt(&mut self, ctx: &CreateStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_createSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_patternWhere(&mut self, ctx: &PatternWhereContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_patternWhere(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_where(&mut self, ctx: &WhereContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_where(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_pattern(&mut self, ctx: &PatternContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_pattern(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_expression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_xorExpression(&mut self, ctx: &XorExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_xorExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_andExpression(&mut self, ctx: &AndExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_andExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_notExpression(&mut self, ctx: &NotExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_notExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_comparisonExpression(&mut self, ctx: &ComparisonExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_comparisonExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_stringListNullExpression(&mut self, ctx: &StringListNullExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_stringListNullExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_inExpression(&mut self, ctx: &InExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_inExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_comparisonSigns(&mut self, ctx: &ComparisonSignsContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_comparisonSigns(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_addSubExpression(&mut self, ctx: &AddSubExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_addSubExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_multDivExpression(&mut self, ctx: &MultDivExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_multDivExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_powerExpression(&mut self, ctx: &PowerExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_powerExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_unaryAddSubExpression(&mut self, ctx: &UnaryAddSubExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_unaryAddSubExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_atomicExpression(&mut self, ctx: &AtomicExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_atomicExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_listExpression(&mut self, ctx: &ListExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_listExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_stringExpression(&mut self, ctx: &StringExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_stringExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_stringExpPrefix(&mut self, ctx: &StringExpPrefixContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_stringExpPrefix(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_nullExpression(&mut self, ctx: &NullExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_nullExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_propertyOrLabelExpression(&mut self, ctx: &PropertyOrLabelExpressionContext<'input>) {
        let result =
            <Self as CypherParserVisitorCompat>::visit_propertyOrLabelExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_propertyExpression(&mut self, ctx: &PropertyExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_propertyExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_patternPart(&mut self, ctx: &PatternPartContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_patternPart(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_shortestPathWrapper(&mut self, ctx: &ShortestPathWrapperContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_shortestPathWrapper(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_patternElem(&mut self, ctx: &PatternElemContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_patternElem(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_patternElemChain(&mut self, ctx: &PatternElemChainContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_patternElemChain(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_qppElemChain(&mut self, ctx: &QppElemChainContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_qppElemChain(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_qppQuantifier(&mut self, ctx: &QppQuantifierContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_qppQuantifier(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_qppInt(&mut self, ctx: &QppIntContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_qppInt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_properties(&mut self, ctx: &PropertiesContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_properties(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_nodePattern(&mut self, ctx: &NodePatternContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_nodePattern(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_atom(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_lhs(&mut self, ctx: &LhsContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_lhs(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relationshipPattern(&mut self, ctx: &RelationshipPatternContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_relationshipPattern(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relationDetail(&mut self, ctx: &RelationDetailContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_relationDetail(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relationshipTypes(&mut self, ctx: &RelationshipTypesContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_relationshipTypes(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_unionSt(&mut self, ctx: &UnionStContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_unionSt(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_subqueryExist(&mut self, ctx: &SubqueryExistContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_subqueryExist(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_invocationName(&mut self, ctx: &InvocationNameContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_invocationName(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_functionInvocation(&mut self, ctx: &FunctionInvocationContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_functionInvocation(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_parenthesizedExpression(&mut self, ctx: &ParenthesizedExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_parenthesizedExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_filterWith(&mut self, ctx: &FilterWithContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_filterWith(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_patternComprehension(&mut self, ctx: &PatternComprehensionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_patternComprehension(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_relationshipsChainPattern(&mut self, ctx: &RelationshipsChainPatternContext<'input>) {
        let result =
            <Self as CypherParserVisitorCompat>::visit_relationshipsChainPattern(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_listComprehension(&mut self, ctx: &ListComprehensionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_listComprehension(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_filterExpression(&mut self, ctx: &FilterExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_filterExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_countAll(&mut self, ctx: &CountAllContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_countAll(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_expressionChain(&mut self, ctx: &ExpressionChainContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_expressionChain(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_caseExpression(&mut self, ctx: &CaseExpressionContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_caseExpression(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_parameter(&mut self, ctx: &ParameterContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_parameter(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_literal(&mut self, ctx: &LiteralContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_literal(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_rangeLit(&mut self, ctx: &RangeLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_rangeLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_boolLit(&mut self, ctx: &BoolLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_boolLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_numLit(&mut self, ctx: &NumLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_numLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_stringLit(&mut self, ctx: &StringLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_stringLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_charLit(&mut self, ctx: &CharLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_charLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_listLit(&mut self, ctx: &ListLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_listLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_mapLit(&mut self, ctx: &MapLitContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_mapLit(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_mapPair(&mut self, ctx: &MapPairContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_mapPair(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_name(&mut self, ctx: &NameContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_name(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_symbol(&mut self, ctx: &SymbolContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_symbol(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }

    fn visit_reservedWord(&mut self, ctx: &ReservedWordContext<'input>) {
        let result = <Self as CypherParserVisitorCompat>::visit_reservedWord(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
    }
}
