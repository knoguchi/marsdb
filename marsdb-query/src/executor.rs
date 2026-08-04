use std::collections::{BTreeMap, HashMap, HashSet};

use marsdb_graph::{
    AdjEntry, Direction, EdgeId, GraphStore, NodeId, PropertyValue, Txn, WriteTransaction,
};

use crate::aggregate::{property_value_hash_key, value_hash_key, AggAcc, HashKey};
use crate::ast::{
    is_aggregate_name, ArithOp, CompareOp, Expr, Literal, MergeClause, NodePattern, Pattern,
    PropAccess, QuantifierKind, QueryClause, QueryPart, RelDirection, RemoveItem, ReturnExpr,
    ReturnItem, ReturnTail, SetItem, SortDir, Statement, Tail, UnwindClause, UnwindSource,
    WithClause, WithExpr,
};
use crate::error::QueryError;
use crate::ir::{ExpandDirection, LogicalPlan};
use crate::planner::{build_match_plan, pattern_all_vars, pattern_new_vars};
use crate::result::QueryResult;
use crate::temporal;
use crate::value::{PathElem, Value};

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
    /// `Value`s, not `Binding`s — `UNWIND` restores graph identity on the
    /// way back out via `value_to_binding_restore`, a separate step from
    /// how this is stored here.
    List(Vec<Value>),
    /// A map literal (`{a: 1, b: 2}`) carried through a `WITH` projection
    /// — same reasoning as `List`: `PropertyValue` has no map variant, so
    /// this is the only place a materialized map has to live between one
    /// `QueryPart` and the next.
    Map(BTreeMap<String, Value>),
    /// A named path (`p = (a)-->(b)`) or `shortestPath()` result — see
    /// `assemble_path`/`eval_shortest_path`. `PathBinding` (not `Binding`
    /// again) because a path element only ever needs graph identity
    /// (`NodeId`/`EdgeId`), never any of `Binding`'s other cases — using
    /// `Binding` itself here would make "a path containing a path" a type
    /// state nothing ever produces or handles.
    Path(Vec<PathBinding>),
}

/// One element of a `Binding::Path`, alternating node/edge/node/.../node
/// — the row-carried counterpart to `Value::Path`'s `PathElem` (which
/// carries full `Node`/`Edge` records instead of just their ids, the same
/// "keep identity in the row, resolve to a full record only when
/// materializing for display" split every other `Binding`/`Value` pair
/// already uses).
#[derive(Debug, Clone)]
enum PathBinding {
    Node(NodeId),
    Edge(EdgeId),
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

    fn execute_create(
        &self,
        write_txn: &WriteTransaction,
        patterns: &[Pattern],
    ) -> Result<QueryResult, QueryError> {
        // A standalone CREATE is a MATCH...CREATE tail run against a
        // single empty row -- `resolve_or_create_node` below never finds
        // any variable already bound in an empty `BindingRow`, so every
        // node token is fresh, exactly like standalone CREATE always was.
        // No trailing RETURN is possible on a standalone `CREATE` statement
        // (that's the `MATCH ... CREATE ... RETURN` tail's job instead), so
        // the resulting bindings are just discarded here.
        self.materialize_create(write_txn, patterns, &[BindingRow::new()])?;
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }

