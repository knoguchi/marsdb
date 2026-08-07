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
`marsdb-query/src/{executor.rs,ir.rs,planner.rs}`), spot-checked against
`main` at `9280ee2` (2026-08-07). File:line citations are from that
survey at that commit; re-verify before implementing, code moves.

## The four questions

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
simply never see a slot-based row.

For `eval_merge`/`execute_match_seeded`/`eval_pattern_comprehension` (the
statement-level forms), that holds cleanly: they sit in AST-driven code
on the clause-loop side of the Q1 boundary, not inside `stream_plan`,
unaffected. `eval_pattern_predicate_exists`/`Expr::Exists` are a
narrower case than this paragraph originally implied, though — see Q4,
which found they're also reachable as `Expr` variants *embedded inside*
a slot-converted `Filter`'s predicate, not just as their own statement
forms. The side table isn't optional there; Q4 covers the resolution.

**`eval_projected_expr` (`executor.rs:9693`) — and everything downstream
of a projection — operates on a *different* row type entirely:**
`HashMap<String, Value>`, not `BindingRow`. Aggregation-finish code
(`rewrite_composed_item`, `executor.rs:2726`) even synthesizes
`__slot{N}`-named keys into that map at runtime
(`format!("__slot{}", subst.len())`) — a naming collision with this
refactor's own "slot" terminology, worth renaming (e.g. `__agg{N}`) in
passing so nobody confuses a runtime-synthesized `HashMap` key with a
real plan-time slot index while reading the diff. Names invented after
the fact, which by construction can't correspond to any plan-time slot
regardless of what they're called. `RETURN`/
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

### 4. How do predicates read a slot row?

Not answered above, and it's the choice that moves the diff size more
than any other open question — deciding it now, not leaving it for
mid-implementation.

Two expression types read rows inside the `LogicalPlan`-driven portion:
`Filter`'s predicate (`Expr`, `ir.rs:192`) and `IndexSeekValue::RowExpr`'s
per-seed-row value (`ReturnExpr`, evaluated at `executor.rs:3548`, e.g.
`row.field` from an `UNWIND`-bound join). Both resolve variables by name
today: `eval_expr` (`executor.rs:3807`) does `row.get(var)` directly for
`HasLabel`/`VarEq`, and `self.lookup_prop(txn, pa, row)` (itself
`row.get(&pa.var)`) for `Compare`/`PropCompare`/`IsNull`.

`Expr` (`ast.rs:37-147`) is **not** the small, uniformly-simple set an
earlier draft of this note claimed — verified directly against `ast.rs`
after a review caught the error before it shipped. 14 variants, three
different shapes:

