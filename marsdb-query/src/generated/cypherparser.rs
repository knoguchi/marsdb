// Generated from CypherParser.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
use super::cypherparserlistener::*;
use super::cypherparservisitor::*;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::error_strategy::{DefaultErrorStrategy, ErrorStrategy};
use antlr4rust::errors::*;
use antlr4rust::int_stream::EOF;
use antlr4rust::parser::{BaseParser, Parser, ParserNodeType, ParserRecog};
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::parser_rule_context::{cast, cast_mut, BaseParserRuleContext, ParserRuleContext};
use antlr4rust::recognizer::{Actions, Recognizer};
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::token::{OwningToken, Token, TOKEN_EOF};
use antlr4rust::token_factory::{CommonTokenFactory, TokenAware, TokenFactory};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::tree::*;
use antlr4rust::vocabulary::{Vocabulary, VocabularyImpl};
use antlr4rust::PredictionContextCache;
use antlr4rust::TokenSource;

use antlr4rust::lazy_static;
use antlr4rust::{TidAble, TidExt};

use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

pub const CypherParser_ASSIGN: i32 = 1;
pub const CypherParser_ADD_ASSIGN: i32 = 2;
pub const CypherParser_LE: i32 = 3;
pub const CypherParser_GE: i32 = 4;
pub const CypherParser_GT: i32 = 5;
pub const CypherParser_LT: i32 = 6;
pub const CypherParser_NOT_EQUAL: i32 = 7;
pub const CypherParser_RANGE: i32 = 8;
pub const CypherParser_SEMI: i32 = 9;
pub const CypherParser_DOT: i32 = 10;
pub const CypherParser_COMMA: i32 = 11;
pub const CypherParser_LPAREN: i32 = 12;
pub const CypherParser_RPAREN: i32 = 13;
pub const CypherParser_LBRACE: i32 = 14;
pub const CypherParser_RBRACE: i32 = 15;
pub const CypherParser_LBRACK: i32 = 16;
pub const CypherParser_RBRACK: i32 = 17;
pub const CypherParser_SUB: i32 = 18;
pub const CypherParser_PLUS: i32 = 19;
pub const CypherParser_DIV: i32 = 20;
pub const CypherParser_MOD: i32 = 21;
pub const CypherParser_CARET: i32 = 22;
pub const CypherParser_MULT: i32 = 23;
pub const CypherParser_ESC: i32 = 24;
pub const CypherParser_COLON: i32 = 25;
pub const CypherParser_STICK: i32 = 26;
pub const CypherParser_DOLLAR: i32 = 27;
pub const CypherParser_CALL: i32 = 28;
pub const CypherParser_YIELD: i32 = 29;
pub const CypherParser_FILTER: i32 = 30;
pub const CypherParser_EXTRACT: i32 = 31;
pub const CypherParser_COUNT: i32 = 32;
pub const CypherParser_ANY: i32 = 33;
pub const CypherParser_NONE: i32 = 34;
pub const CypherParser_SINGLE: i32 = 35;
pub const CypherParser_ALL: i32 = 36;
pub const CypherParser_ASC: i32 = 37;
pub const CypherParser_ASCENDING: i32 = 38;
pub const CypherParser_BY: i32 = 39;
pub const CypherParser_CREATE: i32 = 40;
pub const CypherParser_DELETE: i32 = 41;
pub const CypherParser_DESC: i32 = 42;
pub const CypherParser_DESCENDING: i32 = 43;
pub const CypherParser_DETACH: i32 = 44;
pub const CypherParser_EXISTS: i32 = 45;
pub const CypherParser_EXPLAIN: i32 = 46;
pub const CypherParser_LIMIT: i32 = 47;
pub const CypherParser_MATCH: i32 = 48;
pub const CypherParser_MERGE: i32 = 49;
pub const CypherParser_ON: i32 = 50;
pub const CypherParser_OPTIONAL: i32 = 51;
pub const CypherParser_ORDER: i32 = 52;
pub const CypherParser_REMOVE: i32 = 53;
pub const CypherParser_RETURN: i32 = 54;
pub const CypherParser_SET: i32 = 55;
pub const CypherParser_SKIP_W: i32 = 56;
pub const CypherParser_WHERE: i32 = 57;
pub const CypherParser_WITH: i32 = 58;
pub const CypherParser_UNION: i32 = 59;
pub const CypherParser_UNWIND: i32 = 60;
pub const CypherParser_AND: i32 = 61;
pub const CypherParser_AS: i32 = 62;
pub const CypherParser_CONTAINS: i32 = 63;
pub const CypherParser_DISTINCT: i32 = 64;
pub const CypherParser_ENDS: i32 = 65;
pub const CypherParser_IN: i32 = 66;
pub const CypherParser_INDEX: i32 = 67;
pub const CypherParser_IS: i32 = 68;
pub const CypherParser_NOT: i32 = 69;
pub const CypherParser_OR: i32 = 70;
pub const CypherParser_STARTS: i32 = 71;
pub const CypherParser_XOR: i32 = 72;
pub const CypherParser_SHORTEST_PATH: i32 = 73;
pub const CypherParser_FALSE: i32 = 74;
pub const CypherParser_TRUE: i32 = 75;
pub const CypherParser_NULL_W: i32 = 76;
pub const CypherParser_CONSTRAINT: i32 = 77;
pub const CypherParser_DO: i32 = 78;
pub const CypherParser_FOR: i32 = 79;
pub const CypherParser_REQUIRE: i32 = 80;
pub const CypherParser_UNIQUE: i32 = 81;
pub const CypherParser_CASE: i32 = 82;
pub const CypherParser_WHEN: i32 = 83;
pub const CypherParser_THEN: i32 = 84;
pub const CypherParser_ELSE: i32 = 85;
pub const CypherParser_END: i32 = 86;
pub const CypherParser_MANDATORY: i32 = 87;
pub const CypherParser_SCALAR: i32 = 88;
pub const CypherParser_OF: i32 = 89;
pub const CypherParser_ADD: i32 = 90;
pub const CypherParser_DROP: i32 = 91;
pub const CypherParser_ID: i32 = 92;
pub const CypherParser_ESC_LITERAL: i32 = 93;
pub const CypherParser_CHAR_LITERAL: i32 = 94;
pub const CypherParser_STRING_LITERAL: i32 = 95;
pub const CypherParser_DIGIT: i32 = 96;
pub const CypherParser_FLOAT: i32 = 97;
pub const CypherParser_WS: i32 = 98;
pub const CypherParser_COMMENT: i32 = 99;
pub const CypherParser_LINE_COMMENT: i32 = 100;
pub const CypherParser_Letter: i32 = 101;
pub const CypherParser_EOF: i32 = EOF;
pub const RULE_script: usize = 0;
pub const RULE_queries: usize = 1;
pub const RULE_query: usize = 2;
pub const RULE_explainSt: usize = 3;
pub const RULE_createIndexSt: usize = 4;
pub const RULE_regularQuery: usize = 5;
pub const RULE_singleQuery: usize = 6;
pub const RULE_standaloneCall: usize = 7;
pub const RULE_returnSt: usize = 8;
pub const RULE_withSt: usize = 9;
pub const RULE_skipSt: usize = 10;
pub const RULE_limitSt: usize = 11;
pub const RULE_projectionBody: usize = 12;
pub const RULE_projectionItems: usize = 13;
pub const RULE_projectionItem: usize = 14;
pub const RULE_orderItem: usize = 15;
pub const RULE_orderSt: usize = 16;
pub const RULE_singlePartQ: usize = 17;
pub const RULE_multiPartQ: usize = 18;
pub const RULE_matchSt: usize = 19;
pub const RULE_unwindSt: usize = 20;
pub const RULE_readingStatement: usize = 21;
pub const RULE_updatingStatement: usize = 22;
pub const RULE_deleteSt: usize = 23;
pub const RULE_removeSt: usize = 24;
pub const RULE_removeItem: usize = 25;
pub const RULE_queryCallSt: usize = 26;
pub const RULE_parenExpressionChain: usize = 27;
pub const RULE_yieldItems: usize = 28;
pub const RULE_yieldItem: usize = 29;
pub const RULE_mergeSt: usize = 30;
pub const RULE_mergeAction: usize = 31;
pub const RULE_setSt: usize = 32;
pub const RULE_setItem: usize = 33;
pub const RULE_nodeLabels: usize = 34;
pub const RULE_createSt: usize = 35;
pub const RULE_patternWhere: usize = 36;
pub const RULE_where: usize = 37;
pub const RULE_pattern: usize = 38;
pub const RULE_expression: usize = 39;
pub const RULE_xorExpression: usize = 40;
pub const RULE_andExpression: usize = 41;
pub const RULE_notExpression: usize = 42;
pub const RULE_comparisonExpression: usize = 43;
pub const RULE_comparisonSigns: usize = 44;
pub const RULE_addSubExpression: usize = 45;
pub const RULE_multDivExpression: usize = 46;
pub const RULE_powerExpression: usize = 47;
pub const RULE_unaryAddSubExpression: usize = 48;
pub const RULE_atomicExpression: usize = 49;
pub const RULE_listExpression: usize = 50;
pub const RULE_stringExpression: usize = 51;
pub const RULE_stringExpPrefix: usize = 52;
pub const RULE_nullExpression: usize = 53;
pub const RULE_propertyOrLabelExpression: usize = 54;
pub const RULE_propertyExpression: usize = 55;
pub const RULE_patternPart: usize = 56;
pub const RULE_shortestPathWrapper: usize = 57;
pub const RULE_patternElem: usize = 58;
pub const RULE_patternElemChain: usize = 59;
pub const RULE_qppElemChain: usize = 60;
pub const RULE_qppQuantifier: usize = 61;
pub const RULE_qppInt: usize = 62;
pub const RULE_properties: usize = 63;
pub const RULE_nodePattern: usize = 64;
pub const RULE_atom: usize = 65;
pub const RULE_lhs: usize = 66;
pub const RULE_relationshipPattern: usize = 67;
pub const RULE_relationDetail: usize = 68;
pub const RULE_relationshipTypes: usize = 69;
pub const RULE_unionSt: usize = 70;
pub const RULE_subqueryExist: usize = 71;
pub const RULE_invocationName: usize = 72;
pub const RULE_functionInvocation: usize = 73;
pub const RULE_parenthesizedExpression: usize = 74;
pub const RULE_filterWith: usize = 75;
pub const RULE_patternComprehension: usize = 76;
pub const RULE_relationshipsChainPattern: usize = 77;
pub const RULE_listComprehension: usize = 78;
pub const RULE_filterExpression: usize = 79;
pub const RULE_countAll: usize = 80;
pub const RULE_expressionChain: usize = 81;
pub const RULE_caseExpression: usize = 82;
pub const RULE_parameter: usize = 83;
pub const RULE_literal: usize = 84;
pub const RULE_rangeLit: usize = 85;
pub const RULE_boolLit: usize = 86;
pub const RULE_numLit: usize = 87;
pub const RULE_stringLit: usize = 88;
pub const RULE_charLit: usize = 89;
pub const RULE_listLit: usize = 90;
pub const RULE_mapLit: usize = 91;
pub const RULE_mapPair: usize = 92;
pub const RULE_name: usize = 93;
pub const RULE_symbol: usize = 94;
pub const RULE_reservedWord: usize = 95;
pub const ruleNames: [&'static str; 96] = [
    "script",
    "queries",
    "query",
    "explainSt",
    "createIndexSt",
    "regularQuery",
    "singleQuery",
    "standaloneCall",
    "returnSt",
    "withSt",
    "skipSt",
    "limitSt",
    "projectionBody",
    "projectionItems",
    "projectionItem",
    "orderItem",
    "orderSt",
    "singlePartQ",
    "multiPartQ",
    "matchSt",
    "unwindSt",
    "readingStatement",
    "updatingStatement",
    "deleteSt",
    "removeSt",
    "removeItem",
    "queryCallSt",
    "parenExpressionChain",
    "yieldItems",
    "yieldItem",
    "mergeSt",
    "mergeAction",
    "setSt",
    "setItem",
    "nodeLabels",
    "createSt",
    "patternWhere",
    "where",
    "pattern",
    "expression",
    "xorExpression",
    "andExpression",
    "notExpression",
    "comparisonExpression",
    "comparisonSigns",
    "addSubExpression",
    "multDivExpression",
    "powerExpression",
    "unaryAddSubExpression",
    "atomicExpression",
    "listExpression",
    "stringExpression",
    "stringExpPrefix",
    "nullExpression",
    "propertyOrLabelExpression",
    "propertyExpression",
    "patternPart",
    "shortestPathWrapper",
    "patternElem",
    "patternElemChain",
    "qppElemChain",
    "qppQuantifier",
    "qppInt",
    "properties",
    "nodePattern",
    "atom",
    "lhs",
    "relationshipPattern",
    "relationDetail",
    "relationshipTypes",
    "unionSt",
    "subqueryExist",
    "invocationName",
    "functionInvocation",
    "parenthesizedExpression",
    "filterWith",
    "patternComprehension",
    "relationshipsChainPattern",
    "listComprehension",
    "filterExpression",
    "countAll",
    "expressionChain",
    "caseExpression",
    "parameter",
    "literal",
    "rangeLit",
    "boolLit",
    "numLit",
    "stringLit",
    "charLit",
    "listLit",
    "mapLit",
    "mapPair",
    "name",
    "symbol",
    "reservedWord",
];

pub const _LITERAL_NAMES: [Option<&'static str>; 92] = [
    None,
    Some("'='"),
    Some("'+='"),
    Some("'<='"),
    Some("'>='"),
    Some("'>'"),
    Some("'<'"),
    Some("'<>'"),
    Some("'..'"),
    Some("';'"),
    Some("'.'"),
    Some("','"),
    Some("'('"),
    Some("')'"),
    Some("'{'"),
    Some("'}'"),
    Some("'['"),
    Some("']'"),
    Some("'-'"),
    Some("'+'"),
    Some("'/'"),
    Some("'%'"),
    Some("'^'"),
    Some("'*'"),
    Some("'`'"),
    Some("':'"),
    Some("'|'"),
    Some("'$'"),
    Some("'CALL'"),
    Some("'YIELD'"),
    Some("'FILTER'"),
    Some("'EXTRACT'"),
    Some("'COUNT'"),
    Some("'ANY'"),
    Some("'NONE'"),
    Some("'SINGLE'"),
    Some("'ALL'"),
    Some("'ASC'"),
    Some("'ASCENDING'"),
    Some("'BY'"),
    Some("'CREATE'"),
    Some("'DELETE'"),
    Some("'DESC'"),
    Some("'DESCENDING'"),
    Some("'DETACH'"),
    Some("'EXISTS'"),
    Some("'EXPLAIN'"),
    Some("'LIMIT'"),
    Some("'MATCH'"),
    Some("'MERGE'"),
    Some("'ON'"),
    Some("'OPTIONAL'"),
    Some("'ORDER'"),
    Some("'REMOVE'"),
    Some("'RETURN'"),
    Some("'SET'"),
    Some("'SKIP'"),
    Some("'WHERE'"),
    Some("'WITH'"),
    Some("'UNION'"),
    Some("'UNWIND'"),
    Some("'AND'"),
    Some("'AS'"),
    Some("'CONTAINS'"),
    Some("'DISTINCT'"),
    Some("'ENDS'"),
    Some("'IN'"),
    Some("'INDEX'"),
    Some("'IS'"),
    Some("'NOT'"),
    Some("'OR'"),
    Some("'STARTS'"),
    Some("'XOR'"),
    Some("'shortestPath'"),
    Some("'FALSE'"),
    Some("'TRUE'"),
    Some("'NULL'"),
    Some("'CONSTRAINT'"),
    Some("'DO'"),
    Some("'FOR'"),
    Some("'REQUIRE'"),
    Some("'UNIQUE'"),
    Some("'CASE'"),
    Some("'WHEN'"),
    Some("'THEN'"),
    Some("'ELSE'"),
    Some("'END'"),
    Some("'MANDATORY'"),
    Some("'SCALAR'"),
    Some("'OF'"),
    Some("'ADD'"),
    Some("'DROP'"),
];
pub const _SYMBOLIC_NAMES: [Option<&'static str>; 102] = [
    None,
    Some("ASSIGN"),
    Some("ADD_ASSIGN"),
    Some("LE"),
    Some("GE"),
    Some("GT"),
    Some("LT"),
    Some("NOT_EQUAL"),
    Some("RANGE"),
    Some("SEMI"),
    Some("DOT"),
    Some("COMMA"),
    Some("LPAREN"),
    Some("RPAREN"),
    Some("LBRACE"),
    Some("RBRACE"),
    Some("LBRACK"),
    Some("RBRACK"),
    Some("SUB"),
    Some("PLUS"),
    Some("DIV"),
    Some("MOD"),
    Some("CARET"),
    Some("MULT"),
    Some("ESC"),
    Some("COLON"),
    Some("STICK"),
    Some("DOLLAR"),
    Some("CALL"),
    Some("YIELD"),
    Some("FILTER"),
    Some("EXTRACT"),
    Some("COUNT"),
    Some("ANY"),
    Some("NONE"),
    Some("SINGLE"),
    Some("ALL"),
    Some("ASC"),
    Some("ASCENDING"),
    Some("BY"),
    Some("CREATE"),
    Some("DELETE"),
    Some("DESC"),
    Some("DESCENDING"),
    Some("DETACH"),
    Some("EXISTS"),
    Some("EXPLAIN"),
    Some("LIMIT"),
    Some("MATCH"),
    Some("MERGE"),
    Some("ON"),
    Some("OPTIONAL"),
    Some("ORDER"),
    Some("REMOVE"),
    Some("RETURN"),
    Some("SET"),
    Some("SKIP_W"),
    Some("WHERE"),
    Some("WITH"),
    Some("UNION"),
    Some("UNWIND"),
    Some("AND"),
    Some("AS"),
    Some("CONTAINS"),
    Some("DISTINCT"),
    Some("ENDS"),
    Some("IN"),
    Some("INDEX"),
    Some("IS"),
    Some("NOT"),
    Some("OR"),
    Some("STARTS"),
    Some("XOR"),
    Some("SHORTEST_PATH"),
    Some("FALSE"),
    Some("TRUE"),
    Some("NULL_W"),
    Some("CONSTRAINT"),
    Some("DO"),
    Some("FOR"),
    Some("REQUIRE"),
    Some("UNIQUE"),
    Some("CASE"),
    Some("WHEN"),
    Some("THEN"),
    Some("ELSE"),
    Some("END"),
    Some("MANDATORY"),
    Some("SCALAR"),
    Some("OF"),
    Some("ADD"),
    Some("DROP"),
    Some("ID"),
    Some("ESC_LITERAL"),
    Some("CHAR_LITERAL"),
    Some("STRING_LITERAL"),
    Some("DIGIT"),
    Some("FLOAT"),
    Some("WS"),
    Some("COMMENT"),
    Some("LINE_COMMENT"),
    Some("Letter"),
];
lazy_static! {
    static ref _shared_context_cache: Arc<PredictionContextCache> =
        Arc::new(PredictionContextCache::new());
    static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(
        _LITERAL_NAMES.iter(),
        _SYMBOLIC_NAMES.iter(),
        None
    ));
}

type BaseParserType<'input, I> = BaseParser<
    'input,
    CypherParserExt<'input>,
    I,
    CypherParserContextType,
    dyn CypherParserListener<'input> + 'input,
>;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type CypherParserTreeWalker<'input, 'a> =
    ParseTreeWalker<'input, 'a, CypherParserContextType, dyn CypherParserListener<'input> + 'a>;

/// Parser for CypherParser grammar
pub struct CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    base: BaseParserType<'input, I>,
    interpreter: Arc<ParserATNSimulator>,
    _shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn set_error_strategy(
        &mut self,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) {
        self.err_handler = strategy
    }

    pub fn with_strategy(
        input: I,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) -> Self {
        antlr4rust::recognizer::check_version("0", "5");
        let interpreter = Arc::new(ParserATNSimulator::new(
            _ATN.clone(),
            _decision_to_DFA.clone(),
            _shared_context_cache.clone(),
        ));
        Self {
            base: BaseParser::new_base_parser(
                input,
                Arc::clone(&interpreter),
                CypherParserExt {
                    _pd: Default::default(),
                },
            ),
            interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }
}

type DynStrategy<'input, I> = Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>> + 'input>;

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn new(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for CypherParser
pub trait CypherParserContext<'input>:
    for<'x> Listenable<dyn CypherParserListener<'input> + 'x>
    + for<'x> Visitable<dyn CypherParserVisitor<'input> + 'x>
    + ParserRuleContext<'input, TF = LocalTokenFactory<'input>, Ctx = CypherParserContextType>
{
}

antlr4rust::coerce_from! { 'input : CypherParserContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn CypherParserContext<'input> + 'input
where
    T: CypherParserVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn CypherParserVisitor<'input> + 'x))
    }
}

impl<'input> CypherParserContext<'input> for TerminalNode<'input, CypherParserContextType> {}
impl<'input> CypherParserContext<'input> for ErrorNode<'input, CypherParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn CypherParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn CypherParserListener<'input> + 'input }

pub struct CypherParserContextType;
antlr4rust::tid! {CypherParserContextType}

impl<'input> ParserNodeType<'input> for CypherParserContextType {
    type TF = LocalTokenFactory<'input>;
    type Type = dyn CypherParserContext<'input> + 'input;
}

impl<'input, I> Deref for CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    type Target = BaseParserType<'input, I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct CypherParserExt<'input> {
    _pd: PhantomData<&'input str>,
}

impl<'input> CypherParserExt<'input> {}
antlr4rust::tid! { CypherParserExt<'a> }

impl<'input> TokenAware<'input> for CypherParserExt<'input> {
    type TF = LocalTokenFactory<'input>;
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    ParserRecog<'input, BaseParserType<'input, I>> for CypherParserExt<'input>
{
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    Actions<'input, BaseParserType<'input, I>> for CypherParserExt<'input>
{
    fn get_grammar_file_name(&self) -> &str {
        "CypherParser.g4"
    }

    fn get_rule_names(&self) -> &[&str] {
        &ruleNames
    }

    fn get_vocabulary(&self) -> &dyn Vocabulary {
        &**VOCABULARY
    }
}
//------------------- script ----------------
pub type ScriptContextAll<'input> = ScriptContext<'input>;

pub type ScriptContext<'input> = BaseParserRuleContext<'input, ScriptContextExt<'input>>;

#[derive(Clone)]
pub struct ScriptContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ScriptContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ScriptContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_script(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_script(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ScriptContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_script(self);
    }
}

impl<'input> CustomRuleContext<'input> for ScriptContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_script
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_script }
}
antlr4rust::tid! {ScriptContextExt<'a>}

impl<'input> ScriptContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ScriptContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ScriptContextExt { ph: PhantomData },
        ))
    }
}

pub trait ScriptContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ScriptContextExt<'input>>
{
    fn query(&self) -> Option<Rc<QueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token EOF
    /// Returns `None` if there is no child corresponding to token EOF
    fn EOF(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EOF, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SEMI
    /// Returns `None` if there is no child corresponding to token SEMI
    fn SEMI(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SEMI, 0)
    }
}

impl<'input> ScriptContextAttrs<'input> for ScriptContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn script(&mut self) -> Result<Rc<ScriptContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ScriptContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_script);
        let mut _localctx: Rc<ScriptContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule query*/
                recog.base.set_state(192);
                recog.query()?;

                recog.base.set_state(194);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SEMI {
                    {
                        recog.base.set_state(193);
                        recog
                            .base
                            .match_token(CypherParser_SEMI, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(196);
                recog
                    .base
                    .match_token(CypherParser_EOF, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- queries ----------------
pub type QueriesContextAll<'input> = QueriesContext<'input>;

pub type QueriesContext<'input> = BaseParserRuleContext<'input, QueriesContextExt<'input>>;

#[derive(Clone)]
pub struct QueriesContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QueriesContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for QueriesContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_queries(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_queries(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QueriesContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_queries(self);
    }
}

impl<'input> CustomRuleContext<'input> for QueriesContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_queries
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_queries }
}
antlr4rust::tid! {QueriesContextExt<'a>}

impl<'input> QueriesContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QueriesContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QueriesContextExt { ph: PhantomData },
        ))
    }
}

pub trait QueriesContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QueriesContextExt<'input>>
{
    fn query_all(&self) -> Vec<Rc<QueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn query(&self, i: usize) -> Option<Rc<QueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token EOF
    /// Returns `None` if there is no child corresponding to token EOF
    fn EOF(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EOF, 0)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SEMI in current rule
    fn SEMI_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SEMI, starting from 0.
    /// Returns `None` if number of children corresponding to token SEMI is less or equal than `i`.
    fn SEMI(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SEMI, i)
    }
}

impl<'input> QueriesContextAttrs<'input> for QueriesContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn queries(&mut self) -> Result<Rc<QueriesContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = QueriesContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_queries);
        let mut _localctx: Rc<QueriesContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule query*/
                recog.base.set_state(198);
                recog.query()?;

                recog.base.set_state(203);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_SEMI {
                    {
                        {
                            recog.base.set_state(199);
                            recog
                                .base
                                .match_token(CypherParser_SEMI, &mut recog.err_handler)?;

                            /*InvokeRule query*/
                            recog.base.set_state(200);
                            recog.query()?;
                        }
                    }
                    recog.base.set_state(205);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(206);
                recog
                    .base
                    .match_token(CypherParser_EOF, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- query ----------------
pub type QueryContextAll<'input> = QueryContext<'input>;

pub type QueryContext<'input> = BaseParserRuleContext<'input, QueryContextExt<'input>>;

#[derive(Clone)]
pub struct QueryContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QueryContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for QueryContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_query(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_query(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QueryContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_query(self);
    }
}

impl<'input> CustomRuleContext<'input> for QueryContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_query
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_query }
}
antlr4rust::tid! {QueryContextExt<'a>}

impl<'input> QueryContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QueryContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QueryContextExt { ph: PhantomData },
        ))
    }
}

pub trait QueryContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QueryContextExt<'input>>
{
    fn explainSt(&self) -> Option<Rc<ExplainStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn regularQuery(&self) -> Option<Rc<RegularQueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn standaloneCall(&self) -> Option<Rc<StandaloneCallContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn createIndexSt(&self) -> Option<Rc<CreateIndexStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> QueryContextAttrs<'input> for QueryContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn query(&mut self) -> Result<Rc<QueryContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = QueryContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_query);
        let mut _localctx: Rc<QueryContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(212);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(2, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule explainSt*/
                        recog.base.set_state(208);
                        recog.explainSt()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule regularQuery*/
                        recog.base.set_state(209);
                        recog.regularQuery()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule standaloneCall*/
                        recog.base.set_state(210);
                        recog.standaloneCall()?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule createIndexSt*/
                        recog.base.set_state(211);
                        recog.createIndexSt()?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- explainSt ----------------
pub type ExplainStContextAll<'input> = ExplainStContext<'input>;

pub type ExplainStContext<'input> = BaseParserRuleContext<'input, ExplainStContextExt<'input>>;

#[derive(Clone)]
pub struct ExplainStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ExplainStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ExplainStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_explainSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_explainSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ExplainStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_explainSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for ExplainStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_explainSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_explainSt }
}
antlr4rust::tid! {ExplainStContextExt<'a>}

impl<'input> ExplainStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ExplainStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ExplainStContextExt { ph: PhantomData },
        ))
    }
}

pub trait ExplainStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ExplainStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token EXPLAIN
    /// Returns `None` if there is no child corresponding to token EXPLAIN
    fn EXPLAIN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EXPLAIN, 0)
    }
    fn createIndexSt(&self) -> Option<Rc<CreateIndexStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn regularQuery(&self) -> Option<Rc<RegularQueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ExplainStContextAttrs<'input> for ExplainStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn explainSt(&mut self) -> Result<Rc<ExplainStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ExplainStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_explainSt);
        let mut _localctx: Rc<ExplainStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(214);
                recog
                    .base
                    .match_token(CypherParser_EXPLAIN, &mut recog.err_handler)?;

                recog.base.set_state(217);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.interpreter.adaptive_predict(3, &mut recog.base)? {
                    1 => {
                        {
                            /*InvokeRule createIndexSt*/
                            recog.base.set_state(215);
                            recog.createIndexSt()?;
                        }
                    }
                    2 => {
                        {
                            /*InvokeRule regularQuery*/
                            recog.base.set_state(216);
                            recog.regularQuery()?;
                        }
                    }

                    _ => {}
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- createIndexSt ----------------
pub type CreateIndexStContextAll<'input> = CreateIndexStContext<'input>;

pub type CreateIndexStContext<'input> =
    BaseParserRuleContext<'input, CreateIndexStContextExt<'input>>;

#[derive(Clone)]
pub struct CreateIndexStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for CreateIndexStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for CreateIndexStContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_createIndexSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_createIndexSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for CreateIndexStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_createIndexSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for CreateIndexStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_createIndexSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_createIndexSt }
}
antlr4rust::tid! {CreateIndexStContextExt<'a>}

impl<'input> CreateIndexStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CreateIndexStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CreateIndexStContextExt { ph: PhantomData },
        ))
    }
}

pub trait CreateIndexStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<CreateIndexStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CREATE
    /// Returns `None` if there is no child corresponding to token CREATE
    fn CREATE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CREATE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token INDEX
    /// Returns `None` if there is no child corresponding to token INDEX
    fn INDEX(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_INDEX, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ON
    /// Returns `None` if there is no child corresponding to token ON
    fn ON(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ON, 0)
    }
    /// Retrieves first TerminalNode corresponding to token COLON
    /// Returns `None` if there is no child corresponding to token COLON
    fn COLON(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COLON, 0)
    }
    fn name_all(&self) -> Vec<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn name(&self, i: usize) -> Option<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token UNIQUE
    /// Returns `None` if there is no child corresponding to token UNIQUE
    fn UNIQUE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNIQUE, 0)
    }
}

impl<'input> CreateIndexStContextAttrs<'input> for CreateIndexStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn createIndexSt(&mut self) -> Result<Rc<CreateIndexStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            CreateIndexStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 8, RULE_createIndexSt);
        let mut _localctx: Rc<CreateIndexStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(219);
                recog
                    .base
                    .match_token(CypherParser_CREATE, &mut recog.err_handler)?;

                recog.base.set_state(220);
                recog
                    .base
                    .match_token(CypherParser_INDEX, &mut recog.err_handler)?;

                recog.base.set_state(221);
                recog
                    .base
                    .match_token(CypherParser_ON, &mut recog.err_handler)?;

                recog.base.set_state(222);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(223);
                recog.name()?;

                recog.base.set_state(224);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(225);
                recog.name()?;

                recog.base.set_state(226);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;

                recog.base.set_state(228);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_UNIQUE {
                    {
                        recog.base.set_state(227);
                        recog
                            .base
                            .match_token(CypherParser_UNIQUE, &mut recog.err_handler)?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- regularQuery ----------------
pub type RegularQueryContextAll<'input> = RegularQueryContext<'input>;

pub type RegularQueryContext<'input> =
    BaseParserRuleContext<'input, RegularQueryContextExt<'input>>;

#[derive(Clone)]
pub struct RegularQueryContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RegularQueryContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for RegularQueryContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_regularQuery(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_regularQuery(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for RegularQueryContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_regularQuery(self);
    }
}

impl<'input> CustomRuleContext<'input> for RegularQueryContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_regularQuery
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_regularQuery }
}
antlr4rust::tid! {RegularQueryContextExt<'a>}

impl<'input> RegularQueryContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RegularQueryContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RegularQueryContextExt { ph: PhantomData },
        ))
    }
}

pub trait RegularQueryContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RegularQueryContextExt<'input>>
{
    fn singleQuery(&self) -> Option<Rc<SingleQueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn unionSt_all(&self) -> Vec<Rc<UnionStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn unionSt(&self, i: usize) -> Option<Rc<UnionStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> RegularQueryContextAttrs<'input> for RegularQueryContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn regularQuery(&mut self) -> Result<Rc<RegularQueryContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = RegularQueryContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 10, RULE_regularQuery);
        let mut _localctx: Rc<RegularQueryContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule singleQuery*/
                recog.base.set_state(230);
                recog.singleQuery()?;

