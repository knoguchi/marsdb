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
    #[allow(dead_code)] // property-map on relationships parsed but not filtered on in v1
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
#[derive(Debug, Clone)]
pub struct QueryPart {
    pub optional: bool,
    pub pattern: Pattern,
    pub where_clause: Option<Expr>,
    pub with: Option<WithClause>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Create(Vec<Pattern>),
    Match {
        /// One or more `MATCH ... [WITH ...]` segments. The parser enforces
        /// every part except the last has a `with` (matching real Cypher's
        /// rule that multiple reading clauses must be separated by WITH)
        /// and that at most one part has a `with` at all (v1 doesn't
        /// support chaining past one WITH boundary — nothing in the target
        /// query set needs it, and it keeps a hand-rolled parser's
        /// untested-path surface smaller).
        parts: Vec<QueryPart>,
        tail: Tail,
        /// Only meaningful for `Tail::Return`; evaluated against the
        /// projected/aliased output row, not the raw pattern bindings —
        /// every ORDER BY key in practice is a RETURN alias, not a bare
        /// pattern variable.
        order_by: Option<Vec<(ReturnExpr, SortDir)>>,
        limit: Option<i64>,
    },
}
