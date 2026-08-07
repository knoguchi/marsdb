# Row-slot refactor (mars-32d)

## Motivation

A read-path flamegraph (100x `recommendations/queries.cypher` against the
real 28,863-node dataset, see mars-32d/mars-m79) found 58.04% of total
time in `Vec<HashMap<String, Binding>>::from_iter` — collecting rows out
of a `BindingRow = HashMap<String, Binding>` stream. Bigger than every
other cost in that flame combined. `Expand` (`executor.rs:3189`) clones
the whole map per emitted neighbor; allocation scales `rows × hops ×
vars`.

Fix: resolve variable names to fixed slot indices at plan-build time
(the names are statically known — `VarNamer` guarantees every position
has one), rows become `Vec<Binding>`, a clone becomes a memcpy of small
values instead of a hash-map rebuild with N string allocations.

This is prep before writing code — the source that all evidence below
is drawn from is a full architecture survey (Explore agent, 2026-08-07,
`marsdb-query/src/{executor.rs,ir.rs,planner.rs}`). File:line citations
are from that survey; re-verify before implementing, code moves.

## The three questions

### 1. Where does slot resolution live?

`build_match_plan` (`planner.rs:36`) already computes the exact bound-name
set at every plan-node construction point: `VarNamer` (`planner.rs:12`)
synthesizes `__anon{N}` for every anonymous position, so nothing is
un-named; `pattern_bound_vars`/`prior_rel_vars`/`prior_edge_sets`
together give the live bound set at each hop. This is the natural place
to assign slots — no new name-tracking needed, it's already there.

**But `LogicalPlan` only covers ~10 node types** (`ir.rs:46-196`): scans,
`Expand`, `VarExpand`, `MatchRelList`, `Filter`, `IndexSeek`, `Seed`.
There is no `Sort`, `Aggregate`, `With`, `Union`, `Unwind`, `Distinct`,
`Limit`/`Skip` IR node — those clauses execute straight from the AST in
`execute_match`'s clause loop (`executor.rs:1382-1620`), a separate
architecture from the `LogicalPlan`-driven `stream_plan`/`count_stream`
that `Expand` lives in. A slot table can't live solely in `LogicalPlan`
and cover the whole statement; it can only cover the scan/expand/filter
portion.

**Decision: scope this refactor to the `LogicalPlan`-driven portion
only.** Assign slots in `build_match_plan` (inline, using the schema
state already computed there — no separate resolve pass needed, unlike
`apply_index_seeks`'s post-build rewrite which needs a live `Txn` this
doesn't). `stream_plan`'s terminal output converts back to the current
`HashMap<String, Binding>` shape at the boundary where `execute_match`'s
clause loop takes over. This bounds the diff to exactly where the 58%
lives, without touching WITH/aggregation/ORDER BY/mutations at all —
see "What doesn't change" below for why that's load-bearing, not just
convenient scoping.

### 2. What does `BindingRow` become?

`Vec<Binding>`, populated in slot order, for the `LogicalPlan`-driven
portion only (see above). `Binding` (`executor.rs:277-305`) has 6
variants; `Node`/`Edge`/`Value` are cheap to clone (they're small Copy-
ish types), but `List(Vec<Value>)`, `Map(BTreeMap<String, Value>)`, and
`Path(Vec<PathBinding>)` deep-clone regardless of the row's own
representation. **Switching the row's key representation does not fix
this** — it's an orthogonal cost. Worth noting for later (`Rc`-wrapping
those three variants), not blocking this refactor: none of the
`LogicalPlan`-driven operators (`Expand`, `VarExpand`, `Filter`, scans)
produce `List`/`Map` bindings today; only `VarExpand` produces `Path`
(unconditionally, into every output row, `ir.rs:126-136`) — so this
refactor's own hot path is unaffected, but don't claim the full clone
cost disappears in the PR description.

~18 sites clone a whole `BindingRow` today, not just `Expand`
(`executor.rs:3189`) — `Seed` (`3143`), `apply_with_or_carry`'s
`pre_with_rows` (`1847`), `eval_optional_part`'s three simultaneous
copies (`3059`/`3072`/`3073`/`3083`), and others. Only the ones inside
`stream_plan`/`count_stream` (`Expand`, `Seed`, `VarExpand`,
`MatchRelList`) are in scope here; the rest are in the clause loop, out
of scope by the boundary decision above, and stay `HashMap`-based
untouched.

