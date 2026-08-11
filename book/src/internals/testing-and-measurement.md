# Testing and Measurement

A database earns trust two ways: by being unable to give wrong answers,
and by understanding its own performance. This chapter covers both
mechanisms —
the correctness stack (unit suites, conformance, crash safety,
integrity) and the measurement discipline that the rest of this book
has been quietly citing.

## The correctness stack

**Unit and end-to-end suites** live where the code lives: storage and
graph invariants in `marsdb-graph`'s tests, query shape and mechanics
in `marsdb-query`'s, and full workloads at the `marsdb` crate level —
including a suite that runs the LDBC Social Network Benchmark's
short-read queries via real parameter substitution against a shared
fixture, asserting *exact* results, not "doesn't panic." Stress tests
(50k-node chains, 10k-fanout supernodes with detach-delete, 20k random
operations checked against an in-memory oracle) are `#[ignore]`d by
default and run explicitly.

**Conformance** is the openCypher TCK, run by `marsdb-tck`: a harness
that parses the official Gherkin feature files (vendored as a git
submodule), builds each scenario's initial graph, runs the scenario's
query, and compares against the expected table with TCK value
semantics. Every scenario lands in one of five outcomes — pass, wrong
result, unexpected behavior, parse-rejected, or
runner-unsupported — because "how it fails" is more informative than
a pass rate alone: a wrong *result* is a correctness bug; a clean
parse rejection of an unimplemented feature is a scope decision.
MarsDB currently passes 3,880 of 3,880 scenarios, and the coverage
table (`CYPHER_COVERAGE.md`) is generated from real runs, not
maintained by hand.

The TCK's deepest value showed up in this book repeatedly without
being named: several of the subtlest behaviors in the executor —
`type(r)` surviving `DELETE r`, edge isomorphism reaching across a
pattern into a variable-length hop's BFS, `MERGE` accepting a
bound-variable property — were found *by scenarios*, and the code
comments cite the exact ones. A conformance suite is a
machine-checkable spec, and its scenarios reach corners that
first-principles test-writing does not.

**Crash safety** has its own harness (`marsdb-crash-harness`), and its
scope is stated in its module docs: this is *level-1* crash
safety — process death while the OS survives and the page cache remains
intact — and explicitly not power loss (which loses the page cache and
would need fault injection to test). Within that scope, a
child process commits a random number of single-`CREATE` transactions
with a monotonically numbered counter; the parent SIGKILLs it at an
unpredictable moment, reopens the file cold, and asserts a purely
*structural* invariant — the surviving counter values must be exactly
a contiguous prefix `{1..K}`. No gaps (no half-applied transaction),
no duplicates (no doubly-recorded commit). The parent never
synchronizes with the child's commit progress, because the invariant
holds regardless of where the kill lands — an assertion designed so
that racing is not a bug in the test.

**The integrity checker** (chapter 4) doubles as the final oracle:
stress and crash tests can end with a full logical validation of every
cross-table invariant, converting "the test passed" into "and the file
is coherent."

## The measurement discipline

`BENCHMARKS.md` is the repository's ledger of numbers, and its rules
are as much a part of the engineering culture as any code:

- **Only measured numbers.** Nothing is estimated, extrapolated, or
  "expected to be fast." A feature without a benchmark has no
  performance claim — the file has no placeholder rows.
- **Provenance on every number.** Hardware, date, build profile, and
  the exact reproduction command (`cargo bench -p ...`, using Criterion)
  accompany each table.
- **Caveats are stated, not buried.** The concurrent-read scaling
  table (1.26x at 2 threads up to 1.87x at 8, then a plateau) is
  followed by two known unisolated factors that may cap it, and the
  conclusion is scoped to what the data supports: concurrency
  reliably beats sequential, which is what the feature is *for*. An
  explicit "scope of these numbers" section lists what has *not* been
  measured — file-backed fsync-pressure throughput, unbenchmarked
  operators — so absence of evidence is visible instead of silent.
- **End-to-end checks bracket the micro-benchmarks.** A real dataset
  (28,863 nodes, 166,261 relationships, loaded from plain Cypher) is
  the load/query/update/delete lifecycle gate, re-run as internals
  change; micro-benchmarks alone can miss regressions that only
  compose at scale.

The payoff of running real workloads is not just numbers — it is bug
discovery. The lifecycle benchmark directly surfaced two planner
defects (an `IndexSeek` that never fired for row- and
parameter-bound equalities, and a multi-hop pattern's start-node
`WHERE` never reaching the scan it should narrow), both fixed because
a measurement looked wrong. A benchmark suite that only confirms
expectations is underused.

## Continuous integration

CI runs the test suite on Linux, macOS, and Windows; formatting and
clippy as a gate; coverage collection; a dependency security audit;
and a bindings job that builds the Python extension and runs its
tests against the workspace — plus a build of the C ABI with the
Arrow feature, which is the pre-merge check protecting the
out-of-repo Go binding (its own CI builds this repository's C ABI
from `main` and would otherwise discover breakage only after merge).

The final chapter collects the measurements that *changed decisions*
— including those that led to a feature's removal.