                recog.base.set_state(234);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_UNION {
                    {
                        {
                            /*InvokeRule unionSt*/
                            recog.base.set_state(231);
                            recog.unionSt()?;
                        }
                    }
                    recog.base.set_state(236);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- singleQuery ----------------
pub type SingleQueryContextAll<'input> = SingleQueryContext<'input>;

pub type SingleQueryContext<'input> = BaseParserRuleContext<'input, SingleQueryContextExt<'input>>;

#[derive(Clone)]
pub struct SingleQueryContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SingleQueryContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SingleQueryContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_singleQuery(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_singleQuery(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SingleQueryContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_singleQuery(self);
    }
}

impl<'input> CustomRuleContext<'input> for SingleQueryContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_singleQuery
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_singleQuery }
}
antlr4rust::tid! {SingleQueryContextExt<'a>}

impl<'input> SingleQueryContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SingleQueryContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SingleQueryContextExt { ph: PhantomData },
        ))
    }
}

pub trait SingleQueryContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SingleQueryContextExt<'input>>
{
    fn singlePartQ(&self) -> Option<Rc<SinglePartQContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn multiPartQ(&self) -> Option<Rc<MultiPartQContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> SingleQueryContextAttrs<'input> for SingleQueryContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn singleQuery(&mut self) -> Result<Rc<SingleQueryContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SingleQueryContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 12, RULE_singleQuery);
        let mut _localctx: Rc<SingleQueryContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(239);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(6, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule singlePartQ*/
                        recog.base.set_state(237);
                        recog.singlePartQ()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule multiPartQ*/
                        recog.base.set_state(238);
                        recog.multiPartQ()?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- standaloneCall ----------------
pub type StandaloneCallContextAll<'input> = StandaloneCallContext<'input>;

pub type StandaloneCallContext<'input> =
    BaseParserRuleContext<'input, StandaloneCallContextExt<'input>>;

#[derive(Clone)]
pub struct StandaloneCallContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for StandaloneCallContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for StandaloneCallContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_standaloneCall(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_standaloneCall(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for StandaloneCallContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_standaloneCall(self);
    }
}

impl<'input> CustomRuleContext<'input> for StandaloneCallContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_standaloneCall
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_standaloneCall }
}
antlr4rust::tid! {StandaloneCallContextExt<'a>}

impl<'input> StandaloneCallContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<StandaloneCallContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            StandaloneCallContextExt { ph: PhantomData },
        ))
    }
}

pub trait StandaloneCallContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<StandaloneCallContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CALL
    /// Returns `None` if there is no child corresponding to token CALL
    fn CALL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CALL, 0)
    }
    fn invocationName(&self) -> Option<Rc<InvocationNameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn parenExpressionChain(&self) -> Option<Rc<ParenExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token YIELD
    /// Returns `None` if there is no child corresponding to token YIELD
    fn YIELD(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_YIELD, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MULT
    /// Returns `None` if there is no child corresponding to token MULT
    fn MULT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, 0)
    }
    fn yieldItems(&self) -> Option<Rc<YieldItemsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> StandaloneCallContextAttrs<'input> for StandaloneCallContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn standaloneCall(&mut self) -> Result<Rc<StandaloneCallContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            StandaloneCallContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 14, RULE_standaloneCall);
        let mut _localctx: Rc<StandaloneCallContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(241);
                recog
                    .base
                    .match_token(CypherParser_CALL, &mut recog.err_handler)?;

                /*InvokeRule invocationName*/
                recog.base.set_state(242);
                recog.invocationName()?;

                recog.base.set_state(244);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LPAREN {
                    {
                        /*InvokeRule parenExpressionChain*/
                        recog.base.set_state(243);
                        recog.parenExpressionChain()?;
                    }
                }

                recog.base.set_state(251);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_YIELD {
                    {
                        recog.base.set_state(246);
                        recog
                            .base
                            .match_token(CypherParser_YIELD, &mut recog.err_handler)?;

                        recog.base.set_state(249);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.base.input.la(1) {
                            CypherParser_MULT => {
                                recog.base.set_state(247);
                                recog
                                    .base
                                    .match_token(CypherParser_MULT, &mut recog.err_handler)?;
                            }

                            CypherParser_FILTER
                            | CypherParser_EXTRACT
                            | CypherParser_COUNT
                            | CypherParser_ANY
                            | CypherParser_NONE
                            | CypherParser_SINGLE
                            | CypherParser_ID
                            | CypherParser_ESC_LITERAL => {
                                {
                                    /*InvokeRule yieldItems*/
                                    recog.base.set_state(248);
                                    recog.yieldItems()?;
                                }
                            }

                            _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                                &mut recog.base,
                            )))?,
                        }
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- returnSt ----------------
pub type ReturnStContextAll<'input> = ReturnStContext<'input>;

pub type ReturnStContext<'input> = BaseParserRuleContext<'input, ReturnStContextExt<'input>>;

#[derive(Clone)]
pub struct ReturnStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ReturnStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ReturnStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_returnSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_returnSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ReturnStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_returnSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for ReturnStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_returnSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_returnSt }
}
antlr4rust::tid! {ReturnStContextExt<'a>}

impl<'input> ReturnStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ReturnStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ReturnStContextExt { ph: PhantomData },
        ))
    }
}

pub trait ReturnStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ReturnStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token RETURN
    /// Returns `None` if there is no child corresponding to token RETURN
    fn RETURN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RETURN, 0)
    }
    fn projectionBody(&self) -> Option<Rc<ProjectionBodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ReturnStContextAttrs<'input> for ReturnStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn returnSt(&mut self) -> Result<Rc<ReturnStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ReturnStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 16, RULE_returnSt);
        let mut _localctx: Rc<ReturnStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(253);
                recog
                    .base
                    .match_token(CypherParser_RETURN, &mut recog.err_handler)?;

                /*InvokeRule projectionBody*/
                recog.base.set_state(254);
                recog.projectionBody()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- withSt ----------------
pub type WithStContextAll<'input> = WithStContext<'input>;

pub type WithStContext<'input> = BaseParserRuleContext<'input, WithStContextExt<'input>>;

#[derive(Clone)]
pub struct WithStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for WithStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for WithStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_withSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_withSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for WithStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_withSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for WithStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_withSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_withSt }
}
antlr4rust::tid! {WithStContextExt<'a>}

impl<'input> WithStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<WithStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            WithStContextExt { ph: PhantomData },
        ))
    }
}

pub trait WithStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<WithStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token WITH
    /// Returns `None` if there is no child corresponding to token WITH
    fn WITH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WITH, 0)
    }
    fn projectionBody(&self) -> Option<Rc<ProjectionBodyContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn where_(&self) -> Option<Rc<WhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> WithStContextAttrs<'input> for WithStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn withSt(&mut self) -> Result<Rc<WithStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = WithStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_withSt);
        let mut _localctx: Rc<WithStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(256);
                recog
                    .base
                    .match_token(CypherParser_WITH, &mut recog.err_handler)?;

                /*InvokeRule projectionBody*/
                recog.base.set_state(257);
                recog.projectionBody()?;

                recog.base.set_state(259);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(258);
                        recog.where_()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- skipSt ----------------
pub type SkipStContextAll<'input> = SkipStContext<'input>;

pub type SkipStContext<'input> = BaseParserRuleContext<'input, SkipStContextExt<'input>>;

#[derive(Clone)]
pub struct SkipStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SkipStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SkipStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_skipSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_skipSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SkipStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_skipSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for SkipStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_skipSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_skipSt }
}
antlr4rust::tid! {SkipStContextExt<'a>}

impl<'input> SkipStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SkipStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SkipStContextExt { ph: PhantomData },
        ))
    }
}

pub trait SkipStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SkipStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token SKIP_W
    /// Returns `None` if there is no child corresponding to token SKIP_W
    fn SKIP_W(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SKIP_W, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> SkipStContextAttrs<'input> for SkipStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn skipSt(&mut self) -> Result<Rc<SkipStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SkipStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_skipSt);
        let mut _localctx: Rc<SkipStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(261);
                recog
                    .base
                    .match_token(CypherParser_SKIP_W, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(262);
                recog.expression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- limitSt ----------------
pub type LimitStContextAll<'input> = LimitStContext<'input>;

pub type LimitStContext<'input> = BaseParserRuleContext<'input, LimitStContextExt<'input>>;

#[derive(Clone)]
pub struct LimitStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for LimitStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for LimitStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_limitSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_limitSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for LimitStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_limitSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for LimitStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_limitSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_limitSt }
}
antlr4rust::tid! {LimitStContextExt<'a>}

impl<'input> LimitStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<LimitStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            LimitStContextExt { ph: PhantomData },
        ))
    }
}

pub trait LimitStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<LimitStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LIMIT
    /// Returns `None` if there is no child corresponding to token LIMIT
    fn LIMIT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LIMIT, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> LimitStContextAttrs<'input> for LimitStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn limitSt(&mut self) -> Result<Rc<LimitStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = LimitStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_limitSt);
        let mut _localctx: Rc<LimitStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(264);
                recog
                    .base
                    .match_token(CypherParser_LIMIT, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(265);
                recog.expression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- projectionBody ----------------
pub type ProjectionBodyContextAll<'input> = ProjectionBodyContext<'input>;

pub type ProjectionBodyContext<'input> =
    BaseParserRuleContext<'input, ProjectionBodyContextExt<'input>>;

#[derive(Clone)]
pub struct ProjectionBodyContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ProjectionBodyContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ProjectionBodyContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_projectionBody(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_projectionBody(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ProjectionBodyContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_projectionBody(self);
    }
}

impl<'input> CustomRuleContext<'input> for ProjectionBodyContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_projectionBody
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_projectionBody }
}
antlr4rust::tid! {ProjectionBodyContextExt<'a>}

impl<'input> ProjectionBodyContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ProjectionBodyContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ProjectionBodyContextExt { ph: PhantomData },
        ))
    }
}

pub trait ProjectionBodyContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ProjectionBodyContextExt<'input>>
{
    fn projectionItems(&self) -> Option<Rc<ProjectionItemsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token DISTINCT
    /// Returns `None` if there is no child corresponding to token DISTINCT
    fn DISTINCT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DISTINCT, 0)
    }
    fn orderSt(&self) -> Option<Rc<OrderStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn skipSt(&self) -> Option<Rc<SkipStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn limitSt(&self) -> Option<Rc<LimitStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ProjectionBodyContextAttrs<'input> for ProjectionBodyContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn projectionBody(&mut self) -> Result<Rc<ProjectionBodyContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ProjectionBodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 24, RULE_projectionBody);
        let mut _localctx: Rc<ProjectionBodyContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(268);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DISTINCT {
                    {
                        recog.base.set_state(267);
                        recog
                            .base
                            .match_token(CypherParser_DISTINCT, &mut recog.err_handler)?;
                    }
                }

                /*InvokeRule projectionItems*/
                recog.base.set_state(270);
                recog.projectionItems()?;

                recog.base.set_state(272);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ORDER {
                    {
                        /*InvokeRule orderSt*/
                        recog.base.set_state(271);
                        recog.orderSt()?;
                    }
                }

                recog.base.set_state(275);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SKIP_W {
                    {
                        /*InvokeRule skipSt*/
                        recog.base.set_state(274);
                        recog.skipSt()?;
                    }
                }

                recog.base.set_state(278);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LIMIT {
                    {
                        /*InvokeRule limitSt*/
                        recog.base.set_state(277);
                        recog.limitSt()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- projectionItems ----------------
pub type ProjectionItemsContextAll<'input> = ProjectionItemsContext<'input>;

pub type ProjectionItemsContext<'input> =
    BaseParserRuleContext<'input, ProjectionItemsContextExt<'input>>;

#[derive(Clone)]
pub struct ProjectionItemsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ProjectionItemsContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ProjectionItemsContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_projectionItems(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_projectionItems(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ProjectionItemsContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_projectionItems(self);
    }
}

impl<'input> CustomRuleContext<'input> for ProjectionItemsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_projectionItems
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_projectionItems }
}
antlr4rust::tid! {ProjectionItemsContextExt<'a>}

impl<'input> ProjectionItemsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ProjectionItemsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ProjectionItemsContextExt { ph: PhantomData },
        ))
    }
}

pub trait ProjectionItemsContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ProjectionItemsContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token MULT
    /// Returns `None` if there is no child corresponding to token MULT
    fn MULT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, 0)
    }
    fn projectionItem_all(&self) -> Vec<Rc<ProjectionItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn projectionItem(&self, i: usize) -> Option<Rc<ProjectionItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> ProjectionItemsContextAttrs<'input> for ProjectionItemsContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn projectionItems(&mut self) -> Result<Rc<ProjectionItemsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ProjectionItemsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 26, RULE_projectionItems);
        let mut _localctx: Rc<ProjectionItemsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(282);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_MULT => {
                        recog.base.set_state(280);
                        recog
                            .base
                            .match_token(CypherParser_MULT, &mut recog.err_handler)?;
                    }

                    CypherParser_LPAREN
                    | CypherParser_LBRACE
                    | CypherParser_LBRACK
                    | CypherParser_SUB
                    | CypherParser_PLUS
                    | CypherParser_DOLLAR
                    | CypherParser_FILTER
                    | CypherParser_EXTRACT
                    | CypherParser_COUNT
                    | CypherParser_ANY
                    | CypherParser_NONE
                    | CypherParser_SINGLE
                    | CypherParser_ALL
                    | CypherParser_EXISTS
                    | CypherParser_NOT
                    | CypherParser_FALSE
                    | CypherParser_TRUE
                    | CypherParser_NULL_W
                    | CypherParser_CASE
                    | CypherParser_ID
                    | CypherParser_ESC_LITERAL
                    | CypherParser_CHAR_LITERAL
                    | CypherParser_STRING_LITERAL
                    | CypherParser_DIGIT => {
                        {
                            /*InvokeRule projectionItem*/
                            recog.base.set_state(281);
                            recog.projectionItem()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
                recog.base.set_state(288);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(284);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule projectionItem*/
                            recog.base.set_state(285);
                            recog.projectionItem()?;
                        }
                    }
                    recog.base.set_state(290);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- projectionItem ----------------
pub type ProjectionItemContextAll<'input> = ProjectionItemContext<'input>;

pub type ProjectionItemContext<'input> =
    BaseParserRuleContext<'input, ProjectionItemContextExt<'input>>;

#[derive(Clone)]
pub struct ProjectionItemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ProjectionItemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ProjectionItemContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_projectionItem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_projectionItem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ProjectionItemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_projectionItem(self);
    }
}

impl<'input> CustomRuleContext<'input> for ProjectionItemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_projectionItem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_projectionItem }
}
antlr4rust::tid! {ProjectionItemContextExt<'a>}

impl<'input> ProjectionItemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ProjectionItemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ProjectionItemContextExt { ph: PhantomData },
        ))
    }
}

pub trait ProjectionItemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ProjectionItemContextExt<'input>>
{
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token AS
    /// Returns `None` if there is no child corresponding to token AS
    fn AS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AS, 0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ProjectionItemContextAttrs<'input> for ProjectionItemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn projectionItem(&mut self) -> Result<Rc<ProjectionItemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ProjectionItemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 28, RULE_projectionItem);
        let mut _localctx: Rc<ProjectionItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(291);
                recog.expression()?;

                recog.base.set_state(294);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_AS {
                    {
                        recog.base.set_state(292);
                        recog
                            .base
                            .match_token(CypherParser_AS, &mut recog.err_handler)?;

                        /*InvokeRule symbol*/
                        recog.base.set_state(293);
                        recog.symbol()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- orderItem ----------------
pub type OrderItemContextAll<'input> = OrderItemContext<'input>;

pub type OrderItemContext<'input> = BaseParserRuleContext<'input, OrderItemContextExt<'input>>;

#[derive(Clone)]
pub struct OrderItemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for OrderItemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for OrderItemContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_orderItem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_orderItem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for OrderItemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_orderItem(self);
    }
}

impl<'input> CustomRuleContext<'input> for OrderItemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_orderItem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_orderItem }
}
antlr4rust::tid! {OrderItemContextExt<'a>}

impl<'input> OrderItemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<OrderItemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            OrderItemContextExt { ph: PhantomData },
        ))
    }
}

pub trait OrderItemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<OrderItemContextExt<'input>>
{
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ASCENDING
    /// Returns `None` if there is no child corresponding to token ASCENDING
    fn ASCENDING(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASCENDING, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ASC
    /// Returns `None` if there is no child corresponding to token ASC
    fn ASC(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASC, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DESCENDING
    /// Returns `None` if there is no child corresponding to token DESCENDING
    fn DESCENDING(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DESCENDING, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DESC
    /// Returns `None` if there is no child corresponding to token DESC
    fn DESC(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DESC, 0)
    }
}

impl<'input> OrderItemContextAttrs<'input> for OrderItemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn orderItem(&mut self) -> Result<Rc<OrderItemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = OrderItemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 30, RULE_orderItem);
        let mut _localctx: Rc<OrderItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(296);
                recog.expression()?;

                recog.base.set_state(298);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 37) & !0x3f) == 0 && ((1usize << (_la - 37)) & 99) != 0) {
                    {
                        recog.base.set_state(297);
                        _la = recog.base.input.la(1);
                        if { !(((_la - 37) & !0x3f) == 0 && ((1usize << (_la - 37)) & 99) != 0) } {
                            recog.err_handler.recover_inline(&mut recog.base)?;
                        } else {
                            if recog.base.input.la(1) == TOKEN_EOF {
                                recog.base.matched_eof = true
                            };
                            recog.err_handler.report_match(&mut recog.base);
                            recog.base.consume(&mut recog.err_handler);
                        }
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- orderSt ----------------
pub type OrderStContextAll<'input> = OrderStContext<'input>;

pub type OrderStContext<'input> = BaseParserRuleContext<'input, OrderStContextExt<'input>>;

#[derive(Clone)]
pub struct OrderStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for OrderStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for OrderStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_orderSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_orderSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for OrderStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_orderSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for OrderStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_orderSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_orderSt }
}
antlr4rust::tid! {OrderStContextExt<'a>}

impl<'input> OrderStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<OrderStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            OrderStContextExt { ph: PhantomData },
        ))
    }
}

pub trait OrderStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<OrderStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ORDER
    /// Returns `None` if there is no child corresponding to token ORDER
    fn ORDER(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ORDER, 0)
    }
    /// Retrieves first TerminalNode corresponding to token BY
    /// Returns `None` if there is no child corresponding to token BY
    fn BY(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_BY, 0)
    }
    fn orderItem_all(&self) -> Vec<Rc<OrderItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn orderItem(&self, i: usize) -> Option<Rc<OrderItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> OrderStContextAttrs<'input> for OrderStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn orderSt(&mut self) -> Result<Rc<OrderStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = OrderStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 32, RULE_orderSt);
        let mut _localctx: Rc<OrderStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(300);
                recog
                    .base
                    .match_token(CypherParser_ORDER, &mut recog.err_handler)?;

                recog.base.set_state(301);
                recog
                    .base
                    .match_token(CypherParser_BY, &mut recog.err_handler)?;

                /*InvokeRule orderItem*/
                recog.base.set_state(302);
                recog.orderItem()?;

                recog.base.set_state(307);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(303);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule orderItem*/
                            recog.base.set_state(304);
                            recog.orderItem()?;
                        }
                    }
                    recog.base.set_state(309);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- singlePartQ ----------------
pub type SinglePartQContextAll<'input> = SinglePartQContext<'input>;

pub type SinglePartQContext<'input> = BaseParserRuleContext<'input, SinglePartQContextExt<'input>>;

#[derive(Clone)]
pub struct SinglePartQContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SinglePartQContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SinglePartQContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_singlePartQ(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_singlePartQ(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SinglePartQContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_singlePartQ(self);
    }
}

impl<'input> CustomRuleContext<'input> for SinglePartQContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_singlePartQ
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_singlePartQ }
}
antlr4rust::tid! {SinglePartQContextExt<'a>}

impl<'input> SinglePartQContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SinglePartQContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SinglePartQContextExt { ph: PhantomData },
        ))
    }
}

pub trait SinglePartQContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SinglePartQContextExt<'input>>
{
    fn returnSt(&self) -> Option<Rc<ReturnStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn readingStatement_all(&self) -> Vec<Rc<ReadingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn readingStatement(&self, i: usize) -> Option<Rc<ReadingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn updatingStatement_all(&self) -> Vec<Rc<UpdatingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn updatingStatement(&self, i: usize) -> Option<Rc<UpdatingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> SinglePartQContextAttrs<'input> for SinglePartQContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn singlePartQ(&mut self) -> Result<Rc<SinglePartQContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SinglePartQContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 34, RULE_singlePartQ);
        let mut _localctx: Rc<SinglePartQContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(313);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_CALL
                    || (((_la - 48) & !0x3f) == 0 && ((1usize << (_la - 48)) & 4105) != 0)
                {
                    {
                        {
                            /*InvokeRule readingStatement*/
                            recog.base.set_state(310);
                            recog.readingStatement()?;
                        }
                    }
                    recog.base.set_state(315);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(325);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_RETURN => {
                        {
                            /*InvokeRule returnSt*/
                            recog.base.set_state(316);
                            recog.returnSt()?;
                        }
                    }

                    CypherParser_CREATE | CypherParser_DELETE | CypherParser_DETACH
                    | CypherParser_MERGE | CypherParser_REMOVE | CypherParser_SET => {
                        {
                            recog.base.set_state(318);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            loop {
                                {
                                    {
                                        /*InvokeRule updatingStatement*/
                                        recog.base.set_state(317);
                                        recog.updatingStatement()?;
                                    }
                                }
                                recog.base.set_state(320);
                                recog.err_handler.sync(&mut recog.base)?;
                                _la = recog.base.input.la(1);
                                if !(((_la - 40) & !0x3f) == 0
                                    && ((1usize << (_la - 40)) & 41491) != 0)
                                {
                                    break;
                                }
                            }
                            recog.base.set_state(323);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            if _la == CypherParser_RETURN {
                                {
                                    /*InvokeRule returnSt*/
                                    recog.base.set_state(322);
                                    recog.returnSt()?;
                                }
                            }
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- multiPartQ ----------------
pub type MultiPartQContextAll<'input> = MultiPartQContext<'input>;

pub type MultiPartQContext<'input> = BaseParserRuleContext<'input, MultiPartQContextExt<'input>>;

#[derive(Clone)]
pub struct MultiPartQContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MultiPartQContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MultiPartQContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_multiPartQ(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_multiPartQ(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MultiPartQContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_multiPartQ(self);
    }
}

impl<'input> CustomRuleContext<'input> for MultiPartQContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_multiPartQ
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_multiPartQ }
}
antlr4rust::tid! {MultiPartQContextExt<'a>}

impl<'input> MultiPartQContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MultiPartQContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MultiPartQContextExt { ph: PhantomData },
        ))
    }
}

pub trait MultiPartQContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MultiPartQContextExt<'input>>
{
    fn singlePartQ(&self) -> Option<Rc<SinglePartQContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn readingStatement_all(&self) -> Vec<Rc<ReadingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn readingStatement(&self, i: usize) -> Option<Rc<ReadingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn withSt_all(&self) -> Vec<Rc<WithStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn withSt(&self, i: usize) -> Option<Rc<WithStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn updatingStatement_all(&self) -> Vec<Rc<UpdatingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn updatingStatement(&self, i: usize) -> Option<Rc<UpdatingStatementContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> MultiPartQContextAttrs<'input> for MultiPartQContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn multiPartQ(&mut self) -> Result<Rc<MultiPartQContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MultiPartQContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 36, RULE_multiPartQ);
        let mut _localctx: Rc<MultiPartQContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            let mut _alt: i32;
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(330);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = recog.interpreter.adaptive_predict(24, &mut recog.base)?;
                while { _alt != 2 && _alt != INVALID_ALT } {
                    if _alt == 1 {
                        {
                            {
                                /*InvokeRule readingStatement*/
                                recog.base.set_state(327);
                                recog.readingStatement()?;
                            }
                        }
                    }
                    recog.base.set_state(332);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(24, &mut recog.base)?;
                }
                recog.base.set_state(341);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = 1;
                loop {
                    match _alt {
                        x if x == 1 => {
                            {
                                recog.base.set_state(337);
                                recog.err_handler.sync(&mut recog.base)?;
                                _la = recog.base.input.la(1);
                                while _la == CypherParser_CALL
                                    || (((_la - 40) & !0x3f) == 0
                                        && ((1usize << (_la - 40)) & 1092371) != 0)
                                {
                                    {
                                        recog.base.set_state(335);
                                        recog.err_handler.sync(&mut recog.base)?;
                                        match recog.base.input.la(1) {
                                            CypherParser_CALL
                                            | CypherParser_MATCH
                                            | CypherParser_OPTIONAL
                                            | CypherParser_UNWIND => {
                                                {
                                                    /*InvokeRule readingStatement*/
                                                    recog.base.set_state(333);
                                                    recog.readingStatement()?;
                                                }
                                            }

                                            CypherParser_CREATE | CypherParser_DELETE
                                            | CypherParser_DETACH | CypherParser_MERGE
                                            | CypherParser_REMOVE | CypherParser_SET => {
                                                {
                                                    /*InvokeRule updatingStatement*/
                                                    recog.base.set_state(334);
                                                    recog.updatingStatement()?;
                                                }
                                            }

                                            _ => Err(ANTLRError::NoAltError(
                                                NoViableAltError::new(&mut recog.base),
                                            ))?,
                                        }
                                    }
                                    recog.base.set_state(339);
                                    recog.err_handler.sync(&mut recog.base)?;
                                    _la = recog.base.input.la(1);
                                }
                                /*InvokeRule withSt*/
                                recog.base.set_state(340);
                                recog.withSt()?;
                            }
                        }

                        _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                            &mut recog.base,
                        )))?,
                    }
                    recog.base.set_state(343);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(27, &mut recog.base)?;
                    if _alt == 2 || _alt == INVALID_ALT {
                        break;
                    }
                }
                /*InvokeRule singlePartQ*/
                recog.base.set_state(345);
                recog.singlePartQ()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- matchSt ----------------
pub type MatchStContextAll<'input> = MatchStContext<'input>;

pub type MatchStContext<'input> = BaseParserRuleContext<'input, MatchStContextExt<'input>>;

#[derive(Clone)]
pub struct MatchStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MatchStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MatchStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_matchSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_matchSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MatchStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_matchSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for MatchStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_matchSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_matchSt }
}
antlr4rust::tid! {MatchStContextExt<'a>}

impl<'input> MatchStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MatchStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MatchStContextExt { ph: PhantomData },
        ))
    }
}

pub trait MatchStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MatchStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token MATCH
    /// Returns `None` if there is no child corresponding to token MATCH
    fn MATCH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MATCH, 0)
    }
    fn patternWhere(&self) -> Option<Rc<PatternWhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token OPTIONAL
    /// Returns `None` if there is no child corresponding to token OPTIONAL
    fn OPTIONAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_OPTIONAL, 0)
    }
}

impl<'input> MatchStContextAttrs<'input> for MatchStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn matchSt(&mut self) -> Result<Rc<MatchStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MatchStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 38, RULE_matchSt);
        let mut _localctx: Rc<MatchStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(348);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_OPTIONAL {
                    {
                        recog.base.set_state(347);
                        recog
                            .base
                            .match_token(CypherParser_OPTIONAL, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(350);
                recog
                    .base
                    .match_token(CypherParser_MATCH, &mut recog.err_handler)?;

                /*InvokeRule patternWhere*/
                recog.base.set_state(351);
                recog.patternWhere()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- unwindSt ----------------
pub type UnwindStContextAll<'input> = UnwindStContext<'input>;

pub type UnwindStContext<'input> = BaseParserRuleContext<'input, UnwindStContextExt<'input>>;

#[derive(Clone)]
pub struct UnwindStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for UnwindStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for UnwindStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_unwindSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_unwindSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for UnwindStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_unwindSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for UnwindStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_unwindSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_unwindSt }
}
antlr4rust::tid! {UnwindStContextExt<'a>}

impl<'input> UnwindStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<UnwindStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            UnwindStContextExt { ph: PhantomData },
        ))
    }
}

pub trait UnwindStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<UnwindStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token UNWIND
    /// Returns `None` if there is no child corresponding to token UNWIND
    fn UNWIND(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNWIND, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token AS
    /// Returns `None` if there is no child corresponding to token AS
    fn AS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AS, 0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> UnwindStContextAttrs<'input> for UnwindStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn unwindSt(&mut self) -> Result<Rc<UnwindStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = UnwindStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 40, RULE_unwindSt);
        let mut _localctx: Rc<UnwindStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(353);
                recog
                    .base
                    .match_token(CypherParser_UNWIND, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(354);
                recog.expression()?;

                recog.base.set_state(355);
                recog
                    .base
                    .match_token(CypherParser_AS, &mut recog.err_handler)?;

                /*InvokeRule symbol*/
                recog.base.set_state(356);
                recog.symbol()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- readingStatement ----------------
pub type ReadingStatementContextAll<'input> = ReadingStatementContext<'input>;

pub type ReadingStatementContext<'input> =
    BaseParserRuleContext<'input, ReadingStatementContextExt<'input>>;

#[derive(Clone)]
pub struct ReadingStatementContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ReadingStatementContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ReadingStatementContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_readingStatement(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_readingStatement(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ReadingStatementContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_readingStatement(self);
    }
}

impl<'input> CustomRuleContext<'input> for ReadingStatementContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_readingStatement
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_readingStatement }
}
antlr4rust::tid! {ReadingStatementContextExt<'a>}

impl<'input> ReadingStatementContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ReadingStatementContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ReadingStatementContextExt { ph: PhantomData },
        ))
    }
}

pub trait ReadingStatementContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ReadingStatementContextExt<'input>>
{
    fn matchSt(&self) -> Option<Rc<MatchStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn unwindSt(&self) -> Option<Rc<UnwindStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn queryCallSt(&self) -> Option<Rc<QueryCallStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ReadingStatementContextAttrs<'input> for ReadingStatementContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn readingStatement(
        &mut self,
    ) -> Result<Rc<ReadingStatementContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ReadingStatementContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 42, RULE_readingStatement);
        let mut _localctx: Rc<ReadingStatementContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(361);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_MATCH | CypherParser_OPTIONAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule matchSt*/
                        recog.base.set_state(358);
                        recog.matchSt()?;
                    }
                }

                CypherParser_UNWIND => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule unwindSt*/
                        recog.base.set_state(359);
                        recog.unwindSt()?;
                    }
                }

                CypherParser_CALL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule queryCallSt*/
                        recog.base.set_state(360);
                        recog.queryCallSt()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- updatingStatement ----------------
pub type UpdatingStatementContextAll<'input> = UpdatingStatementContext<'input>;

pub type UpdatingStatementContext<'input> =
    BaseParserRuleContext<'input, UpdatingStatementContextExt<'input>>;

#[derive(Clone)]
pub struct UpdatingStatementContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for UpdatingStatementContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for UpdatingStatementContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_updatingStatement(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_updatingStatement(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for UpdatingStatementContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_updatingStatement(self);
    }
}

impl<'input> CustomRuleContext<'input> for UpdatingStatementContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_updatingStatement
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_updatingStatement }
}
antlr4rust::tid! {UpdatingStatementContextExt<'a>}

impl<'input> UpdatingStatementContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<UpdatingStatementContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            UpdatingStatementContextExt { ph: PhantomData },
        ))
    }
}

pub trait UpdatingStatementContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<UpdatingStatementContextExt<'input>>
{
    fn createSt(&self) -> Option<Rc<CreateStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn mergeSt(&self) -> Option<Rc<MergeStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn deleteSt(&self) -> Option<Rc<DeleteStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn setSt(&self) -> Option<Rc<SetStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn removeSt(&self) -> Option<Rc<RemoveStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> UpdatingStatementContextAttrs<'input> for UpdatingStatementContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn updatingStatement(
        &mut self,
    ) -> Result<Rc<UpdatingStatementContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            UpdatingStatementContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 44, RULE_updatingStatement);
        let mut _localctx: Rc<UpdatingStatementContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(368);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_CREATE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule createSt*/
                        recog.base.set_state(363);
                        recog.createSt()?;
                    }
                }

                CypherParser_MERGE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule mergeSt*/
                        recog.base.set_state(364);
                        recog.mergeSt()?;
                    }
                }

                CypherParser_DELETE | CypherParser_DETACH => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule deleteSt*/
                        recog.base.set_state(365);
                        recog.deleteSt()?;
                    }
                }

                CypherParser_SET => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule setSt*/
                        recog.base.set_state(366);
                        recog.setSt()?;
                    }
                }

                CypherParser_REMOVE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule removeSt*/
                        recog.base.set_state(367);
                        recog.removeSt()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- deleteSt ----------------
