#![allow(nonstandard_style)]
// Generated from CypherParser.g4 by ANTLR 4.13.2
use super::cypherparser::*;
use antlr4rust::tree::ParseTreeListener;

pub trait CypherParserListener<'input>: ParseTreeListener<'input, CypherParserContextType> {
    /**
     * Enter a parse tree produced by {@link CypherParser#script}.
     * @param ctx the parse tree
     */
    fn enter_script(&mut self, _ctx: &ScriptContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#script}.
     * @param ctx the parse tree
     */
    fn exit_script(&mut self, _ctx: &ScriptContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#query}.
     * @param ctx the parse tree
     */
    fn enter_query(&mut self, _ctx: &QueryContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#query}.
     * @param ctx the parse tree
     */
    fn exit_query(&mut self, _ctx: &QueryContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#explainSt}.
     * @param ctx the parse tree
     */
    fn enter_explainSt(&mut self, _ctx: &ExplainStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#explainSt}.
     * @param ctx the parse tree
     */
    fn exit_explainSt(&mut self, _ctx: &ExplainStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#createIndexSt}.
     * @param ctx the parse tree
     */
    fn enter_createIndexSt(&mut self, _ctx: &CreateIndexStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#createIndexSt}.
     * @param ctx the parse tree
     */
    fn exit_createIndexSt(&mut self, _ctx: &CreateIndexStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#regularQuery}.
     * @param ctx the parse tree
     */
    fn enter_regularQuery(&mut self, _ctx: &RegularQueryContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#regularQuery}.
     * @param ctx the parse tree
     */
    fn exit_regularQuery(&mut self, _ctx: &RegularQueryContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#singleQuery}.
     * @param ctx the parse tree
     */
    fn enter_singleQuery(&mut self, _ctx: &SingleQueryContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#singleQuery}.
     * @param ctx the parse tree
     */
    fn exit_singleQuery(&mut self, _ctx: &SingleQueryContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#standaloneCall}.
     * @param ctx the parse tree
     */
    fn enter_standaloneCall(&mut self, _ctx: &StandaloneCallContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#standaloneCall}.
     * @param ctx the parse tree
     */
    fn exit_standaloneCall(&mut self, _ctx: &StandaloneCallContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#returnSt}.
     * @param ctx the parse tree
     */
    fn enter_returnSt(&mut self, _ctx: &ReturnStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#returnSt}.
     * @param ctx the parse tree
     */
    fn exit_returnSt(&mut self, _ctx: &ReturnStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#withSt}.
     * @param ctx the parse tree
     */
    fn enter_withSt(&mut self, _ctx: &WithStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#withSt}.
     * @param ctx the parse tree
     */
    fn exit_withSt(&mut self, _ctx: &WithStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#skipSt}.
     * @param ctx the parse tree
     */
    fn enter_skipSt(&mut self, _ctx: &SkipStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#skipSt}.
     * @param ctx the parse tree
     */
    fn exit_skipSt(&mut self, _ctx: &SkipStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#limitSt}.
     * @param ctx the parse tree
     */
    fn enter_limitSt(&mut self, _ctx: &LimitStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#limitSt}.
     * @param ctx the parse tree
     */
    fn exit_limitSt(&mut self, _ctx: &LimitStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#projectionBody}.
     * @param ctx the parse tree
     */
    fn enter_projectionBody(&mut self, _ctx: &ProjectionBodyContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#projectionBody}.
     * @param ctx the parse tree
     */
    fn exit_projectionBody(&mut self, _ctx: &ProjectionBodyContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#projectionItems}.
     * @param ctx the parse tree
     */
    fn enter_projectionItems(&mut self, _ctx: &ProjectionItemsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#projectionItems}.
     * @param ctx the parse tree
     */
    fn exit_projectionItems(&mut self, _ctx: &ProjectionItemsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#projectionItem}.
     * @param ctx the parse tree
     */
    fn enter_projectionItem(&mut self, _ctx: &ProjectionItemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#projectionItem}.
     * @param ctx the parse tree
     */
    fn exit_projectionItem(&mut self, _ctx: &ProjectionItemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#orderItem}.
     * @param ctx the parse tree
     */
    fn enter_orderItem(&mut self, _ctx: &OrderItemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#orderItem}.
     * @param ctx the parse tree
     */
    fn exit_orderItem(&mut self, _ctx: &OrderItemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#orderSt}.
     * @param ctx the parse tree
     */
    fn enter_orderSt(&mut self, _ctx: &OrderStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#orderSt}.
     * @param ctx the parse tree
     */
    fn exit_orderSt(&mut self, _ctx: &OrderStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#singlePartQ}.
     * @param ctx the parse tree
     */
    fn enter_singlePartQ(&mut self, _ctx: &SinglePartQContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#singlePartQ}.
     * @param ctx the parse tree
     */
    fn exit_singlePartQ(&mut self, _ctx: &SinglePartQContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#multiPartQ}.
     * @param ctx the parse tree
     */
    fn enter_multiPartQ(&mut self, _ctx: &MultiPartQContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#multiPartQ}.
     * @param ctx the parse tree
     */
    fn exit_multiPartQ(&mut self, _ctx: &MultiPartQContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#matchSt}.
     * @param ctx the parse tree
     */
    fn enter_matchSt(&mut self, _ctx: &MatchStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#matchSt}.
     * @param ctx the parse tree
     */
    fn exit_matchSt(&mut self, _ctx: &MatchStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#unwindSt}.
     * @param ctx the parse tree
     */
    fn enter_unwindSt(&mut self, _ctx: &UnwindStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#unwindSt}.
     * @param ctx the parse tree
     */
    fn exit_unwindSt(&mut self, _ctx: &UnwindStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#readingStatement}.
     * @param ctx the parse tree
     */
    fn enter_readingStatement(&mut self, _ctx: &ReadingStatementContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#readingStatement}.
     * @param ctx the parse tree
     */
    fn exit_readingStatement(&mut self, _ctx: &ReadingStatementContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#updatingStatement}.
     * @param ctx the parse tree
     */
    fn enter_updatingStatement(&mut self, _ctx: &UpdatingStatementContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#updatingStatement}.
     * @param ctx the parse tree
     */
    fn exit_updatingStatement(&mut self, _ctx: &UpdatingStatementContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#deleteSt}.
     * @param ctx the parse tree
     */
    fn enter_deleteSt(&mut self, _ctx: &DeleteStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#deleteSt}.
     * @param ctx the parse tree
     */
    fn exit_deleteSt(&mut self, _ctx: &DeleteStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#removeSt}.
     * @param ctx the parse tree
     */
    fn enter_removeSt(&mut self, _ctx: &RemoveStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#removeSt}.
     * @param ctx the parse tree
     */
    fn exit_removeSt(&mut self, _ctx: &RemoveStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#removeItem}.
     * @param ctx the parse tree
     */
    fn enter_removeItem(&mut self, _ctx: &RemoveItemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#removeItem}.
     * @param ctx the parse tree
     */
    fn exit_removeItem(&mut self, _ctx: &RemoveItemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#queryCallSt}.
     * @param ctx the parse tree
     */
    fn enter_queryCallSt(&mut self, _ctx: &QueryCallStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#queryCallSt}.
     * @param ctx the parse tree
     */
    fn exit_queryCallSt(&mut self, _ctx: &QueryCallStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#parenExpressionChain}.
     * @param ctx the parse tree
     */
    fn enter_parenExpressionChain(&mut self, _ctx: &ParenExpressionChainContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#parenExpressionChain}.
     * @param ctx the parse tree
     */
    fn exit_parenExpressionChain(&mut self, _ctx: &ParenExpressionChainContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#yieldItems}.
     * @param ctx the parse tree
     */
    fn enter_yieldItems(&mut self, _ctx: &YieldItemsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#yieldItems}.
     * @param ctx the parse tree
     */
    fn exit_yieldItems(&mut self, _ctx: &YieldItemsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#yieldItem}.
     * @param ctx the parse tree
     */
    fn enter_yieldItem(&mut self, _ctx: &YieldItemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#yieldItem}.
     * @param ctx the parse tree
     */
    fn exit_yieldItem(&mut self, _ctx: &YieldItemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#mergeSt}.
     * @param ctx the parse tree
     */
    fn enter_mergeSt(&mut self, _ctx: &MergeStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#mergeSt}.
     * @param ctx the parse tree
     */
    fn exit_mergeSt(&mut self, _ctx: &MergeStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#mergeAction}.
     * @param ctx the parse tree
     */
    fn enter_mergeAction(&mut self, _ctx: &MergeActionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#mergeAction}.
     * @param ctx the parse tree
     */
    fn exit_mergeAction(&mut self, _ctx: &MergeActionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#setSt}.
     * @param ctx the parse tree
     */
    fn enter_setSt(&mut self, _ctx: &SetStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#setSt}.
     * @param ctx the parse tree
     */
    fn exit_setSt(&mut self, _ctx: &SetStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#setItem}.
     * @param ctx the parse tree
     */
    fn enter_setItem(&mut self, _ctx: &SetItemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#setItem}.
     * @param ctx the parse tree
     */
    fn exit_setItem(&mut self, _ctx: &SetItemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#nodeLabels}.
     * @param ctx the parse tree
     */
    fn enter_nodeLabels(&mut self, _ctx: &NodeLabelsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#nodeLabels}.
     * @param ctx the parse tree
     */
    fn exit_nodeLabels(&mut self, _ctx: &NodeLabelsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#createSt}.
     * @param ctx the parse tree
     */
    fn enter_createSt(&mut self, _ctx: &CreateStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#createSt}.
     * @param ctx the parse tree
     */
    fn exit_createSt(&mut self, _ctx: &CreateStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#patternWhere}.
     * @param ctx the parse tree
     */
    fn enter_patternWhere(&mut self, _ctx: &PatternWhereContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#patternWhere}.
     * @param ctx the parse tree
     */
    fn exit_patternWhere(&mut self, _ctx: &PatternWhereContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#where}.
     * @param ctx the parse tree
     */
    fn enter_where(&mut self, _ctx: &WhereContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#where}.
     * @param ctx the parse tree
     */
    fn exit_where(&mut self, _ctx: &WhereContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#pattern}.
     * @param ctx the parse tree
     */
    fn enter_pattern(&mut self, _ctx: &PatternContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#pattern}.
     * @param ctx the parse tree
     */
    fn exit_pattern(&mut self, _ctx: &PatternContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#expression}.
     * @param ctx the parse tree
     */
    fn enter_expression(&mut self, _ctx: &ExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#expression}.
     * @param ctx the parse tree
     */
    fn exit_expression(&mut self, _ctx: &ExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#xorExpression}.
     * @param ctx the parse tree
     */
    fn enter_xorExpression(&mut self, _ctx: &XorExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#xorExpression}.
     * @param ctx the parse tree
     */
    fn exit_xorExpression(&mut self, _ctx: &XorExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#andExpression}.
     * @param ctx the parse tree
     */
    fn enter_andExpression(&mut self, _ctx: &AndExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#andExpression}.
     * @param ctx the parse tree
     */
    fn exit_andExpression(&mut self, _ctx: &AndExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#notExpression}.
     * @param ctx the parse tree
     */
    fn enter_notExpression(&mut self, _ctx: &NotExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#notExpression}.
     * @param ctx the parse tree
     */
    fn exit_notExpression(&mut self, _ctx: &NotExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#comparisonExpression}.
     * @param ctx the parse tree
     */
    fn enter_comparisonExpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#comparisonExpression}.
     * @param ctx the parse tree
     */
    fn exit_comparisonExpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#comparisonSigns}.
     * @param ctx the parse tree
     */
    fn enter_comparisonSigns(&mut self, _ctx: &ComparisonSignsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#comparisonSigns}.
     * @param ctx the parse tree
     */
    fn exit_comparisonSigns(&mut self, _ctx: &ComparisonSignsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#addSubExpression}.
     * @param ctx the parse tree
     */
    fn enter_addSubExpression(&mut self, _ctx: &AddSubExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#addSubExpression}.
     * @param ctx the parse tree
     */
    fn exit_addSubExpression(&mut self, _ctx: &AddSubExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#multDivExpression}.
     * @param ctx the parse tree
     */
    fn enter_multDivExpression(&mut self, _ctx: &MultDivExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#multDivExpression}.
     * @param ctx the parse tree
     */
    fn exit_multDivExpression(&mut self, _ctx: &MultDivExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#powerExpression}.
     * @param ctx the parse tree
     */
    fn enter_powerExpression(&mut self, _ctx: &PowerExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#powerExpression}.
     * @param ctx the parse tree
     */
    fn exit_powerExpression(&mut self, _ctx: &PowerExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#unaryAddSubExpression}.
     * @param ctx the parse tree
     */
    fn enter_unaryAddSubExpression(&mut self, _ctx: &UnaryAddSubExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#unaryAddSubExpression}.
     * @param ctx the parse tree
     */
    fn exit_unaryAddSubExpression(&mut self, _ctx: &UnaryAddSubExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#atomicExpression}.
     * @param ctx the parse tree
     */
    fn enter_atomicExpression(&mut self, _ctx: &AtomicExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#atomicExpression}.
     * @param ctx the parse tree
     */
    fn exit_atomicExpression(&mut self, _ctx: &AtomicExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#listExpression}.
     * @param ctx the parse tree
     */
    fn enter_listExpression(&mut self, _ctx: &ListExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#listExpression}.
     * @param ctx the parse tree
     */
    fn exit_listExpression(&mut self, _ctx: &ListExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#stringExpression}.
     * @param ctx the parse tree
     */
    fn enter_stringExpression(&mut self, _ctx: &StringExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#stringExpression}.
     * @param ctx the parse tree
     */
    fn exit_stringExpression(&mut self, _ctx: &StringExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#stringExpPrefix}.
     * @param ctx the parse tree
     */
    fn enter_stringExpPrefix(&mut self, _ctx: &StringExpPrefixContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#stringExpPrefix}.
     * @param ctx the parse tree
     */
    fn exit_stringExpPrefix(&mut self, _ctx: &StringExpPrefixContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#nullExpression}.
     * @param ctx the parse tree
     */
    fn enter_nullExpression(&mut self, _ctx: &NullExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#nullExpression}.
     * @param ctx the parse tree
     */
    fn exit_nullExpression(&mut self, _ctx: &NullExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#propertyOrLabelExpression}.
     * @param ctx the parse tree
     */
    fn enter_propertyOrLabelExpression(&mut self, _ctx: &PropertyOrLabelExpressionContext<'input>) {
    }
    /**
     * Exit a parse tree produced by {@link CypherParser#propertyOrLabelExpression}.
     * @param ctx the parse tree
     */
    fn exit_propertyOrLabelExpression(&mut self, _ctx: &PropertyOrLabelExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#propertyExpression}.
     * @param ctx the parse tree
     */
    fn enter_propertyExpression(&mut self, _ctx: &PropertyExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#propertyExpression}.
     * @param ctx the parse tree
     */
    fn exit_propertyExpression(&mut self, _ctx: &PropertyExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#patternPart}.
     * @param ctx the parse tree
     */
    fn enter_patternPart(&mut self, _ctx: &PatternPartContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#patternPart}.
     * @param ctx the parse tree
     */
    fn exit_patternPart(&mut self, _ctx: &PatternPartContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#patternElem}.
     * @param ctx the parse tree
     */
    fn enter_patternElem(&mut self, _ctx: &PatternElemContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#patternElem}.
     * @param ctx the parse tree
     */
    fn exit_patternElem(&mut self, _ctx: &PatternElemContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#patternElemChain}.
     * @param ctx the parse tree
     */
    fn enter_patternElemChain(&mut self, _ctx: &PatternElemChainContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#patternElemChain}.
     * @param ctx the parse tree
     */
    fn exit_patternElemChain(&mut self, _ctx: &PatternElemChainContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#qppElemChain}.
     * @param ctx the parse tree
     */
    fn enter_qppElemChain(&mut self, _ctx: &QppElemChainContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#qppElemChain}.
     * @param ctx the parse tree
     */
    fn exit_qppElemChain(&mut self, _ctx: &QppElemChainContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#qppQuantifier}.
     * @param ctx the parse tree
     */
    fn enter_qppQuantifier(&mut self, _ctx: &QppQuantifierContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#qppQuantifier}.
     * @param ctx the parse tree
     */
    fn exit_qppQuantifier(&mut self, _ctx: &QppQuantifierContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#qppInt}.
     * @param ctx the parse tree
     */
    fn enter_qppInt(&mut self, _ctx: &QppIntContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#qppInt}.
     * @param ctx the parse tree
     */
    fn exit_qppInt(&mut self, _ctx: &QppIntContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#properties}.
     * @param ctx the parse tree
     */
    fn enter_properties(&mut self, _ctx: &PropertiesContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#properties}.
     * @param ctx the parse tree
     */
    fn exit_properties(&mut self, _ctx: &PropertiesContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#nodePattern}.
     * @param ctx the parse tree
     */
    fn enter_nodePattern(&mut self, _ctx: &NodePatternContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#nodePattern}.
     * @param ctx the parse tree
     */
    fn exit_nodePattern(&mut self, _ctx: &NodePatternContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#atom}.
     * @param ctx the parse tree
     */
    fn enter_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#atom}.
     * @param ctx the parse tree
     */
    fn exit_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#lhs}.
     * @param ctx the parse tree
     */
    fn enter_lhs(&mut self, _ctx: &LhsContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#lhs}.
     * @param ctx the parse tree
     */
    fn exit_lhs(&mut self, _ctx: &LhsContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#relationshipPattern}.
     * @param ctx the parse tree
     */
    fn enter_relationshipPattern(&mut self, _ctx: &RelationshipPatternContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#relationshipPattern}.
     * @param ctx the parse tree
     */
    fn exit_relationshipPattern(&mut self, _ctx: &RelationshipPatternContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#relationDetail}.
     * @param ctx the parse tree
     */
    fn enter_relationDetail(&mut self, _ctx: &RelationDetailContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#relationDetail}.
     * @param ctx the parse tree
     */
    fn exit_relationDetail(&mut self, _ctx: &RelationDetailContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#relationshipTypes}.
     * @param ctx the parse tree
     */
    fn enter_relationshipTypes(&mut self, _ctx: &RelationshipTypesContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#relationshipTypes}.
     * @param ctx the parse tree
     */
    fn exit_relationshipTypes(&mut self, _ctx: &RelationshipTypesContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#unionSt}.
     * @param ctx the parse tree
     */
    fn enter_unionSt(&mut self, _ctx: &UnionStContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#unionSt}.
     * @param ctx the parse tree
     */
    fn exit_unionSt(&mut self, _ctx: &UnionStContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#subqueryExist}.
     * @param ctx the parse tree
     */
    fn enter_subqueryExist(&mut self, _ctx: &SubqueryExistContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#subqueryExist}.
     * @param ctx the parse tree
     */
    fn exit_subqueryExist(&mut self, _ctx: &SubqueryExistContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#invocationName}.
     * @param ctx the parse tree
     */
    fn enter_invocationName(&mut self, _ctx: &InvocationNameContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#invocationName}.
     * @param ctx the parse tree
     */
    fn exit_invocationName(&mut self, _ctx: &InvocationNameContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#functionInvocation}.
     * @param ctx the parse tree
     */
    fn enter_functionInvocation(&mut self, _ctx: &FunctionInvocationContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#functionInvocation}.
     * @param ctx the parse tree
     */
    fn exit_functionInvocation(&mut self, _ctx: &FunctionInvocationContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#parenthesizedExpression}.
     * @param ctx the parse tree
     */
    fn enter_parenthesizedExpression(&mut self, _ctx: &ParenthesizedExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#parenthesizedExpression}.
     * @param ctx the parse tree
     */
    fn exit_parenthesizedExpression(&mut self, _ctx: &ParenthesizedExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#filterWith}.
     * @param ctx the parse tree
     */
    fn enter_filterWith(&mut self, _ctx: &FilterWithContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#filterWith}.
     * @param ctx the parse tree
     */
    fn exit_filterWith(&mut self, _ctx: &FilterWithContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#patternComprehension}.
     * @param ctx the parse tree
     */
    fn enter_patternComprehension(&mut self, _ctx: &PatternComprehensionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#patternComprehension}.
     * @param ctx the parse tree
     */
    fn exit_patternComprehension(&mut self, _ctx: &PatternComprehensionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#relationshipsChainPattern}.
     * @param ctx the parse tree
     */
    fn enter_relationshipsChainPattern(&mut self, _ctx: &RelationshipsChainPatternContext<'input>) {
    }
    /**
     * Exit a parse tree produced by {@link CypherParser#relationshipsChainPattern}.
     * @param ctx the parse tree
     */
    fn exit_relationshipsChainPattern(&mut self, _ctx: &RelationshipsChainPatternContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#listComprehension}.
     * @param ctx the parse tree
     */
    fn enter_listComprehension(&mut self, _ctx: &ListComprehensionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#listComprehension}.
     * @param ctx the parse tree
     */
    fn exit_listComprehension(&mut self, _ctx: &ListComprehensionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#filterExpression}.
     * @param ctx the parse tree
     */
    fn enter_filterExpression(&mut self, _ctx: &FilterExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#filterExpression}.
     * @param ctx the parse tree
     */
    fn exit_filterExpression(&mut self, _ctx: &FilterExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#countAll}.
     * @param ctx the parse tree
     */
    fn enter_countAll(&mut self, _ctx: &CountAllContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#countAll}.
     * @param ctx the parse tree
     */
    fn exit_countAll(&mut self, _ctx: &CountAllContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#expressionChain}.
     * @param ctx the parse tree
     */
    fn enter_expressionChain(&mut self, _ctx: &ExpressionChainContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#expressionChain}.
     * @param ctx the parse tree
     */
    fn exit_expressionChain(&mut self, _ctx: &ExpressionChainContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#caseExpression}.
     * @param ctx the parse tree
     */
    fn enter_caseExpression(&mut self, _ctx: &CaseExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#caseExpression}.
     * @param ctx the parse tree
     */
    fn exit_caseExpression(&mut self, _ctx: &CaseExpressionContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#parameter}.
     * @param ctx the parse tree
     */
    fn enter_parameter(&mut self, _ctx: &ParameterContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#parameter}.
     * @param ctx the parse tree
     */
    fn exit_parameter(&mut self, _ctx: &ParameterContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#literal}.
     * @param ctx the parse tree
     */
    fn enter_literal(&mut self, _ctx: &LiteralContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#literal}.
     * @param ctx the parse tree
     */
    fn exit_literal(&mut self, _ctx: &LiteralContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#rangeLit}.
     * @param ctx the parse tree
     */
    fn enter_rangeLit(&mut self, _ctx: &RangeLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#rangeLit}.
     * @param ctx the parse tree
     */
    fn exit_rangeLit(&mut self, _ctx: &RangeLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#boolLit}.
     * @param ctx the parse tree
     */
    fn enter_boolLit(&mut self, _ctx: &BoolLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#boolLit}.
     * @param ctx the parse tree
     */
    fn exit_boolLit(&mut self, _ctx: &BoolLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#numLit}.
     * @param ctx the parse tree
     */
    fn enter_numLit(&mut self, _ctx: &NumLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#numLit}.
     * @param ctx the parse tree
     */
    fn exit_numLit(&mut self, _ctx: &NumLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#stringLit}.
     * @param ctx the parse tree
     */
    fn enter_stringLit(&mut self, _ctx: &StringLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#stringLit}.
     * @param ctx the parse tree
     */
    fn exit_stringLit(&mut self, _ctx: &StringLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#charLit}.
     * @param ctx the parse tree
     */
    fn enter_charLit(&mut self, _ctx: &CharLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#charLit}.
     * @param ctx the parse tree
     */
    fn exit_charLit(&mut self, _ctx: &CharLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#listLit}.
     * @param ctx the parse tree
     */
    fn enter_listLit(&mut self, _ctx: &ListLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#listLit}.
     * @param ctx the parse tree
     */
    fn exit_listLit(&mut self, _ctx: &ListLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#mapLit}.
     * @param ctx the parse tree
     */
    fn enter_mapLit(&mut self, _ctx: &MapLitContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#mapLit}.
     * @param ctx the parse tree
     */
    fn exit_mapLit(&mut self, _ctx: &MapLitContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#mapPair}.
     * @param ctx the parse tree
     */
    fn enter_mapPair(&mut self, _ctx: &MapPairContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#mapPair}.
     * @param ctx the parse tree
     */
    fn exit_mapPair(&mut self, _ctx: &MapPairContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#name}.
     * @param ctx the parse tree
     */
    fn enter_name(&mut self, _ctx: &NameContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#name}.
     * @param ctx the parse tree
     */
    fn exit_name(&mut self, _ctx: &NameContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#symbol}.
     * @param ctx the parse tree
     */
    fn enter_symbol(&mut self, _ctx: &SymbolContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#symbol}.
     * @param ctx the parse tree
     */
    fn exit_symbol(&mut self, _ctx: &SymbolContext<'input>) {}
    /**
     * Enter a parse tree produced by {@link CypherParser#reservedWord}.
     * @param ctx the parse tree
     */
    fn enter_reservedWord(&mut self, _ctx: &ReservedWordContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CypherParser#reservedWord}.
     * @param ctx the parse tree
     */
    fn exit_reservedWord(&mut self, _ctx: &ReservedWordContext<'input>) {}
}

antlr4rust::coerce_from! { 'input : CypherParserListener<'input> }
