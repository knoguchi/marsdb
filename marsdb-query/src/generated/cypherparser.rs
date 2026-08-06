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
pub const RULE_query: usize = 1;
pub const RULE_explainSt: usize = 2;
pub const RULE_createIndexSt: usize = 3;
pub const RULE_regularQuery: usize = 4;
pub const RULE_singleQuery: usize = 5;
pub const RULE_standaloneCall: usize = 6;
pub const RULE_returnSt: usize = 7;
pub const RULE_withSt: usize = 8;
pub const RULE_skipSt: usize = 9;
pub const RULE_limitSt: usize = 10;
pub const RULE_projectionBody: usize = 11;
pub const RULE_projectionItems: usize = 12;
pub const RULE_projectionItem: usize = 13;
pub const RULE_orderItem: usize = 14;
pub const RULE_orderSt: usize = 15;
pub const RULE_singlePartQ: usize = 16;
pub const RULE_multiPartQ: usize = 17;
pub const RULE_matchSt: usize = 18;
pub const RULE_unwindSt: usize = 19;
pub const RULE_readingStatement: usize = 20;
pub const RULE_updatingStatement: usize = 21;
pub const RULE_deleteSt: usize = 22;
pub const RULE_removeSt: usize = 23;
pub const RULE_removeItem: usize = 24;
pub const RULE_queryCallSt: usize = 25;
pub const RULE_parenExpressionChain: usize = 26;
pub const RULE_yieldItems: usize = 27;
pub const RULE_yieldItem: usize = 28;
pub const RULE_mergeSt: usize = 29;
pub const RULE_mergeAction: usize = 30;
pub const RULE_setSt: usize = 31;
pub const RULE_setItem: usize = 32;
pub const RULE_nodeLabels: usize = 33;
pub const RULE_createSt: usize = 34;
pub const RULE_patternWhere: usize = 35;
pub const RULE_where: usize = 36;
pub const RULE_pattern: usize = 37;
pub const RULE_expression: usize = 38;
pub const RULE_xorExpression: usize = 39;
pub const RULE_andExpression: usize = 40;
pub const RULE_notExpression: usize = 41;
pub const RULE_comparisonExpression: usize = 42;
pub const RULE_comparisonSigns: usize = 43;
pub const RULE_addSubExpression: usize = 44;
pub const RULE_multDivExpression: usize = 45;
pub const RULE_powerExpression: usize = 46;
pub const RULE_unaryAddSubExpression: usize = 47;
pub const RULE_atomicExpression: usize = 48;
pub const RULE_listExpression: usize = 49;
pub const RULE_stringExpression: usize = 50;
pub const RULE_stringExpPrefix: usize = 51;
pub const RULE_nullExpression: usize = 52;
pub const RULE_propertyOrLabelExpression: usize = 53;
pub const RULE_propertyExpression: usize = 54;
pub const RULE_patternPart: usize = 55;
pub const RULE_shortestPathWrapper: usize = 56;
pub const RULE_patternElem: usize = 57;
pub const RULE_patternElemChain: usize = 58;
pub const RULE_qppElemChain: usize = 59;
pub const RULE_qppQuantifier: usize = 60;
pub const RULE_qppInt: usize = 61;
pub const RULE_properties: usize = 62;
pub const RULE_nodePattern: usize = 63;
pub const RULE_atom: usize = 64;
pub const RULE_lhs: usize = 65;
pub const RULE_relationshipPattern: usize = 66;
pub const RULE_relationDetail: usize = 67;
pub const RULE_relationshipTypes: usize = 68;
pub const RULE_unionSt: usize = 69;
pub const RULE_subqueryExist: usize = 70;
pub const RULE_invocationName: usize = 71;
pub const RULE_functionInvocation: usize = 72;
pub const RULE_parenthesizedExpression: usize = 73;
pub const RULE_filterWith: usize = 74;
pub const RULE_patternComprehension: usize = 75;
pub const RULE_relationshipsChainPattern: usize = 76;
pub const RULE_listComprehension: usize = 77;
pub const RULE_filterExpression: usize = 78;
pub const RULE_countAll: usize = 79;
pub const RULE_expressionChain: usize = 80;
pub const RULE_caseExpression: usize = 81;
pub const RULE_parameter: usize = 82;
pub const RULE_literal: usize = 83;
pub const RULE_rangeLit: usize = 84;
pub const RULE_boolLit: usize = 85;
pub const RULE_numLit: usize = 86;
pub const RULE_stringLit: usize = 87;
pub const RULE_charLit: usize = 88;
pub const RULE_listLit: usize = 89;
pub const RULE_mapLit: usize = 90;
pub const RULE_mapPair: usize = 91;
pub const RULE_name: usize = 92;
pub const RULE_symbol: usize = 93;
pub const RULE_reservedWord: usize = 94;
pub const ruleNames: [&'static str; 95] = [
    "script",
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
                recog.base.set_state(190);
                recog.query()?;

                recog.base.set_state(192);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SEMI {
                    {
                        recog.base.set_state(191);
                        recog
                            .base
                            .match_token(CypherParser_SEMI, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(194);
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
        recog.base.enter_rule(_localctx.clone(), 2, RULE_query);
        let mut _localctx: Rc<QueryContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(200);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(1, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule explainSt*/
                        recog.base.set_state(196);
                        recog.explainSt()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule regularQuery*/
                        recog.base.set_state(197);
                        recog.regularQuery()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule standaloneCall*/
                        recog.base.set_state(198);
                        recog.standaloneCall()?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule createIndexSt*/
                        recog.base.set_state(199);
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
        recog.base.enter_rule(_localctx.clone(), 4, RULE_explainSt);
        let mut _localctx: Rc<ExplainStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(202);
                recog
                    .base
                    .match_token(CypherParser_EXPLAIN, &mut recog.err_handler)?;

                recog.base.set_state(205);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.interpreter.adaptive_predict(2, &mut recog.base)? {
                    1 => {
                        {
                            /*InvokeRule createIndexSt*/
                            recog.base.set_state(203);
                            recog.createIndexSt()?;
                        }
                    }
                    2 => {
                        {
                            /*InvokeRule regularQuery*/
                            recog.base.set_state(204);
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
            .enter_rule(_localctx.clone(), 6, RULE_createIndexSt);
        let mut _localctx: Rc<CreateIndexStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(207);
                recog
                    .base
                    .match_token(CypherParser_CREATE, &mut recog.err_handler)?;

                recog.base.set_state(208);
                recog
                    .base
                    .match_token(CypherParser_INDEX, &mut recog.err_handler)?;

                recog.base.set_state(209);
                recog
                    .base
                    .match_token(CypherParser_ON, &mut recog.err_handler)?;

                recog.base.set_state(210);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(211);
                recog.name()?;

                recog.base.set_state(212);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(213);
                recog.name()?;

                recog.base.set_state(214);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;

                recog.base.set_state(216);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_UNIQUE {
                    {
                        recog.base.set_state(215);
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
            .enter_rule(_localctx.clone(), 8, RULE_regularQuery);
        let mut _localctx: Rc<RegularQueryContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule singleQuery*/
                recog.base.set_state(218);
                recog.singleQuery()?;

                recog.base.set_state(222);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_UNION {
                    {
                        {
                            /*InvokeRule unionSt*/
                            recog.base.set_state(219);
                            recog.unionSt()?;
                        }
                    }
                    recog.base.set_state(224);
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
            .enter_rule(_localctx.clone(), 10, RULE_singleQuery);
        let mut _localctx: Rc<SingleQueryContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(227);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(5, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule singlePartQ*/
                        recog.base.set_state(225);
                        recog.singlePartQ()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule multiPartQ*/
                        recog.base.set_state(226);
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
            .enter_rule(_localctx.clone(), 12, RULE_standaloneCall);
        let mut _localctx: Rc<StandaloneCallContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(229);
                recog
                    .base
                    .match_token(CypherParser_CALL, &mut recog.err_handler)?;

                /*InvokeRule invocationName*/
                recog.base.set_state(230);
                recog.invocationName()?;

                recog.base.set_state(232);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LPAREN {
                    {
                        /*InvokeRule parenExpressionChain*/
                        recog.base.set_state(231);
                        recog.parenExpressionChain()?;
                    }
                }

                recog.base.set_state(239);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_YIELD {
                    {
                        recog.base.set_state(234);
                        recog
                            .base
                            .match_token(CypherParser_YIELD, &mut recog.err_handler)?;

                        recog.base.set_state(237);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.base.input.la(1) {
                            CypherParser_MULT => {
                                recog.base.set_state(235);
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
                                    recog.base.set_state(236);
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
        recog.base.enter_rule(_localctx.clone(), 14, RULE_returnSt);
        let mut _localctx: Rc<ReturnStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(241);
                recog
                    .base
                    .match_token(CypherParser_RETURN, &mut recog.err_handler)?;

                /*InvokeRule projectionBody*/
                recog.base.set_state(242);
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
        recog.base.enter_rule(_localctx.clone(), 16, RULE_withSt);
        let mut _localctx: Rc<WithStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(244);
                recog
                    .base
                    .match_token(CypherParser_WITH, &mut recog.err_handler)?;

                /*InvokeRule projectionBody*/
                recog.base.set_state(245);
                recog.projectionBody()?;

                recog.base.set_state(247);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(246);
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
        recog.base.enter_rule(_localctx.clone(), 18, RULE_skipSt);
        let mut _localctx: Rc<SkipStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(249);
                recog
                    .base
                    .match_token(CypherParser_SKIP_W, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(250);
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
        recog.base.enter_rule(_localctx.clone(), 20, RULE_limitSt);
        let mut _localctx: Rc<LimitStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(252);
                recog
                    .base
                    .match_token(CypherParser_LIMIT, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(253);
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
            .enter_rule(_localctx.clone(), 22, RULE_projectionBody);
        let mut _localctx: Rc<ProjectionBodyContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(256);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DISTINCT {
                    {
                        recog.base.set_state(255);
                        recog
                            .base
                            .match_token(CypherParser_DISTINCT, &mut recog.err_handler)?;
                    }
                }

                /*InvokeRule projectionItems*/
                recog.base.set_state(258);
                recog.projectionItems()?;

                recog.base.set_state(260);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ORDER {
                    {
                        /*InvokeRule orderSt*/
                        recog.base.set_state(259);
                        recog.orderSt()?;
                    }
                }

                recog.base.set_state(263);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SKIP_W {
                    {
                        /*InvokeRule skipSt*/
                        recog.base.set_state(262);
                        recog.skipSt()?;
                    }
                }

                recog.base.set_state(266);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LIMIT {
                    {
                        /*InvokeRule limitSt*/
                        recog.base.set_state(265);
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
            .enter_rule(_localctx.clone(), 24, RULE_projectionItems);
        let mut _localctx: Rc<ProjectionItemsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(270);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_MULT => {
                        recog.base.set_state(268);
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
                            recog.base.set_state(269);
                            recog.projectionItem()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
                recog.base.set_state(276);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(272);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule projectionItem*/
                            recog.base.set_state(273);
                            recog.projectionItem()?;
                        }
                    }
                    recog.base.set_state(278);
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
            .enter_rule(_localctx.clone(), 26, RULE_projectionItem);
        let mut _localctx: Rc<ProjectionItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(279);
                recog.expression()?;

                recog.base.set_state(282);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_AS {
                    {
                        recog.base.set_state(280);
                        recog
                            .base
                            .match_token(CypherParser_AS, &mut recog.err_handler)?;

                        /*InvokeRule symbol*/
                        recog.base.set_state(281);
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
        recog.base.enter_rule(_localctx.clone(), 28, RULE_orderItem);
        let mut _localctx: Rc<OrderItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(284);
                recog.expression()?;

                recog.base.set_state(286);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 37) & !0x3f) == 0 && ((1usize << (_la - 37)) & 99) != 0) {
                    {
                        recog.base.set_state(285);
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
        recog.base.enter_rule(_localctx.clone(), 30, RULE_orderSt);
        let mut _localctx: Rc<OrderStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(288);
                recog
                    .base
                    .match_token(CypherParser_ORDER, &mut recog.err_handler)?;

                recog.base.set_state(289);
                recog
                    .base
                    .match_token(CypherParser_BY, &mut recog.err_handler)?;

                /*InvokeRule orderItem*/
                recog.base.set_state(290);
                recog.orderItem()?;

                recog.base.set_state(295);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(291);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule orderItem*/
                            recog.base.set_state(292);
                            recog.orderItem()?;
                        }
                    }
                    recog.base.set_state(297);
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
            .enter_rule(_localctx.clone(), 32, RULE_singlePartQ);
        let mut _localctx: Rc<SinglePartQContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(301);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_CALL
                    || (((_la - 48) & !0x3f) == 0 && ((1usize << (_la - 48)) & 4105) != 0)
                {
                    {
                        {
                            /*InvokeRule readingStatement*/
                            recog.base.set_state(298);
                            recog.readingStatement()?;
                        }
                    }
                    recog.base.set_state(303);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(313);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_RETURN => {
                        {
                            /*InvokeRule returnSt*/
                            recog.base.set_state(304);
                            recog.returnSt()?;
                        }
                    }

                    CypherParser_CREATE | CypherParser_DELETE | CypherParser_DETACH
                    | CypherParser_MERGE | CypherParser_REMOVE | CypherParser_SET => {
                        {
                            recog.base.set_state(306);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            loop {
                                {
                                    {
                                        /*InvokeRule updatingStatement*/
                                        recog.base.set_state(305);
                                        recog.updatingStatement()?;
                                    }
                                }
                                recog.base.set_state(308);
                                recog.err_handler.sync(&mut recog.base)?;
                                _la = recog.base.input.la(1);
                                if !(((_la - 40) & !0x3f) == 0
                                    && ((1usize << (_la - 40)) & 41491) != 0)
                                {
                                    break;
                                }
                            }
                            recog.base.set_state(311);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            if _la == CypherParser_RETURN {
                                {
                                    /*InvokeRule returnSt*/
                                    recog.base.set_state(310);
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
            .enter_rule(_localctx.clone(), 34, RULE_multiPartQ);
        let mut _localctx: Rc<MultiPartQContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            let mut _alt: i32;
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(318);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = recog.interpreter.adaptive_predict(23, &mut recog.base)?;
                while { _alt != 2 && _alt != INVALID_ALT } {
                    if _alt == 1 {
                        {
                            {
                                /*InvokeRule readingStatement*/
                                recog.base.set_state(315);
                                recog.readingStatement()?;
                            }
                        }
                    }
                    recog.base.set_state(320);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(23, &mut recog.base)?;
                }
                recog.base.set_state(329);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = 1;
                loop {
                    match _alt {
                        x if x == 1 => {
                            {
                                recog.base.set_state(325);
                                recog.err_handler.sync(&mut recog.base)?;
                                _la = recog.base.input.la(1);
                                while _la == CypherParser_CALL
                                    || (((_la - 40) & !0x3f) == 0
                                        && ((1usize << (_la - 40)) & 1092371) != 0)
                                {
                                    {
                                        recog.base.set_state(323);
                                        recog.err_handler.sync(&mut recog.base)?;
                                        match recog.base.input.la(1) {
                                            CypherParser_CALL
                                            | CypherParser_MATCH
                                            | CypherParser_OPTIONAL
                                            | CypherParser_UNWIND => {
                                                {
                                                    /*InvokeRule readingStatement*/
                                                    recog.base.set_state(321);
                                                    recog.readingStatement()?;
                                                }
                                            }

                                            CypherParser_CREATE | CypherParser_DELETE
                                            | CypherParser_DETACH | CypherParser_MERGE
                                            | CypherParser_REMOVE | CypherParser_SET => {
                                                {
                                                    /*InvokeRule updatingStatement*/
                                                    recog.base.set_state(322);
                                                    recog.updatingStatement()?;
                                                }
                                            }

                                            _ => Err(ANTLRError::NoAltError(
                                                NoViableAltError::new(&mut recog.base),
                                            ))?,
                                        }
                                    }
                                    recog.base.set_state(327);
                                    recog.err_handler.sync(&mut recog.base)?;
                                    _la = recog.base.input.la(1);
                                }
                                /*InvokeRule withSt*/
                                recog.base.set_state(328);
                                recog.withSt()?;
                            }
                        }

                        _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                            &mut recog.base,
                        )))?,
                    }
                    recog.base.set_state(331);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(26, &mut recog.base)?;
                    if _alt == 2 || _alt == INVALID_ALT {
                        break;
                    }
                }
                /*InvokeRule singlePartQ*/
                recog.base.set_state(333);
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
        recog.base.enter_rule(_localctx.clone(), 36, RULE_matchSt);
        let mut _localctx: Rc<MatchStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(336);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_OPTIONAL {
                    {
                        recog.base.set_state(335);
                        recog
                            .base
                            .match_token(CypherParser_OPTIONAL, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(338);
                recog
                    .base
                    .match_token(CypherParser_MATCH, &mut recog.err_handler)?;

                /*InvokeRule patternWhere*/
                recog.base.set_state(339);
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
        recog.base.enter_rule(_localctx.clone(), 38, RULE_unwindSt);
        let mut _localctx: Rc<UnwindStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(341);
                recog
                    .base
                    .match_token(CypherParser_UNWIND, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(342);
                recog.expression()?;

                recog.base.set_state(343);
                recog
                    .base
                    .match_token(CypherParser_AS, &mut recog.err_handler)?;

                /*InvokeRule symbol*/
                recog.base.set_state(344);
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
            .enter_rule(_localctx.clone(), 40, RULE_readingStatement);
        let mut _localctx: Rc<ReadingStatementContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(349);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_MATCH | CypherParser_OPTIONAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule matchSt*/
                        recog.base.set_state(346);
                        recog.matchSt()?;
                    }
                }

                CypherParser_UNWIND => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule unwindSt*/
                        recog.base.set_state(347);
                        recog.unwindSt()?;
                    }
                }

                CypherParser_CALL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule queryCallSt*/
                        recog.base.set_state(348);
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
            .enter_rule(_localctx.clone(), 42, RULE_updatingStatement);
        let mut _localctx: Rc<UpdatingStatementContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(356);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_CREATE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule createSt*/
                        recog.base.set_state(351);
                        recog.createSt()?;
                    }
                }

                CypherParser_MERGE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule mergeSt*/
                        recog.base.set_state(352);
                        recog.mergeSt()?;
                    }
                }

                CypherParser_DELETE | CypherParser_DETACH => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule deleteSt*/
                        recog.base.set_state(353);
                        recog.deleteSt()?;
                    }
                }

                CypherParser_SET => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule setSt*/
                        recog.base.set_state(354);
                        recog.setSt()?;
                    }
                }

                CypherParser_REMOVE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule removeSt*/
                        recog.base.set_state(355);
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
        recog.base.enter_rule(_localctx.clone(), 44, RULE_deleteSt);
        let mut _localctx: Rc<DeleteStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(359);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DETACH {
                    {
                        recog.base.set_state(358);
                        recog
                            .base
                            .match_token(CypherParser_DETACH, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(361);
                recog
                    .base
                    .match_token(CypherParser_DELETE, &mut recog.err_handler)?;

                /*InvokeRule expressionChain*/
                recog.base.set_state(362);
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
        recog.base.enter_rule(_localctx.clone(), 46, RULE_removeSt);
        let mut _localctx: Rc<RemoveStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(364);
                recog
                    .base
                    .match_token(CypherParser_REMOVE, &mut recog.err_handler)?;

                /*InvokeRule removeItem*/
                recog.base.set_state(365);
                recog.removeItem()?;

                recog.base.set_state(370);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(366);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule removeItem*/
                            recog.base.set_state(367);
                            recog.removeItem()?;
                        }
                    }
                    recog.base.set_state(372);
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
            .enter_rule(_localctx.clone(), 48, RULE_removeItem);
        let mut _localctx: Rc<RemoveItemContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(377);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(32, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(373);
                        recog.symbol()?;

                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(374);
                        recog.nodeLabels()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule propertyExpression*/
                        recog.base.set_state(376);
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
            .enter_rule(_localctx.clone(), 50, RULE_queryCallSt);
        let mut _localctx: Rc<QueryCallStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(379);
                recog
                    .base
                    .match_token(CypherParser_CALL, &mut recog.err_handler)?;

                /*InvokeRule invocationName*/
                recog.base.set_state(380);
                recog.invocationName()?;

                /*InvokeRule parenExpressionChain*/
                recog.base.set_state(381);
                recog.parenExpressionChain()?;

                recog.base.set_state(384);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_YIELD {
                    {
                        recog.base.set_state(382);
                        recog
                            .base
                            .match_token(CypherParser_YIELD, &mut recog.err_handler)?;

                        /*InvokeRule yieldItems*/
                        recog.base.set_state(383);
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
            .enter_rule(_localctx.clone(), 52, RULE_parenExpressionChain);
        let mut _localctx: Rc<ParenExpressionChainContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(386);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(388);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(387);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(390);
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
            .enter_rule(_localctx.clone(), 54, RULE_yieldItems);
        let mut _localctx: Rc<YieldItemsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule yieldItem*/
                recog.base.set_state(392);
                recog.yieldItem()?;

                recog.base.set_state(397);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(393);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule yieldItem*/
                            recog.base.set_state(394);
                            recog.yieldItem()?;
                        }
                    }
                    recog.base.set_state(399);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                recog.base.set_state(401);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(400);
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
        recog.base.enter_rule(_localctx.clone(), 56, RULE_yieldItem);
        let mut _localctx: Rc<YieldItemContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(406);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.interpreter.adaptive_predict(37, &mut recog.base)? {
                    x if x == 1 => {
                        {
                            /*InvokeRule symbol*/
                            recog.base.set_state(403);
                            recog.symbol()?;

                            recog.base.set_state(404);
                            recog
                                .base
                                .match_token(CypherParser_AS, &mut recog.err_handler)?;
                        }
                    }

                    _ => {}
                }
                /*InvokeRule symbol*/
                recog.base.set_state(408);
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
        recog.base.enter_rule(_localctx.clone(), 58, RULE_mergeSt);
        let mut _localctx: Rc<MergeStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(410);
                recog
                    .base
                    .match_token(CypherParser_MERGE, &mut recog.err_handler)?;

                /*InvokeRule patternPart*/
                recog.base.set_state(411);
                recog.patternPart()?;

                recog.base.set_state(415);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_ON {
                    {
                        {
                            /*InvokeRule mergeAction*/
                            recog.base.set_state(412);
                            recog.mergeAction()?;
                        }
                    }
                    recog.base.set_state(417);
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
            .enter_rule(_localctx.clone(), 60, RULE_mergeAction);
        let mut _localctx: Rc<MergeActionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(418);
                recog
                    .base
                    .match_token(CypherParser_ON, &mut recog.err_handler)?;

                recog.base.set_state(419);
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
                recog.base.set_state(420);
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
        recog.base.enter_rule(_localctx.clone(), 62, RULE_setSt);
        let mut _localctx: Rc<SetStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(422);
                recog
                    .base
                    .match_token(CypherParser_SET, &mut recog.err_handler)?;

                /*InvokeRule setItem*/
                recog.base.set_state(423);
                recog.setItem()?;

                recog.base.set_state(428);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(424);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule setItem*/
                            recog.base.set_state(425);
                            recog.setItem()?;
                        }
                    }
                    recog.base.set_state(430);
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
        recog.base.enter_rule(_localctx.clone(), 64, RULE_setItem);
        let mut _localctx: Rc<SetItemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(442);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(40, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule propertyExpression*/
                        recog.base.set_state(431);
                        recog.propertyExpression()?;

                        recog.base.set_state(432);
                        recog
                            .base
                            .match_token(CypherParser_ASSIGN, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(433);
                        recog.expression()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(435);
                        recog.symbol()?;

                        recog.base.set_state(436);
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
                        recog.base.set_state(437);
                        recog.expression()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(439);
                        recog.symbol()?;

                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(440);
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
            .enter_rule(_localctx.clone(), 66, RULE_nodeLabels);
        let mut _localctx: Rc<NodeLabelsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(446);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                loop {
                    {
                        {
                            recog.base.set_state(444);
                            recog
                                .base
                                .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                            /*InvokeRule name*/
                            recog.base.set_state(445);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(448);
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
        recog.base.enter_rule(_localctx.clone(), 68, RULE_createSt);
        let mut _localctx: Rc<CreateStContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(450);
                recog
                    .base
                    .match_token(CypherParser_CREATE, &mut recog.err_handler)?;

                /*InvokeRule pattern*/
                recog.base.set_state(451);
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
            .enter_rule(_localctx.clone(), 70, RULE_patternWhere);
        let mut _localctx: Rc<PatternWhereContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule pattern*/
                recog.base.set_state(453);
                recog.pattern()?;

                recog.base.set_state(455);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(454);
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
        recog.base.enter_rule(_localctx.clone(), 72, RULE_where);
        let mut _localctx: Rc<WhereContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(457);
                recog
                    .base
                    .match_token(CypherParser_WHERE, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(458);
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
        recog.base.enter_rule(_localctx.clone(), 74, RULE_pattern);
        let mut _localctx: Rc<PatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule patternPart*/
                recog.base.set_state(460);
                recog.patternPart()?;

                recog.base.set_state(465);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(461);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule patternPart*/
                            recog.base.set_state(462);
                            recog.patternPart()?;
                        }
                    }
                    recog.base.set_state(467);
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
            .enter_rule(_localctx.clone(), 76, RULE_expression);
        let mut _localctx: Rc<ExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule xorExpression*/
                recog.base.set_state(468);
                recog.xorExpression()?;

                recog.base.set_state(473);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_OR {
                    {
                        {
                            recog.base.set_state(469);
                            recog
                                .base
                                .match_token(CypherParser_OR, &mut recog.err_handler)?;

                            /*InvokeRule xorExpression*/
                            recog.base.set_state(470);
                            recog.xorExpression()?;
                        }
                    }
                    recog.base.set_state(475);
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
            .enter_rule(_localctx.clone(), 78, RULE_xorExpression);
        let mut _localctx: Rc<XorExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule andExpression*/
                recog.base.set_state(476);
                recog.andExpression()?;

                recog.base.set_state(481);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_XOR {
                    {
                        {
                            recog.base.set_state(477);
                            recog
                                .base
                                .match_token(CypherParser_XOR, &mut recog.err_handler)?;

                            /*InvokeRule andExpression*/
                            recog.base.set_state(478);
                            recog.andExpression()?;
                        }
                    }
                    recog.base.set_state(483);
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
            .enter_rule(_localctx.clone(), 80, RULE_andExpression);
        let mut _localctx: Rc<AndExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule notExpression*/
                recog.base.set_state(484);
                recog.notExpression()?;

                recog.base.set_state(489);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_AND {
                    {
                        {
                            recog.base.set_state(485);
                            recog
                                .base
                                .match_token(CypherParser_AND, &mut recog.err_handler)?;

                            /*InvokeRule notExpression*/
                            recog.base.set_state(486);
                            recog.notExpression()?;
                        }
                    }
                    recog.base.set_state(491);
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
            .enter_rule(_localctx.clone(), 82, RULE_notExpression);
        let mut _localctx: Rc<NotExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(495);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_NOT {
                    {
                        {
                            recog.base.set_state(492);
                            recog
                                .base
                                .match_token(CypherParser_NOT, &mut recog.err_handler)?;
                        }
                    }
                    recog.base.set_state(497);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                }
                /*InvokeRule comparisonExpression*/
                recog.base.set_state(498);
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
            .enter_rule(_localctx.clone(), 84, RULE_comparisonExpression);
        let mut _localctx: Rc<ComparisonExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule addSubExpression*/
                recog.base.set_state(500);
                recog.addSubExpression()?;

                recog.base.set_state(506);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la) & !0x3f) == 0 && ((1usize << _la) & 250) != 0) {
                    {
                        {
                            /*InvokeRule comparisonSigns*/
                            recog.base.set_state(501);
                            recog.comparisonSigns()?;

                            /*InvokeRule addSubExpression*/
                            recog.base.set_state(502);
                            recog.addSubExpression()?;
                        }
                    }
                    recog.base.set_state(508);
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
            .enter_rule(_localctx.clone(), 86, RULE_comparisonSigns);
        let mut _localctx: Rc<ComparisonSignsContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(509);
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
            .enter_rule(_localctx.clone(), 88, RULE_addSubExpression);
        let mut _localctx: Rc<AddSubExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule multDivExpression*/
                recog.base.set_state(511);
                recog.multDivExpression()?;

                recog.base.set_state(516);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_SUB || _la == CypherParser_PLUS {
                    {
                        {
                            recog.base.set_state(512);
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
                            recog.base.set_state(513);
                            recog.multDivExpression()?;
                        }
                    }
                    recog.base.set_state(518);
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
            .enter_rule(_localctx.clone(), 90, RULE_multDivExpression);
        let mut _localctx: Rc<MultDivExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule powerExpression*/
                recog.base.set_state(519);
                recog.powerExpression()?;

                recog.base.set_state(524);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while (((_la) & !0x3f) == 0 && ((1usize << _la) & 11534336) != 0) {
                    {
                        {
                            recog.base.set_state(520);
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
                            recog.base.set_state(521);
                            recog.powerExpression()?;
                        }
                    }
                    recog.base.set_state(526);
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
            .enter_rule(_localctx.clone(), 92, RULE_powerExpression);
        let mut _localctx: Rc<PowerExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule unaryAddSubExpression*/
                recog.base.set_state(527);
                recog.unaryAddSubExpression()?;

                recog.base.set_state(532);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_CARET {
                    {
                        {
                            recog.base.set_state(528);
                            recog
                                .base
                                .match_token(CypherParser_CARET, &mut recog.err_handler)?;

                            /*InvokeRule unaryAddSubExpression*/
                            recog.base.set_state(529);
                            recog.unaryAddSubExpression()?;
                        }
                    }
                    recog.base.set_state(534);
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
            .enter_rule(_localctx.clone(), 94, RULE_unaryAddSubExpression);
        let mut _localctx: Rc<UnaryAddSubExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(536);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_SUB || _la == CypherParser_PLUS {
                    {
                        recog.base.set_state(535);
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
                recog.base.set_state(538);
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
            .enter_rule(_localctx.clone(), 96, RULE_atomicExpression);
        let mut _localctx: Rc<AtomicExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule propertyOrLabelExpression*/
                recog.base.set_state(540);
                recog.propertyOrLabelExpression()?;

                recog.base.set_state(546);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_LBRACK
                    || (((_la - 63) & !0x3f) == 0 && ((1usize << (_la - 63)) & 301) != 0)
                {
                    {
                        recog.base.set_state(544);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.base.input.la(1) {
                            CypherParser_CONTAINS | CypherParser_ENDS | CypherParser_STARTS => {
                                {
                                    /*InvokeRule stringExpression*/
                                    recog.base.set_state(541);
                                    recog.stringExpression()?;
                                }
                            }

                            CypherParser_LBRACK | CypherParser_IN => {
                                {
                                    /*InvokeRule listExpression*/
                                    recog.base.set_state(542);
                                    recog.listExpression()?;
                                }
                            }

                            CypherParser_IS => {
                                {
                                    /*InvokeRule nullExpression*/
                                    recog.base.set_state(543);
                                    recog.nullExpression()?;
                                }
                            }

                            _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                                &mut recog.base,
                            )))?,
                        }
                    }
                    recog.base.set_state(548);
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
            .enter_rule(_localctx.clone(), 98, RULE_listExpression);
        let mut _localctx: Rc<ListExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(563);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_IN => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(549);
                        recog
                            .base
                            .match_token(CypherParser_IN, &mut recog.err_handler)?;

                        /*InvokeRule propertyOrLabelExpression*/
                        recog.base.set_state(550);
                        recog.propertyOrLabelExpression()?;
                    }
                }

                CypherParser_LBRACK => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(551);
                        recog
                            .base
                            .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                        recog.base.set_state(560);
                        recog.err_handler.sync(&mut recog.base)?;
                        match recog.interpreter.adaptive_predict(57, &mut recog.base)? {
                            1 => {
                                {
                                    recog.base.set_state(553);
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
                                            recog.base.set_state(552);
                                            recog.expression()?;
                                        }
                                    }

                                    recog.base.set_state(555);
                                    recog
                                        .base
                                        .match_token(CypherParser_RANGE, &mut recog.err_handler)?;

                                    recog.base.set_state(557);
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
                                            recog.base.set_state(556);
                                            recog.expression()?;
                                        }
                                    }
                                }
                            }
                            2 => {
                                {
                                    /*InvokeRule expression*/
                                    recog.base.set_state(559);
                                    recog.expression()?;
                                }
                            }

                            _ => {}
                        }
                        recog.base.set_state(562);
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
            .enter_rule(_localctx.clone(), 100, RULE_stringExpression);
        let mut _localctx: Rc<StringExpressionContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule stringExpPrefix*/
                recog.base.set_state(565);
                recog.stringExpPrefix()?;

                /*InvokeRule propertyOrLabelExpression*/
                recog.base.set_state(566);
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
            .enter_rule(_localctx.clone(), 102, RULE_stringExpPrefix);
        let mut _localctx: Rc<StringExpPrefixContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(573);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_STARTS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(568);
                        recog
                            .base
                            .match_token(CypherParser_STARTS, &mut recog.err_handler)?;

                        recog.base.set_state(569);
                        recog
                            .base
                            .match_token(CypherParser_WITH, &mut recog.err_handler)?;
                    }
                }

                CypherParser_ENDS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(570);
                        recog
                            .base
                            .match_token(CypherParser_ENDS, &mut recog.err_handler)?;

                        recog.base.set_state(571);
                        recog
                            .base
                            .match_token(CypherParser_WITH, &mut recog.err_handler)?;
                    }
                }

                CypherParser_CONTAINS => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(572);
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
            .enter_rule(_localctx.clone(), 104, RULE_nullExpression);
        let mut _localctx: Rc<NullExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(575);
                recog
                    .base
                    .match_token(CypherParser_IS, &mut recog.err_handler)?;

                recog.base.set_state(577);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_NOT {
                    {
                        recog.base.set_state(576);
                        recog
                            .base
                            .match_token(CypherParser_NOT, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(579);
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
            .enter_rule(_localctx.clone(), 106, RULE_propertyOrLabelExpression);
        let mut _localctx: Rc<PropertyOrLabelExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule propertyExpression*/
                recog.base.set_state(581);
                recog.propertyExpression()?;

                recog.base.set_state(583);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(582);
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
            .enter_rule(_localctx.clone(), 108, RULE_propertyExpression);
        let mut _localctx: Rc<PropertyExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule atom*/
                recog.base.set_state(585);
                recog.atom()?;

                recog.base.set_state(590);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_DOT {
                    {
                        {
                            recog.base.set_state(586);
                            recog
                                .base
                                .match_token(CypherParser_DOT, &mut recog.err_handler)?;

                            /*InvokeRule name*/
                            recog.base.set_state(587);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(592);
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
            .enter_rule(_localctx.clone(), 110, RULE_patternPart);
        let mut _localctx: Rc<PatternPartContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(596);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(593);
                        recog.symbol()?;

                        recog.base.set_state(594);
                        recog
                            .base
                            .match_token(CypherParser_ASSIGN, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(600);
                recog.err_handler.sync(&mut recog.base)?;
                match recog.base.input.la(1) {
                    CypherParser_SHORTEST_PATH => {
                        {
                            /*InvokeRule shortestPathWrapper*/
                            recog.base.set_state(598);
                            recog.shortestPathWrapper()?;
                        }
                    }

                    CypherParser_LPAREN => {
                        {
                            /*InvokeRule patternElem*/
                            recog.base.set_state(599);
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
            .enter_rule(_localctx.clone(), 112, RULE_shortestPathWrapper);
        let mut _localctx: Rc<ShortestPathWrapperContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(602);
                recog
                    .base
                    .match_token(CypherParser_SHORTEST_PATH, &mut recog.err_handler)?;

                recog.base.set_state(603);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule patternElem*/
                recog.base.set_state(604);
                recog.patternElem()?;

                recog.base.set_state(605);
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
            .enter_rule(_localctx.clone(), 114, RULE_patternElem);
        let mut _localctx: Rc<PatternElemContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(621);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(68, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule nodePattern*/
                        recog.base.set_state(607);
                        recog.nodePattern()?;

                        recog.base.set_state(612);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        while (((_la) & !0x3f) == 0 && ((1usize << _la) & 266304) != 0) {
                            {
                                recog.base.set_state(610);
                                recog.err_handler.sync(&mut recog.base)?;
                                match recog.base.input.la(1) {
                                    CypherParser_LT | CypherParser_SUB => {
                                        {
                                            /*InvokeRule patternElemChain*/
                                            recog.base.set_state(608);
                                            recog.patternElemChain()?;
                                        }
                                    }

                                    CypherParser_LPAREN => {
                                        {
                                            /*InvokeRule qppElemChain*/
                                            recog.base.set_state(609);
                                            recog.qppElemChain()?;
                                        }
                                    }

                                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                                        &mut recog.base,
                                    )))?,
                                }
                            }
                            recog.base.set_state(614);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                        }
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
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

                        recog.base.set_state(619);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if (((_la) & !0x3f) == 0 && ((1usize << _la) & 8929280) != 0) {
                            {
                                /*InvokeRule qppQuantifier*/
                                recog.base.set_state(618);
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
            .enter_rule(_localctx.clone(), 116, RULE_patternElemChain);
        let mut _localctx: Rc<PatternElemChainContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule relationshipPattern*/
                recog.base.set_state(623);
                recog.relationshipPattern()?;

                /*InvokeRule nodePattern*/
                recog.base.set_state(624);
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
            .enter_rule(_localctx.clone(), 118, RULE_qppElemChain);
        let mut _localctx: Rc<QppElemChainContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(626);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule patternElem*/
                recog.base.set_state(627);
                recog.patternElem()?;

                recog.base.set_state(628);
                recog
                    .base
                    .match_token(CypherParser_RPAREN, &mut recog.err_handler)?;

                /*InvokeRule qppQuantifier*/
                recog.base.set_state(629);
                recog.qppQuantifier()?;

                /*InvokeRule nodePattern*/
                recog.base.set_state(630);
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
            .enter_rule(_localctx.clone(), 120, RULE_qppQuantifier);
        let mut _localctx: Rc<QppQuantifierContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(657);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(69, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(632);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(633);
                        recog.qppInt()?;

                        recog.base.set_state(634);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(635);
                        recog.qppInt()?;

                        recog.base.set_state(636);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        recog.base.set_state(638);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(639);
                        recog.qppInt()?;

                        recog.base.set_state(640);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(642);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(643);
                        recog.qppInt()?;

                        recog.base.set_state(644);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        recog.base.set_state(645);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        recog.base.set_state(647);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        recog.base.set_state(648);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        /*InvokeRule qppInt*/
                        recog.base.set_state(649);
                        recog.qppInt()?;

                        recog.base.set_state(650);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                5 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        recog.base.set_state(652);
                        recog
                            .base
                            .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                        recog.base.set_state(653);
                        recog
                            .base
                            .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                        recog.base.set_state(654);
                        recog
                            .base
                            .match_token(CypherParser_RBRACE, &mut recog.err_handler)?;
                    }
                }
                6 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        recog.base.set_state(655);
                        recog
                            .base
                            .match_token(CypherParser_PLUS, &mut recog.err_handler)?;
                    }
                }
                7 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        recog.base.set_state(656);
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
        recog.base.enter_rule(_localctx.clone(), 122, RULE_qppInt);
        let mut _localctx: Rc<QppIntContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(659);
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
            .enter_rule(_localctx.clone(), 124, RULE_properties);
        let mut _localctx: Rc<PropertiesContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(663);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_LBRACE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule mapLit*/
                        recog.base.set_state(661);
                        recog.mapLit()?;
                    }
                }

                CypherParser_DOLLAR => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule parameter*/
                        recog.base.set_state(662);
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
            .enter_rule(_localctx.clone(), 126, RULE_nodePattern);
        let mut _localctx: Rc<NodePatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(665);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(667);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(666);
                        recog.symbol()?;
                    }
                }

                recog.base.set_state(670);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule nodeLabels*/
                        recog.base.set_state(669);
                        recog.nodeLabels()?;
                    }
                }

                recog.base.set_state(673);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LBRACE || _la == CypherParser_DOLLAR {
                    {
                        /*InvokeRule properties*/
                        recog.base.set_state(672);
                        recog.properties()?;
                    }
                }

                recog.base.set_state(675);
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
        recog.base.enter_rule(_localctx.clone(), 128, RULE_atom);
        let mut _localctx: Rc<AtomContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(689);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(74, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule literal*/
                        recog.base.set_state(677);
                        recog.literal()?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule parameter*/
                        recog.base.set_state(678);
                        recog.parameter()?;
                    }
                }
                3 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        /*InvokeRule caseExpression*/
                        recog.base.set_state(679);
                        recog.caseExpression()?;
                    }
                }
                4 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 4)?;
                    recog.base.enter_outer_alt(None, 4)?;
                    {
                        /*InvokeRule countAll*/
                        recog.base.set_state(680);
                        recog.countAll()?;
                    }
                }
                5 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule listComprehension*/
                        recog.base.set_state(681);
                        recog.listComprehension()?;
                    }
                }
                6 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        /*InvokeRule patternComprehension*/
                        recog.base.set_state(682);
                        recog.patternComprehension()?;
                    }
                }
                7 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        /*InvokeRule filterWith*/
                        recog.base.set_state(683);
                        recog.filterWith()?;
                    }
                }
                8 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 8)?;
                    recog.base.enter_outer_alt(None, 8)?;
                    {
                        /*InvokeRule relationshipsChainPattern*/
                        recog.base.set_state(684);
                        recog.relationshipsChainPattern()?;
                    }
                }
                9 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 9)?;
                    recog.base.enter_outer_alt(None, 9)?;
                    {
                        /*InvokeRule parenthesizedExpression*/
                        recog.base.set_state(685);
                        recog.parenthesizedExpression()?;
                    }
                }
                10 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 10)?;
                    recog.base.enter_outer_alt(None, 10)?;
                    {
                        /*InvokeRule functionInvocation*/
                        recog.base.set_state(686);
                        recog.functionInvocation()?;
                    }
                }
                11 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 11)?;
                    recog.base.enter_outer_alt(None, 11)?;
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(687);
                        recog.symbol()?;
                    }
                }
                12 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 12)?;
                    recog.base.enter_outer_alt(None, 12)?;
                    {
                        /*InvokeRule subqueryExist*/
                        recog.base.set_state(688);
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
        recog.base.enter_rule(_localctx.clone(), 130, RULE_lhs);
        let mut _localctx: Rc<LhsContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(691);
                recog.symbol()?;

                recog.base.set_state(692);
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
            .enter_rule(_localctx.clone(), 132, RULE_relationshipPattern);
        let mut _localctx: Rc<RelationshipPatternContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(711);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_LT => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        recog.base.set_state(694);
                        recog
                            .base
                            .match_token(CypherParser_LT, &mut recog.err_handler)?;

                        recog.base.set_state(695);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(697);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_LBRACK {
                            {
                                /*InvokeRule relationDetail*/
                                recog.base.set_state(696);
                                recog.relationDetail()?;
                            }
                        }

                        recog.base.set_state(699);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(701);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_GT {
                            {
                                recog.base.set_state(700);
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
                        recog.base.set_state(703);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(705);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_LBRACK {
                            {
                                /*InvokeRule relationDetail*/
                                recog.base.set_state(704);
                                recog.relationDetail()?;
                            }
                        }

                        recog.base.set_state(707);
                        recog
                            .base
                            .match_token(CypherParser_SUB, &mut recog.err_handler)?;

                        recog.base.set_state(709);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_GT {
                            {
                                recog.base.set_state(708);
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
            .enter_rule(_localctx.clone(), 134, RULE_relationDetail);
        let mut _localctx: Rc<RelationDetailContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(713);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(715);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule symbol*/
                        recog.base.set_state(714);
                        recog.symbol()?;
                    }
                }

                recog.base.set_state(718);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_COLON {
                    {
                        /*InvokeRule relationshipTypes*/
                        recog.base.set_state(717);
                        recog.relationshipTypes()?;
                    }
                }

                recog.base.set_state(721);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_MULT {
                    {
                        /*InvokeRule rangeLit*/
                        recog.base.set_state(720);
                        recog.rangeLit()?;
                    }
                }

                recog.base.set_state(724);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_LBRACE || _la == CypherParser_DOLLAR {
                    {
                        /*InvokeRule properties*/
                        recog.base.set_state(723);
                        recog.properties()?;
                    }
                }

                recog.base.set_state(726);
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
            .enter_rule(_localctx.clone(), 136, RULE_relationshipTypes);
        let mut _localctx: Rc<RelationshipTypesContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(728);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule name*/
                recog.base.set_state(729);
                recog.name()?;

                recog.base.set_state(737);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_STICK {
                    {
                        {
                            recog.base.set_state(730);
                            recog
                                .base
                                .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                            recog.base.set_state(732);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            if _la == CypherParser_COLON {
                                {
                                    recog.base.set_state(731);
                                    recog
                                        .base
                                        .match_token(CypherParser_COLON, &mut recog.err_handler)?;
                                }
                            }

                            /*InvokeRule name*/
                            recog.base.set_state(734);
                            recog.name()?;
                        }
                    }
                    recog.base.set_state(739);
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
        recog.base.enter_rule(_localctx.clone(), 138, RULE_unionSt);
        let mut _localctx: Rc<UnionStContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(740);
                recog
                    .base
                    .match_token(CypherParser_UNION, &mut recog.err_handler)?;

                recog.base.set_state(742);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ALL {
                    {
                        recog.base.set_state(741);
                        recog
                            .base
                            .match_token(CypherParser_ALL, &mut recog.err_handler)?;
                    }
                }

                /*InvokeRule singleQuery*/
                recog.base.set_state(744);
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
            .enter_rule(_localctx.clone(), 140, RULE_subqueryExist);
        let mut _localctx: Rc<SubqueryExistContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(746);
                recog
                    .base
                    .match_token(CypherParser_EXISTS, &mut recog.err_handler)?;

                recog.base.set_state(747);
                recog
                    .base
                    .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                recog.base.set_state(750);
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
                            recog.base.set_state(748);
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
                            recog.base.set_state(749);
                            recog.patternWhere()?;
                        }
                    }

                    _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                        &mut recog.base,
                    )))?,
                }
                recog.base.set_state(752);
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
            .enter_rule(_localctx.clone(), 142, RULE_invocationName);
        let mut _localctx: Rc<InvocationNameContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(754);
                recog.symbol()?;

                recog.base.set_state(759);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_DOT {
                    {
                        {
                            recog.base.set_state(755);
                            recog
                                .base
                                .match_token(CypherParser_DOT, &mut recog.err_handler)?;

                            /*InvokeRule symbol*/
                            recog.base.set_state(756);
                            recog.symbol()?;
                        }
                    }
                    recog.base.set_state(761);
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
            .enter_rule(_localctx.clone(), 144, RULE_functionInvocation);
        let mut _localctx: Rc<FunctionInvocationContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule invocationName*/
                recog.base.set_state(762);
                recog.invocationName()?;

                recog.base.set_state(763);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(765);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DISTINCT {
                    {
                        recog.base.set_state(764);
                        recog
                            .base
                            .match_token(CypherParser_DISTINCT, &mut recog.err_handler)?;
                    }
                }

                recog.base.set_state(768);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(767);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(770);
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
            .enter_rule(_localctx.clone(), 146, RULE_parenthesizedExpression);
        let mut _localctx: Rc<ParenthesizedExpressionContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(772);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(773);
                recog.expression()?;

                recog.base.set_state(774);
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
            .enter_rule(_localctx.clone(), 148, RULE_filterWith);
        let mut _localctx: Rc<FilterWithContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(776);
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
                recog.base.set_state(777);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                /*InvokeRule filterExpression*/
                recog.base.set_state(778);
                recog.filterExpression()?;

                recog.base.set_state(779);
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
            .enter_rule(_localctx.clone(), 150, RULE_patternComprehension);
        let mut _localctx: Rc<PatternComprehensionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(781);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(783);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 63) != 0)
                    || _la == CypherParser_ID
                    || _la == CypherParser_ESC_LITERAL
                {
                    {
                        /*InvokeRule lhs*/
                        recog.base.set_state(782);
                        recog.lhs()?;
                    }
                }

                /*InvokeRule relationshipsChainPattern*/
                recog.base.set_state(785);
                recog.relationshipsChainPattern()?;

                recog.base.set_state(787);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(786);
                        recog.where_()?;
                    }
                }

                recog.base.set_state(789);
                recog
                    .base
                    .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(790);
                recog.expression()?;

                recog.base.set_state(791);
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
            .enter_rule(_localctx.clone(), 152, RULE_relationshipsChainPattern);
        let mut _localctx: Rc<RelationshipsChainPatternContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            let mut _alt: i32;
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule nodePattern*/
                recog.base.set_state(793);
                recog.nodePattern()?;

                recog.base.set_state(795);
                recog.err_handler.sync(&mut recog.base)?;
                _alt = 1;
                loop {
                    match _alt {
                        x if x == 1 => {
                            {
                                /*InvokeRule patternElemChain*/
                                recog.base.set_state(794);
                                recog.patternElemChain()?;
                            }
                        }

                        _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                            &mut recog.base,
                        )))?,
                    }
                    recog.base.set_state(797);
                    recog.err_handler.sync(&mut recog.base)?;
                    _alt = recog.interpreter.adaptive_predict(93, &mut recog.base)?;
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
            .enter_rule(_localctx.clone(), 154, RULE_listComprehension);
        let mut _localctx: Rc<ListComprehensionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(799);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                /*InvokeRule filterExpression*/
                recog.base.set_state(800);
                recog.filterExpression()?;

                recog.base.set_state(803);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_STICK {
                    {
                        recog.base.set_state(801);
                        recog
                            .base
                            .match_token(CypherParser_STICK, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(802);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(805);
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
            .enter_rule(_localctx.clone(), 156, RULE_filterExpression);
        let mut _localctx: Rc<FilterExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule symbol*/
                recog.base.set_state(807);
                recog.symbol()?;

                recog.base.set_state(808);
                recog
                    .base
                    .match_token(CypherParser_IN, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(809);
                recog.expression()?;

                recog.base.set_state(811);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_WHERE {
                    {
                        /*InvokeRule where_*/
                        recog.base.set_state(810);
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
        recog.base.enter_rule(_localctx.clone(), 158, RULE_countAll);
        let mut _localctx: Rc<CountAllContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(813);
                recog
                    .base
                    .match_token(CypherParser_COUNT, &mut recog.err_handler)?;

                recog.base.set_state(814);
                recog
                    .base
                    .match_token(CypherParser_LPAREN, &mut recog.err_handler)?;

                recog.base.set_state(815);
                recog
                    .base
                    .match_token(CypherParser_MULT, &mut recog.err_handler)?;

                recog.base.set_state(816);
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
            .enter_rule(_localctx.clone(), 160, RULE_expressionChain);
        let mut _localctx: Rc<ExpressionChainContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule expression*/
                recog.base.set_state(818);
                recog.expression()?;

                recog.base.set_state(823);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                while _la == CypherParser_COMMA {
                    {
                        {
                            recog.base.set_state(819);
                            recog
                                .base
                                .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(820);
                            recog.expression()?;
                        }
                    }
                    recog.base.set_state(825);
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
            .enter_rule(_localctx.clone(), 162, RULE_caseExpression);
        let mut _localctx: Rc<CaseExpressionContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(826);
                recog
                    .base
                    .match_token(CypherParser_CASE, &mut recog.err_handler)?;

                recog.base.set_state(828);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expression*/
                        recog.base.set_state(827);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(835);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                loop {
                    {
                        {
                            recog.base.set_state(830);
                            recog
                                .base
                                .match_token(CypherParser_WHEN, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(831);
                            recog.expression()?;

                            recog.base.set_state(832);
                            recog
                                .base
                                .match_token(CypherParser_THEN, &mut recog.err_handler)?;

                            /*InvokeRule expression*/
                            recog.base.set_state(833);
                            recog.expression()?;
                        }
                    }
                    recog.base.set_state(837);
                    recog.err_handler.sync(&mut recog.base)?;
                    _la = recog.base.input.la(1);
                    if !(_la == CypherParser_WHEN) {
                        break;
                    }
                }
                recog.base.set_state(841);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_ELSE {
                    {
                        recog.base.set_state(839);
                        recog
                            .base
                            .match_token(CypherParser_ELSE, &mut recog.err_handler)?;

                        /*InvokeRule expression*/
                        recog.base.set_state(840);
                        recog.expression()?;
                    }
                }

                recog.base.set_state(843);
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
            .enter_rule(_localctx.clone(), 164, RULE_parameter);
        let mut _localctx: Rc<ParameterContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(845);
                recog
                    .base
                    .match_token(CypherParser_DOLLAR, &mut recog.err_handler)?;

                recog.base.set_state(848);
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
                            recog.base.set_state(846);
                            recog.symbol()?;
                        }
                    }

                    CypherParser_DIGIT => {
                        {
                            /*InvokeRule numLit*/
                            recog.base.set_state(847);
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
        recog.base.enter_rule(_localctx.clone(), 166, RULE_literal);
        let mut _localctx: Rc<LiteralContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(857);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                CypherParser_FALSE | CypherParser_TRUE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule boolLit*/
                        recog.base.set_state(850);
                        recog.boolLit()?;
                    }
                }

                CypherParser_DIGIT => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule numLit*/
                        recog.base.set_state(851);
                        recog.numLit()?;
                    }
                }

                CypherParser_NULL_W => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 3)?;
                    recog.base.enter_outer_alt(None, 3)?;
                    {
                        recog.base.set_state(852);
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
                        recog.base.set_state(853);
                        recog.stringLit()?;
                    }
                }

                CypherParser_CHAR_LITERAL => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 5)?;
                    recog.base.enter_outer_alt(None, 5)?;
                    {
                        /*InvokeRule charLit*/
                        recog.base.set_state(854);
                        recog.charLit()?;
                    }
                }

                CypherParser_LBRACK => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 6)?;
                    recog.base.enter_outer_alt(None, 6)?;
                    {
                        /*InvokeRule listLit*/
                        recog.base.set_state(855);
                        recog.listLit()?;
                    }
                }

                CypherParser_LBRACE => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 7)?;
                    recog.base.enter_outer_alt(None, 7)?;
                    {
                        /*InvokeRule mapLit*/
                        recog.base.set_state(856);
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
        recog.base.enter_rule(_localctx.clone(), 168, RULE_rangeLit);
        let mut _localctx: Rc<RangeLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(859);
                recog
                    .base
                    .match_token(CypherParser_MULT, &mut recog.err_handler)?;

                recog.base.set_state(861);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_DIGIT {
                    {
                        /*InvokeRule numLit*/
                        recog.base.set_state(860);
                        recog.numLit()?;
                    }
                }

                recog.base.set_state(867);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if _la == CypherParser_RANGE {
                    {
                        recog.base.set_state(863);
                        recog
                            .base
                            .match_token(CypherParser_RANGE, &mut recog.err_handler)?;

                        recog.base.set_state(865);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        if _la == CypherParser_DIGIT {
                            {
                                /*InvokeRule numLit*/
                                recog.base.set_state(864);
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
        recog.base.enter_rule(_localctx.clone(), 170, RULE_boolLit);
        let mut _localctx: Rc<BoolLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(869);
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
        recog.base.enter_rule(_localctx.clone(), 172, RULE_numLit);
        let mut _localctx: Rc<NumLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(871);
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
            .enter_rule(_localctx.clone(), 174, RULE_stringLit);
        let mut _localctx: Rc<StringLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(873);
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
        recog.base.enter_rule(_localctx.clone(), 176, RULE_charLit);
        let mut _localctx: Rc<CharLitContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(875);
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
        recog.base.enter_rule(_localctx.clone(), 178, RULE_listLit);
        let mut _localctx: Rc<ListLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(877);
                recog
                    .base
                    .match_token(CypherParser_LBRACK, &mut recog.err_handler)?;

                recog.base.set_state(879);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la) & !0x3f) == 0 && ((1usize << _la) & 3356315648) != 0)
                    || (((_la - 32) & !0x3f) == 0 && ((1usize << (_la - 32)) & 8223) != 0)
                    || (((_la - 69) & !0x3f) == 0 && ((1usize << (_la - 69)) & 260055265) != 0)
                {
                    {
                        /*InvokeRule expressionChain*/
                        recog.base.set_state(878);
                        recog.expressionChain()?;
                    }
                }

                recog.base.set_state(881);
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
        recog.base.enter_rule(_localctx.clone(), 180, RULE_mapLit);
        let mut _localctx: Rc<MapLitContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(883);
                recog
                    .base
                    .match_token(CypherParser_LBRACE, &mut recog.err_handler)?;

                recog.base.set_state(892);
                recog.err_handler.sync(&mut recog.base)?;
                _la = recog.base.input.la(1);
                if (((_la - 30) & !0x3f) == 0 && ((1usize << (_la - 30)) & 4294967295) != 0)
                    || (((_la - 62) & !0x3f) == 0 && ((1usize << (_la - 62)) & 4294967295) != 0)
                {
                    {
                        /*InvokeRule mapPair*/
                        recog.base.set_state(884);
                        recog.mapPair()?;

                        recog.base.set_state(889);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        while _la == CypherParser_COMMA {
                            {
                                {
                                    recog.base.set_state(885);
                                    recog
                                        .base
                                        .match_token(CypherParser_COMMA, &mut recog.err_handler)?;

                                    /*InvokeRule mapPair*/
                                    recog.base.set_state(886);
                                    recog.mapPair()?;
                                }
                            }
                            recog.base.set_state(891);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                        }
                    }
                }

                recog.base.set_state(894);
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
        recog.base.enter_rule(_localctx.clone(), 182, RULE_mapPair);
        let mut _localctx: Rc<MapPairContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                /*InvokeRule name*/
                recog.base.set_state(896);
                recog.name()?;

                recog.base.set_state(897);
                recog
                    .base
                    .match_token(CypherParser_COLON, &mut recog.err_handler)?;

                /*InvokeRule expression*/
                recog.base.set_state(898);
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
        recog.base.enter_rule(_localctx.clone(), 184, RULE_name);
        let mut _localctx: Rc<NameContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(902);
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
                        recog.base.set_state(900);
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
                        recog.base.set_state(901);
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
        recog.base.enter_rule(_localctx.clone(), 186, RULE_symbol);
        let mut _localctx: Rc<SymbolContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(904);
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
            .enter_rule(_localctx.clone(), 188, RULE_reservedWord);
        let mut _localctx: Rc<ReservedWordContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(906);
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
        4, 1, 101, 909, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 4, 2, 5, 7, 5, 2,
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
        91, 7, 91, 2, 92, 7, 92, 2, 93, 7, 93, 2, 94, 7, 94, 1, 0, 1, 0, 3, 0, 193, 8, 0, 1, 0, 1,
        0, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 201, 8, 1, 1, 2, 1, 2, 1, 2, 3, 2, 206, 8, 2, 1, 3, 1, 3,
        1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 3, 3, 217, 8, 3, 1, 4, 1, 4, 5, 4, 221, 8, 4, 10,
        4, 12, 4, 224, 9, 4, 1, 5, 1, 5, 3, 5, 228, 8, 5, 1, 6, 1, 6, 1, 6, 3, 6, 233, 8, 6, 1, 6,
        1, 6, 1, 6, 3, 6, 238, 8, 6, 3, 6, 240, 8, 6, 1, 7, 1, 7, 1, 7, 1, 8, 1, 8, 1, 8, 3, 8,
        248, 8, 8, 1, 9, 1, 9, 1, 9, 1, 10, 1, 10, 1, 10, 1, 11, 3, 11, 257, 8, 11, 1, 11, 1, 11,
        3, 11, 261, 8, 11, 1, 11, 3, 11, 264, 8, 11, 1, 11, 3, 11, 267, 8, 11, 1, 12, 1, 12, 3, 12,
        271, 8, 12, 1, 12, 1, 12, 5, 12, 275, 8, 12, 10, 12, 12, 12, 278, 9, 12, 1, 13, 1, 13, 1,
        13, 3, 13, 283, 8, 13, 1, 14, 1, 14, 3, 14, 287, 8, 14, 1, 15, 1, 15, 1, 15, 1, 15, 1, 15,
        5, 15, 294, 8, 15, 10, 15, 12, 15, 297, 9, 15, 1, 16, 5, 16, 300, 8, 16, 10, 16, 12, 16,
        303, 9, 16, 1, 16, 1, 16, 4, 16, 307, 8, 16, 11, 16, 12, 16, 308, 1, 16, 3, 16, 312, 8, 16,
        3, 16, 314, 8, 16, 1, 17, 5, 17, 317, 8, 17, 10, 17, 12, 17, 320, 9, 17, 1, 17, 1, 17, 5,
        17, 324, 8, 17, 10, 17, 12, 17, 327, 9, 17, 1, 17, 4, 17, 330, 8, 17, 11, 17, 12, 17, 331,
        1, 17, 1, 17, 1, 18, 3, 18, 337, 8, 18, 1, 18, 1, 18, 1, 18, 1, 19, 1, 19, 1, 19, 1, 19, 1,
        19, 1, 20, 1, 20, 1, 20, 3, 20, 350, 8, 20, 1, 21, 1, 21, 1, 21, 1, 21, 1, 21, 3, 21, 357,
        8, 21, 1, 22, 3, 22, 360, 8, 22, 1, 22, 1, 22, 1, 22, 1, 23, 1, 23, 1, 23, 1, 23, 5, 23,
        369, 8, 23, 10, 23, 12, 23, 372, 9, 23, 1, 24, 1, 24, 1, 24, 1, 24, 3, 24, 378, 8, 24, 1,
        25, 1, 25, 1, 25, 1, 25, 1, 25, 3, 25, 385, 8, 25, 1, 26, 1, 26, 3, 26, 389, 8, 26, 1, 26,
        1, 26, 1, 27, 1, 27, 1, 27, 5, 27, 396, 8, 27, 10, 27, 12, 27, 399, 9, 27, 1, 27, 3, 27,
        402, 8, 27, 1, 28, 1, 28, 1, 28, 3, 28, 407, 8, 28, 1, 28, 1, 28, 1, 29, 1, 29, 1, 29, 5,
        29, 414, 8, 29, 10, 29, 12, 29, 417, 9, 29, 1, 30, 1, 30, 1, 30, 1, 30, 1, 31, 1, 31, 1,
        31, 1, 31, 5, 31, 427, 8, 31, 10, 31, 12, 31, 430, 9, 31, 1, 32, 1, 32, 1, 32, 1, 32, 1,
        32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 1, 32, 3, 32, 443, 8, 32, 1, 33, 1, 33, 4, 33, 447,
        8, 33, 11, 33, 12, 33, 448, 1, 34, 1, 34, 1, 34, 1, 35, 1, 35, 3, 35, 456, 8, 35, 1, 36, 1,
        36, 1, 36, 1, 37, 1, 37, 1, 37, 5, 37, 464, 8, 37, 10, 37, 12, 37, 467, 9, 37, 1, 38, 1,
        38, 1, 38, 5, 38, 472, 8, 38, 10, 38, 12, 38, 475, 9, 38, 1, 39, 1, 39, 1, 39, 5, 39, 480,
        8, 39, 10, 39, 12, 39, 483, 9, 39, 1, 40, 1, 40, 1, 40, 5, 40, 488, 8, 40, 10, 40, 12, 40,
        491, 9, 40, 1, 41, 5, 41, 494, 8, 41, 10, 41, 12, 41, 497, 9, 41, 1, 41, 1, 41, 1, 42, 1,
        42, 1, 42, 1, 42, 5, 42, 505, 8, 42, 10, 42, 12, 42, 508, 9, 42, 1, 43, 1, 43, 1, 44, 1,
        44, 1, 44, 5, 44, 515, 8, 44, 10, 44, 12, 44, 518, 9, 44, 1, 45, 1, 45, 1, 45, 5, 45, 523,
        8, 45, 10, 45, 12, 45, 526, 9, 45, 1, 46, 1, 46, 1, 46, 5, 46, 531, 8, 46, 10, 46, 12, 46,
        534, 9, 46, 1, 47, 3, 47, 537, 8, 47, 1, 47, 1, 47, 1, 48, 1, 48, 1, 48, 1, 48, 5, 48, 545,
        8, 48, 10, 48, 12, 48, 548, 9, 48, 1, 49, 1, 49, 1, 49, 1, 49, 3, 49, 554, 8, 49, 1, 49, 1,
        49, 3, 49, 558, 8, 49, 1, 49, 3, 49, 561, 8, 49, 1, 49, 3, 49, 564, 8, 49, 1, 50, 1, 50, 1,
        50, 1, 51, 1, 51, 1, 51, 1, 51, 1, 51, 3, 51, 574, 8, 51, 1, 52, 1, 52, 3, 52, 578, 8, 52,
        1, 52, 1, 52, 1, 53, 1, 53, 3, 53, 584, 8, 53, 1, 54, 1, 54, 1, 54, 5, 54, 589, 8, 54, 10,
        54, 12, 54, 592, 9, 54, 1, 55, 1, 55, 1, 55, 3, 55, 597, 8, 55, 1, 55, 1, 55, 3, 55, 601,
        8, 55, 1, 56, 1, 56, 1, 56, 1, 56, 1, 56, 1, 57, 1, 57, 1, 57, 5, 57, 611, 8, 57, 10, 57,
        12, 57, 614, 9, 57, 1, 57, 1, 57, 1, 57, 1, 57, 3, 57, 620, 8, 57, 3, 57, 622, 8, 57, 1,
        58, 1, 58, 1, 58, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 59, 1, 60, 1, 60, 1, 60, 1, 60, 1,
        60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1,
        60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 1, 60, 3, 60, 658, 8, 60, 1, 61, 1, 61, 1,
        62, 1, 62, 3, 62, 664, 8, 62, 1, 63, 1, 63, 3, 63, 668, 8, 63, 1, 63, 3, 63, 671, 8, 63, 1,
        63, 3, 63, 674, 8, 63, 1, 63, 1, 63, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1, 64, 1,
        64, 1, 64, 1, 64, 1, 64, 1, 64, 3, 64, 690, 8, 64, 1, 65, 1, 65, 1, 65, 1, 66, 1, 66, 1,
        66, 3, 66, 698, 8, 66, 1, 66, 1, 66, 3, 66, 702, 8, 66, 1, 66, 1, 66, 3, 66, 706, 8, 66, 1,
        66, 1, 66, 3, 66, 710, 8, 66, 3, 66, 712, 8, 66, 1, 67, 1, 67, 3, 67, 716, 8, 67, 1, 67, 3,
        67, 719, 8, 67, 1, 67, 3, 67, 722, 8, 67, 1, 67, 3, 67, 725, 8, 67, 1, 67, 1, 67, 1, 68, 1,
        68, 1, 68, 1, 68, 3, 68, 733, 8, 68, 1, 68, 5, 68, 736, 8, 68, 10, 68, 12, 68, 739, 9, 68,
        1, 69, 1, 69, 3, 69, 743, 8, 69, 1, 69, 1, 69, 1, 70, 1, 70, 1, 70, 1, 70, 3, 70, 751, 8,
        70, 1, 70, 1, 70, 1, 71, 1, 71, 1, 71, 5, 71, 758, 8, 71, 10, 71, 12, 71, 761, 9, 71, 1,
        72, 1, 72, 1, 72, 3, 72, 766, 8, 72, 1, 72, 3, 72, 769, 8, 72, 1, 72, 1, 72, 1, 73, 1, 73,
        1, 73, 1, 73, 1, 74, 1, 74, 1, 74, 1, 74, 1, 74, 1, 75, 1, 75, 3, 75, 784, 8, 75, 1, 75, 1,
        75, 3, 75, 788, 8, 75, 1, 75, 1, 75, 1, 75, 1, 75, 1, 76, 1, 76, 4, 76, 796, 8, 76, 11, 76,
        12, 76, 797, 1, 77, 1, 77, 1, 77, 1, 77, 3, 77, 804, 8, 77, 1, 77, 1, 77, 1, 78, 1, 78, 1,
        78, 1, 78, 3, 78, 812, 8, 78, 1, 79, 1, 79, 1, 79, 1, 79, 1, 79, 1, 80, 1, 80, 1, 80, 5,
        80, 822, 8, 80, 10, 80, 12, 80, 825, 9, 80, 1, 81, 1, 81, 3, 81, 829, 8, 81, 1, 81, 1, 81,
        1, 81, 1, 81, 1, 81, 4, 81, 836, 8, 81, 11, 81, 12, 81, 837, 1, 81, 1, 81, 3, 81, 842, 8,
        81, 1, 81, 1, 81, 1, 82, 1, 82, 1, 82, 3, 82, 849, 8, 82, 1, 83, 1, 83, 1, 83, 1, 83, 1,
        83, 1, 83, 1, 83, 3, 83, 858, 8, 83, 1, 84, 1, 84, 3, 84, 862, 8, 84, 1, 84, 1, 84, 3, 84,
        866, 8, 84, 3, 84, 868, 8, 84, 1, 85, 1, 85, 1, 86, 1, 86, 1, 87, 1, 87, 1, 88, 1, 88, 1,
        89, 1, 89, 3, 89, 880, 8, 89, 1, 89, 1, 89, 1, 90, 1, 90, 1, 90, 1, 90, 5, 90, 888, 8, 90,
        10, 90, 12, 90, 891, 9, 90, 3, 90, 893, 8, 90, 1, 90, 1, 90, 1, 91, 1, 91, 1, 91, 1, 91, 1,
        92, 1, 92, 3, 92, 903, 8, 92, 1, 93, 1, 93, 1, 94, 1, 94, 1, 94, 0, 0, 95, 0, 2, 4, 6, 8,
        10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54,
        56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 90, 92, 94, 96, 98,
        100, 102, 104, 106, 108, 110, 112, 114, 116, 118, 120, 122, 124, 126, 128, 130, 132, 134,
        136, 138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 166, 168, 170,
        172, 174, 176, 178, 180, 182, 184, 186, 188, 0, 11, 2, 0, 37, 38, 42, 43, 2, 0, 40, 40, 48,
        48, 1, 0, 1, 2, 2, 0, 1, 1, 3, 7, 1, 0, 18, 19, 2, 0, 20, 21, 23, 23, 2, 0, 92, 92, 96, 96,
        1, 0, 33, 36, 1, 0, 74, 75, 2, 0, 30, 35, 92, 93, 1, 0, 36, 91, 951, 0, 190, 1, 0, 0, 0, 2,
        200, 1, 0, 0, 0, 4, 202, 1, 0, 0, 0, 6, 207, 1, 0, 0, 0, 8, 218, 1, 0, 0, 0, 10, 227, 1, 0,
        0, 0, 12, 229, 1, 0, 0, 0, 14, 241, 1, 0, 0, 0, 16, 244, 1, 0, 0, 0, 18, 249, 1, 0, 0, 0,
        20, 252, 1, 0, 0, 0, 22, 256, 1, 0, 0, 0, 24, 270, 1, 0, 0, 0, 26, 279, 1, 0, 0, 0, 28,
        284, 1, 0, 0, 0, 30, 288, 1, 0, 0, 0, 32, 301, 1, 0, 0, 0, 34, 318, 1, 0, 0, 0, 36, 336, 1,
        0, 0, 0, 38, 341, 1, 0, 0, 0, 40, 349, 1, 0, 0, 0, 42, 356, 1, 0, 0, 0, 44, 359, 1, 0, 0,
        0, 46, 364, 1, 0, 0, 0, 48, 377, 1, 0, 0, 0, 50, 379, 1, 0, 0, 0, 52, 386, 1, 0, 0, 0, 54,
        392, 1, 0, 0, 0, 56, 406, 1, 0, 0, 0, 58, 410, 1, 0, 0, 0, 60, 418, 1, 0, 0, 0, 62, 422, 1,
        0, 0, 0, 64, 442, 1, 0, 0, 0, 66, 446, 1, 0, 0, 0, 68, 450, 1, 0, 0, 0, 70, 453, 1, 0, 0,
        0, 72, 457, 1, 0, 0, 0, 74, 460, 1, 0, 0, 0, 76, 468, 1, 0, 0, 0, 78, 476, 1, 0, 0, 0, 80,
        484, 1, 0, 0, 0, 82, 495, 1, 0, 0, 0, 84, 500, 1, 0, 0, 0, 86, 509, 1, 0, 0, 0, 88, 511, 1,
        0, 0, 0, 90, 519, 1, 0, 0, 0, 92, 527, 1, 0, 0, 0, 94, 536, 1, 0, 0, 0, 96, 540, 1, 0, 0,
        0, 98, 563, 1, 0, 0, 0, 100, 565, 1, 0, 0, 0, 102, 573, 1, 0, 0, 0, 104, 575, 1, 0, 0, 0,
        106, 581, 1, 0, 0, 0, 108, 585, 1, 0, 0, 0, 110, 596, 1, 0, 0, 0, 112, 602, 1, 0, 0, 0,
        114, 621, 1, 0, 0, 0, 116, 623, 1, 0, 0, 0, 118, 626, 1, 0, 0, 0, 120, 657, 1, 0, 0, 0,
        122, 659, 1, 0, 0, 0, 124, 663, 1, 0, 0, 0, 126, 665, 1, 0, 0, 0, 128, 689, 1, 0, 0, 0,
        130, 691, 1, 0, 0, 0, 132, 711, 1, 0, 0, 0, 134, 713, 1, 0, 0, 0, 136, 728, 1, 0, 0, 0,
        138, 740, 1, 0, 0, 0, 140, 746, 1, 0, 0, 0, 142, 754, 1, 0, 0, 0, 144, 762, 1, 0, 0, 0,
        146, 772, 1, 0, 0, 0, 148, 776, 1, 0, 0, 0, 150, 781, 1, 0, 0, 0, 152, 793, 1, 0, 0, 0,
        154, 799, 1, 0, 0, 0, 156, 807, 1, 0, 0, 0, 158, 813, 1, 0, 0, 0, 160, 818, 1, 0, 0, 0,
        162, 826, 1, 0, 0, 0, 164, 845, 1, 0, 0, 0, 166, 857, 1, 0, 0, 0, 168, 859, 1, 0, 0, 0,
        170, 869, 1, 0, 0, 0, 172, 871, 1, 0, 0, 0, 174, 873, 1, 0, 0, 0, 176, 875, 1, 0, 0, 0,
        178, 877, 1, 0, 0, 0, 180, 883, 1, 0, 0, 0, 182, 896, 1, 0, 0, 0, 184, 902, 1, 0, 0, 0,
        186, 904, 1, 0, 0, 0, 188, 906, 1, 0, 0, 0, 190, 192, 3, 2, 1, 0, 191, 193, 5, 9, 0, 0,
        192, 191, 1, 0, 0, 0, 192, 193, 1, 0, 0, 0, 193, 194, 1, 0, 0, 0, 194, 195, 5, 0, 0, 1,
        195, 1, 1, 0, 0, 0, 196, 201, 3, 4, 2, 0, 197, 201, 3, 8, 4, 0, 198, 201, 3, 12, 6, 0, 199,
        201, 3, 6, 3, 0, 200, 196, 1, 0, 0, 0, 200, 197, 1, 0, 0, 0, 200, 198, 1, 0, 0, 0, 200,
        199, 1, 0, 0, 0, 201, 3, 1, 0, 0, 0, 202, 205, 5, 46, 0, 0, 203, 206, 3, 6, 3, 0, 204, 206,
        3, 8, 4, 0, 205, 203, 1, 0, 0, 0, 205, 204, 1, 0, 0, 0, 206, 5, 1, 0, 0, 0, 207, 208, 5,
        40, 0, 0, 208, 209, 5, 67, 0, 0, 209, 210, 5, 50, 0, 0, 210, 211, 5, 25, 0, 0, 211, 212, 3,
        184, 92, 0, 212, 213, 5, 12, 0, 0, 213, 214, 3, 184, 92, 0, 214, 216, 5, 13, 0, 0, 215,
        217, 5, 81, 0, 0, 216, 215, 1, 0, 0, 0, 216, 217, 1, 0, 0, 0, 217, 7, 1, 0, 0, 0, 218, 222,
        3, 10, 5, 0, 219, 221, 3, 138, 69, 0, 220, 219, 1, 0, 0, 0, 221, 224, 1, 0, 0, 0, 222, 220,
        1, 0, 0, 0, 222, 223, 1, 0, 0, 0, 223, 9, 1, 0, 0, 0, 224, 222, 1, 0, 0, 0, 225, 228, 3,
        32, 16, 0, 226, 228, 3, 34, 17, 0, 227, 225, 1, 0, 0, 0, 227, 226, 1, 0, 0, 0, 228, 11, 1,
        0, 0, 0, 229, 230, 5, 28, 0, 0, 230, 232, 3, 142, 71, 0, 231, 233, 3, 52, 26, 0, 232, 231,
        1, 0, 0, 0, 232, 233, 1, 0, 0, 0, 233, 239, 1, 0, 0, 0, 234, 237, 5, 29, 0, 0, 235, 238, 5,
        23, 0, 0, 236, 238, 3, 54, 27, 0, 237, 235, 1, 0, 0, 0, 237, 236, 1, 0, 0, 0, 238, 240, 1,
        0, 0, 0, 239, 234, 1, 0, 0, 0, 239, 240, 1, 0, 0, 0, 240, 13, 1, 0, 0, 0, 241, 242, 5, 54,
        0, 0, 242, 243, 3, 22, 11, 0, 243, 15, 1, 0, 0, 0, 244, 245, 5, 58, 0, 0, 245, 247, 3, 22,
        11, 0, 246, 248, 3, 72, 36, 0, 247, 246, 1, 0, 0, 0, 247, 248, 1, 0, 0, 0, 248, 17, 1, 0,
        0, 0, 249, 250, 5, 56, 0, 0, 250, 251, 3, 76, 38, 0, 251, 19, 1, 0, 0, 0, 252, 253, 5, 47,
        0, 0, 253, 254, 3, 76, 38, 0, 254, 21, 1, 0, 0, 0, 255, 257, 5, 64, 0, 0, 256, 255, 1, 0,
        0, 0, 256, 257, 1, 0, 0, 0, 257, 258, 1, 0, 0, 0, 258, 260, 3, 24, 12, 0, 259, 261, 3, 30,
        15, 0, 260, 259, 1, 0, 0, 0, 260, 261, 1, 0, 0, 0, 261, 263, 1, 0, 0, 0, 262, 264, 3, 18,
        9, 0, 263, 262, 1, 0, 0, 0, 263, 264, 1, 0, 0, 0, 264, 266, 1, 0, 0, 0, 265, 267, 3, 20,
        10, 0, 266, 265, 1, 0, 0, 0, 266, 267, 1, 0, 0, 0, 267, 23, 1, 0, 0, 0, 268, 271, 5, 23, 0,
        0, 269, 271, 3, 26, 13, 0, 270, 268, 1, 0, 0, 0, 270, 269, 1, 0, 0, 0, 271, 276, 1, 0, 0,
        0, 272, 273, 5, 11, 0, 0, 273, 275, 3, 26, 13, 0, 274, 272, 1, 0, 0, 0, 275, 278, 1, 0, 0,
        0, 276, 274, 1, 0, 0, 0, 276, 277, 1, 0, 0, 0, 277, 25, 1, 0, 0, 0, 278, 276, 1, 0, 0, 0,
        279, 282, 3, 76, 38, 0, 280, 281, 5, 62, 0, 0, 281, 283, 3, 186, 93, 0, 282, 280, 1, 0, 0,
        0, 282, 283, 1, 0, 0, 0, 283, 27, 1, 0, 0, 0, 284, 286, 3, 76, 38, 0, 285, 287, 7, 0, 0, 0,
        286, 285, 1, 0, 0, 0, 286, 287, 1, 0, 0, 0, 287, 29, 1, 0, 0, 0, 288, 289, 5, 52, 0, 0,
        289, 290, 5, 39, 0, 0, 290, 295, 3, 28, 14, 0, 291, 292, 5, 11, 0, 0, 292, 294, 3, 28, 14,
        0, 293, 291, 1, 0, 0, 0, 294, 297, 1, 0, 0, 0, 295, 293, 1, 0, 0, 0, 295, 296, 1, 0, 0, 0,
        296, 31, 1, 0, 0, 0, 297, 295, 1, 0, 0, 0, 298, 300, 3, 40, 20, 0, 299, 298, 1, 0, 0, 0,
        300, 303, 1, 0, 0, 0, 301, 299, 1, 0, 0, 0, 301, 302, 1, 0, 0, 0, 302, 313, 1, 0, 0, 0,
        303, 301, 1, 0, 0, 0, 304, 314, 3, 14, 7, 0, 305, 307, 3, 42, 21, 0, 306, 305, 1, 0, 0, 0,
        307, 308, 1, 0, 0, 0, 308, 306, 1, 0, 0, 0, 308, 309, 1, 0, 0, 0, 309, 311, 1, 0, 0, 0,
        310, 312, 3, 14, 7, 0, 311, 310, 1, 0, 0, 0, 311, 312, 1, 0, 0, 0, 312, 314, 1, 0, 0, 0,
        313, 304, 1, 0, 0, 0, 313, 306, 1, 0, 0, 0, 314, 33, 1, 0, 0, 0, 315, 317, 3, 40, 20, 0,
        316, 315, 1, 0, 0, 0, 317, 320, 1, 0, 0, 0, 318, 316, 1, 0, 0, 0, 318, 319, 1, 0, 0, 0,
        319, 329, 1, 0, 0, 0, 320, 318, 1, 0, 0, 0, 321, 324, 3, 40, 20, 0, 322, 324, 3, 42, 21, 0,
        323, 321, 1, 0, 0, 0, 323, 322, 1, 0, 0, 0, 324, 327, 1, 0, 0, 0, 325, 323, 1, 0, 0, 0,
        325, 326, 1, 0, 0, 0, 326, 328, 1, 0, 0, 0, 327, 325, 1, 0, 0, 0, 328, 330, 3, 16, 8, 0,
        329, 325, 1, 0, 0, 0, 330, 331, 1, 0, 0, 0, 331, 329, 1, 0, 0, 0, 331, 332, 1, 0, 0, 0,
        332, 333, 1, 0, 0, 0, 333, 334, 3, 32, 16, 0, 334, 35, 1, 0, 0, 0, 335, 337, 5, 51, 0, 0,
        336, 335, 1, 0, 0, 0, 336, 337, 1, 0, 0, 0, 337, 338, 1, 0, 0, 0, 338, 339, 5, 48, 0, 0,
        339, 340, 3, 70, 35, 0, 340, 37, 1, 0, 0, 0, 341, 342, 5, 60, 0, 0, 342, 343, 3, 76, 38, 0,
        343, 344, 5, 62, 0, 0, 344, 345, 3, 186, 93, 0, 345, 39, 1, 0, 0, 0, 346, 350, 3, 36, 18,
        0, 347, 350, 3, 38, 19, 0, 348, 350, 3, 50, 25, 0, 349, 346, 1, 0, 0, 0, 349, 347, 1, 0, 0,
        0, 349, 348, 1, 0, 0, 0, 350, 41, 1, 0, 0, 0, 351, 357, 3, 68, 34, 0, 352, 357, 3, 58, 29,
        0, 353, 357, 3, 44, 22, 0, 354, 357, 3, 62, 31, 0, 355, 357, 3, 46, 23, 0, 356, 351, 1, 0,
        0, 0, 356, 352, 1, 0, 0, 0, 356, 353, 1, 0, 0, 0, 356, 354, 1, 0, 0, 0, 356, 355, 1, 0, 0,
        0, 357, 43, 1, 0, 0, 0, 358, 360, 5, 44, 0, 0, 359, 358, 1, 0, 0, 0, 359, 360, 1, 0, 0, 0,
        360, 361, 1, 0, 0, 0, 361, 362, 5, 41, 0, 0, 362, 363, 3, 160, 80, 0, 363, 45, 1, 0, 0, 0,
        364, 365, 5, 53, 0, 0, 365, 370, 3, 48, 24, 0, 366, 367, 5, 11, 0, 0, 367, 369, 3, 48, 24,
        0, 368, 366, 1, 0, 0, 0, 369, 372, 1, 0, 0, 0, 370, 368, 1, 0, 0, 0, 370, 371, 1, 0, 0, 0,
        371, 47, 1, 0, 0, 0, 372, 370, 1, 0, 0, 0, 373, 374, 3, 186, 93, 0, 374, 375, 3, 66, 33, 0,
        375, 378, 1, 0, 0, 0, 376, 378, 3, 108, 54, 0, 377, 373, 1, 0, 0, 0, 377, 376, 1, 0, 0, 0,
        378, 49, 1, 0, 0, 0, 379, 380, 5, 28, 0, 0, 380, 381, 3, 142, 71, 0, 381, 384, 3, 52, 26,
        0, 382, 383, 5, 29, 0, 0, 383, 385, 3, 54, 27, 0, 384, 382, 1, 0, 0, 0, 384, 385, 1, 0, 0,
        0, 385, 51, 1, 0, 0, 0, 386, 388, 5, 12, 0, 0, 387, 389, 3, 160, 80, 0, 388, 387, 1, 0, 0,
        0, 388, 389, 1, 0, 0, 0, 389, 390, 1, 0, 0, 0, 390, 391, 5, 13, 0, 0, 391, 53, 1, 0, 0, 0,
        392, 397, 3, 56, 28, 0, 393, 394, 5, 11, 0, 0, 394, 396, 3, 56, 28, 0, 395, 393, 1, 0, 0,
        0, 396, 399, 1, 0, 0, 0, 397, 395, 1, 0, 0, 0, 397, 398, 1, 0, 0, 0, 398, 401, 1, 0, 0, 0,
        399, 397, 1, 0, 0, 0, 400, 402, 3, 72, 36, 0, 401, 400, 1, 0, 0, 0, 401, 402, 1, 0, 0, 0,
        402, 55, 1, 0, 0, 0, 403, 404, 3, 186, 93, 0, 404, 405, 5, 62, 0, 0, 405, 407, 1, 0, 0, 0,
        406, 403, 1, 0, 0, 0, 406, 407, 1, 0, 0, 0, 407, 408, 1, 0, 0, 0, 408, 409, 3, 186, 93, 0,
        409, 57, 1, 0, 0, 0, 410, 411, 5, 49, 0, 0, 411, 415, 3, 110, 55, 0, 412, 414, 3, 60, 30,
        0, 413, 412, 1, 0, 0, 0, 414, 417, 1, 0, 0, 0, 415, 413, 1, 0, 0, 0, 415, 416, 1, 0, 0, 0,
        416, 59, 1, 0, 0, 0, 417, 415, 1, 0, 0, 0, 418, 419, 5, 50, 0, 0, 419, 420, 7, 1, 0, 0,
        420, 421, 3, 62, 31, 0, 421, 61, 1, 0, 0, 0, 422, 423, 5, 55, 0, 0, 423, 428, 3, 64, 32, 0,
        424, 425, 5, 11, 0, 0, 425, 427, 3, 64, 32, 0, 426, 424, 1, 0, 0, 0, 427, 430, 1, 0, 0, 0,
        428, 426, 1, 0, 0, 0, 428, 429, 1, 0, 0, 0, 429, 63, 1, 0, 0, 0, 430, 428, 1, 0, 0, 0, 431,
        432, 3, 108, 54, 0, 432, 433, 5, 1, 0, 0, 433, 434, 3, 76, 38, 0, 434, 443, 1, 0, 0, 0,
        435, 436, 3, 186, 93, 0, 436, 437, 7, 2, 0, 0, 437, 438, 3, 76, 38, 0, 438, 443, 1, 0, 0,
        0, 439, 440, 3, 186, 93, 0, 440, 441, 3, 66, 33, 0, 441, 443, 1, 0, 0, 0, 442, 431, 1, 0,
        0, 0, 442, 435, 1, 0, 0, 0, 442, 439, 1, 0, 0, 0, 443, 65, 1, 0, 0, 0, 444, 445, 5, 25, 0,
        0, 445, 447, 3, 184, 92, 0, 446, 444, 1, 0, 0, 0, 447, 448, 1, 0, 0, 0, 448, 446, 1, 0, 0,
        0, 448, 449, 1, 0, 0, 0, 449, 67, 1, 0, 0, 0, 450, 451, 5, 40, 0, 0, 451, 452, 3, 74, 37,
        0, 452, 69, 1, 0, 0, 0, 453, 455, 3, 74, 37, 0, 454, 456, 3, 72, 36, 0, 455, 454, 1, 0, 0,
        0, 455, 456, 1, 0, 0, 0, 456, 71, 1, 0, 0, 0, 457, 458, 5, 57, 0, 0, 458, 459, 3, 76, 38,
        0, 459, 73, 1, 0, 0, 0, 460, 465, 3, 110, 55, 0, 461, 462, 5, 11, 0, 0, 462, 464, 3, 110,
        55, 0, 463, 461, 1, 0, 0, 0, 464, 467, 1, 0, 0, 0, 465, 463, 1, 0, 0, 0, 465, 466, 1, 0, 0,
        0, 466, 75, 1, 0, 0, 0, 467, 465, 1, 0, 0, 0, 468, 473, 3, 78, 39, 0, 469, 470, 5, 70, 0,
        0, 470, 472, 3, 78, 39, 0, 471, 469, 1, 0, 0, 0, 472, 475, 1, 0, 0, 0, 473, 471, 1, 0, 0,
        0, 473, 474, 1, 0, 0, 0, 474, 77, 1, 0, 0, 0, 475, 473, 1, 0, 0, 0, 476, 481, 3, 80, 40, 0,
        477, 478, 5, 72, 0, 0, 478, 480, 3, 80, 40, 0, 479, 477, 1, 0, 0, 0, 480, 483, 1, 0, 0, 0,
        481, 479, 1, 0, 0, 0, 481, 482, 1, 0, 0, 0, 482, 79, 1, 0, 0, 0, 483, 481, 1, 0, 0, 0, 484,
        489, 3, 82, 41, 0, 485, 486, 5, 61, 0, 0, 486, 488, 3, 82, 41, 0, 487, 485, 1, 0, 0, 0,
        488, 491, 1, 0, 0, 0, 489, 487, 1, 0, 0, 0, 489, 490, 1, 0, 0, 0, 490, 81, 1, 0, 0, 0, 491,
        489, 1, 0, 0, 0, 492, 494, 5, 69, 0, 0, 493, 492, 1, 0, 0, 0, 494, 497, 1, 0, 0, 0, 495,
        493, 1, 0, 0, 0, 495, 496, 1, 0, 0, 0, 496, 498, 1, 0, 0, 0, 497, 495, 1, 0, 0, 0, 498,
        499, 3, 84, 42, 0, 499, 83, 1, 0, 0, 0, 500, 506, 3, 88, 44, 0, 501, 502, 3, 86, 43, 0,
        502, 503, 3, 88, 44, 0, 503, 505, 1, 0, 0, 0, 504, 501, 1, 0, 0, 0, 505, 508, 1, 0, 0, 0,
        506, 504, 1, 0, 0, 0, 506, 507, 1, 0, 0, 0, 507, 85, 1, 0, 0, 0, 508, 506, 1, 0, 0, 0, 509,
        510, 7, 3, 0, 0, 510, 87, 1, 0, 0, 0, 511, 516, 3, 90, 45, 0, 512, 513, 7, 4, 0, 0, 513,
        515, 3, 90, 45, 0, 514, 512, 1, 0, 0, 0, 515, 518, 1, 0, 0, 0, 516, 514, 1, 0, 0, 0, 516,
        517, 1, 0, 0, 0, 517, 89, 1, 0, 0, 0, 518, 516, 1, 0, 0, 0, 519, 524, 3, 92, 46, 0, 520,
        521, 7, 5, 0, 0, 521, 523, 3, 92, 46, 0, 522, 520, 1, 0, 0, 0, 523, 526, 1, 0, 0, 0, 524,
        522, 1, 0, 0, 0, 524, 525, 1, 0, 0, 0, 525, 91, 1, 0, 0, 0, 526, 524, 1, 0, 0, 0, 527, 532,
        3, 94, 47, 0, 528, 529, 5, 22, 0, 0, 529, 531, 3, 94, 47, 0, 530, 528, 1, 0, 0, 0, 531,
        534, 1, 0, 0, 0, 532, 530, 1, 0, 0, 0, 532, 533, 1, 0, 0, 0, 533, 93, 1, 0, 0, 0, 534, 532,
        1, 0, 0, 0, 535, 537, 7, 4, 0, 0, 536, 535, 1, 0, 0, 0, 536, 537, 1, 0, 0, 0, 537, 538, 1,
        0, 0, 0, 538, 539, 3, 96, 48, 0, 539, 95, 1, 0, 0, 0, 540, 546, 3, 106, 53, 0, 541, 545, 3,
        100, 50, 0, 542, 545, 3, 98, 49, 0, 543, 545, 3, 104, 52, 0, 544, 541, 1, 0, 0, 0, 544,
        542, 1, 0, 0, 0, 544, 543, 1, 0, 0, 0, 545, 548, 1, 0, 0, 0, 546, 544, 1, 0, 0, 0, 546,
        547, 1, 0, 0, 0, 547, 97, 1, 0, 0, 0, 548, 546, 1, 0, 0, 0, 549, 550, 5, 66, 0, 0, 550,
        564, 3, 106, 53, 0, 551, 560, 5, 16, 0, 0, 552, 554, 3, 76, 38, 0, 553, 552, 1, 0, 0, 0,
        553, 554, 1, 0, 0, 0, 554, 555, 1, 0, 0, 0, 555, 557, 5, 8, 0, 0, 556, 558, 3, 76, 38, 0,
        557, 556, 1, 0, 0, 0, 557, 558, 1, 0, 0, 0, 558, 561, 1, 0, 0, 0, 559, 561, 3, 76, 38, 0,
        560, 553, 1, 0, 0, 0, 560, 559, 1, 0, 0, 0, 561, 562, 1, 0, 0, 0, 562, 564, 5, 17, 0, 0,
        563, 549, 1, 0, 0, 0, 563, 551, 1, 0, 0, 0, 564, 99, 1, 0, 0, 0, 565, 566, 3, 102, 51, 0,
        566, 567, 3, 106, 53, 0, 567, 101, 1, 0, 0, 0, 568, 569, 5, 71, 0, 0, 569, 574, 5, 58, 0,
        0, 570, 571, 5, 65, 0, 0, 571, 574, 5, 58, 0, 0, 572, 574, 5, 63, 0, 0, 573, 568, 1, 0, 0,
        0, 573, 570, 1, 0, 0, 0, 573, 572, 1, 0, 0, 0, 574, 103, 1, 0, 0, 0, 575, 577, 5, 68, 0, 0,
        576, 578, 5, 69, 0, 0, 577, 576, 1, 0, 0, 0, 577, 578, 1, 0, 0, 0, 578, 579, 1, 0, 0, 0,
        579, 580, 5, 76, 0, 0, 580, 105, 1, 0, 0, 0, 581, 583, 3, 108, 54, 0, 582, 584, 3, 66, 33,
        0, 583, 582, 1, 0, 0, 0, 583, 584, 1, 0, 0, 0, 584, 107, 1, 0, 0, 0, 585, 590, 3, 128, 64,
        0, 586, 587, 5, 10, 0, 0, 587, 589, 3, 184, 92, 0, 588, 586, 1, 0, 0, 0, 589, 592, 1, 0, 0,
        0, 590, 588, 1, 0, 0, 0, 590, 591, 1, 0, 0, 0, 591, 109, 1, 0, 0, 0, 592, 590, 1, 0, 0, 0,
        593, 594, 3, 186, 93, 0, 594, 595, 5, 1, 0, 0, 595, 597, 1, 0, 0, 0, 596, 593, 1, 0, 0, 0,
        596, 597, 1, 0, 0, 0, 597, 600, 1, 0, 0, 0, 598, 601, 3, 112, 56, 0, 599, 601, 3, 114, 57,
        0, 600, 598, 1, 0, 0, 0, 600, 599, 1, 0, 0, 0, 601, 111, 1, 0, 0, 0, 602, 603, 5, 73, 0, 0,
        603, 604, 5, 12, 0, 0, 604, 605, 3, 114, 57, 0, 605, 606, 5, 13, 0, 0, 606, 113, 1, 0, 0,
        0, 607, 612, 3, 126, 63, 0, 608, 611, 3, 116, 58, 0, 609, 611, 3, 118, 59, 0, 610, 608, 1,
        0, 0, 0, 610, 609, 1, 0, 0, 0, 611, 614, 1, 0, 0, 0, 612, 610, 1, 0, 0, 0, 612, 613, 1, 0,
        0, 0, 613, 622, 1, 0, 0, 0, 614, 612, 1, 0, 0, 0, 615, 616, 5, 12, 0, 0, 616, 617, 3, 114,
        57, 0, 617, 619, 5, 13, 0, 0, 618, 620, 3, 120, 60, 0, 619, 618, 1, 0, 0, 0, 619, 620, 1,
        0, 0, 0, 620, 622, 1, 0, 0, 0, 621, 607, 1, 0, 0, 0, 621, 615, 1, 0, 0, 0, 622, 115, 1, 0,
        0, 0, 623, 624, 3, 132, 66, 0, 624, 625, 3, 126, 63, 0, 625, 117, 1, 0, 0, 0, 626, 627, 5,
        12, 0, 0, 627, 628, 3, 114, 57, 0, 628, 629, 5, 13, 0, 0, 629, 630, 3, 120, 60, 0, 630,
        631, 3, 126, 63, 0, 631, 119, 1, 0, 0, 0, 632, 633, 5, 14, 0, 0, 633, 634, 3, 122, 61, 0,
        634, 635, 5, 11, 0, 0, 635, 636, 3, 122, 61, 0, 636, 637, 5, 15, 0, 0, 637, 658, 1, 0, 0,
        0, 638, 639, 5, 14, 0, 0, 639, 640, 3, 122, 61, 0, 640, 641, 5, 15, 0, 0, 641, 658, 1, 0,
        0, 0, 642, 643, 5, 14, 0, 0, 643, 644, 3, 122, 61, 0, 644, 645, 5, 11, 0, 0, 645, 646, 5,
        15, 0, 0, 646, 658, 1, 0, 0, 0, 647, 648, 5, 14, 0, 0, 648, 649, 5, 11, 0, 0, 649, 650, 3,
        122, 61, 0, 650, 651, 5, 15, 0, 0, 651, 658, 1, 0, 0, 0, 652, 653, 5, 14, 0, 0, 653, 654,
        5, 11, 0, 0, 654, 658, 5, 15, 0, 0, 655, 658, 5, 19, 0, 0, 656, 658, 5, 23, 0, 0, 657, 632,
        1, 0, 0, 0, 657, 638, 1, 0, 0, 0, 657, 642, 1, 0, 0, 0, 657, 647, 1, 0, 0, 0, 657, 652, 1,
        0, 0, 0, 657, 655, 1, 0, 0, 0, 657, 656, 1, 0, 0, 0, 658, 121, 1, 0, 0, 0, 659, 660, 7, 6,
        0, 0, 660, 123, 1, 0, 0, 0, 661, 664, 3, 180, 90, 0, 662, 664, 3, 164, 82, 0, 663, 661, 1,
        0, 0, 0, 663, 662, 1, 0, 0, 0, 664, 125, 1, 0, 0, 0, 665, 667, 5, 12, 0, 0, 666, 668, 3,
        186, 93, 0, 667, 666, 1, 0, 0, 0, 667, 668, 1, 0, 0, 0, 668, 670, 1, 0, 0, 0, 669, 671, 3,
        66, 33, 0, 670, 669, 1, 0, 0, 0, 670, 671, 1, 0, 0, 0, 671, 673, 1, 0, 0, 0, 672, 674, 3,
        124, 62, 0, 673, 672, 1, 0, 0, 0, 673, 674, 1, 0, 0, 0, 674, 675, 1, 0, 0, 0, 675, 676, 5,
        13, 0, 0, 676, 127, 1, 0, 0, 0, 677, 690, 3, 166, 83, 0, 678, 690, 3, 164, 82, 0, 679, 690,
        3, 162, 81, 0, 680, 690, 3, 158, 79, 0, 681, 690, 3, 154, 77, 0, 682, 690, 3, 150, 75, 0,
        683, 690, 3, 148, 74, 0, 684, 690, 3, 152, 76, 0, 685, 690, 3, 146, 73, 0, 686, 690, 3,
        144, 72, 0, 687, 690, 3, 186, 93, 0, 688, 690, 3, 140, 70, 0, 689, 677, 1, 0, 0, 0, 689,
        678, 1, 0, 0, 0, 689, 679, 1, 0, 0, 0, 689, 680, 1, 0, 0, 0, 689, 681, 1, 0, 0, 0, 689,
        682, 1, 0, 0, 0, 689, 683, 1, 0, 0, 0, 689, 684, 1, 0, 0, 0, 689, 685, 1, 0, 0, 0, 689,
        686, 1, 0, 0, 0, 689, 687, 1, 0, 0, 0, 689, 688, 1, 0, 0, 0, 690, 129, 1, 0, 0, 0, 691,
        692, 3, 186, 93, 0, 692, 693, 5, 1, 0, 0, 693, 131, 1, 0, 0, 0, 694, 695, 5, 6, 0, 0, 695,
        697, 5, 18, 0, 0, 696, 698, 3, 134, 67, 0, 697, 696, 1, 0, 0, 0, 697, 698, 1, 0, 0, 0, 698,
        699, 1, 0, 0, 0, 699, 701, 5, 18, 0, 0, 700, 702, 5, 5, 0, 0, 701, 700, 1, 0, 0, 0, 701,
        702, 1, 0, 0, 0, 702, 712, 1, 0, 0, 0, 703, 705, 5, 18, 0, 0, 704, 706, 3, 134, 67, 0, 705,
        704, 1, 0, 0, 0, 705, 706, 1, 0, 0, 0, 706, 707, 1, 0, 0, 0, 707, 709, 5, 18, 0, 0, 708,
        710, 5, 5, 0, 0, 709, 708, 1, 0, 0, 0, 709, 710, 1, 0, 0, 0, 710, 712, 1, 0, 0, 0, 711,
        694, 1, 0, 0, 0, 711, 703, 1, 0, 0, 0, 712, 133, 1, 0, 0, 0, 713, 715, 5, 16, 0, 0, 714,
        716, 3, 186, 93, 0, 715, 714, 1, 0, 0, 0, 715, 716, 1, 0, 0, 0, 716, 718, 1, 0, 0, 0, 717,
        719, 3, 136, 68, 0, 718, 717, 1, 0, 0, 0, 718, 719, 1, 0, 0, 0, 719, 721, 1, 0, 0, 0, 720,
        722, 3, 168, 84, 0, 721, 720, 1, 0, 0, 0, 721, 722, 1, 0, 0, 0, 722, 724, 1, 0, 0, 0, 723,
        725, 3, 124, 62, 0, 724, 723, 1, 0, 0, 0, 724, 725, 1, 0, 0, 0, 725, 726, 1, 0, 0, 0, 726,
        727, 5, 17, 0, 0, 727, 135, 1, 0, 0, 0, 728, 729, 5, 25, 0, 0, 729, 737, 3, 184, 92, 0,
        730, 732, 5, 26, 0, 0, 731, 733, 5, 25, 0, 0, 732, 731, 1, 0, 0, 0, 732, 733, 1, 0, 0, 0,
        733, 734, 1, 0, 0, 0, 734, 736, 3, 184, 92, 0, 735, 730, 1, 0, 0, 0, 736, 739, 1, 0, 0, 0,
        737, 735, 1, 0, 0, 0, 737, 738, 1, 0, 0, 0, 738, 137, 1, 0, 0, 0, 739, 737, 1, 0, 0, 0,
        740, 742, 5, 59, 0, 0, 741, 743, 5, 36, 0, 0, 742, 741, 1, 0, 0, 0, 742, 743, 1, 0, 0, 0,
        743, 744, 1, 0, 0, 0, 744, 745, 3, 10, 5, 0, 745, 139, 1, 0, 0, 0, 746, 747, 5, 45, 0, 0,
        747, 750, 5, 14, 0, 0, 748, 751, 3, 8, 4, 0, 749, 751, 3, 70, 35, 0, 750, 748, 1, 0, 0, 0,
        750, 749, 1, 0, 0, 0, 751, 752, 1, 0, 0, 0, 752, 753, 5, 15, 0, 0, 753, 141, 1, 0, 0, 0,
        754, 759, 3, 186, 93, 0, 755, 756, 5, 10, 0, 0, 756, 758, 3, 186, 93, 0, 757, 755, 1, 0, 0,
        0, 758, 761, 1, 0, 0, 0, 759, 757, 1, 0, 0, 0, 759, 760, 1, 0, 0, 0, 760, 143, 1, 0, 0, 0,
        761, 759, 1, 0, 0, 0, 762, 763, 3, 142, 71, 0, 763, 765, 5, 12, 0, 0, 764, 766, 5, 64, 0,
        0, 765, 764, 1, 0, 0, 0, 765, 766, 1, 0, 0, 0, 766, 768, 1, 0, 0, 0, 767, 769, 3, 160, 80,
        0, 768, 767, 1, 0, 0, 0, 768, 769, 1, 0, 0, 0, 769, 770, 1, 0, 0, 0, 770, 771, 5, 13, 0, 0,
        771, 145, 1, 0, 0, 0, 772, 773, 5, 12, 0, 0, 773, 774, 3, 76, 38, 0, 774, 775, 5, 13, 0, 0,
        775, 147, 1, 0, 0, 0, 776, 777, 7, 7, 0, 0, 777, 778, 5, 12, 0, 0, 778, 779, 3, 156, 78, 0,
        779, 780, 5, 13, 0, 0, 780, 149, 1, 0, 0, 0, 781, 783, 5, 16, 0, 0, 782, 784, 3, 130, 65,
        0, 783, 782, 1, 0, 0, 0, 783, 784, 1, 0, 0, 0, 784, 785, 1, 0, 0, 0, 785, 787, 3, 152, 76,
        0, 786, 788, 3, 72, 36, 0, 787, 786, 1, 0, 0, 0, 787, 788, 1, 0, 0, 0, 788, 789, 1, 0, 0,
        0, 789, 790, 5, 26, 0, 0, 790, 791, 3, 76, 38, 0, 791, 792, 5, 17, 0, 0, 792, 151, 1, 0, 0,
        0, 793, 795, 3, 126, 63, 0, 794, 796, 3, 116, 58, 0, 795, 794, 1, 0, 0, 0, 796, 797, 1, 0,
        0, 0, 797, 795, 1, 0, 0, 0, 797, 798, 1, 0, 0, 0, 798, 153, 1, 0, 0, 0, 799, 800, 5, 16, 0,
        0, 800, 803, 3, 156, 78, 0, 801, 802, 5, 26, 0, 0, 802, 804, 3, 76, 38, 0, 803, 801, 1, 0,
        0, 0, 803, 804, 1, 0, 0, 0, 804, 805, 1, 0, 0, 0, 805, 806, 5, 17, 0, 0, 806, 155, 1, 0, 0,
        0, 807, 808, 3, 186, 93, 0, 808, 809, 5, 66, 0, 0, 809, 811, 3, 76, 38, 0, 810, 812, 3, 72,
        36, 0, 811, 810, 1, 0, 0, 0, 811, 812, 1, 0, 0, 0, 812, 157, 1, 0, 0, 0, 813, 814, 5, 32,
        0, 0, 814, 815, 5, 12, 0, 0, 815, 816, 5, 23, 0, 0, 816, 817, 5, 13, 0, 0, 817, 159, 1, 0,
        0, 0, 818, 823, 3, 76, 38, 0, 819, 820, 5, 11, 0, 0, 820, 822, 3, 76, 38, 0, 821, 819, 1,
        0, 0, 0, 822, 825, 1, 0, 0, 0, 823, 821, 1, 0, 0, 0, 823, 824, 1, 0, 0, 0, 824, 161, 1, 0,
        0, 0, 825, 823, 1, 0, 0, 0, 826, 828, 5, 82, 0, 0, 827, 829, 3, 76, 38, 0, 828, 827, 1, 0,
        0, 0, 828, 829, 1, 0, 0, 0, 829, 835, 1, 0, 0, 0, 830, 831, 5, 83, 0, 0, 831, 832, 3, 76,
        38, 0, 832, 833, 5, 84, 0, 0, 833, 834, 3, 76, 38, 0, 834, 836, 1, 0, 0, 0, 835, 830, 1, 0,
        0, 0, 836, 837, 1, 0, 0, 0, 837, 835, 1, 0, 0, 0, 837, 838, 1, 0, 0, 0, 838, 841, 1, 0, 0,
        0, 839, 840, 5, 85, 0, 0, 840, 842, 3, 76, 38, 0, 841, 839, 1, 0, 0, 0, 841, 842, 1, 0, 0,
        0, 842, 843, 1, 0, 0, 0, 843, 844, 5, 86, 0, 0, 844, 163, 1, 0, 0, 0, 845, 848, 5, 27, 0,
        0, 846, 849, 3, 186, 93, 0, 847, 849, 3, 172, 86, 0, 848, 846, 1, 0, 0, 0, 848, 847, 1, 0,
        0, 0, 849, 165, 1, 0, 0, 0, 850, 858, 3, 170, 85, 0, 851, 858, 3, 172, 86, 0, 852, 858, 5,
        76, 0, 0, 853, 858, 3, 174, 87, 0, 854, 858, 3, 176, 88, 0, 855, 858, 3, 178, 89, 0, 856,
        858, 3, 180, 90, 0, 857, 850, 1, 0, 0, 0, 857, 851, 1, 0, 0, 0, 857, 852, 1, 0, 0, 0, 857,
        853, 1, 0, 0, 0, 857, 854, 1, 0, 0, 0, 857, 855, 1, 0, 0, 0, 857, 856, 1, 0, 0, 0, 858,
        167, 1, 0, 0, 0, 859, 861, 5, 23, 0, 0, 860, 862, 3, 172, 86, 0, 861, 860, 1, 0, 0, 0, 861,
        862, 1, 0, 0, 0, 862, 867, 1, 0, 0, 0, 863, 865, 5, 8, 0, 0, 864, 866, 3, 172, 86, 0, 865,
        864, 1, 0, 0, 0, 865, 866, 1, 0, 0, 0, 866, 868, 1, 0, 0, 0, 867, 863, 1, 0, 0, 0, 867,
        868, 1, 0, 0, 0, 868, 169, 1, 0, 0, 0, 869, 870, 7, 8, 0, 0, 870, 171, 1, 0, 0, 0, 871,
        872, 5, 96, 0, 0, 872, 173, 1, 0, 0, 0, 873, 874, 5, 95, 0, 0, 874, 175, 1, 0, 0, 0, 875,
        876, 5, 94, 0, 0, 876, 177, 1, 0, 0, 0, 877, 879, 5, 16, 0, 0, 878, 880, 3, 160, 80, 0,
        879, 878, 1, 0, 0, 0, 879, 880, 1, 0, 0, 0, 880, 881, 1, 0, 0, 0, 881, 882, 5, 17, 0, 0,
        882, 179, 1, 0, 0, 0, 883, 892, 5, 14, 0, 0, 884, 889, 3, 182, 91, 0, 885, 886, 5, 11, 0,
        0, 886, 888, 3, 182, 91, 0, 887, 885, 1, 0, 0, 0, 888, 891, 1, 0, 0, 0, 889, 887, 1, 0, 0,
        0, 889, 890, 1, 0, 0, 0, 890, 893, 1, 0, 0, 0, 891, 889, 1, 0, 0, 0, 892, 884, 1, 0, 0, 0,
        892, 893, 1, 0, 0, 0, 893, 894, 1, 0, 0, 0, 894, 895, 5, 15, 0, 0, 895, 181, 1, 0, 0, 0,
        896, 897, 3, 184, 92, 0, 897, 898, 5, 25, 0, 0, 898, 899, 3, 76, 38, 0, 899, 183, 1, 0, 0,
        0, 900, 903, 3, 186, 93, 0, 901, 903, 3, 188, 94, 0, 902, 900, 1, 0, 0, 0, 902, 901, 1, 0,
        0, 0, 903, 185, 1, 0, 0, 0, 904, 905, 7, 9, 0, 0, 905, 187, 1, 0, 0, 0, 906, 907, 7, 10, 0,
        0, 907, 189, 1, 0, 0, 0, 109, 192, 200, 205, 216, 222, 227, 232, 237, 239, 247, 256, 260,
        263, 266, 270, 276, 282, 286, 295, 301, 308, 311, 313, 318, 323, 325, 331, 336, 349, 356,
        359, 370, 377, 384, 388, 397, 401, 406, 415, 428, 442, 448, 455, 465, 473, 481, 489, 495,
        506, 516, 524, 532, 536, 544, 546, 553, 557, 560, 563, 573, 577, 583, 590, 596, 600, 610,
        612, 619, 621, 657, 663, 667, 670, 673, 689, 697, 701, 705, 709, 711, 715, 718, 721, 724,
        732, 737, 742, 750, 759, 765, 768, 783, 787, 797, 803, 811, 823, 828, 837, 841, 848, 857,
        861, 865, 867, 879, 889, 892, 902
    ];
}