    /// Runs CREATE patterns once per row in `rows`, returning each row's
    /// bindings extended with whatever the CREATE patterns bound (newly
    /// created node/edge ids, or the reused id for an already-bound
    /// variable) -- this is what lets a trailing `RETURN` after a `MATCH
    /// ... CREATE` tail (e.g. `MATCH (a) CREATE (a)-[:R]->(b) RETURN b`)
    /// see the newly created `b`. Shared by a standalone `CREATE` statement
    /// (`execute_create`, a single empty row, return value discarded -- no
    /// RETURN is possible there) and a `MATCH ... CREATE` tail
    /// (`execute_match`, rows carry bindings from the preceding
    /// MATCH/WITH). The only real difference between the two is what
    /// `resolve_or_create_node` finds already bound in a row -- nothing for
    /// standalone CREATE, real nodes for a MATCH...CREATE tail, which is
    /// what lets the tail form add an edge between two nodes that already
    /// exist.
    fn materialize_create(
        &self,
        write_txn: &WriteTransaction,
        patterns: &[Pattern],
        rows: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            // A variable bound earlier in this same CREATE (an earlier hop,
            // or an earlier comma-separated pattern) must be visible to
            // later tokens naming it again -- e.g. a self-loop `(a)-[:R]->(a)`
            // -- so track newly-created bindings in a local, per-row copy
            // instead of just consulting the original incoming `row`.
            let mut row = row.clone();
            for pattern in patterns {
                let mut prev_id = self.resolve_or_create_node(write_txn, &pattern.start, &row)?;
                if let Some(var) = &pattern.start.var {
                    row.insert(var.clone(), Binding::Node(prev_id));
                }
                for (rel, node) in &pattern.hops {
                    if rel.hop_range.is_some() {
                        return Err(QueryError::Parse(
                            "CREATE doesn't support variable-length relationship patterns (e.g. [:TYPE*1..3])".into(),
                        ));
                    }
                    let node_id = self.resolve_or_create_node(write_txn, node, &row)?;
                    if let Some(var) = &node.var {
                        row.insert(var.clone(), Binding::Node(node_id));
                    }

                    let rel_label = rel.rel_type.clone().unwrap_or_else(|| "REL".to_string());
                    let rel_props =
                        self.eval_props_to_values(Txn::Write(write_txn), &rel.props, &row)?;
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
            out.push(row);
        }
        Ok(out)
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
        let labels: Vec<&str> = node.labels.iter().map(String::as_str).collect();
        let props = self.eval_props_to_values(Txn::Write(write_txn), &node.props, row)?;
        Ok(GraphStore::create_node_in_txn(write_txn, &labels, props)?)
    }

    /// Evaluates a CREATE pattern's `{...}` prop map -- each value is any
    /// `ReturnExpr` (`self.eval_return_expr`), not just a literal, which
    /// is what lets `CREATE (:Val {d: date({year: 1984, ...})})` work
    /// (see `cypher.pest`'s `map_expr` docs). `row` is whatever's already
    /// bound so far in this same CREATE (earlier hops, earlier
    /// comma-separated patterns) -- a prop expression referencing one of
    /// those (unusual, but not disallowed) resolves the same as anywhere
    /// else `eval_return_expr` runs.
    fn eval_props_to_values(
        &self,
        txn: Txn,
        props: &[(String, ReturnExpr)],
        row: &BindingRow,
    ) -> Result<BTreeMap<String, PropertyValue>, QueryError> {
        props
            .iter()
            .map(|(k, expr)| {
                let value = self.eval_return_expr(txn, expr, row)?;
                let pv = value_to_storable_property(&value).ok_or_else(|| {
                    QueryError::Parse(format!(
                        "property '{k}' can't be stored -- MarsDB's node/edge properties are limited to null/\
                         bool/int/float/string/date/duration; a list/map/node/edge/path value (got {value:?}) \
                         isn't storable, matching PropertyValue's real, deliberately fixed set of variants (see \
                         its doc comment)"
                    ))
                })?;
                Ok((k.clone(), pv))
            })
            .collect()
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
            return Ok(found
                .into_iter()
                .map(|r| tag_merge_created(r, false))
                .collect());
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
            let rel_props =
                self.eval_props_to_values(Txn::Write(write_txn), &rel.props, &new_row)?;
            let (src, dst) = match rel.direction {
                RelDirection::Right => (start_id, node_id),
                RelDirection::Left => (node_id, start_id),
                RelDirection::Either => return Err(QueryError::Parse(
                    "MERGE requires a directed relationship (-> or <-), not an undirected pattern"
                        .into(),
                )),
            };
            let edge_id =
                GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
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
                other => unreachable!(
                    "{MERGE_CREATED_KEY} tagged internally as Binding::Value(Bool), got {other:?}"
                ),
            };
            let items = if created {
                &clause.on_create
            } else {
                &clause.on_match
            };
            for item in items {
                apply_set_item(write_txn, row, item)?;
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
        // LIMIT push-down: when the *entire* statement is nothing but one
        // un-filtered, non-optional, single-node MATCH (no hops, no WHERE,
        // no WITH, at most the one label a NodeByLabelScan already narrows
        // by) feeding straight into a LIMIT with no ORDER BY, the scan
        // itself never needs to look past the first `limit` nodes -- there
        // is *nothing* downstream (no Filter/Expand/aggregation) that
        // could still drop a row, so capping the raw storage scan can't
        // change the result. Every more complex shape falls through to the
        // general path below unchanged, which doesn't short-circuit --
        // this executor materializes a `Vec<BindingRow>` at every step
        // rather than pulling lazily, so pushing LIMIT further (past a
        // Filter, an Expand, more than one clause, ...) would need a real
        // streaming executor to stay correct, not just a deeper check here.
        // A DISTINCT RETURN can also drop rows -- capping the raw scan at
        // `limit` before dedup could return fewer than `limit` *distinct*
        // rows even when more exist past what got scanned, so this shape
        // is excluded the same way a WHERE/Filter already is.
        let scan_limit_shortcut = order_by.is_none()
            && limit.is_some()
            && !tail_is_distinct_return(tail)
            && matches!(clauses, [QueryClause::Match(part)] if
                !part.shortest_path
                    && part.path_var.is_none()
                    && !part.optional
                    && part.with.is_none()
                    && part.pattern.hops.is_empty()
                    && part.where_clause.is_none()
                    && part.pattern.start.labels.len() <= 1
                    && part.pattern.start.props.is_empty()
                    && part.pattern.start.var.is_some());
        if scan_limit_shortcut {
            let [QueryClause::Match(part)] = clauses else {
                unreachable!("scan_limit_shortcut's own matches! already checked this shape");
            };
            let var = part
                .pattern
                .start
                .var
                .as_deref()
                .expect("checked by scan_limit_shortcut");
            let label = part.pattern.start.labels.first().map(String::as_str);
            let limit_usize = limit.expect("checked by scan_limit_shortcut").max(0) as usize;
            current_rows = self.scan(txn, var, label, &current_rows, Some(limit_usize))?;
        } else {
            for clause in clauses {
                match clause {
                    QueryClause::Match(part) => {
                        current_rows = if part.shortest_path {
                            // Not a LogicalPlan/eval_plan traversal at all —
                            // see eval_shortest_path's docs.
                            self.eval_shortest_path(txn, part, &current_rows)?
                        } else if let Some(path_var) = &part.path_var {
                            let (named_pattern, synthesized) = name_pattern_for_path(&part.pattern);
                            let plan = build_match_plan(
                                &named_pattern,
                                &part.where_clause,
                                &carried_vars,
                            )?;
                            let mut rows = if part.optional {
                                let new_vars = pattern_new_vars(&named_pattern, &carried_vars);
                                self.eval_optional_part(txn, &plan, &current_rows, &new_vars)?
                            } else {
                                self.eval_plan(txn, &plan, &current_rows)?
                            };
                            for row in &mut rows {
                                let path_binding = assemble_path(&named_pattern, row);
                                for key in &synthesized {
                                    row.remove(key);
                                }
                                row.insert(path_var.clone(), path_binding);
                            }
                            rows
                        } else {
                            let plan =
                                build_match_plan(&part.pattern, &part.where_clause, &carried_vars)?;
                            if part.optional {
                                let new_vars = pattern_new_vars(&part.pattern, &carried_vars);
                                self.eval_optional_part(txn, &plan, &current_rows, &new_vars)?
                            } else {
                                self.eval_plan(txn, &plan, &current_rows)?
                            }
                        };
                        let mut new_vars = pattern_all_vars(&part.pattern);
                        if let Some(path_var) = &part.path_var {
                            new_vars.insert(path_var.clone());
                        }
                        current_rows = self.apply_with_or_carry(
                            txn,
                            &part.with,
                            current_rows,
                            new_vars,
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
                    // A statement-leading WITH -- no pattern was matched, so
                    // there's nothing to seed `new_vars` with beyond what the
                    // WITH clause itself projects (`apply_with_or_carry`
                    // always takes the `Some(with)` branch here, never the
                    // "no WITH, just extend carried_vars" one, since `with` is
                    // always present on this variant by construction).
                    QueryClause::With(with) => {
                        current_rows = self.apply_with_or_carry(
                            txn,
                            &Some(with.clone()),
                            current_rows,
                            HashSet::new(),
                            &mut carried_vars,
                        )?;
                    }
                }
            }
        }
        // ORDER BY must see every matching row before LIMIT truncates —
        // sort, then take N, not the other way around. Only pre-truncate
        // (the v1 "doesn't short-circuit" path) when there's no ORDER BY to
        // invalidate it; DELETE/SET+LIMIT keep their "stop after N
        // bindings" behavior since they have no ORDER BY position in the
        // grammar. RETURN DISTINCT is excluded too, same reasoning as
        // ORDER BY: DISTINCT can still drop rows *after* this point, so
        // pre-truncating the raw input here could return fewer than
        // `limit` distinct rows even when more exist -- its LIMIT gets
        // applied after dedup instead, below.
        let distinct_return = tail_is_distinct_return(tail);
        if order_by.is_none() && !distinct_return {
            if let Some(count) = limit {
                current_rows.truncate(count.max(0) as usize);
            }
        }
        // Delete/Set need real `.insert`/`.remove`-capable write access,
        // not just `Txn`'s read-only `get`/`iter` — but they're only ever
        // reached via `Executor::execute`'s write-dispatch path (see
        // `is_read_only`), which always opens a `WriteTransaction`, so
        // `txn` is guaranteed to be `Txn::Write` here.
        // A non-aggregating RETURN's ORDER BY can reference either a
        // RETURN-introduced alias (`RETURN friend.id AS friendId ORDER BY
        // friendId`) or a variable still in scope that isn't returned at
        // all (`RETURN n.num AS prop ORDER BY n.num` — `n` itself never
        // appears in the RETURN list) — real Cypher allows both. Sorting
        // needs both the pre-projection bindings *and* the post-projection
        // output columns available at once, so it happens after
        // `materialize_return`, against a combined view of the two (see
        // `apply_order_by_with_scope`) rather than either alone. The
        // aggregating case can't use pre-projection bindings at all
        // (grouping has already collapsed the per-row bindings by then), so
        // it keeps sorting the post-projection output alone via
        // `apply_order_by`, further down.
        let mut order_by_pre_applied = false;
        let mut result = match tail {
            // A missing tail only ever occurs with a MERGE clause and
            // nothing after it — a pure write, same empty result shape
            // standalone CREATE already returns (not one blank row per
            // `current_rows`, which a synthetic `Tail::Return(vec![])`
            // would produce instead).
            None => QueryResult {
                columns: vec![],
                rows: vec![],
            },
            Some(Tail::Return(items, distinct)) => {
                let projected = self.materialize_return(txn, items, &current_rows, *distinct)?;
                if let Some(ob) = order_by {
                    // DISTINCT (like aggregation) can drop rows, breaking
                    // the 1:1 correspondence `apply_order_by_with_scope`
                    // needs between `current_rows` and the projected
                    // output -- ORDER BY after DISTINCT can only sort the
                    // post-projection, post-dedup result, same as the
                    // aggregating case just below.
                    if !has_aggregate(items) && !distinct {
                        order_by_pre_applied = true;
                        self.apply_order_by_with_scope(txn, &current_rows, projected, ob, limit)?
                    } else {
                        projected
                    }
                } else {
                    projected
                }
            }
            Some(Tail::Delete(vars, ret)) => {
                self.materialize_delete(txn, vars, &current_rows, false, ret)?
            }
            Some(Tail::DetachDelete(vars, ret)) => {
                self.materialize_delete(txn, vars, &current_rows, true, ret)?
            }
            Some(Tail::Set(items, ret)) => self.materialize_set(txn, items, &current_rows, ret)?,
            Some(Tail::Remove(items, ret)) => {
                self.materialize_remove(txn, items, &current_rows, ret)?
            }
            Some(Tail::Create(patterns, ret)) => {
                let updated_rows =
                    self.materialize_create(require_write_txn(txn), patterns, &current_rows)?;
                match ret {
                    Some(rt) => {
                        self.materialize_return(txn, &rt.items, &updated_rows, rt.distinct)?
                    }
                    None => QueryResult {
                        columns: vec![],
                        rows: vec![],
                    },
                }
            }
        };
        if let Some(order_by) = order_by {
            if !order_by_pre_applied {
                result.rows = apply_order_by(result.rows, &result.columns, order_by, limit)?;
            }
        } else if distinct_return {
            // The pre-truncate above was skipped for exactly this case --
            // apply LIMIT now, after materialize_return's dedup, instead.
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
            rows = self.apply_order_by_bindings(txn, rows, with_order_by, with.limit)?;
        } else if let Some(with_limit) = with.limit {
            rows.truncate(with_limit.max(0) as usize);
        }
        *carried_vars = with
            .items
            .iter()
            .enumerate()
            .map(with_item_output_name)
            .collect();
        Ok(rows)
    }

    /// `UNWIND`'s fan-out. Not a graph traversal — like `WITH`, handled
    /// directly here rather than through a `LogicalPlan`/`eval_plan` (see
    /// `UnwindClause`'s docs). Cross-joins each input row against every
    /// element of that row's resolved list, then applies the clause's own
    /// `WHERE`.
    fn eval_unwind(
        &self,
        txn: Txn,
        clause: &UnwindClause,
        rows: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in rows {
            let elements: Vec<Binding> = match &clause.source {
                UnwindSource::Var(name) => {
                    let binding = row
                        .get(name)
                        .ok_or_else(|| QueryError::UnboundVariable(name.clone()))?;
                    let Binding::List(items) = binding else {
                        return Err(QueryError::Parse(format!(
                            "'{name}' isn't a list — UNWIND needs a list (e.g. from collect())"
                        )));
                    };
                    items.iter().map(value_to_binding_restore).collect()
                }
                UnwindSource::List(literals) => literals
                    .iter()
                    .map(|lit| Binding::Value(literal_to_value(lit)))
                    .collect(),
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
                if self.eval_with_expr(txn, where_clause, &row)? == Some(true) {
                    filtered.push(row);
                }
            }
            out = filtered;
        }
        Ok(out)
    }

    /// `shortestPath((a)-[:TYPE*..N]-(b))` — a real parent-pointer BFS
    /// between two already-bound endpoints, not a `LogicalPlan`/
    /// `VarExpand` traversal (which only tracks final position plus a
    /// visited set, not the hop-by-hop chain a path needs to reconstruct).
    /// BFS visits in non-decreasing depth order, so the first time `b` is
    /// reached is *a* shortest path — stop there and reconstruct via
    /// parent pointers, rather than enumerating every path up to some
    /// bound the way `VarExpand` does.
    ///
    /// Both endpoints must already be bound by a preceding clause (e.g.
    /// `MATCH (a:Person{name:'Alice'}), (b:Person{name:'Bob'}) MATCH p =
    /// shortestPath((a)-[:KNOWS*]-(b)) RETURN p` — parser-enforced, see
    /// `parser::validate_shortest_path_pattern`) — v1 doesn't attempt to
    /// resolve a fresh/scanned endpoint here the way ordinary MATCH does,
    /// since "shortest path to *any* node matching these constraints" is a
    /// different, more ambiguous question than "shortest path between
    /// these two specific nodes."
    ///
    /// Every input row always survives (unlike an ordinary pattern match,
    /// which can produce zero rows for a non-match) — an unreachable pair
    /// binds the path variable to `Null`, same as `OPTIONAL MATCH`'s
    /// null-padding, rather than dropping the row. `part.optional` is
    /// therefore a no-op here, not separately handled. Exceeding the
    /// safety depth cap on an unbounded (`*..`) search also resolves to
    /// `Null`, not an error — unlike `VarExpand`'s cap (which errors,
    /// because truncating there would silently produce an *incomplete
    /// set* of paths, a wrong-answer risk), `shortestPath()` is only ever
    /// answering "is there a path within the searched horizon," which is
    /// a well-defined answer either way.
    fn eval_shortest_path(
        &self,
        txn: Txn,
        part: &QueryPart,
        rows: &[BindingRow],
    ) -> Result<Vec<BindingRow>, QueryError> {
        let Some(path_var) = &part.path_var else {
            // Nothing names the result, so there's nothing to bind and no
            // filtering effect (see this function's docs) — pure no-op.
            return Ok(rows.to_vec());
        };
        let start_var = part.pattern.start.var.as_deref().expect(
            "shortestPath()'s start node always has a var — validated at parse time by \
             validate_shortest_path_pattern",
        );
        let (rel, end_node) = &part.pattern.hops[0];
        let end_var = end_node.var.as_deref().expect(
            "shortestPath()'s end node always has a var — validated at parse time by \
             validate_shortest_path_pattern",
        );
        let (min_hops, max_hops) = rel.hop_range.expect(
            "shortestPath()'s relationship is always variable-length — validated at parse time by \
             validate_shortest_path_pattern",
        );
        let direction = match rel.direction {
            RelDirection::Right => ExpandDirection::Out,
            RelDirection::Left => ExpandDirection::In,
            RelDirection::Either => ExpandDirection::Either,
        };
        let rel_label = rel.rel_type.as_deref();

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let start_id = require_bound_node(row, start_var)?;
            let end_id = require_bound_node(row, end_var)?;
            let path = self.shortest_path_between(
                txn, start_id, end_id, direction, rel_label, min_hops, max_hops,
            )?;
            let mut new_row = row.clone();
            let binding = match path {
                Some(elems) => Binding::Path(elems),
                None => Binding::Value(PropertyValue::Null),
            };
            new_row.insert(path_var.clone(), binding);
            out.push(new_row);
        }
        if let Some(where_clause) = &part.where_clause {
            let mut filtered = Vec::with_capacity(out.len());
            for row in out {
                if self.eval_expr(txn, where_clause, &row)? == Some(true) {
                    filtered.push(row);
                }
            }
            out = filtered;
        }
        Ok(out)
    }

    /// The BFS itself. `min_hops` is only ever 0 or 1 (`validate_shortest_
    /// path_pattern` rejects anything higher) — deliberately: a plain
    /// visited-set BFS can't correctly answer "shortest path of at least N
    /// hops" for N > 1 (a node first reached at a too-early depth would
    /// need to stay revisitable for a later, longer route to it, which a
    /// visited-set structurally can't represent) without a different
    /// (node, depth)-keyed algorithm. Rejecting the case outright at parse
    /// time is safer than silently answering it wrong.
    fn shortest_path_between(
        &self,
        txn: Txn,
        start: NodeId,
        end: NodeId,
        direction: ExpandDirection,
        rel_label: Option<&str>,
        min_hops: u32,
        max_hops: Option<u32>,
    ) -> Result<Option<Vec<PathBinding>>, QueryError> {
        if start == end && min_hops == 0 {
            return Ok(Some(vec![PathBinding::Node(start)]));
        }
        let cap = max_hops.unwrap_or(VAR_EXPAND_DEPTH_CAP);
        let mut parent: HashMap<NodeId, (NodeId, EdgeId)> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        visited.insert(start);
        let mut frontier = vec![start];
        let mut depth = 0u32;
        while depth < cap && !frontier.is_empty() {
            depth += 1;
            let mut next_frontier = Vec::new();
            for node in frontier {
                for entry in neighbors_for_direction(txn, node, direction, rel_label)? {
                    if entry.other == end {
                        parent.insert(entry.other, (node, entry.edge_id));
                        return Ok(Some(reconstruct_path(&parent, start, end)));
                    }
                    if visited.insert(entry.other) {
                        parent.insert(entry.other, (node, entry.edge_id));
                        next_frontier.push(entry.other);
                    }
                }
            }
            frontier = next_frontier;
        }
        Ok(None)
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
                if self.eval_with_expr(txn, where_clause, &row)? == Some(true) {
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
    fn item_binding(
        &self,
        txn: Txn,
        expr: &ReturnExpr,
        row: &BindingRow,
    ) -> Result<Binding, QueryError> {
        match expr {
            ReturnExpr::Var(v) => row
                .get(v)
                .cloned()
                .ok_or_else(|| QueryError::UnboundVariable(v.clone())),
            other => {
                let value = self.eval_return_expr(txn, other, row)?;
                // `value_to_property_value` collapses Node/Edge/List/Path
                // to Null -- fine for a bare Var (handled above, never
                // reaches here) but wrong for any *wrapped* non-Var
                // expression that still evaluates to one of those (a list
                // literal/index/slice, or a CASE branch returning a bound
                // node/edge): those need the matching real Binding kind,
                // not a silently-nulled scalar. `Path` still falls back to
                // Null here -- a real, separate gap (needs a `Value::Path`
                // -> `Binding::Path` conversion this doesn't have yet),
                // not something any currently-reachable expression form
                // produces though.
                Ok(match value {
                    Value::Node(n) => Binding::Node(n.id),
                    Value::Edge(e) => Binding::Edge(e.id),
                    Value::List(items) => Binding::List(items),
                    Value::Map(m) => Binding::Map(m),
                    other => Binding::Value(value_to_property_value(&other)),
                })
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
        limit: Option<i64>,
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
        Ok(top_k_by(keyed, order_by, limit)
            .into_iter()
            .map(|(_, row)| row)
            .collect())
    }

    /// Sorts an already-`materialize_return`d result for a non-aggregating
    /// `RETURN`, evaluating each ORDER BY expression against *both* the
    /// pre-projection `BindingRow` it came from and its own projected
    /// output columns overlaid on top — real Cypher allows ORDER BY to
    /// reference either a RETURN alias or a still-in-scope variable that
    /// wasn't returned at all, so neither view alone is enough (see the
    /// call site in `execute_match`). `binding_rows` and `result.rows` are
    /// the same length and pairwise correspond — `materialize_return`'s
    /// non-aggregating path preserves row order 1:1 with its input.
    fn apply_order_by_with_scope(
        &self,
        txn: Txn,
        binding_rows: &[BindingRow],
        result: QueryResult,
        order_by: &[(ReturnExpr, SortDir)],
        limit: Option<i64>,
    ) -> Result<QueryResult, QueryError> {
        let QueryResult { columns, rows } = result;
        let mut keyed: Vec<(Vec<Value>, Vec<Value>)> = Vec::with_capacity(rows.len());
        for (binding_row, row) in binding_rows.iter().zip(rows) {
            let mut value_map = self.binding_row_to_value_map(txn, binding_row)?;
            for (col, val) in columns.iter().zip(&row) {
                value_map.insert(col.clone(), val.clone());
            }
            let keys = order_by
                .iter()
                .map(|(expr, _)| eval_projected_expr(expr, &value_map))
                .collect::<Result<Vec<_>, _>>()?;
            keyed.push((keys, row));
        }
        let rows = top_k_by(keyed, order_by, limit)
            .into_iter()
            .map(|(_, row)| row)
            .collect();
        Ok(QueryResult { columns, rows })
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
            Binding::Node(id) => Value::Node(deleted_entity_access(GraphStore::get_node_in_txn(
                txn, *id,
            )?)?),
            Binding::Edge(id) => Value::Edge(deleted_entity_access(GraphStore::get_edge_in_txn(
                txn, *id,
            )?)?),
            Binding::Value(PropertyValue::Null) => Value::Null,
            Binding::Value(pv) => Value::Property(pv.clone()),
            Binding::List(items) => Value::List(items.clone()),
            Binding::Map(m) => Value::Map(m.clone()),
            Binding::Path(elems) => Value::Path(self.resolve_path_elems(txn, elems)?),
        })
    }

    /// `binding_to_value`'s per-element helper for `Binding::Path` — fetches
    /// each element's full current record, same "keep just the id in the
    /// row, resolve to a full record only when materializing for display"
    /// split `Binding::Node`/`Edge` already use above.
    fn resolve_path_elems(
        &self,
        txn: Txn,
        elems: &[PathBinding],
    ) -> Result<Vec<PathElem>, QueryError> {
        elems
            .iter()
            .map(|e| {
                Ok(match e {
                    PathBinding::Node(id) => PathElem::Node(deleted_entity_access(
                        GraphStore::get_node_in_txn(txn, *id)?,
                    )?),
                    PathBinding::Edge(id) => PathElem::Edge(deleted_entity_access(
                        GraphStore::get_edge_in_txn(txn, *id)?,
                    )?),
                })
            })
            .collect()
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
            let hash_key: Vec<Option<HashKey>> = key_bindings
                .iter()
                .map(|b| b.as_ref().map(binding_hash_key).transpose())
                .collect::<Result<Vec<_>, _>>()?;
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
                let ReturnExpr::Call { args, .. } = &item.expr else {
                    continue;
                };
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
                    group.key_bindings[i]
                        .clone()
                        .expect("non-aggregate item must have a key binding")
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
    /// `Option<bool>` — `None` is Cypher's "unknown" (see `compare()`'s
    /// docs), propagated through `AND`/`OR`/`NOT` via `and3`/`or3`/`map`
    /// instead of collapsing to `false` partway through. Every call site
    /// filters a row by checking `== Some(true)` — unknown behaves like
    /// `false` for filtering purposes, but *only* at that final step, not
    /// internally, since `AND`/`OR`'s truth tables need to tell "false"
    /// and "unknown" apart to combine correctly.
    fn eval_with_expr(
        &self,
        txn: Txn,
        expr: &WithExpr,
        row: &BindingRow,
    ) -> Result<Option<bool>, QueryError> {
        Ok(match expr {
            WithExpr::And(l, r) => and3(
                self.eval_with_expr(txn, l, row)?,
                self.eval_with_expr(txn, r, row)?,
            ),
            WithExpr::Or(l, r) => or3(
                self.eval_with_expr(txn, l, row)?,
                self.eval_with_expr(txn, r, row)?,
            ),
            WithExpr::Not(e) => self.eval_with_expr(txn, e, row)?.map(|b| !b),
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
                r.insert(
                    OPTIONAL_SEED_IDX_KEY.to_string(),
                    Binding::Value(PropertyValue::Int(i as i64)),
                );
                r
            })
            .collect();
        let results = self.eval_plan(txn, plan, &tagged)?;
        let mut by_idx: HashMap<i64, Vec<BindingRow>> = HashMap::new();
        for mut row in results {
            let idx = match row.remove(OPTIONAL_SEED_IDX_KEY) {
                Some(Binding::Value(PropertyValue::Int(i))) => i,
                other => unreachable!(
                    "__seed_idx tagged internally as Binding::Value(Int), got {other:?}"
                ),
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
            LogicalPlan::AllNodesScan { var } => self.scan(txn, var, None, seed, None),
            LogicalPlan::NodeByLabelScan { var, label } => {
                self.scan(txn, var, Some(label), seed, None)
            }
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
                    let from_id = match row.get(from_var) {
                        Some(Binding::Node(id)) => *id,
                        // A null `from_var` (padded by an outer, already-
                        // resolved `OPTIONAL MATCH` that didn't match) has
                        // no neighbors, same as any other traversal from
                        // null -- contributes zero rows, not an error. A
                        // truly missing/wrong-typed binding still is one.
                        Some(Binding::Value(PropertyValue::Null)) => continue,
                        _ => return Err(QueryError::UnboundVariable(from_var.clone())),
                    };
                    let entries =
                        neighbors_for_direction(txn, from_id, *direction, rel_label.as_deref())?;
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
                    let start_id = match row.get(from_var) {
                        Some(Binding::Node(id)) => *id,
                        // Same null-propagation as `Expand` above.
                        Some(Binding::Value(PropertyValue::Null)) => continue,
                        _ => return Err(QueryError::UnboundVariable(from_var.clone())),
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
                            let entries = neighbors_for_direction(
                                txn,
                                node,
                                *direction,
                                rel_label.as_deref(),
                            )?;
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
                    if self.eval_expr(txn, predicate, &row)? == Some(true) {
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
    /// `row_limit` bounds the underlying storage scan itself (see
    /// `GraphStore::all_nodes_limited_in_txn`) -- only ever `Some` from the
    /// dedicated shortcut in `execute_match` for a plan that's *just* this
    /// one scan feeding straight into `LIMIT`, nothing else (no `Filter`,
    /// no `Expand`, no `ORDER BY`). Every other caller (the general
    /// `eval_plan` recursion) passes `None`, since capping the raw scan is
    /// only safe when nothing downstream could still drop a row.
    fn scan(
        &self,
        txn: Txn,
        var: &str,
        label: Option<&str>,
        seed: &[BindingRow],
        row_limit: Option<usize>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let nodes = match row_limit {
            Some(limit) => GraphStore::all_nodes_limited_in_txn(txn, label, limit)?,
            None => GraphStore::all_nodes_in_txn(txn, label)?,
        };
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

    /// `Option<bool>` — see `eval_with_expr`'s docs, same reasoning.
    /// `HasLabel`/`VarEq` never produce "unknown" (they operate on real
    /// bound node/edge identity, not a possibly-null property), so they
    /// always return `Some`.
    fn eval_expr(
        &self,
        txn: Txn,
        expr: &Expr,
        row: &BindingRow,
    ) -> Result<Option<bool>, QueryError> {
        Ok(match expr {
            Expr::And(l, r) => and3(self.eval_expr(txn, l, row)?, self.eval_expr(txn, r, row)?),
            Expr::Or(l, r) => or3(self.eval_expr(txn, l, row)?, self.eval_expr(txn, r, row)?),
            Expr::Not(e) => self.eval_expr(txn, e, row)?.map(|b| !b),
            Expr::Compare(pa, op, lit) => {
                let prop_value = self.lookup_prop(txn, pa, row)?;
                compare(&prop_value, *op, lit)
            }
            // Always definite -- that's the whole point of IS NULL, so
            // this is the one `Expr` leaf that's always `Some`, same as
            // `HasLabel`/`VarEq` below.
            Expr::IsNull(pa) => Some(matches!(
                self.lookup_prop(txn, pa, row)?,
                None | Some(PropertyValue::Null)
            )),
            Expr::HasLabel(var, label) => {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                let Binding::Node(id) = binding else {
                    return Err(QueryError::UnboundVariable(var.clone()));
                };
                let node = GraphStore::get_node_in_txn(txn, *id)?;
                Some(node.is_some_and(|n| n.labels.iter().any(|l| l == label)))
            }
            Expr::VarEq(a, b) => {
                let ba = row
                    .get(a)
                    .ok_or_else(|| QueryError::UnboundVariable(a.clone()))?;
                let bb = row
                    .get(b)
                    .ok_or_else(|| QueryError::UnboundVariable(b.clone()))?;
                Some(match (ba, bb) {
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
                })
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
            // A missing *property key* on an existing node/edge is a real,
            // legal "absent" (-> null downstream) -- but a missing
            // *node/edge record* means it was deleted earlier in this same
            // statement (`deleted_entity_access`'s docs), which is a real
            // error (`MATCH (n) DELETE n RETURN n.num` -- TCK's Return2
            // scenario [15]), not a silent null. These are two different
            // kinds of "missing" and must not be collapsed into one.
            Binding::Node(id) => {
                let node = deleted_entity_access(GraphStore::get_node_in_txn(txn, *id)?)?;
                Ok(node.props.get(&pa.prop).cloned())
            }
            Binding::Edge(id) => {
                let edge = deleted_entity_access(GraphStore::get_edge_in_txn(txn, *id)?)?;
                Ok(edge.props.get(&pa.prop).cloned())
            }
            // A WITH-projected scalar (or list/map/path) has no scalar
            // `.prop` to access via this path — e.g. `WITH message.id AS
            // messageId` then `messageId.foo` isn't meaningful. Treat as
            // absent rather than erroring, consistent with how a missing
            // property already behaves. `Binding::Map` specifically *does*
            // have real `.prop` access, just not through this method (its
            // values aren't always a scalar `PropertyValue`) — see
            // `lookup_prop_value`, which `ReturnExpr::Prop` actually calls.
            // A `Binding::Value` holding a `Date`/`Duration` also has real
            // `.prop` access (`d.year`, etc) — also handled there, not
            // here, for the same "not always a scalar `PropertyValue`"
            // reason (well, it always *is* one here, but `lookup_prop_value`
            // is where that access actually happens either way).
            Binding::Value(_) | Binding::List(_) | Binding::Map(_) | Binding::Path(_) => Ok(None),
        }
    }

    /// `ReturnExpr::Prop`'s own lookup -- unlike `lookup_prop` (used by
    /// pattern-level `WHERE`, which only ever compares a real node/edge
    /// property against a `Literal`), a map's value can be any `Value`
    /// shape (nested list/map/node), not just a scalar `PropertyValue`,
    /// so this returns the wider type and handles `Binding::Map` itself
    /// rather than collapsing through `lookup_prop`. A `Binding::Value`
    /// holding a `Date`/`Duration` is handled here too, for the same
    /// reason -- `d.year`/`d.months`/etc are real component accessors
    /// (Temporal5's whole scenario shape, `WITH v.date AS d ... RETURN
    /// d.year`), not a stored property `lookup_prop` could ever find.
    fn lookup_prop_value(
        &self,
        txn: Txn,
        pa: &PropAccess,
        row: &BindingRow,
    ) -> Result<Value, QueryError> {
        match row.get(&pa.var) {
            Some(Binding::Map(m)) => Ok(m.get(&pa.prop).cloned().unwrap_or(Value::Null)),
            Some(Binding::Value(pv)) => Ok(match temporal_component(pv, &pa.prop) {
                Some(component) => Value::Property(component),
                None => Value::Null,
            }),
            Some(_) => Ok(match self.lookup_prop(txn, pa, row)? {
                Some(PropertyValue::Null) | None => Value::Null,
                Some(pv) => Value::Property(pv),
            }),
            None => Err(QueryError::UnboundVariable(pa.var.clone())),
        }
    }

    fn materialize_return(
        &self,
        txn: Txn,
        items: &[ReturnItem],
        rows: &[BindingRow],
        distinct: bool,
    ) -> Result<QueryResult, QueryError> {
        let columns = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                item.alias
                    .clone()
                    .unwrap_or_else(|| default_column_name(&item.expr, i))
            })
            .collect();
        let mut out_rows = if !has_aggregate(items) {
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
        if distinct {
            out_rows = dedup_rows(out_rows)?;
        }
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
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                self.binding_to_value(txn, binding)
            }
            ReturnExpr::Prop(pa) => self.lookup_prop_value(txn, pa, row),
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
            ReturnExpr::Arith(l, op, r) => {
                let lv = self.eval_return_expr(txn, l, row)?;
                let rv = self.eval_return_expr(txn, r, row)?;
                apply_arith(*op, &lv, &rv)
            }
            ReturnExpr::ListLit(items) => Ok(Value::List(
                items
                    .iter()
                    .map(|item| self.eval_return_expr(txn, item, row))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            ReturnExpr::Index(base, index) => {
                let base_v = self.eval_return_expr(txn, base, row)?;
                let index_v = self.eval_return_expr(txn, index, row)?;
                apply_index(&base_v, &index_v)
            }
            ReturnExpr::Slice(base, start, end) => {
                let base_v = self.eval_return_expr(txn, base, row)?;
                let start_v = start
                    .as_deref()
                    .map(|s| self.eval_return_expr(txn, s, row))
                    .transpose()?;
                let end_v = end
                    .as_deref()
                    .map(|e| self.eval_return_expr(txn, e, row))
                    .transpose()?;
                apply_slice(&base_v, start_v.as_ref(), end_v.as_ref())
            }
            ReturnExpr::ListComp {
                var,
                source,
                where_clause,
                project,
            } => {
                let source_v = self.eval_return_expr(txn, source, row)?;
                let items = match source_v {
                    Value::List(items) => items,
                    Value::Null => return Ok(Value::Null),
                    other => {
                        return Err(QueryError::Parse(format!(
                            "list comprehension source must be a list, got {other:?}"
                        )))
                    }
                };
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    // A fresh overlay per element -- `var` shadows any
                    // outer binding of the same name for the duration of
                    // this one element, same scoping UNWIND already uses.
                    let mut scoped_row = row.clone();
                    scoped_row.insert(var.clone(), value_to_binding_restore(&item));
                    let keep = match where_clause {
                        Some(w) => self.eval_return_expr_bool3(txn, w, &scoped_row)? == Some(true),
                        None => true,
                    };
                    if !keep {
                        continue;
                    }
                    result.push(match project {
                        Some(p) => self.eval_return_expr(txn, p, &scoped_row)?,
                        None => item,
                    });
                }
                Ok(Value::List(result))
            }
            ReturnExpr::Quantifier {
                kind,
                var,
                source,
                where_clause,
            } => {
                let source_v = self.eval_return_expr(txn, source, row)?;
                let items = match source_v {
                    Value::List(items) => items,
                    Value::Null => return Ok(Value::Null),
                    other => {
                        return Err(QueryError::Parse(format!(
                            "quantifier source must be a list, got {other:?}"
                        )))
                    }
                };
                let mut preds = Vec::with_capacity(items.len());
                for item in &items {
                    let mut scoped_row = row.clone();
                    scoped_row.insert(var.clone(), value_to_binding_restore(item));
                    preds.push(match where_clause {
                        Some(w) => self.eval_return_expr_bool3(txn, w, &scoped_row)?,
                        None => item_truthy(item),
                    });
                }
                Ok(match eval_quantifier(*kind, &preds) {
                    Some(b) => Value::Literal(Literal::Bool(b)),
                    None => Value::Null,
                })
            }
            ReturnExpr::MapLit(entries) => {
                let mut map = BTreeMap::new();
                for (k, v) in entries {
                    map.insert(k.clone(), self.eval_return_expr(txn, v, row)?);
                }
                Ok(Value::Map(map))
            }
            ReturnExpr::And(l, r) => Ok(bool3_to_value(and3(
                self.eval_return_expr_bool3(txn, l, row)?,
                self.eval_return_expr_bool3(txn, r, row)?,
            ))),
            ReturnExpr::Or(l, r) => Ok(bool3_to_value(or3(
                self.eval_return_expr_bool3(txn, l, row)?,
                self.eval_return_expr_bool3(txn, r, row)?,
            ))),
            ReturnExpr::Xor(l, r) => Ok(bool3_to_value(xor3(
                self.eval_return_expr_bool3(txn, l, row)?,
                self.eval_return_expr_bool3(txn, r, row)?,
            ))),
            ReturnExpr::Not(e) => Ok(bool3_to_value(
                self.eval_return_expr_bool3(txn, e, row)?.map(|b| !b),
            )),
            ReturnExpr::Compare(l, op, r) => {
                let lv = self.eval_return_expr(txn, l, row)?;
                let rv = self.eval_return_expr(txn, r, row)?;
                Ok(bool3_to_value(compare_values(&lv, *op, &rv)))
            }
            ReturnExpr::IsNull(e) => {
                let v = self.eval_return_expr(txn, e, row)?;
                Ok(Value::Literal(Literal::Bool(matches!(v, Value::Null))))
            }
        }
    }

    /// A `WHERE`-position `ReturnExpr` (list comprehension/quantifier
    /// filters) evaluated as three-valued logic instead of a plain
    /// `Value` -- delegates to `eval_return_expr` then folds the result
    /// down via `value_to_bool3`.
    fn eval_return_expr_bool3(
        &self,
        txn: Txn,
        expr: &ReturnExpr,
        row: &BindingRow,
    ) -> Result<Option<bool>, QueryError> {
        value_to_bool3(&self.eval_return_expr(txn, expr, row)?)
    }

    /// `ret`, when present, is evaluated *after* the physical delete runs,
    /// not before — real Cypher's own DELETE+RETURN TCK scenarios agree on
    /// this ordering: `MATCH (n) DELETE n RETURN n.num` must raise a
    /// `DeletedEntityAccess` error (TCK's Return2 scenarios [15]/[17]), not
    /// silently return the pre-delete value. `lookup_prop`/
    /// `binding_to_value` (via `deleted_entity_access`) already turn "the
    /// bound id's record is gone" into a proper `QueryError` rather than a
    /// silent null or a panic, which is exactly what makes deleting first
    /// safe here — every other real DELETE+RETURN shape (`count(*)`,
    /// `sum(num)` off a WITH-projected scalar, a literal, a null OPTIONAL
    /// MATCH binding) never touches the just-deleted entity's live record
    /// at all, so this ordering changes nothing for them.
    fn materialize_delete(
        &self,
        txn: Txn,
        vars: &[String],
        rows: &[BindingRow],
        detach: bool,
        ret: &Option<ReturnTail>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        let mut deleted_nodes = HashSet::new();
        let mut deleted_edges = HashSet::new();
        for row in rows {
            for var in vars {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
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
                    // A null binding is a real, legal DELETE target -- an
                    // `OPTIONAL MATCH` that didn't match pads its variables
                    // with null, and deleting that null is specified as a
                    // silent no-op, not an error (real Cypher: "deleting
                    // null does nothing").
                    Binding::Value(PropertyValue::Null) => {}
                    Binding::Value(_) | Binding::List(_) | Binding::Map(_) | Binding::Path(_) => {
                        return Err(QueryError::UnboundVariable(format!(
                            "'{var}' is a WITH-projected scalar, not a node/edge — DELETE needs a graph binding"
                        )))
                    }
                }
            }
        }
        let result = match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct)?,
            None => QueryResult {
                columns: vec![],
                rows: vec![],
            },
        };
        Ok(result)
    }

    fn materialize_set(
        &self,
        txn: Txn,
        items: &[SetItem],
        rows: &[BindingRow],
        ret: &Option<ReturnTail>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        for row in rows {
            for item in items {
                apply_set_item(write_txn, row, item)?;
            }
        }
        match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct),
            None => Ok(QueryResult {
                columns: vec![],
                rows: vec![],
            }),
        }
    }

    fn materialize_remove(
        &self,
        txn: Txn,
        items: &[RemoveItem],
        rows: &[BindingRow],
        ret: &Option<ReturnTail>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        for row in rows {
            for item in items {
                apply_remove_item(write_txn, row, item)?;
            }
        }
        match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct),
            None => Ok(QueryResult {
                columns: vec![],
                rows: vec![],
            }),
        }
    }
}

fn apply_set_item(
    write_txn: &WriteTransaction,
    row: &BindingRow,
    item: &SetItem,
) -> Result<(), QueryError> {
    match item {
        // `SET n.prop = null` *removes* the property in real Cypher (found
        // via TCK's Set2 "Set a Property to Null" scenarios, which this
        // codebase previously couldn't parse at all -- `SET` had no
        // trailing RETURN to observe the result with, so this bug was
        // never exercised until that gap closed). Storing a literal
        // `PropertyValue::Null` instead is observably different: `n.prop`
        // still shows up as a (nulled-out) key when a caller enumerates a
        // node's own props (e.g. this RETURN's own node-to-string
        // rendering), where a real missing property wouldn't.
        SetItem::Prop(pa, Literal::Null) => {
            let binding = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            match binding {
                Binding::Node(id) => {
                    GraphStore::remove_node_prop_in_txn(write_txn, *id, &pa.prop)?;
                }
                Binding::Edge(id) => {
                    GraphStore::remove_edge_prop_in_txn(write_txn, *id, &pa.prop)?;
                }
                // `SET` on a null binding is a documented no-op, same as
                // `DELETE`/`REMOVE` on one -- an `OPTIONAL MATCH` that
                // found nothing pads its variables with null (found via
                // TCK's Set1/Set3 "Ignore null when setting
                // property/label" scenarios; same "previously unparseable
                // without a trailing RETURN to observe it" story as the
                // null-removes-property bug just above).
                Binding::Value(PropertyValue::Null) => {}
                Binding::Value(_) | Binding::List(_) | Binding::Map(_) | Binding::Path(_) => {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding",
                        pa.var
                    )))
                }
            }
        }
        SetItem::Prop(pa, lit) => {
            let binding = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            let value = literal_to_value(lit);
            match binding {
                Binding::Node(id) => {
                    GraphStore::set_node_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                }
                Binding::Edge(id) => {
                    GraphStore::set_edge_prop_in_txn(write_txn, *id, &pa.prop, value)?;
                }
                Binding::Value(PropertyValue::Null) => {}
                Binding::Value(_) | Binding::List(_) | Binding::Map(_) | Binding::Path(_) => {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding",
                        pa.var
                    )))
                }
            }
        }
        SetItem::Labels(var, labels) => {
            let binding = row
                .get(var)
                .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
            match binding {
                Binding::Node(id) => {
                    for label in labels {
                        GraphStore::add_node_label_in_txn(write_txn, *id, label)?;
                    }
                }
                // Same null-is-a-no-op rule as the property arm above.
                Binding::Value(PropertyValue::Null) => {}
                _ => {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{var}' isn't a node — SET can only add labels to a node"
                    )))
                }
            }
        }
    }
    Ok(())
}