pub type DeleteStContextAll<'input> = DeleteStContext<'input>;

pub type DeleteStContext<'input> = BaseParserRuleContext<'input, DeleteStContextExt<'input>>;

#[derive(Clone)]
pub struct DeleteStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for DeleteStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for DeleteStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_deleteSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_deleteSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for DeleteStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_deleteSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for DeleteStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_deleteSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_deleteSt }
}
antlr4rust::tid! {DeleteStContextExt<'a>}

impl<'input> DeleteStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<DeleteStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            DeleteStContextExt { ph: PhantomData },
        ))
    }
}

pub trait DeleteStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<DeleteStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token DELETE
    /// Returns `None` if there is no child corresponding to token DELETE
    fn DELETE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DELETE, 0)
    }
    fn expressionChain(&self) -> Option<Rc<ExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token DETACH
    /// Returns `None` if there is no child corresponding to token DETACH
    fn DETACH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DETACH, 0)
    }
}

impl<'input> DeleteStContextAttrs<'input> for DeleteStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn deleteSt(&mut self) -> Result<Rc<DeleteStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = DeleteStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 46, RULE_deleteSt);
        let mut _localctx: Rc<DeleteStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(371);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DETACH {
                    {
                        recog.base.set_state(370);
                        recog
                            .base
                            .match_token(CypherParser_DETACH, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(373);
                recog
                    .base
                    .match_token(CypherParser_DELETE, &mut recog.err_handler)?;

                /*InvokeRule expressionChain*/
                recog.base.set_state(374);
                recog.expressionChain()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- removeSt ----------------
pub type RemoveStContextAll<'input> = RemoveStContext<'input>;

pub type RemoveStContext<'input> = BaseParserRuleContext<'input, RemoveStContextExt<'input>>;

#[derive(Clone)]
pub struct RemoveStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RemoveStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for RemoveStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_removeSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_removeSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for RemoveStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_removeSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for RemoveStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_removeSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_removeSt }
}
antlr4rust::tid! {RemoveStContextExt<'a>}

impl<'input> RemoveStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RemoveStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RemoveStContextExt { ph: PhantomData },
        ))
    }
}

pub trait RemoveStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RemoveStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token REMOVE
    /// Returns `None` if there is no child corresponding to token REMOVE
    fn REMOVE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_REMOVE, 0)
    }
    fn removeItem_all(&self) -> Vec<Rc<RemoveItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn removeItem(&self, i: usize) -> Option<Rc<RemoveItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> RemoveStContextAttrs<'input> for RemoveStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn removeSt(&mut self) -> Result<Rc<RemoveStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = RemoveStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 48, RULE_removeSt);
        let mut _localctx: Rc<RemoveStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(376);
                recog
                    .base
                    .match_token(CypherParser_REMOVE, &mut recog.err_handler)?;

                /*InvokeRule removeItem*/
                recog.base.set_state(377);
                recog.removeItem()?;

                recog.base.set_state(382);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(378);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule removeItem*/
                            recog.base.set_state(379);
                            recog.removeItem()?;
                        }
                    }
                    recog.base.set_state(384);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- removeItem ----------------
pub type RemoveItemContextAll<'input> = RemoveItemContext<'input>;

pub type RemoveItemContext<'input> = BaseParserRuleContext<'input, RemoveItemContextExt<'input>>;

#[derive(Clone)]
pub struct RemoveItemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RemoveItemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for RemoveItemContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_removeItem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_removeItem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for RemoveItemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_removeItem(self);
    }
}

impl<'input> CustomRuleContext<'input> for RemoveItemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_removeItem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_removeItem }
}
antlr4rust::tid! {RemoveItemContextExt<'a>}

impl<'input> RemoveItemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RemoveItemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RemoveItemContextExt { ph: PhantomData },
        ))
    }
}

pub trait RemoveItemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RemoveItemContextExt<'input>>
{
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn nodeLabels(&self) -> Option<Rc<NodeLabelsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn propertyExpression(&self) -> Option<Rc<PropertyExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> RemoveItemContextAttrs<'input> for RemoveItemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn removeItem(&mut self) -> Result<Rc<RemoveItemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = RemoveItemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 50, RULE_removeItem);
        let mut _localctx: Rc<RemoveItemContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(389);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(33, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(385);
                        recog.symbol()?;

                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(386);
                        recog.nodeLabels()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule propertyExpression*/
                        recog.base.set_state(388);
                        recog.propertyExpression()?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- queryCallSt ----------------
pub type QueryCallStContextAll<'input> = QueryCallStContext<'input>;

pub type QueryCallStContext<'input> = BaseParserRuleContext<'input, QueryCallStContextExt<'input>>;

#[derive(Clone)]
pub struct QueryCallStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QueryCallStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for QueryCallStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_queryCallSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_queryCallSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QueryCallStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_queryCallSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for QueryCallStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_queryCallSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_queryCallSt }
}
antlr4rust::tid! {QueryCallStContextExt<'a>}

impl<'input> QueryCallStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QueryCallStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QueryCallStContextExt { ph: PhantomData },
        ))
    }
}

pub trait QueryCallStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QueryCallStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CALL
    /// Returns `None` if there is no child corresponding to token CALL
    fn CALL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CALL, 0)
    }
    fn invocationName(&self) -> Option<Rc<InvocationNameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn parenExpressionChain(&self) -> Option<Rc<ParenExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token YIELD
    /// Returns `None` if there is no child corresponding to token YIELD
    fn YIELD(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_YIELD, 0)
    }
    fn yieldItems(&self) -> Option<Rc<YieldItemsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> QueryCallStContextAttrs<'input> for QueryCallStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn queryCallSt(&mut self) -> Result<Rc<QueryCallStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = QueryCallStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 52, RULE_queryCallSt);
        let mut _localctx: Rc<QueryCallStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(391);
                recog
                    .base
                    .match_token(CypherParser_CALL, &mut recog.err_handler)?;

                /*InvokeRule invocationName*/
                recog.base.set_state(392);
                recog.invocationName()?;

                /*InvokeRule parenExpressionChain*/
                recog.base.set_state(393);
                recog.parenExpressionChain()?;

                recog.base.set_state(396);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_YIELD {
                    {
                        recog.base.set_state(394);
                        recog
                            .base
                            .match_token(CypherParser_YIELD, &mut recog.err_handler)?;

                        /*InvokeRule yieldItems*/
                        recog.base.set_state(395);
                        recog.yieldItems()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- parenExpressionChain ----------------
pub type ParenExpressionChainContextAll<'input> = ParenExpressionChainContext<'input>;

pub type ParenExpressionChainContext<'input> =
    BaseParserRuleContext<'input, ParenExpressionChainContextExt<'input>>;

#[derive(Clone)]
pub struct ParenExpressionChainContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ParenExpressionChainContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ParenExpressionChainContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_parenExpressionChain(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_parenExpressionChain(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ParenExpressionChainContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_parenExpressionChain(self);
    }
}

impl<'input> CustomRuleContext<'input> for ParenExpressionChainContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_parenExpressionChain
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_parenExpressionChain }
}
antlr4rust::tid! {ParenExpressionChainContextExt<'a>}

impl<'input> ParenExpressionChainContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ParenExpressionChainContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ParenExpressionChainContextExt { ph: PhantomData },
        ))
    }
}

pub trait ParenExpressionChainContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ParenExpressionChainContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    fn expressionChain(&self) -> Option<Rc<ExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ParenExpressionChainContextAttrs<'input> for ParenExpressionChainContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn parenExpressionChain(
        &mut self,
    ) -> Result<Rc<ParenExpressionChainContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ParenExpressionChainContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 54, RULE_parenExpressionChain);
        let mut _localctx: Rc<ParenExpressionChainContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(398);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(400);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(399);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(402);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- yieldItems ----------------
pub type YieldItemsContextAll<'input> = YieldItemsContext<'input>;

pub type YieldItemsContext<'input> = BaseParserRuleContext<'input, YieldItemsContextExt<'input>>;

#[derive(Clone)]
pub struct YieldItemsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for YieldItemsContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for YieldItemsContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_yieldItems(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_yieldItems(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for YieldItemsContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_yieldItems(self);
    }
}

impl<'input> CustomRuleContext<'input> for YieldItemsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_yieldItems
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_yieldItems }
}
antlr4rust::tid! {YieldItemsContextExt<'a>}

impl<'input> YieldItemsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<YieldItemsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            YieldItemsContextExt { ph: PhantomData },
        ))
    }
}

pub trait YieldItemsContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<YieldItemsContextExt<'input>>
{
    fn yieldItem_all(&self) -> Vec<Rc<YieldItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn yieldItem(&self, i: usize) -> Option<Rc<YieldItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
    fn where_(&self) -> Option<Rc<WhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> YieldItemsContextAttrs<'input> for YieldItemsContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn yieldItems(&mut self) -> Result<Rc<YieldItemsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = YieldItemsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 56, RULE_yieldItems);
        let mut _localctx: Rc<YieldItemsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule yieldItem*/
                recog.base.set_state(404);
                recog.yieldItem()?;

                recog.base.set_state(409);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(405);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule yieldItem*/
                            recog.base.set_state(406);
                            recog.yieldItem()?;
                        }
                    }
                    recog.base.set_state(411);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(413);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(412);
                        recog.where_()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- yieldItem ----------------
pub type YieldItemContextAll<'input> = YieldItemContext<'input>;

pub type YieldItemContext<'input> = BaseParserRuleContext<'input, YieldItemContextExt<'input>>;

#[derive(Clone)]
pub struct YieldItemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for YieldItemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for YieldItemContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_yieldItem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_yieldItem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for YieldItemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_yieldItem(self);
    }
}

impl<'input> CustomRuleContext<'input> for YieldItemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_yieldItem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_yieldItem }
}
antlr4rust::tid! {YieldItemContextExt<'a>}

impl<'input> YieldItemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<YieldItemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            YieldItemContextExt { ph: PhantomData },
        ))
    }
}

pub trait YieldItemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<YieldItemContextExt<'input>>
{
    fn symbol_all(&self) -> Vec<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn symbol(&self, i: usize) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token AS
    /// Returns `None` if there is no child corresponding to token AS
    fn AS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AS, 0)
    }
}

impl<'input> YieldItemContextAttrs<'input> for YieldItemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn yieldItem(&mut self) -> Result<Rc<YieldItemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = YieldItemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 58, RULE_yieldItem);
        let mut _localctx: Rc<YieldItemContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(418);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.interpreter.adaptive_predict(38, &mut recog.base)? {
                    x if x == 1 => {
                        {
                            /*InvokeRule symbol*/
                            recog.base.set_state(415);
                            recog.symbol()?;

                            recog.base.set_state(416);
                            recog
                                .base
                                .match_token(CypherParser_AS, &mut recog.err_handler)?;
                        }
                    }

                    _ => {}
                }
                /*InvokeRule symbol*/
                recog.base.set_state(420);
                recog.symbol()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- mergeSt ----------------
pub type MergeStContextAll<'input> = MergeStContext<'input>;

pub type MergeStContext<'input> = BaseParserRuleContext<'input, MergeStContextExt<'input>>;

#[derive(Clone)]
pub struct MergeStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MergeStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MergeStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_mergeSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_mergeSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MergeStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_mergeSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for MergeStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_mergeSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_mergeSt }
}
antlr4rust::tid! {MergeStContextExt<'a>}

impl<'input> MergeStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MergeStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MergeStContextExt { ph: PhantomData },
        ))
    }
}

pub trait MergeStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MergeStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token MERGE
    /// Returns `None` if there is no child corresponding to token MERGE
    fn MERGE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MERGE, 0)
    }
    fn patternPart(&self) -> Option<Rc<PatternPartContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn mergeAction_all(&self) -> Vec<Rc<MergeActionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn mergeAction(&self, i: usize) -> Option<Rc<MergeActionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> MergeStContextAttrs<'input> for MergeStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn mergeSt(&mut self) -> Result<Rc<MergeStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MergeStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 60, RULE_mergeSt);
        let mut _localctx: Rc<MergeStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(422);
                recog
                    .base
                    .match_token(CypherParser_MERGE, &mut recog.err_handler)?;

                /*InvokeRule patternPart*/
                recog.base.set_state(423);
                recog.patternPart()?;

                recog.base.set_state(427);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_ON {
                    {
                        {
                            /*InvokeRule mergeAction*/
                            recog.base.set_state(424);
                            recog.mergeAction()?;
                        }
                    }
                    recog.base.set_state(429);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- mergeAction ----------------
pub type MergeActionContextAll<'input> = MergeActionContext<'input>;

pub type MergeActionContext<'input> = BaseParserRuleContext<'input, MergeActionContextExt<'input>>;

#[derive(Clone)]
pub struct MergeActionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MergeActionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MergeActionContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_mergeAction(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_mergeAction(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MergeActionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_mergeAction(self);
    }
}

impl<'input> CustomRuleContext<'input> for MergeActionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_mergeAction
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_mergeAction }
}
antlr4rust::tid! {MergeActionContextExt<'a>}

impl<'input> MergeActionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MergeActionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MergeActionContextExt { ph: PhantomData },
        ))
    }
}

pub trait MergeActionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MergeActionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ON
    /// Returns `None` if there is no child corresponding to token ON
    fn ON(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ON, 0)
    }
    fn setSt(&self) -> Option<Rc<SetStContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token MATCH
    /// Returns `None` if there is no child corresponding to token MATCH
    fn MATCH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MATCH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CREATE
    /// Returns `None` if there is no child corresponding to token CREATE
    fn CREATE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CREATE, 0)
    }
}

impl<'input> MergeActionContextAttrs<'input> for MergeActionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn mergeAction(&mut self) -> Result<Rc<MergeActionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MergeActionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 62, RULE_mergeAction);
        let mut _localctx: Rc<MergeActionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(430);
                recog
                    .base
                    .match_token(CypherParser_ON, &mut recog.err_handler)?;

                recog.base.set_state(431);
                _la = recog.base.input.la(1);
                if { !(_la == CypherParser_CREATE || _la == CypherParser_MATCH) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
                /*InvokeRule setSt*/
                recog.base.set_state(432);
                recog.setSt()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- setSt ----------------
pub type SetStContextAll<'input> = SetStContext<'input>;

pub type SetStContext<'input> = BaseParserRuleContext<'input, SetStContextExt<'input>>;

#[derive(Clone)]
pub struct SetStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SetStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SetStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_setSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_setSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SetStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_setSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for SetStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_setSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_setSt }
}
antlr4rust::tid! {SetStContextExt<'a>}

impl<'input> SetStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SetStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SetStContextExt { ph: PhantomData },
        ))
    }
}

pub trait SetStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SetStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token SET
    /// Returns `None` if there is no child corresponding to token SET
    fn SET(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SET, 0)
    }
    fn setItem_all(&self) -> Vec<Rc<SetItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn setItem(&self, i: usize) -> Option<Rc<SetItemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> SetStContextAttrs<'input> for SetStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn setSt(&mut self) -> Result<Rc<SetStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SetStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 64, RULE_setSt);
        let mut _localctx: Rc<SetStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(434);
                recog
                    .base
                    .match_token(CypherParser_SET, &mut recog.err_handler)?;

                /*InvokeRule setItem*/
                recog.base.set_state(435);
                recog.setItem()?;

                recog.base.set_state(440);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(436);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule setItem*/
                            recog.base.set_state(437);
                            recog.setItem()?;
                        }
                    }
                    recog.base.set_state(442);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- setItem ----------------
pub type SetItemContextAll<'input> = SetItemContext<'input>;

pub type SetItemContext<'input> = BaseParserRuleContext<'input, SetItemContextExt<'input>>;

#[derive(Clone)]
pub struct SetItemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SetItemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SetItemContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_setItem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_setItem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SetItemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_setItem(self);
    }
}

impl<'input> CustomRuleContext<'input> for SetItemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_setItem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_setItem }
}
antlr4rust::tid! {SetItemContextExt<'a>}

impl<'input> SetItemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SetItemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SetItemContextExt { ph: PhantomData },
        ))
    }
}

pub trait SetItemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SetItemContextExt<'input>>
{
    fn propertyExpression(&self) -> Option<Rc<PropertyExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ASSIGN
    /// Returns `None` if there is no child corresponding to token ASSIGN
    fn ASSIGN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASSIGN, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ADD_ASSIGN
    /// Returns `None` if there is no child corresponding to token ADD_ASSIGN
    fn ADD_ASSIGN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ADD_ASSIGN, 0)
    }
    fn nodeLabels(&self) -> Option<Rc<NodeLabelsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> SetItemContextAttrs<'input> for SetItemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn setItem(&mut self) -> Result<Rc<SetItemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SetItemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 66, RULE_setItem);
        let mut _localctx: Rc<SetItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(454);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(41, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule propertyExpression*/
                        recog.base.set_state(443);
                        recog.propertyExpression()?;

                        recog.base.set_state(444);
                        recog
                            .base
                            .match_token(CypherParser_ASSIGN, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(445);
                        recog.expression()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(447);
                        recog.symbol()?;

                        recog.base.set_state(448);
                        _la = recog.base.input.la(1);
                        if { !(_la == CypherParser_ASSIGN || _la == CypherParser_ADD_ASSIGN) } {
                            recog.err_handler.recover_inline(&mut recog.base)?;
                        } else {
                            if recog.base.input.la(1) == TOKEN_EOF {
                                recog.base.matched_eof = true
                            };
                            recog.err_handler.report_match(&mut recog.base);
                            recog.base.consume(&mut recog.err_handler);
                        }
                        /*InvokeRule expression*/
                        recog.base.set_state(449);
                        recog.expression()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(451);
                        recog.symbol()?;

                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(452);
                        recog.nodeLabels()?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- nodeLabels ----------------
pub type NodeLabelsContextAll<'input> = NodeLabelsContext<'input>;

pub type NodeLabelsContext<'input> = BaseParserRuleContext<'input, NodeLabelsContextExt<'input>>;

#[derive(Clone)]
pub struct NodeLabelsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NodeLabelsContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for NodeLabelsContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_nodeLabels(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_nodeLabels(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NodeLabelsContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_nodeLabels(self);
    }
}

impl<'input> CustomRuleContext<'input> for NodeLabelsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_nodeLabels
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_nodeLabels }
}
antlr4rust::tid! {NodeLabelsContextExt<'a>}

impl<'input> NodeLabelsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NodeLabelsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NodeLabelsContextExt { ph: PhantomData },
        ))
    }
}

pub trait NodeLabelsContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NodeLabelsContextExt<'input>>
{
    /// Retrieves all `TerminalNode`s corresponding to token COLON in current rule
    fn COLON_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COLON, starting from 0.
    /// Returns `None` if number of children corresponding to token COLON is less or equal than `i`.
    fn COLON(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COLON, i)
    }
    fn name_all(&self) -> Vec<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn name(&self, i: usize) -> Option<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> NodeLabelsContextAttrs<'input> for NodeLabelsContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn nodeLabels(&mut self) -> Result<Rc<NodeLabelsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = NodeLabelsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 68, RULE_nodeLabels);
        let mut _localctx: Rc<NodeLabelsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(458);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                loop {
                    {
                        {
                            recog.base.set_state(456);
                            recog
                                .base
                                .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                            /*InvokeRule name*/
                            recog.base.set_state(457);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(460);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                    if !(_la == CypherParser_COLON) {
                        break;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- createSt ----------------
pub type CreateStContextAll<'input> = CreateStContext<'input>;

pub type CreateStContext<'input> = BaseParserRuleContext<'input, CreateStContextExt<'input>>;

#[derive(Clone)]
pub struct CreateStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for CreateStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for CreateStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_createSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_createSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for CreateStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_createSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for CreateStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_createSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_createSt }
}
antlr4rust::tid! {CreateStContextExt<'a>}

impl<'input> CreateStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CreateStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CreateStContextExt { ph: PhantomData },
        ))
    }
}

pub trait CreateStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<CreateStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CREATE
    /// Returns `None` if there is no child corresponding to token CREATE
    fn CREATE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CREATE, 0)
    }
    fn pattern(&self) -> Option<Rc<PatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> CreateStContextAttrs<'input> for CreateStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn createSt(&mut self) -> Result<Rc<CreateStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = CreateStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 70, RULE_createSt);
        let mut _localctx: Rc<CreateStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(462);
                recog
                    .base
                    .match_token(CypherParser_CREATE, &mut recog.err_handler)?;

                /*InvokeRule pattern*/
                recog.base.set_state(463);
                recog.pattern()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- patternWhere ----------------
pub type PatternWhereContextAll<'input> = PatternWhereContext<'input>;

pub type PatternWhereContext<'input> =
    BaseParserRuleContext<'input, PatternWhereContextExt<'input>>;

#[derive(Clone)]
pub struct PatternWhereContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternWhereContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for PatternWhereContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_patternWhere(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_patternWhere(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for PatternWhereContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_patternWhere(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternWhereContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_patternWhere
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_patternWhere }
}
antlr4rust::tid! {PatternWhereContextExt<'a>}

impl<'input> PatternWhereContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternWhereContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternWhereContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternWhereContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternWhereContextExt<'input>>
{
    fn pattern(&self) -> Option<Rc<PatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn where_(&self) -> Option<Rc<WhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PatternWhereContextAttrs<'input> for PatternWhereContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn patternWhere(&mut self) -> Result<Rc<PatternWhereContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PatternWhereContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 72, RULE_patternWhere);
        let mut _localctx: Rc<PatternWhereContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule pattern*/
                recog.base.set_state(465);
                recog.pattern()?;

                recog.base.set_state(467);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(466);
                        recog.where_()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- where ----------------
pub type WhereContextAll<'input> = WhereContext<'input>;

pub type WhereContext<'input> = BaseParserRuleContext<'input, WhereContextExt<'input>>;

#[derive(Clone)]
pub struct WhereContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for WhereContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for WhereContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_where(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_where(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for WhereContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_where(self);
    }
}

impl<'input> CustomRuleContext<'input> for WhereContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_where
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_where }
}
antlr4rust::tid! {WhereContextExt<'a>}

impl<'input> WhereContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<WhereContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            WhereContextExt { ph: PhantomData },
        ))
    }
}

pub trait WhereContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<WhereContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token WHERE
    /// Returns `None` if there is no child corresponding to token WHERE
    fn WHERE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WHERE, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> WhereContextAttrs<'input> for WhereContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn where_(&mut self) -> Result<Rc<WhereContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = WhereContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 74, RULE_where);
        let mut _localctx: Rc<WhereContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(469);
                recog
                    .base
                    .match_token(CypherParser_WHERE, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(470);
                recog.expression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- pattern ----------------
pub type PatternContextAll<'input> = PatternContext<'input>;

pub type PatternContext<'input> = BaseParserRuleContext<'input, PatternContextExt<'input>>;

#[derive(Clone)]
pub struct PatternContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for PatternContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_pattern(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_pattern(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for PatternContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_pattern(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_pattern
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_pattern }
}
antlr4rust::tid! {PatternContextExt<'a>}

impl<'input> PatternContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternContextExt<'input>>
{
    fn patternPart_all(&self) -> Vec<Rc<PatternPartContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn patternPart(&self, i: usize) -> Option<Rc<PatternPartContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> PatternContextAttrs<'input> for PatternContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn pattern(&mut self) -> Result<Rc<PatternContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PatternContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 76, RULE_pattern);
        let mut _localctx: Rc<PatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule patternPart*/
                recog.base.set_state(472);
                recog.patternPart()?;

                recog.base.set_state(477);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(473);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule patternPart*/
                            recog.base.set_state(474);
                            recog.patternPart()?;
                        }
                    }
                    recog.base.set_state(479);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- expression ----------------
pub type ExpressionContextAll<'input> = ExpressionContext<'input>;

pub type ExpressionContext<'input> = BaseParserRuleContext<'input, ExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ExpressionContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_expression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_expression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_expression(self);
    }
}

impl<'input> CustomRuleContext<'input> for ExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_expression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_expression }
}
antlr4rust::tid! {ExpressionContextExt<'a>}

impl<'input> ExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ExpressionContextExt<'input>>
{
    fn xorExpression_all(&self) -> Vec<Rc<XorExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn xorExpression(&self, i: usize) -> Option<Rc<XorExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token OR in current rule
    fn OR_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token OR, starting from 0.
    /// Returns `None` if number of children corresponding to token OR is less or equal than `i`.
    fn OR(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_OR, i)
    }
}

impl<'input> ExpressionContextAttrs<'input> for ExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn expression(&mut self) -> Result<Rc<ExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 78, RULE_expression);
        let mut _localctx: Rc<ExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule xorExpression*/
                recog.base.set_state(480);
                recog.xorExpression()?;

                recog.base.set_state(485);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_OR {
                    {
                        {
                            recog.base.set_state(481);
                            recog
                                .base
                                .match_token(CypherParser_OR, &mut recog.err_handler)?;

                            /*InvokeRule xorExpression*/
                            recog.base.set_state(482);
                            recog.xorExpression()?;
                        }
                    }
                    recog.base.set_state(487);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- xorExpression ----------------
pub type XorExpressionContextAll<'input> = XorExpressionContext<'input>;

pub type XorExpressionContext<'input> =
    BaseParserRuleContext<'input, XorExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct XorExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for XorExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for XorExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_xorExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_xorExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for XorExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_xorExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for XorExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_xorExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_xorExpression }
}
antlr4rust::tid! {XorExpressionContextExt<'a>}

impl<'input> XorExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<XorExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            XorExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait XorExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<XorExpressionContextExt<'input>>
{
    fn andExpression_all(&self) -> Vec<Rc<AndExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn andExpression(&self, i: usize) -> Option<Rc<AndExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token XOR in current rule
    fn XOR_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token XOR, starting from 0.
    /// Returns `None` if number of children corresponding to token XOR is less or equal than `i`.
    fn XOR(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_XOR, i)
    }
}

impl<'input> XorExpressionContextAttrs<'input> for XorExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn xorExpression(&mut self) -> Result<Rc<XorExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            XorExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 80, RULE_xorExpression);
        let mut _localctx: Rc<XorExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule andExpression*/
                recog.base.set_state(488);
                recog.andExpression()?;

                recog.base.set_state(493);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_XOR {
                    {
                        {
                            recog.base.set_state(489);
                            recog
                                .base
                                .match_token(CypherParser_XOR, &mut recog.err_handler)?;

                            /*InvokeRule andExpression*/
                            recog.base.set_state(490);
                            recog.andExpression()?;
                        }
                    }
                    recog.base.set_state(495);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- andExpression ----------------
pub type AndExpressionContextAll<'input> = AndExpressionContext<'input>;

pub type AndExpressionContext<'input> =
    BaseParserRuleContext<'input, AndExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct AndExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for AndExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for AndExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_andExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_andExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for AndExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_andExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for AndExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_andExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_andExpression }
}
antlr4rust::tid! {AndExpressionContextExt<'a>}

impl<'input> AndExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<AndExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            AndExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait AndExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<AndExpressionContextExt<'input>>
{
    fn notExpression_all(&self) -> Vec<Rc<NotExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn notExpression(&self, i: usize) -> Option<Rc<NotExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token AND in current rule
    fn AND_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token AND, starting from 0.
    /// Returns `None` if number of children corresponding to token AND is less or equal than `i`.
    fn AND(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AND, i)
    }
}

impl<'input> AndExpressionContextAttrs<'input> for AndExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn andExpression(&mut self) -> Result<Rc<AndExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            AndExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 82, RULE_andExpression);
        let mut _localctx: Rc<AndExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule notExpression*/
                recog.base.set_state(496);
                recog.notExpression()?;

                recog.base.set_state(501);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_AND {
                    {
                        {
                            recog.base.set_state(497);
                            recog
                                .base
                                .match_token(CypherParser_AND, &mut recog.err_handler)?;

                            /*InvokeRule notExpression*/
                            recog.base.set_state(498);
                            recog.notExpression()?;
                        }
                    }
                    recog.base.set_state(503);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- notExpression ----------------
pub type NotExpressionContextAll<'input> = NotExpressionContext<'input>;

pub type NotExpressionContext<'input> =
    BaseParserRuleContext<'input, NotExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct NotExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NotExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for NotExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_notExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_notExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NotExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_notExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for NotExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_notExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_notExpression }
}
antlr4rust::tid! {NotExpressionContextExt<'a>}

impl<'input> NotExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NotExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NotExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait NotExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NotExpressionContextExt<'input>>
{
    fn comparisonExpression(&self) -> Option<Rc<ComparisonExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves all `TerminalNode`s corresponding to token NOT in current rule
    fn NOT_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token NOT, starting from 0.
    /// Returns `None` if number of children corresponding to token NOT is less or equal than `i`.
    fn NOT(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NOT, i)
    }
}

impl<'input> NotExpressionContextAttrs<'input> for NotExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn notExpression(&mut self) -> Result<Rc<NotExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            NotExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 84, RULE_notExpression);
        let mut _localctx: Rc<NotExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(507);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_NOT {
                    {
                        {
                            recog.base.set_state(504);
                            recog
                                .base
                                .match_token(CypherParser_NOT, &mut recog.err_handler)?;
                        }
                    }
                    recog.base.set_state(509);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                /*InvokeRule comparisonExpression*/
                recog.base.set_state(510);
                recog.comparisonExpression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- comparisonExpression ----------------
pub type ComparisonExpressionContextAll<'input> = ComparisonExpressionContext<'input>;

pub type ComparisonExpressionContext<'input> =
    BaseParserRuleContext<'input, ComparisonExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ComparisonExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ComparisonExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ComparisonExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_comparisonExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_comparisonExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ComparisonExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_comparisonExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for ComparisonExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_comparisonExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_comparisonExpression }
}
antlr4rust::tid! {ComparisonExpressionContextExt<'a>}

impl<'input> ComparisonExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ComparisonExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ComparisonExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ComparisonExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ComparisonExpressionContextExt<'input>>
{
    fn addSubExpression_all(&self) -> Vec<Rc<AddSubExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn addSubExpression(&self, i: usize) -> Option<Rc<AddSubExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn comparisonSigns_all(&self) -> Vec<Rc<ComparisonSignsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn comparisonSigns(&self, i: usize) -> Option<Rc<ComparisonSignsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> ComparisonExpressionContextAttrs<'input> for ComparisonExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn comparisonExpression(
        &mut self,
    ) -> Result<Rc<ComparisonExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ComparisonExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 86, RULE_comparisonExpression);
        let mut _localctx: Rc<ComparisonExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule addSubExpression*/
                recog.base.set_state(512);
                recog.addSubExpression()?;

                recog.base.set_state(518);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la) & !0x3f) == 0 && ((1usize << _la) & 250) != 0) {
                    {
                        {
                            /*InvokeRule comparisonSigns*/
                            recog.base.set_state(513);
                            recog.comparisonSigns()?;

                            /*InvokeRule addSubExpression*/
                            recog.base.set_state(514);
                            recog.addSubExpression()?;
                        }
                    }
                    recog.base.set_state(520);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- comparisonSigns ----------------
pub type ComparisonSignsContextAll<'input> = ComparisonSignsContext<'input>;

pub type ComparisonSignsContext<'input> =
    BaseParserRuleContext<'input, ComparisonSignsContextExt<'input>>;

#[derive(Clone)]
pub struct ComparisonSignsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ComparisonSignsContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ComparisonSignsContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_comparisonSigns(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_comparisonSigns(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ComparisonSignsContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_comparisonSigns(self);
    }
}

impl<'input> CustomRuleContext<'input> for ComparisonSignsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_comparisonSigns
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_comparisonSigns }
}
antlr4rust::tid! {ComparisonSignsContextExt<'a>}

impl<'input> ComparisonSignsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ComparisonSignsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ComparisonSignsContextExt { ph: PhantomData },
        ))
    }
}

