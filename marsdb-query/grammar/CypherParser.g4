/*
 [The "BSD licence"]
 Copyright (c) 2022 Boris Zhguchev
 All rights reserved.

 Redistribution and use in source and binary forms with or without
 modification are permitted provided that the following conditions
 are met:
 1. Redistributions of source code must retain the above copyright
    notice this list of conditions and the following disclaimer.
 2. Redistributions in binary form must reproduce the above copyright
    notice this list of conditions and the following disclaimer in the
    documentation and/or other materials provided with the distribution.
 3. The name of the author may not be used to endorse or promote products
    derived from this software without specific prior written permission.

 THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
 IMPLIED WARRANTIES INCLUDING BUT NOT LIMITED TO THE IMPLIED WARRANTIES
 OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT INDIRECT
 INCIDENTAL SPECIAL EXEMPLARY OR CONSEQUENTIAL DAMAGES (INCLUDING BUT
 NOT LIMITED TO PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE
 DATA OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 THEORY OF LIABILITY WHETHER IN CONTRACT STRICT LIABILITY OR TORT
 (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 THIS SOFTWARE EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

// $antlr-format alignTrailingComments true, columnLimit 150, minEmptyLines 1, maxEmptyLinesToKeep 1, reflowComments false, useTab false
// $antlr-format allowShortRulesOnASingleLine false, allowShortBlocksOnASingleLine true, alignSemicolons hanging, alignColons hanging

parser grammar CypherParser;

options {
    tokenVocab = CypherLexer;
}

script
    : query SEMI? EOF
    ;

// mars-specific extension (see grammar/README.md) -- a `;`-separated batch
// of one or more statements, e.g. `CREATE (a); CREATE (b); MATCH (n)
// RETURN n`. Mirrors cypher.pest's `queries` rule exactly: no trailing
// SEMI here either (parse_antlr_many strips a single genuinely-trailing
// `;` in Rust first, same as parser::parse_many does).
queries
    : query (SEMI query)* EOF
    ;

// statements
query
    : explainSt
    | regularQuery
    | standaloneCall
    | createIndexSt
    ;

// `EXPLAIN <statement>` -- mars-specific, not real openCypher syntax (no
// equivalent in `openCypher.bnf`). Never wraps another `explainSt` (real
// `EXPLAIN EXPLAIN ...` isn't valid either) or `standaloneCall` (CALL isn't
// implemented by this grammar's visitor yet regardless).
explainSt
    : EXPLAIN (createIndexSt | regularQuery)
    ;

// `CREATE INDEX ON :Label(prop)`, optionally `UNIQUE` -- mars-specific,
// deliberately the older, simpler single-property syntax, not real
// openCypher's newer `CREATE INDEX FOR (n:Label) ON (n.prop)` /
// `CREATE CONSTRAINT ... IS UNIQUE`. Mirrors `cypher.pest`'s
// `create_index_stmt` exactly.
createIndexSt
    : CREATE INDEX ON COLON name LPAREN name RPAREN UNIQUE?
    ;

regularQuery
    : singleQuery unionSt*
    ;

singleQuery
    : singlePartQ
    | multiPartQ
    ;

standaloneCall
    : CALL invocationName parenExpressionChain? (YIELD (MULT | yieldItems))?
    ;

returnSt
    : RETURN projectionBody
    ;

withSt
    : WITH projectionBody where?
    ;

skipSt
    : SKIP_W expression
    ;

limitSt
    : LIMIT expression
    ;

projectionBody
    : DISTINCT? projectionItems orderSt? skipSt? limitSt?
    ;

projectionItems
    : (MULT | projectionItem) (COMMA projectionItem)*
    ;

projectionItem
    : expression (AS symbol)?
    ;

orderItem
    : expression (ASCENDING | ASC | DESCENDING | DESC)?
    ;

orderSt
    : ORDER BY orderItem (COMMA orderItem)*
    ;

singlePartQ
    : readingStatement* (returnSt | updatingStatement+ returnSt?)
    ;

// Upstream (antlr/grammars-v4) only allowed `updatingStatement`s between a
// multi-part query's WITH boundaries, not `readingStatement`s (MATCH/
// UNWIND/CALL) -- so `WITH ... UNWIND ... WITH ...` or
// `WITH ... MATCH ... WITH ...` (ordinary, idiomatic Cypher) couldn't
// parse: readingStatement isn't updatingStatement, so it can't satisfy
// `updatingStatement*` before the next `withSt`, and it can't be absorbed
// by `singlePartQ` either since `singlePartQ` has no trailing `withSt` of
// its own. Fixed to allow either kind of statement before each WITH,
// matching openCypher's actual clause-chaining rules.
multiPartQ
    : readingStatement* ((readingStatement | updatingStatement)* withSt)+ singlePartQ
    ;

matchSt
    : OPTIONAL? MATCH patternWhere
    ;

unwindSt
    : UNWIND expression AS symbol
    ;

readingStatement
    : matchSt
    | unwindSt
    | queryCallSt
    ;

updatingStatement
    : createSt
    | mergeSt
    | deleteSt
    | setSt
    | removeSt
    ;

deleteSt
    : DETACH? DELETE expressionChain
    ;

removeSt
    : REMOVE removeItem (COMMA removeItem)*
    ;

removeItem
    : symbol nodeLabels
    | propertyExpression
    ;

queryCallSt
    : CALL invocationName parenExpressionChain (YIELD yieldItems)?
    ;

parenExpressionChain
    : LPAREN expressionChain? RPAREN
    ;

yieldItems
    : yieldItem (COMMA yieldItem)* where?
    ;

yieldItem
    : (symbol AS)? symbol
    ;

mergeSt
    : MERGE patternPart mergeAction*
    ;

mergeAction
    : ON (MATCH | CREATE) setSt
    ;

setSt
    : SET setItem (COMMA setItem)*
    ;

setItem
    : propertyExpression ASSIGN expression
    | symbol (ASSIGN | ADD_ASSIGN) expression
    | symbol nodeLabels
    ;

nodeLabels
    : (COLON name)+
    ;

createSt
    : CREATE pattern
    ;

patternWhere
    : pattern where?
    ;

where
    : WHERE expression
    ;

pattern
    : patternPart (COMMA patternPart)*
    ;

expression
    : xorExpression (OR xorExpression)*
    ;

xorExpression
    : andExpression (XOR andExpression)*
    ;

andExpression
    : notExpression (AND notExpression)*
    ;

// Upstream had `NOT?` -- at most one negation, so `NOT NOT true` (double
// negation, ordinary Cypher) couldn't parse. Fixed to `NOT*` to allow
// chained negation.
notExpression
    : NOT* comparisonExpression
    ;

// Fixed precedence bug (found via a Phase 3 behavioral dry-run, not the
// TCK -- see grammar/README.md): `IN`/`STARTS WITH`/`ENDS WITH`/
// `CONTAINS`/`IS NULL` used to attach at `atomicExpression`'s level,
// binding *tighter* than `+`/`-`/`*`/`/`/`^` (so `n.val + 0 IS NULL`
// parsed as `n.val + (0 IS NULL)`). Per openCypher.bnf's `<comparison
// predicate>` chain (`<simple comparison predicand> ::= <advanced
// comparison predicand> [<advanced comparison predicate part 2>]`, where
// `<advanced comparison predicand> ::= <arithmetic value expression>` and
// `<advanced comparison predicate part 2>` is exactly these operators),
// they belong here instead: above arithmetic, below `=`/`<>`/`</>/<=/>=`.
comparisonExpression
    : stringListNullExpression (comparisonSigns stringListNullExpression)*
    ;

// At most one suffix (spec's own `[...]` is a single optional, not
// repeatable) -- `a IN b IN c` / `a IS NULL IS NULL` aren't valid Cypher
// either way, so no "grammar permissive, visitor enforces" split is
// needed here (unlike this same file's `(symbol ASSIGN)?` pattern) --
// the grammar itself already matches the real constraint.
stringListNullExpression
    : addSubExpression (stringExpression | inExpression | nullExpression)?
    ;

inExpression
    : IN addSubExpression
    ;

comparisonSigns
    : ASSIGN
    | LE
    | GE
    | GT
    | LT
    | NOT_EQUAL
    ;

addSubExpression
    : multDivExpression ((PLUS | SUB) multDivExpression)*
    ;

multDivExpression
    : powerExpression ((MULT | DIV | MOD) powerExpression)*
    ;

powerExpression
    : unaryAddSubExpression (CARET unaryAddSubExpression)*
    ;

unaryAddSubExpression
    : (PLUS | SUB)? atomicExpression
    ;

// `IN`/`stringExpression`/`nullExpression` moved up to
// `stringListNullExpression` (see its own docs) -- only the real postfix
// forms (index/slice) stay here, tight/repeatable, matching
// openCypher.bnf's own `<postfix expression> ::= ... | <postfix
// expression> <postfix operator>` (`<dynamic element reference>`/
// `<slicing>`).
atomicExpression
    : propertyOrLabelExpression (listExpression)*
    ;

listExpression
    : LBRACK (expression? RANGE expression? | expression) RBRACK
    ;

// Operand widened from `propertyOrLabelExpression` to `addSubExpression`
// (moved up a level, see `stringListNullExpression`) -- matches spec's
// `<advanced comparison predicand> ::= <arithmetic value expression>`.
stringExpression
    : stringExpPrefix addSubExpression
    ;

stringExpPrefix
    : STARTS WITH
    | ENDS WITH
    | CONTAINS
    ;

nullExpression
    : IS NOT? NULL_W
    ;

propertyOrLabelExpression
    : propertyExpression nodeLabels?
    ;

propertyExpression
    : atom (DOT name)*
    ;

// `shortestPathWrapper` is grammar-permissive here (usable at any
// comma-separated position, in CREATE/MERGE's own `patternPart` too, not
// just MATCH's first pattern) -- the visitor enforces the real restriction
// (MATCH only, first comma position only), same "grammar permissive,
// visitor enforces the exact constraint" split already used for
// `(symbol ASSIGN)?` named-path capture right below.
patternPart
    : (symbol ASSIGN)? (shortestPathWrapper | patternElem)
    ;

// mars-specific extension, same as `explainSt`/`createIndexSt` (see
// `grammar/README.md`) -- real Cypher's `shortestPath(...)`/
// `allShortestPaths(...)`, though only the single-path form is
// implemented, matching `cypher.pest`'s own `shortest_path_wrapper`.
shortestPathWrapper
    : SHORTEST_PATH LPAREN patternElem RPAREN
    ;

patternElem
    : nodePattern (patternElemChain | qppElemChain)*
    | LPAREN patternElem RPAREN qppQuantifier?
    ;

patternElemChain
    : relationshipPattern nodePattern
    ;

qppElemChain
    : LPAREN patternElem RPAREN qppQuantifier nodePattern
    ;

qppQuantifier
    : LBRACE qppInt COMMA qppInt RBRACE
    | LBRACE qppInt RBRACE
    | LBRACE qppInt COMMA RBRACE
    | LBRACE COMMA qppInt RBRACE
    | LBRACE COMMA RBRACE
    | PLUS
    | MULT
    ;

qppInt
    : DIGIT
    | ID
    ;

properties
    : mapLit
    | parameter
    ;

nodePattern
    : LPAREN symbol? nodeLabels? properties? RPAREN
    ;

// `listComprehension` tried before `literal` -- `[x IN list]` (no WHERE,
// no `| project`) is genuinely ambiguous with a one-element `listLit`
// containing the boolean `x IN list` membership test (`literal`'s own
// `listLit: LBRACK expressionChain? RBRACK` alternative, since a bare `x
// IN list` is itself a legal single `expression`). Per openCypher.bnf's
// `<list comprehension> ::= <left bracket> <list element source> [<list
// element filter and projection>] <right bracket>`, the filter/projection
// half is optional, so `[x IN list]` is real, spec-valid Cypher on its
// own -- ANTLR resolves a genuine ambiguity by picking whichever
// alternative appears first, so this one must come first for that case
// to win (when WHERE/`|` is present there's no ambiguity at all, since
// `literal`'s `expressionChain` has no WHERE production).
atom
    : listComprehension
    | literal
    | parameter
    | caseExpression
    | countAll
    | patternComprehension
    | filterWith
    | relationshipsChainPattern
    | parenthesizedExpression
    | functionInvocation
    | symbol
    | subqueryExist
    ;

lhs
    : symbol ASSIGN
    ;

relationshipPattern
    : LT SUB relationDetail? SUB GT?
    | SUB relationDetail? SUB GT?
    ;

relationDetail
    : LBRACK symbol? relationshipTypes? rangeLit? properties? RBRACK
    ;

relationshipTypes
    : COLON name (STICK COLON? name)*
    ;

unionSt
    : UNION ALL? singleQuery
    ;

subqueryExist
    : EXISTS LBRACE (regularQuery | patternWhere) RBRACE
    ;

invocationName
    : symbol (DOT symbol)*
    ;

functionInvocation
    : invocationName LPAREN DISTINCT? expressionChain? RPAREN
    ;

parenthesizedExpression
    : LPAREN expression RPAREN
    ;

filterWith
    : (ALL | ANY | NONE | SINGLE) LPAREN filterExpression RPAREN
    ;

patternComprehension
    : LBRACK lhs? relationshipsChainPattern where? STICK expression RBRACK
    ;

relationshipsChainPattern
    : nodePattern patternElemChain+
    ;

listComprehension
    : LBRACK filterExpression (STICK expression)? RBRACK
    ;

filterExpression
    : symbol IN expression where?
    ;

countAll
    : COUNT LPAREN MULT RPAREN
    ;

expressionChain
    : expression (COMMA expression)*
    ;

caseExpression
    : CASE expression? (WHEN expression THEN expression)+ (ELSE expression)? END
    ;

parameter
    : DOLLAR (symbol | numLit)
    ;

// literals
literal
    : boolLit
    | numLit
    | NULL_W
    | stringLit
    | charLit
    | listLit
    | mapLit
    ;

rangeLit
    : MULT numLit? (RANGE numLit?)?
    ;

boolLit
    : TRUE
    | FALSE
    ;

numLit
    : DIGIT
    ;

stringLit
    : STRING_LITERAL
    ;

charLit
    : CHAR_LITERAL
    ;

listLit
    : LBRACK expressionChain? RBRACK
    ;

mapLit
    : LBRACE (mapPair (COMMA mapPair)*)? RBRACE
    ;

mapPair
    : name COLON expression
    ;

// primitive ids
name
    : symbol
    | reservedWord
    ;

symbol
    : ESC_LITERAL
    | ID
    | COUNT
    | FILTER
    | EXTRACT
    | ANY
    | NONE
    | SINGLE
    ;

reservedWord
    : ALL
    | ASC
    | ASCENDING
    | BY
    | CREATE
    | DELETE
    | DESC
    | DESCENDING
    | DETACH
    | EXISTS
    | EXPLAIN
    | LIMIT
    | MATCH
    | MERGE
    | ON
    | OPTIONAL
    | ORDER
    | REMOVE
    | RETURN
    | SET
    | SKIP_W
    | WHERE
    | WITH
    | UNION
    | UNWIND
    | AND
    | AS
    | CONTAINS
    | DISTINCT
    | ENDS
    | IN
    | INDEX
    | IS
    | NOT
    | OR
    | STARTS
    | XOR
    | SHORTEST_PATH
    | FALSE
    | TRUE
    | NULL_W
    | CONSTRAINT
    | DO
    | FOR
    | REQUIRE
    | UNIQUE
    | CASE
    | WHEN
    | THEN
    | ELSE
    | END
    | MANDATORY
    | SCALAR
    | OF
    | ADD
    | DROP
    ;