use std::collections::{BTreeMap, HashMap, HashSet};

use marsdb_graph::{AdjEntry, Direction, EdgeId, GraphStore, NodeId, PropertyValue, Txn, WriteTransaction};

use crate::aggregate::{property_value_hash_key, value_hash_key, AggAcc, HashKey};
use crate::ast::{
    is_aggregate_name, CompareOp, Expr, Literal, MergeClause, NodePattern, Pattern, PropAccess, QueryClause,
    RelDirection, ReturnExpr, ReturnItem, SortDir, Statement, Tail, UnwindClause, UnwindSource, WithClause, WithExpr,
};
use crate::error::QueryError;
use crate::ir::{ExpandDirection, LogicalPlan};
use crate::planner::{build_match_plan, pattern_all_vars, pattern_new_vars};
use crate::result::QueryResult;
use crate::value::Value;

/// Hidden key used to correlate `OPTIONAL MATCH` results back to the outer
/// row that seeded them — never visible to user Cypher (not a valid
/// identifier prefix a parsed pattern could ever produce).
const OPTIONAL_SEED_IDX_KEY: &str = "__seed_idx";

/// Hidden key tagging whether a `MERGE`d row came from the create-path or
/// the match-path, consumed (and stripped) by `apply_merge_set` before the
/// row becomes visible to the rest of the query.
const MERGE_CREATED_KEY: &str = "__merge_created";

#[derive(Debug, Clone)]
enum Binding {
    Node(NodeId),
    Edge(EdgeId),
    /// A scalar carried through a `WITH` projection (e.g. `WITH message.id
    /// AS messageId`) — no graph identity, just a value along for the ride
    /// to the next `QueryPart`/the final `Tail`.
    Value(PropertyValue),
    /// A `collect()` result carried through a `WITH` projection. Separate
    /// from `Binding::Value` because `PropertyValue` (storage-layer) has no
    /// list variant — lists are a query-layer-only concept, never
    /// persisted — so a materialized `collect()` has nowhere else to live
    /// between one `QueryPart` and the next. Elements are already-resolved
    /// `Value`s, not `Binding`s: there's no `UNWIND` yet to pull one back
    /// out with restored graph identity.
    List(Vec<Value>),
}

type BindingRow = HashMap<String, Binding>;

/// Safety cap on unbounded variable-length traversal (`[:TYPE*0..]`) depth.
/// Hitting it errors rather than silently truncating — see `VarExpand`
/// evaluation. Node-visited-set BFS (not relationship-uniqueness) is used
/// throughout, which is only correct because the graphs this targets
/// (LDBC's REPLY_OF-style reply chains) form a forest, not a general
/// cyclic graph — not safe to reuse as-is for a variable-length pattern
/// over a cyclic relationship type without revisiting that assumption.
const VAR_EXPAND_DEPTH_CAP: u32 = 30;

pub struct Executor<'a> {
    store: &'a GraphStore,
}

