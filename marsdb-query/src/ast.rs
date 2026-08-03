#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    /// `$name` placeholder — resolved to a concrete `Literal` by
    /// `params::substitute_params` before execution, never seen by the
    /// executor.
    Param(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropAccess {
    pub var: String,
    pub prop: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Compare(PropAccess, CompareOp, Literal),
    /// Does the node bound to `var` have label `label` among its (possibly
    /// multiple) labels? Synthesized by the planner for the 2nd+ label in a
    /// multi-label pattern like `(n:Post:Message)` — never user-typed.
    HasLabel(String, String),
    /// Do these two row bindings refer to the same node/edge? Synthesized
    /// by the planner when a pattern's hop variable is a "bound-node
    /// repetition" — the same variable already bound earlier reappearing
    /// mid-pattern (e.g. IS7's `p`, bound by an earlier MATCH, reappearing
    /// as the endpoint of an OPTIONAL MATCH pattern: `(a)-[r:KNOWS]-(p)`
    /// must mean "KNOWS *this* `p`", not "KNOWS anyone"). Never user-typed.
    VarEq(String, String),
}

#[derive(Debug, Clone)]
pub enum ReturnExpr {
    Var(String),
    Prop(PropAccess),
    Lit(Literal),
    Call {
        name: String,
        args: Vec<ReturnExpr>,
        distinct: bool,
    },
    /// `count(*)` — its own variant, not `Call` with a magic `"*"`-sentinel
    /// argument, so evaluation physically cannot mishandle it as an
    /// ordinary function call (no args to evaluate, no DISTINCT target —
    /// it counts rows, not values).
    CountStar,
    /// Simple/value `CASE`: `CASE <test> WHEN <value> THEN <result> ... [ELSE
    /// <else>] END`. `test` is `Some` for every form the parser currently
    /// produces; kept `Option` so a future searched-`CASE` (`CASE WHEN
    /// <bool_expr> THEN ...`) can reuse this variant without another type
    /// change.
    Case {
        test: Option<Box<ReturnExpr>>,
        whens: Vec<(ReturnExpr, ReturnExpr)>,
        else_: Option<Box<ReturnExpr>>,
    },
}

/// Case-insensitive aggregate-function recognition, shared by `parser.rs`
/// (DISTINCT-validity check) and `executor.rs` (grouping classification —
/// a RETURN/WITH item list "has an aggregate" iff any item's top-level
/// expression is `CountStar` or a `Call` whose name passes this check).
pub fn is_aggregate_name(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "count" | "sum" | "avg" | "min" | "max" | "collect")
}

#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub expr: ReturnExpr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelDirection {
    /// (a)-[..]->(b)
    Right,
    /// (a)<-[..]-(b)
    Left,
    /// (a)-[..]-(b) — matches either direction.
    Either,
}

#[derive(Debug, Clone)]
pub struct NodePattern {
    pub var: Option<String>,
    pub labels: Vec<String>,
    pub props: Vec<(String, Literal)>,
}

#[derive(Debug, Clone)]
pub struct RelPattern {
    pub var: Option<String>,
    pub rel_type: Option<String>,
    pub props: Vec<(String, Literal)>,
    pub direction: RelDirection,
    /// `[:TYPE*min..max]` — `None` means a fixed single hop (existing
    /// behavior). `max: None` means unbounded, capped at a safety depth by
    /// the executor.
    pub hop_range: Option<(u32, Option<u32>)>,
}

/// A linear chain: node, (rel, node)*.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub start: NodePattern,
    pub hops: Vec<(RelPattern, NodePattern)>,
}

#[derive(Debug, Clone)]
pub enum Tail {
    Return(Vec<ReturnItem>),
    Delete(Vec<String>),
    DetachDelete(Vec<String>),
    Set(Vec<(PropAccess, Literal)>),
    /// `MATCH ... CREATE ...` — same pattern syntax as `Statement::Create`,
    /// but runs once per row already bound by the preceding MATCH/WITH: a
    /// node pattern token whose variable is already bound in that row
    /// reuses the existing node instead of creating a new one. This is
    /// the only way to add an edge between two nodes that already exist —
    /// `Statement::Create` alone can't (every node token it sees is
    /// always fresh).
    Create(Vec<Pattern>),
}

