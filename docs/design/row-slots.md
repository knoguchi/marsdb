# Row-slot executor refactor

Query execution's read path passes rows between operators (`Expand`,
`VarExpand`, `Filter`, scans) as `HashMap<String, Binding>`. Profiling a
multi-hop `MATCH` workload found most read-path time going into
rebuilding that map per emitted row during `Expand` — a hash-map
allocation plus per-variable string allocations, once per row per hop.

Proposed fix: resolve variable names to fixed slot indices at plan-build
time (the bound-variable set is already known statically once the
logical plan is built), so rows become `Vec<Binding>`. A row clone
becomes a memcpy instead of a hash-map rebuild.

Scope: only the `LogicalPlan`-driven portion of execution (scans,
`Expand`, `VarExpand`, `Filter`, index seeks) has a plan-time slot table
available. `WITH`, aggregation, `ORDER BY`, and `RETURN`/`WITH`
projection run from a separate, AST-driven path with no equivalent plan
node, so they'd stay on `HashMap` rows; the slot-converted portion would
convert back to `HashMap` once at that boundary.

Expected effect: real but partial. The direct `Expand` clone cost drops,
but a query whose rows cross into `WITH`/aggregation still pays one
terminal `HashMap` build, so the win is bounded by how much of a
query's row-handling time is spent inside the `LogicalPlan`-driven
portion versus after it.

Status: not implemented. `BindingRow` is still `HashMap<String,
Binding>`.
