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

# Regenerate (requires a JDK; e.g. `brew install openjdk`)
java -jar /tmp/antlr4.jar -Dlanguage=Rust -visitor \
  -o marsdb-query/src/generated \
  marsdb-query/grammar/CypherLexer.g4 marsdb-query/grammar/CypherParser.g4

# ANTLR's own formatting doesn't match rustfmt -- CI's fmt --check job
# will fail on freshly generated output otherwise.
cargo fmt -p marsdb-query
```

Commit the resulting `marsdb-query/src/generated/*.rs` alongside the `.g4`
change. `marsdb-query/Cargo.toml` pins the matching runtime:
`antlr4rust = "0.5.2"`.
