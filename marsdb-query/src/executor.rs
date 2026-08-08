use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering as AtomicOrdering},
    Arc,
};
use std::time::{Duration, Instant};

use marsdb_graph::{
    AdjEntry, Direction, Edge, EdgeId, GraphStore, Node, NodeId, PropertyValue, Txn,
    TzId as GraphTzId, WriteTransaction,
};

use crate::aggregate::{property_value_hash_key, value_hash_key, AggAcc, HashKey};
use crate::ast::{
    is_aggregate_name, is_percentile_name, ArithOp, CallClause, CallYield, CompareOp, Expr,
    Literal, MergeClause, NodePattern, Pattern, PropAccess, QuantifierKind, QueryClause, QueryPart,
    RelDirection, RemoveItem, ReturnExpr, ReturnItem, ReturnTail, SetItem, SortDir, Statement,
    Tail, UnwindClause, WithClause, WithExpr,
};
use crate::error::QueryError;
use crate::ir::{ExpandDirection, IndexSeekValue, LogicalPlan};
use crate::parse_helpers::validate_named_path_pattern;
use crate::planner::{
    apply_index_seeks, build_match_plan, pattern_all_vars, pattern_new_vars, plan_reversed_pattern,
};
use crate::procedure::{ProcedureProvider, ProcedureSignature};
use crate::result::QueryResult;
use crate::temporal;
use crate::value::{PathElem, Value};

mod arith;
mod scalar_fns;
mod temporal_fns;
mod value_cmp;

use arith::*;
use scalar_fns::*;
pub(crate) use temporal_fns::tz_from_graph;
use temporal_fns::*;
pub(crate) use value_cmp::comparable_ordering;
use value_cmp::*;

/// Hidden key used to correlate `OPTIONAL MATCH` results back to the outer
/// row that seeded them — never visible to user Cypher (not a valid
/// identifier prefix a parsed pattern could ever produce).
const OPTIONAL_SEED_IDX_KEY: &str = "__seed_idx";

/// Hidden key tagging whether a `MERGE`d row came from the create-path or
/// the match-path, consumed (and stripped) by `apply_merge_set` before the
/// row becomes visible to the rest of the query.
const MERGE_CREATED_KEY: &str = "__merge_created";

/// Cooperative cancellation handle for a running query. Clone it before
/// execution and call [`cancel`](Self::cancel) from another thread.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, AtomicOrdering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(AtomicOrdering::Acquire)
    }
}

/// Coarse, stable outcome category for telemetry. Error messages and query
/// text are deliberately excluded to avoid leaking user data through an
/// observer by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionOutcome {
    Success,
    /// The query text itself never parsed — see `QueryError::Syntax`.
    SyntaxError,
    /// The query parsed but is structurally invalid, independent of any
    /// data/parameters — see `QueryError::Semantic`.
    SemanticError,
    /// A real value (from stored data or a `$parameter`) turned out to be
    /// the wrong shape for what the query does with it — see
    /// `QueryError::Type`.
    TypeError,
    GraphError,
    UnboundVariable,
    MissingParameter,
    Cancelled,
    Timeout,
    ResourceLimit,
}

impl ExecutionOutcome {
    pub fn from_error(error: &QueryError) -> Self {
        match error {
            QueryError::Syntax(_) => Self::SyntaxError,
            QueryError::Semantic(_) => Self::SemanticError,
            QueryError::Type(_) => Self::TypeError,
            QueryError::Graph(_) => Self::GraphError,
            QueryError::UnboundVariable(_) => Self::UnboundVariable,
            QueryError::MissingParam(_) => Self::MissingParameter,
            QueryError::Cancelled => Self::Cancelled,
            QueryError::Timeout => Self::Timeout,
            QueryError::ResourceLimit(_) => Self::ResourceLimit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub elapsed: Duration,
    /// Unknown when parsing failed before a statement was available.
    pub statement_read_only: Option<bool>,
    pub result_rows: Option<usize>,
    pub relationship_expansions: u64,
    pub outcome: ExecutionOutcome,
}

/// Dependency-free callback adapter for sending execution events to an
/// application's logger, metrics collector, or tracing system.
#[derive(Clone)]
pub struct ExecutionObserver(Arc<dyn Fn(&ExecutionEvent) + Send + Sync>);

impl ExecutionObserver {
    pub fn new(callback: impl Fn(&ExecutionEvent) + Send + Sync + 'static) -> Self {
        Self(Arc::new(callback))
    }

    pub fn observe(&self, event: &ExecutionEvent) {
        // Observability must never turn a committed query into a reported
        // failure (or unwind through FFI callers), so observer panics are
        // contained at this boundary.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.0)(event)));
    }
}

impl std::fmt::Debug for ExecutionObserver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ExecutionObserver(..)")
    }
}

/// Per-statement safety limits and optional telemetry. Limit fields default
/// to `None`, preserving unlimited behavior for trusted embedded callers.
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub max_intermediate_rows: Option<usize>,
    pub max_result_rows: Option<usize>,
    pub max_relationship_expansions: Option<u64>,
    pub timeout: Option<Duration>,
    pub cancellation_token: Option<CancellationToken>,
    pub observer: Option<ExecutionObserver>,
    /// `None` (the default) means `CALL` always fails with "procedure not
    /// found" -- MarsDB ships no built-in procedures itself, see
    /// `procedure::ProcedureProvider`'s own docs.
    pub procedures: Option<crate::procedure::Procedures>,
    /// The statement's own `$name` parameters, verbatim -- every other
    /// `$param` position is already resolved to a concrete `Literal`
    /// before `Executor` ever sees the statement (`substitute_params`,
    /// run during `marsdb::prepare_statement`, well before this point),
    /// but a *standalone* `CALL proc` written with no parens at all (TCK's
    /// Call1 `[2]`/`[11]`, Call2 `[3]`) resolves each declared input from
    /// a same-named `$param` -- which declared names even exist isn't
    /// knowable until the procedure's signature is looked up here, at
    /// execution time (the registry itself, `procedures` above, isn't
    /// available any earlier either), so this is the one place `Executor`
    /// still needs the raw map instead of already-substituted AST nodes.
    pub params: HashMap<String, PropertyValue>,
}

struct ExecutionGuard<'a> {
    options: &'a ExecutionOptions,
    deadline: Option<Instant>,
    relationship_expansions: Cell<u64>,
    /// A relationship's *type* is immutable for its whole lifetime, so
    /// `type(r)` is one of the few things real Cypher still lets a
    /// statement read off `r` after `DELETE r` deleted it earlier in the
    /// same statement -- unlike properties/labels (mutable, and a genuine
    /// `DeletedEntityAccess` error, TCK's Return2 `[15]`-`[17]`), it
    /// needs no live record at all, just whatever type it had at match
    /// time. `delete_targets`/`delete_binding`/`delete_value` populate
    /// this right before actually deleting each edge; `type()`'s own
    /// evaluation (`Executor::eval_type_call`) falls back to it only when
    /// the ordinary live lookup fails. `RefCell`, not `&mut` -- `guard`
    /// is threaded everywhere as a shared reference, same interior-
    /// mutability precedent `relationship_expansions` above already sets.
    deleted_edge_types: RefCell<HashMap<EdgeId, String>>,
}

impl<'a> ExecutionGuard<'a> {
    fn new(options: &'a ExecutionOptions) -> Self {
        Self {
            options,
            deadline: options
                .timeout
                .and_then(|timeout| Instant::now().checked_add(timeout)),
            relationship_expansions: Cell::new(0),
            deleted_edge_types: RefCell::new(HashMap::new()),
        }
    }

    fn record_deleted_edge_type(&self, id: EdgeId, label: String) {
        self.deleted_edge_types.borrow_mut().insert(id, label);
    }

    fn deleted_edge_type(&self, id: EdgeId) -> Option<String> {
        self.deleted_edge_types.borrow().get(&id).cloned()
    }

    fn procedure_provider(&self) -> Option<&dyn ProcedureProvider> {
        self.options.procedures.as_ref().map(|p| p.0.as_ref())
    }

    fn checkpoint(&self) -> Result<(), QueryError> {
        if self
            .options
            .cancellation_token
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
        {
            return Err(QueryError::Cancelled);
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(QueryError::Timeout);
        }
        Ok(())
    }

    fn check_intermediate_rows(&self, rows: usize) -> Result<(), QueryError> {
        self.checkpoint()?;
        if self
            .options
            .max_intermediate_rows
            .is_some_and(|limit| rows > limit)
        {
            return Err(QueryError::ResourceLimit(format!(
                "intermediate row count {rows} exceeds configured maximum {}",
                self.options.max_intermediate_rows.unwrap()
            )));
        }
        Ok(())
    }

    fn check_result_rows(&self, rows: usize) -> Result<(), QueryError> {
        self.checkpoint()?;
        if self
            .options
            .max_result_rows
            .is_some_and(|limit| rows > limit)
        {
            return Err(QueryError::ResourceLimit(format!(
                "result row count {rows} exceeds configured maximum {}",
                self.options.max_result_rows.unwrap()
            )));
        }
        Ok(())
    }

    fn relationship_expansion(&self) -> Result<(), QueryError> {
        self.checkpoint()?;
        let count = self
            .relationship_expansions
            .get()
            .checked_add(1)
            .ok_or_else(|| {
                QueryError::ResourceLimit("relationship expansion counter overflow".into())
            })?;
        self.relationship_expansions.set(count);
        if self
            .options
            .max_relationship_expansions
            .is_some_and(|limit| count > limit)
        {
            return Err(QueryError::ResourceLimit(format!(
                "relationship expansion count {count} exceeds configured maximum {}",
                self.options.max_relationship_expansions.unwrap()
            )));
        }
        Ok(())
    }
}

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

struct ShortestPathSpec<'a> {
    direction: ExpandDirection,
    rel_labels: &'a [String],
    min_hops: u32,
    max_hops: Option<u32>,
}

struct VarExpandSpec<'a> {
    from_var: &'a str,
    to_var: &'a str,
    rel_labels: &'a [String],
    direction: ExpandDirection,
    min_hops: u32,
    max_hops: Option<u32>,
    /// Rel-vars bound by earlier fixed hops of the same pattern — see
    /// `LogicalPlan::VarExpand`'s own docs.
    exclude_edge_vars: &'a [String],
    /// See `LogicalPlan::VarExpand::exclude_edge_sets`'s own docs.
    exclude_edge_sets: &'a [String],
    /// See `LogicalPlan::VarExpand::exclude_edge_var`'s own docs.
    exclude_edge_var: &'a str,
    /// See `LogicalPlan::VarExpand::path_segment_var`'s own docs.
    path_segment_var: Option<&'a str>,
    /// See `LogicalPlan::VarExpand::rel_list_var`'s own docs.
    rel_list_var: Option<&'a str>,
    /// See `LogicalPlan::VarExpand::rel_props`'s own docs.
    rel_props: &'a [(String, ReturnExpr)],
}

struct MatchRelListSpec<'a> {
    from_var: &'a str,
    to_var: &'a str,
    rel_list_var: &'a str,
    rel_labels: &'a [String],
    direction: ExpandDirection,
    min_hops: u32,
    max_hops: Option<u32>,
}

struct PatternComprehensionSpec<'a> {
    path_var: &'a Option<String>,
    pattern: &'a Pattern,
    where_clause: &'a Option<Box<Expr>>,
    projection: &'a ReturnExpr,
}

struct IndexSeekSpec<'a> {
    var: &'a str,
    label: &'a str,
    prop: &'a str,
    value: &'a IndexSeekValue,
}

/// Read-only context `Executor::rewrite_composed_item` needs to resolve a
/// composed aggregate item's non-aggregate leaves -- see its own docs.
struct GroupFinishCtx<'a> {
    items: &'a [ReturnItem],
    key_bindings: &'a [Option<Binding>],
}

/// `ORDER BY`/`SKIP`/`LIMIT` bundled into one argument for
/// `execute_match` (clippy's `too_many_arguments`, capped at 7) --
/// mirrors `Statement::Match`'s own trailing fields, always applied in
/// this order regardless of which fields are actually present (`SKIP`
/// after `ORDER BY`, `LIMIT` after `SKIP`).
struct ResultModifiers<'a> {
    order_by: &'a Option<Vec<(ReturnExpr, SortDir)>>,
    skip: Option<i64>,
    limit: Option<i64>,
}

type BindingRow = HashMap<String, Binding>;
/// A fast-path hit: the finished (grouped/ordered/limited) rows plus the
/// clause's output names for `carried_vars`.
type FastCountResult = (Vec<BindingRow>, HashSet<String>);
type RowStream<'a> = Box<dyn Iterator<Item = Result<BindingRow, QueryError>> + 'a>;

/// Safety cap on unbounded variable-length traversal (`[:TYPE*0..]`) depth.
/// Hitting it errors rather than silently truncating — see `VarExpand`
/// evaluation. Expansion uses relationship uniqueness per path: a node may
/// be revisited and two distinct paths to the same node remain distinct, but
/// a relationship cannot occur twice in one path.
const VAR_EXPAND_DEPTH_CAP: u32 = 30;

pub struct Executor<'a> {
    store: &'a GraphStore,
    /// Lazily captured on first use, then reused for every no-arg
    /// `date()`/`localtime()`/`time()`/`localdatetime()`/`datetime()`
    /// call for the rest of this `Executor`'s lifetime (one per
    /// statement execution, see `Executor::new`'s callers) -- real
    /// Cypher's guarantee that every such call *within one query*
    /// returns the same value (see `temporal::NowSnapshot`'s docs).
    now: Cell<Option<temporal::NowSnapshot>>,
    /// `NodeId -> Node` memo, cleared at the start of every statement --
    /// both entry points (`execute_with_guard` and
    /// `execute_in_write_transaction_with_guard`, see their own reset
    /// lines) must do this, since `node_cache` is a field on `Executor`
    /// shared by both, not private to either. Serves *every* statement:
    /// read-only ones have one consistent snapshot for their whole
    /// duration, and write statements stay coherent by evicting a node's
    /// entry at every site that mutates or deletes that node's record
    /// (`uncache_node` -- SET/REMOVE on props or labels, node DELETE).
    /// An earlier version disabled the cache for write statements
    /// wholesale ("the write path was never the hot case") -- wrong for
    /// a predicate-driven bulk `DELETE r`, whose MATCH phase
    /// label-checks both endpoint nodes of every expanded edge: with the
    /// cache off that's a full node decode per *row* (~380ms of a ~490ms
    /// statement on the recommendations benchmark, users re-decoded
    /// ~150x each), with it on it's one decode per *distinct* node.
    /// Found via a real flamegraph both times: `get_node_in_txn`'s
    /// postcard decode of the full `NodeRecord` (every property, not
    /// just the ones a query reads) is the dominant term, much of it the
    /// *same* node decoded repeatedly (`RETURN n.a, n.b ORDER BY n.c`
    /// decodes `n` three times).
    ///
    /// Currently unbounded -- a statement that scans wide retains an
    /// `Rc<Node>` for every node it touches until the statement ends,
    /// where the pre-cache code decoded-and-dropped per row. On a
    /// dataset larger than RAM this can turn a slow query into an OOM
    /// risk; see mars-kvb for a size-capped follow-up (stop inserting
    /// past N entries, keep serving existing hits).
    node_cache: RefCell<HashMap<NodeId, Rc<Node>>>,
    /// Whether the executing statement is read-only. Gates the one memo
    /// entry kind that can go stale mid-write-statement: `prop_id_for`'s
    /// `None` ("name never interned") answers -- a later `CREATE`/`SET`
    /// in the same statement can intern that very name. `Some(id)`
    /// entries are immutable facts and are memoized unconditionally.
    read_only_stmt: Cell<bool>,
    /// Prop-name -> interned-id memo for the per-property read path
    /// (`lookup_prop`), cleared at every statement entry point alongside
    /// `node_cache`. See `read_only_stmt` for the `None`-entry gating.
    prop_id_memo: RefCell<HashMap<String, Option<u32>>>,
}