pub trait ComparisonSignsContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ComparisonSignsContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ASSIGN
    /// Returns `None` if there is no child corresponding to token ASSIGN
    fn ASSIGN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASSIGN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LE
    /// Returns `None` if there is no child corresponding to token LE
    fn LE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token GE
    /// Returns `None` if there is no child corresponding to token GE
    fn GE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_GE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token GT
    /// Returns `None` if there is no child corresponding to token GT
    fn GT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_GT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LT
    /// Returns `None` if there is no child corresponding to token LT
    fn LT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NOT_EQUAL
    /// Returns `None` if there is no child corresponding to token NOT_EQUAL
    fn NOT_EQUAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NOT_EQUAL, 0)
    }
}

impl<'input> ComparisonSignsContextAttrs<'input> for ComparisonSignsContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn comparisonSigns(&mut self) -> Result<Rc<ComparisonSignsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ComparisonSignsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 88, RULE_comparisonSigns);
        let mut _localctx: Rc<ComparisonSignsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(521);
                _la = recog.base.input.la(1);
                if { !(((_la) & !0x3f) == 0 && ((1usize << _la) & 250) != 0) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- addSubExpression ----------------
pub type AddSubExpressionContextAll<'input> = AddSubExpressionContext<'input>;

pub type AddSubExpressionContext<'input> =
    BaseParserRuleContext<'input, AddSubExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct AddSubExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for AddSubExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for AddSubExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_addSubExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_addSubExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for AddSubExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_addSubExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for AddSubExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_addSubExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_addSubExpression }
}
antlr4rust::tid! {AddSubExpressionContextExt<'a>}

impl<'input> AddSubExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<AddSubExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            AddSubExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait AddSubExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<AddSubExpressionContextExt<'input>>
{
    fn multDivExpression_all(&self) -> Vec<Rc<MultDivExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn multDivExpression(&self, i: usize) -> Option<Rc<MultDivExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token PLUS in current rule
    fn PLUS_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token PLUS, starting from 0.
    /// Returns `None` if number of children corresponding to token PLUS is less or equal than `i`.
    fn PLUS(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_PLUS, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SUB in current rule
    fn SUB_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SUB, starting from 0.
    /// Returns `None` if number of children corresponding to token SUB is less or equal than `i`.
    fn SUB(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SUB, i)
    }
}

impl<'input> AddSubExpressionContextAttrs<'input> for AddSubExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn addSubExpression(
        &mut self,
    ) -> Result<Rc<AddSubExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            AddSubExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 90, RULE_addSubExpression);
        let mut _localctx: Rc<AddSubExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule multDivExpression*/
                recog.base.set_state(523);
                recog.multDivExpression()?;

                recog.base.set_state(528);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_SUB || _la == CypherParser_PLUS {
                    {
                        {
                            recog.base.set_state(524);
                            _la = recog.base.input.la(1);
                            if { !(_la == CypherParser_SUB || _la == CypherParser_PLUS) } {
                                recog.err_handler.recover_inline(&mut recog.base)?;
                            } else {
                                if recog.base.input.la(1) == TOKEN_EOF {
                                    recog.base.matched_eof = true
                                };
                                recog.err_handler.report_match(&mut recog.base);
                                recog.base.consume(&mut recog.err_handler);
                            }
                            /*InvokeRule multDivExpression*/
                            recog.base.set_state(525);
                            recog.multDivExpression()?;
                        }
                    }
                    recog.base.set_state(530);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- multDivExpression ----------------
pub type MultDivExpressionContextAll<'input> = MultDivExpressionContext<'input>;

pub type MultDivExpressionContext<'input> =
    BaseParserRuleContext<'input, MultDivExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct MultDivExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MultDivExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for MultDivExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_multDivExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_multDivExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for MultDivExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_multDivExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for MultDivExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_multDivExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_multDivExpression }
}
antlr4rust::tid! {MultDivExpressionContextExt<'a>}

impl<'input> MultDivExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MultDivExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MultDivExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait MultDivExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MultDivExpressionContextExt<'input>>
{
    fn powerExpression_all(&self) -> Vec<Rc<PowerExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn powerExpression(&self, i: usize) -> Option<Rc<PowerExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token MULT in current rule
    fn MULT_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token MULT, starting from 0.
    /// Returns `None` if number of children corresponding to token MULT is less or equal than `i`.
    fn MULT(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token DIV in current rule
    fn DIV_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token DIV, starting from 0.
    /// Returns `None` if number of children corresponding to token DIV is less or equal than `i`.
    fn DIV(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DIV, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token MOD in current rule
    fn MOD_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token MOD, starting from 0.
    /// Returns `None` if number of children corresponding to token MOD is less or equal than `i`.
    fn MOD(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MOD, i)
    }
}

impl<'input> MultDivExpressionContextAttrs<'input> for MultDivExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn multDivExpression(
        &mut self,
    ) -> Result<Rc<MultDivExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            MultDivExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 92, RULE_multDivExpression);
        let mut _localctx: Rc<MultDivExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule powerExpression*/
                recog.base.set_state(531);
                recog.powerExpression()?;

                recog.base.set_state(536);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la) & !0x3f) == 0 && ((1usize << _la) & 11534336) != 0) {
                    {
                        {
                            recog.base.set_state(532);
                            _la = recog.base.input.la(1);
                            if { !(((_la) & !0x3f) == 0 && ((1usize << _la) & 11534336) != 0) } {
                                recog.err_handler.recover_inline(&mut recog.base)?;
                            } else {
                                if recog.base.input.la(1) == TOKEN_EOF {
                                    recog.base.matched_eof = true
                                };
                                recog.err_handler.report_match(&mut recog.base);
                                recog.base.consume(&mut recog.err_handler);
                            }
                            /*InvokeRule powerExpression*/
                            recog.base.set_state(533);
                            recog.powerExpression()?;
                        }
                    }
                    recog.base.set_state(538);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- powerExpression ----------------
pub type PowerExpressionContextAll<'input> = PowerExpressionContext<'input>;

pub type PowerExpressionContext<'input> =
    BaseParserRuleContext<'input, PowerExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct PowerExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PowerExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for PowerExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_powerExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_powerExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for PowerExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_powerExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for PowerExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_powerExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_powerExpression }
}
antlr4rust::tid! {PowerExpressionContextExt<'a>}

impl<'input> PowerExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PowerExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PowerExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait PowerExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PowerExpressionContextExt<'input>>
{
    fn unaryAddSubExpression_all(&self) -> Vec<Rc<UnaryAddSubExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn unaryAddSubExpression(&self, i: usize) -> Option<Rc<UnaryAddSubExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token CARET in current rule
    fn CARET_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token CARET, starting from 0.
    /// Returns `None` if number of children corresponding to token CARET is less or equal than `i`.
    fn CARET(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CARET, i)
    }
}

impl<'input> PowerExpressionContextAttrs<'input> for PowerExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn powerExpression(&mut self) -> Result<Rc<PowerExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            PowerExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 94, RULE_powerExpression);
        let mut _localctx: Rc<PowerExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule unaryAddSubExpression*/
                recog.base.set_state(539);
                recog.unaryAddSubExpression()?;

                recog.base.set_state(544);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_CARET {
                    {
                        {
                            recog.base.set_state(540);
                            recog
                                .base
                                .match_token(CypherParser_CARET, &mut recog.err_handler)?;

                            /*InvokeRule unaryAddSubExpression*/
                            recog.base.set_state(541);
                            recog.unaryAddSubExpression()?;
                        }
                    }
                    recog.base.set_state(546);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- unaryAddSubExpression ----------------
pub type UnaryAddSubExpressionContextAll<'input> = UnaryAddSubExpressionContext<'input>;

pub type UnaryAddSubExpressionContext<'input> =
    BaseParserRuleContext<'input, UnaryAddSubExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct UnaryAddSubExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for UnaryAddSubExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for UnaryAddSubExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_unaryAddSubExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_unaryAddSubExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for UnaryAddSubExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_unaryAddSubExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for UnaryAddSubExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_unaryAddSubExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_unaryAddSubExpression }
}
antlr4rust::tid! {UnaryAddSubExpressionContextExt<'a>}

impl<'input> UnaryAddSubExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<UnaryAddSubExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            UnaryAddSubExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait UnaryAddSubExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<UnaryAddSubExpressionContextExt<'input>>
{
    fn atomicExpression(&self) -> Option<Rc<AtomicExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token PLUS
    /// Returns `None` if there is no child corresponding to token PLUS
    fn PLUS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_PLUS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SUB
    /// Returns `None` if there is no child corresponding to token SUB
    fn SUB(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SUB, 0)
    }
}

impl<'input> UnaryAddSubExpressionContextAttrs<'input> for UnaryAddSubExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn unaryAddSubExpression(
        &mut self,
    ) -> Result<Rc<UnaryAddSubExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            UnaryAddSubExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 96, RULE_unaryAddSubExpression);
        let mut _localctx: Rc<UnaryAddSubExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(548);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SUB || _la == CypherParser_PLUS {
                    {
                        recog.base.set_state(547);
                        _la = recog.base.input.la(1);
                        if { !(_la == CypherParser_SUB || _la == CypherParser_PLUS) } {
                            recog.err_handler.recover_inline(&mut recog.base)?;
                        } else {
                            if recog.base.input.la(1) == TOKEN_EOF {
                                recog.base.matched_eof = true
                            };
                            recog.err_handler.report_match(&mut recog.base);
                            recog.base.consume(&mut recog.err_handler);
                        }
                    }
                }

                /*InvokeRule atomicExpression*/
                recog.base.set_state(550);
                recog.atomicExpression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- atomicExpression ----------------
pub type AtomicExpressionContextAll<'input> = AtomicExpressionContext<'input>;

pub type AtomicExpressionContext<'input> =
    BaseParserRuleContext<'input, AtomicExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct AtomicExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for AtomicExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for AtomicExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_atomicExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_atomicExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for AtomicExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_atomicExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for AtomicExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_atomicExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_atomicExpression }
}
antlr4rust::tid! {AtomicExpressionContextExt<'a>}

impl<'input> AtomicExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<AtomicExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            AtomicExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait AtomicExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<AtomicExpressionContextExt<'input>>
{
    fn propertyOrLabelExpression(&self) -> Option<Rc<PropertyOrLabelExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn stringExpression_all(&self) -> Vec<Rc<StringExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn stringExpression(&self, i: usize) -> Option<Rc<StringExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn listExpression_all(&self) -> Vec<Rc<ListExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn listExpression(&self, i: usize) -> Option<Rc<ListExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn nullExpression_all(&self) -> Vec<Rc<NullExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn nullExpression(&self, i: usize) -> Option<Rc<NullExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> AtomicExpressionContextAttrs<'input> for AtomicExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn atomicExpression(
        &mut self,
    ) -> Result<Rc<AtomicExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            AtomicExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 98, RULE_atomicExpression);
        let mut _localctx: Rc<AtomicExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule propertyOrLabelExpression*/
                recog.base.set_state(552);
                recog.propertyOrLabelExpression()?;

                recog.base.set_state(558);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_LBRACK
                    || (((_la - 63) & !0x3f) == 0 && ((1usize << (_la - 63)) & 301) != 0)
                {
                    {
                        recog.base.set_state(556);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.base.input.la(1) {
                            CypherParser_CONTAINS | CypherParser_ENDS | CypherParser_STARTS => {
                                {
                                    /*InvokeRule stringExpression*/
                                    recog.base.set_state(553);
                                    recog.stringExpression()?;
                                }
                            }

                            CypherParser_LBRACK | CypherParser_IN => {
                                {
                                    /*InvokeRule listExpression*/
                                    recog.base.set_state(554);
                                    recog.listExpression()?;
                                }
                            }

                            CypherParser_IS => {
                                {
                                    /*InvokeRule nullExpression*/
                                    recog.base.set_state(555);
                                    recog.nullExpression()?;
                                }
                            }

                            _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                                &mut recog.base,
                            )))?,
                        }
                    }
                    recog.base.set_state(560);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- listExpression ----------------
pub type ListExpressionContextAll<'input> = ListExpressionContext<'input>;

pub type ListExpressionContext<'input> =
    BaseParserRuleContext<'input, ListExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ListExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ListExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ListExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_listExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_listExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ListExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_listExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for ListExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_listExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_listExpression }
}
antlr4rust::tid! {ListExpressionContextExt<'a>}

impl<'input> ListExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ListExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ListExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ListExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ListExpressionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token IN
    /// Returns `None` if there is no child corresponding to token IN
    fn IN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_IN, 0)
    }
    fn propertyOrLabelExpression(&self) -> Option<Rc<PropertyOrLabelExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RANGE
    /// Returns `None` if there is no child corresponding to token RANGE
    fn RANGE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RANGE, 0)
    }
    fn expression_all(&self) -> Vec<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn expression(&self, i: usize) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> ListExpressionContextAttrs<'input> for ListExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn listExpression(&mut self) -> Result<Rc<ListExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ListExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 100, RULE_listExpression);
        let mut _localctx: Rc<ListExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(575);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_IN => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(561);
                        recog
                            .base
                            .match_token(CypherParser_IN, &mut recog.err_handler)?;

                        /*InvokeRule propertyOrLabelExpression*/
                        recog.base.set_state(562);
                        recog.propertyOrLabelExpression()?;
                    }
                }

                CypherParser_LBRACK => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(563);
                        recog
                            .base
                            .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                        recog.base.set_state(572);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.interpreter.adaptive_predict(58, &mut recog.base)? {
                            1 => {
                                {
                                    recog.base.set_state(565);
                                    recog.err_handler.sync(&mut recog.base)?;
                                    _la = recog.base.input.la(1);
                                    if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                                        || (((_la - 32) & !0x3f) == 0
                                            && ((1usize << (_la - 32)) & 8223) != 0)
                                        || (((_la - 69) & !0x3f) == 0
                                            && ((1usize << (_la - 69)) & 260055265) != 0)
                                    {
                                        {
                                            /*InvokeRule expression*/
                                            recog.base.set_state(564);
                                            recog.expression()?;
                                        }
                                    }

                                    recog.base.set_state(567);
                                    recog
                                        .base
                                        .match_token(CypherParser_RANGE, &mut recog.err_handler)?;

                                    recog.base.set_state(569);
                                    recog.err_handler.sync(&mut recog.base)?;
                                    _la = recog.base.input.la(1);
                                    if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                                        || (((_la - 32) & !0x3f) == 0
                                            && ((1usize << (_la - 32)) & 8223) != 0)
                                        || (((_la - 69) & !0x3f) == 0
                                            && ((1usize << (_la - 69)) & 260055265) != 0)
                                    {
                                        {
                                            /*InvokeRule expression*/
                                            recog.base.set_state(568);
                                            recog.expression()?;
                                        }
                                    }
                                }
                            }
                            2 => {
                                {
                                    /*InvokeRule expression*/
                                    recog.base.set_state(571);
                                    recog.expression()?;
                                }
                            }

                            _ => {}
                        }
                        recog.base.set_state(574);
                        recog
                            .base
                            .match_token(CypherParser_RBRACK, &mut recog.err_handler)?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- stringExpression ----------------
pub type StringExpressionContextAll<'input> = StringExpressionContext<'input>;

pub type StringExpressionContext<'input> =
    BaseParserRuleContext<'input, StringExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct StringExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for StringExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for StringExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_stringExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_stringExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for StringExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_stringExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for StringExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_stringExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_stringExpression }
}
antlr4rust::tid! {StringExpressionContextExt<'a>}

impl<'input> StringExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<StringExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            StringExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait StringExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<StringExpressionContextExt<'input>>
{
    fn stringExpPrefix(&self) -> Option<Rc<StringExpPrefixContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn propertyOrLabelExpression(&self) -> Option<Rc<PropertyOrLabelExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> StringExpressionContextAttrs<'input> for StringExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn stringExpression(
        &mut self,
    ) -> Result<Rc<StringExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            StringExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 102, RULE_stringExpression);
        let mut _localctx: Rc<StringExpressionContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule stringExpPrefix*/
                recog.base.set_state(577);
                recog.stringExpPrefix()?;

                /*InvokeRule propertyOrLabelExpression*/
                recog.base.set_state(578);
                recog.propertyOrLabelExpression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- stringExpPrefix ----------------
pub type StringExpPrefixContextAll<'input> = StringExpPrefixContext<'input>;

pub type StringExpPrefixContext<'input> =
    BaseParserRuleContext<'input, StringExpPrefixContextExt<'input>>;

#[derive(Clone)]
pub struct StringExpPrefixContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for StringExpPrefixContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for StringExpPrefixContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_stringExpPrefix(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_stringExpPrefix(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for StringExpPrefixContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_stringExpPrefix(self);
    }
}

impl<'input> CustomRuleContext<'input> for StringExpPrefixContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_stringExpPrefix
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_stringExpPrefix }
}
antlr4rust::tid! {StringExpPrefixContextExt<'a>}

impl<'input> StringExpPrefixContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<StringExpPrefixContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            StringExpPrefixContextExt { ph: PhantomData },
        ))
    }
}

pub trait StringExpPrefixContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<StringExpPrefixContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token STARTS
    /// Returns `None` if there is no child corresponding to token STARTS
    fn STARTS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STARTS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token WITH
    /// Returns `None` if there is no child corresponding to token WITH
    fn WITH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WITH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ENDS
    /// Returns `None` if there is no child corresponding to token ENDS
    fn ENDS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ENDS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CONTAINS
    /// Returns `None` if there is no child corresponding to token CONTAINS
    fn CONTAINS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CONTAINS, 0)
    }
}

impl<'input> StringExpPrefixContextAttrs<'input> for StringExpPrefixContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn stringExpPrefix(&mut self) -> Result<Rc<StringExpPrefixContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            StringExpPrefixContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 104, RULE_stringExpPrefix);
        let mut _localctx: Rc<StringExpPrefixContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(585);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_STARTS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(580);
                        recog
                            .base
                            .match_token(CypherParser_STARTS, &mut recog.err_handler)?;

                        recog.base.set_state(581);
                        recog
                            .base
                            .match_token(CypherParser_WITH, &mut recog.err_handler)?;
                    }
                }

                CypherParser_ENDS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(582);
                        recog
                            .base
                            .match_token(CypherParser_ENDS, &mut recog.err_handler)?;

                        recog.base.set_state(583);
                        recog
                            .base
                            .match_token(CypherParser_WITH, &mut recog.err_handler)?;
                    }
                }

                CypherParser_CONTAINS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(584);
                        recog
                            .base
                            .match_token(CypherParser_CONTAINS, &mut recog.err_handler)?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- nullExpression ----------------
pub type NullExpressionContextAll<'input> = NullExpressionContext<'input>;

pub type NullExpressionContext<'input> =
    BaseParserRuleContext<'input, NullExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct NullExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NullExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for NullExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_nullExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_nullExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NullExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_nullExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for NullExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_nullExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_nullExpression }
}
antlr4rust::tid! {NullExpressionContextExt<'a>}

impl<'input> NullExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NullExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NullExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait NullExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NullExpressionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token IS
    /// Returns `None` if there is no child corresponding to token IS
    fn IS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_IS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NULL_W
    /// Returns `None` if there is no child corresponding to token NULL_W
    fn NULL_W(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NULL_W, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NOT
    /// Returns `None` if there is no child corresponding to token NOT
    fn NOT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NOT, 0)
    }
}

impl<'input> NullExpressionContextAttrs<'input> for NullExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn nullExpression(&mut self) -> Result<Rc<NullExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            NullExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 106, RULE_nullExpression);
        let mut _localctx: Rc<NullExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(587);
                recog
                    .base
                    .match_token(CypherParser_IS, &mut recog.err_handler)?;

                recog.base.set_state(589);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_NOT {
                    {
                        recog.base.set_state(588);
                        recog
                            .base
                            .match_token(CypherParser_NOT, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(591);
                recog
                    .base
                    .match_token(CypherParser_NULL_W, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- propertyOrLabelExpression ----------------
pub type PropertyOrLabelExpressionContextAll<'input> = PropertyOrLabelExpressionContext<'input>;

pub type PropertyOrLabelExpressionContext<'input> =
    BaseParserRuleContext<'input, PropertyOrLabelExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct PropertyOrLabelExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PropertyOrLabelExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for PropertyOrLabelExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_propertyOrLabelExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_propertyOrLabelExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for PropertyOrLabelExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_propertyOrLabelExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for PropertyOrLabelExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_propertyOrLabelExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_propertyOrLabelExpression }
}
antlr4rust::tid! {PropertyOrLabelExpressionContextExt<'a>}

impl<'input> PropertyOrLabelExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PropertyOrLabelExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PropertyOrLabelExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait PropertyOrLabelExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PropertyOrLabelExpressionContextExt<'input>>
{
    fn propertyExpression(&self) -> Option<Rc<PropertyExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn nodeLabels(&self) -> Option<Rc<NodeLabelsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PropertyOrLabelExpressionContextAttrs<'input>
    for PropertyOrLabelExpressionContext<'input>
{
}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn propertyOrLabelExpression(
        &mut self,
    ) -> Result<Rc<PropertyOrLabelExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            PropertyOrLabelExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 108, RULE_propertyOrLabelExpression);
        let mut _localctx: Rc<PropertyOrLabelExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule propertyExpression*/
                recog.base.set_state(593);
                recog.propertyExpression()?;

                recog.base.set_state(595);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(594);
                        recog.nodeLabels()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- propertyExpression ----------------
pub type PropertyExpressionContextAll<'input> = PropertyExpressionContext<'input>;

pub type PropertyExpressionContext<'input> =
    BaseParserRuleContext<'input, PropertyExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct PropertyExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PropertyExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for PropertyExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_propertyExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_propertyExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for PropertyExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_propertyExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for PropertyExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_propertyExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_propertyExpression }
}
antlr4rust::tid! {PropertyExpressionContextExt<'a>}

impl<'input> PropertyExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PropertyExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PropertyExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait PropertyExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PropertyExpressionContextExt<'input>>
{
    fn atom(&self) -> Option<Rc<AtomContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves all `TerminalNode`s corresponding to token DOT in current rule
    fn DOT_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token DOT, starting from 0.
    /// Returns `None` if number of children corresponding to token DOT is less or equal than `i`.
    fn DOT(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DOT, i)
    }
    fn name_all(&self) -> Vec<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn name(&self, i: usize) -> Option<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> PropertyExpressionContextAttrs<'input> for PropertyExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn propertyExpression(
        &mut self,
    ) -> Result<Rc<PropertyExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            PropertyExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 110, RULE_propertyExpression);
        let mut _localctx: Rc<PropertyExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule atom*/
                recog.base.set_state(597);
                recog.atom()?;

                recog.base.set_state(602);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_DOT {
                    {
                        {
                            recog.base.set_state(598);
                            recog
                                .base
                                .match_token(CypherParser_DOT, &mut recog.err_handler)?;

                            /*InvokeRule name*/
                            recog.base.set_state(599);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(604);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- patternPart ----------------
pub type PatternPartContextAll<'input> = PatternPartContext<'input>;

pub type PatternPartContext<'input> = BaseParserRuleContext<'input, PatternPartContextExt<'input>>;

#[derive(Clone)]
pub struct PatternPartContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternPartContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for PatternPartContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_patternPart(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_patternPart(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for PatternPartContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_patternPart(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternPartContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_patternPart
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_patternPart }
}
antlr4rust::tid! {PatternPartContextExt<'a>}

impl<'input> PatternPartContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternPartContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternPartContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternPartContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternPartContextExt<'input>>
{
    fn shortestPathWrapper(&self) -> Option<Rc<ShortestPathWrapperContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn patternElem(&self) -> Option<Rc<PatternElemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ASSIGN
    /// Returns `None` if there is no child corresponding to token ASSIGN
    fn ASSIGN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASSIGN, 0)
    }
}

impl<'input> PatternPartContextAttrs<'input> for PatternPartContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn patternPart(&mut self) -> Result<Rc<PatternPartContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PatternPartContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 112, RULE_patternPart);
        let mut _localctx: Rc<PatternPartContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(608);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(605);
                        recog.symbol()?;

                        recog.base.set_state(606);
                        recog
                            .base
                            .match_token(CypherParser_ASSIGN, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(612);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_SHORTEST_PATH => {
                        {
                            /*InvokeRule shortestPathWrapper*/
                            recog.base.set_state(610);
                            recog.shortestPathWrapper()?;
                        }
                    }

                    CypherParser_LPAREN => {
                        {
                            /*InvokeRule patternElem*/
                            recog.base.set_state(611);
                            recog.patternElem()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- shortestPathWrapper ----------------
pub type ShortestPathWrapperContextAll<'input> = ShortestPathWrapperContext<'input>;

pub type ShortestPathWrapperContext<'input> =
    BaseParserRuleContext<'input, ShortestPathWrapperContextExt<'input>>;

#[derive(Clone)]
pub struct ShortestPathWrapperContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ShortestPathWrapperContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ShortestPathWrapperContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_shortestPathWrapper(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_shortestPathWrapper(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ShortestPathWrapperContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_shortestPathWrapper(self);
    }
}

impl<'input> CustomRuleContext<'input> for ShortestPathWrapperContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_shortestPathWrapper
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_shortestPathWrapper }
}
antlr4rust::tid! {ShortestPathWrapperContextExt<'a>}

impl<'input> ShortestPathWrapperContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ShortestPathWrapperContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ShortestPathWrapperContextExt { ph: PhantomData },
        ))
    }
}

pub trait ShortestPathWrapperContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ShortestPathWrapperContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token SHORTEST_PATH
    /// Returns `None` if there is no child corresponding to token SHORTEST_PATH
    fn SHORTEST_PATH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SHORTEST_PATH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    fn patternElem(&self) -> Option<Rc<PatternElemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
}

impl<'input> ShortestPathWrapperContextAttrs<'input> for ShortestPathWrapperContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn shortestPathWrapper(
        &mut self,
    ) -> Result<Rc<ShortestPathWrapperContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ShortestPathWrapperContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 114, RULE_shortestPathWrapper);
        let mut _localctx: Rc<ShortestPathWrapperContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(614);
                recog
                    .base
                    .match_token(CypherParser_SHORTEST_PATH, &mut recog.err_handler)?;

                recog.base.set_state(615);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule patternElem*/
                recog.base.set_state(616);
                recog.patternElem()?;

                recog.base.set_state(617);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- patternElem ----------------
pub type PatternElemContextAll<'input> = PatternElemContext<'input>;

pub type PatternElemContext<'input> = BaseParserRuleContext<'input, PatternElemContextExt<'input>>;

#[derive(Clone)]
pub struct PatternElemContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternElemContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for PatternElemContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_patternElem(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_patternElem(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for PatternElemContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_patternElem(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternElemContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_patternElem
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_patternElem }
}
antlr4rust::tid! {PatternElemContextExt<'a>}

impl<'input> PatternElemContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternElemContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternElemContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternElemContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternElemContextExt<'input>>
{
    fn nodePattern(&self) -> Option<Rc<NodePatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn patternElemChain_all(&self) -> Vec<Rc<PatternElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn patternElemChain(&self, i: usize) -> Option<Rc<PatternElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    fn qppElemChain_all(&self) -> Vec<Rc<QppElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn qppElemChain(&self, i: usize) -> Option<Rc<QppElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    fn patternElem(&self) -> Option<Rc<PatternElemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    fn qppQuantifier(&self) -> Option<Rc<QppQuantifierContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PatternElemContextAttrs<'input> for PatternElemContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn patternElem(&mut self) -> Result<Rc<PatternElemContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PatternElemContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 116, RULE_patternElem);
        let mut _localctx: Rc<PatternElemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(633);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(69, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule nodePattern*/
                        recog.base.set_state(619);
                        recog.nodePattern()?;

                        recog.base.set_state(624);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        while (((_la) & !0x3f) == 0 && ((1usize << _la) & 266304) != 0) {
                            {
                                recog.base.set_state(622);
                                recog.err_handler.sync(&mut recog.base)?;
                                match recog.base.input.la(1) {
                                    CypherParser_LT | CypherParser_SUB => {
                                        {
                                            /*InvokeRule patternElemChain*/
                                            recog.base.set_state(620);
                                            recog.patternElemChain()?;
                                        }
                                    }

                                    CypherParser_LPAREN => {
                                        {
                                            /*InvokeRule qppElemChain*/
                                            recog.base.set_state(621);
                                            recog.qppElemChain()?;
                                        }
                                    }

                                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                                        &mut recog.base,
                                    )))?,
                                }
                            }
                            recog.base.set_state(626);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                        }
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(627);
                        recog
                            .base
                            .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                        /*InvokeRule patternElem*/
                        recog.base.set_state(628);
                        recog.patternElem()?;

                        recog.base.set_state(629);
                        recog
                            .base
                            .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;

                        recog.base.set_state(631);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if (((_la) & !0x3f) == 0 && ((1usize << _la) & 8929280) != 0) {
                            {
                                /*InvokeRule qppQuantifier*/
                                recog.base.set_state(630);
                                recog.qppQuantifier()?;
                            }
                        }
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- patternElemChain ----------------
pub type PatternElemChainContextAll<'input> = PatternElemChainContext<'input>;

pub type PatternElemChainContext<'input> =
    BaseParserRuleContext<'input, PatternElemChainContextExt<'input>>;

#[derive(Clone)]
pub struct PatternElemChainContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternElemChainContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for PatternElemChainContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_patternElemChain(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_patternElemChain(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for PatternElemChainContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_patternElemChain(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternElemChainContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_patternElemChain
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_patternElemChain }
}
antlr4rust::tid! {PatternElemChainContextExt<'a>}

impl<'input> PatternElemChainContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternElemChainContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternElemChainContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternElemChainContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternElemChainContextExt<'input>>
{
    fn relationshipPattern(&self) -> Option<Rc<RelationshipPatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn nodePattern(&self) -> Option<Rc<NodePatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PatternElemChainContextAttrs<'input> for PatternElemChainContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn patternElemChain(
        &mut self,
    ) -> Result<Rc<PatternElemChainContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            PatternElemChainContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 118, RULE_patternElemChain);
        let mut _localctx: Rc<PatternElemChainContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule relationshipPattern*/
                recog.base.set_state(635);
                recog.relationshipPattern()?;

                /*InvokeRule nodePattern*/
                recog.base.set_state(636);
                recog.nodePattern()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- qppElemChain ----------------
pub type QppElemChainContextAll<'input> = QppElemChainContext<'input>;

pub type QppElemChainContext<'input> =
    BaseParserRuleContext<'input, QppElemChainContextExt<'input>>;

#[derive(Clone)]
pub struct QppElemChainContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QppElemChainContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for QppElemChainContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_qppElemChain(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_qppElemChain(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QppElemChainContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_qppElemChain(self);
    }
}

impl<'input> CustomRuleContext<'input> for QppElemChainContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_qppElemChain
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_qppElemChain }
}
antlr4rust::tid! {QppElemChainContextExt<'a>}

impl<'input> QppElemChainContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QppElemChainContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QppElemChainContextExt { ph: PhantomData },
        ))
    }
}

pub trait QppElemChainContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QppElemChainContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    fn patternElem(&self) -> Option<Rc<PatternElemContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    fn qppQuantifier(&self) -> Option<Rc<QppQuantifierContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn nodePattern(&self) -> Option<Rc<NodePatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> QppElemChainContextAttrs<'input> for QppElemChainContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn qppElemChain(&mut self) -> Result<Rc<QppElemChainContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = QppElemChainContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 120, RULE_qppElemChain);
        let mut _localctx: Rc<QppElemChainContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(638);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule patternElem*/
                recog.base.set_state(639);
                recog.patternElem()?;

                recog.base.set_state(640);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;

                /*InvokeRule qppQuantifier*/
                recog.base.set_state(641);
                recog.qppQuantifier()?;

                /*InvokeRule nodePattern*/
                recog.base.set_state(642);
                recog.nodePattern()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- qppQuantifier ----------------