fn apply_remove_item(
    write_txn: &WriteTransaction,
    row: &BindingRow,
    item: &RemoveItem,
) -> Result<(), QueryError> {
    match item {
        RemoveItem::Prop(pa) => {
            let binding = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            match binding {
                Binding::Node(id) => {
                    GraphStore::remove_node_prop_in_txn(write_txn, *id, &pa.prop)?;
                }
                Binding::Edge(id) => {
                    GraphStore::remove_edge_prop_in_txn(write_txn, *id, &pa.prop)?;
                }
                // Same null-is-a-no-op rule DELETE already follows (found
                // via TCK's Remove1 "Ignore null when removing property"
                // scenarios).
                Binding::Value(PropertyValue::Null) => {}
                Binding::Value(_) | Binding::List(_) | Binding::Map(_) | Binding::Path(_) => {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{}' is a WITH-projected scalar, not a node/edge — REMOVE needs a graph binding",
                        pa.var
                    )))
                }
            }
        }
        RemoveItem::Labels(var, labels) => {
            let binding = row
                .get(var)
                .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
            match binding {
                Binding::Node(id) => {
                    for label in labels {
                        GraphStore::remove_node_label_in_txn(write_txn, *id, label)?;
                    }
                }
                // Same null-is-a-no-op rule as the property arm above
                // (found via TCK's Remove2 "Ignore null when removing a
                // node label" scenario).
                Binding::Value(PropertyValue::Null) => {}
                _ => {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{var}' isn't a node — REMOVE can only remove labels from a node"
                    )))
                }
            }
        }
    }
    Ok(())
}

