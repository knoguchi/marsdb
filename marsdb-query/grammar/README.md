# ANTLR Cypher grammar

`CypherLexer.g4` / `CypherParser.g4` are vendored from
[`antlr/grammars-v4/cypher`](https://github.com/antlr/grammars-v4/tree/master/cypher)
(BSD-3-Clause, Copyright (c) 2022 Boris Zhguchev — see license header in each
file). This is a community ANTLR4 grammar for Cypher, not an openCypher-project
artifact — openCypher's own `grammar/` directory (vendored via the
`marsdb-tck/openCypher` submodule) is hand-written ISO WG3 BNF prose
(`openCypher.bnf`), not machine-generatable into ANTLR form. There is no
official openCypher `Cypher.g4`.

These two files are the reconciliation target: adapt them against
`marsdb-tck/openCypher/grammar/openCypher.bnf` (spec) and the TCK corpus
(conformance), not treat them as ground truth as-is.

## Local fixes to upstream

`CypherLexer.g4` has two changes from upstream `antlr/grammars-v4/cypher`,
found via the TCK spike below and confirmed with `antlr_debug_tree_text`
(returns the raw parse tree text, showing exactly what got tokenized):

- `CHAR_LITERAL` (single-quoted strings) used `?` instead of `*`, capping
  content at 0-1 characters. Any real single-quoted Cypher string (2+ chars
  — i.e. almost all of them) failed to tokenize as this rule.
- `ERRCHAR` sent any unrecognized character to the `HIDDEN` channel instead
  of erroring — silently discarding it rather than raising a syntax error.

Combined, these two didn't just reject valid strings — they silently
mis-parsed them: `RETURN 'hello world'` tokenized as `RETURN` `hello`
`world` (quotes and space vanished), and `RETURN foo('a:b')` tokenized as
`RETURN foo(a:b)` (a symbol with a label predicate, not a string argument)
— both accepted as different, wrong queries instead of rejected. Fixed by
widening `CHAR_LITERAL` to `*` and removing `ERRCHAR` (ANTLR's default
lexer behavior raises a real `syntax_error` on unmatched input once there's
no catch-all swallowing it first).

Two more fixes, same file, same discovery method:

- `ID: LetterOrDigit+;` let an identifier start with — or be entirely
  composed of — digits. Per `openCypher.bnf`'s
  `<regular identifier> ::= <identifier start> [<identifier extend>...]`,
  where `<identifier start>` is Unicode `XID_START` (excludes digits;
  digits are only valid as `XID_CONTINUE`, i.e. after the first
  character), an identifier can never start with a digit. Since `DIGIT`
  and `ID` tied in length on pure-digit input and `ID` was declared
  first, ANTLR's tie-break silently lexed numbers as identifiers
  everywhere *except* grammar positions that specifically require
  `DIGIT` (e.g. variable-length relationship bounds `[:REL*2]`), where
  it hard-failed instead of parsing the number. Fixed to
  `ID: Letter LetterOrDigit*;` (first character must be a letter).
- `Digits: [1-9] ([0-9_]* [0-9])?;` disallowed a leading zero, but
  `openCypher.bnf`'s `<unsigned decimal integer>` is just
  `<digit> [{[_]<digit>}...]` — no such restriction. `007` couldn't
  tokenize as `DIGIT` at all under the old rule. Fixed to `[0-9] (...)?`.

These fixes raised the TCK parse-acceptance comparison below from 90.5%
to 96.4% and eliminated the entire `Temporal1/2/10` (149 cases) and
`Match5` (27 cases) disagreement categories — those weren't real grammar
gaps, just corrupted date-string literals (`'2018-01-01T12:00'` contains
`-`/`:`) and bounded variable-length relationship patterns
(`[:REL*2]`/`[:REL*1..3]`) failing on their numeric bounds.

One structural (not lexer) fix, in `CypherParser.g4`:

- `multiPartQ : readingStatement* (updatingStatement* withSt)+ singlePartQ;`
  only allowed `updatingStatement`s (CREATE/MERGE/DELETE/SET/REMOVE)
  between a multi-part query's `WITH` boundaries, not `readingStatement`s
  (MATCH/UNWIND/CALL) — so ordinary, idiomatic chains like
  `WITH ... UNWIND ... WITH ...` or `WITH ... MATCH ... WITH ...` couldn't
  parse at all. `readingStatement` can't satisfy `updatingStatement*`, and
  `singlePartQ` (what the grammar falls back to once the `(...)+ ` group
  can't continue) has no `withSt` of its own to absorb a second `WITH`.
  Fixed to `((readingStatement | updatingStatement)* withSt)+`.

This raised acceptance from 96.4% to 98.5% and eliminated the entire
`Quantifier9/10/11/12` disagreement category (64 cases) — those TCK
scenarios chain several `UNWIND`/`WITH` pairs, which simply couldn't
parse under the old rule.

Four more, back in `CypherLexer.g4`/`CypherParser.g4` (the diagnostic was
also extended at this point to cross-check each reject against the TCK
scenario's own expected outcome — `Expected::AnyError` cases are supposed
to reject, so only mismatches count as real bugs):

- `DIGIT`'s hex alternative referenced `HexDigit` (singular — a bare
  `[0-9a-f]` single-char fragment) instead of `HexDigits` (plural — the
  actual `0x`-prefixed fragment, defined but never used). `0x1` etc
  couldn't tokenize as a hex literal at all.
- No `0o`-prefixed octal support existed at all (the old `OctalDigit` was
  `'0' Digits`, a bare-leading-zero form with no basis in
  `openCypher.bnf`, which defines `<unsigned octal integer>` as requiring
  an explicit `0o` prefix). Added a proper `OctalDigits` fragment.
- `notExpression: NOT? comparisonExpression;` allowed at most one `NOT`,
  so `NOT NOT true` (ordinary double negation) couldn't parse. Fixed to
  `NOT*`.
- `DIGIT`'s leading `SUB?` let the lexer's maximal-munch rule greedily
  swallow a *binary* minus into the next operand's token: tokenizing
  `5-1` starting at position 1, `-1` (2 chars, matches `SUB? Digits`)
  beats `-` alone (1 char, `SUB`) under longest-match, leaving two
  adjacent `DIGIT` tokens with no operator between them — a parse error
  on ordinary subtraction with no surrounding whitespace. The parser
  already has correct unary-minus handling at the right precedence level
  (`unaryAddSubExpression: (PLUS | SUB)? atomicExpression`), so `DIGIT`
  embedding its own sign was both redundant and actively wrong. Removed.

These raised acceptance from 98.5% to 99.4% (3858/3880) and brought real
grammar bugs to zero — all 22 remaining rejects are scenarios the TCK
itself expects to fail (`Expected::AnyError`), confirmed by cross-checking
each one, not assumed.

## Local extensions (not from upstream, not real openCypher)

Unlike the fixes above, these additions aren't upstream-worthy — they
have no basis in `openCypher.bnf` at all, mirroring `cypher.pest`'s own
identically-scoped mars-specific extensions (not sent to
`antlr/grammars-v4`, which tracks real Cypher, not this):

