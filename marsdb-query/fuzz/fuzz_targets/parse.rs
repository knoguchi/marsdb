#![no_main]

use libfuzzer_sys::fuzz_target;

// The only thing this asserts is "never panics" -- a Result::Err (a clean
// parse error) is the expected, correct outcome for most fuzzer-generated
// input. `parse` and `parse_many` are the two entry points that take a
// raw, untrusted Cypher string directly (everything else in this crate's
// public API starts from an already-parsed Statement).
fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return };
    let _ = marsdb_query::parse(s);
    let _ = marsdb_query::parse_many(s);
});