- **8 simple leaves** — `And`/`Or`/`Not`/`Compare`/`PropCompare`/
  `IsNull`/`HasLabel`/`VarEq`. Every var reference is a plain `String`
  field (`PropAccess.var`, or the variant's own `String` args). Fully
  enumerable, bounded, safe to compile.
- **3 wrap a `ReturnExpr`** — `GeneralCompare`/`GeneralIsNull`/
  `GeneralBare`. Same "full expression grammar" problem as
  `IndexSeekValue::RowExpr` below, not simple.
- **3 are their own dynamic-plan-building cases** — `Pattern`,
  `Exists { pattern, where_clause }` (`executor.rs:3903-3910`, the
  `row.keys()` site Q3 already cites at `3907`), `ExistsSubquery`. These
  are exactly Q3's five `row.keys()`-dependent sites, reached *as `Expr`
  variants*, not just as top-level statement forms.

**This matters because `push_conjuncts` doesn't filter by variant.**
`build_match_plan` (`planner.rs:68-86`) splits a `WHERE` clause into
top-level `AND` conjuncts and pushes whichever ones it can (start-var-only
ones directly, the rest into the pattern's trailing `Filter`) — nothing
here restricts *which* `Expr` variants can end up inside a
`LogicalPlan::Filter`'s predicate. A `WHERE n.age > 30 AND EXISTS { ... }`
conjunct can genuinely land inside a `Filter` node that's otherwise deep
in the slot-converted portion. Q3's "these five sites are unaffected,
they stay on the `HashMap` row" is true for the *statement-level* forms
(MERGE's own match, a correlated subquery's own seed) but incomplete for
`Expr::Pattern`/`Exists`/`ExistsSubquery` specifically, which can be
*embedded inside* a slot-converted `Filter`'s predicate tree.

Two designs for the simple/compilable cases:
- **(a) Side table.** `Rc<HashMap<String, usize>>` per plan node
  (name → slot index), consulted at eval time — `row.get(var)` becomes
  `row[*names.get(var)?]`. Small diff, keeps the allocation win, not the
  hashing win.
- **(b) Compile to slots at plan-build time.** Walk the expression in
  `build_match_plan` (the same pass Q1 already puts there) and rewrite
  each var reference to its resolved slot index once, up front. Bigger
  diff, full win: zero string hashing per row at eval time.

**Decision, per shape:**
- **8 simple leaves: (b).** Bounded rewrite, and `Filter` sits directly
  in the hot `Expand`→`Filter` chain — worth the full win.
- **`GeneralCompare`/`GeneralIsNull`/`GeneralBare` and
  `IndexSeekValue::RowExpr`'s `ReturnExpr`: (a), as a fallback, not a
  final answer.** Compiling the full `ReturnExpr` grammar to slots is
  close to its own separate refactor (the same recursive evaluator
  `eval_return_expr` uses throughout the clause-loop side too), and
  `RowExpr` is a narrower, more recent addition (row/param-bound
  `IndexSeek`, #141) than the general case. Measure how much cost is
  actually here after (b) lands for the simple leaves — only compile
  this tail if the measurement says it matters.
- **`Pattern`/`Exists`/`ExistsSubquery`: neither — evaluate against a
  `HashMap` reconstructed from the slot row via the same side table (a)
  would use, right at that leaf, and nowhere else.** Don't try to make
  `row.keys()` work on a slot row (no clean way to reconstruct "what's
  bound right now" without carrying the name table anyway, so this pays
  the same cost as (a) with less code). This is why the slot/name side
  table needs to exist regardless of the (a)-vs-(b) choice for the other
  two cases above — it's not just a fallback path, it's required for
  compiling `Expr` to slots *at all*, since a compiled `Filter` predicate
  can still contain an uncompiled `Pattern`/`Exists`/`ExistsSubquery`
  leaf nested inside it.

## What doesn't change (out of scope for this PR)

- WITH, UNWIND, aggregation (`resolve_grouped_rows`), all three ORDER BY
  implementations, RETURN/WITH projection (`eval_projected_expr`'s
  `HashMap<String, Value>` layer) — architecturally separate from
  `LogicalPlan`, stay exactly as they are.
- MERGE, correlated-subquery seeding, pattern comprehension (the
  statement-level forms of the 5 per-row `build_match_plan` call sites)
  — unaffected by construction (Q3). `EXISTS {...}`/pattern-predicate/
  `ExistsSubquery` *as embedded `Filter`-predicate leaves* need the
  slot/name side table at that leaf specifically — see Q4, not
  "unaffected."
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

## The `stream_plan`/clause-loop boundary is two-sided, and not free

The Q1 boundary decision means every row crossing from `stream_plan`
into the clause loop, and every row entering `stream_plan` from the
clause loop, pays a real conversion cost — worth stating explicitly so
neither side is discovered mid-implementation, and so the expected
outcome below doesn't overclaim.

- **Output (slot → `HashMap`).** Every row surviving to the handoff gets
  rebuilt as a `HashMap`, once, via the slot/name side table. What this
  refactor eliminates is the *per-hop* rebuild inside `stream_plan` (`h`
  clones per surviving row becomes `h` memcpys plus one terminal
  `HashMap` build, not zero `HashMap` builds) — not the terminal build
  itself. For the flamegraph's own queries (multi-hop `Expand` feeding
  straight into aggregation, e.g. `crimson_tide_collaborative_filtering`),
  *every* expanded row crosses this boundary, since aggregation is
  clause-loop side (Q1/Q3). The win there is real but partial: fewer,
  cheaper intermediate clones, not zero `HashMap` cost on the query's
  critical path.
- **Input (`HashMap` → slot).** `Seed` (`ir.rs:75`, `executor.rs:3143`)
  receives clause-loop rows (from a prior `WITH`, or a correlated
  subquery's seed row) and feeds them into the slot-converted portion —
  the reverse conversion, name → slot per seed row. Cheap (one lookup
  per bound var per seed row, not per emitted row), but real, and easy
  to miss if the boundary is only thought of as one-directional.

## Expected outcome

`Expand`'s clone becomes a `Vec<Binding>` memcpy instead of a `HashMap`
rebuild with per-row string allocations — the direct fix for the 58%
finding, but (per above) a fraction of that 58%, not all of it, once the
boundary's own terminal `HashMap` build is accounted for. An earlier
verbal estimate of "5-20x on multi-hop MATCH" overstated this for
aggregate-heavy queries specifically, where every expanded row still
crosses the boundary once — correcting that here so the post-land flame
doesn't read as a miss against a number this note never actually
committed to.

Re-flame reads after landing: expected residual is the ~40%
node-decode-cache leftover (single-decode cost, a v2 record-format
question — mars-a5a) and whatever's left of clause-loop row handling
(WITH/aggregation/ORDER BY, explicitly out of scope here) plus the
boundary's own terminal `HashMap` build on aggregate-heavy queries. If
that's right, every remaining read-path tower is a format/representation
decision, same conclusion the write-path floor (mars-3va, 16.4x) already
reached — the point where the v2 design doc's motivation section is
complete.
