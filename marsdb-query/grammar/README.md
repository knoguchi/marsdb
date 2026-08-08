# ANTLR Cypher grammar

`CypherLexer.g4` / `CypherParser.g4` are vendored from
[`antlr/grammars-v4/cypher`](https://github.com/antlr/grammars-v4/tree/master/cypher)
(BSD-3-Clause, Copyright (c) 2022 Boris Zhguchev — see the license header in
each file). That project is a community ANTLR4 grammar for Cypher; it is not
maintained by the openCypher project, and openCypher itself has no official
`.g4` file (its own grammar, vendored here via the `marsdb-tck/openCypher`
submodule, is hand-written ISO WG3 BNF prose in `openCypher.bnf`, not
machine-generatable into ANTLR form).

These two files are patched against `openCypher.bnf` and the openCypher TCK
test corpus — not treated as correct as vendored.

## Fixes to the upstream grammar

All of these are real bugs relative to `openCypher.bnf`, found by running the
TCK against the parser:

- **String literals silently corrupted.** `CHAR_LITERAL` used `?` instead of
  `*`, capping single-quoted string content at 0-1 characters, and `ERRCHAR`
  routed unrecognized characters to a hidden channel instead of raising an
  error. Together these didn't reject bad input, they silently mis-tokenized
  it: `RETURN 'hello world'` parsed as `RETURN hello world` (quotes and space
  dropped). Fixed by widening `CHAR_LITERAL` to `*` and removing `ERRCHAR`.
- **Identifiers could start with a digit.** `ID: LetterOrDigit+;` let a
  number lex as an identifier instead of a `DIGIT` token almost everywhere,
  breaking positions that specifically require a number (e.g. `[:REL*2]`).
  Fixed to `ID: Letter LetterOrDigit*;`, matching `openCypher.bnf`'s
  identifier rule (must start with a letter).
- **No leading zero allowed in integers.** `Digits: [1-9] ([0-9_]* [0-9])?;`
  couldn't tokenize `007`. `openCypher.bnf` has no such restriction. Fixed to
  `[0-9] (...)?`.
- **Hex/octal literals didn't work.** The hex rule referenced the wrong
  fragment name, and octal had no `0o`-prefixed form at all (as
  `openCypher.bnf` requires). Fixed both.
- **Double negation didn't parse.** `notExpression: NOT? comparisonExpression`
  allowed only one `NOT`, so `NOT NOT true` failed. Fixed to `NOT*`.
- **Ordinary subtraction without spaces failed.** `DIGIT`'s optional leading
  `SUB?` let the lexer's longest-match rule swallow a binary minus into the
  next operand's token, so `5-1` mis-tokenized. The parser already handles
  unary minus correctly elsewhere, so this was redundant and wrong. Removed.
- **Multi-part queries rejected valid clause chains.** The rule for what can
  appear between two `WITH`s only allowed updating clauses (CREATE/MERGE/
  DELETE/SET/REMOVE), not reading clauses (MATCH/UNWIND/CALL) — so
  `WITH ... UNWIND ... WITH ...` couldn't parse. Fixed to allow either.
- **Operator precedence was wrong.** `IN`/`STARTS WITH`/`ENDS WITH`/
  `CONTAINS`/`IS NULL` bound tighter than arithmetic, so
  `n.val + 0 IS NULL` parsed as `n.val + (0 IS NULL)` instead of
  `(n.val + 0) IS NULL`. Fixed to match `openCypher.bnf`'s real precedence
  (these bind below the simple comparison operators, above arithmetic).

These fixes took TCK parse-acceptance from 90.5% to 99.4%; the remaining
0.6% are scenarios the TCK itself expects to fail (`Expected::AnyError`).

## Extensions beyond openCypher

Not upstream-worthy — no basis in `openCypher.bnf`. Present here because
MarsDB supports them:

- **`EXPLAIN <statement>`** — describe the plan without running it. Grammar
  only; `EXPLAIN` doesn't yet wrap `CALL`.
- **`CREATE INDEX ON :Label(prop) [UNIQUE]`** — the older single-property
  index syntax, not real Cypher's newer `CREATE INDEX FOR (n:Label) ON
  (n.prop)`.
- **`shortestPath((a)-[*..5]-(b))`** — single-path form only, no
  `allShortestPaths`. Real Neo4j Cypher syntax, but not part of
  `openCypher.bnf` or the TCK.
- **`;`-separated multi-statement batches** — `CREATE (a); CREATE (b);` as
  one textual submission (`parse_many`). Not a Cypher concept; needed for
  this crate's own multi-statement API.
- **`BEGIN` / `COMMIT` / `ROLLBACK`** — session-transaction statements
  (issue #142). Not in the grammar at all: recognized textually by
  `parse_antlr` before ANTLR runs (a whole statement that is exactly one
  of these keywords can never be valid Cypher otherwise), same
  out-of-grammar approach as the `;`-batch splitting above.

## Why ANTLR (and this specific fork)

Mainline ANTLR4 has no Rust code-generation target. This project uses the
actively maintained fork that adds one:

- Generator (Java tool): [`antlr4rust/antlr4`](https://github.com/antlr4rust/antlr4)
  `v0.5.0`, targeting ANTLR 4.13.3.
- Runtime (Rust crate): [`antlr4rust`](https://crates.io/crates/antlr4rust)
  — not the older, unmaintained `antlr-rust` crate.

## Regenerating

Only needed when `CypherLexer.g4`/`CypherParser.g4` change. Generated output
is committed to `marsdb-query/src/generated/` — this is not part of the
normal build.

```sh
# One-time: fetch the generator jar (pinned: v0.5.0 / ANTLR 4.13.3)
curl -sL -o /tmp/antlr4.jar \
  https://github.com/antlr4rust/antlr4/releases/download/v0.5.0/antlr4-4.13.3-SNAPSHOT-complete.jar

# Generate into a scratch dir, not straight into src/generated: the parser
# grammar's `tokenVocab` import looks for CypherLexer.tokens next to the
# source .g4 files, so generating directly into another directory fails
# with "cannot find tokens file".
rm -rf /tmp/antlr-gen && mkdir -p /tmp/antlr-gen
cp marsdb-query/grammar/*.g4 /tmp/antlr-gen/
(cd /tmp/antlr-gen && java -jar /tmp/antlr4.jar -Dlanguage=Rust -visitor \
  CypherLexer.g4 CypherParser.g4)

# Replace the generated files. This also deletes the hand-written
# antlr_accepts/antlr_debug_tree_text/mod declarations at the top of
# mod.rs (same glob) -- recover that part from git after copying.
rm -f marsdb-query/src/generated/*.rs marsdb-query/src/generated/*.tokens \
  marsdb-query/src/generated/*.interp
cp /tmp/antlr-gen/*.rs /tmp/antlr-gen/*.tokens /tmp/antlr-gen/*.interp \
  marsdb-query/src/generated/

# ANTLR's output formatting doesn't match rustfmt; CI's fmt check will
# fail on freshly generated output otherwise.
cargo fmt -p marsdb-query
```

Commit the resulting `marsdb-query/src/generated/*.rs` alongside the `.g4`
change. `marsdb-query/Cargo.toml` pins the matching runtime version
(`antlr4rust = "0.5.2"`).
