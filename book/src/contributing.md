# Contributing

## Before opening a PR

CI (`.github/workflows/rust.yml`) runs on every push/PR to `main`:

- **Format + Clippy**: `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` — zero
  warnings tolerated.
- **Tests**: `cargo test --workspace --verbose` on Linux, macOS, and
  Windows.
- **Bindings**: builds and tests the Python (`maturin` + `unittest`) and
  Go (`go test`) bindings against the same Rust workspace build.

Run the same checks locally before pushing:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Additional local checks

```
cargo test -p marsdb-graph --test stress -- --ignored --nocapture  # ~15s, large-scale
cargo test -p marsdb-crash-harness -- --ignored --nocapture        # ~7s/30 runs, SIGKILL-and-verify
cargo bench -p marsdb-graph
cargo bench -p marsdb
```

`marsdb-crash-harness` is a process-crash durability check (not a full
power-loss test — the OS stays up, page cache intact): it spawns a child
process committing one transaction at a time, `SIGKILL`s it at an
unpredictable point, reopens the file fresh, and asserts every
acknowledged commit survived intact with no gaps or duplicates.

The Cypher parser (the one part of MarsDB that takes raw, untrusted
string input directly) is fuzzed via `cargo-fuzz` — needs nightly:

```
cargo install cargo-fuzz
cd marsdb-query && cargo +nightly fuzz run parse -- -max_total_time=120
```

Only claim: never panics. A parse error (`Result::Err`) is the expected,
correct outcome for most fuzzer-generated input.

## openCypher TCK conformance

Changes that touch `marsdb-query` should be checked against the real
openCypher TCK, not just the crate's own unit/smoke tests — see
[Cypher Language Support](./cypher-support.md) for what this measures and
why:

```
git submodule update --init marsdb-tck/openCypher
cargo run --release -p marsdb-tck
```

For fast iteration, restrict to a category/feature:

```
TCK_FILTER="clauses/create" cargo run -p marsdb-tck --release
```

If a change moves the conformance numbers, update the table in
[`CYPHER_COVERAGE.md`](https://github.com/knoguchi/marsdb/blob/main/CYPHER_COVERAGE.md)
(exact row numbers + the `TOTAL` row) in the same PR — it's the ground
truth this book's own [Cypher Language Support](./cypher-support.md)
page is condensed from.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/knoguchi/marsdb/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/knoguchi/marsdb/blob/main/LICENSE-MIT)
at your option. By contributing, you agree your contribution is licensed
under the same terms.