/// Whether `tail`'s ultimate RETURN (if it has one at all -- either
/// `Tail::Return` itself, or a mutating tail's trailing `ReturnTail`) is a
/// `RETURN DISTINCT`. Used by `execute_match`'s LIMIT pre-truncate and
/// scan-limit-pushdown shortcuts, both of which must NOT fire for a
/// DISTINCT return -- dedup can drop rows, so capping the raw input at
/// `limit` before it runs could return fewer than `limit` distinct rows
/// even when more exist.
fn tail_is_distinct_return(tail: &Option<Tail>) -> bool {
    match tail {
        Some(Tail::Return(_, distinct)) => *distinct,
        Some(Tail::Delete(_, ret))
        | Some(Tail::DetachDelete(_, ret))
        | Some(Tail::Set(_, ret))
        | Some(Tail::Remove(_, ret))
        | Some(Tail::Create(_, ret)) => ret.as_ref().is_some_and(|rt| rt.distinct),
        None => false,
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
    let Statement::Match {
        tail: Some(Tail::Return(_, _)),
        clauses,
        ..
    } = stmt
    else {
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
        ReturnExpr::Arith(..) => format!("col{idx}"),
        ReturnExpr::ListLit(..)
        | ReturnExpr::Index(..)
        | ReturnExpr::Slice(..)
        | ReturnExpr::ListComp { .. }
        | ReturnExpr::Quantifier { .. }
        | ReturnExpr::MapLit(..)
        | ReturnExpr::And(..)
        | ReturnExpr::Or(..)
        | ReturnExpr::Xor(..)
        | ReturnExpr::Not(..)
        | ReturnExpr::Compare(..)
        | ReturnExpr::IsNull(..) => format!("col{idx}"),
    }
}

/// The name a `WITH`/`RETURN` item is known by afterward — its alias, or
/// a name derived from the expression (its bare var name, `col{i}`, etc).
fn with_item_output_name((i, item): (usize, &ReturnItem)) -> String {
    item.alias
        .clone()
        .unwrap_or_else(|| default_column_name(&item.expr, i))
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
        ReturnExpr::Call { name, args, .. } => {
            is_aggregate_name(name) || args.iter().any(contains_aggregate)
        }
        ReturnExpr::Case { test, whens, else_ } => {
            test.as_deref().is_some_and(contains_aggregate)
                || whens
                    .iter()
                    .any(|(w, t)| contains_aggregate(w) || contains_aggregate(t))
                || else_.as_deref().is_some_and(contains_aggregate)
        }
        ReturnExpr::Arith(l, _, r) => contains_aggregate(l) || contains_aggregate(r),
        ReturnExpr::ListLit(items) => items.iter().any(contains_aggregate),
        ReturnExpr::Index(base, index) => contains_aggregate(base) || contains_aggregate(index),
        ReturnExpr::Slice(base, start, end) => {
            contains_aggregate(base)
                || start.as_deref().is_some_and(contains_aggregate)
                || end.as_deref().is_some_and(contains_aggregate)
        }
        // `where_clause` isn't checked -- same scope limitation as
        // `UnwindClause`'s own filter, which never routes through this
        // check either; the source/project halves are the ones a real
        // TCK scenario nests an aggregate in (`size([x IN collect(r) ...])`).
        ReturnExpr::ListComp {
            source, project, ..
        } => contains_aggregate(source) || project.as_deref().is_some_and(contains_aggregate),
        ReturnExpr::Quantifier { source, .. } => contains_aggregate(source),
        ReturnExpr::MapLit(entries) => entries.iter().any(|(_, v)| contains_aggregate(v)),
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            contains_aggregate(l) || contains_aggregate(r)
        }
        ReturnExpr::Not(e) => contains_aggregate(e),
        ReturnExpr::Compare(l, _, r) => contains_aggregate(l) || contains_aggregate(r),
        ReturnExpr::IsNull(e) => contains_aggregate(e),
        ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::Lit(_) => false,
    }
}