pub type QppQuantifierContextAll<'input> = QppQuantifierContext<'input>;

pub type QppQuantifierContext<'input> =
    BaseParserRuleContext<'input, QppQuantifierContextExt<'input>>;

#[derive(Clone)]
pub struct QppQuantifierContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QppQuantifierContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for QppQuantifierContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_qppQuantifier(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_qppQuantifier(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QppQuantifierContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_qppQuantifier(self);
    }
}

impl<'input> CustomRuleContext<'input> for QppQuantifierContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_qppQuantifier
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_qppQuantifier }
}
antlr4rust::tid! {QppQuantifierContextExt<'a>}

impl<'input> QppQuantifierContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QppQuantifierContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QppQuantifierContextExt { ph: PhantomData },
        ))
    }
}

pub trait QppQuantifierContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QppQuantifierContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACE
    /// Returns `None` if there is no child corresponding to token LBRACE
    fn LBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACE, 0)
    }
    fn qppInt_all(&self) -> Vec<Rc<QppIntContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn qppInt(&self, i: usize) -> Option<Rc<QppIntContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token COMMA
    /// Returns `None` if there is no child corresponding to token COMMA
    fn COMMA(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACE
    /// Returns `None` if there is no child corresponding to token RBRACE
    fn RBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token PLUS
    /// Returns `None` if there is no child corresponding to token PLUS
    fn PLUS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_PLUS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MULT
    /// Returns `None` if there is no child corresponding to token MULT
    fn MULT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, 0)
    }
}

impl<'input> QppQuantifierContextAttrs<'input> for QppQuantifierContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn qppQuantifier(&mut self) -> Result<Rc<QppQuantifierContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            QppQuantifierContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 122, RULE_qppQuantifier);
        let mut _localctx: Rc<QppQuantifierContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(669);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(70, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(644);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(645);
                        recog.qppInt()?;

                        recog.base.set_state(646);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(647);
                        recog.qppInt()?;

                        recog.base.set_state(648);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(650);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(651);
                        recog.qppInt()?;

                        recog.base.set_state(652);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(654);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(655);
                        recog.qppInt()?;

                        recog.base.set_state(656);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        recog.base.set_state(657);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        recog.base.set_state(659);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        recog.base.set_state(660);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(661);
                        recog.qppInt()?;

                        recog.base.set_state(662);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                5 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        recog.base.set_state(664);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        recog.base.set_state(665);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        recog.base.set_state(666);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                6 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        recog.base.set_state(667);
                        recog
                            .base
                            .match_token(CypherParser_PLUS, &mut recog.err_handler)?;
                    }
                }
                7 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        recog.base.set_state(668);
                        recog
                            .base
                            .match_token(CypherParser_MULT, &mut recog.err_handler)?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- qppInt ----------------
pub type QppIntContextAll<'input> = QppIntContext<'input>;

pub type QppIntContext<'input> = BaseParserRuleContext<'input, QppIntContextExt<'input>>;

#[derive(Clone)]
pub struct QppIntContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for QppIntContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for QppIntContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_qppInt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_qppInt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for QppIntContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_qppInt(self);
    }
}

impl<'input> CustomRuleContext<'input> for QppIntContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_qppInt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_qppInt }
}
antlr4rust::tid! {QppIntContextExt<'a>}

impl<'input> QppIntContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<QppIntContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            QppIntContextExt { ph: PhantomData },
        ))
    }
}

pub trait QppIntContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<QppIntContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token DIGIT
    /// Returns `None` if there is no child corresponding to token DIGIT
    fn DIGIT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DIGIT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ID
    /// Returns `None` if there is no child corresponding to token ID
    fn ID(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ID, 0)
    }
}

impl<'input> QppIntContextAttrs<'input> for QppIntContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn qppInt(&mut self) -> Result<Rc<QppIntContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = QppIntContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 124, RULE_qppInt);
        let mut _localctx: Rc<QppIntContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(671);
                _la = recog.base.input.la(1);
                if { !(_la == CypherParser_ID || _la == CypherParser_DIGIT) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- properties ----------------
pub type PropertiesContextAll<'input> = PropertiesContext<'input>;

pub type PropertiesContext<'input> = BaseParserRuleContext<'input, PropertiesContextExt<'input>>;

#[derive(Clone)]
pub struct PropertiesContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PropertiesContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for PropertiesContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_properties(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_properties(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for PropertiesContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_properties(self);
    }
}

impl<'input> CustomRuleContext<'input> for PropertiesContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_properties
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_properties }
}
antlr4rust::tid! {PropertiesContextExt<'a>}

impl<'input> PropertiesContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PropertiesContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PropertiesContextExt { ph: PhantomData },
        ))
    }
}

pub trait PropertiesContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PropertiesContextExt<'input>>
{
    fn mapLit(&self) -> Option<Rc<MapLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn parameter(&self) -> Option<Rc<ParameterContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PropertiesContextAttrs<'input> for PropertiesContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn properties(&mut self) -> Result<Rc<PropertiesContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = PropertiesContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 126, RULE_properties);
        let mut _localctx: Rc<PropertiesContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(675);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_LBRACE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule mapLit*/
                        recog.base.set_state(673);
                        recog.mapLit()?;
                    }
                }

                CypherParser_DOLLAR => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule parameter*/
                        recog.base.set_state(674);
                        recog.parameter()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- nodePattern ----------------
pub type NodePatternContextAll<'input> = NodePatternContext<'input>;

pub type NodePatternContext<'input> = BaseParserRuleContext<'input, NodePatternContextExt<'input>>;

#[derive(Clone)]
pub struct NodePatternContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NodePatternContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for NodePatternContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_nodePattern(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_nodePattern(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NodePatternContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_nodePattern(self);
    }
}

impl<'input> CustomRuleContext<'input> for NodePatternContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_nodePattern
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_nodePattern }
}
antlr4rust::tid! {NodePatternContextExt<'a>}

impl<'input> NodePatternContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NodePatternContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NodePatternContextExt { ph: PhantomData },
        ))
    }
}

pub trait NodePatternContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NodePatternContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn nodeLabels(&self) -> Option<Rc<NodeLabelsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn properties(&self) -> Option<Rc<PropertiesContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> NodePatternContextAttrs<'input> for NodePatternContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn nodePattern(&mut self) -> Result<Rc<NodePatternContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = NodePatternContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 128, RULE_nodePattern);
        let mut _localctx: Rc<NodePatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(677);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(679);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(678);
                        recog.symbol()?;
                    }
                }

                recog.base.set_state(682);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(681);
                        recog.nodeLabels()?;
                    }
                }

                recog.base.set_state(685);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LBRACE || _la == CypherParser_DOLLAR {
                    {
                        /*InvokeRule properties*/
                        recog.base.set_state(684);
                        recog.properties()?;
                    }
                }

                recog.base.set_state(687);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- atom ----------------
pub type AtomContextAll<'input> = AtomContext<'input>;

pub type AtomContext<'input> = BaseParserRuleContext<'input, AtomContextExt<'input>>;

#[derive(Clone)]
pub struct AtomContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for AtomContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for AtomContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_atom(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_atom(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for AtomContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_atom(self);
    }
}

impl<'input> CustomRuleContext<'input> for AtomContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_atom
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_atom }
}
antlr4rust::tid! {AtomContextExt<'a>}

impl<'input> AtomContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<AtomContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            AtomContextExt { ph: PhantomData },
        ))
    }
}

pub trait AtomContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<AtomContextExt<'input>>
{
    fn literal(&self) -> Option<Rc<LiteralContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn parameter(&self) -> Option<Rc<ParameterContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn caseExpression(&self) -> Option<Rc<CaseExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn countAll(&self) -> Option<Rc<CountAllContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn listComprehension(&self) -> Option<Rc<ListComprehensionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn patternComprehension(&self) -> Option<Rc<PatternComprehensionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn filterWith(&self) -> Option<Rc<FilterWithContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn relationshipsChainPattern(&self) -> Option<Rc<RelationshipsChainPatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn parenthesizedExpression(&self) -> Option<Rc<ParenthesizedExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn functionInvocation(&self) -> Option<Rc<FunctionInvocationContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn subqueryExist(&self) -> Option<Rc<SubqueryExistContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> AtomContextAttrs<'input> for AtomContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn atom(&mut self) -> Result<Rc<AtomContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = AtomContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 130, RULE_atom);
        let mut _localctx: Rc<AtomContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(701);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(75, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule literal*/
                        recog.base.set_state(689);
                        recog.literal()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule parameter*/
                        recog.base.set_state(690);
                        recog.parameter()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule caseExpression*/
                        recog.base.set_state(691);
                        recog.caseExpression()?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule countAll*/
                        recog.base.set_state(692);
                        recog.countAll()?;
                    }
                }
                5 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule listComprehension*/
                        recog.base.set_state(693);
                        recog.listComprehension()?;
                    }
                }
                6 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        /*InvokeRule patternComprehension*/
                        recog.base.set_state(694);
                        recog.patternComprehension()?;
                    }
                }
                7 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        /*InvokeRule filterWith*/
                        recog.base.set_state(695);
                        recog.filterWith()?;
                    }
                }
                8 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 8)?;
                    recog.base.enter_outer_alt(None, 8)?;
                    {
                        /*InvokeRule relationshipsChainPattern*/
                        recog.base.set_state(696);
                        recog.relationshipsChainPattern()?;
                    }
                }
                9 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 9)?;
                    recog.base.enter_outer_alt(None, 9)?;
                    {
                        /*InvokeRule parenthesizedExpression*/
                        recog.base.set_state(697);
                        recog.parenthesizedExpression()?;
                    }
                }
                10 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 10)?;
                    recog.base.enter_outer_alt(None, 10)?;
                    {
                        /*InvokeRule functionInvocation*/
                        recog.base.set_state(698);
                        recog.functionInvocation()?;
                    }
                }
                11 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 11)?;
                    recog.base.enter_outer_alt(None, 11)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(699);
                        recog.symbol()?;
                    }
                }
                12 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 12)?;
                    recog.base.enter_outer_alt(None, 12)?;
                    {
                        /*InvokeRule subqueryExist*/
                        recog.base.set_state(700);
                        recog.subqueryExist()?;
                    }
                }

                _ => {}
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- lhs ----------------
pub type LhsContextAll<'input> = LhsContext<'input>;

pub type LhsContext<'input> = BaseParserRuleContext<'input, LhsContextExt<'input>>;

#[derive(Clone)]
pub struct LhsContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for LhsContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for LhsContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_lhs(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_lhs(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for LhsContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_lhs(self);
    }
}

impl<'input> CustomRuleContext<'input> for LhsContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_lhs
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_lhs }
}
antlr4rust::tid! {LhsContextExt<'a>}

impl<'input> LhsContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<LhsContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            LhsContextExt { ph: PhantomData },
        ))
    }
}

pub trait LhsContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<LhsContextExt<'input>>
{
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ASSIGN
    /// Returns `None` if there is no child corresponding to token ASSIGN
    fn ASSIGN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASSIGN, 0)
    }
}

impl<'input> LhsContextAttrs<'input> for LhsContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn lhs(&mut self) -> Result<Rc<LhsContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = LhsContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 132, RULE_lhs);
        let mut _localctx: Rc<LhsContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(703);
                recog.symbol()?;

                recog.base.set_state(704);
                recog
                    .base
                    .match_token(CypherParser_ASSIGN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relationshipPattern ----------------
pub type RelationshipPatternContextAll<'input> = RelationshipPatternContext<'input>;

pub type RelationshipPatternContext<'input> =
    BaseParserRuleContext<'input, RelationshipPatternContextExt<'input>>;

#[derive(Clone)]
pub struct RelationshipPatternContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RelationshipPatternContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for RelationshipPatternContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relationshipPattern(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_relationshipPattern(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for RelationshipPatternContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_relationshipPattern(self);
    }
}

impl<'input> CustomRuleContext<'input> for RelationshipPatternContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relationshipPattern
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relationshipPattern }
}
antlr4rust::tid! {RelationshipPatternContextExt<'a>}

impl<'input> RelationshipPatternContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RelationshipPatternContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RelationshipPatternContextExt { ph: PhantomData },
        ))
    }
}

pub trait RelationshipPatternContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RelationshipPatternContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LT
    /// Returns `None` if there is no child corresponding to token LT
    fn LT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LT, 0)
    }
    /// Retrieves all `TerminalNode`s corresponding to token SUB in current rule
    fn SUB_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token SUB, starting from 0.
    /// Returns `None` if number of children corresponding to token SUB is less or equal than `i`.
    fn SUB(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SUB, i)
    }
    fn relationDetail(&self) -> Option<Rc<RelationDetailContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token GT
    /// Returns `None` if there is no child corresponding to token GT
    fn GT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_GT, 0)
    }
}

impl<'input> RelationshipPatternContextAttrs<'input> for RelationshipPatternContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relationshipPattern(
        &mut self,
    ) -> Result<Rc<RelationshipPatternContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            RelationshipPatternContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 134, RULE_relationshipPattern);
        let mut _localctx: Rc<RelationshipPatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(723);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_LT => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(706);
                        recog
                            .base
                            .match_token(CypherParser_LT, &mut recog.err_handler)?;

                        recog.base.set_state(707);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(709);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_LBRACK {
                            {
                                /*InvokeRule relationDetail*/
                                recog.base.set_state(708);
                                recog.relationDetail()?;
                            }
                        }

                        recog.base.set_state(711);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(713);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_GT {
                            {
                                recog.base.set_state(712);
                                recog
                                    .base
                                    .match_token(CypherParser_GT, &mut recog.err_handler)?;
                            }
                        }
                    }
                }

                CypherParser_SUB => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(715);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(717);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_LBRACK {
                            {
                                /*InvokeRule relationDetail*/
                                recog.base.set_state(716);
                                recog.relationDetail()?;
                            }
                        }

                        recog.base.set_state(719);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(721);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_GT {
                            {
                                recog.base.set_state(720);
                                recog
                                    .base
                                    .match_token(CypherParser_GT, &mut recog.err_handler)?;
                            }
                        }
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relationDetail ----------------
pub type RelationDetailContextAll<'input> = RelationDetailContext<'input>;

pub type RelationDetailContext<'input> =
    BaseParserRuleContext<'input, RelationDetailContextExt<'input>>;

#[derive(Clone)]
pub struct RelationDetailContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RelationDetailContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for RelationDetailContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relationDetail(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_relationDetail(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for RelationDetailContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_relationDetail(self);
    }
}

impl<'input> CustomRuleContext<'input> for RelationDetailContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relationDetail
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relationDetail }
}
antlr4rust::tid! {RelationDetailContextExt<'a>}

impl<'input> RelationDetailContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RelationDetailContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RelationDetailContextExt { ph: PhantomData },
        ))
    }
}

pub trait RelationDetailContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RelationDetailContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACK, 0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn relationshipTypes(&self) -> Option<Rc<RelationshipTypesContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn rangeLit(&self) -> Option<Rc<RangeLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn properties(&self) -> Option<Rc<PropertiesContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> RelationDetailContextAttrs<'input> for RelationDetailContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relationDetail(&mut self) -> Result<Rc<RelationDetailContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            RelationDetailContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 136, RULE_relationDetail);
        let mut _localctx: Rc<RelationDetailContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(725);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(727);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(726);
                        recog.symbol()?;
                    }
                }

                recog.base.set_state(730);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule relationshipTypes*/
                        recog.base.set_state(729);
                        recog.relationshipTypes()?;
                    }
                }

                recog.base.set_state(733);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_MULT {
                    {
                        /*InvokeRule rangeLit*/
                        recog.base.set_state(732);
                        recog.rangeLit()?;
                    }
                }

                recog.base.set_state(736);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LBRACE || _la == CypherParser_DOLLAR {
                    {
                        /*InvokeRule properties*/
                        recog.base.set_state(735);
                        recog.properties()?;
                    }
                }

                recog.base.set_state(738);
                recog
                    .base
                    .match_token(CypherParser_RBRACK, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relationshipTypes ----------------
pub type RelationshipTypesContextAll<'input> = RelationshipTypesContext<'input>;

pub type RelationshipTypesContext<'input> =
    BaseParserRuleContext<'input, RelationshipTypesContextExt<'input>>;

#[derive(Clone)]
pub struct RelationshipTypesContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RelationshipTypesContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for RelationshipTypesContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relationshipTypes(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_relationshipTypes(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for RelationshipTypesContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_relationshipTypes(self);
    }
}

impl<'input> CustomRuleContext<'input> for RelationshipTypesContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relationshipTypes
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relationshipTypes }
}
antlr4rust::tid! {RelationshipTypesContextExt<'a>}

impl<'input> RelationshipTypesContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RelationshipTypesContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RelationshipTypesContextExt { ph: PhantomData },
        ))
    }
}

pub trait RelationshipTypesContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RelationshipTypesContextExt<'input>>
{
    /// Retrieves all `TerminalNode`s corresponding to token COLON in current rule
    fn COLON_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COLON, starting from 0.
    /// Returns `None` if number of children corresponding to token COLON is less or equal than `i`.
    fn COLON(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COLON, i)
    }
    fn name_all(&self) -> Vec<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn name(&self, i: usize) -> Option<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token STICK in current rule
    fn STICK_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token STICK, starting from 0.
    /// Returns `None` if number of children corresponding to token STICK is less or equal than `i`.
    fn STICK(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STICK, i)
    }
}

impl<'input> RelationshipTypesContextAttrs<'input> for RelationshipTypesContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relationshipTypes(
        &mut self,
    ) -> Result<Rc<RelationshipTypesContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            RelationshipTypesContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 138, RULE_relationshipTypes);
        let mut _localctx: Rc<RelationshipTypesContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(740);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(741);
                recog.name()?;

                recog.base.set_state(749);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_STICK {
                    {
                        {
                            recog.base.set_state(742);
                            recog
                                .base
                                .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                            recog.base.set_state(744);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            if _la == CypherParser_COLON {
                                {
                                    recog.base.set_state(743);
                                    recog
                                        .base
                                        .match_token(CypherParser_COLON, &mut recog.err_handler)?;
                                }
                            }

                            /*InvokeRule name*/
                            recog.base.set_state(746);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(751);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- unionSt ----------------
pub type UnionStContextAll<'input> = UnionStContext<'input>;

pub type UnionStContext<'input> = BaseParserRuleContext<'input, UnionStContextExt<'input>>;

#[derive(Clone)]
pub struct UnionStContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for UnionStContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for UnionStContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_unionSt(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_unionSt(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for UnionStContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_unionSt(self);
    }
}

impl<'input> CustomRuleContext<'input> for UnionStContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_unionSt
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_unionSt }
}
antlr4rust::tid! {UnionStContextExt<'a>}

impl<'input> UnionStContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<UnionStContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            UnionStContextExt { ph: PhantomData },
        ))
    }
}

pub trait UnionStContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<UnionStContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token UNION
    /// Returns `None` if there is no child corresponding to token UNION
    fn UNION(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNION, 0)
    }
    fn singleQuery(&self) -> Option<Rc<SingleQueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token ALL
    /// Returns `None` if there is no child corresponding to token ALL
    fn ALL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ALL, 0)
    }
}

impl<'input> UnionStContextAttrs<'input> for UnionStContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn unionSt(&mut self) -> Result<Rc<UnionStContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = UnionStContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 140, RULE_unionSt);
        let mut _localctx: Rc<UnionStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(752);
                recog
                    .base
                    .match_token(CypherParser_UNION, &mut recog.err_handler)?;

                recog.base.set_state(754);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ALL {
                    {
                        recog.base.set_state(753);
                        recog
                            .base
                            .match_token(CypherParser_ALL, &mut recog.err_handler)?;
                    }
                }

                /*InvokeRule singleQuery*/
                recog.base.set_state(756);
                recog.singleQuery()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- subqueryExist ----------------
pub type SubqueryExistContextAll<'input> = SubqueryExistContext<'input>;

pub type SubqueryExistContext<'input> =
    BaseParserRuleContext<'input, SubqueryExistContextExt<'input>>;

#[derive(Clone)]
pub struct SubqueryExistContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SubqueryExistContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for SubqueryExistContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_subqueryExist(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_subqueryExist(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SubqueryExistContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_subqueryExist(self);
    }
}

impl<'input> CustomRuleContext<'input> for SubqueryExistContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_subqueryExist
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_subqueryExist }
}
antlr4rust::tid! {SubqueryExistContextExt<'a>}

impl<'input> SubqueryExistContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SubqueryExistContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SubqueryExistContextExt { ph: PhantomData },
        ))
    }
}

pub trait SubqueryExistContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SubqueryExistContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token EXISTS
    /// Returns `None` if there is no child corresponding to token EXISTS
    fn EXISTS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EXISTS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LBRACE
    /// Returns `None` if there is no child corresponding to token LBRACE
    fn LBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACE
    /// Returns `None` if there is no child corresponding to token RBRACE
    fn RBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACE, 0)
    }
    fn regularQuery(&self) -> Option<Rc<RegularQueryContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn patternWhere(&self) -> Option<Rc<PatternWhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> SubqueryExistContextAttrs<'input> for SubqueryExistContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn subqueryExist(&mut self) -> Result<Rc<SubqueryExistContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            SubqueryExistContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 142, RULE_subqueryExist);
        let mut _localctx: Rc<SubqueryExistContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(758);
                recog
                    .base
                    .match_token(CypherParser_EXISTS, &mut recog.err_handler)?;

                recog.base.set_state(759);
                recog
                    .base
                    .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                recog.base.set_state(762);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_CALL
                    | CypherParser_CREATE
                    | CypherParser_DELETE
                    | CypherParser_DETACH
                    | CypherParser_MATCH
                    | CypherParser_MERGE
                    | CypherParser_OPTIONAL
                    | CypherParser_REMOVE
                    | CypherParser_RETURN
                    | CypherParser_SET
                    | CypherParser_WITH
                    | CypherParser_UNWIND => {
                        {
                            /*InvokeRule regularQuery*/
                            recog.base.set_state(760);
                            recog.regularQuery()?;
                        }
                    }

                    CypherParser_LPAREN
                    | CypherParser_FILTER
                    | CypherParser_EXTRACT
                    | CypherParser_COUNT
                    | CypherParser_ANY
                    | CypherParser_NONE
                    | CypherParser_SINGLE
                    | CypherParser_SHORTEST_PATH
                    | CypherParser_ID
                    | CypherParser_ESC_LITERAL => {
                        {
                            /*InvokeRule patternWhere*/
                            recog.base.set_state(761);
                            recog.patternWhere()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
                recog.base.set_state(764);
                recog
                    .base
                    .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- invocationName ----------------
pub type InvocationNameContextAll<'input> = InvocationNameContext<'input>;

pub type InvocationNameContext<'input> =
    BaseParserRuleContext<'input, InvocationNameContextExt<'input>>;

#[derive(Clone)]
pub struct InvocationNameContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for InvocationNameContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for InvocationNameContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_invocationName(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_invocationName(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for InvocationNameContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_invocationName(self);
    }
}

impl<'input> CustomRuleContext<'input> for InvocationNameContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_invocationName
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_invocationName }
}
antlr4rust::tid! {InvocationNameContextExt<'a>}

impl<'input> InvocationNameContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<InvocationNameContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            InvocationNameContextExt { ph: PhantomData },
        ))
    }
}

pub trait InvocationNameContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<InvocationNameContextExt<'input>>
{
    fn symbol_all(&self) -> Vec<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn symbol(&self, i: usize) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token DOT in current rule
    fn DOT_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token DOT, starting from 0.
    /// Returns `None` if number of children corresponding to token DOT is less or equal than `i`.
    fn DOT(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DOT, i)
    }
}

impl<'input> InvocationNameContextAttrs<'input> for InvocationNameContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn invocationName(&mut self) -> Result<Rc<InvocationNameContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            InvocationNameContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 144, RULE_invocationName);
        let mut _localctx: Rc<InvocationNameContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(766);
                recog.symbol()?;

                recog.base.set_state(771);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_DOT {
                    {
                        {
                            recog.base.set_state(767);
                            recog
                                .base
                                .match_token(CypherParser_DOT, &mut recog.err_handler)?;

                            /*InvokeRule symbol*/
                            recog.base.set_state(768);
                            recog.symbol()?;
                        }
                    }
                    recog.base.set_state(773);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- functionInvocation ----------------
pub type FunctionInvocationContextAll<'input> = FunctionInvocationContext<'input>;

pub type FunctionInvocationContext<'input> =
    BaseParserRuleContext<'input, FunctionInvocationContextExt<'input>>;

#[derive(Clone)]
pub struct FunctionInvocationContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for FunctionInvocationContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for FunctionInvocationContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_functionInvocation(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_functionInvocation(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for FunctionInvocationContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_functionInvocation(self);
    }
}

impl<'input> CustomRuleContext<'input> for FunctionInvocationContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_functionInvocation
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_functionInvocation }
}
antlr4rust::tid! {FunctionInvocationContextExt<'a>}

impl<'input> FunctionInvocationContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<FunctionInvocationContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            FunctionInvocationContextExt { ph: PhantomData },
        ))
    }
}

pub trait FunctionInvocationContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<FunctionInvocationContextExt<'input>>
{
    fn invocationName(&self) -> Option<Rc<InvocationNameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DISTINCT
    /// Returns `None` if there is no child corresponding to token DISTINCT
    fn DISTINCT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DISTINCT, 0)
    }
    fn expressionChain(&self) -> Option<Rc<ExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> FunctionInvocationContextAttrs<'input> for FunctionInvocationContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn functionInvocation(
        &mut self,
    ) -> Result<Rc<FunctionInvocationContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            FunctionInvocationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 146, RULE_functionInvocation);
        let mut _localctx: Rc<FunctionInvocationContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule invocationName*/
                recog.base.set_state(774);
                recog.invocationName()?;

                recog.base.set_state(775);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(777);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DISTINCT {
                    {
                        recog.base.set_state(776);
                        recog
                            .base
                            .match_token(CypherParser_DISTINCT, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(780);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(779);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(782);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- parenthesizedExpression ----------------
pub type ParenthesizedExpressionContextAll<'input> = ParenthesizedExpressionContext<'input>;

pub type ParenthesizedExpressionContext<'input> =
    BaseParserRuleContext<'input, ParenthesizedExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct ParenthesizedExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ParenthesizedExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ParenthesizedExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_parenthesizedExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_parenthesizedExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ParenthesizedExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_parenthesizedExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for ParenthesizedExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_parenthesizedExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_parenthesizedExpression }
}
antlr4rust::tid! {ParenthesizedExpressionContextExt<'a>}

impl<'input> ParenthesizedExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ParenthesizedExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ParenthesizedExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ParenthesizedExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ParenthesizedExpressionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
}

impl<'input> ParenthesizedExpressionContextAttrs<'input>
    for ParenthesizedExpressionContext<'input>
{
}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn parenthesizedExpression(
        &mut self,
    ) -> Result<Rc<ParenthesizedExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ParenthesizedExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 148, RULE_parenthesizedExpression);
        let mut _localctx: Rc<ParenthesizedExpressionContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(784);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(785);
                recog.expression()?;

                recog.base.set_state(786);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- filterWith ----------------
pub type FilterWithContextAll<'input> = FilterWithContext<'input>;

pub type FilterWithContext<'input> = BaseParserRuleContext<'input, FilterWithContextExt<'input>>;

#[derive(Clone)]
pub struct FilterWithContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for FilterWithContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for FilterWithContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_filterWith(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_filterWith(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for FilterWithContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_filterWith(self);
    }
}

impl<'input> CustomRuleContext<'input> for FilterWithContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_filterWith
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_filterWith }
}
antlr4rust::tid! {FilterWithContextExt<'a>}

impl<'input> FilterWithContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<FilterWithContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            FilterWithContextExt { ph: PhantomData },
        ))
    }
}

pub trait FilterWithContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<FilterWithContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    fn filterExpression(&self) -> Option<Rc<FilterExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ALL
    /// Returns `None` if there is no child corresponding to token ALL
    fn ALL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ALL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ANY
    /// Returns `None` if there is no child corresponding to token ANY
    fn ANY(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ANY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NONE
    /// Returns `None` if there is no child corresponding to token NONE
    fn NONE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NONE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SINGLE
    /// Returns `None` if there is no child corresponding to token SINGLE
    fn SINGLE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SINGLE, 0)
    }
}

impl<'input> FilterWithContextAttrs<'input> for FilterWithContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn filterWith(&mut self) -> Result<Rc<FilterWithContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = FilterWithContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 150, RULE_filterWith);
        let mut _localctx: Rc<FilterWithContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(788);
                _la = recog.base.input.la(1);
                if { !(((_la - 33) & !0x3f) == 0 && ((1usize << (_la - 33)) & 15) != 0) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
                recog.base.set_state(789);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule filterExpression*/
                recog.base.set_state(790);
                recog.filterExpression()?;

                recog.base.set_state(791);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- patternComprehension ----------------
pub type PatternComprehensionContextAll<'input> = PatternComprehensionContext<'input>;

pub type PatternComprehensionContext<'input> =
    BaseParserRuleContext<'input, PatternComprehensionContextExt<'input>>;

#[derive(Clone)]
pub struct PatternComprehensionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for PatternComprehensionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for PatternComprehensionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_patternComprehension(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_patternComprehension(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for PatternComprehensionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_patternComprehension(self);
    }
}

impl<'input> CustomRuleContext<'input> for PatternComprehensionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_patternComprehension
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_patternComprehension }
}
antlr4rust::tid! {PatternComprehensionContextExt<'a>}

impl<'input> PatternComprehensionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<PatternComprehensionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            PatternComprehensionContextExt { ph: PhantomData },
        ))
    }
}

pub trait PatternComprehensionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<PatternComprehensionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACK, 0)
    }
    fn relationshipsChainPattern(&self) -> Option<Rc<RelationshipsChainPatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token STICK
    /// Returns `None` if there is no child corresponding to token STICK
    fn STICK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STICK, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACK, 0)
    }
    fn lhs(&self) -> Option<Rc<LhsContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn where_(&self) -> Option<Rc<WhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> PatternComprehensionContextAttrs<'input> for PatternComprehensionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn patternComprehension(
        &mut self,
    ) -> Result<Rc<PatternComprehensionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            PatternComprehensionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 152, RULE_patternComprehension);
        let mut _localctx: Rc<PatternComprehensionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(793);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(795);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule lhs*/
                        recog.base.set_state(794);
                        recog.lhs()?;
                    }
                }

                /*InvokeRule relationshipsChainPattern*/
                recog.base.set_state(797);
                recog.relationshipsChainPattern()?;

                recog.base.set_state(799);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(798);
                        recog.where_()?;
                    }
                }

                recog.base.set_state(801);
                recog
                    .base
                    .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(802);
                recog.expression()?;

                recog.base.set_state(803);
                recog
                    .base
                    .match_token(CypherParser_RBRACK, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- relationshipsChainPattern ----------------
pub type RelationshipsChainPatternContextAll<'input> = RelationshipsChainPatternContext<'input>;

pub type RelationshipsChainPatternContext<'input> =
    BaseParserRuleContext<'input, RelationshipsChainPatternContextExt<'input>>;

#[derive(Clone)]
pub struct RelationshipsChainPatternContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RelationshipsChainPatternContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for RelationshipsChainPatternContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_relationshipsChainPattern(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_relationshipsChainPattern(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for RelationshipsChainPatternContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_relationshipsChainPattern(self);
    }
}

impl<'input> CustomRuleContext<'input> for RelationshipsChainPatternContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_relationshipsChainPattern
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_relationshipsChainPattern }
}
antlr4rust::tid! {RelationshipsChainPatternContextExt<'a>}

impl<'input> RelationshipsChainPatternContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RelationshipsChainPatternContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RelationshipsChainPatternContextExt { ph: PhantomData },
        ))
    }
}

