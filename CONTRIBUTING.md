# Contributing to MarsDB

Thanks for considering a contribution. This file is the quick reference;
the full version with more context lives in the manual's [Contributing
chapter](https://knoguchi.github.io/marsdb/contributing.html).

## Before opening a PR

`main` is protected — every change lands via a pull request, and CI must
pass:

- **Format + Clippy**: `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` — zero
  warnings tolerated.
- **Tests**: `cargo test --workspace` on Linux, macOS, and Windows.
- **Bindings**: Python (`maturin` + `unittest`) and Go (`go test`), built
  against the same Rust workspace.
- **`marsdb-python`** is workspace-excluded (see the comment on
  `Cargo.toml`'s own `exclude`) — run
  `cargo check --manifest-path marsdb-python/Cargo.toml` after touching
  any shared type (`PropertyValue`, `Value`, ...); it catches type
  errors that nothing else local will.

Run the same checks locally before pushing:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --manifest-path marsdb-python/Cargo.toml
```

## openCypher TCK conformance

Changes touching `marsdb-query` should be checked against the real
openCypher TCK, not just unit/smoke tests:

```
git submodule update --init marsdb-tck/openCypher
cargo run --release -p marsdb-tck
```

If a change moves the conformance numbers, update the table in
[`CYPHER_COVERAGE.md`](CYPHER_COVERAGE.md) (exact row numbers + the
`TOTAL` row) in the same PR.

## Commit style

Small, focused PRs — one fix or feature per PR, not a bundle. Commit
messages explain *why*, not just what changed (the diff already shows
what). See recent history for the convention this repo follows.

## Reporting bugs / requesting features

Open an issue — templates will guide you through the relevant details.
For a security vulnerability, see [`SECURITY.md`](SECURITY.md) instead
of a public issue.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option. By contributing, you agree
your contribution is licensed under the same terms.
