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
    /// String-only predicates (`a.name STARTS WITH 'x'`, etc.) — anything
    /// but a `String`/`String` operand pair compares `false`, same as every
    /// other type-mismatched `CompareOp` already does in `compare()`.
    StartsWith,
    EndsWith,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Compare(PropAccess, CompareOp, Literal),
    /// `a.id = b.id` / `x.val < y.val` — a property compared against
    /// *another* property, not a constant. Never eligible for the
    /// planner's index-seek fusion (that only matches `Compare`'s
    /// literal-RHS shape), always evaluated as a generic post-scan filter.
    PropCompare(PropAccess, CompareOp, PropAccess),
    /// `n.prop IS NULL` — unlike `Compare`, this is always a definite
    /// `true`/`false`, never "unknown" (that's the whole point of the
    /// check). `IS NOT NULL` parses to `Not(IsNull(..))`, reusing the
    /// existing `Not` variant rather than a fourth boolean-op variant.
    IsNull(PropAccess),
    /// Does the node bound to `var` have label `label` among its (possibly
    /// multiple) labels? Synthesized by the planner for the 2nd+ label in a
    /// multi-label pattern like `(n:Post:Message)`, *and* user-typed
    /// directly in a `WHERE` (`WHERE a:A`, `WHERE a:A:B` desugars to an
    /// `And` chain of one `HasLabel` per label).
    HasLabel(String, String),
    /// Do these two row bindings refer to the same node/edge? Synthesized
    /// by the planner when a pattern's hop variable is a "bound-node
    /// repetition" — the same variable already bound earlier reappearing
    /// mid-pattern (e.g. IS7's `p`, bound by an earlier MATCH, reappearing
    /// as the endpoint of an OPTIONAL MATCH pattern: `(a)-[r:KNOWS]-(p)`
    /// must mean "KNOWS *this* `p`", not "KNOWS anyone"). Also user-typed
    /// directly in a `WHERE` (`WHERE a = b`; `WHERE a <> b` desugars to
    /// `Not(VarEq(a, b))`) — real Cypher's node/relationship identity
    /// comparison, distinct from comparing two of their *properties*
    /// (`PropCompare`) or two arbitrary values (`WithExpr::Compare`,
    /// post-projection only).
    VarEq(String, String),
    /// `WHERE toInteger(n.id) = 1`, `WHERE r.weight * 2 > n.threshold`,
    /// ... -- any comparison whose operand isn't the narrower
    /// `prop_access`/`literal` shape `Compare`/`PropCompare` cover
    /// (a function call, arithmetic, a bare variable compared to
    /// something, ...). Same operand type `WithExpr::Compare` uses
    /// (`ReturnExpr`, built from the shared `add_expr` grammar rule), but
    /// this variant keeps pattern-level `Expr`'s own pre-projection
    /// evaluation context (`Executor::eval_expr`, against the raw
    /// `BindingRow`, not a post-projection value map) -- never eligible
    /// for the planner's index-seek fusion, always a generic post-scan
    /// filter, same as `PropCompare`.
    GeneralCompare(ReturnExpr, CompareOp, ReturnExpr),
    /// `WHERE r IS NULL` (a whole bound variable, e.g. checking an
    /// `OPTIONAL MATCH` miss) or `WHERE toInteger(n.id) IS NULL` -- unlike
    /// `IsNull`, the operand isn't restricted to a bare `prop_access`.
    /// Mirrors `WithExpr::IsNull` exactly, just evaluated pre-projection.
    GeneralIsNull(ReturnExpr),
    /// A boolean-valued expression used directly as a predicate with no
    /// comparison operator at all -- `WHERE single(x IN list WHERE x = 2)
    /// OR all(x IN list WHERE x = 2)`, `WHERE n.flag`, `WHERE NOT
    /// exists(n.prop)`. Three-valued (`Null` is "unknown", same as every
    /// other `Expr` leaf), evaluated via `value_to_bool3`. Mirrors
    /// `WithExpr::Bare` exactly, just evaluated pre-projection.
    GeneralBare(ReturnExpr),
    /// `WHERE (n)-[:REL]->(m)` etc (TCK's Pattern1 "Pattern predicate") --
    /// existential: true iff at least one real match of `Pattern` exists
    /// against the graph, with every named endpoint already bound in the
    /// current row held fixed to that binding rather than searched freely
    /// (a pattern predicate never introduces a new variable -- real
    /// Cypher's `UndefinedVariable`, checked at compile time by
    /// `semantic::validate_pattern_predicate`). Evaluated by
    /// `Executor::eval_expr` via the same `build_match_plan` "already-
    /// bound var -> Seed" mechanism `eval_merge`'s own "try as an
    /// ordinary MATCH first" half already uses.
    Pattern(Pattern),
    /// `WHERE exists { (n)-->(m) WHERE n.prop = m.prop }` (TCK's
    /// ExistentialSubquery1, the "simple" form) -- unlike `Pattern`
    /// above, `pattern` here *can* introduce brand-new variables (`m`
    /// above; real Cypher allows that inside an `exists {}` block, unlike
    /// a bare pattern predicate), and carries its own inline `where?`
    /// clause (the grammar's `patternWhere` rule, shared with `MATCH`'s
    /// own pattern -- `where_clause` is threaded into `build_match_plan`
    /// directly, same as an ordinary `MATCH ... WHERE ...`, rather than
    /// evaluated as a separate post-filter step). Existence check only
    /// (`Some(1)`-limited, same as `Pattern`) -- see `ExistsSubquery` below
    /// for the "full" `exists { MATCH ... RETURN ... }` subquery form.
    Exists {
        pattern: Box<Pattern>,
        where_clause: Option<Box<Expr>>,
    },
    /// `WHERE exists { MATCH ... RETURN ... }` (TCK's
    /// ExistentialSubquery2/3) -- runs an arbitrary read-only
    /// `Statement::Match` correlated against the current row (its already-
    /// bound variables seed the nested statement's own scope, same
    /// already-bound-var -> Seed mechanism `Exists`/`Pattern` above use),
    /// true iff it produces at least one output row. Unlike `Exists`
    /// above, the nested statement can carry its own aggregation/multiple
    /// clauses/nested `exists {}` -- `semantic::validate_statement`
    /// rejects any mutating clause inside it at compile time (real
    /// Cypher's `InvalidClauseComposition`, TCK's ExistentialSubquery2
    /// `[3]`).
    ExistsSubquery(Box<Statement>),
    /// Real Cypher's edge-isomorphism rule (`VarEq`'s own docs), extended
    /// to a variable-length hop: `edge_var`'s bound edge must not be among
    /// the edges an *earlier* variable-length hop in the same pattern
    /// already traversed for this row (`edge_set_var`, a `Binding::Path`
    /// segment `planner::build_match_plan` always threads through a
    /// `VarExpand` for exactly this purpose -- see `LogicalPlan::
    /// VarExpand::exclude_edge_var`'s own docs). Unlike `VarEq` (one edge
    /// vs one edge), this checks one edge against a *set* -- each row can
    /// carry a different set, one per distinct earlier traversal.
    /// Synthesized by the planner only; no surface syntax constructs this
    /// directly.
    EdgeNotInSet {
        edge_var: String,
        edge_set_var: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnExpr {
    Var(String),
    Prop(PropAccess),
    /// `<expr>.prop` where `<expr>` is anything other than a bare variable
    /// (`startNode(r).id`, `head(nodes(p)).name`, `{a: 1}.a`) — `Prop`
    /// only covers the flat `var.prop` case (matching pest's own
    /// `prop_access` rule, a dedicated `identifier DOT identifier`
    /// production). TCK's Graph6 [4]/[8], Map1 [3], Merge5 [11].
    PropOf(Box<ReturnExpr>, String),
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
    /// `CASE <test> WHEN <value> THEN <result> ... [ELSE <else>] END`
    /// (simple form, `test: Some`) or `CASE WHEN <bool_expr> THEN <result>
    /// ... [ELSE <else>] END` (searched form, `test: None` -- each `WHEN`
    /// carries its own full condition instead of a value compared against
    /// `test`).
    Case {
        test: Option<Box<ReturnExpr>>,
        whens: Vec<(ReturnExpr, ReturnExpr)>,
        else_: Option<Box<ReturnExpr>>,
    },
    /// `lhs op rhs` — `+ - * / %`, real precedence (`*`/`/`/`%` bind
    /// tighter than `+`/`-`), usable anywhere a `ReturnExpr` is (RETURN/
    /// WITH items, `CASE` branches, function args, `ORDER BY` keys).
    /// Deliberately not threaded into pattern-level `WHERE` (`Expr`) or
    /// `WITH ... WHERE` (`WithExpr`)'s comparison operands in this pass --
    /// both currently take a bare `PropAccess`/`Literal` on each side, and
    /// widening that is a separate, larger change to the planner's
    /// pre-projection `Filter` pushdown.
    Arith(Box<ReturnExpr>, ArithOp, Box<ReturnExpr>),
    /// `-x` — general unary negation (`-n.prop`, `-(1 + 2)`, `-f()`, ...),
    /// distinct from a negative numeric *literal* (`-3` is still just
    /// `Lit(Int(-3))`, parsed directly by `int_literal`/`float_literal`'s
    /// own optional leading `-` — see `cypher.pest`'s `unary_minus_expr`
    /// docs for why that path is deliberately left untouched, rather than
    /// this variant subsuming it, to avoid losing the planner's index-seek
    /// fusion for `MATCH (n {x: -5})`-shaped literal patterns). Binds
    /// tighter than every other arithmetic operator, including `^`.
    Neg(Box<ReturnExpr>),
    /// `[a, b, c]` — a general expression list, not `UnwindSource::List`'s
    /// literal-only cousin (that one's deliberately scoped to right after
    /// `UNWIND`; this one is a real `ReturnExpr`, usable anywhere one is).
    ListLit(Vec<ReturnExpr>),
    /// `list[index]` — a negative index counts from the end
    /// (`list[-1]` is the last element); out of bounds either way is
    /// `Null`, not an error (matches real Cypher).
    Index(Box<ReturnExpr>, Box<ReturnExpr>),
    /// `list[start..end]` — either bound omitted means "from/to the edge
    /// of the list". Same negative-counts-from-end rule as `Index`, but
    /// out-of-range bounds clamp instead of nulling out, and a start at or
    /// past the (clamped) end yields `[]` rather than erroring.
    Slice(
        Box<ReturnExpr>,
        Option<Box<ReturnExpr>>,
        Option<Box<ReturnExpr>>,
    ),
    /// `[x IN <source> WHERE <cond> | <project>]` — `WHERE`/`| project` are
    /// each independently optional (`[x IN list]` is a legal no-op
    /// identity-filter comprehension). `where_clause` is a `ReturnExpr`
    /// (not pattern-level `Expr`) for the same reason `UnwindClause`'s own
    /// filter used to reuse `WithExpr` — `var` is very often a bare
    /// scalar/node/edge, not something `Expr::Compare`'s
    /// `prop_access`-only LHS can express. Now that boolean logic/
    /// comparisons are real `ReturnExpr` variants (`And`/`Or`/`Not`/
    /// `Compare`), this is the wider type `WithExpr` used to be, letting a
    /// bare `WHERE x`/`WHERE true` parse (previously rejected — `WithExpr`
    /// only ever wrapped a `Compare`, never a standalone boolean value).
    ListComp {
        var: String,
        source: Box<ReturnExpr>,
        where_clause: Option<Box<ReturnExpr>>,
        project: Option<Box<ReturnExpr>>,
    },
    /// `ALL(x IN list WHERE cond)` / `ANY(...)` / `NONE(...)` / `SINGLE(...)`
    /// — shares `ListComp`'s "one bound variable over a list, optionally
    /// filtered" shape (no `project` half; a quantifier always yields a
    /// `Bool`, never a projected list). `where_clause` absent means "every
    /// element's own truthiness", same convention `CASE`'s subject-less
    /// `WHEN` branch already uses (`matches!(v, Literal(Bool(true)))`).
    Quantifier {
        kind: QuantifierKind,
        var: String,
        source: Box<ReturnExpr>,
        where_clause: Option<Box<ReturnExpr>>,
    },
    /// `{a: 1, b: 2 + 1}` — a general expression map. `NodePattern`/
    /// `RelPattern`'s own `props` reuse this same `ReturnExpr` value type
    /// (not a separate `Literal`-only map) for the identical `{...}`
    /// pattern syntax — a `CREATE`/`MERGE` prop value can be any
    /// expression too (`{date: date({year: 1984, ...})}`), evaluated
    /// against the row already bound so far (`Executor::
    /// eval_props_to_values`). `MATCH`/`MERGE`'s own inline pattern props
    /// specifically are further restricted back down to plain literals at
    /// plan-build time (`planner::require_literal_pattern_prop`) — a
    /// computed value there doesn't make sense before any row exists to
    /// evaluate it against, matching real Cypher's own restriction.
    MapLit(Vec<(String, ReturnExpr)>),
    /// `lhs AND/OR/XOR rhs`, `NOT rhs` — real three-valued logic (`Null`
    /// propagates per Cypher's truth tables, see `and3`/`or3`/`xor3` in
    /// executor.rs), evaluating to `Value::Literal(Bool(_))` or
    /// `Value::Null`, not `Option<bool>` the way pattern-level `Expr`/
    /// `WithExpr` do — a `ReturnExpr` always evaluates to one `Value`,
    /// there's no separate "unbound" state to fold in beyond `Null`
    /// itself. A non-bool, non-null operand is a real error (`1 AND
    /// true`), not silently coerced.
    And(Box<ReturnExpr>, Box<ReturnExpr>),
    Or(Box<ReturnExpr>, Box<ReturnExpr>),
    Xor(Box<ReturnExpr>, Box<ReturnExpr>),
    Not(Box<ReturnExpr>),
    /// `lhs op rhs` — a single comparison between two arbitrary
    /// expressions (both operands can be a variable/property/arithmetic
    /// expression, same as `WithExpr::Compare`'s two `ReturnExpr`
    /// operands). A chain (`1 < x < 10`) parses into nested `And`s of
    /// each adjacent pair (`(1 < x) AND (x < 10)`), same as real Cypher's
    /// own chained-comparison semantics — not a separate AST shape.
    /// `WithExpr::Compare` doesn't chain this way (its own grammar level
    /// doesn't recurse into itself), just a single comparison per node.
    Compare(Box<ReturnExpr>, CompareOp, Box<ReturnExpr>),
    /// `x IS NULL` — always a definite `true`/`false`, never "unknown"
    /// (that's the whole point of the check). `IS NOT NULL` parses to
    /// `Not(IsNull(..))`, reusing the existing `Not` variant.
    IsNull(Box<ReturnExpr>),
    /// `x IN list` — real Cypher's list membership test, three-valued
    /// like `=` (`null IN [1]` and `1 IN [null]` are both "unknown", not
    /// `false`): a definite element match wins outright even past a later
    /// `null` element, no match with at least one `null` element compared
    /// is "unknown", no match and no `null` anywhere is a definite
    /// `false`. Binds *tighter* than a surrounding comparison, same
    /// precedence tier as `IsNull` (`a = b IN list` is `a = (b IN
    /// list)`) — see `compare_expr`'s grammar comment.
    In(Box<ReturnExpr>, Box<ReturnExpr>),
    /// `(n:Foo)`/`(n:Foo:Bar)` used as a boolean expression — `true` iff
    /// the bound node has every listed label, `false` otherwise (a
    /// definite bool, not three-valued — same as `Expr::HasLabel`, the
    /// pattern-position sibling this mirrors, but reachable directly from
    /// `RETURN`/`WITH`/`WHERE` instead of only ever being synthesized by
    /// the planner for a multi-label pattern token).
    HasLabel(String, Vec<String>),
    /// `(n)-[]->()` etc used directly as a boolean expression -- existential
    /// pattern-predicate syntax, same shape as `Expr::Pattern` (the
    /// pattern-level sibling this mirrors), but reachable from generic
    /// expression position (parsed via the same `atom` alternative
    /// anything else is). In practice only ever meaningful inside `WHERE`
    /// -- `return_expr_to_expr` folds it into `Expr::Pattern` there before
    /// it ever reaches `RETURN`/`WITH` position or the executor. Reaching
    /// `RETURN (n)-->()`/a RETURN item/property value/etc directly (no
    /// `Expr`-folding step in between) is a real error, not evaluated --
    /// existence can't be checked without a bound row to check it against,
    /// which only `WHERE`'s evaluation context has.
    PatternPredicate(Pattern),
    /// `[p = (n)-->() | p]` / `[(n)-[:T]->(b) | b.name]` -- unlike
    /// `PatternPredicate` (existence-only, folded into `Expr::Pattern`
    /// before it ever reaches the executor), this enumerates *every*
    /// match of `pattern` (each held-fixed against already-bound named
    /// endpoints, same as `Expr::Pattern`/`PatternPredicate`) and projects
    /// `projection` against each match's own bindings -- which include
    /// any *new* node/relationship variables the pattern introduces
    /// (e.g. `b`/`r` above; real Cypher allows that here, unlike a
    /// pattern predicate) -- collecting the results into a `Value::List`.
    /// `where_clause` is pattern-level `Expr` (not `ReturnExpr`), reusing
    /// the exact same evaluation `Executor::eval_expr`'s `Expr::Pattern`
    /// arm already does for a pattern's own `WHERE` -- the grammar's
    /// `patternComprehension` rule shares its `where?` production with
    /// ordinary `MATCH`, not with `ListComp`'s post-projection
    /// `WithExpr`-shaped filter.
    PatternComprehension {
        path_var: Option<String>,
        pattern: Box<Pattern>,
        where_clause: Option<Box<Expr>>,
        projection: Box<ReturnExpr>,
    },
    /// `exists { (n)-->(m) WHERE ... }` reached from general expression
    /// position -- same "only meaningful inside WHERE" story as
    /// `PatternPredicate` (`return_expr_to_expr` folds it into
    /// `Expr::Exists` there before it ever reaches the executor;
    /// reaching `eval_return_expr` directly is a real error).
    ExistsPattern {
        pattern: Box<Pattern>,
        where_clause: Option<Box<Expr>>,
    },
    /// `exists { MATCH ... RETURN ... }` reached from general expression
    /// position -- the "full" nested-subquery form of `exists {}`
    /// (`ExistsPattern`'s complement, TCK's ExistentialSubquery2/3), an
    /// arbitrary read-only `Statement::Match` (any number of MATCH/UNWIND/
    /// WITH clauses, its own aggregation/WHERE, ending in a RETURN) run
    /// correlated against whatever's already bound in the enclosing row --
    /// same "only meaningful inside WHERE" story as `ExistsPattern`
    /// (`return_expr_to_expr` folds it into `Expr::ExistsSubquery` there
    /// before it ever reaches the executor; reaching `eval_return_expr`
    /// directly is a real error). The inner statement's own RETURN items
    /// are never actually projected out -- only whether it produces at
    /// least one row matters, same as `ExistsPattern`.
    ExistsSubquery(Box<Statement>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantifierKind {
    All,
    Any,
    None,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    /// `a ^ b` -- always produces a `Float`, even for two `Int` operands
    /// (real Cypher's own rule; unlike every other `ArithOp`, there's no
    /// Int/Int-stays-Int case). Right-associative and binds tighter than
    /// `*`/`/`/`%` but looser than unary minus (`-3 ^ 2` is `(-3) ^ 2`,
    /// not `-(3 ^ 2)`) -- see `cypher.pest`'s `pow_expr`/`unary_minus_expr`.
    Pow,
}

/// Case-insensitive aggregate-function recognition, shared by `parser.rs`
/// (DISTINCT-validity check) and `executor.rs` (grouping classification —
/// a RETURN/WITH item list "has an aggregate" iff any item's top-level
/// expression is `CountStar` or a `Call` whose name passes this check).
pub fn is_aggregate_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "count" | "sum" | "avg" | "min" | "max" | "collect" | "percentilecont" | "percentiledisc"
    )
}

/// `percentileCont`/`percentileDisc` are the only aggregates that take a
/// second argument (the percentile, `0.0..=1.0`) alongside the value being
/// aggregated -- every other name in `is_aggregate_name` takes exactly one.
pub fn is_percentile_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "percentilecont" | "percentiledisc"
    )
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub var: Option<String>,
    pub labels: Vec<String>,
    /// `ReturnExpr`, not `Literal` — a CREATE prop value can be any
    /// expression (`{date: date({year: 1984, ...})}`, `{x: 1 + 2}`), not
    /// just a literal; see `cypher.pest`'s `map_expr` docs and
    /// `Executor::eval_props_to_values`, which evaluates each one against
    /// the row already bound so far in the same CREATE.
    pub props: Vec<(String, ReturnExpr)>,
    /// Whether an inline `{...}` map token was actually written, even an
    /// empty one (`(n {})`) -- `props` alone can't distinguish that from
    /// no map token at all (`(n)`), both giving an empty `Vec`, but real
    /// Cypher's `VariableAlreadyBound` check cares about the distinction:
    /// `MATCH (n) CREATE (n {})` is still "imposing a new predicate" on
    /// an already-bound node even though the map is empty (TCK's Create1
    /// `[19]`), while `MATCH (n) CREATE (n)-->()` (no map token at all)
    /// is fine.
    pub has_explicit_props: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelPattern {
    pub var: Option<String>,
    /// `[:A]` -- one element; `[:A|B]`/`[:A|:B]` (real Cypher accepts
    /// either separator form) -- more than one, matched if the edge's
    /// type is ANY of them (TCK's Match2 [6]/Match3 [8], Pattern1 [13]).
    /// Empty means untyped (`[]`/`[r]`, any type matches).
    pub rel_types: Vec<String>,
    pub props: Vec<(String, ReturnExpr)>,
    pub direction: RelDirection,
    /// `[:TYPE*min..max]` — `None` means a fixed single hop (existing
    /// behavior). `max: None` means unbounded, capped at a safety depth by
    /// the executor.
    pub hop_range: Option<(u32, Option<u32>)>,
    /// Set only by `executor::name_pattern_for_path` on a variable-length
    /// hop, when assembling a named-path capture (`p = (a)-[*1..3]->(b)`,
    /// TCK's Quantifier1-4 `[8]`/`[9]`) -- asks the planner's `VarExpand`
    /// to also expose its own internally-traversed edge/node sequence (in
    /// order) for `executor::assemble_path` to splice into the whole
    /// pattern's path, via a fresh internal binding name that `var` gets
    /// overwritten to hold (see `name_pattern_for_path`'s own docs). The
    /// user's own real relationship-list variable, if this hop had one
    /// (`p = (a)-[r*1..3]->(b)`, TCK's Match9 `[9]`), is preserved
    /// separately in `rel_list_var` below rather than lost to that
    /// overwrite.
    pub capture_path_segment: bool,
    /// The user's own `[r:TYPE*1..3]` relationship-list variable name, for
    /// a hop that has `capture_path_segment` set -- `None` for an
    /// anonymous hop (`p = (a)-[*1..3]->(b)`, nothing to bind). For a
    /// var-length hop *without* named-path capture, `var` itself already
    /// holds this directly (this field stays `None` in that case; the
    /// planner reads whichever of the two applies). Two separate fields
    /// rather than reusing `var` for both, since `capture_path_segment`
    /// already needs `var` for its own internal bookkeeping name.
    pub rel_list_var: Option<String>,
}

