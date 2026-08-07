## Summary

<!-- What does this change, and why? -->

## Test plan

<!-- What did you run to verify this? cargo test output, a TCK filter,
     a manual repro that now works, etc. -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] If this touches `marsdb-query`: full unfiltered `cargo run -p marsdb-tck --release`, and `CYPHER_COVERAGE.md` updated if the numbers moved
- [ ] If this touches a shared type (`PropertyValue`, `Value`, ...): `cargo check --manifest-path marsdb-python/Cargo.toml`