impl<'a> Executor<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        Self {
            store,
            now: Cell::new(None),
            node_cache: RefCell::new(HashMap::new()),
            read_only_stmt: Cell::new(false),
            prop_id_memo: RefCell::new(HashMap::new()),
        }
    }

    /// Cached equivalent of `GraphStore::get_node_in_txn` -- see
    /// `node_cache`'s own docs. Always caches: write statements keep the
    /// cache coherent by evicting a node's entry at every site that
    /// mutates or deletes that node's record (`uncache_node`), so a
    /// statement that never touches node records -- a predicate-driven
    /// bulk `DELETE r`, whose MATCH phase label-checks both endpoints of
    /// every expanded edge -- gets the same per-distinct-node decode a
    /// read-only statement does instead of a full record decode per row.
    fn get_node_cached(&self, txn: Txn, id: NodeId) -> Result<Option<Rc<Node>>, QueryError> {
        if let Some(cached) = self.node_cache.borrow().get(&id) {
            return Ok(Some(Rc::clone(cached)));
        }
        let node = GraphStore::get_node_in_txn(txn, id)?.map(Rc::new);
        if let Some(n) = &node {
            self.node_cache.borrow_mut().insert(id, Rc::clone(n));
        }
        Ok(node)
    }

    /// Evict one node from `node_cache`. Every write-path site that
    /// mutates or deletes an *existing* node's record (SET/REMOVE on
    /// props or labels, DELETE of the node) must call this with the id
    /// it just changed -- that eviction is the entire coherence story
    /// that lets `get_node_cached` serve write statements at all. Node
    /// *creation* sites don't need it: a fresh id can't have been cached.
    fn uncache_node(&self, id: NodeId) {
        self.node_cache.borrow_mut().remove(&id);
    }

    fn now_snapshot(&self) -> temporal::NowSnapshot {
        if let Some(n) = self.now.get() {
            return n;
        }
        let n = temporal::capture_now();
        self.now.set(Some(n));
        n
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
        self.execute_with_options(stmt, &ExecutionOptions::default())
    }

    pub fn execute_with_options(
        &self,
        stmt: &Statement,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, QueryError> {
        let started = Instant::now();
        let guard = ExecutionGuard::new(options);
        let result = self.execute_with_guard(stmt, &guard);
        Self::notify_observer(options, stmt, started, &guard, &result);
        result
    }

    fn execute_with_guard(
        &self,
        stmt: &Statement,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        crate::semantic::validate_statement(stmt)?;
        guard.checkpoint()?;
        // Fresh cache generation per statement -- an `Executor` is reused
        // across many statements (`execute_batch`, group commit), so a
        // cache that outlived one statement would return stale records
        // for a node a *later* statement mutated.
        self.node_cache.borrow_mut().clear();
        self.prop_id_memo.borrow_mut().clear();
        self.read_only_stmt.set(is_read_only(stmt));
        if let Statement::Explain(inner) = stmt {
            // Never opens a WriteTransaction, regardless of what `inner`
            // itself would otherwise mutate -- EXPLAIN describes a plan,
            // it never runs one.
            return self.execute_explain(inner);
        }
        if is_read_only(stmt) {
            let read_txn = self.store.begin_read()?;
            // No explicit commit/abort — a ReadTransaction is a pure
            // snapshot view with nothing to roll back; it releases on drop.
            return match stmt {
                Statement::Union { parts, all } => {
                    self.materialize_union(Txn::Read(&read_txn), parts, *all, guard)
                }
                Statement::Match {
                    clauses,
                    tail,
                    order_by,
                    skip,
                    limit,
                } => {
                    let skip = self.resolve_skip_limit(
                        Txn::Read(&read_txn),
                        skip.as_deref(),
                        "SKIP",
                        guard,
                    )?;
                    let limit = self.resolve_skip_limit(
                        Txn::Read(&read_txn),
                        limit.as_deref(),
                        "LIMIT",
                        guard,
                    )?;
                    self.execute_match(
                        Txn::Read(&read_txn),
                        clauses,
                        tail,
                        ResultModifiers {
                            order_by,
                            skip,
                            limit,
                        },
                        guard,
                    )
                }
                _ => unreachable!("is_read_only only returns true for Statement::Match/Union"),
            };
        }
        let write_txn = self.store.begin_write()?;
        let outcome = self.execute_in_write_transaction_validated(stmt, &write_txn, guard);
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

    /// Execute without committing against a caller-owned write transaction.
    /// The caller must commit or abort the transaction. This is the low-level
    /// primitive used by `marsdb::Transaction` for atomic multi-statement
    /// units of work.
    pub fn execute_in_write_transaction(
        &self,
        stmt: &Statement,
        write_txn: &WriteTransaction,
    ) -> Result<QueryResult, QueryError> {
        self.execute_in_write_transaction_with_options(
            stmt,
            write_txn,
            &ExecutionOptions::default(),
        )
    }

    pub fn execute_in_write_transaction_with_options(
        &self,
        stmt: &Statement,
        write_txn: &WriteTransaction,
        options: &ExecutionOptions,
    ) -> Result<QueryResult, QueryError> {
        let started = Instant::now();
        let guard = ExecutionGuard::new(options);
        let result = self.execute_in_write_transaction_with_guard(stmt, write_txn, &guard);
        Self::notify_observer(options, stmt, started, &guard, &result);
        result
    }

    fn execute_in_write_transaction_with_guard(
        &self,
        stmt: &Statement,
        write_txn: &WriteTransaction,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        crate::semantic::validate_statement(stmt)?;
        guard.checkpoint()?;
        // Same cache-generation reset as the top-level path
        // (`execute_with_guard`) -- this is a second, separate entry
        // point into statement execution (an explicit multi-statement
        // `Transaction`, or a group-commit loop, calls this directly with
        // an already-open `write_txn` instead of going through
        // `execute`/`execute_with_options`), and `node_cache` is a field
        // on `Executor`, not something either entry point owns privately
        // -- skipping the reset here left the flag/map from whatever this
        // `Executor` last did through the *other* entry point in effect.
        self.node_cache.borrow_mut().clear();
        self.prop_id_memo.borrow_mut().clear();
        self.read_only_stmt.set(is_read_only(stmt));
        if let Statement::Explain(inner) = stmt {
            // Same "never mutates" contract as the top-level path -- opens
            // its own ReadTransaction rather than touching the caller's
            // already-open `write_txn`, even when this runs inside an
            // explicit multi-statement transaction.
            return self.execute_explain(inner);
        }
        self.execute_in_write_transaction_validated(stmt, write_txn, guard)
    }

    /// `EXPLAIN <statement>` — always opens its own `ReadTransaction`
    /// (never the caller's write transaction, never a fresh write
    /// transaction of its own) so describing a plan can never itself
    /// mutate anything, no matter what `inner` would otherwise do.
    fn execute_explain(&self, inner: &Statement) -> Result<QueryResult, QueryError> {
        let read_txn = self.store.begin_read()?;
        let lines = crate::explain::explain_statement(inner, Txn::Read(&read_txn))?;
        Ok(QueryResult {
            columns: vec!["plan".to_string()],
            rows: lines
                .into_iter()
                .map(|line| vec![Value::Literal(Literal::String(line))])
                .collect(),
        })
    }

    fn notify_observer(
        options: &ExecutionOptions,
        stmt: &Statement,
        started: Instant,
        guard: &ExecutionGuard<'_>,
        result: &Result<QueryResult, QueryError>,
    ) {
        let Some(observer) = &options.observer else {
            return;
        };
        let (result_rows, outcome) = match result {
            Ok(result) => (Some(result.rows.len()), ExecutionOutcome::Success),
            Err(error) => (None, ExecutionOutcome::from_error(error)),
        };
        observer.observe(&ExecutionEvent {
            elapsed: started.elapsed(),
            statement_read_only: Some(is_read_only(stmt)),
            result_rows,
            relationship_expansions: guard.relationship_expansions.get(),
            outcome,
        });
    }

    fn execute_in_write_transaction_validated(
        &self,
        stmt: &Statement,
        write_txn: &WriteTransaction,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        match stmt {
            // Session statements never reach a correctly-wired call path:
            // `marsdb::Database` intercepts them before any executor entry
            // point. Reachable only through a caller with its own
            // transaction handling (`marsdb::Transaction::execute`, the
            // group-commit loop, or direct `Executor` use) -- where a
            // nested BEGIN/COMMIT/ROLLBACK has no session to act on and
            // must be a real error, not a silent no-op.
            Statement::Begin | Statement::Commit | Statement::Rollback => {
                Err(QueryError::Semantic(
                    "BEGIN/COMMIT/ROLLBACK are session statements -- valid only through \
                     Database::execute/execute_batch, not inside an explicit Transaction \
                     or a grouped batch"
                        .into(),
                ))
            }
            Statement::Create(patterns) => {
                guard.checkpoint()?;
                self.execute_create(write_txn, patterns, guard)
            }
            Statement::CreateIndex {
                label,
                prop,
                unique,
            } => {
                guard.checkpoint()?;
                GraphStore::create_index_in_txn(write_txn, label, prop, *unique)?;
                Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                })
            }
            Statement::Match {
                clauses,
                tail,
                order_by,
                skip,
                limit,
            } => {
                let skip =
                    self.resolve_skip_limit(Txn::Write(write_txn), skip.as_deref(), "SKIP", guard)?;
                let limit = self.resolve_skip_limit(
                    Txn::Write(write_txn),
                    limit.as_deref(),
                    "LIMIT",
                    guard,
                )?;
                self.execute_match(
                    Txn::Write(write_txn),
                    clauses,
                    tail,
                    ResultModifiers {
                        order_by,
                        skip,
                        limit,
                    },
                    guard,
                )
            }
            Statement::Explain(inner) => {
                // Only reachable if a future caller invokes this directly,
                // bypassing `execute_in_write_transaction_with_guard`'s own
                // interception above -- kept as a real (not `unreachable!`)
                // fallback so that stays true even if this function's
                // caller set ever changes, rather than becoming a latent
                // panic.
                self.execute_explain(inner)
            }
            Statement::Union { parts, all } => {
                self.materialize_union(Txn::Write(write_txn), parts, *all, guard)
            }
            Statement::StandaloneCall(call) => {
                self.eval_standalone_call(Txn::Write(write_txn), call, guard)
            }
        }
    }

    /// `CALL proc(args) [YIELD ...]` with nothing else in the statement
    /// (TCK's Call1 `[1]`/`[2]`/`[5]`, Call2 `[2]`/`[3]`) -- unlike the
    /// in-query form, this *is* the whole query: no outer rows to run the
    /// call once per, and no YIELD at all means "auto-yield every output"
    /// (`CallYield::Star`) rather than "discard everything."
    /// `QueryClause::Call`'s own in-query handling -- calls the procedure
    /// once per input row (TCK's Call1 `[3]`/`[4]`: even a `WHERE`-less,
    /// output-less call still runs once per already-matched row, same as
    /// any other reading clause). `None` (no `YIELD` at all) discards
    /// every output and keeps `row` unchanged -- see `CallClause::
    /// yield_items`'s own docs for why that's not the same as `Star`
    /// (which never actually reaches here, `queryCallSt`'s grammar has no
    /// `YIELD *` alternative). `Items` fans each input row out into one
    /// output row per matching procedure result row (same cross-join
    /// shape `eval_unwind` already gives its own per-row fan-out), each
    /// carrying `row`'s own bindings forward plus the newly yielded ones,
    /// filtered by `yieldItems`' own optional trailing `WHERE`.
    fn eval_call_clause(
        &self,
        txn: Txn,
        call: &CallClause,
        current_rows: &[BindingRow],
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in current_rows {
            guard.checkpoint()?;
            let (sig, proc_rows) = self.call_procedure(txn, call, row, guard)?;
            let Some(yield_items) = &call.yield_items else {
                out.push(row.clone());
                continue;
            };
            let names: Vec<String> = match yield_items {
                CallYield::Star => sig.outputs.clone(),
                CallYield::Items(items, _) => items
                    .iter()
                    .map(|(name, alias)| alias.clone().unwrap_or_else(|| name.clone()))
                    .collect(),
            };
            for proc_row in &proc_rows {
                let projected = project_call_row(&sig, proc_row, yield_items)?;
                let mut new_row = row.clone();
                for (name, value) in names.iter().zip(&projected) {
                    new_row.insert(name.clone(), value_to_binding_restore(value));
                }
                if let CallYield::Items(_, Some(where_expr)) = yield_items {
                    if self.eval_expr(txn, where_expr, &new_row, guard)? != Some(true) {
                        continue;
                    }
                }
                out.push(new_row);
                guard.check_intermediate_rows(out.len())?;
            }
        }
        Ok(out)
    }

    fn eval_standalone_call(
        &self,
        txn: Txn,
        call: &CallClause,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let empty_row = BindingRow::new();
        let (sig, proc_rows) = self.call_procedure(txn, call, &empty_row, guard)?;
        let yield_items = call.yield_items.clone().unwrap_or(CallYield::Star);
        let columns: Vec<String> = match &yield_items {
            CallYield::Star => sig.outputs.clone(),
            CallYield::Items(items, _) => items
                .iter()
                .map(|(name, alias)| alias.clone().unwrap_or_else(|| name.clone()))
                .collect(),
        };
        let mut rows = Vec::with_capacity(proc_rows.len());
        for proc_row in &proc_rows {
            rows.push(project_call_row(&sig, proc_row, &yield_items)?);
        }
        if let CallYield::Items(_, Some(where_expr)) = &yield_items {
            let mut filtered = Vec::with_capacity(rows.len());
            for row_values in &rows {
                let mut binding_row = BindingRow::new();
                for (col, v) in columns.iter().zip(row_values) {
                    binding_row.insert(col.clone(), value_to_binding_restore(v));
                }
                if self.eval_expr(txn, where_expr, &binding_row, guard)? == Some(true) {
                    filtered.push(row_values.clone());
                }
            }
            rows = filtered;
        }
        Ok(QueryResult { columns, rows })
    }

    /// Shared by `eval_standalone_call` and `QueryClause::Call`'s own
    /// in-query handling -- looks up `call.name`'s signature, resolves and
    /// type-checks its arguments against `row`'s already-bound variables
    /// (explicit args) or `guard.options.params` (the implicit-argument
    /// form, `call.args: None`), then invokes the provider. Returns the
    /// signature alongside the raw output rows since both callers need it
    /// again afterward (`sig.outputs`' names, for `YIELD *`/column
    /// naming).
    fn call_procedure(
        &self,
        txn: Txn,
        call: &CallClause,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<(ProcedureSignature, Vec<Vec<Value>>), QueryError> {
        let provider = guard.procedure_provider().ok_or_else(|| {
            QueryError::Semantic(format!(
                "procedure '{}' not found -- no procedure provider is configured",
                call.name
            ))
        })?;
        let sig = provider
            .signature(&call.name)
            .ok_or_else(|| QueryError::Semantic(format!("procedure '{}' not found", call.name)))?;
        let args = self.eval_call_args(txn, call, &sig, row, guard)?;
        let rows = provider.call(&call.name, &args)?;
        Ok((sig, rows))
    }

    fn eval_call_args(
        &self,
        txn: Txn,
        call: &CallClause,
        sig: &ProcedureSignature,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<Value>, QueryError> {
        let values: Vec<Value> = match &call.args {
            Some(args) => {
                if args.len() != sig.inputs.len() {
                    return Err(QueryError::Semantic(format!(
                        "'{}' expects {} argument(s), got {}",
                        call.name,
                        sig.inputs.len(),
                        args.len()
                    )));
                }
                args.iter()
                    .map(|a| self.eval_return_expr(txn, a, row, guard))
                    .collect::<Result<_, _>>()?
            }
            // The implicit-argument form (`CALL proc`, no parens) --
            // each declared input resolves from a same-named `$param`
            // (TCK's Call1 `[11]`, Call2 `[3]`); missing is a
            // `MissingParam`, same error real Cypher's own
            // `ParameterMissing`/`MissingParameter` reports.
            None => sig
                .inputs
                .iter()
                .map(|input_name| {
                    guard
                        .options
                        .params
                        .get(input_name)
                        .cloned()
                        .map(property_value_to_value)
                        .ok_or_else(|| QueryError::MissingParam(input_name.clone()))
                })
                .collect::<Result<_, _>>()?,
        };
        for (value, (input_name, declared_type)) in
            values.iter().zip(sig.inputs.iter().zip(&sig.input_types))
        {
            if !value_matches_declared_type(value, declared_type) {
                return Err(QueryError::Type(format!(
                    "'{}' argument '{input_name}' expects {declared_type}, got {value:?}",
                    call.name
                )));
            }
        }
        Ok(values)
    }

    fn execute_create(
        &self,
        write_txn: &WriteTransaction,
        patterns: &[Pattern],
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        // A standalone CREATE is a MATCH...CREATE tail run against a
        // single empty row -- `resolve_or_create_node` below never finds
        // any variable already bound in an empty `BindingRow`, so every
        // node token is fresh, exactly like standalone CREATE always was.
        // No trailing RETURN is possible on a standalone `CREATE` statement
        // (that's the `MATCH ... CREATE ... RETURN` tail's job instead), so
        // the resulting bindings are just discarded here.
        self.materialize_create(write_txn, patterns, &[BindingRow::new()], guard)?;
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
        guard: &ExecutionGuard<'_>,
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
                let mut prev_id =
                    self.resolve_or_create_node(write_txn, &pattern.start, &row, guard)?;
                if let Some(var) = &pattern.start.var {
                    row.insert(var.clone(), Binding::Node(prev_id));
                }
                for (rel, node) in &pattern.hops {
                    if rel.hop_range.is_some() {
                        return Err(QueryError::Semantic(
                            "CREATE doesn't support variable-length relationship patterns (e.g. [:TYPE*1..3])".into(),
                        ));
                    }
                    let node_id = self.resolve_or_create_node(write_txn, node, &row, guard)?;
                    if let Some(var) = &node.var {
                        row.insert(var.clone(), Binding::Node(node_id));
                    }

                    let rel_label = rel.rel_types.first().cloned().expect(
                        "CREATE relationship has exactly one type -- checked by \
                         semantic::bind_create_pattern",
                    );
                    let rel_props =
                        self.eval_props_to_values(Txn::Write(write_txn), &rel.props, &row, guard)?;
                    let (src, dst) = match rel.direction {
                        RelDirection::Right => (prev_id, node_id),
                        RelDirection::Left => (node_id, prev_id),
                        RelDirection::Either => {
                            return Err(QueryError::Semantic(
                                "CREATE requires a directed relationship (-> or <-), not an undirected pattern".into(),
                            ))
                        }
                    };
                    let edge_id =
                        GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
                    if let Some(var) = &rel.var {
                        row.insert(var.clone(), Binding::Edge(edge_id));
                    }
                    prev_id = node_id;
                }
            }
            out.push(row);
        }
        Ok(out)
    }

    /// A node pattern token reuses an existing binding iff it names a
    /// variable already bound in `row` (from a preceding MATCH/WITH) --
    /// restating labels/props on that token is rejected at compile time
    /// (`semantic::check_create_node_not_already_bound`), since silently
    /// dropping user-written labels/props would be a correctness trap.
    /// Anything else (no variable, or a variable not yet bound in this
    /// row) creates a brand-new node, exactly like standalone CREATE
    /// always has for every node token.
    fn resolve_or_create_node(
        &self,
        write_txn: &WriteTransaction,
        node: &NodePattern,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<NodeId, QueryError> {
        if let Some(var) = &node.var {
            if let Some(binding) = row.get(var) {
                let Binding::Node(id) = binding else {
                    return Err(QueryError::Type(format!(
                        "'{var}' is not a node — can't use it as a CREATE pattern endpoint"
                    )));
                };
                // Reusing an already-bound var with new labels/props is
                // rejected at compile time (`semantic::check_create_node_
                // not_already_bound`) -- unreachable here in practice.
                return Ok(*id);
            }
        }
        let labels: Vec<&str> = node.labels.iter().map(String::as_str).collect();
        let props = self.eval_props_to_values(Txn::Write(write_txn), &node.props, row, guard)?;
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<BTreeMap<String, PropertyValue>, QueryError> {
        props
            .iter()
            .filter_map(|(k, expr)| {
                let value = match self.eval_return_expr(txn, expr, row, guard) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                // `CREATE (n {prop: null})` never actually stores `prop`
                // at all in real Cypher -- the same "setting to null
                // removes/never-creates the property" rule
                // `apply_set_item`'s own `SET n.prop = null` handling
                // already has (see its docs), just never applied here
                // too. Observable via `keys(n)`/property enumeration
                // (TCK's Graph8 [8]) -- a stored `PropertyValue::Null`
                // still shows up as a key, where a real missing property
                // wouldn't.
                if matches!(value, Value::Null) {
                    return None;
                }
                let pv = match value_to_storable_property(&value).ok_or_else(|| {
                    QueryError::Type(format!(
                        "property '{k}' can't be stored -- MarsDB's node/edge properties are limited to null/\
                         bool/int/float/string/date/duration; a list/map/node/edge/path value (got {value:?}) \
                         isn't storable, matching PropertyValue's real, deliberately fixed set of variants (see \
                         its doc comment)"
                    ))
                }) {
                    Ok(pv) => pv,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok((k.clone(), pv)))
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in rows {
            guard.checkpoint()?;
            out.extend(self.merge_one_row(write_txn, clause, row, guard)?);
            guard.check_intermediate_rows(out.len())?;
        }
        self.apply_merge_set(write_txn, clause, &mut out, guard)?;
        Ok(out)
    }

    /// Whether any property expression across `clause.pattern` (the
    /// start node, and every hop's relationship + node) evaluates to
    /// null for this row -- see `merge_one_row`'s call site for why
    /// that's always a real error, never a value MERGE can act on.
    fn merge_pattern_has_null_property(
        &self,
        txn: Txn,
        clause: &MergeClause,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<bool, QueryError> {
        let any_null = |props: &[(String, ReturnExpr)]| -> Result<bool, QueryError> {
            for (_, expr) in props {
                if matches!(self.eval_return_expr(txn, expr, row, guard)?, Value::Null) {
                    return Ok(true);
                }
            }
            Ok(false)
        };
        if any_null(&clause.pattern.start.props)? {
            return Ok(true);
        }
        for (rel, node) in &clause.pattern.hops {
            if any_null(&rel.props)? || any_null(&node.props)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn merge_one_row(
        &self,
        write_txn: &WriteTransaction,
        clause: &MergeClause,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        // The bare-already-bound-start and reused-relationship-variable
        // cases are rejected at compile time (`semantic::bind_merge`),
        // not only here -- a zero-row MATCH would otherwise skip both
        // entirely even though real Cypher's `VariableAlreadyBound` is a
        // structural/scope error, not a data-dependent one. A completely
        // unconstrained, unbound token (bare `MERGE (a)`, no label/
        // property) is real, valid Cypher -- searches for/creates any
        // node with no constraints at all (TCK's Merge1 [1]), not an
        // error; an earlier version of this codebase treated it as an
        // "ambiguous shape" mistake to reject, which real Cypher's own
        // TCK disproves.
        for (rel, _node) in &clause.pattern.hops {
            if rel.hop_range.is_some() {
                return Err(QueryError::Semantic(
                    "MERGE doesn't support variable-length relationship patterns (e.g. [:TYPE*1..3])".into(),
                ));
            }
        }
        // `MERGE p = ...` -- give every anonymous token in the pattern a
        // synthetic name first (same convention ordinary MATCH's own
        // named-path capture uses, see `execute_match`'s `QueryClause::
        // Match` arm), so `assemble_path` below has a real row binding to
        // read at every position regardless of whether the user wrote one
        // -- then strip those synthetic keys back out before this row
        // becomes visible to the rest of the query. A no-`path_var` MERGE
        // clones `clause.pattern` once here rather than working with it
        // by reference throughout, so this function has exactly one
        // pattern to work from either way.
        let (pattern, synthesized) = if clause.path_var.is_some() {
            name_pattern_for_path(&clause.pattern)
        } else {
            (clause.pattern.clone(), HashSet::new())
        };
        let pattern = &pattern;
        // A MERGE pattern's own inline `{...}` property evaluating to
        // null can never be searched-or-created consistently: a null
        // property is never equal to anything (so the search half can
        // never find a node/edge that "has" it), but storing a
        // property as null is equivalent to not storing it at all (see
        // `apply_set_item`'s own SET-to-null convention) -- so the
        // create half would silently produce something that doesn't
        // structurally match the pattern that created it. Real Cypher's
        // MergeReadOwnWrites error, checked once per row (a property
        // expression can reference this row's other bindings, e.g.
        // `MERGE (n {x: m.missing})`).
        if self.merge_pattern_has_null_property(Txn::Write(write_txn), clause, row, guard)? {
            return Err(QueryError::Semantic(
                "MERGE pattern property is null — a MERGE's own {...} properties can never be \
                 null (searching for null never matches anything, but storing null is the same \
                 as not storing the property at all)"
                    .into(),
            ));
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
        let plan = apply_index_seeks(
            build_match_plan(pattern, &None, &carried_vars)?,
            Txn::Write(write_txn),
        )?;
        let found = self.eval_plan(
            Txn::Write(write_txn),
            &plan,
            std::slice::from_ref(row),
            guard,
        )?;
        if !found.is_empty() {
            return Ok(found
                .into_iter()
                .map(|mut r| {
                    if let Some(path_var) = &clause.path_var {
                        let path_binding = assemble_path(pattern, &r);
                        for key in &synthesized {
                            r.remove(key);
                        }
                        r.insert(path_var.clone(), path_binding);
                    }
                    tag_merge_created(r, false)
                })
                .collect());
        }

        // Nothing found — create exactly one new instance. Reuses
        // resolve_or_create_node, the same "reuse if the token's var is
        // already bound in the row, else create fresh" logic
        // Tail::Create/materialize_create already use.
        let mut new_row = row.clone();
        let start_id = self.resolve_or_create_node(write_txn, &pattern.start, &new_row, guard)?;
        if let Some(var) = &pattern.start.var {
            new_row.insert(var.clone(), Binding::Node(start_id));
        }
        // At most one hop (enforced at parse time) -- a plain `if let`,
        // not a loop, so there's no dangling "previous node" state to
        // thread once a 2nd+ hop is ever supported.
        if let Some((rel, node)) = pattern.hops.first() {
            let node_id = self.resolve_or_create_node(write_txn, node, &new_row, guard)?;
            if let Some(var) = &node.var {
                new_row.insert(var.clone(), Binding::Node(node_id));
            }
            let rel_label = rel.rel_types.first().cloned().expect(
                "MERGE relationship has exactly one type -- checked by semantic::bind_merge",
            );
            let rel_props =
                self.eval_props_to_values(Txn::Write(write_txn), &rel.props, &new_row, guard)?;
            // An undirected pattern (`-[r]-`) with nothing to match
            // defaults to an outgoing relationship when creating -- real
            // Cypher's own rule (TCK's Merge5 [11], "Use outgoing
            // direction when unspecified").
            let (src, dst) = match rel.direction {
                RelDirection::Right | RelDirection::Either => (start_id, node_id),
                RelDirection::Left => (node_id, start_id),
            };
            let edge_id =
                GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
            if let Some(var) = &rel.var {
                new_row.insert(var.clone(), Binding::Edge(edge_id));
            }
        }
        if let Some(path_var) = &clause.path_var {
            let path_binding = assemble_path(pattern, &new_row);
            for key in &synthesized {
                new_row.remove(key);
            }
            new_row.insert(path_var.clone(), path_binding);
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
        rows: &mut [BindingRow],
        guard: &ExecutionGuard<'_>,
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
                self.apply_set_item(Txn::Write(write_txn), write_txn, row, item, guard)?;
            }
        }
        Ok(())
    }

    fn execute_match(
        &self,
        txn: Txn,
        clauses: &[QueryClause],
        tail: &Option<Tail>,
        modifiers: ResultModifiers<'_>,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        self.execute_match_seeded(txn, clauses, tail, modifiers, None, guard)
    }

    /// `execute_match`'s general form -- `seed` is `None` for an ordinary
    /// top-level statement (nothing carried in, same as `execute_match`'s
    /// old fixed behavior) or `Some(row)` for a correlated `exists { MATCH
    /// ... RETURN ... }` subquery (`eval_exists_subquery`): the outer row's
    /// own bindings become this statement's starting `current_rows`/
    /// `carried_vars`, so a pattern referencing an outer-bound name (`(n)
    /// -->(m)` where `n` is already bound) seeds from it (`LogicalPlan::
    /// Seed`) instead of scanning fresh, exactly like a later clause in an
    /// ordinary multi-clause statement already does with an earlier
    /// clause's bindings.
    fn execute_match_seeded(
        &self,
        txn: Txn,
        clauses: &[QueryClause],
        tail: &Option<Tail>,
        modifiers: ResultModifiers<'_>,
        seed: Option<&BindingRow>,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let ResultModifiers {
            order_by,
            skip,
            limit,
        } = modifiers;
        // Threads bindings through each MATCH/UNWIND/WITH clause.
        // `carried_vars` tells the planner which of the next MATCH clause's
        // pattern variables are already bound (-> LogicalPlan::Seed) rather
        // than fresh (-> a scan). Starts empty (except for `seed`'s own
        // vars, if any): the first clause never has anything else carried
        // into it.
        let mut carried_vars: HashSet<String> = match seed {
            Some(row) => row.keys().cloned().collect(),
            None => HashSet::new(),
        };
        let mut current_rows: Vec<BindingRow> = vec![seed.cloned().unwrap_or_default()];
        // A plain, non-blocking RETURN can stop the final MATCH pipeline as
        // soon as SKIP+LIMIT rows have arrived (SKIP rows still have to
        // physically flow through the pipeline to be counted and dropped
        // below -- only the *count* the stream stops at grows, not
        // anything about what SKIP itself does). ORDER BY, DISTINCT,
        // aggregation, mutations, and WITH must still consume/materialize
        // their complete input before applying a final limit.
        let final_stream_limit = match (order_by, limit, tail) {
            (None, Some(limit), Some(Tail::Return(items, false))) if !has_aggregate(items) => {
                Some(skip.unwrap_or(0).max(0) as usize + limit.max(0) as usize)
            }
            _ => None,
        };
        for (clause_index, clause) in clauses.iter().enumerate() {
            let is_final_clause = clause_index + 1 == clauses.len();
            match clause {
                QueryClause::Match(part) => {
                    let plan_limit = is_final_clause
                        .then_some(final_stream_limit)
                        .flatten()
                        .filter(|_| !part.shortest_path && !part.optional && part.with.is_none());
                    current_rows = if part.shortest_path {
                        // Not a LogicalPlan/eval_plan traversal at all —
                        // see eval_shortest_path's docs.
                        self.eval_shortest_path(txn, part, &current_rows, guard)?
                    } else if let Some(path_var) = &part.path_var {
                        let (named_pattern, synthesized) = name_pattern_for_path(&part.pattern);
                        // A named path's own inline `WHERE` can reference
                        // the path variable itself (`WHERE length(p) =
                        // 1`, TCK's MatchWhere1 `[12]`/`[13]`) -- `p`
                        // isn't in the row until *after* `assemble_path`
                        // below, so (for a plain, non-`OPTIONAL` MATCH)
                        // it can't be pushed into the plan the way an
                        // ordinary pattern's `WHERE` is; applied as a
                        // post-filter instead, once every row really has
                        // `p`. `OPTIONAL MATCH` still pushes it into the
                        // plan -- its own null-padding semantics need the
                        // filter fused into the "did this seed row match
                        // anything" check `eval_optional_part` does, and
                        // a `WHERE` referencing `p` there is a narrower,
                        // untested-by-the-TCK edge case left as-is.
                        let defer_where = !part.optional && part.where_clause.is_some();
                        let plan_where = if defer_where {
                            &None
                        } else {
                            &part.where_clause
                        };
                        let plan = apply_index_seeks(
                            build_match_plan(&named_pattern, plan_where, &carried_vars)?,
                            txn,
                        )?;
                        let mut rows = if part.optional {
                            let new_vars = pattern_new_vars(&named_pattern, &carried_vars);
                            self.eval_optional_part(txn, &plan, &current_rows, &new_vars, guard)?
                        } else {
                            // `plan_limit`'s own early-stop assumes every
                            // emitted row is already a real, final row --
                            // not true when the WHERE filter above got
                            // deferred (a limited prefix could still get
                            // filtered further below), so it's skipped
                            // for that case (limiting instead happens
                            // naturally via the smaller `rows` this
                            // clause returns).
                            let limit = plan_limit.filter(|_| !defer_where);
                            self.eval_plan_with_limit(txn, &plan, &current_rows, guard, limit)?
                        };
                        for row in &mut rows {
                            let path_binding = assemble_path(&named_pattern, row);
                            for key in &synthesized {
                                row.remove(key);
                            }
                            row.insert(path_var.clone(), path_binding);
                        }
                        if defer_where {
                            let where_clause = part
                                .where_clause
                                .as_ref()
                                .expect("defer_where implies where_clause is Some");
                            let mut filtered = Vec::with_capacity(rows.len());
                            for row in rows {
                                if self.eval_expr(txn, where_clause, &row, guard)? == Some(true) {
                                    filtered.push(row);
                                }
                            }
                            rows = filtered;
                        }
                        rows
                    } else {
                        // Start-point selection: walk the pattern from its
                        // cheaper endpoint (see `plan_reversed_pattern`).
                        // Only this plain branch — a named path or
                        // shortestPath exposes traversal order, and MERGE's
                        // match phase stays as-written.
                        let reversed = plan_reversed_pattern(
                            &part.pattern,
                            &part.where_clause,
                            &carried_vars,
                            txn,
                        )?;
                        let pattern = reversed.as_ref().unwrap_or(&part.pattern);
                        let plan = apply_index_seeks(
                            build_match_plan(pattern, &part.where_clause, &carried_vars)?,
                            txn,
                        )?;
                        if part.optional {
                            let new_vars = pattern_new_vars(&part.pattern, &carried_vars);
                            self.eval_optional_part(txn, &plan, &current_rows, &new_vars, guard)?
                        } else {
                            // Aggregating-expansion fast path: when the
                            // plan+WITH match the counted-double-expand
                            // shape, the tight loop replaces BOTH the row
                            // materialization and the WITH's own grouping
                            // pass — so on a hit, this clause is done.
                            let tail_hint = if is_final_clause {
                                match (order_by, limit, tail) {
                                    (
                                        Some(keys),
                                        Some(tail_limit),
                                        Some(Tail::Return(items, false)),
                                    ) if keys.len() == 1 && !has_aggregate(items) => {
                                        let (key, dir) = &keys[0];
                                        Some((
                                            key,
                                            *dir,
                                            skip.unwrap_or(0).max(0) as usize
                                                + tail_limit.max(0) as usize,
                                        ))
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            };
                            if let Some((rows, out_names)) = self.try_fast_expand_expand_count(
                                txn,
                                &plan,
                                &part.with,
                                &current_rows,
                                tail_hint,
                                guard,
                            )? {
                                current_rows = rows;
                                carried_vars = out_names;
                                continue;
                            }
                            self.eval_plan_with_limit(txn, &plan, &current_rows, guard, plan_limit)?
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
                        guard,
                    )?;
                }
                QueryClause::Unwind(u) => {
                    current_rows = self.eval_unwind(txn, u, &current_rows, guard)?;
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &u.with,
                        current_rows,
                        HashSet::from([u.var.clone()]),
                        &mut carried_vars,
                        guard,
                    )?;
                }
                QueryClause::Call(call) => {
                    current_rows = self.eval_call_clause(txn, call, &current_rows, guard)?;
                    let new_vars: HashSet<String> = match &call.yield_items {
                        Some(CallYield::Items(items, _)) => items
                            .iter()
                            .map(|(name, alias)| alias.clone().unwrap_or_else(|| name.clone()))
                            .collect(),
                        // `Star` never reaches here (`queryCallSt`'s own
                        // grammar has no `YIELD *` alternative) and `None`
                        // binds nothing new.
                        Some(CallYield::Star) | None => HashSet::new(),
                    };
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &call.with,
                        current_rows,
                        new_vars,
                        &mut carried_vars,
                        guard,
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
                    current_rows = self.eval_merge(write_txn, m, &current_rows, guard)?;
                    let mut new_vars = pattern_all_vars(&m.pattern);
                    if let Some(path_var) = &m.path_var {
                        new_vars.insert(path_var.clone());
                    }
                    current_rows = self.apply_with_or_carry(
                        txn,
                        &m.with,
                        current_rows,
                        new_vars,
                        &mut carried_vars,
                        guard,
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
                        guard,
                    )?;
                }
                // `SET ... WITH ...` -- same real `.set_*_prop_in_txn`
                // write access `materialize_set`'s own per-row loop
                // already needs (guaranteed `Txn::Write` here for the
                // same reason its own docs give). Doesn't change any
                // row's bindings, only mutates the underlying graph --
                // `current_rows`/`carried_vars` both pass through
                // unchanged, the following `clause` (always a `WITH`,
                // see `set_as_clause`'s grammar) handles its own
                // projection/`WHERE`/`ORDER BY` normally from there.
                QueryClause::Set(items) => {
                    let write_txn = require_write_txn(txn);
                    for row in &current_rows {
                        for item in items {
                            self.apply_set_item(txn, write_txn, row, item, guard)?;
                        }
                    }
                }
                // `DELETE/DETACH DELETE ... WITH ...` -- same passthrough
                // reasoning as `QueryClause::Set` above (see
                // `delete_as_clause`'s grammar docs). Reuses the same
                // `delete_binding`/`delete_value` helpers `materialize_delete`
                // itself calls.
                QueryClause::Delete { items, detach } => {
                    let write_txn = require_write_txn(txn);
                    self.delete_targets(txn, write_txn, items, &current_rows, *detach, guard)?;
                }
                // `REMOVE ... WITH ...` -- same passthrough reasoning as
                // `QueryClause::Set` above (see `remove_as_clause`'s
                // grammar docs).
                QueryClause::Remove(items) => {
                    let write_txn = require_write_txn(txn);
                    for row in &current_rows {
                        for item in items {
                            apply_remove_item(self, write_txn, row, item)?;
                        }
                    }
                }
                // `CREATE ... WITH ...` -- unlike Set/Delete/Remove above,
                // this DOES change every row's bindings (each pattern's
                // own fresh/reused vars), so `current_rows` is replaced,
                // not passed through, and `carried_vars` is extended
                // directly (no bundled `.with` field on this variant to
                // route through `apply_with_or_carry` the way `Merge`
                // does above -- the following `WITH` is its own separate
                // `QueryClause::With` entry, picked up by this same loop's
                // next iteration, which needs `carried_vars` to already
                // reflect these new names by then).
                QueryClause::Create(patterns) => {
                    let write_txn = require_write_txn(txn);
                    current_rows =
                        self.materialize_create(write_txn, patterns, &current_rows, guard)?;
                    carried_vars.extend(patterns.iter().flat_map(pattern_all_vars));
                }
            }
            guard.check_intermediate_rows(current_rows.len())?;
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
            let skip_n = skip.unwrap_or(0).max(0) as usize;
            if skip_n > 0 {
                current_rows.drain(0..skip_n.min(current_rows.len()));
            }
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
                if let Some(ob) = order_by {
                    // DISTINCT (like aggregation) can drop rows, breaking
                    // the 1:1 correspondence `apply_order_by_with_scope`
                    // needs between `current_rows` and the projected
                    // output -- ORDER BY after DISTINCT can only sort the
                    // post-projection, post-dedup result, same as the
                    // aggregating case just below.
                    if !has_aggregate(items) && !distinct {
                        let projected =
                            self.materialize_return(txn, items, &current_rows, *distinct, guard)?;
                        order_by_pre_applied = true;
                        self.apply_order_by_with_scope(
                            txn,
                            &current_rows,
                            projected,
                            ob,
                            skip,
                            limit,
                        )?
                    } else if !distinct {
                        order_by_pre_applied = true;
                        self.materialize_aggregating_return_with_order(
                            txn,
                            items,
                            &current_rows,
                            ob,
                            (skip, limit),
                            guard,
                        )?
                    } else {
                        self.materialize_return(txn, items, &current_rows, *distinct, guard)?
                    }
                } else {
                    self.materialize_return(txn, items, &current_rows, *distinct, guard)?
                }
            }
            Some(Tail::ReturnStar(distinct)) => {
                let items = return_star_items(carried_vars.iter().cloned())?;
                let projected =
                    self.materialize_return(txn, &items, &current_rows, *distinct, guard)?;
                if let Some(ob) = order_by {
                    if !distinct {
                        order_by_pre_applied = true;
                        self.apply_order_by_with_scope(
                            txn,
                            &current_rows,
                            projected,
                            ob,
                            skip,
                            limit,
                        )?
                    } else {
                        projected
                    }
                } else {
                    projected
                }
            }
            Some(Tail::Delete(vars, ret)) => {
                self.materialize_delete(txn, vars, &current_rows, false, ret, guard)?
            }
            Some(Tail::DetachDelete(vars, ret)) => {
                self.materialize_delete(txn, vars, &current_rows, true, ret, guard)?
            }
            Some(Tail::Set(items, ret)) => {
                self.materialize_set(txn, items, &current_rows, ret, guard)?
            }
            Some(Tail::Remove(items, ret)) => {
                self.materialize_remove(txn, items, &current_rows, ret, guard)?
            }
            Some(Tail::Create(patterns, ret)) => {
                let updated_rows = self.materialize_create(
                    require_write_txn(txn),
                    patterns,
                    &current_rows,
                    guard,
                )?;
                match ret {
                    Some(rt) => {
                        self.materialize_return(txn, &rt.items, &updated_rows, rt.distinct, guard)?
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
                let tail_items: Option<&[ReturnItem]> = match tail {
                    Some(Tail::Return(items, _)) => Some(items),
                    _ => None,
                };
                result.rows = apply_order_by(
                    result.rows,
                    &result.columns,
                    order_by,
                    tail_items,
                    skip,
                    limit,
                )?;
            }
        } else if distinct_return {
            // The pre-truncate above was skipped for exactly this case --
            // apply SKIP/LIMIT now, after materialize_return's dedup,
            // instead.
            let skip_n = skip.unwrap_or(0).max(0) as usize;
            if skip_n > 0 {
                result.rows.drain(0..skip_n.min(result.rows.len()));
            }
            if let Some(count) = limit {
                result.rows.truncate(count.max(0) as usize);
            }
        }
        guard.check_result_rows(result.rows.len())?;
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let Some(with) = with else {
            carried_vars.extend(new_vars);
            return Ok(rows);
        };
        // `WITH *` -- expand to every name already carried into this
        // clause *plus* whatever this same clause's own pattern just
        // bound (`new_vars`, e.g. MERGE's own target -- `carried_vars`
        // alone wouldn't have that yet, since it's only ever updated at
        // this function's very end). `with_owned` only exists to give
        // the rest of this function a `&WithClause` with `items` already
        // containing the expanded names, without touching any of its
        // other fields (`order_by`/`skip`/`limit`/`distinct`/
        // `where_clause` all stay exactly as parsed).
        let with_owned;
        let with: &WithClause = if with.star {
            // A `HashSet` union, not a plain chain -- `new_vars` can
            // legitimately overlap with `carried_vars` (e.g. `MATCH (a)
            // MERGE (a)-[:R]->(b)` reuses the already-bound `a`), and a
            // raw chain would double it up into two identical columns.
            let star_items = with_star_items(carried_vars.union(&new_vars).cloned());
            let mut owned = with.clone();
            let mut items = star_items;
            items.extend(owned.items);
            owned.items = items;
            with_owned = owned;
            &with_owned
        } else {
            with
        };
        let with_skip = self.resolve_skip_limit(txn, with.skip.as_ref(), "SKIP", guard)?;
        let with_limit = self.resolve_skip_limit(txn, with.limit.as_ref(), "LIMIT", guard)?;
        let rows = if let Some(with_order_by) = with
            .order_by
            .as_ref()
            .filter(|_| has_aggregate(&with.items))
        {
            // `materialize_aggregating_with_with_order` folds its own
            // extra composed ORDER BY keys through the same grouping pass
            // as `with.items` -- also covers `with.distinct` correctly
            // without any extra handling here, since grouping already
            // makes every output row unique by its own grouping-key
            // columns (see that function's `RETURN`-side twin's own docs
            // on why that makes `DISTINCT` a no-op downstream of
            // aggregation).
            self.materialize_aggregating_with_with_order(
                txn,
                &with.items,
                &rows,
                with_order_by,
                (with_skip, with_limit),
                guard,
            )?
        } else {
            // Only cloned when actually needed below (ORDER BY on a
            // non-aggregating, non-`DISTINCT` WITH) -- avoids the copy on
            // every other WITH shape.
            let pre_with_rows = (with.order_by.is_some() && !with.distinct).then(|| rows.clone());
            let mut rows = self.materialize_with(txn, with, &rows, guard)?;
            if let Some(with_order_by) = &with.order_by {
                // Only a non-aggregating, non-`DISTINCT` WITH keeps a 1:1
                // row correspondence with its pre-WITH input -- see
                // `apply_order_by_bindings`'s own docs on why that's
                // exactly when ORDER BY can also see the pre-WITH scope.
                rows = self.apply_order_by_bindings(
                    txn,
                    rows,
                    pre_with_rows.as_deref(),
                    &with.items,
                    with_order_by,
                    (with_skip, with_limit),
                )?;
            } else {
                let skip_n = with_skip.unwrap_or(0).max(0) as usize;
                if skip_n > 0 {
                    rows.drain(0..skip_n.min(rows.len()));
                }
                if let Some(with_limit) = with_limit {
                    rows.truncate(with_limit.max(0) as usize);
                }
            }
            rows
        };
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let mut out = Vec::new();
        for row in rows {
            let source_value = self.eval_return_expr(txn, &clause.source.0, row, guard)?;
            let elements: Vec<Binding> = match source_value {
                Value::List(items) => items.iter().map(value_to_binding_restore).collect(),
                // `UNWIND null AS x` behaves like unwinding an empty list
                // (zero rows) in real Cypher, not an error.
                Value::Null => Vec::new(),
                other => {
                    return Err(QueryError::Type(format!(
                        "UNWIND needs a list, got {other:?}"
                    )))
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
                if self.eval_with_expr(txn, where_clause, &row, guard)? == Some(true) {
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
        guard: &ExecutionGuard<'_>,
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
        let rel_labels = &rel.rel_types;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let start_id = require_bound_node(row, start_var)?;
            let end_id = require_bound_node(row, end_var)?;
            let path = self.shortest_path_between(
                txn,
                start_id,
                end_id,
                ShortestPathSpec {
                    direction,
                    rel_labels,
                    min_hops,
                    max_hops,
                },
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
                if self.eval_expr(txn, where_clause, &row, guard)? == Some(true) {
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
        spec: ShortestPathSpec<'_>,
    ) -> Result<Option<Vec<PathBinding>>, QueryError> {
        if start == end && spec.min_hops == 0 {
            return Ok(Some(vec![PathBinding::Node(start)]));
        }
        let cap = spec.max_hops.unwrap_or(VAR_EXPAND_DEPTH_CAP);
        let mut parent: HashMap<NodeId, (NodeId, EdgeId)> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        visited.insert(start);
        let mut frontier = vec![start];
        let mut depth = 0u32;
        while depth < cap && !frontier.is_empty() {
            depth += 1;
            let mut next_frontier = Vec::new();
            for node in frontier {
                for entry in neighbors_for_direction(txn, node, spec.direction, spec.rel_labels)? {
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let is_aggregating = has_aggregate(&with.items);
        let mut out = if !is_aggregating {
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let mut new_row = BindingRow::new();
                for (i, item) in with.items.iter().enumerate() {
                    let name = with_item_output_name((i, item));
                    let binding = self.item_binding(txn, &item.expr, row, guard)?;
                    new_row.insert(name, binding);
                }
                out.push(new_row);
            }
            out
        } else {
            validate_return_items(&with.items)?;
            let grouped = self.resolve_grouped_rows(txn, &with.items, rows, guard)?;
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
            if is_aggregating {
                // Aggregation collapses many input rows into one group --
                // there's no single pre-WITH row left to fall back to, so
                // (matching real Cypher) WHERE only sees the grouped/
                // aggregated names, same as `RETURN`'s own aggregate WHERE.
                for row in out {
                    if self.eval_with_expr(txn, where_clause, &row, guard)? == Some(true) {
                        filtered.push(row);
                    }
                }
            } else {
                // Real Cypher lets a `WITH x AS y WHERE ...` immediately
                // following see *both* the pre-WITH binding (`x`) and the
                // new alias (`y`) -- confirmed via the TCK's own
                // `WithWhere7` scenarios. New aliases shadow same-named
                // old bindings on conflict. Still true with `DISTINCT` --
                // unlike aggregation, `DISTINCT` alone doesn't collapse
                // several pre-WITH rows into one *ambiguous* group; it's
                // a dedup applied to the *surviving*, still individually-
                // real rows, which is why the dedup itself happens below,
                // after this filter, not before it (TCK's WithWhere1
                // `[2]`: `WITH DISTINCT a.name2 AS name WHERE a.name2 =
                // 'B'` needs `a` from the row that produced each
                // candidate `name`, not just `name` itself).
                for (row, new_row) in rows.iter().zip(out) {
                    let mut merged = row.clone();
                    merged.extend(new_row.iter().map(|(k, v)| (k.clone(), v.clone())));
                    if self.eval_with_expr(txn, where_clause, &merged, guard)? == Some(true) {
                        filtered.push(new_row);
                    }
                }
            }
            out = filtered;
        }
        if with.distinct {
            out = dedup_binding_rows(&with.items, out)?;
        }
        Ok(out)
    }

    /// `materialize_aggregating_return_with_order`'s `WITH`-side twin --
    /// same "fold extra composed ORDER BY keys through the same grouping
    /// pass as `with_items` themselves" approach (TCK's WithOrderBy4
    /// `[16]`-`[18]`), just producing `Vec<BindingRow>` (preserving graph
    /// identity for whatever clause comes after this `WITH`) instead of a
    /// final `QueryResult` -- the extra keys' own values are only ever
    /// used for sorting here, never carried into the output rows.
    fn materialize_aggregating_with_with_order(
        &self,
        txn: Txn,
        with_items: &[ReturnItem],
        rows: &[BindingRow],
        order_by: &[(ReturnExpr, SortDir)],
        skip_limit: (Option<i64>, Option<i64>),
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let (skip, limit) = skip_limit;
        enum OrderKeySource {
            RealColumn(usize),
            Extra(usize),
        }
        let mut extra_exprs: Vec<ReturnExpr> = Vec::new();
        let order_by_source: Vec<OrderKeySource> = order_by
            .iter()
            .map(|(expr, _)| {
                match with_items
                    .iter()
                    .enumerate()
                    .position(|(i, it)| item_matches_leaf(expr, i, it))
                {
                    Some(i) => OrderKeySource::RealColumn(i),
                    None => {
                        let idx = extra_exprs.len();
                        extra_exprs.push(expr.clone());
                        OrderKeySource::Extra(idx)
                    }
                }
            })
            .collect();
        let extended_items: Vec<ReturnItem> = with_items
            .iter()
            .cloned()
            .chain(
                extra_exprs
                    .into_iter()
                    .map(|expr| ReturnItem { expr, alias: None }),
            )
            .collect();
        validate_return_items(&extended_items)?;
        let grouped = self.resolve_grouped_rows(txn, &extended_items, rows, guard)?;
        let real_len = with_items.len();
        let mut keyed: Vec<(Vec<Value>, BindingRow)> = Vec::with_capacity(grouped.len());
        for bindings in grouped {
            let (real, extra) = bindings.split_at(real_len);
            let real_values: Vec<Value> = real
                .iter()
                .map(|b| self.binding_to_value(txn, b))
                .collect::<Result<Vec<_>, _>>()?;
            let extra_values: Vec<Value> = extra
                .iter()
                .map(|b| self.binding_to_value(txn, b))
                .collect::<Result<Vec<_>, _>>()?;
            let keys: Vec<Value> = order_by_source
                .iter()
                .map(|src| match src {
                    OrderKeySource::RealColumn(i) => real_values[*i].clone(),
                    OrderKeySource::Extra(k) => extra_values[*k].clone(),
                })
                .collect();
            let real_row: BindingRow = with_items
                .iter()
                .enumerate()
                .zip(real)
                .map(|((i, item), binding)| (with_item_output_name((i, item)), binding.clone()))
                .collect();
            keyed.push((keys, real_row));
        }
        Ok(top_k_by(keyed, order_by, skip, limit)
            .into_iter()
            .map(|(_, row)| row)
            .collect())
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Binding, QueryError> {
        match expr {
            ReturnExpr::Var(v) => row
                .get(v)
                .cloned()
                .ok_or_else(|| QueryError::UnboundVariable(v.clone())),
            other => {
                let value = self.eval_return_expr(txn, other, row, guard)?;
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
        // `Some`, same length as `rows`, only for a non-aggregating,
        // non-`DISTINCT` WITH (1:1 row correspondence with the pre-WITH
        // input) -- lets ORDER BY see both the pre-WITH scope and the
        // new aliases, matching `where_clause`'s own merge (real Cypher:
        // `WITH a.count AS count ORDER BY a.count`, `a` isn't projected
        // but is still a valid sort key, TCK's With4 [6]). `None` for an
        // aggregating/`DISTINCT` WITH -- many pre-WITH rows collapse
        // into one output row there, so no single pre-WITH scope exists
        // to merge in.
        pre_with_rows: Option<&[BindingRow]>,
        with_items: &[ReturnItem],
        order_by: &[(ReturnExpr, SortDir)],
        skip_limit: (Option<i64>, Option<i64>),
    ) -> Result<Vec<BindingRow>, QueryError> {
        let (skip, limit) = skip_limit;
        // Same reasoning as `apply_order_by`'s `order_by_col` shortcut: an
        // ORDER BY item that repeats a WITH item's expression verbatim
        // (`WITH sum(x) AS s ORDER BY sum(x)`, TCK's WithOrderBy4 [11])
        // refers to that already-computed item, not a fresh expression --
        // look it up by its output name directly (works whether or not
        // that item has an alias) rather than re-evaluating the
        // expression, which would need pre-aggregation bindings that no
        // longer exist at this post-`materialize_with` point (an
        // aggregate call reaching `eval_projected_expr` always errors, by
        // design).
        let order_by_output: Vec<Option<String>> = order_by
            .iter()
            .map(|(expr, _)| {
                with_items
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.expr == *expr)
                    .map(with_item_output_name)
            })
            .collect();
        let mut keyed: Vec<(Vec<Value>, BindingRow)> = Vec::with_capacity(rows.len());
        for (i, row) in rows.into_iter().enumerate() {
            let mut value_map = self.binding_row_to_value_map(txn, &row)?;
            if let Some(pre) = pre_with_rows {
                // Pre-WITH names fill in gaps only -- a new alias with the
                // same name already occupies that key in `value_map` and
                // must keep winning (matches `materialize_with`'s own
                // "new aliases shadow same-named old bindings" rule).
                for (k, v) in self.binding_row_to_value_map(txn, &pre[i])? {
                    value_map.entry(k).or_insert(v);
                }
            }
            let keys = order_by
                .iter()
                .zip(&order_by_output)
                .map(|((expr, _), output_name)| match output_name {
                    Some(name) => Ok(value_map.get(name).cloned().unwrap_or(Value::Null)),
                    None => eval_projected_expr(expr, &value_map),
                })
                .collect::<Result<Vec<_>, _>>()?;
            keyed.push((keys, row));
        }
        Ok(top_k_by(keyed, order_by, skip, limit)
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
        skip: Option<i64>,
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
        let rows = top_k_by(keyed, order_by, skip, limit)
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
            Binding::Node(id) => {
                Value::Node((*deleted_entity_access(self.get_node_cached(txn, *id)?)?).clone())
            }
            Binding::Edge(id) => Value::Edge(deleted_entity_access(GraphStore::get_edge_in_txn(
                txn, *id,
            )?)?),
            Binding::Value(PropertyValue::Null) => Value::Null,
            Binding::Value(pv) => property_value_to_value(pv.clone()),
            Binding::List(items) => Value::List(items.clone()),
            Binding::Map(m) => Value::Map(m.clone()),
            Binding::Path(elems) => Value::Path(self.resolve_path_elems(txn, elems)?),
        })
    }

    /// `startNode(r)`/`endNode(r)` — unlike every other builtin function
    /// (`labels()`, `type()`, ...), which reads straight off the already-
    /// materialized `Value::Node`/`Edge` it's given, this needs a *second*
    /// `GraphStore` lookup: `Edge.src`/`.dst` are bare `NodeId`s, not full
    /// records. `call_builtin` (the free function every other builtin
    /// dispatches through) has no `Txn` to do that lookup with, so these
    /// two are special-cased here instead, before ever reaching it.
    fn start_or_end_node(
        &self,
        txn: Txn,
        which: &str,
        arg: Option<&Value>,
    ) -> Result<Value, QueryError> {
        match arg {
            None | Some(Value::Null) => Ok(Value::Null),
            Some(Value::Edge(e)) => {
                let id = if which == "startnode" { e.src } else { e.dst };
                let node = deleted_entity_access(self.get_node_cached(txn, id)?)?;
                Ok(Value::Node((*node).clone()))
            }
            Some(other) => Err(QueryError::Type(format!(
                "{which}() expects a relationship, got {other:?}"
            ))),
        }
    }

    /// `type(r)` -- unlike every other property/label access, real Cypher
    /// still allows this after `DELETE r` deleted the relationship
    /// earlier in the same statement (a relationship's type never
    /// changes, so there's nothing mutable a live record could be hiding
    /// -- unlike `labels()`/property access, which stay real
    /// `DeletedEntityAccess` errors, TCK's Return2 `[14]`-`[17]`). Tries
    /// the ordinary evaluation first; only on failure, and only for a
    /// bare `Var` bound to an edge, falls back to `guard`'s cached type
    /// from the moment it was deleted (`ExecutionGuard::
    /// deleted_edge_types`'s own docs). Any other failure (unbound
    /// variable, a genuinely wrong argument type, ...) propagates
    /// unchanged.
    fn eval_type_call(
        &self,
        txn: Txn,
        arg_expr: Option<&ReturnExpr>,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Value, QueryError> {
        let Some(arg_expr) = arg_expr else {
            return type_builtin(None);
        };
        match self.eval_return_expr(txn, arg_expr, row, guard) {
            Ok(v) => type_builtin(Some(&v)),
            Err(err) => {
                if let ReturnExpr::Var(v) = arg_expr {
                    if let Some(Binding::Edge(id)) = row.get(v) {
                        if let Some(label) = guard.deleted_edge_type(*id) {
                            return Ok(Value::Property(PropertyValue::String(label)));
                        }
                    }
                }
                Err(err)
            }
        }
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
                    PathBinding::Node(id) => PathElem::Node(
                        (*deleted_entity_access(self.get_node_cached(txn, *id)?)?).clone(),
                    ),
                    PathBinding::Edge(id) => PathElem::Edge(deleted_entity_access(
                        GraphStore::get_edge_in_txn(txn, *id)?,
                    )?),
                })
            })
            .collect()
    }

    /// Folds `rows` into groups keyed by every non-aggregate item's per-row
    /// `Binding` (via `item_binding`), then finishes each aggregating
    /// item's accumulator(s) per group. Returns one `Vec<Binding>` per
    /// output group, column-aligned with `items`. Shared by
    /// `materialize_with` and `materialize_return` — both already take the
    /// same `rows: &[BindingRow]` input type, so the grouping core stays
    /// in `Binding`-space (preserving graph identity for bare-var grouping
    /// keys) and each caller does its own thin final conversion.
    ///
    /// An item "aggregates" (`contains_aggregate`) in one of two shapes:
    /// purely (`count(a)`, `count(*)`, the only shape this used to
    /// support) or composed with other expressions (`count(a) + 3`, `a,
    /// count(a)` isn't this -- `a` is its own separate, non-aggregating
    /// item). Either way, `Group.accs[i]` holds one `AggAcc` per
    /// aggregate-bearing subexpression found in that item's tree
    /// (`collect_agg_nodes`'s order — empty for a non-aggregating item,
    /// exactly one for the purely-aggregating shape), and finishing a
    /// composed item evaluates its whole expression tree via
    /// `rewrite_composed_item` rather than just unwrapping a single
    /// accumulator. `validate_return_items` (which callers must run
    /// first) already guarantees every non-aggregate leaf inside a
    /// composed item's tree matches some *other* item's own top-level
    /// expression verbatim, so this function trusts that invariant rather
    /// than re-checking it.
    ///
    /// Grouping-key lookup is a hash-map lookup (`group_index`, keyed by
    /// `binding_hash_key`'s output — `Binding`/`PropertyValue` don't
    /// derive `Eq`/`Hash` themselves, `PropertyValue::Float` can't, so
    /// `HashKey` stands in for them; see its docs) into `groups`, which
    /// stays a plain `Vec` for insertion-order-stable output when there's
    /// no ORDER BY. O(1) average per row, not the O(rows × groups) linear
    /// scan this used to be — see BENCHMARKS.md for the measured
    /// before/after.
    fn resolve_grouped_rows(
        &self,
        txn: Txn,
        items: &[ReturnItem],
        rows: &[BindingRow],
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<Vec<Binding>>, QueryError> {
        struct Group {
            // Aligned to `items`: `Some` at a non-aggregating item's
            // index, `None` at an aggregating one's (whether purely
            // aggregating or composed) -- exactly one of
            // `key_bindings[i]`/`!accs[i].is_empty()` holds per `i`.
            key_bindings: Vec<Option<Binding>>,
            accs: Vec<Vec<AggAcc>>,
            row_count: i64,
        }
        fn fresh_accs(items: &[ReturnItem]) -> Vec<Vec<AggAcc>> {
            items
                .iter()
                .map(|item| {
                    let mut nodes = Vec::new();
                    collect_agg_nodes(&item.expr, &mut nodes);
                    nodes
                        .into_iter()
                        .map(|node| match node {
                            ReturnExpr::CountStar => AggAcc::identity("count", false),
                            ReturnExpr::Call { name, distinct, .. } => {
                                AggAcc::identity(name, *distinct)
                            }
                            _ => unreachable!(
                                "collect_agg_nodes only ever collects CountStar/aggregate Call nodes"
                            ),
                        })
                        .collect()
                })
                .collect()
        }
        // Computed once, not per row -- `item_agg_nodes[i][k]` is exactly
        // the node `group.accs[i][k]` accumulates for, every row.
        let item_agg_nodes: Vec<Vec<&ReturnExpr>> = items
            .iter()
            .map(|item| {
                let mut nodes = Vec::new();
                collect_agg_nodes(&item.expr, &mut nodes);
                nodes
            })
            .collect();

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
                key_bindings.push(if contains_aggregate(&item.expr) {
                    None
                } else {
                    Some(self.item_binding(txn, &item.expr, row, guard)?)
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
            for (i, nodes) in item_agg_nodes.iter().enumerate() {
                for (k, node) in nodes.iter().enumerate() {
                    match node {
                        // `count(*)` counts rows, not values -- folded
                        // unconditionally (no null-skip: there's no
                        // per-row expression to be null) via a dummy
                        // always-non-null argument, reusing `AggAcc::
                        // Count`'s existing fold logic instead of a
                        // separate no-accumulator path (see `fresh_accs`).
                        ReturnExpr::CountStar => {
                            group.accs[i][k].fold(&Value::Literal(Literal::Bool(true)))?;
                        }
                        ReturnExpr::Call { name, args, .. } => {
                            // Standard Cypher null-skipping: a null
                            // argument (e.g. an unmatched OPTIONAL MATCH
                            // variable) contributes to neither the
                            // accumulator nor its DISTINCT dedup set.
                            let value = self.eval_return_expr(txn, &args[0], row, guard)?;
                            if is_percentile_name(name) {
                                // percentileCont/percentileDisc's second
                                // argument (the percentile) is evaluated
                                // per row too -- in practice always a
                                // constant across the group, but nothing
                                // structurally requires that, so it's just
                                // evaluated fresh every row like any other
                                // expression rather than memoized once.
                                let percentile =
                                    self.eval_return_expr(txn, &args[1], row, guard)?;
                                if !matches!(value, Value::Null) {
                                    group.accs[i][k].fold_percentile(&value, &percentile)?;
                                }
                            } else if !matches!(value, Value::Null) {
                                group.accs[i][k].fold(&value)?;
                            }
                        }
                        _ => unreachable!(
                            "collect_agg_nodes only ever collects CountStar/aggregate Call nodes"
                        ),
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
        let no_key_items = items.iter().all(|item| contains_aggregate(&item.expr));
        if groups.is_empty() && no_key_items {
            groups.push(Group {
                key_bindings: vec![None; items.len()],
                accs: fresh_accs(items),
                row_count: 0,
            });
        }

        let mut out = Vec::with_capacity(groups.len());
        for mut group in groups {
            let ctx = GroupFinishCtx {
                items,
                key_bindings: &group.key_bindings,
            };
            let mut row_out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let binding = match &group.key_bindings[i] {
                    Some(b) => b.clone(),
                    None => {
                        let mut accs = std::mem::take(&mut group.accs[i]).into_iter();
                        let mut subst = HashMap::new();
                        let rewritten = self
                            .rewrite_composed_item(txn, &item.expr, &ctx, &mut accs, &mut subst)?;
                        value_to_binding(eval_projected_expr(&rewritten, &subst)?)
                    }
                };
                row_out.push(binding);
            }
            out.push(row_out);
        }
        Ok(out)
    }

    /// Finishing half of a composed aggregate item (`count(a) + 3`):
    /// rewrites `expr`'s tree into an equivalent one `eval_projected_expr`
    /// can evaluate without any further graph access, replacing every
    /// aggregate-bearing subexpression with a synthetic `Var` referencing
    /// its now-finished accumulator's value in `subst` (consumed from
    /// `accs` in `collect_agg_nodes`'s order, the same order `fresh_accs`/
    /// the per-row fold loop in `resolve_grouped_rows` built them in), and
    /// every non-aggregate `Var`/`Prop` leaf with a synthetic `Var`
    /// referencing whichever *other* item's own grouping-key `Binding` it
    /// structurally matches (`validate_return_items` already guarantees
    /// exactly one such match exists — never reached otherwise). Each
    /// substituted value gets its own fresh, guaranteed-unique slot name
    /// (`subst.len()` at insertion time), so nothing here can collide with
    /// a real Cypher identifier the user wrote.
    fn rewrite_composed_item(
        &self,
        txn: Txn,
        expr: &ReturnExpr,
        ctx: &GroupFinishCtx<'_>,
        accs: &mut std::vec::IntoIter<AggAcc>,
        subst: &mut HashMap<String, Value>,
    ) -> Result<ReturnExpr, QueryError> {
        if matches!(expr, ReturnExpr::CountStar)
            || matches!(expr, ReturnExpr::Call { name, .. } if is_aggregate_name(name))
        {
            let value = accs
                .next()
                .expect("accs is aligned with this same expr's collect_agg_nodes traversal order")
                .finish();
            let slot = format!("__slot{}", subst.len());
            subst.insert(slot.clone(), value);
            return Ok(ReturnExpr::Var(slot));
        }
        if matches!(expr, ReturnExpr::Var(_) | ReturnExpr::Prop(_)) {
            let j = ctx
                .items
                .iter()
                .enumerate()
                .position(|(i, it)| item_matches_leaf(expr, i, it) && !contains_aggregate(&it.expr))
                .expect(
                    "validate_return_items already checked this leaf matches a grouping-key item",
                );
            let binding = ctx.key_bindings[j]
                .clone()
                .expect("a non-aggregating item always has a key binding");
            let value = self.binding_to_value(txn, &binding)?;
            let slot = format!("__slot{}", subst.len());
            subst.insert(slot.clone(), value);
            return Ok(ReturnExpr::Var(slot));
        }
        Ok(match expr {
            ReturnExpr::Lit(lit) => ReturnExpr::Lit(lit.clone()),
            ReturnExpr::Call {
                name,
                args,
                distinct,
            } => ReturnExpr::Call {
                name: name.clone(),
                distinct: *distinct,
                args: args
                    .iter()
                    .map(|a| self.rewrite_composed_item(txn, a, ctx, accs, subst))
                    .collect::<Result<_, _>>()?,
            },
            ReturnExpr::Case { test, whens, else_ } => ReturnExpr::Case {
                test: test
                    .as_deref()
                    .map(|t| self.rewrite_composed_item(txn, t, ctx, accs, subst))
                    .transpose()?
                    .map(Box::new),
                whens: whens
                    .iter()
                    .map(|(w, t)| {
                        Ok::<_, QueryError>((
                            self.rewrite_composed_item(txn, w, ctx, accs, subst)?,
                            self.rewrite_composed_item(txn, t, ctx, accs, subst)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                else_: else_
                    .as_deref()
                    .map(|e| self.rewrite_composed_item(txn, e, ctx, accs, subst))
                    .transpose()?
                    .map(Box::new),
            },
            ReturnExpr::Arith(l, op, r) => ReturnExpr::Arith(
                Box::new(self.rewrite_composed_item(txn, l, ctx, accs, subst)?),
                *op,
                Box::new(self.rewrite_composed_item(txn, r, ctx, accs, subst)?),
            ),
            ReturnExpr::Neg(e) => ReturnExpr::Neg(Box::new(
                self.rewrite_composed_item(txn, e, ctx, accs, subst)?,
            )),
            ReturnExpr::ListLit(list_items) => ReturnExpr::ListLit(
                list_items
                    .iter()
                    .map(|i| self.rewrite_composed_item(txn, i, ctx, accs, subst))
                    .collect::<Result<_, _>>()?,
            ),
            ReturnExpr::Index(base, index) => ReturnExpr::Index(
                Box::new(self.rewrite_composed_item(txn, base, ctx, accs, subst)?),
                Box::new(self.rewrite_composed_item(txn, index, ctx, accs, subst)?),
            ),
            ReturnExpr::PropOf(base, prop) => ReturnExpr::PropOf(
                Box::new(self.rewrite_composed_item(txn, base, ctx, accs, subst)?),
                prop.clone(),
            ),
            ReturnExpr::Slice(base, start, end) => ReturnExpr::Slice(
                Box::new(self.rewrite_composed_item(txn, base, ctx, accs, subst)?),
                start
                    .as_deref()
                    .map(|s| self.rewrite_composed_item(txn, s, ctx, accs, subst))
                    .transpose()?
                    .map(Box::new),
                end.as_deref()
                    .map(|e| self.rewrite_composed_item(txn, e, ctx, accs, subst))
                    .transpose()?
                    .map(Box::new),
            ),
            // `where_clause`/`project` are deliberately left untouched
            // (cloned verbatim), not recursed into -- they run once per
            // *element* of `source`'s own already-rewritten result, in a
            // scope `eval_projected_expr`'s own `ListComp`/`Quantifier`
            // handling builds itself (the outer `subst` map plus a fresh
            // binding for `var`, per element). Rewriting a `Var`/`Prop`
            // leaf in here the same way `source` gets rewritten would
            // wrongly try to resolve the comprehension's own *local* loop
            // variable (`x`/`ok`) as if it had to be some other item's
            // grouping key -- there's no such item, since it's not an
            // outer reference at all (found via TCK's List11 [3]: `ALL(ok
            // IN collect(...) WHERE ok)` panicked trying to resolve `ok`
            // this way). `validate_composed_expr`'s own `ListComp` arm
            // already guarantees `project` has no aggregate to substitute
            // in the first place; `where_clause` is the same documented
            // scope gap `contains_aggregate` has everywhere else.
            ReturnExpr::ListComp {
                var,
                source,
                where_clause,
                project,
            } => ReturnExpr::ListComp {
                var: var.clone(),
                source: Box::new(self.rewrite_composed_item(txn, source, ctx, accs, subst)?),
                where_clause: where_clause.clone(),
                project: project.clone(),
            },
            ReturnExpr::Quantifier {
                kind,
                var,
                source,
                where_clause,
            } => ReturnExpr::Quantifier {
                kind: *kind,
                var: var.clone(),
                source: Box::new(self.rewrite_composed_item(txn, source, ctx, accs, subst)?),
                where_clause: where_clause.clone(),
            },
            ReturnExpr::MapLit(entries) => ReturnExpr::MapLit(
                entries
                    .iter()
                    .map(|(k, v)| {
                        Ok::<_, QueryError>((
                            k.clone(),
                            self.rewrite_composed_item(txn, v, ctx, accs, subst)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
            ),
            ReturnExpr::And(l, r) => ReturnExpr::And(
                Box::new(self.rewrite_composed_item(txn, l, ctx, accs, subst)?),
                Box::new(self.rewrite_composed_item(txn, r, ctx, accs, subst)?),
            ),
            ReturnExpr::Or(l, r) => ReturnExpr::Or(
                Box::new(self.rewrite_composed_item(txn, l, ctx, accs, subst)?),
                Box::new(self.rewrite_composed_item(txn, r, ctx, accs, subst)?),
            ),
            ReturnExpr::Xor(l, r) => ReturnExpr::Xor(
                Box::new(self.rewrite_composed_item(txn, l, ctx, accs, subst)?),
                Box::new(self.rewrite_composed_item(txn, r, ctx, accs, subst)?),
            ),
            ReturnExpr::Not(e) => ReturnExpr::Not(Box::new(
                self.rewrite_composed_item(txn, e, ctx, accs, subst)?,
            )),
            ReturnExpr::Compare(l, op, r) => ReturnExpr::Compare(
                Box::new(self.rewrite_composed_item(txn, l, ctx, accs, subst)?),
                *op,
                Box::new(self.rewrite_composed_item(txn, r, ctx, accs, subst)?),
            ),
            ReturnExpr::IsNull(e) => ReturnExpr::IsNull(Box::new(
                self.rewrite_composed_item(txn, e, ctx, accs, subst)?,
            )),
            ReturnExpr::In(needle, haystack) => ReturnExpr::In(
                Box::new(self.rewrite_composed_item(txn, needle, ctx, accs, subst)?),
                Box::new(self.rewrite_composed_item(txn, haystack, ctx, accs, subst)?),
            ),
            ReturnExpr::HasLabel(v, l) => ReturnExpr::HasLabel(v.clone(), l.clone()),
            ReturnExpr::PatternPredicate(p) => ReturnExpr::PatternPredicate(p.clone()),
            ReturnExpr::PatternComprehension { .. } => expr.clone(),
            ReturnExpr::ExistsPattern { .. } => expr.clone(),
            ReturnExpr::ExistsSubquery(_) => expr.clone(),
            ReturnExpr::Var(_) | ReturnExpr::Prop(_) | ReturnExpr::CountStar => {
                unreachable!("handled above, before this match")
            }
        })
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Option<bool>, QueryError> {
        Ok(match expr {
            WithExpr::And(l, r) => and3(
                self.eval_with_expr(txn, l, row, guard)?,
                self.eval_with_expr(txn, r, row, guard)?,
            ),
            WithExpr::Or(l, r) => or3(
                self.eval_with_expr(txn, l, row, guard)?,
                self.eval_with_expr(txn, r, row, guard)?,
            ),
            WithExpr::Not(e) => self.eval_with_expr(txn, e, row, guard)?.map(|b| !b),
            WithExpr::Compare(lhs, op, rhs) => {
                let lv = self.eval_return_expr(txn, lhs, row, guard)?;
                let rv = self.eval_return_expr(txn, rhs, row, guard)?;
                compare_values(&lv, *op, &rv)
            }
            // Always definite -- same reasoning as `Expr::IsNull`.
            WithExpr::IsNull(e) => Some(matches!(
                self.eval_return_expr(txn, e, row, guard)?,
                Value::Null
            )),
            // Unlike an ordinary MATCH's own `WHERE` (`Expr`), which folds
            // a bare pattern predicate into `Expr::Pattern` at parse time
            // (`return_expr_to_expr`), `WithExpr` has no such folding --
            // `WITH ... WHERE a.id = 0 AND (a)-->(b)` embeds it straight
            // as a `ReturnExpr::PatternPredicate` inside `Bare`/`And`/`Or`.
            // Special-cased here (rather than in `eval_return_expr`, which
            // errors on it -- a pattern predicate is only ever meaningful
            // as a predicate, never as a real projected value) so `WITH
            // ... WHERE` gets the same existential-search semantics MATCH's
            // own `WHERE` already has (TCK's WithWhere4 `[2]`).
            WithExpr::Bare(ReturnExpr::PatternPredicate(pattern)) => {
                Some(self.eval_pattern_predicate_exists(txn, pattern, row, guard)?)
            }
            WithExpr::Bare(e) => self.eval_return_expr_bool3(txn, e, row, guard)?,
        })
    }

    /// `WHERE (n)-[:REL]->()` etc (TCK's Pattern1) -- existential: true
    /// iff at least one real match of `pattern` exists, with every
    /// already-bound named endpoint (`n`, and `m` in `(n)-->(m)` when `m`
    /// is also bound by an earlier MATCH) held fixed to this row's own
    /// binding rather than searched freely. `semantic::
    /// validate_pattern_predicate` already rejected any named endpoint
    /// that ISN'T already bound (real Cypher's `UndefinedVariable`), so
    /// every named var here is safe to seed. Reuses the exact same
    /// `build_match_plan` "already-bound var -> Seed, not a fresh scan"
    /// mechanism `eval_merge`'s own "try as an ordinary MATCH first" half
    /// already relies on -- for a one-hop pattern this is a real
    /// connected-subgraph search (Expand + Filter), not an isolated
    /// per-node check. `Some(1)`-limited: existence is all that's needed,
    /// so there's no reason to enumerate every match. Shared by `Expr::
    /// Pattern` (an ordinary MATCH's own WHERE) and `WithExpr::Bare`'s
    /// `PatternPredicate` case (a WITH's own WHERE) -- same semantics
    /// either way, just reached from two different expression shapes.
    fn eval_pattern_predicate_exists(
        &self,
        txn: Txn,
        pattern: &Pattern,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<bool, QueryError> {
        let carried_vars: HashSet<String> = row.keys().cloned().collect();
        let plan = apply_index_seeks(build_match_plan(pattern, &None, &carried_vars)?, txn)?;
        let found =
            self.eval_plan_with_limit(txn, &plan, std::slice::from_ref(row), guard, Some(1))?;
        Ok(!found.is_empty())
    }

    /// `exists { MATCH ... RETURN ... }`'s "full" form (TCK's
    /// ExistentialSubquery2/3) -- runs `stmt` (always a `Statement::Match`,
    /// `semantic::validate_statement` rejects anything else reaching here
    /// and rejects every mutating clause inside it, so this only ever sees
    /// a real read-only pipeline) correlated against `row` via
    /// `execute_match_seeded`, then checks whether it produced at least
    /// one output row -- the inner RETURN's own projected *values* are
    /// never inspected, only whether the row exists at all, same as
    /// `eval_pattern_predicate_exists`/`Expr::Exists` above.
    fn eval_exists_subquery(
        &self,
        txn: Txn,
        stmt: &Statement,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<bool, QueryError> {
        let Statement::Match {
            clauses,
            tail,
            order_by,
            skip,
            limit,
        } = stmt
        else {
            unreachable!(
                "semantic::validate_statement only allows Statement::Match inside exists {{}}"
            )
        };
        let skip = self.resolve_skip_limit(txn, skip.as_deref(), "SKIP", guard)?;
        let limit = self.resolve_skip_limit(txn, limit.as_deref(), "LIMIT", guard)?;
        let result = self.execute_match_seeded(
            txn,
            clauses,
            tail,
            ResultModifiers {
                order_by,
                skip,
                limit,
            },
            Some(row),
            guard,
        )?;
        Ok(!result.rows.is_empty())
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
        guard: &ExecutionGuard<'_>,
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
        guard.check_intermediate_rows(tagged.len())?;
        let results = self.eval_plan(txn, plan, &tagged, guard)?;
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
            guard.check_intermediate_rows(out.len())?;
        }
        Ok(out)
    }

    fn eval_plan(
        &self,
        txn: Txn,
        plan: &LogicalPlan,
        seed: &[BindingRow],
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        self.eval_plan_with_limit(txn, plan, seed, guard, None)
    }

    fn eval_plan_with_limit(
        &self,
        txn: Txn,
        plan: &LogicalPlan,
        seed: &[BindingRow],
        guard: &ExecutionGuard<'_>,
        limit: Option<usize>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let stream = self.stream_plan(txn, plan, seed, guard, limit);
        match limit {
            Some(limit) => stream.take(limit).collect(),
            None => stream.collect(),
        }
    }

    /// Build a pull-based row pipeline. Each iterator owns only its current
    /// row (plus one relationship fan-out at an Expand), so scan/filter/
    /// expand chains no longer allocate a Vec at every logical-plan node.
    /// Blocking clause boundaries still collect this stream explicitly.
    fn stream_plan<'s>(
        &'s self,
        txn: Txn<'s>,
        plan: &'s LogicalPlan,
        seed: &'s [BindingRow],
        guard: &'s ExecutionGuard<'_>,
        scan_limit: Option<usize>,
    ) -> RowStream<'s> {
        match plan {
            LogicalPlan::Seed { var } => {
                debug_assert!(
                    seed.first().is_none_or(|row| row.contains_key(var)),
                    "Seed{{var: {var:?}}} planned for a var not present in the carried-forward rows"
                );
                Self::count_stream(Box::new(seed.iter().cloned().map(Ok)), guard)
            }
            LogicalPlan::AllNodesScan { var } => {
                self.stream_scan(txn, var, None, seed, guard, scan_limit)
            }
            LogicalPlan::NodeByLabelScan { var, label } => {
                self.stream_scan(txn, var, Some(label), seed, guard, scan_limit)
            }
            LogicalPlan::IndexSeek {
                var,
                label,
                prop,
                value,
            } => self.stream_index_seek(
                txn,
                IndexSeekSpec {
                    var,
                    label,
                    prop,
                    value,
                },
                seed,
                guard,
                scan_limit,
            ),
            LogicalPlan::Expand {
                input,
                from_var,
                to_var,
                rel_var,
                rel_labels,
                direction,
            } => {
                let input = self.stream_plan(txn, input, seed, guard, None);
                let stream = input.flat_map(move |res| -> RowStream<'s> {
                    let row = match res {
                        Ok(row) => row,
                        Err(error) => return Box::new(std::iter::once(Err(error))),
                    };
                    let from_id = match row.get(from_var) {
                        Some(Binding::Node(id)) => *id,
                        // A null binding has no neighbors and contributes
                        // no rows. Missing or structurally invalid bindings
                        // remain errors.
                        Some(Binding::Value(PropertyValue::Null)) => {
                            return Box::new(std::iter::empty())
                        }
                        _ => {
                            return Box::new(std::iter::once(Err(QueryError::UnboundVariable(
                                from_var.clone(),
                            ))))
                        }
                    };
                    match neighbors_for_direction(txn, from_id, *direction, rel_labels) {
                        Ok(entries) => Box::new(entries.into_iter().map(move |entry| {
                            guard.relationship_expansion()?;
                            let mut new_row = row.clone();
                            new_row.insert(to_var.clone(), Binding::Node(entry.other));
                            if let Some(rel_var) = rel_var {
                                new_row.insert(rel_var.clone(), Binding::Edge(entry.edge_id));
                            }
                            Ok(new_row)
                        })),
                        Err(error) => Box::new(std::iter::once(Err(error))),
                    }
                });
                Self::count_stream(Box::new(stream), guard)
            }
            LogicalPlan::VarExpand {
                input,
                from_var,
                to_var,
                rel_labels,
                direction,
                min_hops,
                max_hops,
                exclude_edge_vars,
                exclude_edge_sets,
                exclude_edge_var,
                path_segment_var,
                rel_list_var,
                rel_props,
            } => {
                let input = self.stream_plan(txn, input, seed, guard, None);
                let stream = input.flat_map(move |res| {
                    let rows = res.and_then(|row| {
                        self.expand_variable_row(
                            txn,
                            row,
                            VarExpandSpec {
                                from_var,
                                to_var,
                                rel_labels,
                                direction: *direction,
                                min_hops: *min_hops,
                                max_hops: *max_hops,
                                exclude_edge_vars,
                                exclude_edge_sets,
                                exclude_edge_var,
                                path_segment_var: path_segment_var.as_deref(),
                                rel_list_var: rel_list_var.as_deref(),
                                rel_props,
                            },
                            guard,
                        )
                    });
                    match rows {
                        Ok(rows) => Box::new(rows.into_iter().map(Ok)) as RowStream<'s>,
                        Err(error) => Box::new(std::iter::once(Err(error))),
                    }
                });
                Self::count_stream(Box::new(stream), guard)
            }
            LogicalPlan::MatchRelList {
                input,
                from_var,
                to_var,
                rel_list_var,
                rel_labels,
                direction,
                min_hops,
                max_hops,
            } => {
                let input = self.stream_plan(txn, input, seed, guard, None);
                let stream = input.filter_map(move |res| {
                    let row = match res {
                        Ok(row) => row,
                        Err(error) => return Some(Err(error)),
                    };
                    self.match_bound_rel_list_row(
                        row,
                        MatchRelListSpec {
                            from_var,
                            to_var,
                            rel_list_var,
                            rel_labels,
                            direction: *direction,
                            min_hops: *min_hops,
                            max_hops: *max_hops,
                        },
                    )
                    .transpose()
                });
                Self::count_stream(Box::new(stream), guard)
            }
            LogicalPlan::Filter { input, predicate } => {
                let input = self.stream_plan(txn, input, seed, guard, None);
                let stream = input.filter_map(move |res| {
                    let row = match res {
                        Ok(row) => row,
                        Err(error) => return Some(Err(error)),
                    };
                    if let Err(error) = guard.checkpoint() {
                        return Some(Err(error));
                    }
                    match self.eval_expr(txn, predicate, &row, guard) {
                        Ok(Some(true)) => Some(Ok(row)),
                        Ok(_) => None,
                        Err(error) => Some(Err(error)),
                    }
                });
                Self::count_stream(Box::new(stream), guard)
            }
        }
    }

    /// Wraps every `stream_plan` operator's output: counts produced rows
    /// against the guard's intermediate-row limit, and FUSES the stream
    /// after the first `Err` — `next()` returns `None` from then on, so
    /// the erroring operator (and everything beneath it) is never polled
    /// again. The operator closures in `stream_plan` rely on this instead
    /// of each tracking its own post-error `done` flag: after they emit an
    /// `Err`, this wrapper guarantees they're not resumed.
    fn count_stream<'s>(mut stream: RowStream<'s>, guard: &'s ExecutionGuard<'_>) -> RowStream<'s> {
        let mut produced = 0usize;
        let mut done = false;
        Box::new(std::iter::from_fn(move || {
            if done {
                return None;
            }
            let item = stream.next()?;
            if item.is_ok() {
                produced = match produced.checked_add(1) {
                    Some(produced) => produced,
                    None => {
                        done = true;
                        return Some(Err(QueryError::ResourceLimit(
                            "stream row counter overflow".into(),
                        )));
                    }
                };
                if let Err(error) = guard.check_intermediate_rows(produced) {
                    done = true;
                    return Some(Err(error));
                }
            } else {
                done = true;
            }
            Some(item)
        }))
    }

    /// Fast path for aggregating expansion chains -- one or two `Expand`
    /// hops feeding a `WITH` that groups by the final node and computes
    /// `count(*)` and/or `collect(<mid-node>.prop)`:
    ///
    /// ```text
    /// MATCH (s ...)-[:X]-(b)            WITH b, count(*) ...           (1 hop)
    /// MATCH (s ...)-[:X]-(a)-[:Y]-(b)   WITH b, count(*) ...           (2 hops)
    /// MATCH (s ...)-[:X]-(a)-[:Y]-(b)   WITH b, collect(a.p), count(*) (2 hops)
    /// ```
    ///
    /// Counts/collects in a tight loop over `neighbors_in_txn` instead of
    /// materializing a `BindingRow` per intermediate path. Motivation is
    /// measured, not assumed: the same algorithm hand-rolled runs in ~1ms
    /// where the generic pipeline takes ~100ms on the recommendations
    /// dataset (`marsdb/examples/csr_falsifier.rs`) -- the row machinery,
    /// not storage, is ~99% of that query's time; the first (2-hop count)
    /// entry measured ~25x end-to-end on that suite.
    ///
    /// Deliberately conservative: returns `Ok(None)` (generic path) for
    /// ANY shape it doesn't fully recognize. What it accepts:
    /// - plan = `[Filter*] Expand([Filter*] Expand(leaf))` or
    ///   `[Filter*] Expand(leaf)`, every expansion single-typed (or
    ///   untyped) and directed (no `Either`), leaf free of any
    ///   expansion/`Seed` (evaluated via the generic stream);
    /// - filters drawn only from the shapes `build_match_plan`
    ///   synthesizes here: `HasLabel` on the hop nodes, and the
    ///   edge-isomorphism `Not(VarEq(r2, r1))` between the two hops
    ///   (honored in-loop by skipping `e2.edge_id == e1.edge_id`);
    /// - `WITH` = `Var(final-node)` plus any mix of `count(*)` and
    ///   `collect(<mid-node>.prop)` (2-hop only, non-DISTINCT), no
    ///   `*`/`WHERE`, ORDER BY only on the count column;
    /// - no carried bindings entering the clause.
    ///
    /// `collect()` skips null/absent values (real Cypher's rule), reads
    /// the property through the per-prop directory path, and memoizes it
    /// per mid-node. Group and in-group encounter order both follow
    /// traversal order, matching the generic grouping pass's
    /// first-encounter semantics for ORDER BY ties and collect contents.
    ///
    /// `HasLabel` checks use per-label node-id sets loaded once via
    /// `NODE_LABEL_INDEX` -- O(label size) setup instead of a per-candidate
    /// record read in the hot loop.
    fn try_fast_expand_expand_count(
        &self,
        txn: Txn,
        plan: &LogicalPlan,
        with: &Option<WithClause>,
        current_rows: &[BindingRow],
        // When this MATCH is the statement's final clause and the tail is
        // a plain (non-aggregating, non-DISTINCT) RETURN whose ORDER
        // BY/SKIP/LIMIT ride on the count column, the hint lets the loop
        // sort groups and keep only skip+limit of them BEFORE building
        // any rows -- the generic tail then re-sorts and slices that tiny
        // prefix exactly (same key, same tie order), so semantics are
        // unchanged while the 6k-groups-for-a-LIMIT-5 case stops
        // materializing 6k rows. Measured motivation: inception's
        // remaining ~40ms was almost entirely this tail.
        tail_hint: Option<(&ReturnExpr, SortDir, usize)>,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Option<FastCountResult>, QueryError> {
        // -- clause-context checks --------------------------------------
        if current_rows.len() != 1 || !current_rows[0].is_empty() {
            return Ok(None);
        }
        let Some(with) = with else { return Ok(None) };
        if with.star || with.distinct || with.where_clause.is_some() || with.items.len() < 2 {
            return Ok(None);
        }

        // -- plan shape: 1 or 2 Expand stages over a non-expanding leaf --
        fn peel<'p>(mut plan: &'p LogicalPlan, preds: &mut Vec<&'p Expr>) -> &'p LogicalPlan {
            while let LogicalPlan::Filter { input, predicate } = plan {
                push_conjunct_refs(predicate, preds);
                plan = input;
            }
            plan
        }
        fn push_conjunct_refs<'p>(expr: &'p Expr, out: &mut Vec<&'p Expr>) {
            if let Expr::And(l, r) = expr {
                push_conjunct_refs(l, out);
                push_conjunct_refs(r, out);
            } else {
                out.push(expr);
            }
        }
        struct Stage<'p> {
            from: &'p str,
            to: &'p str,
            rel_var: Option<&'p str>,
            label: Option<&'p str>,
            dir: Direction,
            preds: Vec<&'p Expr>,
        }
        // Collected outermost-first, reversed to innermost-first below.
        let mut stages: Vec<Stage<'_>> = Vec::new();
        let mut cursor = plan;
        let leaf = loop {
            let mut preds = Vec::new();
            match peel(cursor, &mut preds) {
                LogicalPlan::Expand {
                    input,
                    from_var,
                    to_var,
                    rel_var,
                    rel_labels,
                    direction,
                } if stages.len() < 2 => {
                    let (Some(dir), Some(label)) =
                        (fast_direction(*direction), fast_label(rel_labels))
                    else {
                        return Ok(None);
                    };
                    stages.push(Stage {
                        from: from_var,
                        to: to_var,
                        rel_var: rel_var.as_deref(),
                        label,
                        dir,
                        preds,
                    });
                    cursor = input;
                }
                _ => {
                    if stages.is_empty() || plan_contains_expansion(cursor) {
                        return Ok(None);
                    }
                    // The leaf keeps its own filter chain (`cursor`, not
                    // the peeled node): a start-node predicate the planner
                    // pushed down (`WHERE m.title = ...` without an index)
                    // is just part of leaf evaluation, which runs through
                    // the generic stream anyway.
                    break cursor;
                }
            }
        };
        stages.reverse(); // innermost (hop 1) first
        if stages.len() == 2 && stages[1].from != stages[0].to {
            return Ok(None);
        }
        let final_to = stages.last().expect("at least one stage").to;
        let origin = stages[0].from;
        let mid_var = (stages.len() == 2).then(|| stages[0].to);

        // -- WITH-shape: Var(final) + {count(*) | collect(mid.prop)}* ----
        enum OutCol<'p> {
            Group,
            Count,
            Collect(&'p str), // mid-node property name
        }
        let mut cols: Vec<OutCol<'_>> = Vec::with_capacity(with.items.len());
        // The grouping key: either the chain's far end (collaborative
        // filtering) or its origin (matrix_review_counts groups by the
        // seed and counts its expansions).
        let mut group_seen = false;
        let mut group_by_origin = false;
        let mut count_seen = false;
        for item in &with.items {
            match &item.expr {
                ReturnExpr::Var(v) if v == final_to && !group_seen => {
                    group_seen = true;
                    cols.push(OutCol::Group);
                }
                ReturnExpr::Var(v) if v == origin && !group_seen => {
                    group_seen = true;
                    group_by_origin = true;
                    cols.push(OutCol::Group);
                }
                ReturnExpr::CountStar if !count_seen => {
                    count_seen = true;
                    cols.push(OutCol::Count);
                }
                ReturnExpr::Call {
                    name,
                    args,
                    distinct: false,
                } if name.eq_ignore_ascii_case("collect") => {
                    let [ReturnExpr::Prop(pa)] = args.as_slice() else {
                        return Ok(None);
                    };
                    let Some(mid) = mid_var else { return Ok(None) };
                    if pa.var != mid {
                        return Ok(None);
                    }
                    cols.push(OutCol::Collect(&pa.prop));
                }
                _ => return Ok(None),
            }
        }
        if !group_seen {
            return Ok(None);
        }
        let names: Vec<String> = with
            .items
            .iter()
            .enumerate()
            .map(with_item_output_name)
            .collect();
        let count_name = cols
            .iter()
            .position(|c| matches!(c, OutCol::Count))
            .map(|i| names[i].as_str());
        // ORDER BY: only "by the count column" (any direction) or absent.
        let mut pre_keep: Option<usize> = None;
        let count_sort: Option<SortDir> = match &with.order_by {
            None => {
                // No WITH-level ordering: the tail hint (final clause,
                // plain RETURN ordered by the count column) can take over.
                match tail_hint {
                    Some((key, dir, keep)) if with.skip.is_none() && with.limit.is_none() => {
                        let matches_count = match key {
                            ReturnExpr::Var(v) => count_name == Some(v.as_str()),
                            ReturnExpr::CountStar => count_seen,
                            _ => false,
                        };
                        if matches_count {
                            pre_keep = Some(keep);
                            Some(dir)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Some(keys) => {
                let [(key, dir)] = keys.as_slice() else {
                    return Ok(None);
                };
                let matches_count = match key {
                    ReturnExpr::Var(v) => count_name == Some(v.as_str()),
                    ReturnExpr::CountStar => count_seen,
                    _ => false,
                };
                if !matches_count {
                    return Ok(None);
                }
                Some(*dir)
            }
        };

        // -- predicate classification per stage --------------------------
        let mut stage_label_filters: Vec<Vec<&str>> = vec![Vec::new(); stages.len()];
        let mut isomorphism = false;
        for (i, stage) in stages.iter().enumerate() {
            for pred in &stage.preds {
                match pred {
                    Expr::HasLabel(v, l) if v == stage.to => stage_label_filters[i].push(l),
                    Expr::Not(inner) if i == 1 => {
                        match (&**inner, stages[0].rel_var, stage.rel_var) {
                            (Expr::VarEq(x, y), Some(r1), Some(r2))
                                if (x == r1 && y == r2) || (x == r2 && y == r1) =>
                            {
                                isomorphism = true;
                            }
                            _ => return Ok(None),
                        }
                    }
                    _ => return Ok(None),
                }
            }
        }

        // -- resolve everything the loop needs ---------------------------
        let skip = self.resolve_skip_limit(txn, with.skip.as_ref(), "SKIP", guard)?;
        let limit = self.resolve_skip_limit(txn, with.limit.as_ref(), "LIMIT", guard)?;
        let label_set = |label: &str| -> Result<std::collections::HashSet<u64>, QueryError> {
            Ok(
                GraphStore::all_node_ids_limited_in_txn(txn, Some(label), usize::MAX)?
                    .into_iter()
                    .map(|n| n.0)
                    .collect(),
            )
        };
        let stage_sets: Vec<Vec<std::collections::HashSet<u64>>> = stage_label_filters
            .iter()
            .map(|labels| labels.iter().map(|l| label_set(l)).collect())
            .collect::<Result<_, _>>()?;
        // Collected properties: resolve names to interned ids once.
        let collect_prop_ids: Vec<Option<u32>> = cols
            .iter()
            .map(|c| match c {
                OutCol::Collect(prop) => self.prop_id_for(txn, prop),
                _ => Ok(None),
            })
            .collect::<Result<_, _>>()?;

        // Seed nodes. For a filtered scan/seek leaf, enumerate candidate
        // ids directly and evaluate the leaf's predicates against ONE
        // reused row buffer -- the generic stream builds a fresh
        // `HashMap` row per candidate, which for an unindexed predicate
        // over a big label (matrix_review_counts: `title CONTAINS` over
        // 9k movies) was the query's remaining cost. Any leaf shape this
        // doesn't cover falls back to the generic stream.
        let mut seeds = Vec::new();
        let mut leaf_preds = Vec::new();
        let leaf_base = peel(leaf, &mut leaf_preds);
        let leaf_candidates: Option<Vec<NodeId>> = match leaf_base {
            LogicalPlan::AllNodesScan { var } if var == stages[0].from => Some(
                GraphStore::all_node_ids_limited_in_txn(txn, None, usize::MAX)?,
            ),
            LogicalPlan::NodeByLabelScan { var, label } if var == stages[0].from => Some(
                GraphStore::all_node_ids_limited_in_txn(txn, Some(label), usize::MAX)?,
            ),
            LogicalPlan::IndexSeek {
                var,
                label,
                prop,
                value: crate::ir::IndexSeekValue::Fixed(value),
            } if var == stages[0].from => {
                Some(GraphStore::lookup_by_index_in_txn(txn, label, prop, value)?)
            }
            _ => None,
        };
        match leaf_candidates {
            Some(candidates) => {
                // All-simple-predicate leaves (`var.prop <op> literal`,
                // matrix's `title CONTAINS ...`) evaluate through one
                // pre-opened NODES handle and the shared `compare` --
                // no per-candidate table open, no probe row, no
                // `eval_expr` dispatch. Anything else keeps the probe-row
                // route below.
                let simple: Option<Vec<(&PropAccess, CompareOp, &Literal)>> = leaf_preds
                    .iter()
                    .map(|pred| match pred {
                        Expr::Compare(pa, op, lit) if pa.var == stages[0].from => {
                            Some((pa, *op, lit))
                        }
                        _ => None,
                    })
                    .collect();
                if let Some(simple) = simple {
                    let pred_ids: Vec<Option<u32>> = simple
                        .iter()
                        .map(|(pa, _, _)| self.prop_id_for(txn, &pa.prop))
                        .collect::<Result<_, _>>()?;
                    let mut read_prop = GraphStore::node_prop_reader(txn)?;
                    'cand: for id in candidates {
                        guard.checkpoint()?;
                        for ((_, op, lit), prop_id) in simple.iter().zip(&pred_ids) {
                            let value = match prop_id {
                                Some(pid) => read_prop(id, *pid)?.flatten(),
                                None => None,
                            };
                            if compare(&value, *op, lit) != Some(true) {
                                continue 'cand;
                            }
                        }
                        seeds.push(id);
                    }
                } else {
                    let mut probe = BindingRow::new();
                    for id in candidates {
                        guard.checkpoint()?;
                        probe.insert(stages[0].from.to_string(), Binding::Node(id));
                        let mut pass = true;
                        for pred in &leaf_preds {
                            if self.eval_expr(txn, pred, &probe, guard)? != Some(true) {
                                pass = false;
                                break;
                            }
                        }
                        if pass {
                            seeds.push(id);
                        }
                    }
                }
            }
            None => {
                for row in self.eval_plan(txn, leaf, current_rows, guard)? {
                    match row.get(stages[0].from) {
                        Some(Binding::Node(id)) => seeds.push(*id),
                        _ => return Ok(None),
                    }
                }
            }
        }

        // -- the tight loop ----------------------------------------------
        struct Group {
            count: i64,
            collects: Vec<Vec<Value>>,
        }
        let n_collects = cols
            .iter()
            .filter(|c| matches!(c, OutCol::Collect(_)))
            .count();
        let mut order: Vec<u64> = Vec::new();
        let mut groups: HashMap<u64, Group> = HashMap::new();
        // Per-mid-node property memo: the same mid node recurs across
        // seeds/edges and its collected property is stable within the
        // snapshot.
        let mut mid_prop_memo: HashMap<(u64, u32), Option<Value>> = HashMap::new();
        let mut mid_values: Vec<Option<Value>> = vec![None; n_collects];
        let one_hop = stages.len() == 1;
        for &s in &seeds {
            guard.checkpoint()?;
            for e1 in GraphStore::neighbors_in_txn(txn, s, stages[0].dir, stages[0].label)? {
                guard.relationship_expansion()?;
                if !stage_sets[0].iter().all(|set| set.contains(&e1.other.0)) {
                    continue;
                }
                if one_hop {
                    let key = if group_by_origin { s.0 } else { e1.other.0 };
                    let group = groups.entry(key).or_insert_with(|| {
                        order.push(key);
                        Group {
                            count: 0,
                            collects: vec![Vec::new(); n_collects],
                        }
                    });
                    group.count += 1;
                    continue;
                }
                // Resolve this mid node's collected properties once.
                let mut ci = 0usize;
                for (col, prop_id) in cols.iter().zip(&collect_prop_ids) {
                    if let OutCol::Collect(_) = col {
                        mid_values[ci] = match prop_id {
                            Some(pid) => mid_prop_memo
                                .entry((e1.other.0, *pid))
                                .or_insert_with(|| {
                                    GraphStore::get_node_prop_in_txn(txn, e1.other, *pid)
                                        .ok()
                                        .flatten()
                                        .flatten()
                                        .map(property_value_to_value)
                                })
                                .clone(),
                            None => None, // never-interned property: absent everywhere
                        };
                        ci += 1;
                    }
                }
                guard.checkpoint()?;
                for e2 in
                    GraphStore::neighbors_in_txn(txn, e1.other, stages[1].dir, stages[1].label)?
                {
                    guard.relationship_expansion()?;
                    if isomorphism && e2.edge_id == e1.edge_id {
                        continue;
                    }
                    if !stage_sets[1].iter().all(|set| set.contains(&e2.other.0)) {
                        continue;
                    }
                    let key = if group_by_origin { s.0 } else { e2.other.0 };
                    let group = groups.entry(key).or_insert_with(|| {
                        order.push(key);
                        Group {
                            count: 0,
                            collects: vec![Vec::new(); n_collects],
                        }
                    });
                    group.count += 1;
                    for (ci, value) in mid_values.iter().enumerate() {
                        // collect() skips nulls, real Cypher's rule.
                        if let Some(v) = value {
                            group.collects[ci].push(v.clone());
                        }
                    }
                }
            }
        }

        // -- project, order, skip/limit ----------------------------------
        let mut grouped: Vec<(u64, Group)> = order
            .into_iter()
            .map(|id| {
                let group = groups.remove(&id).expect("group recorded in order");
                (id, group)
            })
            .collect();
        match count_sort {
            Some(SortDir::Asc) => grouped.sort_by_key(|(_, g)| g.count),
            Some(SortDir::Desc) => grouped.sort_by_key(|(_, g)| std::cmp::Reverse(g.count)),
            None => {}
        }
        if let Some(keep) = pre_keep {
            grouped.truncate(keep);
        }
        let skip_n = skip.unwrap_or(0).max(0) as usize;
        if skip_n > 0 {
            grouped.drain(0..skip_n.min(grouped.len()));
        }
        if let Some(limit) = limit {
            grouped.truncate(limit.max(0) as usize);
        }
        let rows: Vec<BindingRow> = grouped
            .into_iter()
            .map(|(id, group)| {
                let mut row = BindingRow::new();
                let mut collects = group.collects.into_iter();
                for (col, name) in cols.iter().zip(&names) {
                    let binding = match col {
                        OutCol::Group => Binding::Node(NodeId(id)),
                        OutCol::Count => Binding::Value(PropertyValue::Int(group.count)),
                        OutCol::Collect(_) => {
                            Binding::List(collects.next().expect("one list per collect column"))
                        }
                    };
                    row.insert(name.clone(), binding);
                }
                row
            })
            .collect();
        if std::env::var("MARSDB_FAST_DEBUG").is_ok() {
            eprintln!(
                "[fast-path FIRED] stages={} groups={}",
                stages.len(),
                rows.len()
            );
        }
        Ok(Some((rows, names.into_iter().collect())))
    }

    fn stream_scan<'s>(
        &'s self,
        txn: Txn<'s>,
        var: &'s str,
        label: Option<&'s str>,
        seed: &'s [BindingRow],
        guard: &'s ExecutionGuard<'_>,
        row_limit: Option<usize>,
    ) -> RowStream<'s> {
        let mut initialized = false;
        let mut node_ids = Vec::new();
        let mut seed_index = 0usize;
        let mut node_index = 0usize;
        let mut done = false;
        let stream = std::iter::from_fn(move || {
            if done || seed.is_empty() {
                return None;
            }
            if !initialized {
                initialized = true;
                let budget_node_limit = guard.options.max_intermediate_rows.map(|max_rows| {
                    max_rows
                        .checked_div(seed.len())
                        .unwrap_or(0)
                        .saturating_add(1)
                });
                let storage_limit = match (row_limit, budget_node_limit) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
                let storage_limit = storage_limit.unwrap_or(usize::MAX);
                match GraphStore::all_node_ids_limited_in_txn(txn, label, storage_limit) {
                    Ok(ids) => node_ids = ids,
                    Err(error) => {
                        done = true;
                        return Some(Err(error.into()));
                    }
                }
            }
            if node_ids.is_empty() || seed_index >= seed.len() {
                return None;
            }
            if let Err(error) = guard.checkpoint() {
                done = true;
                return Some(Err(error));
            }
            let mut row = seed[seed_index].clone();
            row.insert(var.to_string(), Binding::Node(node_ids[node_index]));
            node_index += 1;
            if node_index == node_ids.len() {
                node_index = 0;
                seed_index += 1;
            }
            Some(Ok(row))
        });
        Self::count_stream(Box::new(stream), guard)
    }

    /// `LogicalPlan::IndexSeek`'s streaming operator -- same cross-join-
    /// against-`seed` shape as `stream_scan`, but the id list comes from
    /// one exact-match `PROPERTY_INDEX` lookup instead of a label scan.
    /// `row_limit` bounds the lookup itself the same way `stream_scan`'s
    /// does -- a non-unique index can still match far more nodes than a
    /// `LIMIT` needs, so the same "ask storage for at most the budget,
    /// not everything" reasoning applies, just against `PROPERTY_INDEX`
    /// instead of `NODE_LABEL_INDEX`.
    ///
    /// `spec.value` is either fixed for the whole seek (a literal, or a
    /// `$param` already resolved to one -- looked up once, reused across
    /// every seed row, same as before this `enum` existed) or row-
    /// dependent (`IndexSeekValue::RowExpr`, e.g. `row.field` from an
    /// enclosing `UNWIND`) -- re-evaluated and re-looked-up for each seed
    /// row, since a different row can mean a different lookup value. This
    /// is the fix for what was previously *always* a `NodeByLabelScan` +
    /// `Filter` for that shape (`planner::apply_index_seeks` only
    /// recognized a literal-valued equality, never a per-row one) -- an
    /// O(label size) scan repeated per incoming row, the exact pattern a
    /// bulk import's relationship-creation pass hits hardest.
    fn stream_index_seek<'s>(
        &'s self,
        txn: Txn<'s>,
        spec: IndexSeekSpec<'s>,
        seed: &'s [BindingRow],
        guard: &'s ExecutionGuard<'_>,
        row_limit: Option<usize>,
    ) -> RowStream<'s> {
        let budget_node_limit = guard.options.max_intermediate_rows.map(|max_rows| {
            max_rows
                .checked_div(seed.len().max(1))
                .unwrap_or(0)
                .saturating_add(1)
        });
        let storage_limit = match (row_limit, budget_node_limit) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let lookup = move |value: &PropertyValue| -> Result<Vec<NodeId>, QueryError> {
            match storage_limit {
                Some(limit) => GraphStore::lookup_by_index_limited_in_txn(
                    txn, spec.label, spec.prop, value, limit,
                )
                .map_err(Into::into),
                None => GraphStore::lookup_by_index_in_txn(txn, spec.label, spec.prop, value)
                    .map_err(Into::into),
            }
        };
        match spec.value {
            // One lookup, reused across every seed row -- identical shape
            // to `stream_scan`'s own cross join, and to this function
            // before `IndexSeekValue` existed.
            IndexSeekValue::Fixed(value) => {
                let mut node_ids: Option<Vec<NodeId>> = None;
                let mut seed_index = 0usize;
                let mut node_index = 0usize;
                let mut done = false;
                let stream = std::iter::from_fn(move || {
                    if done || seed.is_empty() {
                        return None;
                    }
                    let ids = match &node_ids {
                        Some(ids) => ids,
                        None => match lookup(value) {
                            Ok(ids) => node_ids.insert(ids),
                            Err(error) => {
                                done = true;
                                return Some(Err(error));
                            }
                        },
                    };
                    if ids.is_empty() || seed_index >= seed.len() {
                        return None;
                    }
                    if let Err(error) = guard.checkpoint() {
                        done = true;
                        return Some(Err(error));
                    }
                    let mut row = seed[seed_index].clone();
                    row.insert(spec.var.to_string(), Binding::Node(ids[node_index]));
                    node_index += 1;
                    if node_index == ids.len() {
                        node_index = 0;
                        seed_index += 1;
                    }
                    Some(Ok(row))
                });
                Self::count_stream(Box::new(stream), guard)
            }
            // A fresh lookup per seed row -- `expr` (e.g. `row.field` from
            // an enclosing `UNWIND`) can evaluate to a different value for
            // each one, so last row's `node_ids` can't be reused for the
            // next.
            IndexSeekValue::RowExpr(expr) => {
                let mut node_ids: Vec<NodeId> = Vec::new();
                let mut seed_index = 0usize;
                let mut node_index = 0usize;
                let mut done = false;
                let stream = std::iter::from_fn(move || loop {
                    if done || seed_index >= seed.len() {
                        return None;
                    }
                    if node_index == 0 {
                        let evaluated =
                            match self.eval_return_expr(txn, expr, &seed[seed_index], guard) {
                                Ok(v) => v,
                                Err(error) => {
                                    done = true;
                                    return Some(Err(error));
                                }
                            };
                        let value = value_to_property_value(&evaluated);
                        // Real Cypher's three-valued logic: comparing
                        // against `null` is "unknown", not "find nodes
                        // whose stored value happens to be Null" -- this
                        // row contributes zero rows, same as the Filter
                        // fallback this replaces would reject it outright.
                        if matches!(value, PropertyValue::Null) {
                            seed_index += 1;
                            continue;
                        }
                        node_ids = match lookup(&value) {
                            Ok(ids) => ids,
                            Err(error) => {
                                done = true;
                                return Some(Err(error));
                            }
                        };
                        if node_ids.is_empty() {
                            seed_index += 1;
                            continue;
                        }
                    }
                    if let Err(error) = guard.checkpoint() {
                        done = true;
                        return Some(Err(error));
                    }
                    let mut row = seed[seed_index].clone();
                    row.insert(spec.var.to_string(), Binding::Node(node_ids[node_index]));
                    node_index += 1;
                    if node_index == node_ids.len() {
                        node_index = 0;
                        seed_index += 1;
                    }
                    return Some(Ok(row));
                });
                Self::count_stream(Box::new(stream), guard)
            }
        }
    }

    fn expand_variable_row(
        &self,
        txn: Txn,
        row: BindingRow,
        spec: VarExpandSpec<'_>,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Vec<BindingRow>, QueryError> {
        let start_id = match row.get(spec.from_var) {
            Some(Binding::Node(id)) => *id,
            Some(Binding::Value(PropertyValue::Null)) => return Ok(Vec::new()),
            _ => return Err(QueryError::UnboundVariable(spec.from_var.to_string())),
        };
        let mut out = Vec::new();
        if spec.min_hops == 0 {
            let mut new_row = row.clone();
            new_row.insert(spec.to_var.to_string(), Binding::Node(start_id));
            if let Some(path_segment_var) = spec.path_segment_var {
                new_row.insert(path_segment_var.to_string(), Binding::Path(Vec::new()));
            }
            if let Some(rel_list_var) = spec.rel_list_var {
                new_row.insert(rel_list_var.to_string(), Binding::List(Vec::new()));
            }
            new_row.insert(spec.exclude_edge_var.to_string(), Binding::Path(Vec::new()));
            out.push(new_row);
        }
        // `[:TYPE* {year: 1988}]` -- evaluated once here (constant across
        // the whole BFS, not per-candidate; the values can reference this
        // row's own already-bound variables, same as a fixed hop's inline
        // props already can) and checked against each candidate edge's
        // own stored properties during expansion below (TCK's Match4
        // `[5]`).
        let rel_props = spec
            .rel_props
            .iter()
            .map(|(key, expr)| {
                let value = self.eval_return_expr(txn, expr, &row, guard)?;
                Ok::<_, QueryError>((key.as_str(), value_to_property_value(&value)))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let unbounded = spec.max_hops.is_none();
        let effective_max = spec.max_hops.unwrap_or(VAR_EXPAND_DEPTH_CAP);
        // Real Cypher's edge-isomorphism rule (no relationship repeated
        // within one MATCH pattern) applies across the *whole* pattern, not
        // just within this hop's own BFS -- seed the excluded set with
        // whatever edges earlier fixed hops of this same pattern already
        // bound, so this traversal can't walk back over one of them (see
        // `LogicalPlan::VarExpand`'s docs; found via TCK's Match5 `[27]`).
        // Complementary direction: an *earlier variable-length* hop's own
        // traversed-edge set (deposited under its own `exclude_edge_var`,
        // see `LogicalPlan::VarExpand`'s docs) -- union every such row's
        // `Binding::Path` edge ids in too (TCK's Match4 `[7]`).
        let seed_used_edges: HashSet<EdgeId> = spec
            .exclude_edge_vars
            .iter()
            .filter_map(|v| match row.get(v) {
                Some(Binding::Edge(id)) => Some(*id),
                _ => None,
            })
            .chain(spec.exclude_edge_sets.iter().flat_map(|v| {
                match row.get(v) {
                    Some(Binding::Path(segment)) => segment
                        .iter()
                        .filter_map(|p| match p {
                            PathBinding::Edge(id) => Some(*id),
                            PathBinding::Node(_) => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                }
            }))
            .collect();
        // The ordered `Edge, Node, Edge, Node, ...` sequence built up so
        // far, alongside the existing `used_edges` isomorphism set --
        // only actually consulted when `path_segment_var` is set (named-
        // path capture over this hop, see `LogicalPlan::VarExpand`'s own
        // docs), but always threaded through the BFS regardless (a plain
        // `Vec`, cheap to carry and clone even when unused).
        let mut frontier = vec![(start_id, seed_used_edges, Vec::<PathBinding>::new())];
        let mut depth = 0u32;
        while depth < effective_max && !frontier.is_empty() {
            depth += 1;
            let mut next_frontier = Vec::new();
            for (node, used_edges, segment) in frontier {
                for entry in neighbors_for_direction(txn, node, spec.direction, spec.rel_labels)? {
                    guard.relationship_expansion()?;
                    if used_edges.contains(&entry.edge_id) {
                        continue;
                    }
                    if !rel_props.is_empty() {
                        let edge = deleted_entity_access(GraphStore::get_edge_in_txn(
                            txn,
                            entry.edge_id,
                        )?)?;
                        let matches = rel_props
                            .iter()
                            .all(|(key, expected)| edge.props.get(*key) == Some(expected));
                        if !matches {
                            continue;
                        }
                    }
                    let mut next_used_edges = used_edges.clone();
                    next_used_edges.insert(entry.edge_id);
                    let mut next_segment = segment.clone();
                    next_segment.push(PathBinding::Edge(entry.edge_id));
                    next_segment.push(PathBinding::Node(entry.other));
                    next_frontier.push((entry.other, next_used_edges, next_segment.clone()));
                    guard.check_intermediate_rows(next_frontier.len())?;
                    if depth >= spec.min_hops {
                        let mut new_row = row.clone();
                        new_row.insert(spec.to_var.to_string(), Binding::Node(entry.other));
                        if let Some(path_segment_var) = spec.path_segment_var {
                            new_row.insert(
                                path_segment_var.to_string(),
                                Binding::Path(next_segment.clone()),
                            );
                        }
                        if let Some(rel_list_var) = spec.rel_list_var {
                            let edges = segment_edges_to_list(txn, &next_segment)?;
                            new_row.insert(rel_list_var.to_string(), edges);
                        }
                        new_row.insert(
                            spec.exclude_edge_var.to_string(),
                            Binding::Path(next_segment.clone()),
                        );
                        out.push(new_row);
                        guard.check_intermediate_rows(out.len())?;
                    }
                }
            }
            frontier = next_frontier;
            if depth == effective_max && unbounded && !frontier.is_empty() {
                return Err(QueryError::ResourceLimit(format!(
                    "variable-length traversal exceeded the safety depth cap ({VAR_EXPAND_DEPTH_CAP} \
                     hops) — likely a cyclic graph or unexpectedly large fanout; narrow the pattern or \
                     add an explicit upper bound (e.g. *0..10)"
                )));
            }
        }
        Ok(out)
    }

    /// `LogicalPlan::MatchRelList`'s own docs -- deterministic, no search:
    /// `spec.rel_list_var`'s edges are already concrete, so there's
    /// exactly one possible walk to check, starting from `spec.from_var`'s
    /// already-bound node. Returns `Ok(None)` (row dropped, not an error)
    /// for every "doesn't match" case -- wrong hop count, a broken chain,
    /// an edge whose label isn't in `spec.rel_labels` -- same "no match
    /// survives" convention `Expand`/`VarExpand` already use for a filter
    /// that simply excludes a row.
    fn match_bound_rel_list_row(
        &self,
        row: BindingRow,
        spec: MatchRelListSpec<'_>,
    ) -> Result<Option<BindingRow>, QueryError> {
        let start_id = match row.get(spec.from_var) {
            Some(Binding::Node(id)) => *id,
            Some(Binding::Value(PropertyValue::Null)) => return Ok(None),
            _ => return Err(QueryError::UnboundVariable(spec.from_var.to_string())),
        };
        let edges: Vec<&Edge> = match row.get(spec.rel_list_var) {
            Some(Binding::List(items)) => items
                .iter()
                .map(|v| match v {
                    Value::Edge(e) => Ok(e),
                    other => Err(QueryError::Type(format!(
                        "'{}' must be a list of relationships, found {other:?} in it",
                        spec.rel_list_var
                    ))),
                })
                .collect::<Result<_, _>>()?,
            Some(Binding::Value(PropertyValue::Null)) => return Ok(None),
            _ => return Err(QueryError::UnboundVariable(spec.rel_list_var.to_string())),
        };
        let hops = edges.len() as u32;
        if hops < spec.min_hops || spec.max_hops.is_some_and(|max| hops > max) {
            return Ok(None);
        }
        if !spec.rel_labels.is_empty() && edges.iter().any(|e| !spec.rel_labels.contains(&e.label))
        {
            return Ok(None);
        }
        let mut current = start_id;
        for edge in &edges {
            let next = match spec.direction {
                ExpandDirection::Out if edge.src == current => edge.dst,
                ExpandDirection::In if edge.dst == current => edge.src,
                ExpandDirection::Either if edge.src == current => edge.dst,
                ExpandDirection::Either if edge.dst == current => edge.src,
                _ => return Ok(None),
            };
            current = next;
        }
        let mut new_row = row.clone();
        new_row.insert(spec.to_var.to_string(), Binding::Node(current));
        Ok(Some(new_row))
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Option<bool>, QueryError> {
        Ok(match expr {
            Expr::And(l, r) => and3(
                self.eval_expr(txn, l, row, guard)?,
                self.eval_expr(txn, r, row, guard)?,
            ),
            Expr::Or(l, r) => or3(
                self.eval_expr(txn, l, row, guard)?,
                self.eval_expr(txn, r, row, guard)?,
            ),
            Expr::Not(e) => self.eval_expr(txn, e, row, guard)?.map(|b| !b),
            Expr::Compare(pa, op, lit) => {
                let prop_value = self.lookup_prop(txn, pa, row)?;
                compare(&prop_value, *op, lit)
            }
            Expr::PropCompare(left, op, right) => {
                let a = self.lookup_prop(txn, left, row)?;
                let b = self.lookup_prop(txn, right, row)?;
                compare_property_pair_opt(&a, *op, &b)
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
                let node = self.get_node_cached(txn, *id)?;
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
            Expr::GeneralCompare(lhs, op, rhs) => {
                let lv = self.eval_return_expr(txn, lhs, row, guard)?;
                let rv = self.eval_return_expr(txn, rhs, row, guard)?;
                compare_values(&lv, *op, &rv)
            }
            Expr::GeneralIsNull(e) => Some(matches!(
                self.eval_return_expr(txn, e, row, guard)?,
                Value::Null
            )),
            Expr::GeneralBare(e) => self.eval_return_expr_bool3(txn, e, row, guard)?,
            // `WHERE (n)-[:REL]->()` etc (TCK's Pattern1) -- existential:
            // true iff at least one real match of `pattern` exists, with
            // every already-bound named endpoint (`n`, and `m` in `(n)-->
            // (m)` when `m` is also bound by an earlier MATCH) held fixed
            // to this row's own binding rather than searched freely.
            // `semantic::bind_pattern_predicate` already rejected any
            // named endpoint that ISN'T already bound (real Cypher's
            // UndefinedVariable), so every named var here is safe to seed.
            // Reuses the exact same `build_match_plan` "already-bound var
            // -> Seed, not a fresh scan" mechanism `eval_merge`'s own
            // "try as an ordinary MATCH first" half already relies on --
            // for a one-hop pattern this is a real connected-subgraph
            // search (Expand + Filter), not an isolated per-node check.
            // `Some(1)`-limited: existence is all that's needed, so
            // there's no reason to enumerate every match.
            Expr::Pattern(pattern) => {
                Some(self.eval_pattern_predicate_exists(txn, pattern, row, guard)?)
            }
            // `exists { (n)-->(m) WHERE ... }` (TCK's ExistentialSubquery1,
            // the "simple" form) -- same existential search as `Pattern`
            // above, just with its own inline `where?` threaded straight
            // into `build_match_plan`, same as an ordinary `MATCH ...
            // WHERE ...` (not evaluated as a separate post-filter step).
            Expr::Exists {
                pattern,
                where_clause,
            } => {
                let carried_vars: HashSet<String> = row.keys().cloned().collect();
                let wc: Option<Expr> = where_clause.as_deref().cloned();
                let plan = apply_index_seeks(build_match_plan(pattern, &wc, &carried_vars)?, txn)?;
                let found = self.eval_plan_with_limit(
                    txn,
                    &plan,
                    std::slice::from_ref(row),
                    guard,
                    Some(1),
                )?;
                Some(!found.is_empty())
            }
            // `exists { MATCH ... RETURN ... }` (TCK's
            // ExistentialSubquery2/3, the "full" form) -- runs the nested
            // statement correlated against `row` (`execute_match_seeded`)
            // and checks whether it produced at least one output row.
            Expr::ExistsSubquery(stmt) => Some(self.eval_exists_subquery(txn, stmt, row, guard)?),
            // See `Expr::EdgeNotInSet`'s own docs -- `edge_var` is always
            // a real `Binding::Edge` (a fixed hop's own filter var, the
            // only thing this gets generated for) and `edge_set_var` is
            // always the `Binding::Path` `expand_variable_row` deposits
            // for *every* variable-length hop, unconditionally (see
            // `LogicalPlan::VarExpand::exclude_edge_var`'s own docs) --
            // never anything else, so there's no null/wrong-kind case to
            // handle here the way `VarEq` above has to.
            Expr::EdgeNotInSet {
                edge_var,
                edge_set_var,
            } => {
                let Some(Binding::Edge(edge_id)) = row.get(edge_var) else {
                    return Err(QueryError::UnboundVariable(edge_var.clone()));
                };
                let Some(Binding::Path(segment)) = row.get(edge_set_var) else {
                    return Err(QueryError::UnboundVariable(edge_set_var.clone()));
                };
                Some(
                    !segment
                        .iter()
                        .any(|elem| matches!(elem, PathBinding::Edge(id) if id == edge_id)),
                )
            }
        })
    }

    /// Prop name -> interned id, memoized per statement for read-only
    /// statements only -- see `prop_id_memo`'s docs for why write
    /// statements bypass the memo (mid-statement interning would make a
    /// cached `None` stale within the same statement).
    fn prop_id_for(&self, txn: Txn, name: &str) -> Result<Option<u32>, QueryError> {
        if let Some(cached) = self.prop_id_memo.borrow().get(name) {
            return Ok(*cached);
        }
        let id = GraphStore::lookup_prop_id_in_txn(txn, name)?;
        // A name -> Some(id) interning is immutable once made, so a hit
        // is safe to memoize in any statement. A `None` ("never
        // interned") can go stale *within a write statement* -- a later
        // `CREATE`/`SET` can intern that very name -- so `None` is only
        // memoized where nothing can intern: a read-only statement.
        if id.is_some() || self.read_only_stmt.get() {
            self.prop_id_memo.borrow_mut().insert(name.to_string(), id);
        }
        Ok(id)
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
            //
            // Per-property read path (v2 step 1b): a node already
            // materialized in this statement's cache answers from the map;
            // otherwise this reads ONE directory entry from the stored
            // record -- no full decode, no name resolution, no cache
            // population (repeat per-prop reads are ~a point lookup each,
            // cheaper than materializing a whole record to answer one of
            // them). The nested Option from `get_node_prop_in_txn`
            // preserves the deleted-vs-absent split above.
            Binding::Node(id) => {
                // Safe for write statements too: every node-mutating
                // site evicts (`uncache_node`), so a surviving cache
                // entry is current by construction.
                if let Some(cached) = self.node_cache.borrow().get(id) {
                    return Ok(cached.props.get(&pa.prop).cloned());
                }
                match self.prop_id_for(txn, &pa.prop)? {
                    Some(prop_id) => Ok(deleted_entity_access(GraphStore::get_node_prop_in_txn(
                        txn, *id, prop_id,
                    )?)?),
                    // Name never interned anywhere: absent on every record
                    // by construction -- but a deleted node must still
                    // error, so existence is checked without any decode.
                    None => {
                        deleted_entity_access(
                            GraphStore::node_exists_in_txn(txn, *id)?.then_some(()),
                        )?;
                        Ok(None)
                    }
                }
            }
            Binding::Edge(id) => match self.prop_id_for(txn, &pa.prop)? {
                Some(prop_id) => Ok(deleted_entity_access(GraphStore::get_edge_prop_in_txn(
                    txn, *id, prop_id,
                )?)?),
                None => {
                    deleted_entity_access(GraphStore::edge_exists_in_txn(txn, *id)?.then_some(()))?;
                    Ok(None)
                }
            },
            // A WITH-projected scalar (or list/map) has no scalar `.prop`
            // to access via this path — e.g. `WITH message.id AS
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
            Binding::Value(_) | Binding::List(_) | Binding::Map(_) => Ok(None),
            // Unlike the others, a path is a real type error, not just an
            // "absent" property -- real Cypher's `InvalidArgumentType`
            // (TCK's MatchWhere1 `[14]`: `MATCH r = (n)-[*]->() WHERE
            // r.name = 'apa'`). Property access never had a meaning for a
            // path to begin with (it's not a graph-object-shaped value).
            Binding::Path(_) => Err(QueryError::Type(format!(
                "'{}' is a path — property access requires a node, relationship, or map",
                pa.var
            ))),
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
    ///
    /// Only a node, relationship, map, or temporal value has any `.prop`
    /// to access at all -- a plain scalar (`Bool`/`Int`/`Float`/`String`)
    /// or a `List` is a real type error here (real Cypher's own
    /// `InvalidArgumentType` is raised at *compile* time; this codebase's
    /// `Kind` system can't see through a WITH-projected value's real
    /// runtime shape to catch it any earlier -- see `infer_expr`'s own
    /// `Kind::Scalar` docs -- so it surfaces here instead), not a silent
    /// `null` (TCK's Graph6 [9] / Map1 [6]). `null` itself is exempt --
    /// real Cypher's null propagation rule, not a type error.
    fn lookup_prop_value(
        &self,
        txn: Txn,
        pa: &PropAccess,
        row: &BindingRow,
    ) -> Result<Value, QueryError> {
        match row.get(&pa.var) {
            Some(Binding::Map(m)) => Ok(m.get(&pa.prop).cloned().unwrap_or(Value::Null)),
            Some(Binding::Value(PropertyValue::Null)) => Ok(Value::Null),
            Some(Binding::Value(pv)) => match temporal_component(pv, &pa.prop) {
                Some(component) => Ok(Value::Property(component)),
                None if is_temporal_property_value(pv) => Ok(Value::Null),
                None => Err(QueryError::Type(format!(
                    "'{}' can't have properties accessed on it -- property access requires a \
                     node, relationship, map, or temporal value",
                    pa.var
                ))),
            },
            Some(Binding::List(_)) => Err(QueryError::Type(format!(
                "'{}' can't have properties accessed on it -- property access requires a node, \
                 relationship, map, or temporal value, not a list",
                pa.var
            ))),
            Some(_) => Ok(match self.lookup_prop(txn, pa, row)? {
                Some(PropertyValue::Null) | None => Value::Null,
                Some(pv) => property_value_to_value(pv),
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
        guard: &ExecutionGuard<'_>,
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
                    out_row.push(self.eval_return_expr(txn, &item.expr, row, guard)?);
                }
                out_rows.push(out_row);
            }
            out_rows
        } else {
            validate_return_items(items)?;
            let grouped = self.resolve_grouped_rows(txn, items, rows, guard)?;
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

    /// An aggregating `RETURN`'s own `ORDER BY`, when at least one key
    /// doesn't verbatim/alias-match any item -- `RETURN me.age AS age,
    /// count(you.age) AS cnt ORDER BY age + count(you.age)` (TCK's
    /// ReturnOrderBy6). Folds those extra keys through the *same*
    /// grouping pass as `items` themselves, as synthetic unreturned extra
    /// items (reusing `resolve_grouped_rows`/`rewrite_composed_item`
    /// exactly as a composed RETURN item would, including an aggregate
    /// call that appears *only* in the ORDER BY key, nowhere in `items`
    /// -- real Cypher allows that too, it just needs to fold consistently
    /// with `items`' own implicit grouping, not literally reuse one of
    /// their accumulators), then uses their per-group values as
    /// additional sort keys before stripping them back off. Degrades to
    /// exactly the ordinary "sort by already-computed columns" behavior
    /// when every key does verbatim/alias-match (`extra_exprs` empty) --
    /// callers can route every aggregating-`RETURN`-with-`ORDER-BY` case
    /// through this one function rather than branching on whether extras
    /// are actually needed.
    ///
    /// `DISTINCT` isn't handled here -- deliberately: grouping already
    /// makes every output row unique by its own grouping-key columns (two
    /// groups can't have the same grouping key and still be different
    /// groups), so `RETURN DISTINCT` combined with aggregation is
    /// provably always a no-op downstream of this function regardless.
    fn materialize_aggregating_return_with_order(
        &self,
        txn: Txn,
        items: &[ReturnItem],
        rows: &[BindingRow],
        order_by: &[(ReturnExpr, SortDir)],
        skip_limit: (Option<i64>, Option<i64>),
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let (skip, limit) = skip_limit;
        enum OrderKeySource {
            RealColumn(usize),
            Extra(usize),
        }
        let mut extra_exprs: Vec<ReturnExpr> = Vec::new();
        let order_by_source: Vec<OrderKeySource> = order_by
            .iter()
            .map(|(expr, _)| {
                match items
                    .iter()
                    .enumerate()
                    .position(|(i, it)| item_matches_leaf(expr, i, it))
                {
                    Some(i) => OrderKeySource::RealColumn(i),
                    None => {
                        let idx = extra_exprs.len();
                        extra_exprs.push(expr.clone());
                        OrderKeySource::Extra(idx)
                    }
                }
            })
            .collect();
        let extended_items: Vec<ReturnItem> = items
            .iter()
            .cloned()
            .chain(
                extra_exprs
                    .into_iter()
                    .map(|expr| ReturnItem { expr, alias: None }),
            )
            .collect();
        validate_return_items(&extended_items)?;
        let grouped = self.resolve_grouped_rows(txn, &extended_items, rows, guard)?;
        let columns: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                item.alias
                    .clone()
                    .unwrap_or_else(|| default_column_name(&item.expr, i))
            })
            .collect();
        let real_len = items.len();
        let mut keyed: Vec<(Vec<Value>, Vec<Value>)> = Vec::with_capacity(grouped.len());
        for bindings in grouped {
            let values: Vec<Value> = bindings
                .iter()
                .map(|b| self.binding_to_value(txn, b))
                .collect::<Result<Vec<_>, _>>()?;
            let (real, extra) = values.split_at(real_len);
            let keys: Vec<Value> = order_by_source
                .iter()
                .map(|src| match src {
                    OrderKeySource::RealColumn(i) => real[*i].clone(),
                    OrderKeySource::Extra(k) => extra[*k].clone(),
                })
                .collect();
            keyed.push((keys, real.to_vec()));
        }
        let rows = top_k_by(keyed, order_by, skip, limit)
            .into_iter()
            .map(|(_, row)| row)
            .collect();
        Ok(QueryResult { columns, rows })
    }

    /// `SKIP`/`LIMIT` accept any expression, not just a literal integer
    /// (`SKIP $n`, `SKIP toInteger(rand()*9)` -- TCK's `ReturnSkipLimit1
    /// [2]`/`[3]`) -- evaluated exactly once here, against an empty row,
    /// since no pattern variable can be in scope at a statement's own
    /// SKIP/LIMIT (an `UnboundVariable` error from `eval_return_expr`
    /// below is exactly the right outcome if one is referenced). Params
    /// are already resolved to concrete `Literal`s by this point (see
    /// `params::substitute_params`).
    fn resolve_skip_limit(
        &self,
        txn: Txn,
        expr: Option<&ReturnExpr>,
        clause: &str,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Option<i64>, QueryError> {
        let Some(expr) = expr else {
            return Ok(None);
        };
        let value = self.eval_return_expr(txn, expr, &BindingRow::new(), guard)?;
        let n = match value {
            Value::Literal(Literal::Int(n)) | Value::Property(PropertyValue::Int(n)) => n,
            _ => {
                return Err(QueryError::Semantic(format!(
                    "{clause} must evaluate to an integer"
                )));
            }
        };
        if n < 0 {
            return Err(QueryError::Semantic(format!("{clause} can't be negative")));
        }
        Ok(Some(n))
    }

    fn eval_return_expr(
        &self,
        txn: Txn,
        expr: &ReturnExpr,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Value, QueryError> {
        match expr {
            ReturnExpr::Var(var) => {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                self.binding_to_value(txn, binding)
            }
            ReturnExpr::Prop(pa) => self.lookup_prop_value(txn, pa, row),
            ReturnExpr::PropOf(base, prop) => {
                let v = self.eval_return_expr(txn, base, row, guard)?;
                property_of_value(&v, prop)
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
                    return Err(QueryError::Semantic(format!(
                        "aggregate function '{name}' can only be used as a return item's top-level expression"
                    )));
                }
                let lower = name.to_ascii_lowercase();
                if lower == "type" {
                    // Special-cased *before* the generic arg-evaluation
                    // below -- that would eagerly fail on a deleted
                    // relationship (`deleted_entity_access`), before
                    // `eval_type_call` ever gets a chance to fall back to
                    // its cached type. See `ExecutionGuard::
                    // deleted_edge_types`'s own docs.
                    return self.eval_type_call(txn, args.first(), row, guard);
                }
                let arg_values = args
                    .iter()
                    .map(|a| self.eval_return_expr(txn, a, row, guard))
                    .collect::<Result<Vec<_>, _>>()?;
                if lower == "startnode" || lower == "endnode" {
                    return self.start_or_end_node(txn, &lower, arg_values.first());
                }
                call_builtin(name, &arg_values, self.now_snapshot())
            }
            ReturnExpr::CountStar => Err(QueryError::Semantic(
                "count(*) can only be used as a return item's top-level expression".into(),
            )),
            ReturnExpr::Case { test, whens, else_ } => {
                let test_value = match test {
                    Some(t) => Some(self.eval_return_expr(txn, t, row, guard)?),
                    None => None,
                };
                for (when, then) in whens {
                    let when_value = self.eval_return_expr(txn, when, row, guard)?;
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
                        return self.eval_return_expr(txn, then, row, guard);
                    }
                }
                match else_ {
                    Some(e) => self.eval_return_expr(txn, e, row, guard),
                    None => Ok(Value::Null),
                }
            }
            ReturnExpr::Arith(l, op, r) => {
                let lv = self.eval_return_expr(txn, l, row, guard)?;
                let rv = self.eval_return_expr(txn, r, row, guard)?;
                apply_arith(*op, &lv, &rv)
            }
            ReturnExpr::Neg(e) => {
                let v = self.eval_return_expr(txn, e, row, guard)?;
                apply_neg(&v)
            }
            ReturnExpr::ListLit(items) => Ok(Value::List(
                items
                    .iter()
                    .map(|item| self.eval_return_expr(txn, item, row, guard))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            ReturnExpr::Index(base, index) => {
                let base_v = self.eval_return_expr(txn, base, row, guard)?;
                let index_v = self.eval_return_expr(txn, index, row, guard)?;
                apply_index(&base_v, &index_v)
            }
            ReturnExpr::Slice(base, start, end) => {
                let base_v = self.eval_return_expr(txn, base, row, guard)?;
                let start_v = start
                    .as_deref()
                    .map(|s| self.eval_return_expr(txn, s, row, guard))
                    .transpose()?;
                let end_v = end
                    .as_deref()
                    .map(|e| self.eval_return_expr(txn, e, row, guard))
                    .transpose()?;
                apply_slice(&base_v, start_v.as_ref(), end_v.as_ref())
            }
            ReturnExpr::ListComp {
                var,
                source,
                where_clause,
                project,
            } => {
                let source_v = self.eval_return_expr(txn, source, row, guard)?;
                let items = match source_v {
                    Value::List(items) => items,
                    Value::Null => return Ok(Value::Null),
                    other => {
                        return Err(QueryError::Type(format!(
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
                        Some(w) => {
                            self.eval_return_expr_bool3(txn, w, &scoped_row, guard)? == Some(true)
                        }
                        None => true,
                    };
                    if !keep {
                        continue;
                    }
                    result.push(match project {
                        Some(p) => self.eval_return_expr(txn, p, &scoped_row, guard)?,
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
                let source_v = self.eval_return_expr(txn, source, row, guard)?;
                let items = match source_v {
                    Value::List(items) => items,
                    Value::Null => return Ok(Value::Null),
                    other => {
                        return Err(QueryError::Type(format!(
                            "quantifier source must be a list, got {other:?}"
                        )))
                    }
                };
                let mut preds = Vec::with_capacity(items.len());
                for item in &items {
                    let mut scoped_row = row.clone();
                    scoped_row.insert(var.clone(), value_to_binding_restore(item));
                    preds.push(match where_clause {
                        Some(w) => self.eval_return_expr_bool3(txn, w, &scoped_row, guard)?,
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
                    map.insert(k.clone(), self.eval_return_expr(txn, v, row, guard)?);
                }
                Ok(Value::Map(map))
            }
            ReturnExpr::And(l, r) => Ok(bool3_to_value(and3(
                self.eval_return_expr_bool3(txn, l, row, guard)?,
                self.eval_return_expr_bool3(txn, r, row, guard)?,
            ))),
            ReturnExpr::Or(l, r) => Ok(bool3_to_value(or3(
                self.eval_return_expr_bool3(txn, l, row, guard)?,
                self.eval_return_expr_bool3(txn, r, row, guard)?,
            ))),
            ReturnExpr::Xor(l, r) => Ok(bool3_to_value(xor3(
                self.eval_return_expr_bool3(txn, l, row, guard)?,
                self.eval_return_expr_bool3(txn, r, row, guard)?,
            ))),
            ReturnExpr::Not(e) => Ok(bool3_to_value(
                self.eval_return_expr_bool3(txn, e, row, guard)?.map(|b| !b),
            )),
            ReturnExpr::Compare(l, op, r) => {
                let lv = self.eval_return_expr(txn, l, row, guard)?;
                let rv = self.eval_return_expr(txn, r, row, guard)?;
                Ok(bool3_to_value(compare_values(&lv, *op, &rv)))
            }
            ReturnExpr::IsNull(e) => {
                let v = self.eval_return_expr(txn, e, row, guard)?;
                Ok(Value::Literal(Literal::Bool(matches!(v, Value::Null))))
            }
            ReturnExpr::In(needle, haystack) => {
                let nv = self.eval_return_expr(txn, needle, row, guard)?;
                let hv = self.eval_return_expr(txn, haystack, row, guard)?;
                Ok(bool3_to_value(list_membership_ternary(&nv, &hv)?))
            }
            ReturnExpr::HasLabel(var, labels) => {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                match binding {
                    Binding::Node(id) => {
                        let node = deleted_entity_access(self.get_node_cached(txn, *id)?)?;
                        Ok(Value::Literal(Literal::Bool(
                            labels.iter().all(|l| node.labels.contains(l)),
                        )))
                    }
                    // `r:TYPE` -- a relationship has exactly one type, so
                    // this is just an equality check, not a set-membership
                    // one; a conjunctive `r:A:B` (only reachable from
                    // general expression position, never real Cypher's own
                    // pattern-level `WHERE` -- relationships can't carry
                    // more than one type) is trivially always false unless
                    // every listed name is the same one type (TCK's Graph5
                    // "Node and edge label expressions" [2]).
                    Binding::Edge(id) => {
                        let edge = deleted_entity_access(GraphStore::get_edge_in_txn(txn, *id)?)?;
                        Ok(Value::Literal(Literal::Bool(
                            labels.iter().all(|l| edge.label == *l),
                        )))
                    }
                    Binding::Value(PropertyValue::Null) => Ok(Value::Null),
                    other => Err(QueryError::Type(format!(
                        "'{var}' isn't a node or relationship — (n:Label) needs one, got {other:?}"
                    ))),
                }
            }
            ReturnExpr::PatternPredicate(_) => Err(QueryError::Semantic(
                "a pattern predicate (`(n)-->()` etc) can only be used inside WHERE".into(),
            )),
            ReturnExpr::PatternComprehension {
                path_var,
                pattern,
                where_clause,
                projection,
            } => self.eval_pattern_comprehension(
                txn,
                PatternComprehensionSpec {
                    path_var,
                    pattern,
                    where_clause,
                    projection,
                },
                row,
                guard,
            ),
            ReturnExpr::ExistsPattern { .. } | ReturnExpr::ExistsSubquery(_) => Err(
                QueryError::Semantic("an exists {} subquery can only be used inside WHERE".into()),
            ),
        }
    }

    /// `[p = (n)-->() | p]` / `[(n)-[:T]->(b) | b.name]` -- enumerates
    /// every match of `pattern` against the graph (already-bound named
    /// endpoints in `row` held fixed, exactly like `Expr::Pattern`'s own
    /// existential search reuses `build_match_plan`'s "already-bound var
    /// -> Seed, not a fresh scan" mechanism) and projects `projection`
    /// against each match's own resulting row, collecting into a
    /// `Value::List`. No limit on `eval_plan_with_limit` here (unlike
    /// `Expr::Pattern`'s `Some(1)`) -- a comprehension needs every match,
    /// not just whether one exists.
    ///
    /// A named path (`path_var: Some`) reuses `execute_match`'s own
    /// `name_pattern_for_path`/`assemble_path` pair verbatim -- same
    /// "synthesize internal names for any unnamed hop, assemble the path
    /// from those, then strip the synthesized keys (and the reserved
    /// variable-length-hop segment key, if any) back out" approach a real
    /// `MATCH p = ...` clause already uses, including over a single
    /// variable-length hop (TCK's Pattern2 `[9]`) -- also reuses
    /// `validate_named_path_pattern`'s own restriction on anything wider
    /// (a variable-length hop mixed with another hop) for the same reason
    /// it already applies to `MATCH`.
    fn eval_pattern_comprehension(
        &self,
        txn: Txn,
        spec: PatternComprehensionSpec<'_>,
        row: &BindingRow,
        guard: &ExecutionGuard<'_>,
    ) -> Result<Value, QueryError> {
        let PatternComprehensionSpec {
            path_var,
            pattern,
            where_clause,
            projection,
        } = spec;
        if path_var.is_some() {
            validate_named_path_pattern(pattern)?;
        }
        let carried_vars: HashSet<String> = row.keys().cloned().collect();
        let (named_pattern, synthesized) = match path_var {
            Some(_) => name_pattern_for_path(pattern),
            None => (pattern.clone(), HashSet::new()),
        };
        let wc: Option<Expr> = where_clause.as_deref().cloned();
        let plan = apply_index_seeks(build_match_plan(&named_pattern, &wc, &carried_vars)?, txn)?;
        let rows = self.eval_plan_with_limit(txn, &plan, std::slice::from_ref(row), guard, None)?;
        let mut out = Vec::with_capacity(rows.len());
        for mut r in rows {
            if let Some(pv) = path_var {
                let path_binding = assemble_path(&named_pattern, &r);
                for key in &synthesized {
                    r.remove(key);
                }
                r.insert(pv.clone(), path_binding);
            }
            out.push(self.eval_return_expr(txn, projection, &r, guard)?);
        }
        Ok(Value::List(out))
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<Option<bool>, QueryError> {
        value_to_bool3(&self.eval_return_expr(txn, expr, row, guard)?)
    }

    /// Deletes every `targets` expression's value, across every row --
    /// shared by `materialize_delete` (`DELETE`/`DETACH DELETE` as a
    /// statement tail) and `execute_match`'s own `QueryClause::Delete`
    /// (`DELETE ... WITH ...` mid-pattern). Edges are deleted immediately
    /// (no ordering constraint), but nodes are only *collected* into
    /// `pending_nodes` and deleted in a second pass, after every target
    /// across every row has contributed its own edges -- not deleted
    /// inline the way `delete_binding`/`delete_value` used to. A single
    /// non-`DETACH` `DELETE` naming *several* targets that collectively
    /// cover all of a node's edges (e.g. `DELETE pathColls.key[0],
    /// pathColls.key[1]`, two paths sharing a node, each contributing one
    /// of its two incident edges) must succeed -- deleting inline would
    /// try to delete the first path's node while the second path's edge
    /// (not yet processed) was still attached, a real bug found via TCK's
    /// Delete5 `[7]` once `{key: collect(p)}`-shaped composed expressions
    /// could reach this code path at all (previously rejected outright at
    /// compile time, before general aggregate composition was supported).
    fn delete_targets(
        &self,
        txn: Txn,
        write_txn: &WriteTransaction,
        targets: &[ReturnExpr],
        rows: &[BindingRow],
        detach: bool,
        guard: &ExecutionGuard<'_>,
    ) -> Result<(), QueryError> {
        let mut deleted_edges = HashSet::new();
        let mut pending_nodes = HashSet::new();
        // All-bare-variable target lists (`DELETE r`, `DELETE r, a, b` --
        // by far the common case, and the only shape a predicate-driven
        // bulk delete produces) never evaluate anything between edge
        // deletions, so the edge ids can be collected across every row
        // first and deleted in one `delete_edges_in_txn` batch: one
        // `WriteCtx` and one label-name resolution per distinct type,
        // instead of a whole-edge fetch plus a fresh `WriteCtx` (and its
        // table opens) per edge. Observably identical to deleting
        // inline -- with no expression evaluation in the loop there is no
        // read that could distinguish "deleted already" from "deleted at
        // the end", and `guard`'s deleted-edge-type bookkeeping is only
        // consulted by later statements. Any computed target (`list[0]`,
        // `map.key`, ...) falls back to the per-edge path below, whose
        // immediate deletes are what let a later target's evaluation
        // correctly error via `deleted_entity_access` on touching an
        // already-deleted entity.
        if targets.iter().all(|t| matches!(t, ReturnExpr::Var(_))) {
            let mut edge_ids: Vec<EdgeId> = Vec::new();
            for row in rows {
                for target in targets {
                    let ReturnExpr::Var(name) = target else {
                        unreachable!("checked all-Var above");
                    };
                    let binding = row
                        .get(name)
                        .ok_or_else(|| QueryError::UnboundVariable(name.clone()))?;
                    collect_delete_binding(
                        binding,
                        &mut deleted_edges,
                        &mut edge_ids,
                        &mut pending_nodes,
                    )?;
                }
            }
            for (id, label) in GraphStore::delete_edges_in_txn(write_txn, &edge_ids)? {
                guard.record_deleted_edge_type(id, label);
            }
        } else {
            for row in rows {
                for target in targets {
                    // A bare variable (`DELETE r, a, b`, by far the common
                    // case) deletes by the raw id already sitting in the row's
                    // `Binding` -- no existence check, no property fetch.
                    // That's what lets `DELETE r, a, b` work when two rows of
                    // the same undirected match both reference the same `a`/
                    // `b`/`r` (real, from TCK's Delete4 `[1]`): the second
                    // row's own dedup lookup must succeed even though the
                    // first row already deleted them. Anything else (`list[0]`,
                    // `map.key`, a whole path variable's *elements* accessed
                    // computedly, ...) has no such raw shortcut and goes
                    // through real evaluation instead -- which correctly does
                    // still error via `deleted_entity_access` if it tries to
                    // read a property off something already gone, since that's
                    // a genuine access, not just a re-statement of identity.
                    if let ReturnExpr::Var(name) = target {
                        let binding = row
                            .get(name)
                            .ok_or_else(|| QueryError::UnboundVariable(name.clone()))?;
                        delete_binding(
                            txn,
                            binding,
                            write_txn,
                            &mut deleted_edges,
                            &mut pending_nodes,
                            guard,
                        )?;
                    } else {
                        let value = self.eval_return_expr(txn, target, row, guard)?;
                        delete_value(
                            &value,
                            write_txn,
                            &mut deleted_edges,
                            &mut pending_nodes,
                            guard,
                        )?;
                    }
                }
            }
        }
        for id in pending_nodes {
            self.uncache_node(id);
            GraphStore::delete_node_in_txn(write_txn, id, detach)?;
        }
        Ok(())
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
        targets: &[ReturnExpr],
        rows: &[BindingRow],
        detach: bool,
        ret: &Option<ReturnTail>,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        self.delete_targets(txn, write_txn, targets, rows, detach, guard)?;
        let result = match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct, guard)?,
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        for row in rows {
            for item in items {
                self.apply_set_item(txn, write_txn, row, item, guard)?;
            }
        }
        match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct, guard),
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
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let write_txn = require_write_txn(txn);
        for row in rows {
            for item in items {
                apply_remove_item(self, write_txn, row, item)?;
            }
        }
        match ret {
            Some(rt) => self.materialize_return(txn, &rt.items, rows, rt.distinct, guard),
            None => Ok(QueryResult {
                columns: vec![],
                rows: vec![],
            }),
        }
    }

    /// `<match_stmt> UNION [ALL] <match_stmt> ...` — every part shares the
    /// same `txn` (one snapshot for a read-only union, one write
    /// transaction otherwise — see `is_read_only`'s own `Union` handling)
    /// but no bindings: each part is `execute_match`'d completely
    /// independently, matching real Cypher's own scoping. Column names
    /// must match exactly across every part (real Cypher's
    /// `DifferentColumnsInUnion` — checked here, once each part's real
    /// `QueryResult.columns` is in hand, rather than statically, since
    /// nothing else in this codebase infers a `RETURN` list's column
    /// names without evaluating it). `all: false` (plain `UNION`) dedups
    /// the combined rows via the same `dedup_rows` `RETURN DISTINCT`
    /// already uses; `all: true` keeps every row.
    fn materialize_union(
        &self,
        txn: Txn,
        parts: &[Statement],
        all: bool,
        guard: &ExecutionGuard<'_>,
    ) -> Result<QueryResult, QueryError> {
        let mut combined: Option<QueryResult> = None;
        for part in parts {
            let Statement::Match {
                clauses,
                tail,
                order_by,
                skip,
                limit,
            } = part
            else {
                unreachable!(
                    "union_stmt parts are always Statement::Match -- see parser::parse_union_stmt"
                )
            };
            let skip = self.resolve_skip_limit(txn, skip.as_deref(), "SKIP", guard)?;
            let limit = self.resolve_skip_limit(txn, limit.as_deref(), "LIMIT", guard)?;
            let result = self.execute_match(
                txn,
                clauses,
                tail,
                ResultModifiers {
                    order_by,
                    skip,
                    limit,
                },
                guard,
            )?;
            combined = Some(match combined {
                None => result,
                Some(mut acc) => {
                    if acc.columns != result.columns {
                        return Err(QueryError::Semantic(format!(
                            "UNION requires every part to return the same columns -- got {:?} \
                             and {:?}",
                            acc.columns, result.columns
                        )));
                    }
                    acc.rows.extend(result.rows);
                    acc
                }
            });
            guard.check_intermediate_rows(combined.as_ref().map(|r| r.rows.len()).unwrap_or(0))?;
        }
        let mut result = combined.expect("union_stmt grammar guarantees at least 2 parts");
        if !all {
            result.rows = dedup_rows(result.rows)?;
        }
        Ok(result)
    }

    fn apply_set_item(
        &self,
        txn: Txn,
        write_txn: &WriteTransaction,
        row: &BindingRow,
        item: &SetItem,
        guard: &ExecutionGuard<'_>,
    ) -> Result<(), QueryError> {
        match item {
            SetItem::Prop(pa, expr) => {
                let binding = row
                    .get(&pa.var)
                    .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
                // `SET` on a null binding is a documented no-op, same as
                // `DELETE`/`REMOVE` on one -- an `OPTIONAL MATCH` that found
                // nothing pads its variables with null (found via TCK's
                // Set1/Set3 "Ignore null when setting property/label"
                // scenarios).
                if matches!(binding, Binding::Value(PropertyValue::Null)) {
                    return Ok(());
                }
                let node_id = if let Binding::Node(id) = binding {
                    Some(*id)
                } else {
                    None
                };
                let edge_id = if let Binding::Edge(id) = binding {
                    Some(*id)
                } else {
                    None
                };
                if node_id.is_none() && edge_id.is_none() {
                    return Err(QueryError::UnboundVariable(format!(
                    "'{}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding",
                    pa.var
                )));
                }
                let value = self.eval_return_expr(txn, expr, row, guard)?;
                // `SET n.prop = null` *removes* the property in real Cypher
                // (found via TCK's Set2 "Set a Property to Null" scenarios,
                // which this codebase previously couldn't parse at all --
                // `SET` had no trailing RETURN to observe the result with, so
                // this bug was never exercised until that gap closed).
                // Storing a literal `PropertyValue::Null` instead is
                // observably different: `n.prop` still shows up as a
                // (nulled-out) key when a caller enumerates a node's own
                // props (e.g. this RETURN's own node-to-string rendering),
                // where a real missing property wouldn't. The RHS being
                // `null` is now a *runtime* fact (it's any `ReturnExpr`, not
                // just the `Literal::Null` token), not something checkable
                // from the AST alone -- `SET n.prop = coalesce(x, null)`
                // must remove the property too if `x` turns out null.
                if let Some(id) = node_id {
                    self.uncache_node(id);
                }
                if matches!(value, Value::Null) {
                    if let Some(id) = node_id {
                        GraphStore::remove_node_prop_in_txn(write_txn, id, &pa.prop)?;
                    }
                    if let Some(id) = edge_id {
                        GraphStore::remove_edge_prop_in_txn(write_txn, id, &pa.prop)?;
                    }
                } else {
                    let pv = value_to_storable_property(&value).ok_or_else(|| {
                    QueryError::Type(format!(
                        "property '{}' can't be stored -- MarsDB's node/edge properties are limited \
                         to null/bool/int/float/string/date/duration; a list/map/node/edge/path value \
                         (got {value:?}) isn't storable",
                        pa.prop
                    ))
                })?;
                    if let Some(id) = node_id {
                        GraphStore::set_node_prop_in_txn(write_txn, id, &pa.prop, pv.clone())?;
                    }
                    if let Some(id) = edge_id {
                        GraphStore::set_edge_prop_in_txn(write_txn, id, &pa.prop, pv)?;
                    }
                }
            }
            SetItem::Labels(var, labels) => {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                match binding {
                    Binding::Node(id) => {
                        self.uncache_node(*id);
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
            SetItem::MapAssign { var, value, merge } => {
                let binding = row
                    .get(var)
                    .ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                // Same null-is-a-no-op rule as the property arm above.
                if matches!(binding, Binding::Value(PropertyValue::Null)) {
                    return Ok(());
                }
                let node_id = if let Binding::Node(id) = binding {
                    Some(*id)
                } else {
                    None
                };
                let edge_id = if let Binding::Edge(id) = binding {
                    Some(*id)
                } else {
                    None
                };
                if node_id.is_none() && edge_id.is_none() {
                    return Err(QueryError::UnboundVariable(format!(
                        "'{var}' is a WITH-projected scalar, not a node/edge — SET needs a graph binding"
                    )));
                }
                if let Some(id) = node_id {
                    self.uncache_node(id);
                }
                let map_value = self.eval_return_expr(txn, value, row, guard)?;
                // A map literal is the common case, but real Cypher also
                // allows `SET r = a`/`SET r += a` where `a` is itself a
                // bound node/relationship -- copies its properties, same
                // as a map built from them would (TCK's Merge6 [6]/
                // Merge7 [4], "Copying properties from node").
                let entries = match map_value {
                    Value::Map(entries) => entries,
                    Value::Node(n) => n
                        .props
                        .into_iter()
                        .map(|(k, v)| (k, property_value_to_value(v)))
                        .collect(),
                    Value::Edge(e) => e
                        .props
                        .into_iter()
                        .map(|(k, v)| (k, property_value_to_value(v)))
                        .collect(),
                    other => {
                        return Err(QueryError::Type(format!(
                            "SET {var} = ...{} needs a map, node, or relationship, got {other:?}",
                            if *merge { " (+=)" } else { "" }
                        )))
                    }
                };
                // `SET n = {...}` (`merge: false`) replaces every existing
                // property -- delete whatever's already there first, not
                // just overwrite the map's own keys, or a key n already
                // had that the map doesn't mention would wrongly survive
                // (TCK's Set4 [2]/[3]/[4]).
                if !merge {
                    let existing_keys: Vec<String> = if let Some(id) = node_id {
                        deleted_entity_access(GraphStore::get_node_in_txn(txn, id)?)?
                            .props
                            .into_keys()
                            .collect()
                    } else {
                        deleted_entity_access(GraphStore::get_edge_in_txn(
                            txn,
                            edge_id.expect("node_id or edge_id is Some, checked above"),
                        )?)?
                        .props
                        .into_keys()
                        .collect()
                    };
                    for key in existing_keys {
                        if let Some(id) = node_id {
                            GraphStore::remove_node_prop_in_txn(write_txn, id, &key)?;
                        }
                        if let Some(id) = edge_id {
                            GraphStore::remove_edge_prop_in_txn(write_txn, id, &key)?;
                        }
                    }
                }
                // Either way, apply the map's own entries -- a `null`
                // value removes that one key (real Cypher's rule, same
                // "null means remove" convention `SetItem::Prop` already
                // has -- TCK's Set5 [4]), anything else sets it.
                for (key, entry_value) in entries {
                    if matches!(entry_value, Value::Null) {
                        if let Some(id) = node_id {
                            GraphStore::remove_node_prop_in_txn(write_txn, id, &key)?;
                        }
                        if let Some(id) = edge_id {
                            GraphStore::remove_edge_prop_in_txn(write_txn, id, &key)?;
                        }
                        continue;
                    }
                    let pv = value_to_storable_property(&entry_value).ok_or_else(|| {
                        QueryError::Type(format!(
                            "property '{key}' can't be stored -- MarsDB's node/edge properties are \
                             limited to null/bool/int/float/string/date/duration/list; a map/node/\
                             edge/path value (got {entry_value:?}) isn't storable"
                        ))
                    })?;
                    if let Some(id) = node_id {
                        GraphStore::set_node_prop_in_txn(write_txn, id, &key, pv.clone())?;
                    }
                    if let Some(id) = edge_id {
                        GraphStore::set_edge_prop_in_txn(write_txn, id, &key, pv)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// `materialize_delete`'s bare-variable fast path -- deletes straight off
/// the row's raw `Binding` (just an id), no existence check and no
/// property fetch, so re-referencing an already-deleted-this-statement
/// entity by identity (a later row of the same multi-row `DELETE`) is a
/// silent dedup no-op, not an error. Mirrors `delete_value`'s shape
/// (including the path/null/type-error handling) but over `Binding`/
/// `PathBinding` (raw ids) instead of `Value`/`PathElem` (fully
/// materialized records).
/// Deletes edge `id`, first stashing its (immutable, so safe to cache)
/// type into `guard` -- see `ExecutionGuard::deleted_edge_types`'s own
/// docs for why. The lookup can't fail with a real error here: `id` was
/// just read out of a live `Binding::Edge`/`PathBinding::Edge` this same
/// transaction, so its record is still there to fetch (deletion hasn't
/// happened yet -- that's the very next line).
fn record_and_delete_edge(
    txn: Txn,
    write_txn: &WriteTransaction,
    id: EdgeId,
    guard: &ExecutionGuard<'_>,
) -> Result<(), QueryError> {
    if let Some(edge) = GraphStore::get_edge_in_txn(txn, id)? {
        guard.record_deleted_edge_type(id, edge.label);
    }
    GraphStore::delete_edge_in_txn(write_txn, id)?;
    Ok(())
}

/// `delete_binding`'s collect-only twin for the batched all-bare-variable
/// path in `delete_targets`: identical target-shape rules (nodes pended,
/// path edges before path nodes, null a no-op, scalar/list/map a type
/// error), but edge ids go into `edge_ids` (deduped through
/// `deleted_edges`, preserving first-encounter order) for one
/// `delete_edges_in_txn` call instead of being deleted one `WriteCtx`
/// apiece.
fn collect_delete_binding(
    binding: &Binding,
    deleted_edges: &mut HashSet<EdgeId>,
    edge_ids: &mut Vec<EdgeId>,
    pending_nodes: &mut HashSet<NodeId>,
) -> Result<(), QueryError> {
    match binding {
        Binding::Node(id) => {
            pending_nodes.insert(*id);
        }
        Binding::Edge(id) => {
            if deleted_edges.insert(*id) {
                edge_ids.push(*id);
            }
        }
        Binding::Path(elems) => {
            for elem in elems {
                if let PathBinding::Edge(id) = elem {
                    if deleted_edges.insert(*id) {
                        edge_ids.push(*id);
                    }
                }
            }
            for elem in elems {
                if let PathBinding::Node(id) = elem {
                    pending_nodes.insert(*id);
                }
            }
        }
        // A null binding is a real, legal DELETE target -- an `OPTIONAL
        // MATCH` that didn't match pads its variables with null, and
        // deleting that is a documented no-op, not an error.
        Binding::Value(PropertyValue::Null) => {}
        Binding::Value(_) | Binding::List(_) | Binding::Map(_) => {
            return Err(QueryError::Type(
                "DELETE needs a node, relationship, or path, not a scalar/list/map".into(),
            ))
        }
    }
    Ok(())
}

fn delete_binding(
    txn: Txn,
    binding: &Binding,
    write_txn: &WriteTransaction,
    deleted_edges: &mut HashSet<EdgeId>,
    pending_nodes: &mut HashSet<NodeId>,
    guard: &ExecutionGuard<'_>,
) -> Result<(), QueryError> {
    match binding {
        Binding::Node(id) => {
            pending_nodes.insert(*id);
        }
        Binding::Edge(id) => {
            if deleted_edges.insert(*id) {
                record_and_delete_edge(txn, write_txn, *id, guard)?;
            }
        }
        Binding::Path(elems) => {
            for elem in elems {
                if let PathBinding::Edge(id) = elem {
                    if deleted_edges.insert(*id) {
                        record_and_delete_edge(txn, write_txn, *id, guard)?;
                    }
                }
            }
            for elem in elems {
                if let PathBinding::Node(id) = elem {
                    pending_nodes.insert(*id);
                }
            }
        }
        // A null binding is a real, legal DELETE target -- an `OPTIONAL
        // MATCH` that didn't match pads its variables with null, and
        // deleting that is a documented no-op, not an error.
        Binding::Value(PropertyValue::Null) => {}
        Binding::Value(_) | Binding::List(_) | Binding::Map(_) => {
            return Err(QueryError::Type(
                "DELETE needs a node, relationship, or path, not a scalar/list/map".into(),
            ))
        }
    }
    Ok(())
}

/// Deletes whatever `value` evaluated to -- a node, a relationship, every
/// node/edge in a path, or nothing at all for `null` (a documented no-op:
/// an `OPTIONAL MATCH` that didn't match pads its variables with null, and
/// deleting that is specified as silent, not an error). Anything else (a
/// list, a map, a bare scalar, ...) is a real `QueryError::Type` --
/// `DELETE`'s target must resolve to a graph element, unlike `SET`'s RHS.
/// Edges are deleted immediately; nodes are only collected into
/// `pending_nodes` -- `delete_targets` (the only caller) deletes them in
/// its own second pass, after every target across every row has had a
/// chance to delete its own edges first (see its own docs for why).
fn delete_value(
    value: &Value,
    write_txn: &WriteTransaction,
    deleted_edges: &mut HashSet<EdgeId>,
    pending_nodes: &mut HashSet<NodeId>,
    guard: &ExecutionGuard<'_>,
) -> Result<(), QueryError> {
    match value {
        Value::Node(n) => {
            pending_nodes.insert(n.id);
        }
        Value::Edge(e) => {
            if deleted_edges.insert(e.id) {
                guard.record_deleted_edge_type(e.id, e.label.clone());
                GraphStore::delete_edge_in_txn(write_txn, e.id)?;
            }
        }
        Value::Path(elems) => {
            for elem in elems {
                if let PathElem::Edge(e) = elem {
                    if deleted_edges.insert(e.id) {
                        guard.record_deleted_edge_type(e.id, e.label.clone());
                        GraphStore::delete_edge_in_txn(write_txn, e.id)?;
                    }
                }
            }
            for elem in elems {
                if let PathElem::Node(n) = elem {
                    pending_nodes.insert(n.id);
                }
            }
        }
        Value::Null => {}
        other => {
            return Err(QueryError::Type(format!(
                "DELETE needs a node, relationship, or path, got {other:?}"
            )))
        }
    }
    Ok(())
}

fn apply_remove_item(
    executor: &Executor<'_>,
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
                    executor.uncache_node(*id);
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
                    executor.uncache_node(*id);
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
        Some(Tail::Return(_, distinct)) | Some(Tail::ReturnStar(distinct)) => *distinct,
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
/// Returns whether executing `stmt` can mutate the graph. Public so callers
/// which execute generated or otherwise untrusted Cypher can enforce a
/// read-only policy using the same classification as the executor.
pub fn is_read_only(stmt: &Statement) -> bool {
    if let Statement::Union { parts, .. } = stmt {
        return parts.iter().all(is_read_only);
    }
    let Statement::Match {
        tail: Some(Tail::Return(_, _)) | Some(Tail::ReturnStar(_)),
        clauses,
        ..
    } = stmt
    else {
        return false;
    };
    !clauses.iter().any(|c| {
        matches!(
            c,
            QueryClause::Merge(_)
                | QueryClause::Set(_)
                | QueryClause::Delete { .. }
                | QueryClause::Remove(_)
                | QueryClause::Create(_)
                // A procedure is opaque to MarsDB -- it might write, so
                // any statement calling one is conservatively treated as
                // non-read-only too, same reasoning `Statement::
                // StandaloneCall` already gets for free (it isn't a
                // `Statement::Match` at all, so it never matches this
                // function's own read-only pattern above).
                | QueryClause::Call(_)
        )
    })
}

/// Recovers the real `&WriteTransaction` from a `Txn` for `execute_match`
/// tail/clause arms (`DELETE`/`SET`, both the terminal-tail and
/// `QueryClause::Set`'s own mid-statement form) that need `.insert`/
/// `.remove`, not just `Txn`'s read-only `get`/`iter`. Panics if given
/// `Txn::Read` — which can't happen: any of these make `is_read_only`
/// return `false`, so `Executor::execute` always opens a
/// `WriteTransaction` (and thus `Txn::Write`) before reaching this path.
fn require_write_txn(txn: Txn<'_>) -> &WriteTransaction {
    let Txn::Write(write_txn) = txn else {
        unreachable!(
            "materialize_delete/materialize_set/QueryClause::Set only reached via the \
             write-dispatch path in Executor::execute — is_read_only(stmt) is false for any \
             statement with one of these, so execute always opens a WriteTransaction for them"
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
        ReturnExpr::Arith(..) | ReturnExpr::Neg(..) => format!("col{idx}"),
        ReturnExpr::ListLit(..)
        | ReturnExpr::Index(..)
        | ReturnExpr::PropOf(..)
        | ReturnExpr::Slice(..)
        | ReturnExpr::ListComp { .. }
        | ReturnExpr::Quantifier { .. }
        | ReturnExpr::MapLit(..)
        | ReturnExpr::And(..)
        | ReturnExpr::Or(..)
        | ReturnExpr::Xor(..)
        | ReturnExpr::Not(..)
        | ReturnExpr::Compare(..)
        | ReturnExpr::IsNull(..)
        | ReturnExpr::In(..)
        | ReturnExpr::HasLabel(..)
        | ReturnExpr::PatternPredicate(..)
        | ReturnExpr::PatternComprehension { .. }
        | ReturnExpr::ExistsPattern { .. }
        | ReturnExpr::ExistsSubquery(_) => format!("col{idx}"),
    }
}

/// The name a `WITH`/`RETURN` item is known by afterward — its alias, or
/// a name derived from the expression (its bare var name, `col{i}`, etc).
/// `pub(crate)` so `explain.rs` can compute the same post-`WITH`
/// `carried_vars` set EXPLAIN needs without executing any rows.
pub(crate) fn with_item_output_name((i, item): (usize, &ReturnItem)) -> String {
    item.alias
        .clone()
        .unwrap_or_else(|| default_column_name(&item.expr, i))
}

/// True iff `expr` contains an aggregate call anywhere inside it, at any
/// depth — used to reject an aggregate nested inside another aggregate's
/// argument, or inside a non-aggregate expression's `CASE`/`Call`
/// arguments (an aggregate must be a return item's *entire* top-level
/// expression — see `validate_return_items`).
/// Collects every aggregate-bearing subexpression in `expr` (a `CountStar`
/// or an aggregate-named `Call`), in a fixed pre-order -- the same
/// traversal `contains_aggregate` uses, just gathering references instead
/// of stopping at the first `true`. Doesn't recurse *into* a found node's
/// own arguments (an aggregate's argument is folded per-row as a whole,
/// not decomposed further -- see `resolve_grouped_rows`). The resulting
/// order is what makes a composed item's per-row folding
/// (`resolve_grouped_rows`) and its per-group finishing
/// (`Executor::rewrite_composed_item`) agree on which accumulator is
/// which, without needing to name or otherwise identify individual
/// aggregate calls within one item's expression tree.
fn collect_agg_nodes<'a>(expr: &'a ReturnExpr, out: &mut Vec<&'a ReturnExpr>) {
    match expr {
        ReturnExpr::CountStar => out.push(expr),
        ReturnExpr::Call { name, args, .. } => {
            if is_aggregate_name(name) {
                out.push(expr);
            } else {
                for arg in args {
                    collect_agg_nodes(arg, out);
                }
            }
        }
        ReturnExpr::Case { test, whens, else_ } => {
            if let Some(t) = test.as_deref() {
                collect_agg_nodes(t, out);
            }
            for (w, t) in whens {
                collect_agg_nodes(w, out);
                collect_agg_nodes(t, out);
            }
            if let Some(e) = else_.as_deref() {
                collect_agg_nodes(e, out);
            }
        }
        ReturnExpr::Arith(l, _, r) => {
            collect_agg_nodes(l, out);
            collect_agg_nodes(r, out);
        }
        ReturnExpr::Neg(e) => collect_agg_nodes(e, out),
        ReturnExpr::ListLit(items) => {
            for item in items {
                collect_agg_nodes(item, out);
            }
        }
        ReturnExpr::Index(base, index) => {
            collect_agg_nodes(base, out);
            collect_agg_nodes(index, out);
        }
        ReturnExpr::PropOf(base, _) => collect_agg_nodes(base, out),
        ReturnExpr::Slice(base, start, end) => {
            collect_agg_nodes(base, out);
            if let Some(s) = start.as_deref() {
                collect_agg_nodes(s, out);
            }
            if let Some(e) = end.as_deref() {
                collect_agg_nodes(e, out);
            }
        }
        // Same `where_clause`-not-checked scope limitation as
        // `contains_aggregate`'s matching arm.
        ReturnExpr::ListComp {
            source, project, ..
        } => {
            collect_agg_nodes(source, out);
            if let Some(p) = project.as_deref() {
                collect_agg_nodes(p, out);
            }
        }
        ReturnExpr::Quantifier { source, .. } => collect_agg_nodes(source, out),
        ReturnExpr::MapLit(entries) => {
            for (_, v) in entries {
                collect_agg_nodes(v, out);
            }
        }
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            collect_agg_nodes(l, out);
            collect_agg_nodes(r, out);
        }
        ReturnExpr::Not(e) => collect_agg_nodes(e, out),
        ReturnExpr::Compare(l, _, r) => {
            collect_agg_nodes(l, out);
            collect_agg_nodes(r, out);
        }
        ReturnExpr::IsNull(e) => collect_agg_nodes(e, out),
        ReturnExpr::In(needle, haystack) => {
            collect_agg_nodes(needle, out);
            collect_agg_nodes(haystack, out);
        }
        ReturnExpr::Var(_)
        | ReturnExpr::Prop(_)
        | ReturnExpr::Lit(_)
        | ReturnExpr::HasLabel(..)
        | ReturnExpr::PatternPredicate(..)
        | ReturnExpr::PatternComprehension { .. }
        | ReturnExpr::ExistsPattern { .. }
        | ReturnExpr::ExistsSubquery(_) => {}
    }
}

pub(crate) fn contains_aggregate(expr: &ReturnExpr) -> bool {
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
        ReturnExpr::Neg(e) => contains_aggregate(e),
        ReturnExpr::ListLit(items) => items.iter().any(contains_aggregate),
        ReturnExpr::Index(base, index) => contains_aggregate(base) || contains_aggregate(index),
        ReturnExpr::PropOf(base, _) => contains_aggregate(base),
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
        ReturnExpr::In(needle, haystack) => {
            contains_aggregate(needle) || contains_aggregate(haystack)
        }
        ReturnExpr::Var(_)
        | ReturnExpr::Prop(_)
        | ReturnExpr::Lit(_)
        | ReturnExpr::HasLabel(..)
        | ReturnExpr::PatternPredicate(..)
        // A pattern comprehension's projection runs against its own
        // per-match row, not the outer query's group -- an aggregate
        // inside it wouldn't mean "aggregate across the outer group,"
        // it'd need its own separate grouping concept this codebase
        // doesn't have, so (like `PatternPredicate`) it's opaque here
        // rather than searched into.
        | ReturnExpr::PatternComprehension { .. }
        | ReturnExpr::ExistsPattern { .. }
        | ReturnExpr::ExistsSubquery(_) => false,
    }
}

/// True iff any item's top-level expression is an aggregate call —
/// `materialize_with`/`materialize_return` dispatch to the grouping path
/// iff this is true, otherwise the existing row-at-a-time path runs
/// completely unchanged (zero perf/behavior impact on non-aggregating
/// queries).
/// `try_fast_expand_expand_count`'s direction support: single concrete
/// direction only — `Either` needs the two-call-plus-dedupe treatment the
/// generic path does, out of the fast path's scope.
fn fast_direction(dir: ExpandDirection) -> Option<Direction> {
    match dir {
        ExpandDirection::Out => Some(Direction::Out),
        ExpandDirection::In => Some(Direction::In),
        ExpandDirection::Either => None,
    }
}

/// Single-type (`Some`) or untyped (`None`) relationship filter — the
/// multi-type `[:A|B]` list needs per-type iteration, out of scope.
/// Outer `None` = unsupported shape, inner `Option` = the filter itself.
#[allow(clippy::option_option)]
fn fast_label(labels: &[String]) -> Option<Option<&str>> {
    match labels {
        [] => Some(None),
        [one] => Some(Some(one.as_str())),
        _ => None,
    }
}

/// Does this (sub)plan contain any expansion or externally-seeded input?
/// The fast path evaluates its leaf through the generic stream, but only
/// when the leaf is a pure scan/seek/filter chain.
fn plan_contains_expansion(plan: &LogicalPlan) -> bool {
    match plan {
        LogicalPlan::Expand { .. }
        | LogicalPlan::VarExpand { .. }
        | LogicalPlan::MatchRelList { .. }
        | LogicalPlan::Seed { .. } => true,
        LogicalPlan::Filter { input, .. } => plan_contains_expansion(input),
        LogicalPlan::AllNodesScan { .. }
        | LogicalPlan::NodeByLabelScan { .. }
        | LogicalPlan::IndexSeek { .. } => false,
    }
}

pub(crate) fn has_aggregate(items: &[ReturnItem]) -> bool {
    // `contains_aggregate`, not a narrower "is the item's whole top-level
    // expression itself an aggregate call" check -- an aggregate nested
    // inside a wrapping expression (`1 + count(x)`, real Cypher composition
    // -- see `resolve_grouped_rows`) still needs to route to the grouping
    // path, both to actually compute it and so `validate_return_items` gets
    // a chance to reject an invalid composition with a clear error. A
    // narrower top-level-only check here would let such a query silently
    // take the ordinary per-row path instead (iterating `rows` directly,
    // which is empty for an empty MATCH), producing the wrong row count
    // instead of the right (or correctly rejected) one.
    items.iter().any(|item| contains_aggregate(&item.expr))
}

/// True iff `expr` contains a call to `rand()` anywhere inside it, at any
/// depth -- same traversal shape as `contains_aggregate`, used only to
/// reject `rand()` as (part of) an aggregate's own argument (see
/// `validate_return_items`); `rand()` elsewhere in a query is completely
/// fine.
fn contains_rand_call(expr: &ReturnExpr) -> bool {
    match expr {
        ReturnExpr::Call { name, args, .. } => {
            name.eq_ignore_ascii_case("rand") || args.iter().any(contains_rand_call)
        }
        ReturnExpr::Case { test, whens, else_ } => {
            test.as_deref().is_some_and(contains_rand_call)
                || whens
                    .iter()
                    .any(|(w, t)| contains_rand_call(w) || contains_rand_call(t))
                || else_.as_deref().is_some_and(contains_rand_call)
        }
        ReturnExpr::Arith(l, _, r) => contains_rand_call(l) || contains_rand_call(r),
        ReturnExpr::Neg(e) => contains_rand_call(e),
        ReturnExpr::ListLit(items) => items.iter().any(contains_rand_call),
        ReturnExpr::Index(base, index) => contains_rand_call(base) || contains_rand_call(index),
        ReturnExpr::PropOf(base, _) => contains_rand_call(base),
        ReturnExpr::Slice(base, start, end) => {
            contains_rand_call(base)
                || start.as_deref().is_some_and(contains_rand_call)
                || end.as_deref().is_some_and(contains_rand_call)
        }
        ReturnExpr::ListComp {
            source, project, ..
        } => contains_rand_call(source) || project.as_deref().is_some_and(contains_rand_call),
        ReturnExpr::Quantifier { source, .. } => contains_rand_call(source),
        ReturnExpr::MapLit(entries) => entries.iter().any(|(_, v)| contains_rand_call(v)),
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            contains_rand_call(l) || contains_rand_call(r)
        }
        ReturnExpr::Not(e) => contains_rand_call(e),
        ReturnExpr::Compare(l, _, r) => contains_rand_call(l) || contains_rand_call(r),
        ReturnExpr::IsNull(e) => contains_rand_call(e),
        ReturnExpr::In(needle, haystack) => {
            contains_rand_call(needle) || contains_rand_call(haystack)
        }
        ReturnExpr::CountStar
        | ReturnExpr::Var(_)
        | ReturnExpr::Prop(_)
        | ReturnExpr::Lit(_)
        | ReturnExpr::HasLabel(..)
        | ReturnExpr::PatternPredicate(..)
        // Same opaque treatment as `contains_aggregate`'s own arm above --
        // a pattern comprehension's projection is checked once it's
        // actually evaluated per match, not searched into ahead of time.
        | ReturnExpr::PatternComprehension { .. }
        | ReturnExpr::ExistsPattern { .. }
        | ReturnExpr::ExistsSubquery(_) => false,
    }
}

/// `RETURN *`/`RETURN DISTINCT *` resolved into the equivalent concrete
/// item list -- one bare-`Var` item per currently-bound name, sorted
/// alphabetically (real Cypher's own `RETURN *` column order, confirmed
/// against the TCK's own multi-variable scenarios, not introduction
/// order). Shared by `semantic.rs` (`scope.keys()`) and this file's own
/// `execute_match` (`carried_vars`) -- each already has its own accurate
/// bound-name set on hand at the point `Tail::ReturnStar` is reached, so
/// resolving it there (rather than via a separate whole-AST-mutation
/// pass before execution) needs no `&mut Statement` ripple through
/// `Executor::execute`'s public signature. Real Cypher's own
/// `NoVariablesInScope` compile-time error when nothing is bound at all
/// (TCK's Return7 `[2]`, `MATCH () RETURN *`). `WITH *` doesn't share this
/// restriction -- an empty `WITH *` is a legal, if useless, "carry forward
/// nothing" no-op (TCK's Create3 `[2]`/`[3]`: `MATCH () CREATE () WITH *
/// CREATE ()`, every token anonymous) -- see `with_star_items` below.
pub(crate) fn return_star_items(
    names: impl Iterator<Item = String>,
) -> Result<Vec<ReturnItem>, QueryError> {
    let names: Vec<String> = names.collect();
    if names.is_empty() {
        return Err(QueryError::Semantic(
            "RETURN * needs at least one variable in scope".into(),
        ));
    }
    Ok(star_items(names))
}

/// `WITH *`'s own version of `return_star_items` -- same alphabetical
/// `Var`-per-name expansion, but tolerates an empty name set instead of
/// erroring (see that function's docs for why the two differ).
pub(crate) fn with_star_items(names: impl Iterator<Item = String>) -> Vec<ReturnItem> {
    star_items(names.collect())
}

fn star_items(mut names: Vec<String>) -> Vec<ReturnItem> {
    names.sort();
    names
        .into_iter()
        .map(|name| ReturnItem {
            expr: ReturnExpr::Var(name),
            alias: None,
        })
        .collect()
}

/// Validates a RETURN/WITH item list before any row is processed. Two
/// checks, both real Cypher compile-time errors:
///
/// - Every aggregate call (found anywhere -- not just a return item's
///   whole top-level expression, since `RETURN a, count(a) + 3`-style
///   composition is real Cypher, TCK's Return6 `[2]` etc) has the right
///   number of arguments, doesn't nest another aggregate inside its own
///   argument (`NestedAggregation`), and isn't given a non-deterministic
///   argument like `rand()` (`NonConstantExpression`).
/// - Once *any* item aggregates, every other item's own non-aggregate
///   leaf (a bare `Var`/`Prop` used outside any aggregate call) must
///   match some *other* item's whole top-level expression verbatim
///   (`AmbiguousAggregationExpression`, TCK's Return6 `[20]`/`[21]`) --
///   real Cypher's rule that a value used alongside an aggregate must
///   itself be an explicit grouping key, not just something that happens
///   to be in scope. A literal/param is always fine (same value on every
///   row, nothing to group by). This is checked by recursing into every
///   item whose expression contains an aggregate anywhere, stopping at
///   each aggregate-bearing subexpression itself (its own argument
///   doesn't need to be grouping-key-safe -- it's folded per row).
pub(crate) fn validate_return_items(items: &[ReturnItem]) -> Result<(), QueryError> {
    for item in items {
        if contains_aggregate(&item.expr) {
            validate_composed_expr(&item.expr, items)?;
        }
    }
    Ok(())
}

/// Whether `expr` (a leaf found inside some *other* composed expression)
/// refers to `item` -- either structurally (`item.expr == *expr`) or, for
/// a bare `Var`, by `item`'s own output *alias* (`RETURN me.age AS age
/// ... ORDER BY age + count(...)`, TCK's ReturnOrderBy6 `[2]`: `age`
/// alone doesn't structurally equal `me.age`, but it's still exactly
/// item `age`'s value). Shared by `validate_composed_expr`'s compile-time
/// check and `Executor::rewrite_composed_item`'s matching runtime lookup
/// -- both need to agree on what counts as "the same grouping key,"
/// including this by-alias case, or one would accept what the other
/// can't actually evaluate.
pub(crate) fn item_matches_leaf(expr: &ReturnExpr, index: usize, item: &ReturnItem) -> bool {
    item.expr == *expr
        || matches!(expr, ReturnExpr::Var(name) if *name == with_item_output_name((index, item)))
}

pub(crate) fn validate_composed_expr(
    expr: &ReturnExpr,
    items: &[ReturnItem],
) -> Result<(), QueryError> {
    if matches!(expr, ReturnExpr::CountStar) {
        return Ok(());
    }
    if let ReturnExpr::Call { name, args, .. } = expr {
        if is_aggregate_name(name) {
            // `percentileCont`/`percentileDisc` take a second argument
            // (the percentile) alongside the value being aggregated —
            // every other aggregate takes exactly one.
            let expected_args = if is_percentile_name(name) { 2 } else { 1 };
            if args.len() != expected_args {
                return Err(QueryError::Semantic(if expected_args == 2 {
                    format!("{name}() takes exactly two arguments (the value, then the percentile)")
                } else {
                    format!(
                        "{name}() takes exactly one argument (use count(*) for a row count with no argument)"
                    )
                }));
            }
            for arg in args {
                if contains_aggregate(arg) {
                    return Err(QueryError::Semantic(format!(
                        "aggregate function '{name}' can't take another aggregate as an argument"
                    )));
                }
                // `count(rand())` etc -- an aggregate's argument must be
                // deterministic per row for grouping/re-execution to have
                // well-defined semantics, which `rand()` (a fresh value on
                // every call, see its own docs) fundamentally breaks. Real
                // Cypher rejects this at compile time (TCK's Return6
                // [15], `NonConstantExpression`), not just "whatever value
                // it happens to produce."
                if contains_rand_call(arg) {
                    return Err(QueryError::Semantic(format!(
                        "aggregate function '{name}' can't take a non-deterministic expression \
                         (e.g. rand()) as an argument"
                    )));
                }
            }
            return Ok(());
        }
    }
    if matches!(expr, ReturnExpr::Var(_) | ReturnExpr::Prop(_)) {
        let is_grouping_key = items
            .iter()
            .enumerate()
            .any(|(i, it)| item_matches_leaf(expr, i, it) && !contains_aggregate(&it.expr));
        return if is_grouping_key {
            Ok(())
        } else {
            Err(QueryError::Semantic(format!(
                "{expr:?} is used alongside an aggregate function but isn't itself one of this \
                 RETURN/WITH's own items -- once any item aggregates, every other value used \
                 with it must be listed as its own explicit grouping key"
            )))
        };
    }
    // `Lit`/`HasLabel`/`PatternPredicate`/`PatternComprehension` need no
    // check here: a literal is the same value on every row (nothing to
    // group by), and the other three are opaque leaves for this same
    // reason `contains_aggregate`/`collect_agg_nodes` treat them that way
    // (see their own docs) -- not reachable with real content to check
    // since none can themselves contain an aggregate.
    match expr {
        ReturnExpr::Case { test, whens, else_ } => {
            if let Some(t) = test.as_deref() {
                validate_composed_expr(t, items)?;
            }
            for (w, t) in whens {
                validate_composed_expr(w, items)?;
                validate_composed_expr(t, items)?;
            }
            if let Some(e) = else_.as_deref() {
                validate_composed_expr(e, items)?;
            }
        }
        ReturnExpr::Call { args, .. } => {
            for arg in args {
                validate_composed_expr(arg, items)?;
            }
        }
        ReturnExpr::Arith(l, _, r) => {
            validate_composed_expr(l, items)?;
            validate_composed_expr(r, items)?;
        }
        ReturnExpr::Neg(e) => validate_composed_expr(e, items)?,
        ReturnExpr::ListLit(list_items) => {
            for item in list_items {
                validate_composed_expr(item, items)?;
            }
        }
        ReturnExpr::Index(base, index) => {
            validate_composed_expr(base, items)?;
            validate_composed_expr(index, items)?;
        }
        ReturnExpr::PropOf(base, _) => validate_composed_expr(base, items)?,
        ReturnExpr::Slice(base, start, end) => {
            validate_composed_expr(base, items)?;
            if let Some(s) = start.as_deref() {
                validate_composed_expr(s, items)?;
            }
            if let Some(e) = end.as_deref() {
                validate_composed_expr(e, items)?;
            }
        }
        // `source` may itself be a (possibly composed) aggregate --
        // `[x IN collect(p) | head(nodes(x))]` aggregates once per group
        // to build the list, then the comprehension iterates its result
        // normally (TCK's List12 [4]/[5], real and required) -- recursed
        // into below via the generic `Call`/`Arith`/etc. machinery, same
        // as any other composed leaf. `project`, in contrast, runs once
        // *per element* of that already-built list -- an aggregate
        // there has no defined semantics at all (real Cypher flatly
        // rejects it, TCK's List12 [7], "Fail when using aggregation in
        // list comprehension") and `resolve_grouped_rows` has no
        // "fold once per group, then run per element" fold shape for it
        // anyway, so it's checked directly here rather than falling
        // through to the generic recursion below, which would otherwise
        // validate (and `rewrite_composed_item` would then evaluate) a
        // nested aggregate as if it were an ordinary composed leaf.
        ReturnExpr::ListComp {
            source,
            project,
            where_clause,
            ..
        } => {
            if project.as_deref().is_some_and(contains_aggregate) {
                return Err(QueryError::Semantic(
                    "an aggregate function can't be used inside a list comprehension's projection"
                        .into(),
                ));
            }
            validate_composed_expr(source, items)?;
            // `where_clause` isn't checked -- same scope limitation as
            // `contains_aggregate`'s own matching arm.
            let _ = where_clause;
        }
        ReturnExpr::Quantifier { source, .. } => validate_composed_expr(source, items)?,
        ReturnExpr::MapLit(entries) => {
            for (_, v) in entries {
                validate_composed_expr(v, items)?;
            }
        }
        ReturnExpr::And(l, r) | ReturnExpr::Or(l, r) | ReturnExpr::Xor(l, r) => {
            validate_composed_expr(l, items)?;
            validate_composed_expr(r, items)?;
        }
        ReturnExpr::Not(e) => validate_composed_expr(e, items)?,
        ReturnExpr::Compare(l, _, r) => {
            validate_composed_expr(l, items)?;
            validate_composed_expr(r, items)?;
        }
        ReturnExpr::IsNull(e) => validate_composed_expr(e, items)?,
        ReturnExpr::In(needle, haystack) => {
            validate_composed_expr(needle, items)?;
            validate_composed_expr(haystack, items)?;
        }
        ReturnExpr::CountStar
        | ReturnExpr::Var(_)
        | ReturnExpr::Prop(_)
        | ReturnExpr::Lit(_)
        | ReturnExpr::HasLabel(..)
        | ReturnExpr::PatternPredicate(..)
        | ReturnExpr::PatternComprehension { .. }
        | ReturnExpr::ExistsPattern { .. }
        | ReturnExpr::ExistsSubquery(_) => {}
    }
    Ok(())
}

/// Same rules as `validate_composed_expr` (reused directly, first), plus
/// one more real Cypher only enforces for an ORDER BY key specifically,
/// not for a RETURN/WITH item's own composed expression: every
/// aggregate-bearing subexpression found anywhere in it must itself
/// verbatim/alias-match some existing RETURN/WITH item (TCK's
/// WithOrderBy4 `[14]`, "Fail on sorting by a non-projected aggregation
/// on an expression" -- `ORDER BY sum(x)` when the WITH only computes
/// `min(x)`, a *different* aggregate over the same argument, is a real
/// compile-time error, not "just fold it separately"). A RETURN/WITH
/// item's own composed expression has no such restriction -- `RETURN a,
/// count(a) + sum(b)` folds both `count(a)` and `sum(b)` fresh as part of
/// evaluating that one item, with nothing else either needs to match.
pub(crate) fn validate_order_by_composed_expr(
    expr: &ReturnExpr,
    items: &[ReturnItem],
) -> Result<(), QueryError> {
    validate_composed_expr(expr, items)?;
    let mut agg_nodes = Vec::new();
    collect_agg_nodes(expr, &mut agg_nodes);
    for node in agg_nodes {
        let matches_item = items
            .iter()
            .enumerate()
            .any(|(i, it)| item_matches_leaf(node, i, it));
        if !matches_item {
            return Err(QueryError::Semantic(
                "ORDER BY aggregate does not match any RETURN/WITH item".into(),
            ));
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
        Binding::List(items) => HashKey::List(
            items
                .iter()
                .map(value_hash_key)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        // A path's identity is its exact node/edge sequence, in order --
        // same graph-identity-by-id convention as the `Node`/`Edge` arms
        // above, just walked element-by-element (found via TCK's
        // Pattern2 [8]: `WITH [p = (n)-->() | p] AS ps, count(b) AS c`
        // makes `ps` -- a list of paths -- an implicit GROUP BY key,
        // real Cypher's own rule that every non-aggregate WITH/RETURN
        // item groups by).
        Binding::Path(elems) => HashKey::List(
            elems
                .iter()
                .map(|e| match e {
                    PathBinding::Node(id) => HashKey::Node(*id),
                    PathBinding::Edge(id) => HashKey::Edge(*id),
                })
                .collect(),
        ),
        // Same canonical-sorted-entries encoding as `value_hash_key`'s
        // matching `Value::Map` arm (a `BTreeMap` already iterates in
        // sorted key order).
        Binding::Map(m) => HashKey::List(
            m.iter()
                .map(|(k, v)| -> Result<HashKey, QueryError> {
                    Ok(HashKey::List(vec![
                        HashKey::Str(k.clone()),
                        value_hash_key(v)?,
                    ]))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    })
}

/// Projects one of `ProcedureProvider::call`'s raw output rows (positional,
/// `sig.outputs.len()` values in that order) down to whatever `yield_items`
/// actually asked for -- `YIELD *` keeps every output under its own name;
/// an explicit item list picks out just those (by the procedure's own
/// declared name, not any rename yet) and pairs each with its `AS` alias
/// if it had one, same output order the `YIELD` itself was written in
/// (TCK's Call5 `[3]`: order is irrelevant to the *result*, but this still
/// preserves whatever order was written, which `materialize_return`-style
/// column ordering downstream expects to already be correct).
fn project_call_row(
    sig: &ProcedureSignature,
    proc_row: &[Value],
    yield_items: &CallYield,
) -> Result<Vec<Value>, QueryError> {
    match yield_items {
        CallYield::Star => Ok(proc_row.to_vec()),
        CallYield::Items(items, _) => items
            .iter()
            .map(|(name, _)| {
                let idx = sig.outputs.iter().position(|o| o == name).ok_or_else(|| {
                    QueryError::Semantic(format!(
                        "'{name}' isn't a declared output of this procedure"
                    ))
                })?;
                Ok(proc_row[idx].clone())
            })
            .collect(),
    }
}

/// Coarse compile-time-shaped argument-type check (TCK's Call2
/// `[5]`/`[6]`: passing a `BOOLEAN` where `INTEGER` is declared must
/// error, even against an empty mock table that would otherwise just
/// silently return zero rows). `Value::Null` always matches regardless of
/// declared type -- every signature this codebase's own callers declare
/// is nullable (`INTEGER?` etc, TCK's Call4), and there's no dedicated
/// non-null marker to check against anyway. An unrecognized type name is
/// tolerated (accepts anything) rather than rejected -- this is a coarse
/// sanity check for the handful of type names TCK's own procedures
/// actually declare (`INTEGER`/`FLOAT`/`NUMBER`/`STRING`/`BOOLEAN`), not a
/// full type system.
fn value_matches_declared_type(value: &Value, declared: &str) -> bool {
    if matches!(value, Value::Null) {
        return true;
    }
    let is_int = matches!(
        value,
        Value::Literal(Literal::Int(_)) | Value::Property(PropertyValue::Int(_))
    );
    let is_float = matches!(
        value,
        Value::Literal(Literal::Float(_)) | Value::Property(PropertyValue::Float(_))
    );
    match declared.trim_end_matches('?').to_ascii_uppercase().as_str() {
        "INTEGER" => is_int,
        "FLOAT" | "NUMBER" => is_int || is_float,
        "STRING" => matches!(
            value,
            Value::Literal(Literal::String(_)) | Value::Property(PropertyValue::String(_))
        ),
        "BOOLEAN" => matches!(
            value,
            Value::Literal(Literal::Bool(_)) | Value::Property(PropertyValue::Bool(_))
        ),
        _ => true,
    }
}

/// Converts a finished `AggAcc::finish()` result to the `Binding` it's
/// carried as through a `WITH` boundary — `collect()`'s `Value::List`
/// needs `Binding::List`, not `Binding::Value(PropertyValue::List(_))`:
/// `Binding::List` carries full `Value` elements (a `Node`/`Edge`'s real
/// id, restorable graph identity), while `PropertyValue::List` is the
/// flatter, storage-format shape (scalar elements only) -- collapsing a
/// `collect()` of nodes down to that would lose the ability to keep
/// traversing from them after the `WITH`. Everything else collapses to
/// `Binding::Value` same as any other computed WITH item.
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
            if rel.hop_range.is_some() {
                // A variable-length hop's own internally-traversed edges
                // are exposed via a fresh synthesized binding name (same
                // `fresh()` mechanism as every other anonymous token
                // here, so multiple variable-length hops in one pattern
                // each get their own, no collision -- TCK's Match6
                // `[17]`), read by `planner::build_match_plan` (its
                // `VarExpand`'s `path_segment_var`) and `assemble_path`.
                // The user's own real rel-list variable, if this hop had
                // one (`p = (a)-[r*1..3]->(b)`, TCK's Match9 `[9]`), is
                // preserved separately in `rel_list_var` rather than lost
                // to this overwrite -- `var` itself is always this hop's
                // internal path-segment bookkeeping name from here on.
                rel.rel_list_var = rel.var.take();
                rel.var = Some(fresh(&mut counter, &mut synthesized));
                rel.capture_path_segment = true;
            } else if rel.var.is_none() {
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
        if rel.capture_path_segment {
            // A variable-length hop's own segment, deposited by
            // `expand_variable_row` under this hop's own synthesized
            // `rel.var` -- already the exact alternating Edge/Node/.../
            // Node sequence this hop contributes, ending at `node`'s own
            // binding (so no separate `path_node_id(node.var, ...)` read
            // is needed after this).
            let Some(Binding::Path(segment)) = rel.var.as_deref().and_then(|v| row.get(v)) else {
                return Binding::Value(PropertyValue::Null);
            };
            elems.extend(segment.iter().cloned());
            continue;
        }
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

/// `[r:TYPE*1..3]`'s own `r` -- real Cypher binds the traversed
/// relationships as a *list*, fully materialized (not just ids the way
/// `path_segment_var`'s cheaper `Binding::Path` segment stays), since
/// `Binding::List` -- like every other post-projection value shape --
/// only ever holds already-resolved `Value`s (TCK's Match4 `[1]`/`[6]`).
fn segment_edges_to_list(txn: Txn, segment: &[PathBinding]) -> Result<Binding, QueryError> {
    let edges = segment
        .iter()
        .filter_map(|elem| match elem {
            PathBinding::Edge(id) => Some(*id),
            PathBinding::Node(_) => None,
        })
        .map(|id| {
            let edge = deleted_entity_access(GraphStore::get_edge_in_txn(txn, id)?)?;
            Ok(Value::Edge(edge))
        })
        .collect::<Result<Vec<_>, QueryError>>()?;
    Ok(Binding::List(edges))
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

/// Coerces a materialized `Value` down to a `PropertyValue` for storing in
/// `Binding::Value` — used by `item_binding` for a computed (non-bare-var)
/// WITH/RETURN item. `Value::Node`/`Edge` can't occur here in practice (no
/// non-aggregate `ReturnExpr` form produces one except `Var`, which takes
/// the bare-variable path instead), and a bare `collect()` result is
/// routed to `Binding::List` before reaching here (see `has_aggregate`) --
/// both still fall back to `Null` rather than needing a fallible signature
/// for an unreachable case. `Value::List` genuinely *can* reach here now,
/// though (`WITH n.numbers + [4] AS x` -- a real computed list expression,
/// not a bare `collect()`, once list-valued properties round-trip through
/// `lookup_prop_value` as real `Value::List`s) -- recurses per-element,
/// same as `value_to_storable_property`'s own list handling.
fn value_to_property_value(v: &Value) -> PropertyValue {
    match v {
        Value::Null => PropertyValue::Null,
        Value::Property(pv) => pv.clone(),
        Value::Literal(lit) => literal_to_value(lit),
        Value::List(items) => {
            PropertyValue::List(items.iter().map(value_to_property_value).collect())
        }
        Value::Node(_) | Value::Edge(_) | Value::Map(_) | Value::Path(_) => PropertyValue::Null,
    }
}

/// `eval_props_to_values`'s stricter cousin of `value_to_property_value`
/// above -- a CREATE/SET prop value that evaluates to a node/edge/path/map
/// is a real, reportable error (`None` here), not a silent `Null`.
/// `value_to_property_value`'s silent-`Null` fallback is correct at *its*
/// call sites (a WITH-projected scalar, where those shapes genuinely can't
/// occur — see its own doc comment) but was never meant for CREATE/SET's
/// prop value, where writing one of those is a real, everyday mistake
/// (`CREATE (n {tags: some_node})`) that should say so, not silently store
/// `null`. `Value::List` *is* storable (`PropertyValue::List`, real
/// Cypher/Neo4j's own "homogeneous array property" shape) -- recurses
/// per-element, so a list containing something unstorable (a nested list
/// isn't rejected here, since no TCK scenario tests that restriction and
/// nothing about `PropertyValue::List`'s own storage format requires it,
/// but a node/edge/path/map element still correctly fails the whole list).
fn value_to_storable_property(v: &Value) -> Option<PropertyValue> {
    match v {
        Value::Null => Some(PropertyValue::Null),
        Value::Property(pv) => Some(pv.clone()),
        Value::Literal(lit) => Some(literal_to_value(lit)),
        Value::List(items) => Some(PropertyValue::List(
            items
                .iter()
                .map(value_to_storable_property)
                .collect::<Option<Vec<_>>>()?,
        )),
        Value::Node(_) | Value::Edge(_) | Value::Map(_) | Value::Path(_) => None,
    }
}

/// `value_to_storable_property`'s inverse -- turns a raw stored/bound
/// `PropertyValue` back into a real `Value`, the read-time counterpart
/// every property-access site (`lookup_prop_value`, `binding_to_value`,
/// `eval_projected_expr`'s node/edge prop arms) needs. A scalar wraps as
/// `Value::Property` exactly as before; `PropertyValue::List` becomes a
/// genuine `Value::List` (not `Value::Property(PropertyValue::List(_))`)
/// so every existing list operation (`size()`, `tail()`, indexing, `IN`,
/// `UNWIND`, ...) -- all of which pattern-match on `Value::List`
/// specifically -- works transparently on a property-sourced list the
/// same as a list literal/`collect()` result, with no special-casing
/// needed anywhere else. `PropertyValue::Null` collapses to `Value::Null`,
/// matching every other property-read site's existing null convention.
fn property_value_to_value(pv: PropertyValue) -> Value {
    match pv {
        PropertyValue::Null => Value::Null,
        PropertyValue::List(items) => {
            Value::List(items.into_iter().map(property_value_to_value).collect())
        }
        other => Value::Property(other),
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

pub(crate) fn literal_to_value(lit: &Literal) -> PropertyValue {
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

/// `Either` (undirected `-[r:TYPE]-`) has no single storage-level call —
/// query both directions and dedupe by `edge_id` (a self-loop would
/// otherwise appear twice, once from each direction's adjacency table).
/// Multiple `rel_labels` (`[:A|B]`) has no single storage-level call
/// either — `GraphStore::neighbors_in_txn` only ever filters by one label
/// at a time, so this makes one call per type (per direction) and
/// dedupes by `edge_id` across all of them, same technique as `Either`
/// above (an edge whose type is in `rel_labels` is only ever returned by
/// exactly one of those per-type calls, so the only real duplication risk
/// is the same direction-crossing one `Either` already handles). Empty
/// `rel_labels` means untyped — matches any relationship, same as
/// `neighbors_in_txn`'s own `None` behavior.
fn neighbors_for_direction(
    txn: Txn,
    node: NodeId,
    direction: ExpandDirection,
    rel_labels: &[String],
) -> Result<Vec<AdjEntry>, QueryError> {
    let dirs: &[Direction] = match direction {
        ExpandDirection::Out => &[Direction::Out],
        ExpandDirection::In => &[Direction::In],
        ExpandDirection::Either => &[Direction::Out, Direction::In],
    };
    let mut out = Vec::new();
    let mut seen: HashSet<EdgeId> = HashSet::new();
    let label_filters: Vec<Option<&str>> = if rel_labels.is_empty() {
        vec![None]
    } else {
        rel_labels.iter().map(|l| Some(l.as_str())).collect()
    };
    for label in label_filters {
        for &dir in dirs {
            for entry in GraphStore::neighbors_in_txn(txn, node, dir, label)? {
                if seen.insert(entry.edge_id) {
                    out.push(entry);
                }
            }
        }
    }
    Ok(out)
}

/// `<expr>.prop` where `<expr>` isn't a bare row variable (`ReturnExpr::
/// PropOf`, e.g. `startNode(r).id`, `head(nodes(p)).name`, `{a: 1}.a`) --
/// unlike `lookup_prop_value`'s `Prop(PropAccess)` arm, there's no row/txn
/// lookup to do here, `v` already *is* the fully-evaluated base value, so
/// this reads straight off it. Same node/edge/map/temporal-value-or-error
/// shape as `lookup_prop_value`, minus the "unbound variable" case (there's
/// no variable name to report -- a `PropOf` base that evaluates to
/// `Value::Null` propagates `Null` here the same way a bound-but-null row
/// variable's own `.prop` access already does).
fn property_of_value(v: &Value, prop: &str) -> Result<Value, QueryError> {
    match v {
        Value::Node(n) => Ok(n
            .props
            .get(prop)
            .cloned()
            .map(property_value_to_value)
            .unwrap_or(Value::Null)),
        Value::Edge(e) => Ok(e
            .props
            .get(prop)
            .cloned()
            .map(property_value_to_value)
            .unwrap_or(Value::Null)),
        Value::Map(m) => Ok(m.get(prop).cloned().unwrap_or(Value::Null)),
        Value::Null => Ok(Value::Null),
        Value::Property(PropertyValue::Null) => Ok(Value::Null),
        Value::Property(pv) => match temporal_component(pv, prop) {
            Some(component) => Ok(Value::Property(component)),
            None if is_temporal_property_value(pv) => Ok(Value::Null),
            None => Err(QueryError::Type(
                "property access requires a node, relationship, map, or temporal value".into(),
            )),
        },
        Value::List(_) | Value::Path(_) => Err(QueryError::Type(
            "property access requires a node, relationship, map, or temporal value, not a list \
             or path"
                .into(),
        )),
        Value::Literal(_) => Err(QueryError::Type(
            "property access requires a node, relationship, map, or temporal value".into(),
        )),
    }
}
