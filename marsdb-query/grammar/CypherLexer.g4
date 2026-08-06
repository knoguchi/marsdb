/*
 [The "BSD licence"]
 Copyright (c) 2022 Boris Zhguchev
 All rights reserved.

 Redistribution and use in source and binary forms, with or without
 modification, are permitted provided that the following conditions
 are met:
 1. Redistributions of source code must retain the above copyright
    notice, this list of conditions and the following disclaimer.
 2. Redistributions in binary form must reproduce the above copyright
    notice, this list of conditions and the following disclaimer in the
    documentation and/or other materials provided with the distribution.
 3. The name of the author may not be used to endorse or promote products
    derived from this software without specific prior written permission.

 THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
 IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
 INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

// $antlr-format alignTrailingComments true, columnLimit 150, maxEmptyLinesToKeep 1, reflowComments false, useTab false
// $antlr-format allowShortRulesOnASingleLine true, allowShortBlocksOnASingleLine true, minEmptyLines 0, alignSemicolons ownLine
// $antlr-format alignColons trailing, singleLineOverrulesHangingColon true, alignLexerCommands true, alignLabels true, alignTrailers true

lexer grammar CypherLexer;
channels {
    COMMENTS
}
options {
    caseInsensitive = true;
}

ASSIGN     : '=';
ADD_ASSIGN : '+=';
LE         : '<=';
GE         : '>=';
GT         : '>';
LT         : '<';
NOT_EQUAL  : '<>';
RANGE      : '..';
SEMI       : ';';
DOT        : '.';
COMMA      : ',';
LPAREN     : '(';
RPAREN     : ')';
LBRACE     : '{';
RBRACE     : '}';
LBRACK     : '[';
RBRACK     : ']';
SUB        : '-';
PLUS       : '+';
DIV        : '/';
MOD        : '%';
CARET      : '^';
MULT       : '*';
ESC        : '`';
COLON      : ':';
STICK      : '|';
DOLLAR     : '$';

CALL       : 'CALL';
YIELD      : 'YIELD';
FILTER     : 'FILTER';
EXTRACT    : 'EXTRACT';
COUNT      : 'COUNT';
ANY        : 'ANY';
NONE       : 'NONE';
SINGLE     : 'SINGLE';
ALL        : 'ALL';
ASC        : 'ASC';
ASCENDING  : 'ASCENDING';
BY         : 'BY';
CREATE     : 'CREATE';
DELETE     : 'DELETE';
DESC       : 'DESC';
DESCENDING : 'DESCENDING';
DETACH     : 'DETACH';
EXISTS     : 'EXISTS';
LIMIT      : 'LIMIT';
MATCH      : 'MATCH';
MERGE      : 'MERGE';
ON         : 'ON';
OPTIONAL   : 'OPTIONAL';
ORDER      : 'ORDER';
REMOVE     : 'REMOVE';
RETURN     : 'RETURN';
SET        : 'SET';
SKIP_W     : 'SKIP';
WHERE      : 'WHERE';
WITH       : 'WITH';
UNION      : 'UNION';
UNWIND     : 'UNWIND';
AND        : 'AND';
AS         : 'AS';
CONTAINS   : 'CONTAINS';
DISTINCT   : 'DISTINCT';
ENDS       : 'ENDS';
IN         : 'IN';
IS         : 'IS';
NOT        : 'NOT';
OR         : 'OR';
STARTS     : 'STARTS';
XOR        : 'XOR';
FALSE      : 'FALSE';
TRUE       : 'TRUE';
NULL_W     : 'NULL';
CONSTRAINT : 'CONSTRAINT';
DO         : 'DO';
FOR        : 'FOR';
REQUIRE    : 'REQUIRE';
UNIQUE     : 'UNIQUE';
CASE       : 'CASE';
WHEN       : 'WHEN';
THEN       : 'THEN';
ELSE       : 'ELSE';
END        : 'END';
MANDATORY  : 'MANDATORY';
SCALAR     : 'SCALAR';
OF         : 'OF';
ADD        : 'ADD';
DROP       : 'DROP';

ID: LetterOrDigit+;

ESC_LITERAL    : '`' .*? '`';
// Upstream (antlr/grammars-v4) had `?` here, capping single-quoted content
// at 0-1 characters -- any real (2+ char) single-quoted Cypher string
// silently failed to tokenize as this rule, and the orphaned quote/content
// fell through to ERRCHAR below instead of erroring. Fixed to `*` to match
// STRING_LITERAL's shape; Cypher treats '...' and "..." as equivalent
// string literal forms, not distinct char-vs-string types.
CHAR_LITERAL   : '\'' (~['\\\r\n] | EscapeSequence)* '\'';
STRING_LITERAL : '"' (~["\\\r\n] | EscapeSequence)* '"';

DIGIT : SUB? (HexDigit | OctalDigit | Digits | FLOAT);
FLOAT : (Digits '.' Digits | '.' Digits) ExponentPart? [fd]? | Digits (ExponentPart [fd]? | [fd]);

WS           : [ \t\r\n\u000C]+ -> channel(HIDDEN);
COMMENT      : '/*' .*? '*/'    -> channel(COMMENTS);
LINE_COMMENT : '//' ~[\r\n]*    -> channel(COMMENTS);
// Upstream sent any unrecognized character to the HIDDEN channel instead of
// erroring -- silently swallowing it rather than causing a syntax error,
// which is how the CHAR_LITERAL bug above turned into silent mis-parses
// instead of clean rejections. Removed: without a catch-all, ANTLR's
// default lexer behavior raises a real syntax_error on unmatched input,
// which is what we want.

fragment EscapeSequence:
    '\\' [btnfr"'\\]
    | '\\' ([0-3]? [0-7])? [0-7]
    | '\\' 'u'+ HexDigit HexDigit HexDigit HexDigit
;

fragment ExponentPart: [e] [+-]? Digits;

fragment HexDigits  : '0x' HexDigit ((HexDigit | '_')* HexDigit)?;
fragment HexDigit   : [0-9a-f];
fragment OctalDigit : '0' Digits;
fragment Digits     : [1-9] ([0-9_]* [0-9])?;

fragment LetterOrDigit: Letter | [0-9];

Letter:
    [a-z_]
    | ~[\u0000-\u007F\uD800-\uDBFF] // covers all characters above 0x7F which are not a surrogate
    | [\uD800-\uDBFF] [\uDC00-\uDFFF]
; // covers UTF-16 surrogate pairs encodings for U+10000 to U+10FFFF