pub trait RelationshipsChainPatternContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RelationshipsChainPatternContextExt<'input>>
{
    fn nodePattern(&self) -> Option<Rc<NodePatternContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn patternElemChain_all(&self) -> Vec<Rc<PatternElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn patternElemChain(&self, i: usize) -> Option<Rc<PatternElemChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> RelationshipsChainPatternContextAttrs<'input>
    for RelationshipsChainPatternContext<'input>
{
}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn relationshipsChainPattern(
        &mut self,
    ) -> Result<Rc<RelationshipsChainPatternContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            RelationshipsChainPatternContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 154, RULE_relationshipsChainPattern);
        let mut _localctx: Rc<RelationshipsChainPatternContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            let mut _alt: i32;
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule nodePattern*/
                recog.base.set_state(805);
                recog.nodePattern()?;

                recog.base.set_state(807);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = 1;
                loop {
                    match _alt {
                        x if x == 1 => {
                            {
                                /*InvokeRule patternElemChain*/
                                recog.base.set_state(806);
                                recog.patternElemChain()?;
                            }
                        }

                        _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                            &mut recog.base,
                        )))?,
                    }
                    recog.base.set_state(809);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(94, &mut recog.base)?;
                    if _alt == 2 || _alt == INVALID_ALT {
                        break;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- listComprehension ----------------
pub type ListComprehensionContextAll<'input> = ListComprehensionContext<'input>;

pub type ListComprehensionContext<'input> =
    BaseParserRuleContext<'input, ListComprehensionContextExt<'input>>;

#[derive(Clone)]
pub struct ListComprehensionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ListComprehensionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ListComprehensionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_listComprehension(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_listComprehension(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ListComprehensionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_listComprehension(self);
    }
}

impl<'input> CustomRuleContext<'input> for ListComprehensionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_listComprehension
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_listComprehension }
}
antlr4rust::tid! {ListComprehensionContextExt<'a>}

impl<'input> ListComprehensionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ListComprehensionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ListComprehensionContextExt { ph: PhantomData },
        ))
    }
}

pub trait ListComprehensionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ListComprehensionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACK, 0)
    }
    fn filterExpression(&self) -> Option<Rc<FilterExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token STICK
    /// Returns `None` if there is no child corresponding to token STICK
    fn STICK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STICK, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ListComprehensionContextAttrs<'input> for ListComprehensionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn listComprehension(
        &mut self,
    ) -> Result<Rc<ListComprehensionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ListComprehensionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 156, RULE_listComprehension);
        let mut _localctx: Rc<ListComprehensionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(811);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                /*InvokeRule filterExpression*/
                recog.base.set_state(812);
                recog.filterExpression()?;

                recog.base.set_state(815);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_STICK {
                    {
                        recog.base.set_state(813);
                        recog
                            .base
                            .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(814);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(817);
                recog
                    .base
                    .match_token(CypherParser_RBRACK, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- filterExpression ----------------
pub type FilterExpressionContextAll<'input> = FilterExpressionContext<'input>;

pub type FilterExpressionContext<'input> =
    BaseParserRuleContext<'input, FilterExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct FilterExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for FilterExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for FilterExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_filterExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_filterExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for FilterExpressionContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_filterExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for FilterExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_filterExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_filterExpression }
}
antlr4rust::tid! {FilterExpressionContextExt<'a>}

impl<'input> FilterExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<FilterExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            FilterExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait FilterExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<FilterExpressionContextExt<'input>>
{
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token IN
    /// Returns `None` if there is no child corresponding to token IN
    fn IN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_IN, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn where_(&self) -> Option<Rc<WhereContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> FilterExpressionContextAttrs<'input> for FilterExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn filterExpression(
        &mut self,
    ) -> Result<Rc<FilterExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            FilterExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 158, RULE_filterExpression);
        let mut _localctx: Rc<FilterExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(819);
                recog.symbol()?;

                recog.base.set_state(820);
                recog
                    .base
                    .match_token(CypherParser_IN, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(821);
                recog.expression()?;

                recog.base.set_state(823);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(822);
                        recog.where_()?;
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- countAll ----------------
pub type CountAllContextAll<'input> = CountAllContext<'input>;

pub type CountAllContext<'input> = BaseParserRuleContext<'input, CountAllContextExt<'input>>;

#[derive(Clone)]
pub struct CountAllContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for CountAllContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for CountAllContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_countAll(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_countAll(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for CountAllContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_countAll(self);
    }
}

impl<'input> CustomRuleContext<'input> for CountAllContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_countAll
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_countAll }
}
antlr4rust::tid! {CountAllContextExt<'a>}

impl<'input> CountAllContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CountAllContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CountAllContextExt { ph: PhantomData },
        ))
    }
}

pub trait CountAllContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<CountAllContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token COUNT
    /// Returns `None` if there is no child corresponding to token COUNT
    fn COUNT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COUNT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LPAREN
    /// Returns `None` if there is no child corresponding to token LPAREN
    fn LPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LPAREN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MULT
    /// Returns `None` if there is no child corresponding to token MULT
    fn MULT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RPAREN
    /// Returns `None` if there is no child corresponding to token RPAREN
    fn RPAREN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RPAREN, 0)
    }
}

impl<'input> CountAllContextAttrs<'input> for CountAllContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn countAll(&mut self) -> Result<Rc<CountAllContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = CountAllContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 160, RULE_countAll);
        let mut _localctx: Rc<CountAllContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(825);
                recog
                    .base
                    .match_token(CypherParser_COUNT, &mut recog.err_handler)?;

                recog.base.set_state(826);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(827);
                recog
                    .base
                    .match_token(CypherParser_MULT, &mut recog.err_handler)?;

                recog.base.set_state(828);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- expressionChain ----------------
pub type ExpressionChainContextAll<'input> = ExpressionChainContext<'input>;

pub type ExpressionChainContext<'input> =
    BaseParserRuleContext<'input, ExpressionChainContextExt<'input>>;

#[derive(Clone)]
pub struct ExpressionChainContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ExpressionChainContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for ExpressionChainContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_expressionChain(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_expressionChain(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a>
    for ExpressionChainContext<'input>
{
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_expressionChain(self);
    }
}

impl<'input> CustomRuleContext<'input> for ExpressionChainContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_expressionChain
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_expressionChain }
}
antlr4rust::tid! {ExpressionChainContextExt<'a>}

impl<'input> ExpressionChainContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ExpressionChainContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ExpressionChainContextExt { ph: PhantomData },
        ))
    }
}

pub trait ExpressionChainContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ExpressionChainContextExt<'input>>
{
    fn expression_all(&self) -> Vec<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn expression(&self, i: usize) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> ExpressionChainContextAttrs<'input> for ExpressionChainContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn expressionChain(&mut self) -> Result<Rc<ExpressionChainContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            ExpressionChainContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 162, RULE_expressionChain);
        let mut _localctx: Rc<ExpressionChainContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(830);
                recog.expression()?;

                recog.base.set_state(835);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(831);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(832);
                            recog.expression()?;
                        }
                    }
                    recog.base.set_state(837);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- caseExpression ----------------
pub type CaseExpressionContextAll<'input> = CaseExpressionContext<'input>;

pub type CaseExpressionContext<'input> =
    BaseParserRuleContext<'input, CaseExpressionContextExt<'input>>;

#[derive(Clone)]
pub struct CaseExpressionContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for CaseExpressionContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a>
    for CaseExpressionContext<'input>
{
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_caseExpression(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_caseExpression(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for CaseExpressionContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_caseExpression(self);
    }
}

impl<'input> CustomRuleContext<'input> for CaseExpressionContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_caseExpression
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_caseExpression }
}
antlr4rust::tid! {CaseExpressionContextExt<'a>}

impl<'input> CaseExpressionContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CaseExpressionContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CaseExpressionContextExt { ph: PhantomData },
        ))
    }
}

pub trait CaseExpressionContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<CaseExpressionContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CASE
    /// Returns `None` if there is no child corresponding to token CASE
    fn CASE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CASE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token END
    /// Returns `None` if there is no child corresponding to token END
    fn END(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_END, 0)
    }
    fn expression_all(&self) -> Vec<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn expression(&self, i: usize) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token WHEN in current rule
    fn WHEN_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token WHEN, starting from 0.
    /// Returns `None` if number of children corresponding to token WHEN is less or equal than `i`.
    fn WHEN(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WHEN, i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token THEN in current rule
    fn THEN_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token THEN, starting from 0.
    /// Returns `None` if number of children corresponding to token THEN is less or equal than `i`.
    fn THEN(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_THEN, i)
    }
    /// Retrieves first TerminalNode corresponding to token ELSE
    /// Returns `None` if there is no child corresponding to token ELSE
    fn ELSE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ELSE, 0)
    }
}

impl<'input> CaseExpressionContextAttrs<'input> for CaseExpressionContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn caseExpression(&mut self) -> Result<Rc<CaseExpressionContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx =
            CaseExpressionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 164, RULE_caseExpression);
        let mut _localctx: Rc<CaseExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(838);
                recog
                    .base
                    .match_token(CypherParser_CASE, &mut recog.err_handler)?;

                recog.base.set_state(840);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expression*/
                        recog.base.set_state(839);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(847);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                loop {
                    {
                        {
                            recog.base.set_state(842);
                            recog
                                .base
                                .match_token(CypherParser_WHEN, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(843);
                            recog.expression()?;

                            recog.base.set_state(844);
                            recog
                                .base
                                .match_token(CypherParser_THEN, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(845);
                            recog.expression()?;
                        }
                    }
                    recog.base.set_state(849);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                    if !(_la == CypherParser_WHEN) {
                        break;
                    }
                }
                recog.base.set_state(853);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ELSE {
                    {
                        recog.base.set_state(851);
                        recog
                            .base
                            .match_token(CypherParser_ELSE, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(852);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(855);
                recog
                    .base
                    .match_token(CypherParser_END, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- parameter ----------------
pub type ParameterContextAll<'input> = ParameterContext<'input>;

pub type ParameterContext<'input> = BaseParserRuleContext<'input, ParameterContextExt<'input>>;

#[derive(Clone)]
pub struct ParameterContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ParameterContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ParameterContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_parameter(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_parameter(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ParameterContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_parameter(self);
    }
}

impl<'input> CustomRuleContext<'input> for ParameterContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_parameter
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_parameter }
}
antlr4rust::tid! {ParameterContextExt<'a>}

impl<'input> ParameterContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ParameterContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ParameterContextExt { ph: PhantomData },
        ))
    }
}

pub trait ParameterContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ParameterContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token DOLLAR
    /// Returns `None` if there is no child corresponding to token DOLLAR
    fn DOLLAR(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DOLLAR, 0)
    }
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn numLit(&self) -> Option<Rc<NumLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ParameterContextAttrs<'input> for ParameterContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn parameter(&mut self) -> Result<Rc<ParameterContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ParameterContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 166, RULE_parameter);
        let mut _localctx: Rc<ParameterContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(857);
                recog
                    .base
                    .match_token(CypherParser_DOLLAR, &mut recog.err_handler)?;

                recog.base.set_state(860);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_FILTER
                    | CypherParser_EXTRACT
                    | CypherParser_COUNT
                    | CypherParser_ANY
                    | CypherParser_NONE
                    | CypherParser_SINGLE
                    | CypherParser_ID
                    | CypherParser_ESC_LITERAL => {
                        {
                            /*InvokeRule symbol*/
                            recog.base.set_state(858);
                            recog.symbol()?;
                        }
                    }

                    CypherParser_DIGIT => {
                        {
                            /*InvokeRule numLit*/
                            recog.base.set_state(859);
                            recog.numLit()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- literal ----------------
pub type LiteralContextAll<'input> = LiteralContext<'input>;

pub type LiteralContext<'input> = BaseParserRuleContext<'input, LiteralContextExt<'input>>;

#[derive(Clone)]
pub struct LiteralContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for LiteralContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for LiteralContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_literal(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_literal(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for LiteralContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_literal(self);
    }
}

impl<'input> CustomRuleContext<'input> for LiteralContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_literal
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_literal }
}
antlr4rust::tid! {LiteralContextExt<'a>}

impl<'input> LiteralContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<LiteralContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            LiteralContextExt { ph: PhantomData },
        ))
    }
}

pub trait LiteralContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<LiteralContextExt<'input>>
{
    fn boolLit(&self) -> Option<Rc<BoolLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn numLit(&self) -> Option<Rc<NumLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token NULL_W
    /// Returns `None` if there is no child corresponding to token NULL_W
    fn NULL_W(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NULL_W, 0)
    }
    fn stringLit(&self) -> Option<Rc<StringLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn charLit(&self) -> Option<Rc<CharLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn listLit(&self) -> Option<Rc<ListLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn mapLit(&self) -> Option<Rc<MapLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> LiteralContextAttrs<'input> for LiteralContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn literal(&mut self) -> Result<Rc<LiteralContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = LiteralContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 168, RULE_literal);
        let mut _localctx: Rc<LiteralContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(869);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_FALSE | CypherParser_TRUE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule boolLit*/
                        recog.base.set_state(862);
                        recog.boolLit()?;
                    }
                }

                CypherParser_DIGIT => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule numLit*/
                        recog.base.set_state(863);
                        recog.numLit()?;
                    }
                }

                CypherParser_NULL_W => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(864);
                        recog
                            .base
                            .match_token(CypherParser_NULL_W, &mut recog.err_handler)?;
                    }
                }

                CypherParser_STRING_LITERAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule stringLit*/
                        recog.base.set_state(865);
                        recog.stringLit()?;
                    }
                }

                CypherParser_CHAR_LITERAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule charLit*/
                        recog.base.set_state(866);
                        recog.charLit()?;
                    }
                }

                CypherParser_LBRACK => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        /*InvokeRule listLit*/
                        recog.base.set_state(867);
                        recog.listLit()?;
                    }
                }

                CypherParser_LBRACE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        /*InvokeRule mapLit*/
                        recog.base.set_state(868);
                        recog.mapLit()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- rangeLit ----------------
pub type RangeLitContextAll<'input> = RangeLitContext<'input>;

pub type RangeLitContext<'input> = BaseParserRuleContext<'input, RangeLitContextExt<'input>>;

#[derive(Clone)]
pub struct RangeLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for RangeLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for RangeLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_rangeLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_rangeLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for RangeLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_rangeLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for RangeLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_rangeLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_rangeLit }
}
antlr4rust::tid! {RangeLitContextExt<'a>}

impl<'input> RangeLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<RangeLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            RangeLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait RangeLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<RangeLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token MULT
    /// Returns `None` if there is no child corresponding to token MULT
    fn MULT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MULT, 0)
    }
    fn numLit_all(&self) -> Vec<Rc<NumLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn numLit(&self, i: usize) -> Option<Rc<NumLitContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves first TerminalNode corresponding to token RANGE
    /// Returns `None` if there is no child corresponding to token RANGE
    fn RANGE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RANGE, 0)
    }
}

impl<'input> RangeLitContextAttrs<'input> for RangeLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn rangeLit(&mut self) -> Result<Rc<RangeLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = RangeLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 170, RULE_rangeLit);
        let mut _localctx: Rc<RangeLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(871);
                recog
                    .base
                    .match_token(CypherParser_MULT, &mut recog.err_handler)?;

                recog.base.set_state(873);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DIGIT {
                    {
                        /*InvokeRule numLit*/
                        recog.base.set_state(872);
                        recog.numLit()?;
                    }
                }

                recog.base.set_state(879);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_RANGE {
                    {
                        recog.base.set_state(875);
                        recog
                            .base
                            .match_token(CypherParser_RANGE, &mut recog.err_handler)?;

                        recog.base.set_state(877);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_DIGIT {
                            {
                                /*InvokeRule numLit*/
                                recog.base.set_state(876);
                                recog.numLit()?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- boolLit ----------------
pub type BoolLitContextAll<'input> = BoolLitContext<'input>;

pub type BoolLitContext<'input> = BaseParserRuleContext<'input, BoolLitContextExt<'input>>;

#[derive(Clone)]
pub struct BoolLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for BoolLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for BoolLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_boolLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_boolLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for BoolLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_boolLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for BoolLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_boolLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_boolLit }
}
antlr4rust::tid! {BoolLitContextExt<'a>}

impl<'input> BoolLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<BoolLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            BoolLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait BoolLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<BoolLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token TRUE
    /// Returns `None` if there is no child corresponding to token TRUE
    fn TRUE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_TRUE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token FALSE
    /// Returns `None` if there is no child corresponding to token FALSE
    fn FALSE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_FALSE, 0)
    }
}

impl<'input> BoolLitContextAttrs<'input> for BoolLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn boolLit(&mut self) -> Result<Rc<BoolLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = BoolLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 172, RULE_boolLit);
        let mut _localctx: Rc<BoolLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(881);
                _la = recog.base.input.la(1);
                if { !(_la == CypherParser_FALSE || _la == CypherParser_TRUE) } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- numLit ----------------
pub type NumLitContextAll<'input> = NumLitContext<'input>;

pub type NumLitContext<'input> = BaseParserRuleContext<'input, NumLitContextExt<'input>>;

#[derive(Clone)]
pub struct NumLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NumLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for NumLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_numLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_numLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NumLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_numLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for NumLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_numLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_numLit }
}
antlr4rust::tid! {NumLitContextExt<'a>}

impl<'input> NumLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NumLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NumLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait NumLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NumLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token DIGIT
    /// Returns `None` if there is no child corresponding to token DIGIT
    fn DIGIT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DIGIT, 0)
    }
}

impl<'input> NumLitContextAttrs<'input> for NumLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn numLit(&mut self) -> Result<Rc<NumLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = NumLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 174, RULE_numLit);
        let mut _localctx: Rc<NumLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(883);
                recog
                    .base
                    .match_token(CypherParser_DIGIT, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- stringLit ----------------
pub type StringLitContextAll<'input> = StringLitContext<'input>;

pub type StringLitContext<'input> = BaseParserRuleContext<'input, StringLitContextExt<'input>>;

#[derive(Clone)]
pub struct StringLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for StringLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for StringLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_stringLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_stringLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for StringLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_stringLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for StringLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_stringLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_stringLit }
}
antlr4rust::tid! {StringLitContextExt<'a>}

impl<'input> StringLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<StringLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            StringLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait StringLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<StringLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token STRING_LITERAL
    /// Returns `None` if there is no child corresponding to token STRING_LITERAL
    fn STRING_LITERAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STRING_LITERAL, 0)
    }
}

impl<'input> StringLitContextAttrs<'input> for StringLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn stringLit(&mut self) -> Result<Rc<StringLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = StringLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 176, RULE_stringLit);
        let mut _localctx: Rc<StringLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(885);
                recog
                    .base
                    .match_token(CypherParser_STRING_LITERAL, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- charLit ----------------
pub type CharLitContextAll<'input> = CharLitContext<'input>;

pub type CharLitContext<'input> = BaseParserRuleContext<'input, CharLitContextExt<'input>>;

#[derive(Clone)]
pub struct CharLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for CharLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for CharLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_charLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_charLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for CharLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_charLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for CharLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_charLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_charLit }
}
antlr4rust::tid! {CharLitContextExt<'a>}

impl<'input> CharLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<CharLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            CharLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait CharLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<CharLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token CHAR_LITERAL
    /// Returns `None` if there is no child corresponding to token CHAR_LITERAL
    fn CHAR_LITERAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CHAR_LITERAL, 0)
    }
}

impl<'input> CharLitContextAttrs<'input> for CharLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn charLit(&mut self) -> Result<Rc<CharLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = CharLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 178, RULE_charLit);
        let mut _localctx: Rc<CharLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(887);
                recog
                    .base
                    .match_token(CypherParser_CHAR_LITERAL, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- listLit ----------------
pub type ListLitContextAll<'input> = ListLitContext<'input>;

pub type ListLitContext<'input> = BaseParserRuleContext<'input, ListLitContextExt<'input>>;

#[derive(Clone)]
pub struct ListLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ListLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ListLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_listLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_listLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ListLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_listLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for ListLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_listLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_listLit }
}
antlr4rust::tid! {ListLitContextExt<'a>}

impl<'input> ListLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ListLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ListLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait ListLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ListLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACK
    /// Returns `None` if there is no child corresponding to token LBRACK
    fn LBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACK, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACK
    /// Returns `None` if there is no child corresponding to token RBRACK
    fn RBRACK(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACK, 0)
    }
    fn expressionChain(&self) -> Option<Rc<ExpressionChainContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> ListLitContextAttrs<'input> for ListLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn listLit(&mut self) -> Result<Rc<ListLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ListLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 180, RULE_listLit);
        let mut _localctx: Rc<ListLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(889);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(891);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(890);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(893);
                recog
                    .base
                    .match_token(CypherParser_RBRACK, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- mapLit ----------------
pub type MapLitContextAll<'input> = MapLitContext<'input>;

pub type MapLitContext<'input> = BaseParserRuleContext<'input, MapLitContextExt<'input>>;

#[derive(Clone)]
pub struct MapLitContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MapLitContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MapLitContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_mapLit(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_mapLit(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MapLitContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_mapLit(self);
    }
}

impl<'input> CustomRuleContext<'input> for MapLitContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_mapLit
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_mapLit }
}
antlr4rust::tid! {MapLitContextExt<'a>}

impl<'input> MapLitContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MapLitContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MapLitContextExt { ph: PhantomData },
        ))
    }
}

pub trait MapLitContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MapLitContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token LBRACE
    /// Returns `None` if there is no child corresponding to token LBRACE
    fn LBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LBRACE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RBRACE
    /// Returns `None` if there is no child corresponding to token RBRACE
    fn RBRACE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RBRACE, 0)
    }
    fn mapPair_all(&self) -> Vec<Rc<MapPairContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn mapPair(&self, i: usize) -> Option<Rc<MapPairContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
    /// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
    fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    /// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
    /// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
    fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COMMA, i)
    }
}

impl<'input> MapLitContextAttrs<'input> for MapLitContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn mapLit(&mut self) -> Result<Rc<MapLitContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MapLitContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 182, RULE_mapLit);
        let mut _localctx: Rc<MapLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(895);
                recog
                    .base
                    .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                recog.base.set_state(904);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 4294967295) != 0)
                    || (((_la - 62) & !0x3f) == 0 && ((1usize << (_la - 62)) & 4294967295) != 0)
                {
                    {
                        /*InvokeRule mapPair*/
                        recog.base.set_state(896);
                        recog.mapPair()?;

                        recog.base.set_state(901);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        while _la == CypherParser_COMMA {
                            {
                                {
                                    recog.base.set_state(897);
                                    recog
                                        .base
                                        .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                                    /*InvokeRule mapPair*/
                                    recog.base.set_state(898);
                                    recog.mapPair()?;
                                }
                            }
                            recog.base.set_state(903);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                        }
                    }
                }

                recog.base.set_state(906);
                recog
                    .base
                    .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- mapPair ----------------
pub type MapPairContextAll<'input> = MapPairContext<'input>;

pub type MapPairContext<'input> = BaseParserRuleContext<'input, MapPairContextExt<'input>>;

#[derive(Clone)]
pub struct MapPairContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for MapPairContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for MapPairContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_mapPair(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_mapPair(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for MapPairContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_mapPair(self);
    }
}

impl<'input> CustomRuleContext<'input> for MapPairContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_mapPair
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_mapPair }
}
antlr4rust::tid! {MapPairContextExt<'a>}

impl<'input> MapPairContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<MapPairContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            MapPairContextExt { ph: PhantomData },
        ))
    }
}

pub trait MapPairContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<MapPairContextExt<'input>>
{
    fn name(&self) -> Option<Rc<NameContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token COLON
    /// Returns `None` if there is no child corresponding to token COLON
    fn COLON(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COLON, 0)
    }
    fn expression(&self) -> Option<Rc<ExpressionContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> MapPairContextAttrs<'input> for MapPairContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn mapPair(&mut self) -> Result<Rc<MapPairContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = MapPairContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 184, RULE_mapPair);
        let mut _localctx: Rc<MapPairContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule name*/
                recog.base.set_state(908);
                recog.name()?;

                recog.base.set_state(909);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(910);
                recog.expression()?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- name ----------------
pub type NameContextAll<'input> = NameContext<'input>;

pub type NameContext<'input> = BaseParserRuleContext<'input, NameContextExt<'input>>;

#[derive(Clone)]
pub struct NameContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for NameContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for NameContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_name(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_name(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for NameContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_name(self);
    }
}

impl<'input> CustomRuleContext<'input> for NameContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_name
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_name }
}
antlr4rust::tid! {NameContextExt<'a>}

impl<'input> NameContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<NameContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            NameContextExt { ph: PhantomData },
        ))
    }
}

pub trait NameContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<NameContextExt<'input>>
{
    fn symbol(&self) -> Option<Rc<SymbolContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    fn reservedWord(&self) -> Option<Rc<ReservedWordContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
}

impl<'input> NameContextAttrs<'input> for NameContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn name(&mut self) -> Result<Rc<NameContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = NameContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 186, RULE_name);
        let mut _localctx: Rc<NameContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(914);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_FILTER
                | CypherParser_EXTRACT
                | CypherParser_COUNT
                | CypherParser_ANY
                | CypherParser_NONE
                | CypherParser_SINGLE
                | CypherParser_ID
                | CypherParser_ESC_LITERAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(912);
                        recog.symbol()?;
                    }
                }

                CypherParser_ALL
                | CypherParser_ASC
                | CypherParser_ASCENDING
                | CypherParser_BY
                | CypherParser_CREATE
                | CypherParser_DELETE
                | CypherParser_DESC
                | CypherParser_DESCENDING
                | CypherParser_DETACH
                | CypherParser_EXISTS
                | CypherParser_EXPLAIN
                | CypherParser_LIMIT
                | CypherParser_MATCH
                | CypherParser_MERGE
                | CypherParser_ON
                | CypherParser_OPTIONAL
                | CypherParser_ORDER
                | CypherParser_REMOVE
                | CypherParser_RETURN
                | CypherParser_SET
                | CypherParser_SKIP_W
                | CypherParser_WHERE
                | CypherParser_WITH
                | CypherParser_UNION
                | CypherParser_UNWIND
                | CypherParser_AND
                | CypherParser_AS
                | CypherParser_CONTAINS
                | CypherParser_DISTINCT
                | CypherParser_ENDS
                | CypherParser_IN
                | CypherParser_INDEX
                | CypherParser_IS
                | CypherParser_NOT
                | CypherParser_OR
                | CypherParser_STARTS
                | CypherParser_XOR
                | CypherParser_SHORTEST_PATH
                | CypherParser_FALSE
                | CypherParser_TRUE
                | CypherParser_NULL_W
                | CypherParser_CONSTRAINT
                | CypherParser_DO
                | CypherParser_FOR
                | CypherParser_REQUIRE
                | CypherParser_UNIQUE
                | CypherParser_CASE
                | CypherParser_WHEN
                | CypherParser_THEN
                | CypherParser_ELSE
                | CypherParser_END
                | CypherParser_MANDATORY
                | CypherParser_SCALAR
                | CypherParser_OF
                | CypherParser_ADD
                | CypherParser_DROP => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule reservedWord*/
                        recog.base.set_state(913);
                        recog.reservedWord()?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- symbol ----------------
pub type SymbolContextAll<'input> = SymbolContext<'input>;

pub type SymbolContext<'input> = BaseParserRuleContext<'input, SymbolContextExt<'input>>;

#[derive(Clone)]
pub struct SymbolContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for SymbolContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for SymbolContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_symbol(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_symbol(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for SymbolContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_symbol(self);
    }
}

impl<'input> CustomRuleContext<'input> for SymbolContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_symbol
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_symbol }
}
antlr4rust::tid! {SymbolContextExt<'a>}

impl<'input> SymbolContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<SymbolContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            SymbolContextExt { ph: PhantomData },
        ))
    }
}

pub trait SymbolContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<SymbolContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ESC_LITERAL
    /// Returns `None` if there is no child corresponding to token ESC_LITERAL
    fn ESC_LITERAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ESC_LITERAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ID
    /// Returns `None` if there is no child corresponding to token ID
    fn ID(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ID, 0)
    }
    /// Retrieves first TerminalNode corresponding to token COUNT
    /// Returns `None` if there is no child corresponding to token COUNT
    fn COUNT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_COUNT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token FILTER
    /// Returns `None` if there is no child corresponding to token FILTER
    fn FILTER(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_FILTER, 0)
    }
    /// Retrieves first TerminalNode corresponding to token EXTRACT
    /// Returns `None` if there is no child corresponding to token EXTRACT
    fn EXTRACT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EXTRACT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ANY
    /// Returns `None` if there is no child corresponding to token ANY
    fn ANY(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ANY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NONE
    /// Returns `None` if there is no child corresponding to token NONE
    fn NONE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NONE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SINGLE
    /// Returns `None` if there is no child corresponding to token SINGLE
    fn SINGLE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SINGLE, 0)
    }
}

impl<'input> SymbolContextAttrs<'input> for SymbolContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn symbol(&mut self) -> Result<Rc<SymbolContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = SymbolContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 188, RULE_symbol);
        let mut _localctx: Rc<SymbolContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(916);
                _la = recog.base.input.la(1);
                if {
                    !((((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                        || _la == CypherParser_ID
                        || _la == CypherParser_ESC_LITERAL)
                } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
//------------------- reservedWord ----------------
pub type ReservedWordContextAll<'input> = ReservedWordContext<'input>;

pub type ReservedWordContext<'input> =
    BaseParserRuleContext<'input, ReservedWordContextExt<'input>>;

#[derive(Clone)]
pub struct ReservedWordContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> CypherParserContext<'input> for ReservedWordContext<'input> {}

impl<'input, 'a> Listenable<dyn CypherParserListener<'input> + 'a> for ReservedWordContext<'input> {
    fn enter(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_reservedWord(self);
        Ok(())
    }
    fn exit(
        &self,
        listener: &mut (dyn CypherParserListener<'input> + 'a),
    ) -> Result<(), ANTLRError> {
        listener.exit_reservedWord(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input, 'a> Visitable<dyn CypherParserVisitor<'input> + 'a> for ReservedWordContext<'input> {
    fn accept(&self, visitor: &mut (dyn CypherParserVisitor<'input> + 'a)) {
        visitor.visit_reservedWord(self);
    }
}

impl<'input> CustomRuleContext<'input> for ReservedWordContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = CypherParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_reservedWord
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_reservedWord }
}
antlr4rust::tid! {ReservedWordContextExt<'a>}

impl<'input> ReservedWordContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn CypherParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ReservedWordContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ReservedWordContextExt { ph: PhantomData },
        ))
    }
}