/// True iff any item's top-level expression is an aggregate call —
/// `materialize_with`/`materialize_return` dispatch to the grouping path
/// iff this is true, otherwise the existing row-at-a-time path runs
/// completely unchanged (zero perf/behavior impact on non-aggregating
/// queries).
fn has_aggregate(items: &[ReturnItem]) -> bool {
    // `contains_aggregate`, not `is_top_level_aggregate` -- an aggregate
    // nested inside a wrapping expression (`1 + count(x)`, now parseable
    // since ReturnExpr::Arith exists) still needs to route to the
    // grouping path so `validate_return_items` gets a chance to reject it
    // with a clear error. With the narrower top-level-only check, such a
    // query silently took the ordinary per-row path instead (iterating
    // `rows` directly, which is empty for an empty MATCH) and produced
    // the wrong row count instead of erroring -- a real bug this exact
    // widening fixed, not just future-proofing.
    items.iter().any(|item| contains_aggregate(&item.expr))
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
fn binding_hash_key(b: &Binding) -> Result<HashKey, QueryError> {
    Ok(match b {
        Binding::Node(id) => HashKey::Node(*id),
        Binding::Edge(id) => HashKey::Edge(*id),
        Binding::Value(pv) => property_value_hash_key(pv),
        Binding::List(items) => HashKey::List(items.iter().map(value_hash_key).collect::<Result<Vec<_>, _>>()?),
        // Explicit error, not a silent hash-by-something-arbitrary —
        // grouping/collecting by a captured path isn't a case any real
        // usage needs, and this codebase's stance is to reject an
        // untested shape rather than guess at its semantics.
        Binding::Path(_) => {
            return Err(QueryError::Parse(
                "grouping or collecting by a path (e.g. a named-path/shortestPath() variable) isn't supported"
                    .into(),
            ))
        }
        // Same stance as `Path` above -- see `value_hash_key`'s matching
        // `Value::Map` arm.
        Binding::Map(_) => {
            return Err(QueryError::Parse(
                "grouping or using DISTINCT with a map value isn't supported".into(),
            ))
        }
    })
}

/// Converts a finished `AggAcc::finish()` result to the `Binding` it's
/// carried as through a `WITH` boundary — `collect()`'s `Value::List`
/// needs `Binding::List` (no list variant in `PropertyValue`, the
/// storage-layer type `Binding::Value` wraps), everything else collapses
/// to `Binding::Value` same as any other computed WITH item.
fn value_to_binding(v: Value) -> Binding {
    match v {
        Value::List(items) => Binding::List(items),
        Value::Map(m) => Binding::Map(m),
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
        Value::Map(m) => Binding::Map(m.clone()),
        Value::Path(elems) => Binding::Path(elems.iter().map(path_elem_to_binding).collect()),
        Value::Null => Binding::Value(PropertyValue::Null),
    }
}

fn path_elem_to_binding(elem: &PathElem) -> PathBinding {
    match elem {
        PathElem::Node(n) => PathBinding::Node(n.id),
        PathElem::Edge(e) => PathBinding::Edge(e.id),
    }
}

/// When a path is being captured, every hop's rel/node needs a trackable
/// binding even if the user left it anonymous — `Expand` only inserts a
/// `rel_var` into the row `if let Some(rv) = rel_var`, silently dropping
/// anonymous rels, which is fine for ordinary matching but loses exactly
/// the information path assembly needs. Returns a clone of `pattern` with
/// every position named (synthesizing `__path_elemN` for anything
/// anonymous), plus the set of names that were synthesized so
/// `execute_match` can strip them from the row again after `assemble_path`
/// runs — they were never something the user could reference. Only this
/// renamed clone is used for plan-building/OPTIONAL-MATCH null-padding
/// bookkeeping *within this one clause*; `carried_vars` (what's exposed to
/// later clauses) is still computed from the original `part.pattern`
/// elsewhere, so synthesized names never leak past this function's caller.
fn name_pattern_for_path(pattern: &Pattern) -> (Pattern, HashSet<String>) {
    fn fresh(counter: &mut usize, synthesized: &mut HashSet<String>) -> String {
        *counter += 1;
        let name = format!("__path_elem{counter}");
        synthesized.insert(name.clone());
        name
    }
    let mut counter = 0usize;
    let mut synthesized = HashSet::new();
    let mut start = pattern.start.clone();
    if start.var.is_none() {
        start.var = Some(fresh(&mut counter, &mut synthesized));
    }
    let hops = pattern
        .hops
        .iter()
        .map(|(rel, node)| {
            let mut rel = rel.clone();
            if rel.var.is_none() {
                rel.var = Some(fresh(&mut counter, &mut synthesized));
            }
            let mut node = node.clone();
            if node.var.is_none() {
                node.var = Some(fresh(&mut counter, &mut synthesized));
            }
            (rel, node)
        })
        .collect();
    (Pattern { start, hops }, synthesized)
}

/// Assembles a `Binding::Path` from `pattern`'s (fully-named, via
/// `name_pattern_for_path`) start/hop variables, in pattern order. Falls
/// back to `Binding::Value(Null)` — never errors — if any position isn't a
/// real node/edge binding, which only happens when this row came from
/// `OPTIONAL MATCH` null-padding (every position `name_pattern_for_path`
/// named is guaranteed present in the row either way, as a real binding or
/// as `Binding::Value(Null)`, so "missing key" isn't a case this needs to
/// handle) — same "no match survives as Null, not a dropped row" outcome
/// `OPTIONAL MATCH` already gives every other variable.
fn assemble_path(pattern: &Pattern, row: &BindingRow) -> Binding {
    let Some(start_id) = path_node_id(pattern.start.var.as_deref(), row) else {
        return Binding::Value(PropertyValue::Null);
    };
    let mut elems = vec![PathBinding::Node(start_id)];
    for (rel, node) in &pattern.hops {
        let Some(edge_id) = path_edge_id(rel.var.as_deref(), row) else {
            return Binding::Value(PropertyValue::Null);
        };
        let Some(node_id) = path_node_id(node.var.as_deref(), row) else {
            return Binding::Value(PropertyValue::Null);
        };
        elems.push(PathBinding::Edge(edge_id));
        elems.push(PathBinding::Node(node_id));
    }
    Binding::Path(elems)
}

fn path_node_id(var: Option<&str>, row: &BindingRow) -> Option<NodeId> {
    match var.and_then(|v| row.get(v)) {
        Some(Binding::Node(id)) => Some(*id),
        _ => None,
    }
}

fn path_edge_id(var: Option<&str>, row: &BindingRow) -> Option<EdgeId> {
    match var.and_then(|v| row.get(v)) {
        Some(Binding::Edge(id)) => Some(*id),
        _ => None,
    }
}

fn require_bound_node(row: &BindingRow, var: &str) -> Result<NodeId, QueryError> {
    match row.get(var) {
        Some(Binding::Node(id)) => Ok(*id),
        _ => Err(QueryError::UnboundVariable(format!(
            "'{var}' must already be bound to a node before shortestPath() — match it in a preceding MATCH"
        ))),
    }
}

/// Walks `parent` (populated by `shortest_path_between`'s BFS) backward
/// from `end` to `start`, then reverses — `parent` only ever needs to
/// answer "how did BFS first reach this node," not support any other
/// traversal, so a plain `HashMap` (not a `LogicalPlan`/adjacency
/// structure) is enough.
fn reconstruct_path(
    parent: &HashMap<NodeId, (NodeId, EdgeId)>,
    start: NodeId,
    end: NodeId,
) -> Vec<PathBinding> {
    let mut hops = Vec::new();
    let mut current = end;
    while current != start {
        let (prev, edge_id) = parent[&current];
        hops.push((edge_id, current));
        current = prev;
    }
    hops.reverse();
    let mut elems = vec![PathBinding::Node(start)];
    for (edge_id, node) in hops {
        elems.push(PathBinding::Edge(edge_id));
        elems.push(PathBinding::Node(node));
    }
    elems
}

/// `WithExpr::Compare`'s value-vs-literal comparison — reuses `compare()`
/// (below) by reducing a `Value` down to the `Option<PropertyValue>` shape
/// it expects; `Node`/`Edge`/`List` have no meaningful comparison against
/// a `Literal` and fall back to "absent", same as a missing property does.
fn compare_value(value: &Value, op: CompareOp, lit: &Literal) -> Option<bool> {
    let prop = match value {
        Value::Null => None,
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(l) => Some(literal_to_value(l)),
        Value::Node(_) | Value::Edge(_) | Value::List(_) | Value::Map(_) | Value::Path(_) => None,
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
        Value::Node(_) | Value::Edge(_) | Value::List(_) | Value::Map(_) | Value::Path(_) => {
            PropertyValue::Null
        }
    }
}

/// `eval_props_to_values`'s stricter cousin of `value_to_property_value`
/// above -- a CREATE prop value that evaluates to a list/map/node/edge/
/// path is a real, reportable error (`None` here), not a silent `Null`.
/// `value_to_property_value`'s silent-`Null` fallback is correct at *its*
/// call sites (a WITH-projected scalar, where a `Value::List`/`Map` genuinely
/// can't occur — see its own doc comment) but was never meant for CREATE's
/// prop map, where a list/map literal is a real, everyday thing to write
/// (`CREATE (n {tags: [1, 2, 3]})`) that MarsDB's storage layer just
/// doesn't support persisting yet -- silently storing `null` instead
/// would be a wrong answer, not a graceful degradation.
fn value_to_storable_property(v: &Value) -> Option<PropertyValue> {
    match v {
        Value::Null => Some(PropertyValue::Null),
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(lit) => Some(literal_to_value(lit)),
        Value::Node(_) | Value::Edge(_) | Value::List(_) | Value::Map(_) | Value::Path(_) => None,
    }
}

/// A bound `NodeId`/`EdgeId` whose record is no longer in the store means
/// exactly one thing within a single statement's transaction: it was
/// deleted earlier in this same statement (e.g. `MATCH (n) DELETE n RETURN
/// n.num` -- real Cypher's `DeletedEntityAccess` error, TCK's Return2
/// scenarios [15]/[16]/[17]). Nothing else can cause a `None` here --
/// there's no concurrent deletion mid-statement, and a `Binding::Node`/
/// `Edge` only ever gets constructed from an id a prior MATCH/CREATE/MERGE
/// in this same transaction actually found or made. Centralized here
/// (rather than each of `binding_to_value`/`resolve_path_elems`/
/// `lookup_prop` re-deriving the message) so the wording stays one place.
fn deleted_entity_access<T>(record: Option<T>) -> Result<T, QueryError> {
    record.ok_or_else(|| {
        QueryError::UnboundVariable(
            "refers to a node/relationship that no longer exists — it was deleted earlier in this statement".into(),
        )
    })
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

fn tag_merge_created(mut row: BindingRow, created: bool) -> BindingRow {
    row.insert(
        MERGE_CREATED_KEY.to_string(),
        Binding::Value(PropertyValue::Bool(created)),
    );
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

/// Three-valued: `None` is Cypher's "unknown", not `false` -- any
/// comparison touching a null (a missing property, or a literal `null` on
/// either side) is unknown, always, regardless of operator -- including
/// `Eq` (`x = null` is unknown, never true, same as real Cypher; it is
/// *not* how `x`'s own missing-ness is tested -- there's no `IS NULL`
/// operator yet). Callers combine this with `and3`/`or3`/`Option::map`
/// (for `NOT`) rather than unwrapping early, so unknown propagates
/// correctly through `AND`/`OR`/`NOT` instead of collapsing to `false`.
fn compare(prop: &Option<PropertyValue>, op: CompareOp, lit: &Literal) -> Option<bool> {
    let Some(prop) = prop else { return None };
    if matches!(prop, PropertyValue::Null) || matches!(lit, Literal::Null) {
        return None;
    }
    compare_property_pair(prop, op, &literal_to_value(lit))
}

/// The actual per-type comparison rules, shared by `compare()`
/// (`PropertyValue` vs a `Literal`, reduced to a `PropertyValue` via
/// `literal_to_value`) and `compare_values` (two arbitrary `Value`s,
/// each reduced to a `PropertyValue` via `value_to_property_value`) --
/// both callers have already handled the "either side is null" case
/// before reaching here. Returns `Option<bool>`, not `bool` -- a
/// type-mismatched pair (`1 < 'a'`) isn't a uniform "false" the way an
/// earlier version of this function had it: real Cypher's `=`/`<>` on
/// mismatched types is a definite `false`/`true` (never equal, so
/// "not equal" is true), but ordering (`<`/`<=`/`>`/`>=`) on mismatched
/// types is `null` (no defined ordering exists to be definite about) --
/// confirmed against real TCK scenarios (`'1.0' < 1.0` is `null`, not
/// `false`; `NaN <> 'a'` is `true`, not `false`), not assumed.
fn compare_property_pair(a: &PropertyValue, op: CompareOp, b: &PropertyValue) -> Option<bool> {
    match (a, b) {
        (PropertyValue::Int(a), PropertyValue::Int(b)) => Some(cmp_ord(op, *a, *b)),
        (PropertyValue::Int(a), PropertyValue::Float(b)) => Some(cmp_f64(op, *a as f64, *b)),
        (PropertyValue::Float(a), PropertyValue::Float(b)) => Some(cmp_f64(op, *a, *b)),
        (PropertyValue::Float(a), PropertyValue::Int(b)) => Some(cmp_f64(op, *a, *b as f64)),
        (PropertyValue::String(a), PropertyValue::String(b)) => Some(match op {
            CompareOp::StartsWith => a.starts_with(b.as_str()),
            CompareOp::EndsWith => a.ends_with(b.as_str()),
            CompareOp::Contains => a.contains(b.as_str()),
            _ => cmp_ord(op, a.as_str(), b.as_str()),
        }),
        // Real Cypher defines boolean ordering (`false < true`), same as
        // Rust's own `bool: PartialOrd` -- confirmed via a real TCK
        // scenario (`Quantifier7 :: [3]`) that specifically compares two
        // boolean expressions with `<=`.
        (PropertyValue::Bool(a), PropertyValue::Bool(b)) => Some(cmp_ord(op, *a, *b)),
        _ => match op {
            CompareOp::Eq => Some(false),
            CompareOp::Ne => Some(true),
            // A string predicate on a non-null, non-string operand has no
            // defined answer (undefined, not "definitely false") -- same
            // "type mismatch -> null" stance as ordering, confirmed via a
            // real TCK scenario (`'abc' STARTS WITH true` must be `null`,
            // not `false`, so `(x STARTS WITH true) <> (x STARTS WITH
            // true)` correctly stays `null` rather than folding to a
            // spurious `false`/`true`).
            CompareOp::StartsWith
            | CompareOp::EndsWith
            | CompareOp::Contains
            | CompareOp::Lt
            | CompareOp::Le
            | CompareOp::Gt
            | CompareOp::Ge => None,
        },
    }
}

/// A `ReturnExpr` boolean operand -- `Null` is "unknown" (`None`), a real
/// bool passes through, anything else is a genuine type error (real
/// Cypher: `1 AND true` doesn't silently coerce).
fn value_to_bool3(v: &Value) -> Result<Option<bool>, QueryError> {
    match v {
        Value::Null => Ok(None),
        Value::Literal(Literal::Bool(b)) | Value::Property(PropertyValue::Bool(b)) => Ok(Some(*b)),
        other => Err(QueryError::Parse(format!(
            "expected a boolean, got {other:?}"
        ))),
    }
}

fn bool3_to_value(b: Option<bool>) -> Value {
    match b {
        Some(b) => Value::Literal(Literal::Bool(b)),
        None => Value::Null,
    }
}

/// `None`/`None` (both unknown) combines to unknown, matching Cypher's
/// `AND` truth table -- `false` wins over `unknown` (`false AND unknown =
/// false`), but `true AND unknown = unknown`, not `true`.
fn and3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

/// Mirrors `and3` for `OR` -- `true` wins over `unknown`.
fn or3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

/// `XOR` has no "one side already decides it" shortcut the way `AND`/`OR`
/// do -- either operand being unknown makes the whole result unknown,
/// since flipping the unknown side could flip the answer either way.
fn xor3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a != b),
        _ => None,
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
        // Only meaningful for String/String, handled separately in
        // `compare()` before reaching here -- a numeric operand with one
        // of these ops is a type mismatch, same as any other.
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => false,
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
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => false,
    }
}

/// Value equality for CASE's WHEN-comparison (and, elsewhere, DISTINCT
/// dedup within an aggregate). Null == Null -> true here deliberately,
/// unlike `compare()`'s three-valued `WHERE`-filter semantics -- CASE and
/// DISTINCT need a definite yes/no ("is this the same value as a value
/// already collected", "does this WHEN branch match") rather than
/// "unknown", so plain equality is the correct, separate choice here, not
/// an oversight. `Node`/`Edge` compare by id (graph identity), not
/// full-struct contents — cheaper, and the correct semantics regardless
/// (two bindings are "the same node" iff the same node, not iff their
/// label/prop snapshots happen to match).
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
        (Value::List(la), Value::List(lb)) => {
            la.len() == lb.len() && la.iter().zip(lb).all(|(x, y)| value_eq(x, y))
        }
        _ => false,
    }
}

/// A number coerced out of a `Value`, for `apply_arith` below -- separate
/// from `PropertyValue`/`Literal` since either could hold the operand
/// (`n.price + 1` mixes a stored property with a literal).
enum ArithNum {
    Int(i64),
    Float(f64),
}