/// WITH's HAVING-equivalent: filters on the already-projected/aggregated
/// row (e.g. `WITH p, count(f) AS c WHERE c > 10`). Same And/Or/Not/
/// Compare shape as `Expr`, but the comparison's LHS is a `ReturnExpr`
/// (a WITH alias or raw expression) instead of a raw-property
/// `PropAccess` — deliberately a separate type from `Expr` rather than a
/// widened reuse of it, since `Expr::Compare` is what the planner pushes
/// into pre-projection `Filter`/`Expand` nodes, and this filter
/// fundamentally belongs *post*-projection instead (see `materialize_with`).
#[derive(Debug, Clone)]
pub enum WithExpr {
    And(Box<WithExpr>, Box<WithExpr>),
    Or(Box<WithExpr>, Box<WithExpr>),
    Not(Box<WithExpr>),
    Compare(ReturnExpr, CompareOp, Literal),
}

/// A `WITH` clause: projects/renames the current bindings, optionally
/// filtered/sorted/limited at that boundary, and becomes the binding scope
/// for whatever follows (the next `QueryPart`, or the final `Tail`).
#[derive(Debug, Clone)]
pub struct WithClause {
    pub items: Vec<ReturnItem>,
    pub where_clause: Option<WithExpr>,
    pub order_by: Option<Vec<(ReturnExpr, SortDir)>>,
    pub limit: Option<i64>,
}

/// One `MATCH <pattern>[, <pattern>...] [WHERE ...] [WITH ...]` segment.
/// Comma-separated patterns within one part are spliced into a single
/// linear `Pattern` at parse time (see `parser::splice_patterns`) — this
/// only ever holds one already-combined `Pattern`, not several.
///
/// `path_var` is `Some` for `p = (a)-->(b)` / `p = shortestPath(...)` —
/// capturing the whole matched path, not just its endpoints. General
/// named-path capture (`shortest_path: false`) is limited to fixed-hop
/// patterns — `pattern` must contain no variable-length (`*`) hop, parser-
/// enforced, since reconstructing a path over `VarExpand`'s BFS would need
/// the same parent-pointer tracking `shortestPath()` already has, but
/// generalized, which isn't worth it for the narrow payoff. `shortest_path
/// : true` is the opposite: `pattern` must be exactly one variable-length
/// hop (`shortestPath((a)-[:TYPE*..N]-(b))`), and both endpoints must
/// already be bound by a preceding clause (see `executor::eval_shortest_
/// path`'s docs for why unbound endpoints aren't supported in v1).
#[derive(Debug, Clone)]
pub struct QueryPart {
    pub optional: bool,
    pub path_var: Option<String>,
    pub shortest_path: bool,
    pub pattern: Pattern,
    pub where_clause: Option<Expr>,
    pub with: Option<WithClause>,
}

/// `UNWIND <source> AS <var> [WHERE ...] [WITH ...]` — fans a list out into
/// one row per element, cross-joined against whatever rows already exist
/// (same "row-vector-in, row-vector-out, no graph traversal" shape as a
/// `WithClause`, not a graph-traversal `LogicalPlan` node — see
/// `executor::eval_unwind`). Its own `where_clause` (rather than requiring
/// a `WITH` right after it just to filter) is what makes `UNWIND [1,2,3]
/// AS x WHERE x > 2` — or `WITH ... collect(m) AS ms UNWIND ms AS m2
/// WHERE m2.x > 1` — work within the one-`WITH`-per-statement cap (see
/// `QueryClause`'s docs). Deliberately typed as `WithExpr`, not the
/// pattern-level `Expr`: an unwound variable is very often a bare scalar
/// (`x > 2`), which `Expr::Compare`'s always-`PropAccess` LHS structurally
/// cannot express (only `x.prop > 2` is) — `WithExpr::Compare`'s
/// `ReturnExpr` LHS covers both.
#[derive(Debug, Clone)]
pub struct UnwindClause {
    pub source: UnwindSource,
    pub var: String,
    pub where_clause: Option<WithExpr>,
    pub with: Option<WithClause>,
}