/// A linear chain: node, (rel, node)*.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub start: NodePattern,
    pub hops: Vec<(RelPattern, NodePattern)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tail {
    /// `distinct`: `RETURN DISTINCT ...` -- a result-set-level dedup of the
    /// whole projected row, applied after projection (and after grouping,
    /// for an aggregating RETURN) -- not the same knob as `DISTINCT` inside
    /// an aggregate call (`count(DISTINCT x)`), which only affects that one
    /// aggregate's own accumulation.
    Return(Vec<ReturnItem>, bool),
    /// `RETURN *` (`distinct`: `RETURN DISTINCT *`) -- every currently-
    /// bound variable, alphabetically. Can't be resolved into a concrete
    /// `Return(Vec<ReturnItem>, _)` at parse time (no scope exists yet);
    /// resolved independently wherever the real bound-variable-name set
    /// is already on hand (`execute_match`'s own `carried_vars` in
    /// `executor.rs`, `Scope`'s keys in `semantic.rs`) rather than via a
    /// separate AST-mutation pass, avoiding a `&mut Statement` ripple
    /// through `Executor::execute`'s public signature. `MATCH ()
    /// RETURN *` (nothing bound at all) is a compile-time
    /// `NoVariablesInScope` error, not an empty projection.
    ReturnStar(bool),
    /// Every mutating tail variant's trailing `Option<ReturnTail>` is real
    /// Cypher: `MATCH (n) SET n.x = 1 RETURN n`, `MATCH (n) DELETE n RETURN
    /// count(n)`, etc — see `ReturnTail`'s docs for why it's `None` (the
    /// pre-existing terminal-mutation shape) vs `Some` (this statement's
    /// final clause is actually this RETURN, projected off whatever the
    /// mutation left in scope).
    /// Each target is any expression, not just a bare variable — real
    /// Cypher allows `DELETE list[0]`/`DELETE map.key`/`DELETE aPath`
    /// (deletes every node/edge in the path). Evaluated per row
    /// (`executor::materialize_delete`); `Value::Null` is a documented
    /// no-op, anything that isn't a node/relationship/path is a real
    /// `QueryError::Type`.
    Delete(Vec<ReturnExpr>, Option<ReturnTail>),
    DetachDelete(Vec<ReturnExpr>, Option<ReturnTail>),
    Set(Vec<SetItem>, Option<ReturnTail>),
    Remove(Vec<RemoveItem>, Option<ReturnTail>),
    /// `MATCH ... CREATE ...` — same pattern syntax as `Statement::Create`,
    /// but runs once per row already bound by the preceding MATCH/WITH: a
    /// node pattern token whose variable is already bound in that row
    /// reuses the existing node instead of creating a new one. This is
    /// the only way to add an edge between two nodes that already exist —
    /// `Statement::Create` alone can't (every node token it sees is
    /// always fresh).
    Create(Vec<Pattern>, Option<ReturnTail>),
}