fn as_arith_num(v: &Value) -> Option<ArithNum> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => {
            Some(ArithNum::Int(*i))
        }
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => {
            Some(ArithNum::Float(*f))
        }
        _ => None,
    }
}

fn as_arith_str(v: &Value) -> Option<&str> {
    match v {
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => {
            Some(s.as_str())
        }
        _ => None,
    }
}

/// `lhs op rhs` for `ReturnExpr::Arith`. Null propagates (matches every
/// other operator's null-handling convention in this file). `+` also
/// concatenates two strings, real Cypher's other overload for that
/// operator; every other combination of non-numeric operands is a real
/// type error, not a silent `Null`/`false` fallback -- an arithmetic
/// expression that can't be evaluated should say so, not produce a
/// plausible-looking wrong answer.
fn apply_arith(op: ArithOp, a: &Value, b: &Value) -> Result<Value, QueryError> {
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return Ok(Value::Null);
    }
    if op == ArithOp::Add {
        if let (Some(sa), Some(sb)) = (as_arith_str(a), as_arith_str(b)) {
            return Ok(Value::Property(PropertyValue::String(format!("{sa}{sb}"))));
        }
    }
    if let Some(result) = apply_temporal_arith(op, a, b)? {
        return Ok(result);
    }
    let (Some(na), Some(nb)) = (as_arith_num(a), as_arith_num(b)) else {
        return Err(QueryError::Parse(format!(
            "arithmetic needs two numbers (or, for +, two strings) -- got {a:?} and {b:?}"
        )));
    };
    // Int/Int stays Int (truncating division/modulo, matching Rust's `/`/
    // `%` on integers) -- any Float operand promotes the whole expression
    // to Float, same numeric-promotion rule `compare()` already follows.
    Ok(match (na, nb) {
        (ArithNum::Int(x), ArithNum::Int(y)) => {
            if matches!(op, ArithOp::Div | ArithOp::Mod) && y == 0 {
                return Err(QueryError::Parse("division by zero".into()));
            }
            Value::Property(PropertyValue::Int(match op {
                ArithOp::Add => x + y,
                ArithOp::Sub => x - y,
                ArithOp::Mul => x * y,
                ArithOp::Div => x / y,
                ArithOp::Mod => x % y,
            }))
        }
        (x, y) => {
            let x = match x {
                ArithNum::Int(i) => i as f64,
                ArithNum::Float(f) => f,
            };
            let y = match y {
                ArithNum::Int(i) => i as f64,
                ArithNum::Float(f) => f,
            };
            Value::Property(PropertyValue::Float(match op {
                ArithOp::Add => x + y,
                ArithOp::Sub => x - y,
                ArithOp::Mul => x * y,
                ArithOp::Div => x / y,
                ArithOp::Mod => x % y,
            }))
        }
    })
}

fn as_date(v: &Value) -> Option<i32> {
    match v {
        Value::Property(PropertyValue::Date(d)) => Some(*d),
        _ => None,
    }
}

fn as_duration(v: &Value) -> Option<temporal::DurationParts> {
    match v {
        Value::Property(PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => Some((*months, *days, *seconds, *nanos)),
        _ => None,
    }
}

fn duration_value((months, days, seconds, nanos): temporal::DurationParts) -> Value {
    Value::Property(PropertyValue::Duration {
        months,
        days,
        seconds,
        nanos,
    })
}

/// The `Date`/`Duration` cases of `+`/`-`/`*`/`/` -- tried before
/// `apply_arith`'s generic numeric path, since a `Date`/`Duration`
/// operand is never an `ArithNum`. Returns `Ok(None)` (not an error) for
/// any operand-type combination it doesn't recognize, so `apply_arith`
/// falls through to its own "not two numbers" error with the *original*
/// operands in the message, rather than this function needing to
/// duplicate that error text.
///
/// Only `Date`/`Duration` arithmetic is implemented here -- there's no
/// `Time`/`DateTime`/`LocalDateTime` to add a `Duration` to (see this
/// module's temporal-support docs), and `Date - Date` (which real Cypher
/// doesn't define as a direct operator either -- `duration.between(...)`
/// is the real spelling, itself out of scope, see the README) is
/// deliberately *not* handled, falling through to the same "not two
/// numbers" error a truly nonsensical subtraction would already get.
fn apply_temporal_arith(op: ArithOp, a: &Value, b: &Value) -> Result<Option<Value>, QueryError> {
    let date_plus_duration =
        |d: i32, dur: temporal::DurationParts, negate: bool| -> Result<Value, QueryError> {
            let (months, days, seconds, nanos) = dur;
            temporal::add_duration_to_date(d, months, days, seconds, nanos, negate)
                .map(|d| Value::Property(PropertyValue::Date(d)))
                .ok_or_else(|| {
                    QueryError::Parse("date +/- duration produced an out-of-range date".into())
                })
        };
    Ok(match op {
        ArithOp::Add => {
            if let (Some(d), Some(dur)) = (as_date(a), as_duration(b)) {
                Some(date_plus_duration(d, dur, false)?)
            } else if let (Some(dur), Some(d)) = (as_duration(a), as_date(b)) {
                Some(date_plus_duration(d, dur, false)?)
            } else if let (Some(x), Some(y)) = (as_duration(a), as_duration(b)) {
                Some(duration_value(temporal::add_duration(x, y)))
            } else {
                None
            }
        }
        ArithOp::Sub => {
            if let (Some(d), Some(dur)) = (as_date(a), as_duration(b)) {
                Some(date_plus_duration(d, dur, true)?)
            } else if let (Some(x), Some(y)) = (as_duration(a), as_duration(b)) {
                Some(duration_value(temporal::sub_duration(x, y)))
            } else {
                None
            }
        }
        ArithOp::Mul => {
            if let (Some(dur), Some(f)) = (as_duration(a), value_as_f64(b)) {
                Some(duration_value(temporal::scale_duration(dur, f)))
            } else if let (Some(f), Some(dur)) = (value_as_f64(a), as_duration(b)) {
                Some(duration_value(temporal::scale_duration(dur, f)))
            } else {
                None
            }
        }
        ArithOp::Div => {
            if let (Some(dur), Some(f)) = (as_duration(a), value_as_f64(b)) {
                if f == 0.0 {
                    return Err(QueryError::Parse("division by zero".into()));
                }
                Some(duration_value(temporal::scale_duration(dur, 1.0 / f)))
            } else {
                None
            }
        }
        ArithOp::Mod => None,
    })
}

/// `list[index]` -- a negative index counts from the end (`-1` is the
/// last element). Out of bounds either way is `Null`, not an error --
/// matches real Cypher (`[1,2,3][10]` is `null`, not a failure), and is
/// the only sane behavior for an index that's itself a runtime expression
/// rather than a literal a human could sanity-check up front.
fn apply_index(list: &Value, index: &Value) -> Result<Value, QueryError> {
    if matches!(list, Value::Null) || matches!(index, Value::Null) {
        return Ok(Value::Null);
    }
    // `map[key]` -- real Cypher's dynamic map-field access (`map['name']`,
    // as opposed to `map.name`'s static form -- `lookup_prop`/`ReturnExpr
    // ::Prop` above). Unlike `.prop`, this can return a full nested
    // `Value` (a list/map field value), not just a scalar `PropertyValue`
    // -- `apply_index`'s return type already allows that, no narrowing
    // needed the way `map_value_as_property` has to for `.prop`.
    if let Value::Map(entries) = list {
        let Some(key) = as_arith_str(index) else {
            return Err(QueryError::Parse(format!(
                "a map index must be a string, got {index:?}"
            )));
        };
        return Ok(entries.get(key).cloned().unwrap_or(Value::Null));
    }
    let Value::List(items) = list else {
        return Err(QueryError::Parse(format!(
            "[] indexing needs a list or map, got {list:?}"
        )));
    };
    let Some(ArithNum::Int(i)) = as_arith_num(index) else {
        return Err(QueryError::Parse(format!(
            "a list index must be an integer, got {index:?}"
        )));
    };
    let len = items.len() as i64;
    let i = if i < 0 { i + len } else { i };
    if i < 0 || i >= len {
        return Ok(Value::Null);
    }
    Ok(items[i as usize].clone())
}

/// `list[start..end]` -- same negative-counts-from-end rule as
/// `apply_index`, but bounds clamp to `[0, len]` instead of nulling out
/// (`[1,2,3][-5..5]` is the whole list, not `null`), and a start at or
/// past the (clamped) end yields `[]` rather than erroring
/// (`[1,2,3][3..1]` is `[]`) -- both match real Cypher, and both were
/// real TCK scenarios, not guessed behavior.
fn apply_slice(
    list: &Value,
    start: Option<&Value>,
    end: Option<&Value>,
) -> Result<Value, QueryError> {
    if matches!(list, Value::Null) {
        return Ok(Value::Null);
    }
    let Value::List(items) = list else {
        return Err(QueryError::Parse(format!(
            "[..] slicing needs a list, got {list:?}"
        )));
    };
    let len = items.len() as i64;
    let clamp = |i: i64| -> i64 {
        let i = if i < 0 { i + len } else { i };
        i.clamp(0, len)
    };
    let bound_index = |v: Option<&Value>, default: i64| -> Result<Option<i64>, QueryError> {
        match v {
            None => Ok(Some(default)),
            Some(Value::Null) => Ok(None),
            Some(other) => match as_arith_num(other) {
                Some(ArithNum::Int(i)) => Ok(Some(clamp(i))),
                _ => Err(QueryError::Parse(format!(
                    "a slice bound must be an integer, got {other:?}"
                ))),
            },
        }
    };
    // A null bound (as opposed to an *omitted* one, already handled by
    // `start`/`end` being `None` at the AST level) propagates -- same
    // null-handling convention as every other operator here.
    let (Some(start_idx), Some(end_idx)) = (bound_index(start, 0)?, bound_index(end, len)?) else {
        return Ok(Value::Null);
    };
    if start_idx >= end_idx {
        return Ok(Value::List(Vec::new()));
    }
    Ok(Value::List(
        items[start_idx as usize..end_idx as usize].to_vec(),
    ))
}

fn call_builtin(name: &str, args: &[Value]) -> Result<Value, QueryError> {
    match name.to_ascii_lowercase().as_str() {
        "coalesce" => Ok(args
            .iter()
            .find(|v| !matches!(v, Value::Null))
            .cloned()
            .unwrap_or(Value::Null)),
        "tointeger" => match args.first() {
            Some(v) => to_integer(v),
            None => Ok(Value::Null),
        },
        "tostring" => match args.first() {
            Some(v) => to_string_value(v),
            None => Ok(Value::Null),
        },
        "date" => date_builtin(args),
        "duration" => duration_builtin(args),
        // The dominant real-world use of shortestPath() is measuring it
        // (degrees-of-separation queries), not returning/rendering the
        // raw path object — path elements alternate node/edge/.../node,
        // so edge count is (elements.len() - 1) / 2.
        "length" => Ok(match args.first() {
            Some(Value::Path(elems)) => {
                Value::Property(PropertyValue::Int(((elems.len().max(1) - 1) / 2) as i64))
            }
            Some(Value::Null) | None => Value::Null,
            Some(other) => {
                return Err(QueryError::Parse(format!(
                    "length() expects a path, got {other:?}"
                )))
            }
        }),
        other => Err(QueryError::Parse(format!("unknown function: {other}"))),
    }
}

/// A quantifier's own per-element truthiness check when it has no `WHERE`
/// at all (`ANY(x IN list)`, not `ANY(x IN list WHERE ...)`) -- three-
/// valued, same as a real `WHERE` predicate: `null` propagates as
/// "unknown" (`None`), a literal bool passes through, anything else
/// (non-bool, non-null) is definitely-false, same convention `CASE`'s
/// subject-less `WHEN` branch already uses for a non-bool test value.
fn item_truthy(v: &Value) -> Option<bool> {
    match v {
        Value::Null => None,
        Value::Literal(Literal::Bool(b)) | Value::Property(PropertyValue::Bool(b)) => Some(*b),
        _ => Some(false),
    }
}

/// Real Cypher quantifiers use three-valued logic, not a simple count --
/// a single definite `true`/`false` among the elements can already decide
/// the answer even in the presence of other `null` elements, and only
/// "no definite answer, but at least one unknown" actually yields `null`.
/// Confirmed against the real TCK scenarios (Quantifier1-4, scenario 10,
/// "... on lists containing nulls") rather than assumed -- a first version
/// of this collapsed `null` predicates to `false`, which silently passed
/// every non-null-list scenario but produced 19 real wrong answers on
/// exactly these null-list cases.
fn eval_quantifier(kind: QuantifierKind, preds: &[Option<bool>]) -> Option<bool> {
    let true_count = preds.iter().filter(|p| **p == Some(true)).count();
    let any_false = preds.iter().any(|p| *p == Some(false));
    let any_null = preds.iter().any(|p| p.is_none());
    match kind {
        QuantifierKind::Any => {
            if true_count > 0 {
                Some(true)
            } else if any_null {
                None
            } else {
                Some(false)
            }
        }
        QuantifierKind::None => {
            if true_count > 0 {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true)
            }
        }
        QuantifierKind::All => {
            if any_false {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true)
            }
        }
        QuantifierKind::Single => {
            if true_count >= 2 {
                Some(false)
            } else if any_null {
                None
            } else {
                Some(true_count == 1)
            }
        }
    }
}

