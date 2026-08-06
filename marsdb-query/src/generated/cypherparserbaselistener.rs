// Generated from CypherParser.g4 by ANTLR 4.13.2

use super::cypherparser::*;
use antlr4rust::tree::ParseTreeListener;

// A complete Visitor for a parse tree produced by CypherParser.

pub trait CypherParserBaseListener<'input>:
    ParseTreeListener<'input, CypherParserContextType>
{
    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_script(&mut self, _ctx: &ScriptContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_script(&mut self, _ctx: &ScriptContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_queries(&mut self, _ctx: &QueriesContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_queries(&mut self, _ctx: &QueriesContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_query(&mut self, _ctx: &QueryContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_query(&mut self, _ctx: &QueryContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_explainst(&mut self, _ctx: &ExplainStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_explainst(&mut self, _ctx: &ExplainStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_createindexst(&mut self, _ctx: &CreateIndexStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_createindexst(&mut self, _ctx: &CreateIndexStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_regularquery(&mut self, _ctx: &RegularQueryContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_regularquery(&mut self, _ctx: &RegularQueryContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_singlequery(&mut self, _ctx: &SingleQueryContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_singlequery(&mut self, _ctx: &SingleQueryContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_standalonecall(&mut self, _ctx: &StandaloneCallContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_standalonecall(&mut self, _ctx: &StandaloneCallContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_returnst(&mut self, _ctx: &ReturnStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_returnst(&mut self, _ctx: &ReturnStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_withst(&mut self, _ctx: &WithStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_withst(&mut self, _ctx: &WithStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_skipst(&mut self, _ctx: &SkipStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_skipst(&mut self, _ctx: &SkipStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_limitst(&mut self, _ctx: &LimitStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_limitst(&mut self, _ctx: &LimitStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_projectionbody(&mut self, _ctx: &ProjectionBodyContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_projectionbody(&mut self, _ctx: &ProjectionBodyContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_projectionitems(&mut self, _ctx: &ProjectionItemsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_projectionitems(&mut self, _ctx: &ProjectionItemsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_projectionitem(&mut self, _ctx: &ProjectionItemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_projectionitem(&mut self, _ctx: &ProjectionItemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_orderitem(&mut self, _ctx: &OrderItemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_orderitem(&mut self, _ctx: &OrderItemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_orderst(&mut self, _ctx: &OrderStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_orderst(&mut self, _ctx: &OrderStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_singlepartq(&mut self, _ctx: &SinglePartQContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_singlepartq(&mut self, _ctx: &SinglePartQContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_multipartq(&mut self, _ctx: &MultiPartQContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_multipartq(&mut self, _ctx: &MultiPartQContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_matchst(&mut self, _ctx: &MatchStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_matchst(&mut self, _ctx: &MatchStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unwindst(&mut self, _ctx: &UnwindStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unwindst(&mut self, _ctx: &UnwindStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_readingstatement(&mut self, _ctx: &ReadingStatementContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_readingstatement(&mut self, _ctx: &ReadingStatementContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_updatingstatement(&mut self, _ctx: &UpdatingStatementContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_updatingstatement(&mut self, _ctx: &UpdatingStatementContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_deletest(&mut self, _ctx: &DeleteStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_deletest(&mut self, _ctx: &DeleteStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_removest(&mut self, _ctx: &RemoveStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_removest(&mut self, _ctx: &RemoveStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_removeitem(&mut self, _ctx: &RemoveItemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_removeitem(&mut self, _ctx: &RemoveItemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_querycallst(&mut self, _ctx: &QueryCallStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_querycallst(&mut self, _ctx: &QueryCallStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_parenexpressionchain(&mut self, _ctx: &ParenExpressionChainContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_parenexpressionchain(&mut self, _ctx: &ParenExpressionChainContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_yielditems(&mut self, _ctx: &YieldItemsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_yielditems(&mut self, _ctx: &YieldItemsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_yielditem(&mut self, _ctx: &YieldItemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_yielditem(&mut self, _ctx: &YieldItemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_mergest(&mut self, _ctx: &MergeStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_mergest(&mut self, _ctx: &MergeStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_mergeaction(&mut self, _ctx: &MergeActionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_mergeaction(&mut self, _ctx: &MergeActionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_setst(&mut self, _ctx: &SetStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_setst(&mut self, _ctx: &SetStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_setitem(&mut self, _ctx: &SetItemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_setitem(&mut self, _ctx: &SetItemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_nodelabels(&mut self, _ctx: &NodeLabelsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_nodelabels(&mut self, _ctx: &NodeLabelsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_createst(&mut self, _ctx: &CreateStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_createst(&mut self, _ctx: &CreateStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternwhere(&mut self, _ctx: &PatternWhereContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternwhere(&mut self, _ctx: &PatternWhereContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_where(&mut self, _ctx: &WhereContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_where(&mut self, _ctx: &WhereContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_pattern(&mut self, _ctx: &PatternContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_pattern(&mut self, _ctx: &PatternContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_expression(&mut self, _ctx: &ExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_expression(&mut self, _ctx: &ExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_xorexpression(&mut self, _ctx: &XorExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_xorexpression(&mut self, _ctx: &XorExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_andexpression(&mut self, _ctx: &AndExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_andexpression(&mut self, _ctx: &AndExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_notexpression(&mut self, _ctx: &NotExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_notexpression(&mut self, _ctx: &NotExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_comparisonexpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_comparisonexpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_stringlistnullexpression(&mut self, _ctx: &StringListNullExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_stringlistnullexpression(&mut self, _ctx: &StringListNullExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_inexpression(&mut self, _ctx: &InExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_inexpression(&mut self, _ctx: &InExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_comparisonsigns(&mut self, _ctx: &ComparisonSignsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_comparisonsigns(&mut self, _ctx: &ComparisonSignsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_addsubexpression(&mut self, _ctx: &AddSubExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_addsubexpression(&mut self, _ctx: &AddSubExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_multdivexpression(&mut self, _ctx: &MultDivExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_multdivexpression(&mut self, _ctx: &MultDivExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_powerexpression(&mut self, _ctx: &PowerExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_powerexpression(&mut self, _ctx: &PowerExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unaryaddsubexpression(&mut self, _ctx: &UnaryAddSubExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unaryaddsubexpression(&mut self, _ctx: &UnaryAddSubExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_atomicexpression(&mut self, _ctx: &AtomicExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_atomicexpression(&mut self, _ctx: &AtomicExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_listexpression(&mut self, _ctx: &ListExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_listexpression(&mut self, _ctx: &ListExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_stringexpression(&mut self, _ctx: &StringExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_stringexpression(&mut self, _ctx: &StringExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_stringexpprefix(&mut self, _ctx: &StringExpPrefixContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_stringexpprefix(&mut self, _ctx: &StringExpPrefixContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_nullexpression(&mut self, _ctx: &NullExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_nullexpression(&mut self, _ctx: &NullExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_propertyorlabelexpression(&mut self, _ctx: &PropertyOrLabelExpressionContext<'input>) {
    }
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_propertyorlabelexpression(&mut self, _ctx: &PropertyOrLabelExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_propertyexpression(&mut self, _ctx: &PropertyExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_propertyexpression(&mut self, _ctx: &PropertyExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternpart(&mut self, _ctx: &PatternPartContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternpart(&mut self, _ctx: &PatternPartContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_shortestpathwrapper(&mut self, _ctx: &ShortestPathWrapperContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_shortestpathwrapper(&mut self, _ctx: &ShortestPathWrapperContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternelem(&mut self, _ctx: &PatternElemContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternelem(&mut self, _ctx: &PatternElemContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patternelemchain(&mut self, _ctx: &PatternElemChainContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patternelemchain(&mut self, _ctx: &PatternElemChainContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_qppelemchain(&mut self, _ctx: &QppElemChainContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_qppelemchain(&mut self, _ctx: &QppElemChainContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_qppquantifier(&mut self, _ctx: &QppQuantifierContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_qppquantifier(&mut self, _ctx: &QppQuantifierContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_qppint(&mut self, _ctx: &QppIntContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_qppint(&mut self, _ctx: &QppIntContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_properties(&mut self, _ctx: &PropertiesContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_properties(&mut self, _ctx: &PropertiesContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_nodepattern(&mut self, _ctx: &NodePatternContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_nodepattern(&mut self, _ctx: &NodePatternContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_atom(&mut self, _ctx: &AtomContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_atom(&mut self, _ctx: &AtomContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_lhs(&mut self, _ctx: &LhsContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_lhs(&mut self, _ctx: &LhsContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relationshippattern(&mut self, _ctx: &RelationshipPatternContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relationshippattern(&mut self, _ctx: &RelationshipPatternContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relationdetail(&mut self, _ctx: &RelationDetailContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relationdetail(&mut self, _ctx: &RelationDetailContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relationshiptypes(&mut self, _ctx: &RelationshipTypesContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relationshiptypes(&mut self, _ctx: &RelationshipTypesContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_unionst(&mut self, _ctx: &UnionStContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_unionst(&mut self, _ctx: &UnionStContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_subqueryexist(&mut self, _ctx: &SubqueryExistContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_subqueryexist(&mut self, _ctx: &SubqueryExistContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_invocationname(&mut self, _ctx: &InvocationNameContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_invocationname(&mut self, _ctx: &InvocationNameContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_functioninvocation(&mut self, _ctx: &FunctionInvocationContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_functioninvocation(&mut self, _ctx: &FunctionInvocationContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_parenthesizedexpression(&mut self, _ctx: &ParenthesizedExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_parenthesizedexpression(&mut self, _ctx: &ParenthesizedExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_filterwith(&mut self, _ctx: &FilterWithContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_filterwith(&mut self, _ctx: &FilterWithContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_patterncomprehension(&mut self, _ctx: &PatternComprehensionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_patterncomprehension(&mut self, _ctx: &PatternComprehensionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_relationshipschainpattern(&mut self, _ctx: &RelationshipsChainPatternContext<'input>) {
    }
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_relationshipschainpattern(&mut self, _ctx: &RelationshipsChainPatternContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_listcomprehension(&mut self, _ctx: &ListComprehensionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_listcomprehension(&mut self, _ctx: &ListComprehensionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_filterexpression(&mut self, _ctx: &FilterExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_filterexpression(&mut self, _ctx: &FilterExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_countall(&mut self, _ctx: &CountAllContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_countall(&mut self, _ctx: &CountAllContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_expressionchain(&mut self, _ctx: &ExpressionChainContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_expressionchain(&mut self, _ctx: &ExpressionChainContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_caseexpression(&mut self, _ctx: &CaseExpressionContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_caseexpression(&mut self, _ctx: &CaseExpressionContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_parameter(&mut self, _ctx: &ParameterContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_parameter(&mut self, _ctx: &ParameterContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_literal(&mut self, _ctx: &LiteralContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_literal(&mut self, _ctx: &LiteralContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_rangelit(&mut self, _ctx: &RangeLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_rangelit(&mut self, _ctx: &RangeLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_boollit(&mut self, _ctx: &BoolLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_boollit(&mut self, _ctx: &BoolLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_numlit(&mut self, _ctx: &NumLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_numlit(&mut self, _ctx: &NumLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_stringlit(&mut self, _ctx: &StringLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_stringlit(&mut self, _ctx: &StringLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_charlit(&mut self, _ctx: &CharLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_charlit(&mut self, _ctx: &CharLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_listlit(&mut self, _ctx: &ListLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_listlit(&mut self, _ctx: &ListLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_maplit(&mut self, _ctx: &MapLitContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_maplit(&mut self, _ctx: &MapLitContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_mappair(&mut self, _ctx: &MapPairContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_mappair(&mut self, _ctx: &MapPairContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_name(&mut self, _ctx: &NameContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_name(&mut self, _ctx: &NameContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_symbol(&mut self, _ctx: &SymbolContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_symbol(&mut self, _ctx: &SymbolContext<'input>) {}

    /**
     * Enter a parse tree produced by \{@link CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_reservedword(&mut self, _ctx: &ReservedWordContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  CypherParserBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_reservedword(&mut self, _ctx: &ReservedWordContext<'input>) {}
}