/// A `RETURN` trailing a mutating `Tail` (`SET`/`DELETE`/`DETACH DELETE`/
/// `REMOVE`/`MATCH ... CREATE`) in the same statement, e.g. `MATCH (n) SET
/// n.prop = 1 RETURN n`. Same two fields `Tail::Return` itself carries
/// (`items`, `distinct`) — wrapped in its own type so every mutating `Tail`
/// variant can carry one `Option<ReturnTail>` instead of repeating a
/// `(Vec<ReturnItem>, bool)` tuple five times. Arbitrary multi-clause
/// chaining (`SET ... DELETE ... RETURN`, a mutating clause followed by
/// `WITH` before the final `RETURN`) isn't supported yet — this only covers
/// exactly one mutating clause directly followed by exactly one `RETURN`,
/// which is the shape the real TCK scenarios for SET/DELETE/REMOVE
/// overwhelmingly use.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnTail {
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetItem {
    /// `SET n.prop = <expr>` — the value is any `ReturnExpr` (arithmetic,
    /// a property read, a function call, ...), not just a literal, same
    /// as `CREATE`'s inline `{...}` prop values already are. Evaluating
    /// to `Value::Null` removes the property (matches real Cypher; see
    /// `executor::apply_set_item`'s docs), not storing a literal null.
    Prop(PropAccess, ReturnExpr),
    /// `SET n:A:B` — adds each label to the node's label set (idempotent,
    /// not an error if already present).
    Labels(String, Vec<String>),
    /// `SET n = {...}` (`merge: false`, replaces every existing property)
    /// or `SET n += {...}` (`merge: true`, only overrides/adds/removes
    /// -- a `null` value -- the map's own listed keys, everything else on
    /// `n` stays as-is). `value` must evaluate to a `Value::Map` — TCK's
    /// Set4/Set5.
    MapAssign {
        var: String,
        value: ReturnExpr,
        merge: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveItem {
    Prop(PropAccess),
    /// `REMOVE n:A:B` — removes each label from the node's label set (not
    /// an error if it wasn't there).
    Labels(String, Vec<String>),
}

/// WITH's HAVING-equivalent: filters on the already-projected/aggregated
/// row (e.g. `WITH p, count(f) AS c WHERE c > 10`, or `WITH a, b WHERE
/// a = b`). Same And/Or/Not/Compare shape as `Expr`, but both comparison
/// operands are a `ReturnExpr` (a WITH alias or raw expression, so either
/// side can be a bound variable/property, not just the LHS against a
/// constant) instead of a raw-property `PropAccess`/`Literal` pair —
/// deliberately a separate type from `Expr` rather than a widened reuse
/// of it, since `Expr::Compare` is what the planner pushes into
/// pre-projection `Filter`/`Expand` nodes, and this filter fundamentally
/// belongs *post*-projection instead (see `materialize_with`).
#[derive(Debug, Clone, PartialEq)]
pub enum WithExpr {
    And(Box<WithExpr>, Box<WithExpr>),
    Or(Box<WithExpr>, Box<WithExpr>),
    Not(Box<WithExpr>),
    Compare(ReturnExpr, CompareOp, ReturnExpr),
    /// `x IS NULL` -- `x` is any `add_expr`, not just a property access
    /// (`WHERE r IS NULL`, checking an OPTIONAL MATCH miss on a whole
    /// bound var). `IS NOT NULL` parses to `Not(IsNull(..))`, same
    /// convention as pattern-level `Expr::IsNull`.
    IsNull(ReturnExpr),
    /// A boolean-valued expression used directly as a predicate with no
    /// comparison operator at all -- `WHERE single(x IN list WHERE x = 2)
    /// OR all(x IN list WHERE x = 2)`, `WHERE n.flag`, `WHERE
    /// exists(n.prop)`. Three-valued (`Null` is "unknown"), evaluated via
    /// `value_to_bool3` (real Cypher: a non-boolean value here is a type
    /// error, not silently coerced).
    Bare(ReturnExpr),
}

/// A `WITH` clause: projects/renames the current bindings, optionally
/// filtered/sorted/limited at that boundary, and becomes the binding scope
/// for whatever follows (the next `QueryPart`, or the final `Tail`).
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub items: Vec<ReturnItem>,
    /// `WITH *` (optionally followed by more items, `WITH *, x AS y`) --
    /// every currently-bound variable, alphabetically, same convention
    /// `Tail::ReturnStar`'s own `return_star_items` already established
    /// for `RETURN *`. Can't be resolved into concrete `items` at parse
    /// time (no scope exists yet) -- resolved independently wherever the
    /// real bound-variable-name set entering this WITH is already on
    /// hand (`executor::apply_with_or_carry`'s own `carried_vars`,
    /// `semantic::project_with`'s `input: &Scope`, `explain.rs`'s own
    /// `carried_vars`), mirroring `ReturnStar`'s "resolve at each call
    /// site" approach rather than a separate whole-AST-mutation pass.
    /// When both `star` and `items` are present, star-expanded names
    /// come first (real Cypher has no TCK-tested requirement either way
    /// for this combination, but this ordering is the common convention).
    pub star: bool,
    /// `WITH DISTINCT ...` -- dedups the projected rows, same as `RETURN
    /// DISTINCT` (`Tail::Return`'s own `distinct` flag), applied right
    /// after projection/aggregation, before `where_clause` (matching
    /// `WHERE`'s own "only sees the projected/aggregated names" rule for
    /// an aggregating `WITH` -- `DISTINCT` puts `WITH` in that same
    /// post-projection-only regime).
    pub distinct: bool,
    pub where_clause: Option<WithExpr>,
    pub order_by: Option<Vec<(ReturnExpr, SortDir)>>,
    /// Always applied *after* `order_by` (real Cypher's own rule — skip N,
    /// then take the following `limit`, against the sorted sequence when
    /// one exists), regardless of which field a caller happens to read
    /// first.
    ///
    /// Any expression, not just a literal integer — `SKIP $n`, `SKIP
    /// toInteger(rand()*9)` (TCK's `ReturnSkipLimit1 [2]`/`[3]`) are real
    /// Cypher. Evaluated exactly once against an empty row (no pattern
    /// variable can be in scope here) — see
    /// `executor::resolve_skip_limit`.
    pub skip: Option<ReturnExpr>,
    pub limit: Option<ReturnExpr>,
}

/// One `MATCH <pattern>[, <pattern>...] [WHERE ...] [WITH ...]` segment.
/// Comma-separated patterns that continue each other (a later pattern's
/// start is the previous one's last-introduced variable) are spliced into
/// a single linear `Pattern` at parse time (see
/// `parser::group_into_linear_patterns`) — this only ever holds one
/// already-combined `Pattern`, not several. A genuine disjoint cross join
/// (`MATCH (a:A), (b:B)`) instead becomes *multiple* `QueryPart`s, one per
/// disjoint group (see `parser::parse_match_part`'s docs).
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct UnwindClause {
    pub source: UnwindSource,
    pub var: String,
    pub where_clause: Option<WithExpr>,
    pub with: Option<WithClause>,
}

/// Where an `UNWIND`'s list comes from — any expression (`range(0, 2)`,
/// `n.tags`, a bound `collect()` result, an inline `[1, 2, 3]`, ...),
/// evaluated per input row and required to produce a `Value::List`
/// (`executor::eval_unwind`). Element bindings restore graph identity via
/// the same `value_to_binding_restore` regardless of where the list came
/// from — there's no `PropertyValue::List` yet, so a `$param` bound
/// directly to a list still isn't reachable here on its own (every
/// `$param` is a single scalar); a `$param` used *inside* an inline list
/// literal (`[1, 2, $p]`) still works, since each element substitutes
/// independently.
#[derive(Debug, Clone, PartialEq)]
pub struct UnwindSource(pub ReturnExpr);

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
#[derive(Debug, Clone, PartialEq)]
pub struct MergeClause {
    pub pattern: Pattern,
    /// `MERGE p = (a)-[:R]->(b)` -- captures the whole matched-or-created
    /// pattern as a path, same as ordinary `MATCH`'s own named-path
    /// capture (`Pattern::path_var`) but on `MergeClause` directly, since
    /// `Pattern` itself has no notion of a MERGE-vs-MATCH distinction.
    /// `MergeClause::pattern` caps at one relationship hop (see this
    /// struct's own construction site), so assembling the path is just
    /// "the start node, plus the one hop's edge and node if present" --
    /// no BFS/parent-pointer tracking needed the way a general
    /// variable-length pattern's path capture would (TCK's Merge1 [13],
    /// Merge5 [10]).
    pub path_var: Option<String>,
    pub on_create: Vec<SetItem>,
    pub on_match: Vec<SetItem>,
    pub with: Option<WithClause>,
}

/// `CALL proc.name(args) [YIELD ...]` -- both the in-query reading-clause
/// form (`QueryClause::Call`, `args` always `Some` per the grammar's own
/// `queryCallSt : CALL invocationName parenExpressionChain (YIELD
/// yieldItems)?`, parens mandatory) and the standalone top-level form
/// (`Statement::StandaloneCall`, whose `standaloneCall` rule's own
/// `parenExpressionChain?` is optional -- `args: None` is that implicit-
/// argument shape, `CALL proc` with no parens at all, where each declared
/// input instead resolves from a same-named `$param`, TCK's Call1
/// `[2]`/`[11]`, Call2 `[3]`).
#[derive(Debug, Clone, PartialEq)]
pub struct CallClause {
    pub name: String,
    pub args: Option<Vec<ReturnExpr>>,
    /// A trailing `WITH` (TCK's Call6 `[1]`: `CALL ... YIELD label WITH
    /// count(*) AS c CALL ... YIELD label RETURN *`) -- same "glued onto
    /// the preceding reading clause while walking `multiPartQ`'s children"
    /// mechanism `QueryPart`/`UnwindClause`/`MergeClause` already use for
    /// their own trailing `with`. Always `None` on a `Statement::
    /// StandaloneCall` (never read there -- a standalone call is always
    /// the whole statement, nothing can follow it).
    pub with: Option<WithClause>,
    /// `None` -- no `YIELD` written at all. For the in-query form this
    /// means every output is discarded, nothing bound into scope (TCK's
    /// Call1 `[12]`: referencing an un-yielded output afterward is
    /// `UndefinedVariable`); for the standalone form it instead means
    /// "auto-yield every output," same as `Some(CallYield::Star)` would,
    /// since a standalone `CALL` *is* the whole query (TCK's Call1 `[5]`,
    /// Call2 `[2]`) -- `executor::eval_standalone_call` is what actually
    /// applies that standalone-only distinction; this AST shape alone
    /// can't tell the two apart.
    pub yield_items: Option<CallYield>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallYield {
    /// `YIELD *` -- every declared output, bound under its own name.
    Star,
    /// `YIELD a, b AS c, ...` -- explicit output name (optionally
    /// renamed), plus `yieldItems`' own optional trailing `WHERE` (only
    /// this variant's grammar production carries one -- `YIELD *` has no
    /// `where?` of its own).
    Items(Vec<(String, Option<String>)>, Option<Box<Expr>>),
}

/// One reading clause in a `MATCH`/`UNWIND`/`MERGE` sequence. `Match` is
/// today's `MATCH`/`OPTIONAL MATCH ... [WHERE] [WITH]` segment; `Unwind`
/// fans out a list; `Merge` matches-or-creates. All three can optionally
/// end in a `WITH` — see `Statement::Match`'s docs for the WITH-
/// separation/one-WITH-total rules this enum's variants are validated
/// against.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryClause {
    Match(QueryPart),
    Unwind(UnwindClause),
    Merge(MergeClause),
    /// A statement-leading `WITH` -- no pattern to match, just projects/
    /// aliases values (`WITH [1,2,3] AS list ...`). Distinct from the
    /// trailing `with: Option<WithClause>` every other clause kind
    /// already carries (that one follows a real pattern match; this one
    /// has nothing preceding it at all).
    With(WithClause),
    /// `SET ...` immediately followed by `WITH` -- continues the query
    /// past the mutation instead of only ever allowing one trailing
    /// `RETURN` (the pre-existing `Tail::Set`'s own `ReturnTail`).
    /// Doesn't itself change any row's bindings, only the underlying
    /// graph -- `execute_match`'s clause loop applies each item per row
    /// and passes `current_rows` through unchanged, same as `Merge`
    /// already does for its own non-binding side effects.
    Set(Vec<SetItem>),
    /// `DELETE`/`DETACH DELETE ...` immediately followed by `WITH` -- same
    /// reasoning as `Set` above (TCK's Delete6 "Persistence of delete
    /// clause side effects"). `detach`: whether `DETACH` was present
    /// (mirrors `Tail::Delete` vs `Tail::DetachDelete`'s own split).
    Delete {
        items: Vec<ReturnExpr>,
        detach: bool,
    },
    /// `REMOVE ...` immediately followed by `WITH` -- same reasoning as
    /// `Set` above (TCK's Remove3 "Persistence of remove clause side
    /// effects").
    Remove(Vec<RemoveItem>),
    /// `CREATE ...` immediately followed by `WITH` -- same positive-
    /// lookahead reasoning as `Set` above, but unlike `Set`/`Delete`/
    /// `Remove`, CREATE *does* change row bindings (each pattern's
    /// fresh/reused node-and-relationship vars) -- `execute_match`'s
    /// clause loop reuses `materialize_create` (the exact same function
    /// `Tail::Create`/`Executor::execute_create` already call) and
    /// extends `carried_vars` with every pattern's own vars, same as
    /// `Merge`'s own binding-changing clause already does.
    Create(Vec<Pattern>),
    /// `CALL proc.name(args) [YIELD ...]` used as a reading clause --
    /// `CallClause::args` is always `Some` here (the grammar's own
    /// `queryCallSt` requires parens), see `CallClause`'s own docs.
    Call(CallClause),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// `BEGIN` / `COMMIT` / `ROLLBACK` — MarsDB's session-transaction
    /// extension (issue #142; not openCypher, which has no transaction
    /// statements at all — real deployments do this at the protocol/
    /// session layer). Recognized textually by `parse` before the ANTLR
    /// grammar ever runs (single bare keywords, nothing for a grammar to
    /// disambiguate — same out-of-grammar precedent as `;`-separated
    /// batches, see grammar/README.md), and handled entirely by
    /// `marsdb::Database`'s session layer; the executor rejects them
    /// with a pointer there, since it has no session to act on.
    Begin,
    Commit,
    Rollback,
    Create(Vec<Pattern>),
    /// `CREATE INDEX ON :Label(prop)`, optionally `UNIQUE`.
    CreateIndex {
        label: String,
        prop: String,
        unique: bool,
    },
    /// `EXPLAIN <statement>` — describes the plan `<statement>` would run
    /// (scan vs seek, pushdown applied) without executing any of it. Never
    /// nests (the parser only ever wraps a `create_index_stmt`/
    /// `create_stmt`/`match_stmt`, not another `explain_stmt`).
    Explain(Box<Statement>),
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
        /// Applied after `order_by`, before `limit` — same convention as
        /// `WithClause::skip`. Boxed (clippy's `large_enum_variant`) --
        /// `Statement::Match` would otherwise be far larger than
        /// `Statement`'s other variants just for this rarely-non-`None`
        /// field.
        skip: Option<Box<ReturnExpr>>,
        limit: Option<Box<ReturnExpr>>,
    },
    /// `<match_stmt> UNION [ALL] <match_stmt> (UNION [ALL] <match_stmt>)*`
    /// — each `parts` entry is itself a `Statement::Match`, own scope, no
    /// bindings shared across parts (real Cypher: a UNION member can't see
    /// a preceding member's variables). `all` applies uniformly to the
    /// whole statement — real Cypher rejects mixing bare `UNION` and
    /// `UNION ALL` in one statement (a semantic check, `parser::
    /// parse_union_stmt`, since it's only checkable once every part's own
    /// `UNION`/`UNION ALL` keyword is in hand). `false` = dedup the
    /// combined rows (`UNION`'s default); `true` = keep every row
    /// (`UNION ALL`).
    Union {
        parts: Vec<Statement>,
        all: bool,
    },
    /// A bare `CALL proc.name(args) [YIELD ...]` with nothing else in the
    /// statement (the grammar's own `standaloneCall`, a top-level
    /// alternative alongside `regularQuery` -- never wrapped in
    /// `Statement::Match`, since there's no pattern to match at all). See
    /// `CallClause`'s own docs for why `args`/`yield_items` mean something
    /// subtly different here than in `QueryClause::Call`. Boxed (clippy's
    /// `large_enum_variant`) -- `CallClause` is far bigger than
    /// `Statement`'s other variants just for this rarely-taken one.
    StandaloneCall(Box<CallClause>),
}