### 3. What do name-lookback paths do?

This is where the "just replace HashMap with Vec" framing breaks, and
why the boundary in Q1 isn't just convenient scoping — it's necessary.

**Five call sites build a *fresh `LogicalPlan` per row*, keyed off that
row's runtime key set:** `eval_merge` (`executor.rs:1215`),
`execute_match_seeded` (`1365`), `eval_pattern_predicate_exists`
(`2981`), `Expr::Exists` (`3907`), `eval_pattern_comprehension` (`4546`).
Each does `row.keys().cloned().collect()` to build `carried_vars`, then
calls `build_match_plan(pattern, &None, &carried_vars)`. A pure
slot-indexed row has no `.keys()` — this needs either a side table
(slot index → name, cheap to share via `Rc` across rows of the same
shape since it's static per plan node, not per row) or these five sites
simply never see a slot-based row, because they all sit in AST-driven
code (MERGE, EXISTS, pattern comprehension) that's on the clause-loop
side of the Q1 boundary, not inside `stream_plan`. **With the Q1
boundary as decided, these five are unaffected — they keep operating on
the `HashMap` row exactly as today.** This is the concrete reason the
boundary has to be where it is, not a looser "convert everything
eventually" plan.

**`eval_projected_expr` (`executor.rs:9693`) — and everything downstream
of a projection — operates on a *different* row type entirely:**
`HashMap<String, Value>`, not `BindingRow`. Aggregation-finish code
(`rewrite_composed_item`, `executor.rs:2726`) even synthesizes
`__slot{N}`-named keys into that map at runtime
(`format!("__slot{}", subst.len())`) — names invented after the fact,
which by construction can't correspond to any plan-time slot. `RETURN`/
`WITH` projection, all three `ORDER BY` implementations
(`apply_order_by`/`apply_order_by_with_scope`/`apply_order_by_bindings`),
and `resolve_grouped_rows`' grouping-key lookup all resolve names against
this separate map or against the clause-loop's `HashMap<String,
Binding>` row — none of them see the `LogicalPlan`-driven row directly.
**This confirms the Q1 boundary again: the slot refactor's blast radius
stops at the `stream_plan`/clause-loop handoff. WITH, ORDER BY,
aggregation, RETURN projection are a separate, later architecture
question (possibly `mars-9or`-adjacent), not part of this PR.**

`RETURN *`/`WITH *` (`executor.rs:1694`/`1805`) are safe — they resolve
against `carried_vars`, which is tracked statically through the clause
loop already (not derived from a live row's keys), so they're unaffected
either way.

## What doesn't change (out of scope for this PR)

- WITH, UNWIND, aggregation (`resolve_grouped_rows`), all three ORDER BY
  implementations, RETURN/WITH projection (`eval_projected_expr`'s
  `HashMap<String, Value>` layer) — architecturally separate from
  `LogicalPlan`, stay exactly as they are.
- MERGE, `EXISTS { ... }`, pattern comprehension — the 5 per-row
  `build_match_plan` call sites, unaffected by construction (Q3).
- `Binding::List`/`Map`/`Path` deep-clone cost — real, not fixed by this
  refactor, worth its own follow-up.
- `mars-9or` (statement-scoped `WriteCtx`) — explicitly not coupled to
  this PR per review guidance on #157; rides its own future executor
  pass.
- `explain.rs` maintains its own parallel `carried_vars` threading and
  calls `build_match_plan` directly (`explain.rs:202`); it prints var
  names from plan nodes (`format_plan`, `explain.rs:380`), so plan nodes
  must keep carrying `String` names for EXPLAIN output even after
  execution switches to slots internally — the slot table is additive,
  not a replacement for the names already on `LogicalPlan` nodes.

## Expected outcome

`Expand`'s clone becomes a `Vec<Binding>` memcpy instead of a `HashMap`
rebuild with per-row string allocations — the direct fix for the 58%
finding. Re-flame reads after landing: expected residual is the ~40%
node-decode-cache leftover (single-decode cost, a v2 record-format
question — mars-a5a) and whatever's left of clause-loop row handling
(WITH/aggregation/ORDER BY, explicitly out of scope here). If that's
right, every remaining read-path tower is a format/representation
decision, same conclusion the write-path floor (mars-3va, 16.4x) already
reached — the point where the v2 design doc's motivation section is
complete.
