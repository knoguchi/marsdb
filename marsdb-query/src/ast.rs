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
}

#[derive(Debug, Clone)]
pub enum ReturnExpr {
    Var(String),
    Prop(PropAccess),
    Lit(Literal),
    Call(String, Vec<ReturnExpr>),
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

#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub expr: ReturnExpr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelDirection {
    /// (a)-[..]->(b)
    Right,
    /// (a)<-[..]-(b)
    Left,
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

#[derive(Debug, Clone)]
pub enum Statement {
    Create(Vec<Pattern>),
    Match {
        pattern: Pattern,
        where_clause: Option<Expr>,
        tail: Tail,
        limit: Option<i64>,
    },
}