fn to_integer(v: &Value) -> Result<Value, QueryError> {
    // A float-formatted string ('1.7', '2.9') isn't an i64, but real
    // Cypher's toInteger() still accepts it -- parse as a float and
    // truncate, same as the Float arm below, rather than failing straight
    // to null the way a bare `i64::parse` would (found via a real TCK
    // scenario: `toInteger('1.7')` must be `1`, not `null`).
    let as_str_parse = |s: &str| match s.trim().parse::<i64>() {
        Ok(i) => Value::Property(PropertyValue::Int(i)),
        Err(_) => match s.trim().parse::<f64>() {
            Ok(f) => Value::Property(PropertyValue::Int(f as i64)),
            Err(_) => Value::Null,
        },
    };
    Ok(match v {
        Value::Property(PropertyValue::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Property(PropertyValue::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Property(PropertyValue::String(s)) => as_str_parse(s),
        Value::Literal(Literal::Int(i)) => Value::Property(PropertyValue::Int(*i)),
        Value::Literal(Literal::Float(f)) => Value::Property(PropertyValue::Int(*f as i64)),
        Value::Literal(Literal::String(s)) => as_str_parse(s),
        Value::Property(PropertyValue::Bool(_) | PropertyValue::Null)
        | Value::Literal(Literal::Bool(_) | Literal::Null)
        | Value::Null => Value::Null,
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        // A node/edge/list/map/path has no numeric conversion at all -- a
        // real error (found via a real TCK scenario expecting exactly
        // this), not a silent null the way an out-of-range/unparseable
        // scalar is.
        Value::Property(PropertyValue::Date(_) | PropertyValue::Duration { .. })
        | Value::Node(_)
        | Value::Edge(_)
        | Value::List(_)
        | Value::Map(_)
        | Value::Path(_) => {
            return Err(QueryError::Parse(format!(
                "toInteger() cannot convert {v:?} to an integer"
            )))
        }
    })
}

/// `toString(...)` — Int/Float/Bool render the same as their `Display`
/// impl already does elsewhere (`marsdb-cli`'s `format_property`/
/// `format_literal`); `Date`/`Duration` go through `temporal::format_*`.
/// Null propagates, while graph, collection, map, and path values are a
/// runtime type error rather than silently becoming null (TypeConversion4
/// scenario [10]).
fn to_string_value(v: &Value) -> Result<Value, QueryError> {
    let s = match v {
        Value::Property(PropertyValue::String(s)) | Value::Literal(Literal::String(s)) => s.clone(),
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => i.to_string(),
        Value::Property(PropertyValue::Float(f)) | Value::Literal(Literal::Float(f)) => {
            f.to_string()
        }
        Value::Property(PropertyValue::Bool(b)) | Value::Literal(Literal::Bool(b)) => b.to_string(),
        Value::Property(PropertyValue::Date(d)) => temporal::format_date(*d),
        Value::Property(PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        }) => temporal::format_duration(*months, *days, *seconds, *nanos),
        Value::Property(PropertyValue::Null) | Value::Literal(Literal::Null) | Value::Null => {
            return Ok(Value::Null);
        }
        Value::Literal(Literal::Param(name)) => {
            unreachable!("param ${name} must be substituted before execution — see params::substitute_params")
        }
        Value::Node(_) | Value::Edge(_) | Value::List(_) | Value::Map(_) | Value::Path(_) => {
            return Err(QueryError::Parse(format!(
                "toString() cannot convert {v:?} to a string"
            )))
        }
    };
    Ok(Value::Property(PropertyValue::String(s)))
}

/// `date()` — zero args (today, UTC — see `temporal::today_epoch_day`'s
/// docs), a string (`date('2015-07-21')`, the calendar forms `temporal::
/// parse_date` supports), a map (`date({year: 1984, month: 10, day:
/// 11})`, calendar construction only), or another `Date` (identity —
/// `date(d)` where `d` is already a `Date`, e.g. from `toString`
/// round-tripping through `date(toString(d))`). Deliberately does *not*
/// support the week-date/ordinal-date/quarter map or string construction
/// forms real Cypher also has (`date({year: 2015, week: 1})`,
/// `date('2015-W30-2')`, ...) — a real, documented gap (see the README),
/// not a silent wrong answer: both `parse_date` and `date_from_map`
/// return a clear error/`None` for those rather than guessing.
fn date_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if args.len() > 1 {
        return Err(QueryError::Parse(format!(
            "date() expects zero or one argument, got {}",
            args.len()
        )));
    }
    let Some(arg) = args.first() else {
        return Ok(Value::Property(PropertyValue::Date(
            temporal::today_epoch_day(),
        )));
    };
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    if let Value::Property(PropertyValue::Date(d)) = arg {
        return Ok(Value::Property(PropertyValue::Date(*d)));
    }
    if let Some(s) = as_arith_str(arg) {
        let d = temporal::parse_date(s).ok_or_else(|| {
            QueryError::Parse(format!(
                "'{s}' isn't a date string MarsDB can parse -- only the calendar forms YYYY-MM-DD/YYYYMMDD/\
                 YYYY-MM/YYYYMM/YYYY are supported, not week-date or ordinal-date forms"
            ))
        })?;
        return Ok(Value::Property(PropertyValue::Date(d)));
    }
    if let Value::Map(m) = arg {
        return Ok(Value::Property(PropertyValue::Date(date_from_map(m)?)));
    }
    Err(QueryError::Parse(format!(
        "date() doesn't support this argument: {arg:?}"
    )))
}

/// `date({year, month, day})` — the calendar construction form only (see
/// `date_builtin`'s docs for what's deliberately missing).
fn date_from_map(m: &BTreeMap<String, Value>) -> Result<i32, QueryError> {
    const ALLOWED: &[&str] = &["year", "month", "day"];
    if let Some(bad) = m.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
        return Err(QueryError::Parse(format!(
            "date({{...}}) key '{bad}' isn't supported -- MarsDB only builds a Date from a calendar {{year, month, \
             day}} map, not week-date/quarter/ordinal-day construction"
        )));
    }
    let integer_field = |key: &str, value: &Value| {
        value_as_i64(value)
            .ok_or_else(|| QueryError::Parse(format!("date({{...}})'s '{key}' must be an integer")))
    };
    let year_raw = integer_field(
        "year",
        m.get("year")
            .ok_or_else(|| QueryError::Parse("date({...}) requires a 'year' key".into()))?,
    )?;
    let year = i32::try_from(year_raw).map_err(|_| {
        QueryError::Parse(format!(
            "date({{...}})'s 'year' is out of range: {year_raw}"
        ))
    })?;
    let month_raw = match m.get("month") {
        Some(v) => integer_field("month", v)?,
        None => 1,
    };
    let month = u32::try_from(month_raw).map_err(|_| {
        QueryError::Parse(format!(
            "date({{...}})'s 'month' is out of range: {month_raw}"
        ))
    })?;
    let day_raw = match m.get("day") {
        Some(v) => integer_field("day", v)?,
        None => 1,
    };
    let day = u32::try_from(day_raw).map_err(|_| {
        QueryError::Parse(format!("date({{...}})'s 'day' is out of range: {day_raw}"))
    })?;
    temporal::epoch_day_from_ymd(year, month, day).ok_or_else(|| {
        QueryError::Parse(format!(
            "{year:04}-{month:02}-{day:02} isn't a valid calendar date"
        ))
    })
}

/// `duration(...)` — a string (ISO-8601 `'P...'` text, `temporal::
/// parse_duration`) or a map (`duration({days: 14, hours: 16})`,
/// `temporal::normalize_duration`). No zero-arg form (real Cypher has
/// none either — a duration has no "current" value the way a date/time
/// does).
fn duration_builtin(args: &[Value]) -> Result<Value, QueryError> {
    if args.len() != 1 {
        return Err(QueryError::Parse(format!(
            "duration() expects exactly one argument, got {}",
            args.len()
        )));
    }
    let arg = &args[0];
    if matches!(arg, Value::Null) {
        return Ok(Value::Null);
    }
    let (months, days, seconds, nanos) = if let Some(s) = as_arith_str(arg) {
        temporal::parse_duration(s).ok_or_else(|| {
            QueryError::Parse(format!(
                "'{s}' isn't a duration string MarsDB can parse -- only ISO-8601 'PnYnMnWnDTnHnMnS' text is \
                 supported, not the alternate combined date-time duration syntax"
            ))
        })?
    } else if let Value::Map(m) = arg {
        temporal::normalize_duration(duration_fields_from_map(m)?)
    } else {
        return Err(QueryError::Parse(format!(
            "duration() doesn't support this argument: {arg:?}"
        )));
    };
    Ok(Value::Property(PropertyValue::Duration {
        months,
        days,
        seconds,
        nanos,
    }))
}

fn duration_fields_from_map(
    m: &BTreeMap<String, Value>,
) -> Result<temporal::DurationFields, QueryError> {
    const ALLOWED: &[&str] = &[
        "years",
        "months",
        "weeks",
        "days",
        "hours",
        "minutes",
        "seconds",
        "milliseconds",
        "microseconds",
        "nanoseconds",
    ];
    if let Some(bad) = m.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
        return Err(QueryError::Parse(format!(
            "duration({{...}}) key '{bad}' isn't a recognized duration unit"
        )));
    }
    let field = |key: &str| -> Result<f64, QueryError> {
        match m.get(key) {
            None => Ok(0.0),
            Some(v) => value_as_f64(v).ok_or_else(|| {
                QueryError::Parse(format!("duration({{...}})'s '{key}' must be a number"))
            }),
        }
    };
    Ok(temporal::DurationFields {
        years: field("years")?,
        months: field("months")?,
        weeks: field("weeks")?,
        days: field("days")?,
        hours: field("hours")?,
        minutes: field("minutes")?,
        seconds: field("seconds")?,
        milliseconds: field("milliseconds")?,
        microseconds: field("microseconds")?,
        nanoseconds: field("nanoseconds")?,
    })
}

fn value_as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Property(PropertyValue::Int(i)) | Value::Literal(Literal::Int(i)) => Some(*i),
        _ => None,
    }
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match as_arith_num(v)? {
        ArithNum::Int(i) => Some(i as f64),
        ArithNum::Float(f) => Some(f),
    }
}

