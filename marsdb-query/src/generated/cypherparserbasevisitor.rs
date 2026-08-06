// Generated from CypherParser.g4 by ANTLR 4.13.2

use super::cypherparser::*;
use antlr4rust::tree::ParseTreeVisitor;

// A complete Visitor for a parse tree produced by CypherParser.

pub trait CypherParserBaseVisitor<'input>:
    ParseTreeVisitor<'input, CypherParserContextType>
{
    // Visit a parse tree produced by CypherParser#script.
    fn visit_script(&mut self, ctx: &ScriptContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#query.
    fn visit_query(&mut self, ctx: &QueryContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#explainSt.
    fn visit_explainst(&mut self, ctx: &ExplainStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#createIndexSt.
    fn visit_createindexst(&mut self, ctx: &CreateIndexStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#regularQuery.
    fn visit_regularquery(&mut self, ctx: &RegularQueryContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#singleQuery.
    fn visit_singlequery(&mut self, ctx: &SingleQueryContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#standaloneCall.
    fn visit_standalonecall(&mut self, ctx: &StandaloneCallContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#returnSt.
    fn visit_returnst(&mut self, ctx: &ReturnStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#withSt.
    fn visit_withst(&mut self, ctx: &WithStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#skipSt.
    fn visit_skipst(&mut self, ctx: &SkipStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#limitSt.
    fn visit_limitst(&mut self, ctx: &LimitStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#projectionBody.
    fn visit_projectionbody(&mut self, ctx: &ProjectionBodyContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#projectionItems.
    fn visit_projectionitems(&mut self, ctx: &ProjectionItemsContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#projectionItem.
    fn visit_projectionitem(&mut self, ctx: &ProjectionItemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#orderItem.
    fn visit_orderitem(&mut self, ctx: &OrderItemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#orderSt.
    fn visit_orderst(&mut self, ctx: &OrderStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#singlePartQ.
    fn visit_singlepartq(&mut self, ctx: &SinglePartQContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#multiPartQ.
    fn visit_multipartq(&mut self, ctx: &MultiPartQContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#matchSt.
    fn visit_matchst(&mut self, ctx: &MatchStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#unwindSt.
    fn visit_unwindst(&mut self, ctx: &UnwindStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#readingStatement.
    fn visit_readingstatement(&mut self, ctx: &ReadingStatementContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#updatingStatement.
    fn visit_updatingstatement(&mut self, ctx: &UpdatingStatementContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#deleteSt.
    fn visit_deletest(&mut self, ctx: &DeleteStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#removeSt.
    fn visit_removest(&mut self, ctx: &RemoveStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#removeItem.
    fn visit_removeitem(&mut self, ctx: &RemoveItemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#queryCallSt.
    fn visit_querycallst(&mut self, ctx: &QueryCallStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#parenExpressionChain.
    fn visit_parenexpressionchain(&mut self, ctx: &ParenExpressionChainContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#yieldItems.
    fn visit_yielditems(&mut self, ctx: &YieldItemsContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#yieldItem.
    fn visit_yielditem(&mut self, ctx: &YieldItemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#mergeSt.
    fn visit_mergest(&mut self, ctx: &MergeStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#mergeAction.
    fn visit_mergeaction(&mut self, ctx: &MergeActionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#setSt.
    fn visit_setst(&mut self, ctx: &SetStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#setItem.
    fn visit_setitem(&mut self, ctx: &SetItemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#nodeLabels.
    fn visit_nodelabels(&mut self, ctx: &NodeLabelsContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#createSt.
    fn visit_createst(&mut self, ctx: &CreateStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#patternWhere.
    fn visit_patternwhere(&mut self, ctx: &PatternWhereContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#where.
    fn visit_where(&mut self, ctx: &WhereContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#pattern.
    fn visit_pattern(&mut self, ctx: &PatternContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#expression.
    fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#xorExpression.
    fn visit_xorexpression(&mut self, ctx: &XorExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#andExpression.
    fn visit_andexpression(&mut self, ctx: &AndExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#notExpression.
    fn visit_notexpression(&mut self, ctx: &NotExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#comparisonExpression.
    fn visit_comparisonexpression(&mut self, ctx: &ComparisonExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#comparisonSigns.
    fn visit_comparisonsigns(&mut self, ctx: &ComparisonSignsContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#addSubExpression.
    fn visit_addsubexpression(&mut self, ctx: &AddSubExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#multDivExpression.
    fn visit_multdivexpression(&mut self, ctx: &MultDivExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#powerExpression.
    fn visit_powerexpression(&mut self, ctx: &PowerExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#unaryAddSubExpression.
    fn visit_unaryaddsubexpression(&mut self, ctx: &UnaryAddSubExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#atomicExpression.
    fn visit_atomicexpression(&mut self, ctx: &AtomicExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#listExpression.
    fn visit_listexpression(&mut self, ctx: &ListExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#stringExpression.
    fn visit_stringexpression(&mut self, ctx: &StringExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#stringExpPrefix.
    fn visit_stringexpprefix(&mut self, ctx: &StringExpPrefixContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#nullExpression.
    fn visit_nullexpression(&mut self, ctx: &NullExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#propertyOrLabelExpression.
    fn visit_propertyorlabelexpression(&mut self, ctx: &PropertyOrLabelExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#propertyExpression.
    fn visit_propertyexpression(&mut self, ctx: &PropertyExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#patternPart.
    fn visit_patternpart(&mut self, ctx: &PatternPartContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#patternElem.
    fn visit_patternelem(&mut self, ctx: &PatternElemContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#patternElemChain.
    fn visit_patternelemchain(&mut self, ctx: &PatternElemChainContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#qppElemChain.
    fn visit_qppelemchain(&mut self, ctx: &QppElemChainContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#qppQuantifier.
    fn visit_qppquantifier(&mut self, ctx: &QppQuantifierContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#qppInt.
    fn visit_qppint(&mut self, ctx: &QppIntContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#properties.
    fn visit_properties(&mut self, ctx: &PropertiesContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#nodePattern.
    fn visit_nodepattern(&mut self, ctx: &NodePatternContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#atom.
    fn visit_atom(&mut self, ctx: &AtomContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#lhs.
    fn visit_lhs(&mut self, ctx: &LhsContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#relationshipPattern.
    fn visit_relationshippattern(&mut self, ctx: &RelationshipPatternContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#relationDetail.
    fn visit_relationdetail(&mut self, ctx: &RelationDetailContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#relationshipTypes.
    fn visit_relationshiptypes(&mut self, ctx: &RelationshipTypesContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#unionSt.
    fn visit_unionst(&mut self, ctx: &UnionStContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#subqueryExist.
    fn visit_subqueryexist(&mut self, ctx: &SubqueryExistContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#invocationName.
    fn visit_invocationname(&mut self, ctx: &InvocationNameContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#functionInvocation.
    fn visit_functioninvocation(&mut self, ctx: &FunctionInvocationContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#parenthesizedExpression.
    fn visit_parenthesizedexpression(&mut self, ctx: &ParenthesizedExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#filterWith.
    fn visit_filterwith(&mut self, ctx: &FilterWithContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#patternComprehension.
    fn visit_patterncomprehension(&mut self, ctx: &PatternComprehensionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#relationshipsChainPattern.
    fn visit_relationshipschainpattern(&mut self, ctx: &RelationshipsChainPatternContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#listComprehension.
    fn visit_listcomprehension(&mut self, ctx: &ListComprehensionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#filterExpression.
    fn visit_filterexpression(&mut self, ctx: &FilterExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#countAll.
    fn visit_countall(&mut self, ctx: &CountAllContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#expressionChain.
    fn visit_expressionchain(&mut self, ctx: &ExpressionChainContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#caseExpression.
    fn visit_caseexpression(&mut self, ctx: &CaseExpressionContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#parameter.
    fn visit_parameter(&mut self, ctx: &ParameterContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#literal.
    fn visit_literal(&mut self, ctx: &LiteralContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#rangeLit.
    fn visit_rangelit(&mut self, ctx: &RangeLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#boolLit.
    fn visit_boollit(&mut self, ctx: &BoolLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#numLit.
    fn visit_numlit(&mut self, ctx: &NumLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#stringLit.
    fn visit_stringlit(&mut self, ctx: &StringLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#charLit.
    fn visit_charlit(&mut self, ctx: &CharLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#listLit.
    fn visit_listlit(&mut self, ctx: &ListLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#mapLit.
    fn visit_maplit(&mut self, ctx: &MapLitContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#mapPair.
    fn visit_mappair(&mut self, ctx: &MapPairContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#name.
    fn visit_name(&mut self, ctx: &NameContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#symbol.
    fn visit_symbol(&mut self, ctx: &SymbolContext<'input>) {
        self.visit_children(ctx)
    }

    // Visit a parse tree produced by CypherParser#reservedWord.
    fn visit_reservedword(&mut self, ctx: &ReservedWordContext<'input>) {
        self.visit_children(ctx)
    }
}