pub trait ReservedWordContextAttrs<'input>:
    CypherParserContext<'input> + BorrowMut<ReservedWordContextExt<'input>>
{
    /// Retrieves first TerminalNode corresponding to token ALL
    /// Returns `None` if there is no child corresponding to token ALL
    fn ALL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ALL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ASC
    /// Returns `None` if there is no child corresponding to token ASC
    fn ASC(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASC, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ASCENDING
    /// Returns `None` if there is no child corresponding to token ASCENDING
    fn ASCENDING(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ASCENDING, 0)
    }
    /// Retrieves first TerminalNode corresponding to token BY
    /// Returns `None` if there is no child corresponding to token BY
    fn BY(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_BY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CREATE
    /// Returns `None` if there is no child corresponding to token CREATE
    fn CREATE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CREATE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DELETE
    /// Returns `None` if there is no child corresponding to token DELETE
    fn DELETE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DELETE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DESC
    /// Returns `None` if there is no child corresponding to token DESC
    fn DESC(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DESC, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DESCENDING
    /// Returns `None` if there is no child corresponding to token DESCENDING
    fn DESCENDING(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DESCENDING, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DETACH
    /// Returns `None` if there is no child corresponding to token DETACH
    fn DETACH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DETACH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token EXISTS
    /// Returns `None` if there is no child corresponding to token EXISTS
    fn EXISTS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EXISTS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token EXPLAIN
    /// Returns `None` if there is no child corresponding to token EXPLAIN
    fn EXPLAIN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_EXPLAIN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token LIMIT
    /// Returns `None` if there is no child corresponding to token LIMIT
    fn LIMIT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_LIMIT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MATCH
    /// Returns `None` if there is no child corresponding to token MATCH
    fn MATCH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MATCH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MERGE
    /// Returns `None` if there is no child corresponding to token MERGE
    fn MERGE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MERGE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ON
    /// Returns `None` if there is no child corresponding to token ON
    fn ON(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ON, 0)
    }
    /// Retrieves first TerminalNode corresponding to token OPTIONAL
    /// Returns `None` if there is no child corresponding to token OPTIONAL
    fn OPTIONAL(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_OPTIONAL, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ORDER
    /// Returns `None` if there is no child corresponding to token ORDER
    fn ORDER(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ORDER, 0)
    }
    /// Retrieves first TerminalNode corresponding to token REMOVE
    /// Returns `None` if there is no child corresponding to token REMOVE
    fn REMOVE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_REMOVE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token RETURN
    /// Returns `None` if there is no child corresponding to token RETURN
    fn RETURN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_RETURN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SET
    /// Returns `None` if there is no child corresponding to token SET
    fn SET(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SET, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SKIP_W
    /// Returns `None` if there is no child corresponding to token SKIP_W
    fn SKIP_W(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SKIP_W, 0)
    }
    /// Retrieves first TerminalNode corresponding to token WHERE
    /// Returns `None` if there is no child corresponding to token WHERE
    fn WHERE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WHERE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token WITH
    /// Returns `None` if there is no child corresponding to token WITH
    fn WITH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WITH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token UNION
    /// Returns `None` if there is no child corresponding to token UNION
    fn UNION(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNION, 0)
    }
    /// Retrieves first TerminalNode corresponding to token UNWIND
    /// Returns `None` if there is no child corresponding to token UNWIND
    fn UNWIND(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNWIND, 0)
    }
    /// Retrieves first TerminalNode corresponding to token AND
    /// Returns `None` if there is no child corresponding to token AND
    fn AND(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AND, 0)
    }
    /// Retrieves first TerminalNode corresponding to token AS
    /// Returns `None` if there is no child corresponding to token AS
    fn AS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_AS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CONTAINS
    /// Returns `None` if there is no child corresponding to token CONTAINS
    fn CONTAINS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CONTAINS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DISTINCT
    /// Returns `None` if there is no child corresponding to token DISTINCT
    fn DISTINCT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DISTINCT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ENDS
    /// Returns `None` if there is no child corresponding to token ENDS
    fn ENDS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ENDS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token IN
    /// Returns `None` if there is no child corresponding to token IN
    fn IN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_IN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token INDEX
    /// Returns `None` if there is no child corresponding to token INDEX
    fn INDEX(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_INDEX, 0)
    }
    /// Retrieves first TerminalNode corresponding to token IS
    /// Returns `None` if there is no child corresponding to token IS
    fn IS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_IS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NOT
    /// Returns `None` if there is no child corresponding to token NOT
    fn NOT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NOT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token OR
    /// Returns `None` if there is no child corresponding to token OR
    fn OR(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_OR, 0)
    }
    /// Retrieves first TerminalNode corresponding to token STARTS
    /// Returns `None` if there is no child corresponding to token STARTS
    fn STARTS(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_STARTS, 0)
    }
    /// Retrieves first TerminalNode corresponding to token XOR
    /// Returns `None` if there is no child corresponding to token XOR
    fn XOR(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_XOR, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SHORTEST_PATH
    /// Returns `None` if there is no child corresponding to token SHORTEST_PATH
    fn SHORTEST_PATH(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SHORTEST_PATH, 0)
    }
    /// Retrieves first TerminalNode corresponding to token FALSE
    /// Returns `None` if there is no child corresponding to token FALSE
    fn FALSE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_FALSE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token TRUE
    /// Returns `None` if there is no child corresponding to token TRUE
    fn TRUE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_TRUE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token NULL_W
    /// Returns `None` if there is no child corresponding to token NULL_W
    fn NULL_W(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_NULL_W, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CONSTRAINT
    /// Returns `None` if there is no child corresponding to token CONSTRAINT
    fn CONSTRAINT(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CONSTRAINT, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DO
    /// Returns `None` if there is no child corresponding to token DO
    fn DO(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DO, 0)
    }
    /// Retrieves first TerminalNode corresponding to token FOR
    /// Returns `None` if there is no child corresponding to token FOR
    fn FOR(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_FOR, 0)
    }
    /// Retrieves first TerminalNode corresponding to token REQUIRE
    /// Returns `None` if there is no child corresponding to token REQUIRE
    fn REQUIRE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_REQUIRE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token UNIQUE
    /// Returns `None` if there is no child corresponding to token UNIQUE
    fn UNIQUE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_UNIQUE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token CASE
    /// Returns `None` if there is no child corresponding to token CASE
    fn CASE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_CASE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token WHEN
    /// Returns `None` if there is no child corresponding to token WHEN
    fn WHEN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_WHEN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token THEN
    /// Returns `None` if there is no child corresponding to token THEN
    fn THEN(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_THEN, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ELSE
    /// Returns `None` if there is no child corresponding to token ELSE
    fn ELSE(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ELSE, 0)
    }
    /// Retrieves first TerminalNode corresponding to token END
    /// Returns `None` if there is no child corresponding to token END
    fn END(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_END, 0)
    }
    /// Retrieves first TerminalNode corresponding to token MANDATORY
    /// Returns `None` if there is no child corresponding to token MANDATORY
    fn MANDATORY(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_MANDATORY, 0)
    }
    /// Retrieves first TerminalNode corresponding to token SCALAR
    /// Returns `None` if there is no child corresponding to token SCALAR
    fn SCALAR(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_SCALAR, 0)
    }
    /// Retrieves first TerminalNode corresponding to token OF
    /// Returns `None` if there is no child corresponding to token OF
    fn OF(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_OF, 0)
    }
    /// Retrieves first TerminalNode corresponding to token ADD
    /// Returns `None` if there is no child corresponding to token ADD
    fn ADD(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_ADD, 0)
    }
    /// Retrieves first TerminalNode corresponding to token DROP
    /// Returns `None` if there is no child corresponding to token DROP
    fn DROP(&self) -> Option<Rc<TerminalNode<'input, CypherParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(CypherParser_DROP, 0)
    }
}

impl<'input> ReservedWordContextAttrs<'input> for ReservedWordContext<'input> {}