/// Shared `Date`/`Duration` component access for `d.<prop>` — used by
/// both `lookup_prop` (a bound row variable, e.g. `WITH v.date AS d ...
/// d.year`) and `eval_projected_expr`'s `Prop` arm (the post-projection/
/// ORDER BY path). Returns `None` for any property name that isn't a
/// recognized component (or a non-temporal `PropertyValue`), the same
/// "treat as absent, not an error" convention every other `.prop` access
/// already follows for an unknown property.
fn temporal_component(pv: &PropertyValue, prop: &str) -> Option<PropertyValue> {
    match pv {
        PropertyValue::Date(d) => temporal::date_component(*d, prop).map(PropertyValue::Int),
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => temporal::duration_component(*months, *days, *seconds, *nanos, prop)
            .map(PropertyValue::Int),
        _ => None,
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
    limit: Option<i64>,
) -> Result<Vec<Vec<Value>>, QueryError> {
    // An ORDER BY expression that repeats a returned expression verbatim
    // (`RETURN n.name, count(*) AS foo ORDER BY n.name`) names a real
    // output column by its default name -- match it directly by position
    // rather than re-evaluating the expression, which would need bindings
    // (e.g. `n`) that only the pre-aggregation rows had and are gone by
    // this post-projection point.
    let order_by_col: Vec<Option<usize>> = order_by
        .iter()
        .map(|(expr, _)| {
            columns
                .iter()
                .position(|c| *c == default_column_name(expr, 0))
        })
        .collect();
    let mut keyed: Vec<(Vec<Value>, Vec<Value>)> = Vec::with_capacity(rows.len());
    for row in rows {
        let row_map: HashMap<String, Value> =
            columns.iter().cloned().zip(row.iter().cloned()).collect();
        let keys = order_by
            .iter()
            .zip(&order_by_col)
            .map(|((expr, _), col)| match col {
                Some(i) => Ok(row[*i].clone()),
                None => eval_projected_expr(expr, &row_map),
            })
            .collect::<Result<Vec<_>, _>>()?;
        keyed.push((keys, row));
    }
    Ok(top_k_by(keyed, order_by, limit)
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

/// Same expression shape as `eval_return_expr`, but resolves `Var`/`Prop`
/// against already-projected output columns instead of the graph-bound
/// `BindingRow` — no `WriteTransaction`/`GraphStore` access needed, since a
/// projected `Value::Node`/`Value::Edge` already carries its full record
/// (including props) from when it was first materialized.
fn eval_projected_expr(
    expr: &ReturnExpr,
    row: &HashMap<String, Value>,
) -> Result<Value, QueryError> {
    match expr {
        ReturnExpr::Var(name) => row
            .get(name)
            .cloned()
            .ok_or_else(|| QueryError::UnboundVariable(name.clone())),
        ReturnExpr::Prop(pa) => {
            let base = row
                .get(&pa.var)
                .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
            match base {
                Value::Map(m) => Ok(m.get(&pa.prop).cloned().unwrap_or(Value::Null)),
                Value::Node(n) => Ok(match n.props.get(&pa.prop).cloned() {
                    Some(PropertyValue::Null) | None => Value::Null,
                    Some(v) => Value::Property(v),
                }),
                Value::Edge(e) => Ok(match e.props.get(&pa.prop).cloned() {
                    Some(PropertyValue::Null) | None => Value::Null,
                    Some(v) => Value::Property(v),
                }),
                // `d.year`/`d.months`/etc component access on a `Date`/
                // `Duration` in projected/ORDER BY position -- mirrors
                // `lookup_prop_value`'s equivalent `Binding::Value(pv)`
                // handling for the pre-projection path.
                Value::Property(pv) => Ok(match temporal_component(pv, &pa.prop) {
                    Some(component) => Value::Property(component),
                    None => Value::Null,
                }),
                _ => Ok(Value::Null),
            }
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
        ReturnExpr::Arith(l, op, r) => {
            let lv = eval_projected_expr(l, row)?;
            let rv = eval_projected_expr(r, row)?;
            apply_arith(*op, &lv, &rv)
        }
        ReturnExpr::ListLit(items) => Ok(Value::List(
            items
                .iter()
                .map(|item| eval_projected_expr(item, row))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        ReturnExpr::Index(base, index) => {
            let base_v = eval_projected_expr(base, row)?;
            let index_v = eval_projected_expr(index, row)?;
            apply_index(&base_v, &index_v)
        }
        ReturnExpr::Slice(base, start, end) => {
            let base_v = eval_projected_expr(base, row)?;
            let start_v = start
                .as_deref()
                .map(|s| eval_projected_expr(s, row))
                .transpose()?;
            let end_v = end
                .as_deref()
                .map(|e| eval_projected_expr(e, row))
                .transpose()?;
            apply_slice(&base_v, start_v.as_ref(), end_v.as_ref())
        }
        ReturnExpr::ListComp {
            var,
            source,
            where_clause,
            project,
        } => {
            let source_v = eval_projected_expr(source, row)?;
            let items = match source_v {
                Value::List(items) => items,
                Value::Null => return Ok(Value::Null),
                other => {
                    return Err(QueryError::Parse(format!(
                        "list comprehension source must be a list, got {other:?}"
                    )))
                }
            };
            let mut result = Vec::with_capacity(items.len());
            for item in items {
                let mut scoped_row = row.clone();
                scoped_row.insert(var.clone(), item.clone());
                let keep = match where_clause {
                    Some(w) => value_to_bool3(&eval_projected_expr(w, &scoped_row)?)? == Some(true),
                    None => true,
                };
                if !keep {
                    continue;
                }
                result.push(match project {
                    Some(p) => eval_projected_expr(p, &scoped_row)?,
                    None => item,
                });
            }
            Ok(Value::List(result))
        }
        ReturnExpr::Quantifier {
            kind,
            var,
            source,
            where_clause,
        } => {
            let source_v = eval_projected_expr(source, row)?;
            let items = match source_v {
                Value::List(items) => items,
                Value::Null => return Ok(Value::Null),
                other => {
                    return Err(QueryError::Parse(format!(
                        "quantifier source must be a list, got {other:?}"
                    )))
                }
            };
            let mut preds = Vec::with_capacity(items.len());
            for item in &items {
                let mut scoped_row = row.clone();
                scoped_row.insert(var.clone(), item.clone());
                preds.push(match where_clause {
                    Some(w) => value_to_bool3(&eval_projected_expr(w, &scoped_row)?)?,
                    None => item_truthy(item),
                });
            }
            Ok(match eval_quantifier(*kind, &preds) {
                Some(b) => Value::Literal(Literal::Bool(b)),
                None => Value::Null,
            })
        }
        ReturnExpr::MapLit(entries) => {
            let mut map = BTreeMap::new();
            for (k, v) in entries {
                map.insert(k.clone(), eval_projected_expr(v, row)?);
            }
            Ok(Value::Map(map))
        }
        ReturnExpr::And(l, r) => Ok(bool3_to_value(and3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Or(l, r) => Ok(bool3_to_value(or3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Xor(l, r) => Ok(bool3_to_value(xor3(
            value_to_bool3(&eval_projected_expr(l, row)?)?,
            value_to_bool3(&eval_projected_expr(r, row)?)?,
        ))),
        ReturnExpr::Not(e) => Ok(bool3_to_value(
            value_to_bool3(&eval_projected_expr(e, row)?)?.map(|b| !b),
        )),
        ReturnExpr::Compare(l, op, r) => {
            let lv = eval_projected_expr(l, row)?;
            let rv = eval_projected_expr(r, row)?;
            Ok(bool3_to_value(compare_values(&lv, *op, &rv)))
        }
        ReturnExpr::IsNull(e) => Ok(Value::Literal(Literal::Bool(matches!(
            eval_projected_expr(e, row)?,
            Value::Null
        )))),
    }
}

/// `RETURN DISTINCT`'s result-set-level dedup -- structural equality of
/// the whole row (same `HashKey` machinery `DISTINCT` inside an aggregate
/// call and `resolve_grouped_rows`' grouping already use, not `value_eq`'s
/// definite-equality-only comparison, since a `HashSet` needs `Hash` too).
/// Keeps the first occurrence of each distinct row, preserving order --
/// what every other DB's `DISTINCT` does, and what a human reading the
/// query would expect.
fn dedup_rows(rows: Vec<Vec<Value>>) -> Result<Vec<Vec<Value>>, QueryError> {
    let mut seen: HashSet<Vec<HashKey>> = HashSet::with_capacity(rows.len());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key = row
            .iter()
            .map(value_hash_key)
            .collect::<Result<Vec<_>, _>>()?;
        if seen.insert(key) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Sorts `keyed` (each entry paired with its precomputed per-column sort
/// keys) by `order_by`'s directions, keeping only the first `limit` items
/// when one is given and smaller than the row count. When it is, uses
/// `select_nth_unstable_by` to partition around the k-th smallest element
/// (O(n) average) and sorts only that k-sized prefix (O(k log k)), instead
/// of a full O(n log n) sort of every row just to immediately discard all
/// but the first few -- the "ORDER BY + LIMIT -> TOP-K" rewrite real query
/// engines apply. Shared by all three ORDER BY sites (`WITH`'s own,
/// non-aggregating `RETURN`'s, and aggregating `RETURN`'s), which otherwise
/// each build the identical `keyed`-then-sort shape around a different row
/// type.
fn top_k_by<T>(
    mut keyed: Vec<(Vec<Value>, T)>,
    order_by: &[(ReturnExpr, SortDir)],
    limit: Option<i64>,
) -> Vec<(Vec<Value>, T)> {
    let cmp = |a: &(Vec<Value>, T), b: &(Vec<Value>, T)| -> std::cmp::Ordering {
        for (i, (_, dir)) in order_by.iter().enumerate() {
            let ord = compare_with_dir(&a.0[i], &b.0[i], *dir);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    };
    match limit {
        Some(n) => {
            let k = n.max(0) as usize;
            if k == 0 {
                keyed.clear();
            } else if k < keyed.len() {
                keyed.select_nth_unstable_by(k - 1, cmp);
                keyed.truncate(k);
                keyed.sort_by(cmp);
            } else {
                keyed.sort_by(cmp);
            }
        }
        None => keyed.sort_by(cmp),
    }
    keyed
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
        (Some(PropertyValue::Float(x)), Some(PropertyValue::Float(y))) => {
            x.partial_cmp(&y).unwrap_or(Ordering::Equal)
        }
        (Some(PropertyValue::String(x)), Some(PropertyValue::String(y))) => x.cmp(&y),
        (Some(PropertyValue::Bool(x)), Some(PropertyValue::Bool(y))) => x.cmp(&y),
        (Some(PropertyValue::Date(x)), Some(PropertyValue::Date(y))) => x.cmp(&y),
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
        (PropertyValue::Int(x), PropertyValue::Float(y)) => {
            (x as f64).partial_cmp(&y).unwrap_or(Ordering::Equal)
        }
        (PropertyValue::Float(x), PropertyValue::Int(y)) => {
            x.partial_cmp(&(y as f64)).unwrap_or(Ordering::Equal)
        }
        (PropertyValue::Float(x), PropertyValue::Float(y)) => {
            x.partial_cmp(&y).unwrap_or(Ordering::Equal)
        }
        (PropertyValue::String(x), PropertyValue::String(y)) => x.cmp(&y),
        (PropertyValue::Bool(x), PropertyValue::Bool(y)) => x.cmp(&y),
        // `Duration` deliberately has no arm here (falls through to
        // `None` below) -- no defined ordering, only equality (see
        // `compare_values`'s docs on why months/days/seconds aren't
        // fungible enough to order against each other).
        (PropertyValue::Date(x), PropertyValue::Date(y)) => x.cmp(&y),
        _ => return None,
    })
}

/// General `lhs op rhs` for `ReturnExpr::Compare` -- unlike `compare()`
/// (a `PropertyValue`-vs-`Literal` comparison for pattern-level `WHERE`,
/// where the RHS is always a literal), both sides here are already-
/// evaluated `Value`s, since either can be a *computed* result (e.g. two
/// `date(...)` calls) with no `Literal` able to stand in for it.
/// Three-valued like `compare()`: `None` (Cypher's "unknown") for a null
/// operand, an operator with no meaning for the operands' types (e.g. `<`
/// between two `Duration`s), or a type mismatch.
fn compare_values(a: &Value, op: CompareOp, b: &Value) -> Option<bool> {
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return None;
    }
    match op {
        CompareOp::Eq => value_equal_ternary(a, b),
        CompareOp::Ne => value_equal_ternary(a, b).map(|eq| !eq),
        CompareOp::Lt => ordered_compare(a, b, |o| o == std::cmp::Ordering::Less),
        CompareOp::Le => ordered_compare(a, b, |o| o != std::cmp::Ordering::Greater),
        CompareOp::Gt => ordered_compare(a, b, |o| o == std::cmp::Ordering::Greater),
        CompareOp::Ge => ordered_compare(a, b, |o| o != std::cmp::Ordering::Less),
        CompareOp::StartsWith | CompareOp::EndsWith | CompareOp::Contains => {
            let (Some(s), Some(p)) = (as_arith_str(a), as_arith_str(b)) else {
                return None;
            };
            Some(match op {
                CompareOp::StartsWith => s.starts_with(p),
                CompareOp::EndsWith => s.ends_with(p),
                CompareOp::Contains => s.contains(p),
                _ => unreachable!("only StartsWith/EndsWith/Contains reach this arm"),
            })
        }
    }
}

/// `<`/`<=`/`>`/`>=` -- numeric operands are special-cased (not folded
/// into `value_partial_cmp` below) specifically so `NaN` compares as a
/// definite `false` on every operator, matching real Cypher (`0.0/0.0 >
/// 1` is `false`, not `null`) -- verified against Comparison2's
/// "Comparing NaN" scenario, which is what exposed `comparable_ordering`'s
/// `unwrap_or(Equal)` silently making `NaN >= x`/`NaN <= x` both `true`.
/// Every other type (`List`, `Date`, `String`, `Bool`, ...) has no NaN-like
/// "exists but is unorderable" value, so `None` there really does mean
/// Cypher's ordinary "unknown" (a null operand, a null found while
/// lexicographically comparing two lists, or a genuine type mismatch),
/// not something to special-case to `false`.
fn ordered_compare(
    a: &Value,
    b: &Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Option<bool> {
    if let (Some(x), Some(y)) = (value_as_f64(a), value_as_f64(b)) {
        return Some(x.partial_cmp(&y).map(pred).unwrap_or(false));
    }
    value_partial_cmp(a, b).map(pred)
}

/// `<`/`<=`/`>`/`>=` between two `List`s -- real Cypher orders lists
/// lexicographically: the first position where the two lists differ
/// decides the result; if every position up to the shorter list's length
/// is equal, the shorter list is "less". A `null` found at a
/// not-yet-decided position makes the *whole* comparison unknown (`None`)
/// -- lexicographic order can't skip past an undecided position to look
/// for a later one that happens to differ, since whether that later
/// position is even reached depends on what the undecided one turns out
/// to be. Verified element-by-element against every row of Comparison2's
/// "Comparing lists" scenario (`[1, 2] >= [1, null]` is `null`, not
/// `false`, even though `2 >= null` alone would also be `null` -- the
/// point is *why*: position 0 is equal, so position 1 is where the
/// answer would come from, and it's undecided). Delegates to
/// `comparable_ordering` for every non-list, non-numeric pair (`Date`,
/// `String`, `Bool`, ...), which has no list case to get wrong.
fn value_partial_cmp(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if matches!(a, Value::Null) || matches!(b, Value::Null) {
        return None;
    }
    if let (Value::List(xs), Value::List(ys)) = (a, b) {
        for (x, y) in xs.iter().zip(ys) {
            match value_partial_cmp(x, y) {
                Some(Ordering::Equal) => continue,
                other => return other,
            }
        }
        return Some(xs.len().cmp(&ys.len()));
    }
    comparable_ordering(a, b)
}

/// `=`/`<>`'s equality -- three-valued (`None` is Cypher's "unknown"),
/// recursing into `List`/`Map` element-by-element so a `null` *inside* a
/// list/map only makes the overall result unknown when it actually
/// matters, not automatically `false`/`true`: a length/key-set mismatch
/// is `false` outright (definite, regardless of any null present --
/// `{k: null} = {}` is `false`, not `null`, since the key sets alone
/// already prove inequality), a definite element mismatch anywhere makes
/// the whole comparison `false` (short-circuits, `false` outranks
/// `unknown` the same way `and3`/`or3` already rank them), and only once
/// every element is confirmed equal or unknown (never definitely
/// unequal) does an unknown element propagate to an unknown overall
/// result. Verified against every row of List3's and Comparison1's
/// list/map equality scenarios. Scalars fall back to numeric-cross-type-
/// aware equality (`1 = 1.0` is `true`, unlike `value_eq`'s plain
/// `PropertyValue` equality, which doesn't promote `Int`/`Float` against
/// each other) or plain `value_eq` for everything else (`Date`,
/// `Duration`'s component equality, `Node`/`Edge` identity, ...).
fn value_equal_ternary(a: &Value, b: &Value) -> Option<bool> {
    match (a, b) {
        (Value::Null, _) | (_, Value::Null) => None,
        (Value::List(xs), Value::List(ys)) => {
            if xs.len() != ys.len() {
                return Some(false);
            }
            fold_ternary_eq(xs.iter().zip(ys).map(|(x, y)| value_equal_ternary(x, y)))
        }
        (Value::Map(x), Value::Map(y)) => {
            if !x.keys().eq(y.keys()) {
                return Some(false);
            }
            fold_ternary_eq(x.iter().map(|(k, xv)| value_equal_ternary(xv, &y[k])))
        }
        _ => Some(values_equal_numeric_aware(a, b)),
    }
}

/// Combines a sequence of per-element three-valued equality results into
/// one overall result: any definite `Some(false)` wins outright
/// (short-circuits), otherwise `Some(true)` only if every element was a
/// definite `Some(true)`, else `None` (at least one element's equality
/// was itself unknown, and nothing else disproved the match).
fn fold_ternary_eq(mut results: impl Iterator<Item = Option<bool>>) -> Option<bool> {
    let mut saw_unknown = false;
    for r in results.by_ref() {
        match r {
            Some(false) => return Some(false),
            Some(true) => {}
            None => saw_unknown = true,
        }
    }
    if saw_unknown {
        None
    } else {
        Some(true)
    }
}

/// `=`/`<>`'s scalar leaf case: numeric cross-type promotion (`1 = 1.0`
/// is `true` in real Cypher, matching `compare()`'s existing `Int`-vs-
/// `Float` handling) that `value_eq`'s plain `PropertyValue` equality
/// doesn't give (`PropertyValue::Int(1) != PropertyValue::Float(1.0)`,
/// different enum variants) -- falls back to `value_eq` for every non-
/// numeric pair (`Date`, `Duration`'s component equality, `String`,
/// `Bool`, `Node`/`Edge` identity, ...), which is already correct for
/// those.
fn values_equal_numeric_aware(a: &Value, b: &Value) -> bool {
    match (as_arith_num(a), as_arith_num(b)) {
        (Some(ArithNum::Int(x)), Some(ArithNum::Int(y))) => x == y,
        (Some(ArithNum::Int(x)), Some(ArithNum::Float(y)))
        | (Some(ArithNum::Float(y)), Some(ArithNum::Int(x))) => x as f64 == y,
        (Some(ArithNum::Float(x)), Some(ArithNum::Float(y))) => x == y,
        _ => value_eq(a, b),
    }
}