- `explainSt : EXPLAIN (createIndexSt | regularQuery)` — `EXPLAIN
  <statement>` (describe the plan without running it). Never wraps another
  `explainSt` or `standaloneCall` (CALL has no `Statement` representation
  in this grammar's visitor yet regardless — see mars-82w).
- `createIndexSt : CREATE INDEX ON COLON name LPAREN name RPAREN UNIQUE?`
  — `CREATE INDEX ON :Label(prop)`, optionally `UNIQUE`. Deliberately the
  older, simpler single-property syntax, not real openCypher's newer
  `CREATE INDEX FOR (n:Label) ON (n.prop)` / `CREATE CONSTRAINT ... IS
  UNIQUE`.

Both needed two new lexer tokens (`EXPLAIN`, `INDEX`) added to
`reservedWord` too, so `name` (label/property positions) can still absorb
them — only `symbol` (bound-variable positions) excludes reserved words,
matching every other keyword already in that list.

A third, same class (real Neo4j Cypher syntax, but absent from
`openCypher.bnf`/the TCK — confirmed by grep, not assumed — so treated as
a local extension here too, mirroring `cypher.pest`'s own
`shortest_path_wrapper`):

- `patternPart : (symbol ASSIGN)? (shortestPathWrapper | patternElem)`,
  `shortestPathWrapper : SHORTEST_PATH LPAREN patternElem RPAREN` —
  `shortestPath((a)-[*..5]-(b))`. Only the single-path form, not
  `allShortestPaths(...)` (pest doesn't have that either). Grammar-
  permissive — `patternPart` is shared by MATCH/CREATE/MERGE, so this adds
  a new lexer token, `SHORTEST_PATH` (also added to `reservedWord`), and
  syntactically-legal-but-visitor-rejected positions (CREATE, MERGE, any
  comma position but the first) rather than threading a MATCH-only rule
  through three call sites — same "grammar permissive, visitor enforces
  the exact constraint" split already used for `(symbol ASSIGN)?` on the
  same rule.

A fourth: `queries : query (SEMI query)* EOF` — a `;`-separated batch of
one or more statements (`"CREATE (a); CREATE (b); MATCH (n) RETURN n"`),
parsed via `parse_antlr_many`. Mirrors `cypher.pest`'s own `queries` rule
exactly, including having no trailing `SEMI?` of its own (a single
genuinely-trailing `;` is stripped in Rust before parsing, same as
`parser::parse_many` already does, avoiding the same ambiguity its own
doc comment describes). Not from openCypher either — real Cypher has no
concept of a single textual submission containing multiple statements at
all — but needed for parity with `parser::parse_many`, part of this
crate's real public API.

## Why this toolchain

ANTLR4's Rust code-generation target is not in mainline ANTLR4 (never merged
upstream). The actively maintained fork living under the `antlr4rust` GitHub
org is what this repo uses:

- Generator (Java tool): [`antlr4rust/antlr4`](https://github.com/antlr4rust/antlr4),
  release `v0.5.0`, jar `antlr4-4.13.3-SNAPSHOT-complete.jar` — targets ANTLR
  4.13.3, current with mainline ANTLR4.
- Runtime (Rust crate): [`antlr4rust`](https://crates.io/crates/antlr4rust)
  (note: **not** the older, stale `antlr-rust` crate — same project lineage,
  different/renamed crate, actively published through 0.5.2).

Verified working end-to-end (jar → generated Rust → compiles → parses) before
adopting this path; see beads issue `mars-0mn`.

## Regenerating

Only needed when `CypherLexer.g4`/`CypherParser.g4` change (grammar fixes) —
not part of the normal build. Generated output is committed to
`marsdb-query/src/generated/`.

```
# One-time: fetch the generator jar (pin: v0.5.0 / ANTLR 4.13.3)
curl -sL -o /tmp/antlr4.jar \
  https://github.com/antlr4rust/antlr4/releases/download/v0.5.0/antlr4-4.13.3-SNAPSHOT-complete.jar

# Generate into a scratch dir first -- the parser grammar's `tokenVocab`
# import looks for CypherLexer.tokens next to the source .g4 files, so
# `-o marsdb-query/src/generated` directly fails with "cannot find tokens
# file". Generating alongside the .g4 source and then copying works.
rm -rf /tmp/antlr-gen && mkdir -p /tmp/antlr-gen
cp marsdb-query/grammar/*.g4 /tmp/antlr-gen/
(cd /tmp/antlr-gen && java -jar /tmp/antlr4.jar -Dlanguage=Rust -visitor \
  CypherLexer.g4 CypherParser.g4)
rm -f marsdb-query/src/generated/*.rs marsdb-query/src/generated/*.tokens \
  marsdb-query/src/generated/*.interp
cp /tmp/antlr-gen/*.rs /tmp/antlr-gen/*.tokens /tmp/antlr-gen/*.interp \
  marsdb-query/src/generated/

# ANTLR's own formatting doesn't match rustfmt -- CI's fmt --check job
# will fail on freshly generated output otherwise. Also re-add the hand
# written antlr_accepts/antlr_debug_tree_text/mod declarations at the top
# of mod.rs -- the `rm -f *.rs` above deletes it along with the generated
# files since it matches the same glob; recover from git and re-copy in
# the generated `pub mod` lines if the file list has changed.
cargo fmt -p marsdb-query
```

Commit the resulting `marsdb-query/src/generated/*.rs` alongside the `.g4`
change. `marsdb-query/Cargo.toml` pins the matching runtime:
`antlr4rust = "0.5.2"`.