impl<'input, I> CypherParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn reservedWord(&mut self) -> Result<Rc<ReservedWordContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ReservedWordContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog
            .base
            .enter_rule(_localctx.clone(), 190, RULE_reservedWord);
        let mut _localctx: Rc<ReservedWordContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(918);
                _la = recog.base.input.la(1);
                if {
                    !((((_la - 36) & !0x3f) == 0 && ((1usize << (_la - 36)) & 4294967295) != 0)
                        || (((_la - 68) & !0x3f) == 0 && ((1usize << (_la - 68)) & 16777215) != 0))
                } {
                    recog.err_handler.recover_inline(&mut recog.base)?;
                } else {
                    if recog.base.input.la(1) == TOKEN_EOF {
                        recog.base.matched_eof = true
                    };
                    recog.err_handler.report_match(&mut recog.base);
                    recog.base.consume(&mut recog.err_handler);
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
lazy_static! {
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(_ATN.clone(), _ATN.get_decision_state(i), i).into())
        }
        Arc::new(dfa)
    };
    static ref _serializedATN: Vec<i32> = vec![
        4, 1, 101, 921, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 4, 2, 5, 7, 5, 2,
        6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2,
        13, 7, 13, 2, 14, 7, 14, 2, 15, 7, 15, 2, 16, 7, 16, 2, 17, 7, 17, 2, 18, 7, 18, 2, 19, 7,
        19, 2, 20, 7, 20, 2, 21, 7, 21, 2, 22, 7, 22, 2, 23, 7, 23, 2, 24, 7, 24, 2, 25, 7, 25, 2,
        26, 7, 26, 2, 27, 7, 27, 2, 28, 7, 28, 2, 29, 7, 29, 2, 30, 7, 30, 2, 31, 7, 31, 2, 32, 7,
        32, 2, 33, 7, 33, 2, 34, 7, 34, 2, 35, 7, 35, 2, 36, 7, 36, 2, 37, 7, 37, 2, 38, 7, 38, 2,
        39, 7, 39, 2, 40, 7, 40, 2, 41, 7, 41, 2, 42, 7, 42, 2, 43, 7, 43, 2, 44, 7, 44, 2, 45, 7,
        45, 2, 46, 7, 46, 2, 47, 7, 47, 2, 48, 7, 48, 2, 49, 7, 49, 2, 50, 7, 50, 2, 51, 7, 51, 2,
        52, 7, 52, 2, 53, 7, 53, 2, 54, 7, 54, 2, 55, 7, 55, 2, 56, 7, 56, 2, 57, 7, 57, 2, 58, 7,
        58, 2, 59, 7, 59, 2, 60, 7, 60, 2, 61, 7, 61, 2, 62, 7, 62, 2, 63, 7, 63, 2, 64, 7, 64, 2,
        65, 7, 65, 2, 66, 7, 66, 2, 67, 7, 67, 2, 68, 7, 68, 2, 69, 7, 69, 2, 70, 7, 70, 2, 71, 7,
        71, 2, 72, 7, 72, 2, 73, 7, 73, 2, 74, 7, 74, 2, 75, 7, 75, 2, 76, 7, 76, 2, 77, 7, 77, 2,
        78, 7, 78, 2, 79, 7, 79, 2, 80, 7, 80, 2, 81, 7, 81, 2, 82, 7, 82, 2, 83, 7, 83, 2, 84, 7,
        84, 2, 85, 7, 85, 2, 86, 7, 86, 2, 87, 7, 87, 2, 88, 7, 88, 2, 89, 7, 89, 2, 90, 7, 90, 2,
        91, 7, 91, 2, 92, 7, 92, 2, 93, 7, 93, 2, 94, 7, 94, 2, 95, 7, 95, 1, 0, 1, 0, 3, 0, 195,
        8, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 5, 1, 202, 8, 1, 10, 1, 12, 1, 205, 9, 1, 1, 1, 1, 1,
        1, 2, 1, 2, 1, 2, 1, 2, 3, 2, 213, 8, 2, 1, 3, 1, 3, 1, 3, 3, 3, 218, 8, 3, 1, 4, 1, 4, 1,
        4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 1, 4, 3, 4, 229, 8, 4, 1, 5, 1, 5, 5, 5, 233, 8, 5, 10, 5,
        12, 5, 236, 9, 5, 1, 6, 1, 6, 3, 6, 240, 8, 6, 1, 7, 1, 7, 1, 7, 3, 7, 245, 8, 7, 1, 7, 1,
        7, 1, 7, 3, 7, 250, 8, 7, 3, 7, 252, 8, 7, 1, 8, 1, 8, 1, 8, 1, 9, 1, 9, 1, 9, 3, 9, 260,
        8, 9, 1, 10, 1, 10, 1, 10, 1, 11, 1, 11, 1, 11, 1, 12, 3, 12, 269, 8, 12, 1, 12, 1, 12, 3,
        12, 273, 8, 12, 1, 12, 3, 12, 276, 8, 12, 1, 12, 3, 12, 279, 8, 12, 1, 13, 1, 13, 3, 13,
        283, 8, 13, 1, 13, 1, 13, 5, 13, 287, 8, 13, 10, 13, 12, 13, 290, 9, 13, 1, 14, 1, 14, 1,
        14, 3, 14, 295, 8, 14, 1, 15, 1, 15, 3, 15, 299, 8, 15, 1, 16, 1, 16, 1, 16, 1, 16, 1, 16,
        5, 16, 306, 8, 16, 10, 16, 12, 16, 309, 9, 16, 1, 17, 5, 17, 312, 8, 17, 10, 17, 12, 17,
        315, 9, 17, 1, 17, 1, 17, 4, 17, 319, 8, 17, 11, 17, 12, 17, 320, 1, 17, 3, 17, 324, 8, 17,
        3, 17, 326, 8, 17, 1, 18, 5, 18, 329, 8, 18, 10, 18, 12, 18, 332, 9, 18, 1, 18, 1, 18, 5,
        18, 336, 8, 18, 10, 18, 12, 18, 339, 9, 18, 1, 18, 4, 18, 342, 8, 18, 11, 18, 12, 18, 343,
        1, 18, 1, 18, 1, 19, 3, 19, 349, 8, 19, 1, 19, 1, 19, 1, 19, 1, 20, 1, 20, 1, 20, 1, 20, 1,
        20, 1, 21, 1, 21, 1, 21, 3, 21, 362, 8, 21, 1, 22, 1, 22, 1, 22, 1, 22, 1, 22, 3, 22, 369,
        8, 22, 1, 23, 3, 23, 372, 8, 23, 1, 23, 1, 23, 1, 23, 1, 24, 1, 24, 1, 24, 1, 24, 5, 24,
        381, 8, 24, 10, 24, 12, 24, 384, 9, 24, 1, 25, 1, 25, 1, 25, 1, 25, 3, 25, 390, 8, 25, 1,
        26, 1, 26, 1, 26, 1, 26, 1, 26, 3, 26, 397, 8, 26, 1, 27, 1, 27, 3, 27, 401, 8, 27, 1, 27,
        1, 27, 1, 28, 1, 28, 1, 28, 5, 28, 408, 8, 28, 10, 28, 12, 28, 411, 9, 28, 1, 28, 3, 28,
        414, 8, 28, 1, 29, 1, 29, 1, 29, 3, 29, 419, 8, 29, 1, 29, 1, 29, 1, 30, 1, 30, 1, 30, 5,
        30, 426, 8, 30, 10, 30, 12, 30, 429, 9, 30, 1, 31, 1, 31, 1, 31, 1, 31, 1, 32, 1, 32, 1,
        32, 1, 32, 5, 32, 439, 8, 32, 10, 32, 12, 32, 442, 9, 32, 1, 33, 1, 33, 1, 33, 1, 33, 1,
        33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 3, 33, 455, 8, 33, 1, 34, 1, 34, 4, 34, 459,
        8, 34, 11, 34, 12, 34, 460, 1, 35, 1, 35, 1, 35, 1, 36, 1, 36, 3, 36, 468, 8, 36, 1, 37, 1,
        37, 1, 37, 1, 38, 1, 38, 1, 38, 5, 38, 476, 8, 38, 10, 38, 12, 38, 479, 9, 38, 1, 39, 1,
        39, 1, 39, 5, 39, 484, 8, 39, 10, 39, 12, 39, 487, 9, 39, 1, 40, 1, 40, 1, 40, 5, 40, 492,
        8, 40, 10, 40, 12, 40, 495, 9, 40, 1, 41, 1, 41, 1, 41, 5, 41, 500, 8, 41, 10, 41, 12, 41,
        503, 9, 41, 1, 42, 5, 42, 506, 8, 42, 10, 42, 12, 42, 509, 9, 42, 1, 42, 1, 42, 1, 43, 1,
        43, 1, 43, 1, 43, 5, 43, 517, 8, 43, 10, 43, 12, 43, 520, 9, 43, 1, 44, 1, 44, 1, 45, 1,
        45, 1, 45, 5, 45, 527, 8, 45, 10, 45, 12, 45, 530, 9, 45, 1, 46, 1, 46, 1, 46, 5, 46, 535,
        8, 46, 10, 46, 12, 46, 538, 9, 46, 1, 47, 1, 47, 1, 47, 5, 47, 543, 8, 47, 10, 47, 12, 47,
        546, 9, 47, 1, 48, 3, 48, 549, 8, 48, 1, 48, 1, 48, 1, 49, 1, 49, 1, 49, 1, 49, 5, 49, 557,
        8, 49, 10, 49, 12, 49, 560, 9, 49, 1, 50, 1, 50, 1, 50, 1, 50, 3, 50, 566, 8, 50, 1, 50, 1,
        50, 3, 50, 570, 8, 50, 1, 50, 3, 50, 573, 8, 50, 1, 50, 3, 50, 576, 8, 50, 1, 51, 1, 51, 1,
        51, 1, 52, 1, 52, 1, 52, 1, 52, 1, 52, 3, 52, 586, 8, 52, 1, 53, 1, 53, 3, 53, 590, 8, 53,
        1, 53, 1, 53, 1, 54, 1, 54, 3, 54, 596, 8, 54, 1, 55, 1, 55, 1, 55, 5, 55, 601, 8, 55, 10,
        55, 12, 55, 604, 9, 55, 1, 56, 1, 56, 1, 56, 3, 56, 609, 8, 56, 1, 56, 1, 56, 3, 56, 613,
        8, 56, 1, 57, 1, 57, 1, 57, 1, 57, 1, 57, 1, 58, 1, 58, 1, 58, 5, 58, 623, 8, 58, 10, 58,
        12, 58, 626, 9, 58, 1, 58, 1, 58, 1, 58, 1, 58, 3, 58, 632, 8, 58, 3, 58, 634, 8, 58, 1,
        59, 1, 59, 1, 59, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 61, 1, 61, 1, 61, 1, 61, 1,
        61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1,
        61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 1, 61, 3, 61, 670, 8, 61, 1, 62, 1, 62, 1,
        63, 1, 63, 3, 63, 676, 8, 63, 1, 64, 1, 64, 3, 64, 680, 8, 64, 1, 64, 3, 64, 683, 8, 64, 1,
        64, 3, 64, 686, 8, 64, 1, 64, 1, 64, 1, 65, 1, 65, 1, 65, 1, 65, 1, 65, 1, 65, 1, 65, 1,
        65, 1, 65, 1, 65, 1, 65, 1, 65, 3, 65, 702, 8, 65, 1, 66, 1, 66, 1, 66, 1, 67, 1, 67, 1,
        67, 3, 67, 710, 8, 67, 1, 67, 1, 67, 3, 67, 714, 8, 67, 1, 67, 1, 67, 3, 67, 718, 8, 67, 1,
        67, 1, 67, 3, 67, 722, 8, 67, 3, 67, 724, 8, 67, 1, 68, 1, 68, 3, 68, 728, 8, 68, 1, 68, 3,
        68, 731, 8, 68, 1, 68, 3, 68, 734, 8, 68, 1, 68, 3, 68, 737, 8, 68, 1, 68, 1, 68, 1, 69, 1,
        69, 1, 69, 1, 69, 3, 69, 745, 8, 69, 1, 69, 5, 69, 748, 8, 69, 10, 69, 12, 69, 751, 9, 69,
        1, 70, 1, 70, 3, 70, 755, 8, 70, 1, 70, 1, 70, 1, 71, 1, 71, 1, 71, 1, 71, 3, 71, 763, 8,
        71, 1, 71, 1, 71, 1, 72, 1, 72, 1, 72, 5, 72, 770, 8, 72, 10, 72, 12, 72, 773, 9, 72, 1,
        73, 1, 73, 1, 73, 3, 73, 778, 8, 73, 1, 73, 3, 73, 781, 8, 73, 1, 73, 1, 73, 1, 74, 1, 74,
        1, 74, 1, 74, 1, 75, 1, 75, 1, 75, 1, 75, 1, 75, 1, 76, 1, 76, 3, 76, 796, 8, 76, 1, 76, 1,
        76, 3, 76, 800, 8, 76, 1, 76, 1, 76, 1, 76, 1, 76, 1, 77, 1, 77, 4, 77, 808, 8, 77, 11, 77,
        12, 77, 809, 1, 78, 1, 78, 1, 78, 1, 78, 3, 78, 816, 8, 78, 1, 78, 1, 78, 1, 79, 1, 79, 1,
        79, 1, 79, 3, 79, 824, 8, 79, 1, 80, 1, 80, 1, 80, 1, 80, 1, 80, 1, 81, 1, 81, 1, 81, 5,
        81, 834, 8, 81, 10, 81, 12, 81, 837, 9, 81, 1, 82, 1, 82, 3, 82, 841, 8, 82, 1, 82, 1, 82,
        1, 82, 1, 82, 1, 82, 4, 82, 848, 8, 82, 11, 82, 12, 82, 849, 1, 82, 1, 82, 3, 82, 854, 8,
        82, 1, 82, 1, 82, 1, 83, 1, 83, 1, 83, 3, 83, 861, 8, 83, 1, 84, 1, 84, 1, 84, 1, 84, 1,
        84, 1, 84, 1, 84, 3, 84, 870, 8, 84, 1, 85, 1, 85, 3, 85, 874, 8, 85, 1, 85, 1, 85, 3, 85,
        878, 8, 85, 3, 85, 880, 8, 85, 1, 86, 1, 86, 1, 87, 1, 87, 1, 88, 1, 88, 1, 89, 1, 89, 1,
        90, 1, 90, 3, 90, 892, 8, 90, 1, 90, 1, 90, 1, 91, 1, 91, 1, 91, 1, 91, 5, 91, 900, 8, 91,
        10, 91, 12, 91, 903, 9, 91, 3, 91, 905, 8, 91, 1, 91, 1, 91, 1, 92, 1, 92, 1, 92, 1, 92, 1,
        93, 1, 93, 3, 93, 915, 8, 93, 1, 94, 1, 94, 1, 95, 1, 95, 1, 95, 0, 0, 96, 0, 2, 4, 6, 8,
        10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54,
        56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 90, 92, 94, 96, 98,
        100, 102, 104, 106, 108, 110, 112, 114, 116, 118, 120, 122, 124, 126, 128, 130, 132, 134,
        136, 138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 166, 168, 170,
        172, 174, 176, 178, 180, 182, 184, 186, 188, 190, 0, 11, 2, 0, 37, 38, 42, 43, 2, 0, 40,
        40, 48, 48, 1, 0, 1, 2, 2, 0, 1, 1, 3, 7, 1, 0, 18, 19, 2, 0, 20, 21, 23, 23, 2, 0, 92, 92,
        96, 96, 1, 0, 33, 36, 1, 0, 74, 75, 2, 0, 30, 35, 92, 93, 1, 0, 36, 91, 963, 0, 192, 1, 0,
        0, 0, 2, 198, 1, 0, 0, 0, 4, 212, 1, 0, 0, 0, 6, 214, 1, 0, 0, 0, 8, 219, 1, 0, 0, 0, 10,
        230, 1, 0, 0, 0, 12, 239, 1, 0, 0, 0, 14, 241, 1, 0, 0, 0, 16, 253, 1, 0, 0, 0, 18, 256, 1,
        0, 0, 0, 20, 261, 1, 0, 0, 0, 22, 264, 1, 0, 0, 0, 24, 268, 1, 0, 0, 0, 26, 282, 1, 0, 0,
        0, 28, 291, 1, 0, 0, 0, 30, 296, 1, 0, 0, 0, 32, 300, 1, 0, 0, 0, 34, 313, 1, 0, 0, 0, 36,
        330, 1, 0, 0, 0, 38, 348, 1, 0, 0, 0, 40, 353, 1, 0, 0, 0, 42, 361, 1, 0, 0, 0, 44, 368, 1,
        0, 0, 0, 46, 371, 1, 0, 0, 0, 48, 376, 1, 0, 0, 0, 50, 389, 1, 0, 0, 0, 52, 391, 1, 0, 0,
        0, 54, 398, 1, 0, 0, 0, 56, 404, 1, 0, 0, 0, 58, 418, 1, 0, 0, 0, 60, 422, 1, 0, 0, 0, 62,
        430, 1, 0, 0, 0, 64, 434, 1, 0, 0, 0, 66, 454, 1, 0, 0, 0, 68, 458, 1, 0, 0, 0, 70, 462, 1,
        0, 0, 0, 72, 465, 1, 0, 0, 0, 74, 469, 1, 0, 0, 0, 76, 472, 1, 0, 0, 0, 78, 480, 1, 0, 0,
        0, 80, 488, 1, 0, 0, 0, 82, 496, 1, 0, 0, 0, 84, 507, 1, 0, 0, 0, 86, 512, 1, 0, 0, 0, 88,
        521, 1, 0, 0, 0, 90, 523, 1, 0, 0, 0, 92, 531, 1, 0, 0, 0, 94, 539, 1, 0, 0, 0, 96, 548, 1,
        0, 0, 0, 98, 552, 1, 0, 0, 0, 100, 575, 1, 0, 0, 0, 102, 577, 1, 0, 0, 0, 104, 585, 1, 0,
        0, 0, 106, 587, 1, 0, 0, 0, 108, 593, 1, 0, 0, 0, 110, 597, 1, 0, 0, 0, 112, 608, 1, 0, 0,
        0, 114, 614, 1, 0, 0, 0, 116, 633, 1, 0, 0, 0, 118, 635, 1, 0, 0, 0, 120, 638, 1, 0, 0, 0,
        122, 669, 1, 0, 0, 0, 124, 671, 1, 0, 0, 0, 126, 675, 1, 0, 0, 0, 128, 677, 1, 0, 0, 0,
        130, 701, 1, 0, 0, 0, 132, 703, 1, 0, 0, 0, 134, 723, 1, 0, 0, 0, 136, 725, 1, 0, 0, 0,
        138, 740, 1, 0, 0, 0, 140, 752, 1, 0, 0, 0, 142, 758, 1, 0, 0, 0, 144, 766, 1, 0, 0, 0,
        146, 774, 1, 0, 0, 0, 148, 784, 1, 0, 0, 0, 150, 788, 1, 0, 0, 0, 152, 793, 1, 0, 0, 0,
        154, 805, 1, 0, 0, 0, 156, 811, 1, 0, 0, 0, 158, 819, 1, 0, 0, 0, 160, 825, 1, 0, 0, 0,
        162, 830, 1, 0, 0, 0, 164, 838, 1, 0, 0, 0, 166, 857, 1, 0, 0, 0, 168, 869, 1, 0, 0, 0,
        170, 871, 1, 0, 0, 0, 172, 881, 1, 0, 0, 0, 174, 883, 1, 0, 0, 0, 176, 885, 1, 0, 0, 0,
        178, 887, 1, 0, 0, 0, 180, 889, 1, 0, 0, 0, 182, 895, 1, 0, 0, 0, 184, 908, 1, 0, 0, 0,
        186, 914, 1, 0, 0, 0, 188, 916, 1, 0, 0, 0, 190, 918, 1, 0, 0, 0, 192, 194, 3, 4, 2, 0,
        193, 195, 5, 9, 0, 0, 194, 193, 1, 0, 0, 0, 194, 195, 1, 0, 0, 0, 195, 196, 1, 0, 0, 0,
        196, 197, 5, 0, 0, 1, 197, 1, 1, 0, 0, 0, 198, 203, 3, 4, 2, 0, 199, 200, 5, 9, 0, 0, 200,
        202, 3, 4, 2, 0, 201, 199, 1, 0, 0, 0, 202, 205, 1, 0, 0, 0, 203, 201, 1, 0, 0, 0, 203,
        204, 1, 0, 0, 0, 204, 206, 1, 0, 0, 0, 205, 203, 1, 0, 0, 0, 206, 207, 5, 0, 0, 1, 207, 3,
        1, 0, 0, 0, 208, 213, 3, 6, 3, 0, 209, 213, 3, 10, 5, 0, 210, 213, 3, 14, 7, 0, 211, 213,
        3, 8, 4, 0, 212, 208, 1, 0, 0, 0, 212, 209, 1, 0, 0, 0, 212, 210, 1, 0, 0, 0, 212, 211, 1,
        0, 0, 0, 213, 5, 1, 0, 0, 0, 214, 217, 5, 46, 0, 0, 215, 218, 3, 8, 4, 0, 216, 218, 3, 10,
        5, 0, 217, 215, 1, 0, 0, 0, 217, 216, 1, 0, 0, 0, 218, 7, 1, 0, 0, 0, 219, 220, 5, 40, 0,
        0, 220, 221, 5, 67, 0, 0, 221, 222, 5, 50, 0, 0, 222, 223, 5, 25, 0, 0, 223, 224, 3, 186,
        93, 0, 224, 225, 5, 12, 0, 0, 225, 226, 3, 186, 93, 0, 226, 228, 5, 13, 0, 0, 227, 229, 5,
        81, 0, 0, 228, 227, 1, 0, 0, 0, 228, 229, 1, 0, 0, 0, 229, 9, 1, 0, 0, 0, 230, 234, 3, 12,
        6, 0, 231, 233, 3, 140, 70, 0, 232, 231, 1, 0, 0, 0, 233, 236, 1, 0, 0, 0, 234, 232, 1, 0,
        0, 0, 234, 235, 1, 0, 0, 0, 235, 11, 1, 0, 0, 0, 236, 234, 1, 0, 0, 0, 237, 240, 3, 34, 17,
        0, 238, 240, 3, 36, 18, 0, 239, 237, 1, 0, 0, 0, 239, 238, 1, 0, 0, 0, 240, 13, 1, 0, 0, 0,
        241, 242, 5, 28, 0, 0, 242, 244, 3, 144, 72, 0, 243, 245, 3, 54, 27, 0, 244, 243, 1, 0, 0,
        0, 244, 245, 1, 0, 0, 0, 245, 251, 1, 0, 0, 0, 246, 249, 5, 29, 0, 0, 247, 250, 5, 23, 0,
        0, 248, 250, 3, 56, 28, 0, 249, 247, 1, 0, 0, 0, 249, 248, 1, 0, 0, 0, 250, 252, 1, 0, 0,
        0, 251, 246, 1, 0, 0, 0, 251, 252, 1, 0, 0, 0, 252, 15, 1, 0, 0, 0, 253, 254, 5, 54, 0, 0,
        254, 255, 3, 24, 12, 0, 255, 17, 1, 0, 0, 0, 256, 257, 5, 58, 0, 0, 257, 259, 3, 24, 12, 0,
        258, 260, 3, 74, 37, 0, 259, 258, 1, 0, 0, 0, 259, 260, 1, 0, 0, 0, 260, 19, 1, 0, 0, 0,
        261, 262, 5, 56, 0, 0, 262, 263, 3, 78, 39, 0, 263, 21, 1, 0, 0, 0, 264, 265, 5, 47, 0, 0,
        265, 266, 3, 78, 39, 0, 266, 23, 1, 0, 0, 0, 267, 269, 5, 64, 0, 0, 268, 267, 1, 0, 0, 0,
        268, 269, 1, 0, 0, 0, 269, 270, 1, 0, 0, 0, 270, 272, 3, 26, 13, 0, 271, 273, 3, 32, 16, 0,
        272, 271, 1, 0, 0, 0, 272, 273, 1, 0, 0, 0, 273, 275, 1, 0, 0, 0, 274, 276, 3, 20, 10, 0,
        275, 274, 1, 0, 0, 0, 275, 276, 1, 0, 0, 0, 276, 278, 1, 0, 0, 0, 277, 279, 3, 22, 11, 0,
        278, 277, 1, 0, 0, 0, 278, 279, 1, 0, 0, 0, 279, 25, 1, 0, 0, 0, 280, 283, 5, 23, 0, 0,
        281, 283, 3, 28, 14, 0, 282, 280, 1, 0, 0, 0, 282, 281, 1, 0, 0, 0, 283, 288, 1, 0, 0, 0,
        284, 285, 5, 11, 0, 0, 285, 287, 3, 28, 14, 0, 286, 284, 1, 0, 0, 0, 287, 290, 1, 0, 0, 0,
        288, 286, 1, 0, 0, 0, 288, 289, 1, 0, 0, 0, 289, 27, 1, 0, 0, 0, 290, 288, 1, 0, 0, 0, 291,
        294, 3, 78, 39, 0, 292, 293, 5, 62, 0, 0, 293, 295, 3, 188, 94, 0, 294, 292, 1, 0, 0, 0,
        294, 295, 1, 0, 0, 0, 295, 29, 1, 0, 0, 0, 296, 298, 3, 78, 39, 0, 297, 299, 7, 0, 0, 0,
        298, 297, 1, 0, 0, 0, 298, 299, 1, 0, 0, 0, 299, 31, 1, 0, 0, 0, 300, 301, 5, 52, 0, 0,
        301, 302, 5, 39, 0, 0, 302, 307, 3, 30, 15, 0, 303, 304, 5, 11, 0, 0, 304, 306, 3, 30, 15,
        0, 305, 303, 1, 0, 0, 0, 306, 309, 1, 0, 0, 0, 307, 305, 1, 0, 0, 0, 307, 308, 1, 0, 0, 0,
        308, 33, 1, 0, 0, 0, 309, 307, 1, 0, 0, 0, 310, 312, 3, 42, 21, 0, 311, 310, 1, 0, 0, 0,
        312, 315, 1, 0, 0, 0, 313, 311, 1, 0, 0, 0, 313, 314, 1, 0, 0, 0, 314, 325, 1, 0, 0, 0,
        315, 313, 1, 0, 0, 0, 316, 326, 3, 16, 8, 0, 317, 319, 3, 44, 22, 0, 318, 317, 1, 0, 0, 0,
        319, 320, 1, 0, 0, 0, 320, 318, 1, 0, 0, 0, 320, 321, 1, 0, 0, 0, 321, 323, 1, 0, 0, 0,
        322, 324, 3, 16, 8, 0, 323, 322, 1, 0, 0, 0, 323, 324, 1, 0, 0, 0, 324, 326, 1, 0, 0, 0,
        325, 316, 1, 0, 0, 0, 325, 318, 1, 0, 0, 0, 326, 35, 1, 0, 0, 0, 327, 329, 3, 42, 21, 0,
        328, 327, 1, 0, 0, 0, 329, 332, 1, 0, 0, 0, 330, 328, 1, 0, 0, 0, 330, 331, 1, 0, 0, 0,
        331, 341, 1, 0, 0, 0, 332, 330, 1, 0, 0, 0, 333, 336, 3, 42, 21, 0, 334, 336, 3, 44, 22, 0,
        335, 333, 1, 0, 0, 0, 335, 334, 1, 0, 0, 0, 336, 339, 1, 0, 0, 0, 337, 335, 1, 0, 0, 0,
        337, 338, 1, 0, 0, 0, 338, 340, 1, 0, 0, 0, 339, 337, 1, 0, 0, 0, 340, 342, 3, 18, 9, 0,
        341, 337, 1, 0, 0, 0, 342, 343, 1, 0, 0, 0, 343, 341, 1, 0, 0, 0, 343, 344, 1, 0, 0, 0,
        344, 345, 1, 0, 0, 0, 345, 346, 3, 34, 17, 0, 346, 37, 1, 0, 0, 0, 347, 349, 5, 51, 0, 0,
        348, 347, 1, 0, 0, 0, 348, 349, 1, 0, 0, 0, 349, 350, 1, 0, 0, 0, 350, 351, 5, 48, 0, 0,
        351, 352, 3, 72, 36, 0, 352, 39, 1, 0, 0, 0, 353, 354, 5, 60, 0, 0, 354, 355, 3, 78, 39, 0,
        355, 356, 5, 62, 0, 0, 356, 357, 3, 188, 94, 0, 357, 41, 1, 0, 0, 0, 358, 362, 3, 38, 19,
        0, 359, 362, 3, 40, 20, 0, 360, 362, 3, 52, 26, 0, 361, 358, 1, 0, 0, 0, 361, 359, 1, 0, 0,
        0, 361, 360, 1, 0, 0, 0, 362, 43, 1, 0, 0, 0, 363, 369, 3, 70, 35, 0, 364, 369, 3, 60, 30,
        0, 365, 369, 3, 46, 23, 0, 366, 369, 3, 64, 32, 0, 367, 369, 3, 48, 24, 0, 368, 363, 1, 0,
        0, 0, 368, 364, 1, 0, 0, 0, 368, 365, 1, 0, 0, 0, 368, 366, 1, 0, 0, 0, 368, 367, 1, 0, 0,
        0, 369, 45, 1, 0, 0, 0, 370, 372, 5, 44, 0, 0, 371, 370, 1, 0, 0, 0, 371, 372, 1, 0, 0, 0,
        372, 373, 1, 0, 0, 0, 373, 374, 5, 41, 0, 0, 374, 375, 3, 162, 81, 0, 375, 47, 1, 0, 0, 0,
        376, 377, 5, 53, 0, 0, 377, 382, 3, 50, 25, 0, 378, 379, 5, 11, 0, 0, 379, 381, 3, 50, 25,
        0, 380, 378, 1, 0, 0, 0, 381, 384, 1, 0, 0, 0, 382, 380, 1, 0, 0, 0, 382, 383, 1, 0, 0, 0,
        383, 49, 1, 0, 0, 0, 384, 382, 1, 0, 0, 0, 385, 386, 3, 188, 94, 0, 386, 387, 3, 68, 34, 0,
        387, 390, 1, 0, 0, 0, 388, 390, 3, 110, 55, 0, 389, 385, 1, 0, 0, 0, 389, 388, 1, 0, 0, 0,
        390, 51, 1, 0, 0, 0, 391, 392, 5, 28, 0, 0, 392, 393, 3, 144, 72, 0, 393, 396, 3, 54, 27,
        0, 394, 395, 5, 29, 0, 0, 395, 397, 3, 56, 28, 0, 396, 394, 1, 0, 0, 0, 396, 397, 1, 0, 0,
        0, 397, 53, 1, 0, 0, 0, 398, 400, 5, 12, 0, 0, 399, 401, 3, 162, 81, 0, 400, 399, 1, 0, 0,
        0, 400, 401, 1, 0, 0, 0, 401, 402, 1, 0, 0, 0, 402, 403, 5, 13, 0, 0, 403, 55, 1, 0, 0, 0,
        404, 409, 3, 58, 29, 0, 405, 406, 5, 11, 0, 0, 406, 408, 3, 58, 29, 0, 407, 405, 1, 0, 0,
        0, 408, 411, 1, 0, 0, 0, 409, 407, 1, 0, 0, 0, 409, 410, 1, 0, 0, 0, 410, 413, 1, 0, 0, 0,
        411, 409, 1, 0, 0, 0, 412, 414, 3, 74, 37, 0, 413, 412, 1, 0, 0, 0, 413, 414, 1, 0, 0, 0,
        414, 57, 1, 0, 0, 0, 415, 416, 3, 188, 94, 0, 416, 417, 5, 62, 0, 0, 417, 419, 1, 0, 0, 0,
        418, 415, 1, 0, 0, 0, 418, 419, 1, 0, 0, 0, 419, 420, 1, 0, 0, 0, 420, 421, 3, 188, 94, 0,
        421, 59, 1, 0, 0, 0, 422, 423, 5, 49, 0, 0, 423, 427, 3, 112, 56, 0, 424, 426, 3, 62, 31,
        0, 425, 424, 1, 0, 0, 0, 426, 429, 1, 0, 0, 0, 427, 425, 1, 0, 0, 0, 427, 428, 1, 0, 0, 0,
        428, 61, 1, 0, 0, 0, 429, 427, 1, 0, 0, 0, 430, 431, 5, 50, 0, 0, 431, 432, 7, 1, 0, 0,
        432, 433, 3, 64, 32, 0, 433, 63, 1, 0, 0, 0, 434, 435, 5, 55, 0, 0, 435, 440, 3, 66, 33, 0,
        436, 437, 5, 11, 0, 0, 437, 439, 3, 66, 33, 0, 438, 436, 1, 0, 0, 0, 439, 442, 1, 0, 0, 0,
        440, 438, 1, 0, 0, 0, 440, 441, 1, 0, 0, 0, 441, 65, 1, 0, 0, 0, 442, 440, 1, 0, 0, 0, 443,
        444, 3, 110, 55, 0, 444, 445, 5, 1, 0, 0, 445, 446, 3, 78, 39, 0, 446, 455, 1, 0, 0, 0,
        447, 448, 3, 188, 94, 0, 448, 449, 7, 2, 0, 0, 449, 450, 3, 78, 39, 0, 450, 455, 1, 0, 0,
        0, 451, 452, 3, 188, 94, 0, 452, 453, 3, 68, 34, 0, 453, 455, 1, 0, 0, 0, 454, 443, 1, 0,
        0, 0, 454, 447, 1, 0, 0, 0, 454, 451, 1, 0, 0, 0, 455, 67, 1, 0, 0, 0, 456, 457, 5, 25, 0,
        0, 457, 459, 3, 186, 93, 0, 458, 456, 1, 0, 0, 0, 459, 460, 1, 0, 0, 0, 460, 458, 1, 0, 0,
        0, 460, 461, 1, 0, 0, 0, 461, 69, 1, 0, 0, 0, 462, 463, 5, 40, 0, 0, 463, 464, 3, 76, 38,
        0, 464, 71, 1, 0, 0, 0, 465, 467, 3, 76, 38, 0, 466, 468, 3, 74, 37, 0, 467, 466, 1, 0, 0,
        0, 467, 468, 1, 0, 0, 0, 468, 73, 1, 0, 0, 0, 469, 470, 5, 57, 0, 0, 470, 471, 3, 78, 39,
        0, 471, 75, 1, 0, 0, 0, 472, 477, 3, 112, 56, 0, 473, 474, 5, 11, 0, 0, 474, 476, 3, 112,
        56, 0, 475, 473, 1, 0, 0, 0, 476, 479, 1, 0, 0, 0, 477, 475, 1, 0, 0, 0, 477, 478, 1, 0, 0,
        0, 478, 77, 1, 0, 0, 0, 479, 477, 1, 0, 0, 0, 480, 485, 3, 80, 40, 0, 481, 482, 5, 70, 0,
        0, 482, 484, 3, 80, 40, 0, 483, 481, 1, 0, 0, 0, 484, 487, 1, 0, 0, 0, 485, 483, 1, 0, 0,
        0, 485, 486, 1, 0, 0, 0, 486, 79, 1, 0, 0, 0, 487, 485, 1, 0, 0, 0, 488, 493, 3, 82, 41, 0,
        489, 490, 5, 72, 0, 0, 490, 492, 3, 82, 41, 0, 491, 489, 1, 0, 0, 0, 492, 495, 1, 0, 0, 0,
        493, 491, 1, 0, 0, 0, 493, 494, 1, 0, 0, 0, 494, 81, 1, 0, 0, 0, 495, 493, 1, 0, 0, 0, 496,
        501, 3, 84, 42, 0, 497, 498, 5, 61, 0, 0, 498, 500, 3, 84, 42, 0, 499, 497, 1, 0, 0, 0,
        500, 503, 1, 0, 0, 0, 501, 499, 1, 0, 0, 0, 501, 502, 1, 0, 0, 0, 502, 83, 1, 0, 0, 0, 503,
        501, 1, 0, 0, 0, 504, 506, 5, 69, 0, 0, 505, 504, 1, 0, 0, 0, 506, 509, 1, 0, 0, 0, 507,
        505, 1, 0, 0, 0, 507, 508, 1, 0, 0, 0, 508, 510, 1, 0, 0, 0, 509, 507, 1, 0, 0, 0, 510,
        511, 3, 86, 43, 0, 511, 85, 1, 0, 0, 0, 512, 518, 3, 90, 45, 0, 513, 514, 3, 88, 44, 0,
        514, 515, 3, 90, 45, 0, 515, 517, 1, 0, 0, 0, 516, 513, 1, 0, 0, 0, 517, 520, 1, 0, 0, 0,
        518, 516, 1, 0, 0, 0, 518, 519, 1, 0, 0, 0, 519, 87, 1, 0, 0, 0, 520, 518, 1, 0, 0, 0, 521,
        522, 7, 3, 0, 0, 522, 89, 1, 0, 0, 0, 523, 528, 3, 92, 46, 0, 524, 525, 7, 4, 0, 0, 525,
        527, 3, 92, 46, 0, 526, 524, 1, 0, 0, 0, 527, 530, 1, 0, 0, 0, 528, 526, 1, 0, 0, 0, 528,
        529, 1, 0, 0, 0, 529, 91, 1, 0, 0, 0, 530, 528, 1, 0, 0, 0, 531, 536, 3, 94, 47, 0, 532,
        533, 7, 5, 0, 0, 533, 535, 3, 94, 47, 0, 534, 532, 1, 0, 0, 0, 535, 538, 1, 0, 0, 0, 536,
        534, 1, 0, 0, 0, 536, 537, 1, 0, 0, 0, 537, 93, 1, 0, 0, 0, 538, 536, 1, 0, 0, 0, 539, 544,
        3, 96, 48, 0, 540, 541, 5, 22, 0, 0, 541, 543, 3, 96, 48, 0, 542, 540, 1, 0, 0, 0, 543,
        546, 1, 0, 0, 0, 544, 542, 1, 0, 0, 0, 544, 545, 1, 0, 0, 0, 545, 95, 1, 0, 0, 0, 546, 544,
        1, 0, 0, 0, 547, 549, 7, 4, 0, 0, 548, 547, 1, 0, 0, 0, 548, 549, 1, 0, 0, 0, 549, 550, 1,
        0, 0, 0, 550, 551, 3, 98, 49, 0, 551, 97, 1, 0, 0, 0, 552, 558, 3, 108, 54, 0, 553, 557, 3,
        102, 51, 0, 554, 557, 3, 100, 50, 0, 555, 557, 3, 106, 53, 0, 556, 553, 1, 0, 0, 0, 556,
        554, 1, 0, 0, 0, 556, 555, 1, 0, 0, 0, 557, 560, 1, 0, 0, 0, 558, 556, 1, 0, 0, 0, 558,
        559, 1, 0, 0, 0, 559, 99, 1, 0, 0, 0, 560, 558, 1, 0, 0, 0, 561, 562, 5, 66, 0, 0, 562,
        576, 3, 108, 54, 0, 563, 572, 5, 16, 0, 0, 564, 566, 3, 78, 39, 0, 565, 564, 1, 0, 0, 0,
        565, 566, 1, 0, 0, 0, 566, 567, 1, 0, 0, 0, 567, 569, 5, 8, 0, 0, 568, 570, 3, 78, 39, 0,
        569, 568, 1, 0, 0, 0, 569, 570, 1, 0, 0, 0, 570, 573, 1, 0, 0, 0, 571, 573, 3, 78, 39, 0,
        572, 565, 1, 0, 0, 0, 572, 571, 1, 0, 0, 0, 573, 574, 1, 0, 0, 0, 574, 576, 5, 17, 0, 0,
        575, 561, 1, 0, 0, 0, 575, 563, 1, 0, 0, 0, 576, 101, 1, 0, 0, 0, 577, 578, 3, 104, 52, 0,
        578, 579, 3, 108, 54, 0, 579, 103, 1, 0, 0, 0, 580, 581, 5, 71, 0, 0, 581, 586, 5, 58, 0,
        0, 582, 583, 5, 65, 0, 0, 583, 586, 5, 58, 0, 0, 584, 586, 5, 63, 0, 0, 585, 580, 1, 0, 0,
        0, 585, 582, 1, 0, 0, 0, 585, 584, 1, 0, 0, 0, 586, 105, 1, 0, 0, 0, 587, 589, 5, 68, 0, 0,
        588, 590, 5, 69, 0, 0, 589, 588, 1, 0, 0, 0, 589, 590, 1, 0, 0, 0, 590, 591, 1, 0, 0, 0,
        591, 592, 5, 76, 0, 0, 592, 107, 1, 0, 0, 0, 593, 595, 3, 110, 55, 0, 594, 596, 3, 68, 34,
        0, 595, 594, 1, 0, 0, 0, 595, 596, 1, 0, 0, 0, 596, 109, 1, 0, 0, 0, 597, 602, 3, 130, 65,
        0, 598, 599, 5, 10, 0, 0, 599, 601, 3, 186, 93, 0, 600, 598, 1, 0, 0, 0, 601, 604, 1, 0, 0,
        0, 602, 600, 1, 0, 0, 0, 602, 603, 1, 0, 0, 0, 603, 111, 1, 0, 0, 0, 604, 602, 1, 0, 0, 0,
        605, 606, 3, 188, 94, 0, 606, 607, 5, 1, 0, 0, 607, 609, 1, 0, 0, 0, 608, 605, 1, 0, 0, 0,
        608, 609, 1, 0, 0, 0, 609, 612, 1, 0, 0, 0, 610, 613, 3, 114, 57, 0, 611, 613, 3, 116, 58,
        0, 612, 610, 1, 0, 0, 0, 612, 611, 1, 0, 0, 0, 613, 113, 1, 0, 0, 0, 614, 615, 5, 73, 0, 0,
        615, 616, 5, 12, 0, 0, 616, 617, 3, 116, 58, 0, 617, 618, 5, 13, 0, 0, 618, 115, 1, 0, 0,
        0, 619, 624, 3, 128, 64, 0, 620, 623, 3, 118, 59, 0, 621, 623, 3, 120, 60, 0, 622, 620, 1,
        0, 0, 0, 622, 621, 1, 0, 0, 0, 623, 626, 1, 0, 0, 0, 624, 622, 1, 0, 0, 0, 624, 625, 1, 0,
        0, 0, 625, 634, 1, 0, 0, 0, 626, 624, 1, 0, 0, 0, 627, 628, 5, 12, 0, 0, 628, 629, 3, 116,
        58, 0, 629, 631, 5, 13, 0, 0, 630, 632, 3, 122, 61, 0, 631, 630, 1, 0, 0, 0, 631, 632, 1,
        0, 0, 0, 632, 634, 1, 0, 0, 0, 633, 619, 1, 0, 0, 0, 633, 627, 1, 0, 0, 0, 634, 117, 1, 0,
        0, 0, 635, 636, 3, 134, 67, 0, 636, 637, 3, 128, 64, 0, 637, 119, 1, 0, 0, 0, 638, 639, 5,
        12, 0, 0, 639, 640, 3, 116, 58, 0, 640, 641, 5, 13, 0, 0, 641, 642, 3, 122, 61, 0, 642,
        643, 3, 128, 64, 0, 643, 121, 1, 0, 0, 0, 644, 645, 5, 14, 0, 0, 645, 646, 3, 124, 62, 0,
        646, 647, 5, 11, 0, 0, 647, 648, 3, 124, 62, 0, 648, 649, 5, 15, 0, 0, 649, 670, 1, 0, 0,
        0, 650, 651, 5, 14, 0, 0, 651, 652, 3, 124, 62, 0, 652, 653, 5, 15, 0, 0, 653, 670, 1, 0,
        0, 0, 654, 655, 5, 14, 0, 0, 655, 656, 3, 124, 62, 0, 656, 657, 5, 11, 0, 0, 657, 658, 5,
        15, 0, 0, 658, 670, 1, 0, 0, 0, 659, 660, 5, 14, 0, 0, 660, 661, 5, 11, 0, 0, 661, 662, 3,
        124, 62, 0, 662, 663, 5, 15, 0, 0, 663, 670, 1, 0, 0, 0, 664, 665, 5, 14, 0, 0, 665, 666,
        5, 11, 0, 0, 666, 670, 5, 15, 0, 0, 667, 670, 5, 19, 0, 0, 668, 670, 5, 23, 0, 0, 669, 644,
        1, 0, 0, 0, 669, 650, 1, 0, 0, 0, 669, 654, 1, 0, 0, 0, 669, 659, 1, 0, 0, 0, 669, 664, 1,
        0, 0, 0, 669, 667, 1, 0, 0, 0, 669, 668, 1, 0, 0, 0, 670, 123, 1, 0, 0, 0, 671, 672, 7, 6,
        0, 0, 672, 125, 1, 0, 0, 0, 673, 676, 3, 182, 91, 0, 674, 676, 3, 166, 83, 0, 675, 673, 1,
        0, 0, 0, 675, 674, 1, 0, 0, 0, 676, 127, 1, 0, 0, 0, 677, 679, 5, 12, 0, 0, 678, 680, 3,
        188, 94, 0, 679, 678, 1, 0, 0, 0, 679, 680, 1, 0, 0, 0, 680, 682, 1, 0, 0, 0, 681, 683, 3,
        68, 34, 0, 682, 681, 1, 0, 0, 0, 682, 683, 1, 0, 0, 0, 683, 685, 1, 0, 0, 0, 684, 686, 3,
        126, 63, 0, 685, 684, 1, 0, 0, 0, 685, 686, 1, 0, 0, 0, 686, 687, 1, 0, 0, 0, 687, 688, 5,
        13, 0, 0, 688, 129, 1, 0, 0, 0, 689, 702, 3, 168, 84, 0, 690, 702, 3, 166, 83, 0, 691, 702,
        3, 164, 82, 0, 692, 702, 3, 160, 80, 0, 693, 702, 3, 156, 78, 0, 694, 702, 3, 152, 76, 0,
        695, 702, 3, 150, 75, 0, 696, 702, 3, 154, 77, 0, 697, 702, 3, 148, 74, 0, 698, 702, 3,
        146, 73, 0, 699, 702, 3, 188, 94, 0, 700, 702, 3, 142, 71, 0, 701, 689, 1, 0, 0, 0, 701,
        690, 1, 0, 0, 0, 701, 691, 1, 0, 0, 0, 701, 692, 1, 0, 0, 0, 701, 693, 1, 0, 0, 0, 701,
        694, 1, 0, 0, 0, 701, 695, 1, 0, 0, 0, 701, 696, 1, 0, 0, 0, 701, 697, 1, 0, 0, 0, 701,
        698, 1, 0, 0, 0, 701, 699, 1, 0, 0, 0, 701, 700, 1, 0, 0, 0, 702, 131, 1, 0, 0, 0, 703,
        704, 3, 188, 94, 0, 704, 705, 5, 1, 0, 0, 705, 133, 1, 0, 0, 0, 706, 707, 5, 6, 0, 0, 707,
        709, 5, 18, 0, 0, 708, 710, 3, 136, 68, 0, 709, 708, 1, 0, 0, 0, 709, 710, 1, 0, 0, 0, 710,
        711, 1, 0, 0, 0, 711, 713, 5, 18, 0, 0, 712, 714, 5, 5, 0, 0, 713, 712, 1, 0, 0, 0, 713,
        714, 1, 0, 0, 0, 714, 724, 1, 0, 0, 0, 715, 717, 5, 18, 0, 0, 716, 718, 3, 136, 68, 0, 717,
        716, 1, 0, 0, 0, 717, 718, 1, 0, 0, 0, 718, 719, 1, 0, 0, 0, 719, 721, 5, 18, 0, 0, 720,
        722, 5, 5, 0, 0, 721, 720, 1, 0, 0, 0, 721, 722, 1, 0, 0, 0, 722, 724, 1, 0, 0, 0, 723,
        706, 1, 0, 0, 0, 723, 715, 1, 0, 0, 0, 724, 135, 1, 0, 0, 0, 725, 727, 5, 16, 0, 0, 726,
        728, 3, 188, 94, 0, 727, 726, 1, 0, 0, 0, 727, 728, 1, 0, 0, 0, 728, 730, 1, 0, 0, 0, 729,
        731, 3, 138, 69, 0, 730, 729, 1, 0, 0, 0, 730, 731, 1, 0, 0, 0, 731, 733, 1, 0, 0, 0, 732,
        734, 3, 170, 85, 0, 733, 732, 1, 0, 0, 0, 733, 734, 1, 0, 0, 0, 734, 736, 1, 0, 0, 0, 735,
        737, 3, 126, 63, 0, 736, 735, 1, 0, 0, 0, 736, 737, 1, 0, 0, 0, 737, 738, 1, 0, 0, 0, 738,
        739, 5, 17, 0, 0, 739, 137, 1, 0, 0, 0, 740, 741, 5, 25, 0, 0, 741, 749, 3, 186, 93, 0,
        742, 744, 5, 26, 0, 0, 743, 745, 5, 25, 0, 0, 744, 743, 1, 0, 0, 0, 744, 745, 1, 0, 0, 0,
        745, 746, 1, 0, 0, 0, 746, 748, 3, 186, 93, 0, 747, 742, 1, 0, 0, 0, 748, 751, 1, 0, 0, 0,
        749, 747, 1, 0, 0, 0, 749, 750, 1, 0, 0, 0, 750, 139, 1, 0, 0, 0, 751, 749, 1, 0, 0, 0,
        752, 754, 5, 59, 0, 0, 753, 755, 5, 36, 0, 0, 754, 753, 1, 0, 0, 0, 754, 755, 1, 0, 0, 0,
        755, 756, 1, 0, 0, 0, 756, 757, 3, 12, 6, 0, 757, 141, 1, 0, 0, 0, 758, 759, 5, 45, 0, 0,
        759, 762, 5, 14, 0, 0, 760, 763, 3, 10, 5, 0, 761, 763, 3, 72, 36, 0, 762, 760, 1, 0, 0, 0,
        762, 761, 1, 0, 0, 0, 763, 764, 1, 0, 0, 0, 764, 765, 5, 15, 0, 0, 765, 143, 1, 0, 0, 0,
        766, 771, 3, 188, 94, 0, 767, 768, 5, 10, 0, 0, 768, 770, 3, 188, 94, 0, 769, 767, 1, 0, 0,
        0, 770, 773, 1, 0, 0, 0, 771, 769, 1, 0, 0, 0, 771, 772, 1, 0, 0, 0, 772, 145, 1, 0, 0, 0,
        773, 771, 1, 0, 0, 0, 774, 775, 3, 144, 72, 0, 775, 777, 5, 12, 0, 0, 776, 778, 5, 64, 0,
        0, 777, 776, 1, 0, 0, 0, 777, 778, 1, 0, 0, 0, 778, 780, 1, 0, 0, 0, 779, 781, 3, 162, 81,
        0, 780, 779, 1, 0, 0, 0, 780, 781, 1, 0, 0, 0, 781, 782, 1, 0, 0, 0, 782, 783, 5, 13, 0, 0,
        783, 147, 1, 0, 0, 0, 784, 785, 5, 12, 0, 0, 785, 786, 3, 78, 39, 0, 786, 787, 5, 13, 0, 0,
        787, 149, 1, 0, 0, 0, 788, 789, 7, 7, 0, 0, 789, 790, 5, 12, 0, 0, 790, 791, 3, 158, 79, 0,
        791, 792, 5, 13, 0, 0, 792, 151, 1, 0, 0, 0, 793, 795, 5, 16, 0, 0, 794, 796, 3, 132, 66,
        0, 795, 794, 1, 0, 0, 0, 795, 796, 1, 0, 0, 0, 796, 797, 1, 0, 0, 0, 797, 799, 3, 154, 77,
        0, 798, 800, 3, 74, 37, 0, 799, 798, 1, 0, 0, 0, 799, 800, 1, 0, 0, 0, 800, 801, 1, 0, 0,
        0, 801, 802, 5, 26, 0, 0, 802, 803, 3, 78, 39, 0, 803, 804, 5, 17, 0, 0, 804, 153, 1, 0, 0,
        0, 805, 807, 3, 128, 64, 0, 806, 808, 3, 118, 59, 0, 807, 806, 1, 0, 0, 0, 808, 809, 1, 0,
        0, 0, 809, 807, 1, 0, 0, 0, 809, 810, 1, 0, 0, 0, 810, 155, 1, 0, 0, 0, 811, 812, 5, 16, 0,
        0, 812, 815, 3, 158, 79, 0, 813, 814, 5, 26, 0, 0, 814, 816, 3, 78, 39, 0, 815, 813, 1, 0,
        0, 0, 815, 816, 1, 0, 0, 0, 816, 817, 1, 0, 0, 0, 817, 818, 5, 17, 0, 0, 818, 157, 1, 0, 0,
        0, 819, 820, 3, 188, 94, 0, 820, 821, 5, 66, 0, 0, 821, 823, 3, 78, 39, 0, 822, 824, 3, 74,
        37, 0, 823, 822, 1, 0, 0, 0, 823, 824, 1, 0, 0, 0, 824, 159, 1, 0, 0, 0, 825, 826, 5, 32,
        0, 0, 826, 827, 5, 12, 0, 0, 827, 828, 5, 23, 0, 0, 828, 829, 5, 13, 0, 0, 829, 161, 1, 0,
        0, 0, 830, 835, 3, 78, 39, 0, 831, 832, 5, 11, 0, 0, 832, 834, 3, 78, 39, 0, 833, 831, 1,
        0, 0, 0, 834, 837, 1, 0, 0, 0, 835, 833, 1, 0, 0, 0, 835, 836, 1, 0, 0, 0, 836, 163, 1, 0,
        0, 0, 837, 835, 1, 0, 0, 0, 838, 840, 5, 82, 0, 0, 839, 841, 3, 78, 39, 0, 840, 839, 1, 0,
        0, 0, 840, 841, 1, 0, 0, 0, 841, 847, 1, 0, 0, 0, 842, 843, 5, 83, 0, 0, 843, 844, 3, 78,
        39, 0, 844, 845, 5, 84, 0, 0, 845, 846, 3, 78, 39, 0, 846, 848, 1, 0, 0, 0, 847, 842, 1, 0,
        0, 0, 848, 849, 1, 0, 0, 0, 849, 847, 1, 0, 0, 0, 849, 850, 1, 0, 0, 0, 850, 853, 1, 0, 0,
        0, 851, 852, 5, 85, 0, 0, 852, 854, 3, 78, 39, 0, 853, 851, 1, 0, 0, 0, 853, 854, 1, 0, 0,
        0, 854, 855, 1, 0, 0, 0, 855, 856, 5, 86, 0, 0, 856, 165, 1, 0, 0, 0, 857, 860, 5, 27, 0,
        0, 858, 861, 3, 188, 94, 0, 859, 861, 3, 174, 87, 0, 860, 858, 1, 0, 0, 0, 860, 859, 1, 0,
        0, 0, 861, 167, 1, 0, 0, 0, 862, 870, 3, 172, 86, 0, 863, 870, 3, 174, 87, 0, 864, 870, 5,
        76, 0, 0, 865, 870, 3, 176, 88, 0, 866, 870, 3, 178, 89, 0, 867, 870, 3, 180, 90, 0, 868,
        870, 3, 182, 91, 0, 869, 862, 1, 0, 0, 0, 869, 863, 1, 0, 0, 0, 869, 864, 1, 0, 0, 0, 869,
        865, 1, 0, 0, 0, 869, 866, 1, 0, 0, 0, 869, 867, 1, 0, 0, 0, 869, 868, 1, 0, 0, 0, 870,
        169, 1, 0, 0, 0, 871, 873, 5, 23, 0, 0, 872, 874, 3, 174, 87, 0, 873, 872, 1, 0, 0, 0, 873,
        874, 1, 0, 0, 0, 874, 879, 1, 0, 0, 0, 875, 877, 5, 8, 0, 0, 876, 878, 3, 174, 87, 0, 877,
        876, 1, 0, 0, 0, 877, 878, 1, 0, 0, 0, 878, 880, 1, 0, 0, 0, 879, 875, 1, 0, 0, 0, 879,
        880, 1, 0, 0, 0, 880, 171, 1, 0, 0, 0, 881, 882, 7, 8, 0, 0, 882, 173, 1, 0, 0, 0, 883,
        884, 5, 96, 0, 0, 884, 175, 1, 0, 0, 0, 885, 886, 5, 95, 0, 0, 886, 177, 1, 0, 0, 0, 887,
        888, 5, 94, 0, 0, 888, 179, 1, 0, 0, 0, 889, 891, 5, 16, 0, 0, 890, 892, 3, 162, 81, 0,
        891, 890, 1, 0, 0, 0, 891, 892, 1, 0, 0, 0, 892, 893, 1, 0, 0, 0, 893, 894, 5, 17, 0, 0,
        894, 181, 1, 0, 0, 0, 895, 904, 5, 14, 0, 0, 896, 901, 3, 184, 92, 0, 897, 898, 5, 11, 0,
        0, 898, 900, 3, 184, 92, 0, 899, 897, 1, 0, 0, 0, 900, 903, 1, 0, 0, 0, 901, 899, 1, 0, 0,
        0, 901, 902, 1, 0, 0, 0, 902, 905, 1, 0, 0, 0, 903, 901, 1, 0, 0, 0, 904, 896, 1, 0, 0, 0,
        904, 905, 1, 0, 0, 0, 905, 906, 1, 0, 0, 0, 906, 907, 5, 15, 0, 0, 907, 183, 1, 0, 0, 0,
        908, 909, 3, 186, 93, 0, 909, 910, 5, 25, 0, 0, 910, 911, 3, 78, 39, 0, 911, 185, 1, 0, 0,
        0, 912, 915, 3, 188, 94, 0, 913, 915, 3, 190, 95, 0, 914, 912, 1, 0, 0, 0, 914, 913, 1, 0,
        0, 0, 915, 187, 1, 0, 0, 0, 916, 917, 7, 9, 0, 0, 917, 189, 1, 0, 0, 0, 918, 919, 7, 10, 0,
        0, 919, 191, 1, 0, 0, 0, 110, 194, 203, 212, 217, 228, 234, 239, 244, 249, 251, 259, 268,
        272, 275, 278, 282, 288, 294, 298, 307, 313, 320, 323, 325, 330, 335, 337, 343, 348, 361,
        368, 371, 382, 389, 396, 400, 409, 413, 418, 427, 440, 454, 460, 467, 477, 485, 493, 501,
        507, 518, 528, 536, 544, 548, 556, 558, 565, 569, 572, 575, 585, 589, 595, 602, 608, 612,
        622, 624, 631, 633, 669, 675, 679, 682, 685, 701, 709, 713, 717, 721, 723, 727, 730, 733,
        736, 744, 749, 754, 762, 771, 777, 780, 795, 799, 809, 815, 823, 835, 840, 849, 853, 860,
        869, 873, 877, 879, 891, 901, 904, 914
    ];
}