/// Where an `UNWIND`'s list comes from. `Var` restores graph identity per
/// element when the list came from `collect()`-ing nodes/edges (see
/// `executor::value_to_binding_restore`) — there's no `PropertyValue::List`
/// yet, so a `$param`-supplied list isn't reachable here; only a
/// previously-bound `collect()` result or an inline Cypher-text list.
#[derive(Debug, Clone)]
pub enum UnwindSource {
    Var(String),
    List(Vec<Literal>),
}

/// `MERGE <pattern> [ON CREATE SET ...] [ON MATCH SET ...] [WITH ...]` —
/// match-or-create: try the pattern as an ordinary MATCH first (reusing
/// `build_match_plan`/`eval_plan` — this already does the right "search
/// the *connected* sub-pattern, not each node in isolation" thing for a
/// hop pattern, since `Expand` only follows real edges and `Filter` only
/// keeps matches against the target's own constraints); if that finds
/// nothing, create exactly one new pattern instance (reusing
/// `resolve_or_create_node`, the same "reuse if the token's var is
/// already bound in the row" logic `Tail::Create` uses). `pattern.hops`
/// is capped at one relationship by the parser — whole-pattern atomicity
/// across multiple simultaneously-unbound hops isn't attempted in v1, see
/// `executor::eval_merge`'s docs for why.
#[derive(Debug, Clone)]
pub struct MergeClause {
    pub pattern: Pattern,
    pub on_create: Vec<(PropAccess, Literal)>,
    pub on_match: Vec<(PropAccess, Literal)>,
    pub with: Option<WithClause>,
}

/// One reading clause in a `MATCH`/`UNWIND`/`MERGE` sequence. `Match` is
/// today's `MATCH`/`OPTIONAL MATCH ... [WHERE] [WITH]` segment; `Unwind`
/// fans out a list; `Merge` matches-or-creates. All three can optionally
/// end in a `WITH` — see `Statement::Match`'s docs for the WITH-
/// separation/one-WITH-total rules this enum's variants are validated
/// against.
#[derive(Debug, Clone)]
pub enum QueryClause {
    Match(QueryPart),
    Unwind(UnwindClause),
    Merge(MergeClause),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Create(Vec<Pattern>),
    Match {
        /// One or more `MATCH`/`UNWIND`/`MERGE ... [WITH ...]` clauses. The
        /// parser enforces every `Match` clause except the statement's
        /// last has a `with` before the next `Match` clause (matching real
        /// Cypher's rule that multiple reading clauses must be separated
        /// by WITH) — `Unwind`/`Merge` clauses are exempt from this
        /// specific check (they share one binding scope the same way
        /// `OPTIONAL MATCH` already does, real Cypher needs no WITH around
        /// a bare UNWIND/MERGE either) — and that at most one clause (of
        /// any kind) has a `with` at all across the whole statement (v1
        /// doesn't support chaining past one WITH boundary — nothing in
        /// the target query set needs it, and it keeps a hand-rolled
        /// parser's untested-path surface smaller).
        clauses: Vec<QueryClause>,
        /// `None` only when a `MERGE` clause is present with nothing after
        /// it (`MERGE (n:Label)` alone, no `RETURN`/etc — a pure write,
        /// same as standalone `CREATE`). The parser rejects a missing tail
        /// otherwise (`MATCH (n)` alone is almost certainly a mistake, not
        /// a deliberate no-op).
        tail: Option<Tail>,
        /// Only meaningful for `Tail::Return`; evaluated against the
        /// projected/aliased output row, not the raw pattern bindings —
        /// every ORDER BY key in practice is a RETURN alias, not a bare
        /// pattern variable.
        order_by: Option<Vec<(ReturnExpr, SortDir)>>,
        limit: Option<i64>,
    },
}