impl<'a> Executor<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        Self { store }
    }

    /// Dispatches on whether `stmt` ever mutates anything. A read-only
    /// statement (`MATCH ... RETURN`, `is_read_only` below) runs inside a
    /// `ReadTransaction` — a consistent snapshot that doesn't contend for
    /// redb's single-writer lock, so concurrent readers run in parallel
    /// instead of queueing behind each other. Everything else runs inside
    /// a `WriteTransaction`, committed or aborted as a whole — the
    /// crash-safety boundary from the plan (one statement = one commit).
    /// Every graph access below this point must go through the `*_in_txn`
    /// GraphStore methods, never the standalone `self.store.*` methods,
    /// which open (and would deadlock trying to re-open) their own
    /// transaction.
    pub fn execute(&self, stmt: &Statement) -> Result<QueryResult, QueryError> {
        if is_read_only(stmt) {
            let read_txn = self.store.begin_read()?;
            let Statement::Match {
                clauses,
                tail,
                order_by,
                limit,
            } = stmt
            else {
                unreachable!("is_read_only only returns true for Statement::Match")
            };
            // No explicit commit/abort — a ReadTransaction is a pure
            // snapshot view with nothing to roll back; it releases on drop.
            return self.execute_match(Txn::Read(&read_txn), clauses, tail, order_by, *limit);
        }
        let write_txn = self.store.begin_write()?;
        let outcome = match stmt {
            Statement::Create(patterns) => self.execute_create(&write_txn, patterns),
            Statement::Match {
                clauses,
                tail,
                order_by,
                limit,
            } => self.execute_match(Txn::Write(&write_txn), clauses, tail, order_by, *limit),
        };
        match outcome {
            Ok(result) => {
                GraphStore::commit(write_txn)?;
                Ok(result)
            }
            Err(e) => {
                // Best-effort rollback; the original error is what matters.
                let _ = GraphStore::abort(write_txn);
                Err(e)
            }
        }
    }

    fn execute_create(&self, write_txn: &WriteTransaction, patterns: &[Pattern]) -> Result<QueryResult, QueryError> {
        // A standalone CREATE is a MATCH...CREATE tail run against a
        // single empty row -- `resolve_or_create_node` below never finds
        // any variable already bound in an empty `BindingRow`, so every
        // node token is fresh, exactly like standalone CREATE always was.
        self.materialize_create(write_txn, patterns, &[BindingRow::new()])
    }

    /// Runs CREATE patterns once per row in `rows`. Shared by a
    /// standalone `CREATE` statement (`execute_create`, a single empty
    /// row) and a `MATCH ... CREATE` tail (`execute_match`, rows carry
    /// bindings from the preceding MATCH/WITH). The only real difference
    /// between the two is what `resolve_or_create_node` finds already
    /// bound in a row -- nothing for standalone CREATE, real nodes for a
    /// MATCH...CREATE tail, which is what lets the tail form add an edge
    /// between two nodes that already exist.
    fn materialize_create(
        &self,
        write_txn: &WriteTransaction,
        patterns: &[Pattern],
        rows: &[BindingRow],
    ) -> Result<QueryResult, QueryError> {
        for row in rows {
            for pattern in patterns {
                let mut prev_id = self.resolve_or_create_node(write_txn, &pattern.start, row)?;
                for (rel, node) in &pattern.hops {
                    if rel.hop_range.is_some() {
                        return Err(QueryError::Parse(
                            "CREATE doesn't support variable-length relationship patterns (e.g. [:TYPE*1..3])".into(),
                        ));
                    }
                    let node_id = self.resolve_or_create_node(write_txn, node, row)?;

                    let rel_label = rel.rel_type.clone().unwrap_or_else(|| "REL".to_string());
                    let rel_props = literal_props_to_values(&rel.props);
                    let (src, dst) = match rel.direction {
                        RelDirection::Right => (prev_id, node_id),
                        RelDirection::Left => (node_id, prev_id),
                        RelDirection::Either => {
                            return Err(QueryError::Parse(
                                "CREATE requires a directed relationship (-> or <-), not an undirected pattern".into(),
                            ))
                        }
                    };
                    GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
                    prev_id = node_id;
                }
            }
        }
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }

    /// A node pattern token reuses an existing binding iff it names a
    /// variable already bound in `row` (from a preceding MATCH/WITH) --
    /// restating labels/props on that token is rejected with a clear
    /// error rather than silently ignored, since silently dropping
    /// user-written labels/props would be a correctness trap. Anything
    /// else (no variable, or a variable not yet bound in this row)
    /// creates a brand-new node, exactly like standalone CREATE always
    /// has for every node token.
    fn resolve_or_create_node(
        &self,
        write_txn: &WriteTransaction,
        node: &NodePattern,
        row: &BindingRow,
    ) -> Result<NodeId, QueryError> {
        if let Some(var) = &node.var {
            if let Some(binding) = row.get(var) {
                let Binding::Node(id) = binding else {
                    return Err(QueryError::Parse(format!(
                        "'{var}' is not a node — can't use it as a CREATE pattern endpoint"
                    )));
                };
                if !node.labels.is_empty() || !node.props.is_empty() {
                    return Err(QueryError::Parse(format!(
                        "'{var}' is already bound — CREATE can't add labels/properties to an existing node"
                    )));
                }
                return Ok(*id);
            }
        }
        let labels = pattern_labels(&node.labels);
        let props = literal_props_to_values(&node.props);
        Ok(GraphStore::create_node_in_txn(write_txn, &labels, props)?)
    }

    /// Runs `MERGE` once per row in `rows` (`clause.pattern.hops.len() <=
    /// 1`, enforced at parse time — whole-pattern atomicity across
    /// multiple simultaneously-unbound hops isn't attempted in v1: which
    /// hop's "not found" should trigger creation of what, in what order,
    /// gets genuinely hard to reason about correctly for longer chains).
    fn eval_merge(
        &self,
        write_txn: &WriteTransaction,
        clause: &MergeClause,
        rows: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in rows {
            out.extend(self.merge_one_row(write_txn, clause, row)?);
        }
        self.apply_merge_set(write_txn, clause, &mut out)?;
        Ok(out)
    }

    fn merge_one_row(
        &self,
        write_txn: &WriteTransaction,
        clause: &MergeClause,
        row: &BindingRow,
    ) -> Result<Vec<BindingRow>, QueryError> {
        // Validate every token before doing any graph work (search or
        // create) — an unconstrained node pattern that isn't already bound
        // would otherwise let the search below silently "match" every
        // node in the graph (AllNodesScan, no Filter), which is a
        // wrong-answer footgun, not a helpful default.
        require_mergeable(&clause.pattern.start, row)?;
        for (rel, node) in &clause.pattern.hops {
            if rel.hop_range.is_some() {
                return Err(QueryError::Parse(
                    "MERGE doesn't support variable-length relationship patterns (e.g. [:TYPE*1..3])".into(),
                ));
            }
            require_mergeable(node, row)?;
        }

        // Try the pattern as an ordinary MATCH first. Whatever's already
        // bound in `row` (e.g. `a` from a preceding MATCH) becomes a Seed,
        // not a fresh scan — build_match_plan already knows how to do
        // this, the same mechanism every ordinary MATCH clause uses. For a
        // one-hop pattern this already searches the *connected*
        // sub-pattern (Expand from the resolved source, Filter by the
        // target's own constraints), not each node independently — which
        // is exactly the correctness property MERGE needs and gets for
        // free by reusing this instead of inventing bespoke search logic.
        let carried_vars: HashSet<String> = row.keys().cloned().collect();
        let plan = build_match_plan(&clause.pattern, &None, &carried_vars)?;
        let found = self.eval_plan(Txn::Write(write_txn), &plan, std::slice::from_ref(row))?;
        if !found.is_empty() {
            return Ok(found.into_iter().map(|r| tag_merge_created(r, false)).collect());
        }

        // Nothing found — create exactly one new instance. Reuses
        // resolve_or_create_node, the same "reuse if the token's var is
        // already bound in the row, else create fresh" logic
        // Tail::Create/materialize_create already use.
        let mut new_row = row.clone();
        let start_id = self.resolve_or_create_node(write_txn, &clause.pattern.start, &new_row)?;
        if let Some(var) = &clause.pattern.start.var {
            new_row.insert(var.clone(), Binding::Node(start_id));
        }
        // At most one hop (enforced at parse time) -- a plain `if let`,
        // not a loop, so there's no dangling "previous node" state to
        // thread once a 2nd+ hop is ever supported.
        if let Some((rel, node)) = clause.pattern.hops.first() {
            let node_id = self.resolve_or_create_node(write_txn, node, &new_row)?;
            if let Some(var) = &node.var {
                new_row.insert(var.clone(), Binding::Node(node_id));
            }
            let rel_label = rel.rel_type.clone().unwrap_or_else(|| "REL".to_string());
            let rel_props = literal_props_to_values(&rel.props);
            let (src, dst) = match rel.direction {
                RelDirection::Right => (start_id, node_id),
                RelDirection::Left => (node_id, start_id),
                RelDirection::Either => {
                    return Err(QueryError::Parse(
                        "MERGE requires a directed relationship (-> or <-), not an undirected pattern".into(),
                    ))
                }
            };
            let edge_id = GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
            if let Some(var) = &rel.var {
                new_row.insert(var.clone(), Binding::Edge(edge_id));
            }
        }
        Ok(vec![tag_merge_created(new_row, true)])
    }

    /// Applies `ON CREATE SET`/`ON MATCH SET` to the right rows (matching
    /// real Cypher semantics exactly: `ON CREATE` fires whenever anything
    /// in the pattern was newly created, `ON MATCH` only when the whole
    /// pattern already existed as-is — the single per-row
    /// `MERGE_CREATED_KEY` tag is the correct model for this, not a
    /// simplification of it — see `eval_optional_part`'s
    /// `OPTIONAL_SEED_IDX_KEY` for the same hidden-tag precedent), then
    /// strips the tag before the rows become visible to the rest of the
    /// query.
    fn apply_merge_set(
        &self,
        write_txn: &WriteTransaction,
        clause: &MergeClause,
        rows: &mut Vec<BindingRow>,
    ) -> Result<(), QueryError> {
        for row in rows.iter_mut() {
            let created = match row.remove(MERGE_CREATED_KEY) {
                Some(Binding::Value(PropertyValue::Bool(b))) => b,
                other => unreachable!("{MERGE_CREATED_KEY} tagged internally as Binding::Value(Bool), got {other:?}"),
            };
            let items = if created { &clause.on_create } else { &clause.on_match };
            for (pa, lit) in items {
                let binding = row.get(&pa.var).ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
                let value = literal_to_value(lit);
                match binding {
                    Binding::Node(id) => {
                        GraphStore::set_node_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                    }
                    Binding::Edge(id) => {
                        GraphStore::set_edge_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                    }
                    Binding::Value(_) | Binding::List(_) => {
                        return Err(QueryError::UnboundVariable(format!(
                            "'{}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding",
                            pa.var
                        )))
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_match(
        &self,
        txn: Txn,
        clauses: &[QueryClause],
        tail: &Option<Tail>,
        order_by: &Option<Vec<(ReturnExpr, SortDir)>>,
        limit: Option<i64>,
    ) -> Result<QueryResult, QueryError> {
        // Threads bindings through each MATCH/UNWIND/WITH clause.
        // `carried_vars` tells the planner which of the next MATCH clause's
        // pattern variables are already bound (-> LogicalPlan::Seed) rather
        // than fresh (-> a scan). Starts empty: the first clause never has
        // anything carried into it.
        let mut carried_vars: HashSet<String> = HashSet::new();
        let mut current_rows: Vec<BindingRow> = vec![BindingRow::new()];
        for clause in clauses {
            match clause {
                QueryClause::Match(part) => {
                    let plan = build_match_plan(&part.pattern, &part.where_clause, &carried_vars)?;
                    current_rows = if part.optional {
                        let new_vars = pattern_new_vars(&part.pattern, &carried_vars);
                        self.eval_optional_part(txn, &plan, &current_rows, &new_vars)?
                    } else {
                        self.eval_plan(txn, &plan, &current_rows)?
                    };
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &part.with,
                        current_rows,
                        pattern_all_vars(&part.pattern),
                        &mut carried_vars,
                    )?;
                }
                QueryClause::Unwind(u) => {
                    current_rows = self.eval_unwind(txn, u, &current_rows)?;
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &u.with,
                        current_rows,
                        HashSet::from([u.var.clone()]),
                        &mut carried_vars,
                    )?;
                }
                QueryClause::Merge(m) => {
                    // MERGE always needs real `.insert`-capable write
                    // access, whether or not the rest of the statement
                    // would otherwise be read-only (e.g. `MERGE (n) RETURN
                    // n`) — see `is_read_only`, which already accounts for
                    // this by checking `clauses` too, so `txn` is
                    // guaranteed to be `Txn::Write` here.
                    let write_txn = require_write_txn(txn);
                    current_rows = self.eval_merge(write_txn, m, &current_rows)?;
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &m.with,
                        current_rows,
                        pattern_all_vars(&m.pattern),
                        &mut carried_vars,
                    )?;
                }
            }
        }
        // ORDER BY must see every matching row before LIMIT truncates —
        // sort, then take N, not the other way around. Only pre-truncate
        // (the v1 "doesn't short-circuit" path) when there's no ORDER BY to
        // invalidate it; DELETE/SET+LIMIT keep their "stop after N
        // bindings" behavior since they have no ORDER BY position in the
        // grammar.
        if order_by.is_none() {
            if let Some(count) = limit {
                current_rows.truncate(count.max(0) as usize);
            }
        }
        // Delete/Set need real `.insert`/`.remove`-capable write access,
        // not just `Txn`'s read-only `get`/`iter` — but they're only ever
        // reached via `Executor::execute`'s write-dispatch path (see
        // `is_read_only`), which always opens a `WriteTransaction`, so
        // `txn` is guaranteed to be `Txn::Write` here.
        let mut result = match tail {
            // A missing tail only ever occurs with a MERGE clause and
            // nothing after it — a pure write, same empty result shape
            // standalone CREATE already returns (not one blank row per
            // `current_rows`, which a synthetic `Tail::Return(vec![])`
            // would produce instead).
            None => QueryResult { columns: vec![], rows: vec![] },
            Some(Tail::Return(items)) => self.materialize_return(txn, items, &current_rows)?,
            Some(Tail::Delete(vars)) => {
                self.materialize_delete(require_write_txn(txn), vars, &current_rows, false)?
            }
            Some(Tail::DetachDelete(vars)) => {
                self.materialize_delete(require_write_txn(txn), vars, &current_rows, true)?
            }
            Some(Tail::Set(items)) => self.materialize_set(require_write_txn(txn), items, &current_rows)?,
            Some(Tail::Create(patterns)) => {
                self.materialize_create(require_write_txn(txn), patterns, &current_rows)?
            }
        };
        if let Some(order_by) = order_by {
            result.rows = apply_order_by(result.rows, &result.columns, order_by)?;
            if let Some(count) = limit {
                result.rows.truncate(count.max(0) as usize);
            }
        }
        Ok(result)
    }

    /// Applies a clause's optional trailing `WITH` (shared by both
    /// `QueryClause::Match` and `QueryClause::Unwind`, which can each end
    /// in one — see `QueryClause`'s docs), or, with no `WITH`, grows
    /// `carried_vars` by `new_vars` so the next clause shares this one's
    /// binding scope — same "no WITH means stay in scope" rule `OPTIONAL
    /// MATCH` already gets, now uniform across clause kinds.
    fn apply_with_or_carry(
        &self,
        txn: Txn,
        with: &Option<WithClause>,
        rows: Vec<BindingRow>,
        new_vars: HashSet<String>,
        carried_vars: &mut HashSet<String>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let Some(with) = with else {
            carried_vars.extend(new_vars);
            return Ok(rows);
        };
        let mut rows = self.materialize_with(txn, with, &rows)?;
        if let Some(with_order_by) = &with.order_by {
            rows = self.apply_order_by_bindings(txn, rows, with_order_by)?;
        }
        if let Some(with_limit) = with.limit {
            rows.truncate(with_limit.max(0) as usize);
        }
        *carried_vars = with.items.iter().enumerate().map(with_item_output_name).collect();
        Ok(rows)
    }

    /// `UNWIND`'s fan-out. Not a graph traversal — like `WITH`, handled
    /// directly here rather than through a `LogicalPlan`/`eval_plan` (see
    /// `UnwindClause`'s docs). Cross-joins each input row against every
    /// element of that row's resolved list, then applies the clause's own
    /// `WHERE`.
    fn eval_unwind(&self, txn: Txn, clause: &UnwindClause, rows: &[BindingRow]) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in rows {
            let elements: Vec<Binding> = match &clause.source {
                UnwindSource::Var(name) => {
                    let binding = row.get(name).ok_or_else(|| QueryError::UnboundVariable(name.clone()))?;
                    let Binding::List(items) = binding else {
                        return Err(QueryError::Parse(format!(
                            "'{name}' isn't a list — UNWIND needs a list (e.g. from collect())"
                        )));
                    };
                    items.iter().map(value_to_binding_restore).collect()
                }
                UnwindSource::List(literals) => {
                    literals.iter().map(|lit| Binding::Value(literal_to_value(lit))).collect()
                }
            };
            for element in elements {
                let mut new_row = row.clone();
                new_row.insert(clause.var.clone(), element);
                out.push(new_row);
            }
        }
        if let Some(where_clause) = &clause.where_clause {
            let mut filtered = Vec::with_capacity(out.len());
            for row in out {
                if self.eval_with_expr(txn, where_clause, &row)? {
                    filtered.push(row);
                }
            }
            out = filtered;
        }
        Ok(out)
    }

    /// Projects `rows` through a `WITH` clause. Unlike `materialize_return`
    /// (which resolves everything down to display `Value`s), a bare
    /// variable reference (`WITH message`) must keep its graph identity
    /// (`Binding::Node`/`Edge`) so the next `QueryPart` can keep
    /// traversing from it — only computed expressions collapse to a
    /// scalar `Binding::Value`.
    fn materialize_with(
        &self,
        txn: Txn,
        with: &WithClause,
        rows: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = if !has_aggregate(&with.items) {
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let mut new_row = BindingRow::new();
                for (i, item) in with.items.iter().enumerate() {
                    let name = with_item_output_name((i, item));
                    let binding = self.item_binding(txn, &item.expr, row)?;
                    new_row.insert(name, binding);
                }
                out.push(new_row);
            }
            out
        } else {
            validate_return_items(&with.items)?;
            let grouped = self.resolve_grouped_rows(txn, &with.items, rows)?;
            grouped
                .into_iter()
                .map(|bindings| {
                    with.items
                        .iter()
                        .enumerate()
                        .zip(bindings)
                        .map(|((i, item), b)| (with_item_output_name((i, item)), b))
                        .collect()
                })
                .collect()
        };
        if let Some(where_clause) = &with.where_clause {
            let mut filtered = Vec::with_capacity(out.len());
            for row in out {
                if self.eval_with_expr(txn, where_clause, &row)? {
                    filtered.push(row);
                }
            }
            out = filtered;
        }
        Ok(out)
    }

    /// The `Binding` one WITH/RETURN item evaluates to for one input row. A
    /// bare `Var` keeps its graph identity (`Binding::Node`/`Edge`) so a
    /// later `QueryPart` can keep traversing from it; anything else
    /// (computed expressions) collapses to `Binding::Value`. Shared by the
    /// non-aggregating `materialize_with` path and grouping-key evaluation.
    fn item_binding(&self, txn: Txn, expr: &ReturnExpr, row: &BindingRow) -> Result<Binding, QueryError> {
        match expr {
            ReturnExpr::Var(v) => row.get(v).cloned().ok_or_else(|| QueryError::UnboundVariable(v.clone())),
            other => {
                let value = self.eval_return_expr(txn, other, row)?;
                Ok(Binding::Value(value_to_property_value(&value)))
            }
        }
    }

    /// Same sort as `apply_order_by`, but over `BindingRow`s (a `WITH`
    /// clause's own ORDER BY, which must run before that row set becomes
    /// the seed for the next `QueryPart` — sorting/limiting a WITH changes
    /// *which* rows continue, not just their presentation order).
    fn apply_order_by_bindings(
        &self,
        txn: Txn,
        rows: Vec<BindingRow>,
        order_by: &[(ReturnExpr, SortDir)],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut keyed: Vec<(Vec<Value>, BindingRow)> = Vec::with_capacity(rows.len());
        for row in rows {
            let value_map = self.binding_row_to_value_map(txn, &row)?;
            let keys = order_by
                .iter()
                .map(|(expr, _)| eval_projected_expr(expr, &value_map))
                .collect::<Result<Vec<_>, _>>()?;
            keyed.push((keys, row));
        }
        keyed.sort_by(|(ka, _), (kb, _)| {
            for (i, (_, dir)) in order_by.iter().enumerate() {
                let ord = compare_with_dir(&ka[i], &kb[i], *dir);
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
        Ok(keyed.into_iter().map(|(_, row)| row).collect())
    }

    fn binding_row_to_value_map(
        &self,
        txn: Txn,
        row: &BindingRow,
    ) -> Result<HashMap<String, Value>, QueryError> {
        let mut map = HashMap::with_capacity(row.len());
        for (k, binding) in row {
            map.insert(k.clone(), self.binding_to_value(txn, binding)?);
        }
        Ok(map)
    }

    /// Resolves a `Binding` to its display `Value` — a `Node`/`Edge`
    /// binding fetches the full current record, a scalar `Value` binding
    /// passes through (collapsing a stored `PropertyValue::Null` to
    /// `Value::Null`, same as everywhere else null is represented).
    fn binding_to_value(&self, txn: Txn, b: &Binding) -> Result<Value, QueryError> {
        Ok(match b {
            Binding::Node(id) => Value::Node(
                GraphStore::get_node_in_txn(txn, *id)?
                    .expect("bound node exists within this statement's transaction"),
            ),
            Binding::Edge(id) => Value::Edge(
                GraphStore::get_edge_in_txn(txn, *id)?
                    .expect("bound edge exists within this statement's transaction"),
            ),
            Binding::Value(PropertyValue::Null) => Value::Null,
            Binding::Value(pv) => Value::Property(pv.clone()),
            Binding::List(items) => Value::List(items.clone()),
        })
    }

    /// Folds `rows` into groups keyed by every non-aggregate item's per-row
    /// `Binding` (via `item_binding`), then finishes each aggregate item's
    /// accumulator per group. Returns one `Vec<Binding>` per output group,
    /// column-aligned with `items`. Shared by `materialize_with` and
    /// `materialize_return` — both already take the same `rows: &[BindingRow]`
    /// input type, so the grouping core stays in `Binding`-space (preserving
    /// graph identity for bare-var grouping keys) and each caller does its
    /// own thin final conversion.
    ///
    /// Grouping-key lookup is a hash-map lookup (`group_index`, keyed by
    /// `binding_hash_key`'s output — `Binding`/`PropertyValue` don't
    /// derive `Eq`/`Hash` themselves, `PropertyValue::Float` can't, so
    /// `HashKey` stands in for them; see its docs) into `groups`, which
    /// stays a plain `Vec` for insertion-order-stable output when there's
    /// no ORDER BY. O(1) average per row, not the O(rows × groups) linear
    /// scan this used to be — see BENCHMARKS.md for the measured
    /// before/after.
    ///
    /// Callers must call `validate_return_items` first — this function
    /// assumes every aggregate `Call` item has already been checked to
    /// have exactly one argument.
    fn resolve_grouped_rows(
        &self,
        txn: Txn,
        items: &[ReturnItem],
        rows: &[BindingRow],
    ) -> Result<Vec<Vec<Binding>>, QueryError> {
        struct Group {
            // Aligned to `items`: `Some` at a non-aggregate item's index,
            // `None` at an aggregate item's index (both vecs below are
            // index-aligned to `items` the same way, so exactly one of
            // `key_bindings[i]`/`accs[i]` is populated per `i`).
            key_bindings: Vec<Option<Binding>>,
            accs: Vec<Option<AggAcc>>,
            row_count: i64,
        }
        fn fresh_accs(items: &[ReturnItem]) -> Vec<Option<AggAcc>> {
            items
                .iter()
                .map(|item| match &item.expr {
                    ReturnExpr::Call { name, distinct, .. } if is_aggregate_name(name) => {
                        Some(AggAcc::identity(name, *distinct))
                    }
                    _ => None,
                })
                .collect()
        }

        // Groups live in `groups` (insertion order, for stable output when
        // there's no ORDER BY) with `group_index` as a hash-based lookup
        // into it, keyed by a hashable stand-in for `key_bindings` (see
        // `HashKey` — `Binding`/`PropertyValue` don't derive `Eq`/`Hash`
        // themselves, `PropertyValue::Float` can't). O(1) average lookup
        // per row instead of the O(groups) linear scan this replaced —
        // see BENCHMARKS.md for the measured before/after.
        let mut groups: Vec<Group> = Vec::new();
        let mut group_index: HashMap<Vec<Option<HashKey>>, usize> = HashMap::new();
        for row in rows {
            let mut key_bindings = Vec::with_capacity(items.len());
            for item in items {
                key_bindings.push(if is_top_level_aggregate(&item.expr) {
                    None
                } else {
                    Some(self.item_binding(txn, &item.expr, row)?)
                });
            }
            let hash_key: Vec<Option<HashKey>> =
                key_bindings.iter().map(|b| b.as_ref().map(binding_hash_key)).collect();
            let group_idx = *group_index.entry(hash_key).or_insert_with(|| {
                groups.push(Group {
                    key_bindings: key_bindings.clone(),
                    accs: fresh_accs(items),
                    row_count: 0,
                });
                groups.len() - 1
            });
            let group = &mut groups[group_idx];
            group.row_count += 1;
            for (i, item) in items.iter().enumerate() {
                let ReturnExpr::Call { args, .. } = &item.expr else { continue };
                if !is_top_level_aggregate(&item.expr) {
                    continue;
                }
                // Standard Cypher null-skipping: a null argument (e.g. an
                // unmatched OPTIONAL MATCH variable) contributes to
                // neither the accumulator nor its DISTINCT dedup set —
                // this is what makes `count(x)` exclude a null-padded row
                // while `count(*)` (tracked via `row_count`, not an
                // accumulator at all) includes it.
                let value = self.eval_return_expr(txn, &args[0], row)?;
                if !matches!(value, Value::Null) {
                    if let Some(acc) = &mut group.accs[i] {
                        acc.fold(&value)?;
                    }
                }
            }
        }

        // Global aggregate over an empty result set (no grouping-key items
        // at all, and no rows to seed a group from) still produces exactly
        // one output row — `count`/`count(*)` -> 0, `sum` -> 0,
        // `avg`/`min`/`max` -> Null, `collect` -> [] — via the same
        // fresh-accumulator `finish()` path a normal empty-contribution
        // group already uses below, not a separate code path.
        let no_key_items = items.iter().all(|item| is_top_level_aggregate(&item.expr));
        if groups.is_empty() && no_key_items {
            groups.push(Group {
                key_bindings: vec![None; items.len()],
                accs: fresh_accs(items),
                row_count: 0,
            });
        }

        let mut out = Vec::with_capacity(groups.len());
        for mut group in groups {
            let mut row_out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let binding = if matches!(item.expr, ReturnExpr::CountStar) {
                    Binding::Value(PropertyValue::Int(group.row_count))
                } else if is_top_level_aggregate(&item.expr) {
                    let value = group.accs[i]
                        .take()
                        .expect("aggregate item must have an accumulator")
                        .finish();
                    value_to_binding(value)
                } else {
                    group.key_bindings[i].clone().expect("non-aggregate item must have a key binding")
                };
                row_out.push(binding);
            }
            out.push(row_out);
        }
        Ok(out)
    }

    /// WITH's HAVING-equivalent — evaluated against the already-projected/
    /// grouped row, same as ORDER BY. Never pushed into the planner (see
    /// `WithExpr`'s docs).
    fn eval_with_expr(&self, txn: Txn, expr: &WithExpr, row: &BindingRow) -> Result<bool, QueryError> {
        Ok(match expr {
            WithExpr::And(l, r) => self.eval_with_expr(txn, l, row)? && self.eval_with_expr(txn, r, row)?,
            WithExpr::Or(l, r) => self.eval_with_expr(txn, l, row)? || self.eval_with_expr(txn, r, row)?,
            WithExpr::Not(e) => !self.eval_with_expr(txn, e, row)?,
            WithExpr::Compare(lhs, op, lit) => {
                let value = self.eval_return_expr(txn, lhs, row)?;
                compare_value(&value, *op, lit)
            }
        })
    }

    /// Evaluates an `OPTIONAL MATCH` part with left-outer-join semantics:
    /// every outer row survives, whether or not the optional pattern
    /// matched anything for it. Must wrap the *whole* subplan rather than
    /// null-padding inside `Expand`/`VarExpand` themselves — baking it in
    /// there would turn every default (non-optional) `Expand` into a
    /// left-outer-join too (breaking existing inner-join semantics), and
    /// would mis-handle multi-hop optional patterns: IS7's optional
    /// pattern is 2 hops, and per-hop null-padding would emit one
    /// null-padded row per *hop-1* match even when hop 2 also matched,
    /// instead of collapsing to exactly one row per outer row that had
    /// zero end-to-end matches.
    ///
    /// Implementation: tag each outer row with its index, evaluate the
    /// subplan once over the whole tagged batch (a single seed, not one
    /// call per row), group results back by that index, then for any
    /// outer index with zero results, emit the outer row unchanged plus
    /// `Null` for every variable the optional pattern would have newly
    /// introduced.
    fn eval_optional_part(
        &self,
        txn: Txn,
        plan: &LogicalPlan,
        outer_rows: &[BindingRow],
        new_vars: &HashSet<String>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let tagged: Vec<BindingRow> = outer_rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let mut r = row.clone();
                r.insert(OPTIONAL_SEED_IDX_KEY.to_string(), Binding::Value(PropertyValue::Int(i as i64)));
                r
            })
            .collect();
        let results = self.eval_plan(txn, plan, &tagged)?;
        let mut by_idx: HashMap<i64, Vec<BindingRow>> = HashMap::new();
        for mut row in results {
            let idx = match row.remove(OPTIONAL_SEED_IDX_KEY) {
                Some(Binding::Value(PropertyValue::Int(i))) => i,
                other => unreachable!("__seed_idx tagged internally as Binding::Value(Int), got {other:?}"),
            };
            by_idx.entry(idx).or_default().push(row);
        }
        let mut out = Vec::with_capacity(outer_rows.len());
        for (i, outer_row) in outer_rows.iter().enumerate() {
            match by_idx.remove(&(i as i64)) {
                Some(matches) => out.extend(matches),
                None => {
                    let mut padded = outer_row.clone();
                    for var in new_vars {
                        padded.insert(var.clone(), Binding::Value(PropertyValue::Null));
                    }
                    out.push(padded);
                }
            }
        }
        Ok(out)
    }

    fn eval_plan(
        &self,
        txn: Txn,
        plan: &LogicalPlan,
        seed: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        match plan {
            LogicalPlan::Seed { var } => {
                debug_assert!(
                    seed.first().is_none_or(|row| row.contains_key(var)),
                    "Seed{{var: {var:?}}} planned for a var not present in the carried-forward rows"
                );
                Ok(seed.to_vec())
            }
            LogicalPlan::AllNodesScan { var } => self.scan(txn, var, None, seed),
            LogicalPlan::NodeByLabelScan { var, label } => self.scan(txn, var, Some(label), seed),
            LogicalPlan::Expand {
                input,
                from_var,
                to_var,
                rel_var,
                rel_label,
                direction,
            } => {
                let base_rows = self.eval_plan(txn, input, seed)?;
                let mut out = Vec::new();
                for row in base_rows {
                    let Some(Binding::Node(from_id)) = row.get(from_var).cloned() else {
                        return Err(QueryError::UnboundVariable(from_var.clone()));
                    };
                    let entries = neighbors_for_direction(txn, from_id, *direction, rel_label.as_deref())?;
                    for entry in entries {
                        let mut new_row = row.clone();
                        new_row.insert(to_var.clone(), Binding::Node(entry.other));
                        if let Some(rv) = rel_var {
                            new_row.insert(rv.clone(), Binding::Edge(entry.edge_id));
                        }
                        out.push(new_row);
                    }
                }
                Ok(out)
            }
            LogicalPlan::VarExpand {
                input,
                from_var,
                to_var,
                rel_label,
                direction,
                min_hops,
                max_hops,
            } => {
                let base_rows = self.eval_plan(txn, input, seed)?;
                let mut out = Vec::new();
                let unbounded = max_hops.is_none();
                let effective_max = max_hops.unwrap_or(VAR_EXPAND_DEPTH_CAP);
                for row in base_rows {
                    let Some(Binding::Node(start_id)) = row.get(from_var).cloned() else {
                        return Err(QueryError::UnboundVariable(from_var.clone()));
                    };
                    let mut visited = HashSet::new();
                    visited.insert(start_id);
                    if *min_hops == 0 {
                        let mut new_row = row.clone();
                        new_row.insert(to_var.clone(), Binding::Node(start_id));
                        out.push(new_row);
                    }
                    let mut frontier = vec![start_id];
                    let mut depth = 0u32;
                    while depth < effective_max && !frontier.is_empty() {
                        depth += 1;
                        let mut next_frontier = Vec::new();
                        for node in frontier {
                            let entries = neighbors_for_direction(txn, node, *direction, rel_label.as_deref())?;
                            for entry in entries {
                                if visited.insert(entry.other) {
                                    next_frontier.push(entry.other);
                                    if depth >= *min_hops {
                                        let mut new_row = row.clone();
                                        new_row.insert(to_var.clone(), Binding::Node(entry.other));
                                        out.push(new_row);
                                    }
                                }
                            }
                        }
                        frontier = next_frontier;
                        if depth == effective_max && unbounded && !frontier.is_empty() {
                            // Unbounded (`*N..`) traversal hit the safety
                            // cap with more still reachable — error rather
                            // than silently truncate results, which would
                            // be a wrong-answer failure mode for a
                            // correctness-benchmark tool.
                            return Err(QueryError::Parse(format!(
                                "variable-length traversal exceeded the safety depth cap ({VAR_EXPAND_DEPTH_CAP} \
                                 hops) — likely a cyclic graph or unexpectedly large fanout; narrow the pattern or \
                                 add an explicit upper bound (e.g. *0..10)"
                            )));
                        }
                    }
                }
                Ok(out)
            }
            LogicalPlan::Filter { input, predicate } => {
                let rows = self.eval_plan(txn, input, seed)?;
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    if self.eval_expr(txn, predicate, &row)? {
                        out.push(row);
                    }
                }
                Ok(out)
            }
        }
    }

    /// Cross-joins the scan against `seed` — for the first `QueryPart` in a
    /// statement, `seed` is always exactly one empty row (see
    /// `execute_match`), so this reduces to "one row per scanned node,"
    /// the same as before this scan ever needed a `seed` parameter at
    /// all. It matters for a later `QueryPart` (after a `WITH` boundary)
    /// whose pattern doesn't chain from an already-bound variable — e.g.
    /// `MATCH (a) WITH a MATCH (b) ...` — real Cypher's cross-join
    /// semantics require every carried-forward binding (`a`) to survive
    /// alongside every row this scan produces (`b`), not get silently
    /// dropped. This is a real cost, not just a correctness fix: a scan
    /// against N carried rows does N × (scanned rows) work, same as any
    /// cross join.
    fn scan(&self, txn: Txn, var: &str, label: Option<&str>, seed: &[BindingRow]) -> Result<Vec<BindingRow>, QueryError> {
        let nodes = GraphStore::all_nodes_in_txn(txn, label)?;
        let mut out = Vec::with_capacity(seed.len() * nodes.len());
        for base_row in seed {
            for n in &nodes {
                let mut row = base_row.clone();
                row.insert(var.to_string(), Binding::Node(n.id));
                out.push(row);
            }
        }
        Ok(out)
    }

    fn eval_expr(&self, txn: Txn, expr: &Expr, row: &BindingRow) -> Result<bool, QueryError> {
        Ok(match expr {
            Expr::And(l, r) => self.eval_expr(txn, l, row)? && self.eval_expr(txn, r, row)?,
            Expr::Or(l, r) => self.eval_expr(txn, l, row)? || self.eval_expr(txn, r, row)?,
            Expr::Not(e) => !self.eval_expr(txn, e, row)?,
            Expr::Compare(pa, op, lit) => {
                let prop_value = self.lookup_prop(txn, pa, row)?;
                compare(&prop_value, *op, lit)
            }
            Expr::HasLabel(var, label) => {
                let binding = row.get(var).ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                let Binding::Node(id) = binding else {
                    return Err(QueryError::UnboundVariable(var.clone()));
                };
                let node = GraphStore::get_node_in_txn(txn, *id)?;
                node.is_some_and(|n| n.labels.iter().any(|l| l == label))
            }
            Expr::VarEq(a, b) => {
                let ba = row.get(a).ok_or_else(|| QueryError::UnboundVariable(a.clone()))?;
                let bb = row.get(b).ok_or_else(|| QueryError::UnboundVariable(b.clone()))?;
                match (ba, bb) {
                    (Binding::Node(x), Binding::Node(y)) => x == y,
                    (Binding::Edge(x), Binding::Edge(y)) => x == y,
                    // A null-padded `Binding::Value` (from an earlier
                    // OPTIONAL MATCH that didn't match) can't equal a
                    // real node/edge, and comparing across binding kinds
                    // (a node vs an edge) is never meaningful here — the
                    // planner only ever synthesizes VarEq between two
                    // occurrences of the same pattern variable, which are
                    // always the same kind when both are real.
                    _ => false,
                }
            }
        })
    }

    fn lookup_prop(
        &self,
        txn: Txn,
        pa: &PropAccess,
        row: &BindingRow,
    ) -> Result<Option<PropertyValue>, QueryError> {
        let binding = row
            .get(&pa.var)
            .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
        match binding {
            Binding::Node(id) => {
                let node = GraphStore::get_node_in_txn(txn, *id)?;
                Ok(node.and_then(|n| n.props.get(&pa.prop).cloned()))
            }
            Binding::Edge(id) => {
                let edge = GraphStore::get_edge_in_txn(txn, *id)?;
                Ok(edge.and_then(|e| e.props.get(&pa.prop).cloned()))
            }
            // A WITH-projected scalar (or list) has no `.prop` to access —
            // e.g. `WITH message.id AS messageId` then `messageId.foo`
            // isn't meaningful. Treat as absent rather than erroring,
            // consistent with how a missing property already behaves.
            Binding::Value(_) | Binding::List(_) => Ok(None),
        }
    }

    fn materialize_return(
        &self,
        txn: Txn,
        items: &[ReturnItem],
        rows: &[BindingRow],
    ) -> Result<QueryResult, QueryError> {
        let columns = items
            .iter()
            .enumerate()
            .map(|(i, item)| item.alias.clone().unwrap_or_else(|| default_column_name(&item.expr, i)))
            .collect();
        let out_rows = if !has_aggregate(items) {
            let mut out_rows = Vec::with_capacity(rows.len());
            for row in rows {
                let mut out_row = Vec::with_capacity(items.len());
                for item in items {
                    out_row.push(self.eval_return_expr(txn, &item.expr, row)?);
                }
                out_rows.push(out_row);
            }
            out_rows
        } else {
            validate_return_items(items)?;
            let grouped = self.resolve_grouped_rows(txn, items, rows)?;
            grouped
                .into_iter()
                .map(|bindings| {
                    bindings
                        .iter()
                        .map(|b| self.binding_to_value(txn, b))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(QueryResult {
            columns,
            rows: out_rows,
        })
    }

    fn eval_return_expr(
        &self,
        txn: Txn,
        expr: &ReturnExpr,
        row: &BindingRow,
    ) -> Result<Value, QueryError> {
        match expr {
            ReturnExpr::Var(var) => {
                let binding = row.get(var).ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                self.binding_to_value(txn, binding)
            }
            ReturnExpr::Prop(pa) => {
                let value = self.lookup_prop(txn, pa, row)?;
                Ok(match value {
                    // Collapse "prop missing" and "prop stored as null" into
                    // one null representation — see Value::Null docs.
                    Some(PropertyValue::Null) | None => Value::Null,
                    Some(pv) => Value::Property(pv),
                })
            }
            ReturnExpr::Lit(lit) => Ok(match lit {
                Literal::Null => Value::Null,
                other => Value::Literal(other.clone()),
            }),
            ReturnExpr::Call { name, args, .. } => {
                // Reaching here with an aggregate name means an aggregate
                // call slipped past `validate_return_items` (which only
                // allows one at a return item's top level) — grouping
                // itself never calls `eval_return_expr` on the aggregate
                // wrapper, only on each aggregate's own argument
                // subexpression (see `resolve_grouped_rows`), so this is
                // an internal-consistency error, not a normal user path.
                if is_aggregate_name(name) {
                    return Err(QueryError::Parse(format!(
                        "aggregate function '{name}' can only be used as a return item's top-level expression"
                    )));
                }
                let arg_values = args
                    .iter()
                    .map(|a| self.eval_return_expr(txn, a, row))
                    .collect::<Result<Vec<_>, _>>()?;
                call_builtin(name, &arg_values)
            }
            ReturnExpr::CountStar => Err(QueryError::Parse(
                "count(*) can only be used as a return item's top-level expression".into(),
            )),
            ReturnExpr::Case { test, whens, else_ } => {
                let test_value = match test {
                    Some(t) => Some(self.eval_return_expr(txn, t, row)?),
                    None => None,
                };
                for (when, then) in whens {
                    let when_value = self.eval_return_expr(txn, when, row)?;
                    // Deliberately reuses the same Null == Null -> true
                    // convention as `compare()` below, not standard
                    // three-valued NULL logic — IS7's `CASE r WHEN null
                    // THEN false ELSE true END` depends on this exact
                    // semantics to detect an OPTIONAL MATCH non-match.
                    let matched = match &test_value {
                        Some(tv) => value_eq(tv, &when_value),
                        None => matches!(when_value, Value::Literal(Literal::Bool(true))),
                    };
                    if matched {
                        return self.eval_return_expr(txn, then, row);
                    }
                }
                match else_ {
                    Some(e) => self.eval_return_expr(txn, e, row),
                    None => Ok(Value::Null),
                }
            }
        }
    }

    fn materialize_delete(
        &self,
        write_txn: &WriteTransaction,
        vars: &[String],
        rows: &[BindingRow],
        detach: bool,
    ) -> Result<QueryResult, QueryError> {
        let mut deleted_nodes = HashSet::new();
        let mut deleted_edges = HashSet::new();
        for row in rows {
            for var in vars {
                let binding = row.get(var).ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                match binding {
                    Binding::Node(id) => {
                        if deleted_nodes.insert(*id) {
                            GraphStore::delete_node_in_txn(write_txn, *id, detach)?;
                        }
                    }
                    Binding::Edge(id) => {
                        if deleted_edges.insert(*id) {
                            GraphStore::delete_edge_in_txn(write_txn, *id)?;
                        }
                    }
                    Binding::Value(_) | Binding::List(_) => {
                        return Err(QueryError::UnboundVariable(format!(
                            "'{var}' is a WITH-projected scalar, not a node/edge — DELETE needs a graph binding"
                        )))
                    }
                }
            }
        }
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }

    fn materialize_set(
        &self,
        write_txn: &WriteTransaction,
        items: &[(PropAccess, Literal)],
        rows: &[BindingRow],
    ) -> Result<QueryResult, QueryError> {
        for row in rows {
            for (pa, lit) in items {
                let binding = row.get(&pa.var).ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
                let value = literal_to_value(lit);
                match binding {
                    Binding::Node(id) => {
                        GraphStore::set_node_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                    }
                    Binding::Edge(id) => {
                        GraphStore::set_edge_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                    }
                    Binding::Value(_) | Binding::List(_) => {
                        return Err(QueryError::UnboundVariable(format!(
                            "'{}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding",
                            pa.var
                        )))
                    }
                }
            }
        }
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }
}

/// A statement never mutates anything iff it's a `MATCH ... RETURN` with no
/// `DELETE`/`DETACH DELETE`/`SET` tail *and* no `MERGE` clause anywhere in
/// it (`MERGE (n) RETURN n` has a `Tail::Return`, but still writes whenever
/// it has to create — checking `tail` alone here would be a real bug, not
/// just an incomplete check: it would send a MERGE-that-creates through a
/// `ReadTransaction`, which has no `.insert`). `Statement::Create` and
/// every other `Tail` variant always write. Confirmed by tracing every
/// function reachable from pattern/WHERE/WITH evaluation: none of them
/// ever call a table-mutating `*_in_txn` method for a `Tail::Return`
/// statement with no `MERGE` clause (a label-filtered scan looks up an
/// existing label id, it never allocates one — allocation only happens in
/// `create_node_in_txn`/`create_edge_in_txn`). `Executor::execute` uses
/// this to decide whether to open a `ReadTransaction` (no contention with
/// concurrent readers or a concurrent writer) or a `WriteTransaction`.
fn is_read_only(stmt: &Statement) -> bool {
    let Statement::Match { tail: Some(Tail::Return(_)), clauses, .. } = stmt else {
        return false;
    };
    !clauses.iter().any(|c| matches!(c, QueryClause::Merge(_)))
}

/// Recovers the real `&WriteTransaction` from a `Txn` for the two
/// `execute_match` tail arms (`DELETE`/`SET`) that need `.insert`/
/// `.remove`, not just `Txn`'s read-only `get`/`iter`. Panics if given
/// `Txn::Read` — which can't happen: `Tail::Delete`/`DetachDelete`/`Set`
/// make `is_read_only` return `false`, so `Executor::execute` always opens
/// a `WriteTransaction` (and thus `Txn::Write`) before reaching this path.
fn require_write_txn(txn: Txn<'_>) -> &WriteTransaction {
    let Txn::Write(write_txn) = txn else {
        unreachable!(
            "materialize_delete/materialize_set only reached via the write-dispatch path in \
             Executor::execute — is_read_only(stmt) is false for any statement with a Delete/ \
             DetachDelete/Set tail, so execute always opens a WriteTransaction for these"
        )
    };
    write_txn
}

fn default_column_name(expr: &ReturnExpr, idx: usize) -> String {
    match expr {
        ReturnExpr::Var(v) => v.clone(),
        ReturnExpr::Prop(pa) => format!("{}.{}", pa.var, pa.prop),
        ReturnExpr::Lit(_) => format!("col{idx}"),
        ReturnExpr::Call { name, .. } => format!("{name}(...)"),
        ReturnExpr::CountStar => "count(*)".to_string(),
        ReturnExpr::Case { .. } => format!("case{idx}"),
    }
}

/// The name a `WITH`/`RETURN` item is known by afterward — its alias, or
/// a name derived from the expression (its bare var name, `col{i}`, etc).
fn with_item_output_name((i, item): (usize, &ReturnItem)) -> String {
    item.alias.clone().unwrap_or_else(|| default_column_name(&item.expr, i))
}

/// True iff `expr` is itself an aggregate call — `count(*)`, or a `Call`
/// whose name is in `is_aggregate_name`'s fixed set. Does NOT look inside
/// `expr` for a nested aggregate — see `contains_aggregate` for that.
fn is_top_level_aggregate(expr: &ReturnExpr) -> bool {
    match expr {
        ReturnExpr::CountStar => true,
        ReturnExpr::Call { name, .. } => is_aggregate_name(name),
        _ => false,
    }
}

/// True iff `expr` contains an aggregate call anywhere inside it, at any
/// depth — used to reject an aggregate nested inside another aggregate's
/// argument, or inside a non-aggregate expression's `CASE`/`Call`
/// arguments (an aggregate must be a return item's *entire* top-level
/// expression — see `validate_return_items`).
fn contains_aggregate(expr: &ReturnExpr) -> bool {
    match expr {
        ReturnExpr::CountStar => true,
        ReturnExpr::Call { name, args, .. } => is_aggregate_name(name) || args.iter().any(contains_aggregate),
        ReturnExpr::Case { test, whens, else_ } => {
            test.as_deref().is_some_and(contains_aggregate)
                || whens.iter().any(|(w, t)| contains_aggregate(w) || contains_aggregate(t))
                || else_.as_deref().is_some_and(contains_aggregate)
        }
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::Lit(_) => false,
    }
}

/// True iff any item's top-level expression is an aggregate call —
/// `materialize_with`/`materialize_return` dispatch to the grouping path
/// iff this is true, otherwise the existing row-at-a-time path runs
/// completely unchanged (zero perf/behavior impact on non-aggregating
/// queries).
fn has_aggregate(items: &[ReturnItem]) -> bool {
    items.iter().any(|item| is_top_level_aggregate(&item.expr))
}

/// Validates a RETURN/WITH item list before any row is processed: every
/// aggregate call has exactly one argument (`count(*)`, the zero-argument
/// form, is `CountStar`, a separate variant — never reaches the `Call`
/// arm here), no aggregate's own argument contains a nested aggregate
/// call, and no non-aggregate item's expression contains an aggregate
/// call anywhere inside it (aggregates must be a return item's entire
/// top-level expression — justified by there being no arithmetic
/// operators anywhere in this engine yet, so `count(n) * 2`-style
/// composition is already impossible, and nothing in the target query set
/// needs an aggregate nested inside a `CASE` branch).
fn validate_return_items(items: &[ReturnItem]) -> Result<(), QueryError> {
    for item in items {
        match &item.expr {
            ReturnExpr::CountStar => {}
            ReturnExpr::Call { name, args, .. } if is_aggregate_name(name) => {
                if args.len() != 1 {
                    return Err(QueryError::Parse(format!(
                        "{name}() takes exactly one argument (use count(*) for a row count with no argument)"
                    )));
                }
                if contains_aggregate(&args[0]) {
                    return Err(QueryError::Parse(format!(
                        "aggregate function '{name}' can't take another aggregate as an argument"
                    )));
                }
            }
            other => {
                if contains_aggregate(other) {
                    return Err(QueryError::Parse(
                        "an aggregate function must be a return item's entire expression, not nested inside \
                         another expression"
                            .into(),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Grouping-key hashing — deliberately at the `Binding` level (`NodeId`/
/// `EdgeId`/`PropertyValue`), not `Value`: cheaper (no `GraphStore` fetch
/// just to compute) and the correct semantics (two `Binding::Node`s are
/// the same group iff the same node **identity**, not equal-by-struct-
/// contents). `Binding::List`'s elements are `Value`s already, so those
/// delegate to `value_hash_key` directly.
fn binding_hash_key(b: &Binding) -> HashKey {
    match b {
        Binding::Node(id) => HashKey::Node(*id),
        Binding::Edge(id) => HashKey::Edge(*id),
        Binding::Value(pv) => property_value_hash_key(pv),
        Binding::List(items) => HashKey::List(items.iter().map(value_hash_key).collect()),
    }
}

/// Converts a finished `AggAcc::finish()` result to the `Binding` it's
/// carried as through a `WITH` boundary — `collect()`'s `Value::List`
/// needs `Binding::List` (no list variant in `PropertyValue`, the
/// storage-layer type `Binding::Value` wraps), everything else collapses
/// to `Binding::Value` same as any other computed WITH item.
fn value_to_binding(v: Value) -> Binding {
    match v {
        Value::List(items) => Binding::List(items),
        other => Binding::Value(value_to_property_value(&other)),
    }
}

/// `UNWIND`'s counterpart to `value_to_binding` — restores graph identity
/// from a `collect()`'d element instead of collapsing it. `Value::Node`/
/// `Edge` carry their full `id`, so this isn't lossy the way carrying only
/// a display value would be: a `MATCH` after the `UNWIND` can keep
/// traversing from the restored `Binding::Node`/`Edge`, exactly as if it
/// had been bound by a fresh scan/expand. See `Binding::List`'s docs,
/// which anticipated this exact restoration.
fn value_to_binding_restore(v: &Value) -> Binding {
    match v {
        Value::Node(n) => Binding::Node(n.id),
        Value::Edge(e) => Binding::Edge(e.id),
        Value::Property(pv) => Binding::Value(pv.clone()),
        Value::Literal(lit) => Binding::Value(literal_to_value(lit)),
        Value::List(items) => Binding::List(items.clone()),
        Value::Null => Binding::Value(PropertyValue::Null),
    }
}

/// `WithExpr::Compare`'s value-vs-literal comparison — reuses `compare()`
/// (below) by reducing a `Value` down to the `Option<PropertyValue>` shape
/// it expects; `Node`/`Edge`/`List` have no meaningful comparison against
/// a `Literal` and fall back to "absent", same as a missing property does.
fn compare_value(value: &Value, op: CompareOp, lit: &Literal) -> bool {
    let prop = match value {
        Value::Null => None,
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(l) => Some(literal_to_value(l)),
        Value::Node(_) | Value::Edge(_) | Value::List(_) => None,
    };
    compare(&prop, op, lit)
}

/// Coerces a materialized `Value` down to a `PropertyValue` for storing in
/// `Binding::Value` — used by `item_binding` for a computed (non-bare-var)
/// WITH/RETURN item. `Value::Node`/`Edge` can't occur here in practice (no
/// non-aggregate `ReturnExpr` form produces one except `Var`, which takes
/// the bare-variable path instead). `Value::List` can't occur here either
/// — `collect()` only ever appears in an aggregating item list, which
/// `has_aggregate` routes to `resolve_grouped_rows`/`Binding::List`
/// instead of through `item_binding` at all. Both fall back to `Null`
/// rather than needing a fallible signature for an unreachable case.
fn value_to_property_value(v: &Value) -> PropertyValue {
    match v {
        Value::Null => PropertyValue::Null,
        Value::Property(pv) => pv.clone(),
        Value::Literal(lit) => literal_to_value(lit),
        Value::Node(_) | Value::Edge(_) | Value::List(_) => PropertyValue::Null,
    }
}

fn literal_to_value(lit: &Literal) -> PropertyValue {
    match lit {
        Literal::Int(i) => PropertyValue::Int(*i),
        Literal::Float(f) => PropertyValue::Float(*f),
        Literal::String(s) => PropertyValue::String(s.clone()),
        Literal::Bool(b) => PropertyValue::Bool(*b),
        Literal::Null => PropertyValue::Null,
        Literal::Param(name) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
    }
}

fn literal_props_to_values(props: &[(String, Literal)]) -> BTreeMap<String, PropertyValue> {
    props.iter().map(|(k, v)| (k.clone(), literal_to_value(v))).collect()
}

fn tag_merge_created(mut row: BindingRow, created: bool) -> BindingRow {
    row.insert(MERGE_CREATED_KEY.to_string(), Binding::Value(PropertyValue::Bool(created)));
    row
}

/// Rejects a `MERGE` pattern token that's neither already bound in `row`
/// nor constrained by any label/property — matching or creating it would
/// mean guessing at "any node," which this codebase's "error on an
/// ambiguous shape" stance treats as a mistake to catch (not a silent
/// "match/create arbitrarily" default). Called before any graph work, not
/// just before the create-fallback branch — an unconstrained, unbound
/// token would otherwise let `eval_merge`'s search phase silently "match"
/// every node in the graph (`AllNodesScan`, no `Filter`) instead of
/// erroring.
fn require_mergeable(node: &NodePattern, row: &BindingRow) -> Result<(), QueryError> {
    let already_bound = node.var.as_ref().is_some_and(|v| row.contains_key(v));
    if !already_bound && node.labels.is_empty() && node.props.is_empty() {
        return Err(QueryError::Parse(
            "MERGE requires a label or property to match/create by — an unconstrained node pattern is ambiguous"
                .into(),
        ));
    }
    Ok(())
}

fn pattern_labels(labels: &[String]) -> Vec<&str> {
    if labels.is_empty() {
        vec!["Node"]
    } else {
        labels.iter().map(|s| s.as_str()).collect()
    }
}

/// `Either` (undirected `-[r:TYPE]-`) has no single storage-level call —
/// query both directions and dedupe by `edge_id` (a self-loop would
/// otherwise appear twice, once from each direction's adjacency table).
fn neighbors_for_direction(
    txn: Txn,
    node: NodeId,
    direction: ExpandDirection,
    rel_label: Option<&str>,
) -> Result<Vec<AdjEntry>, QueryError> {
    Ok(match direction {
        ExpandDirection::Out => GraphStore::neighbors_in_txn(txn, node, Direction::Out, rel_label)?,
        ExpandDirection::In => GraphStore::neighbors_in_txn(txn, node, Direction::In, rel_label)?,
        ExpandDirection::Either => {
            let mut out = GraphStore::neighbors_in_txn(txn, node, Direction::Out, rel_label)?;
            let inbound = GraphStore::neighbors_in_txn(txn, node, Direction::In, rel_label)?;
            let seen: HashSet<EdgeId> = out.iter().map(|e| e.edge_id).collect();
            out.extend(inbound.into_iter().filter(|e| !seen.contains(&e.edge_id)));
            out
        }
    })
}

fn compare(prop: &Option<PropertyValue>, op: CompareOp, lit: &Literal) -> bool {
    let Some(prop) = prop else { return false };
    match (prop, lit) {
        (PropertyValue::Int(a), Literal::Int(b)) => cmp_f64(op, *a as f64, *b as f64),
        (PropertyValue::Int(a), Literal::Float(b)) => cmp_f64(op, *a as f64, *b),
        (PropertyValue::Float(a), Literal::Float(b)) => cmp_f64(op, *a, *b),
        (PropertyValue::Float(a), Literal::Int(b)) => cmp_f64(op, *a, *b as f64),
        (PropertyValue::String(a), Literal::String(b)) => cmp_ord(op, a.as_str(), b.as_str()),
        (PropertyValue::Bool(a), Literal::Bool(b)) => match op {
            CompareOp::Eq => a == b,
            CompareOp::Ne => a != b,
            _ => false,
        },
        (PropertyValue::Null, Literal::Null) => matches!(op, CompareOp::Eq),
        _ => false,
    }
}

fn cmp_f64(op: CompareOp, a: f64, b: f64) -> bool {
    match op {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
    }
}

fn cmp_ord<T: PartialOrd>(op: CompareOp, a: T, b: T) -> bool {
    match op {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
    }
}

/// Value equality for CASE's WHEN-comparison (and, elsewhere, DISTINCT
/// dedup within an aggregate). Null == Null -> true here deliberately,
/// matching `compare()`'s convention above, not standard three-valued NULL
/// logic. `Node`/`Edge` compare by id (graph identity), not full-struct
/// contents — cheaper, and the correct semantics regardless (two bindings
/// are "the same node" iff the same node, not iff their label/prop
/// snapshots happen to match).
pub(crate) fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Null, _) | (_, Value::Null) => false,
        (Value::Property(pa), Value::Property(pb)) => pa == pb,
        (Value::Literal(la), Value::Literal(lb)) => la == lb,
        (Value::Property(pa), Value::Literal(lb)) => *pa == literal_to_value(lb),
        (Value::Literal(la), Value::Property(pb)) => literal_to_value(la) == *pb,
        (Value::Node(na), Value::Node(nb)) => na.id == nb.id,
        (Value::Edge(ea), Value::Edge(eb)) => ea.id == eb.id,
        (Value::List(la), Value::List(lb)) => la.len() == lb.len() && la.iter().zip(lb).all(|(x, y)| value_eq(x, y)),
        _ => false,
    }
}

fn call_builtin(name: &str, args: &[Value]) -> Result<Value, QueryError> {
    match name.to_ascii_lowercase().as_str() {
        "coalesce" => Ok(args
            .iter()
            .find(|v| !matches!(v, Value::Null))
            .cloned()
            .unwrap_or(Value::Null)),
        "tointeger" => Ok(args.first().map(to_integer).unwrap_or(Value::Null)),
        other => Err(QueryError::Parse(format!("unknown function: {other}"))),
    }
}

fn to_integer(v: &Value) -> Value {
    let as_str_parse = |s: &str| match s.trim().parse::<i64>() {
        Ok(i) => Value::Property(PropertyValue::Int(i)),
        Err(_) => Value::Null,
    };
    match v {
        Value::Property(PropertyValue::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Property(PropertyValue::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Property(PropertyValue::String(s)) => as_str_parse(s),
        Value::Literal(Literal::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Literal(Literal::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Literal(Literal::String(s)) => as_str_parse(s),
        _ => Value::Null,
    }
}

/// Sorts `rows` (already-projected `RETURN`/`WITH` output, `columns`
/// aligned by index) by `order_by`, which evaluates against the projected
/// column names — never the raw pattern `BindingRow` — since every ORDER BY
/// key in practice is a RETURN/WITH alias, not a bare pattern variable.
fn apply_order_by(
    rows: Vec<Vec<Value>>,
    columns: &[String],
    order_by: &[(ReturnExpr, SortDir)],
) -> Result<Vec<Vec<Value>>, QueryError> {
    let mut keyed: Vec<(Vec<Value>, Vec<Value>)> = Vec::with_capacity(rows.len());
    for row in rows {
        let row_map: HashMap<String, Value> = columns.iter().cloned().zip(row.iter().cloned()).collect();
        let keys = order_by
            .iter()
            .map(|(expr, _)| eval_projected_expr(expr, &row_map))
            .collect::<Result<Vec<_>, _>>()?;
        keyed.push((keys, row));
    }
    keyed.sort_by(|(ka, _), (kb, _)| {
        for (i, (_, dir)) in order_by.iter().enumerate() {
            let ord = compare_with_dir(&ka[i], &kb[i], *dir);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
    Ok(keyed.into_iter().map(|(_, row)| row).collect())
}

/// Same expression shape as `eval_return_expr`, but resolves `Var`/`Prop`
/// against already-projected output columns instead of the graph-bound
/// `BindingRow` — no `WriteTransaction`/`GraphStore` access needed, since a
/// projected `Value::Node`/`Value::Edge` already carries its full record
/// (including props) from when it was first materialized.
fn eval_projected_expr(expr: &ReturnExpr, row: &HashMap<String, Value>) -> Result<Value, QueryError> {
    match expr {
        ReturnExpr::Var(name) => row
            .get(name)
            .cloned()
            .ok_or_else(|| QueryError::UnboundVariable(name.clone())),
        ReturnExpr::Prop(pa) => {
            let base = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            let pv = match base {
                Value::Node(n) => n.props.get(&pa.prop).cloned(),
                Value::Edge(e) => e.props.get(&pa.prop).cloned(),
                _ => None,
            };
            Ok(match pv {
                Some(PropertyValue::Null) | None => Value::Null,
                Some(v) => Value::Property(v),
            })
        }
        ReturnExpr::Lit(lit) => Ok(match lit {
            Literal::Null => Value::Null,
            other => Value::Literal(other.clone()),
        }),
        ReturnExpr::Call { name, args, .. } => {
            // Same internal-consistency stance as `eval_return_expr`'s
            // `Call` arm: by the time ORDER BY runs, aggregation has
            // already resolved into ordinary named output columns
            // (referenced here via `Var`), so a raw aggregate `Call`
            // reaching this point means it wasn't top-level as
            // `validate_return_items` requires.
            if is_aggregate_name(name) {
                return Err(QueryError::Parse(format!(
                    "aggregate function '{name}' can only be used as a return item's top-level expression"
                )));
            }
            let arg_values = args
                .iter()
                .map(|a| eval_projected_expr(a, row))
                .collect::<Result<Vec<_>, _>>()?;
            call_builtin(name, &arg_values)
        }
        ReturnExpr::CountStar => Err(QueryError::Parse(
            "count(*) can only be used as a return item's top-level expression".into(),
        )),
        ReturnExpr::Case { test, whens, else_ } => {
            let test_value = match test {
                Some(t) => Some(eval_projected_expr(t, row)?),
                None => None,
            };
            for (when, then) in whens {
                let when_value = eval_projected_expr(when, row)?;
                let matched = match &test_value {
                    Some(tv) => value_eq(tv, &when_value),
                    None => matches!(when_value, Value::Literal(Literal::Bool(true))),
                };
                if matched {
                    return eval_projected_expr(then, row);
                }
            }
            match else_ {
                Some(e) => eval_projected_expr(e, row),
                None => Ok(Value::Null),
            }
        }
    }
}

/// NULLs sort last regardless of ASC/DESC (matches Neo4j's documented
/// behavior) — only non-null comparisons get reversed for DESC.
fn compare_with_dir(a: &Value, b: &Value, dir: SortDir) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let a_null = matches!(a, Value::Null);
    let b_null = matches!(b, Value::Null);
    match (a_null, b_null) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }
    let ord = compare_non_null(a, b);
    if dir == SortDir::Desc {
        ord.reverse()
    } else {
        ord
    }
}

fn compare_non_null(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let pa = value_to_comparable(a);
    let pb = value_to_comparable(b);
    match (pa, pb) {
        (Some(PropertyValue::Int(x)), Some(PropertyValue::Int(y))) => x.cmp(&y),
        (Some(PropertyValue::Int(x)), Some(PropertyValue::Float(y))) => {
            (x as f64).partial_cmp(&y).unwrap_or(Ordering::Equal)
        }
        (Some(PropertyValue::Float(x)), Some(PropertyValue::Int(y))) => {
            x.partial_cmp(&(y as f64)).unwrap_or(Ordering::Equal)
        }
        (Some(PropertyValue::Float(x)), Some(PropertyValue::Float(y))) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        (Some(PropertyValue::String(x)), Some(PropertyValue::String(y))) => x.cmp(&y),
        (Some(PropertyValue::Bool(x)), Some(PropertyValue::Bool(y))) => x.cmp(&y),
        _ => Ordering::Equal,
    }
}

fn value_to_comparable(v: &Value) -> Option<PropertyValue> {
    match v {
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(lit) => Some(literal_to_value(lit)),
        _ => None,
    }
}

/// Ordering for `min`/`max` aggregate folding — `None` for values with no
/// natural order (`Node`/`Edge`/`List`, or a `Null`, which `AggAcc::fold`
/// never passes here anyway since null contributions are skipped before
/// folding). The caller turns `None` into a clear error rather than an
/// arbitrary "always equal" fallback — unlike ORDER BY's
/// `compare_non_null`, which tolerates that for presentation ordering
/// (see its docs), silently treating two nodes as "equal" inside an
/// aggregate would be a wrong-answer failure mode, not just an
/// unhelpful sort order.
pub(crate) fn comparable_ordering(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    let pa = value_to_comparable(a)?;
    let pb = value_to_comparable(b)?;
    Some(match (pa, pb) {
        (PropertyValue::Int(x), PropertyValue::Int(y)) => x.cmp(&y),
        (PropertyValue::Int(x), PropertyValue::Float(y)) => (x as f64).partial_cmp(&y).unwrap_or(Ordering::Equal),
        (PropertyValue::Float(x), PropertyValue::Int(y)) => x.partial_cmp(&(y as f64)).unwrap_or(Ordering::Equal),
        (PropertyValue::Float(x), PropertyValue::Float(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        (PropertyValue::String(x), PropertyValue::String(y)) => x.cmp(&y),
        (PropertyValue::Bool(x), PropertyValue::Bool(y)) => x.cmp(&y),
        _ => return None,
    })
}
