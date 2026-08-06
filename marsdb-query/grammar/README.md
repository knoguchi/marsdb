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

This fix alone raised the TCK parse-acceptance comparison below from
90.5% to 95.1% and eliminated the entire `Temporal1/2/10` disagreement
category (149 cases) — those weren't a real grammar gap, just corrupted
date-string literals (`'2018-01-01T12:00'` contains `-`/`:`).

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
