use crate::ast::{CompareOp, Expr, NodePattern, Pattern, PropAccess, RelDirection};
use crate::error::QueryError;
use crate::ir::{ExpandDirection, LogicalPlan};

struct VarNamer {
    next: usize,
}

impl VarNamer {
    fn new() -> Self {
        Self { next: 0 }
    }

    /// Anonymous nodes/rels (e.g. `(a)-->()`) still need a name to track
    /// their binding through the plan; synthesize one that can't collide
    /// with a user-written identifier.
    fn name(&mut self, given: &Option<String>) -> String {
        match given {
            Some(v) => v.clone(),
            None => {
                let n = format!("__anon{}", self.next);
                self.next += 1;
                n
            }
        }
    }
}

pub fn build_match_plan(pattern: &Pattern, where_clause: &Option<Expr>) -> Result<LogicalPlan, QueryError> {
    let mut namer = VarNamer::new();
    let start_var = namer.name(&pattern.start.var);
    let mut plan = scan_for(&start_var, &pattern.start);
    let mut from_var = start_var;
    for (rel, node) in &pattern.hops {
        let to_var = namer.name(&node.var);
        let direction = match rel.direction {
            RelDirection::Right => ExpandDirection::Out,
            RelDirection::Left => ExpandDirection::In,
            RelDirection::Either => ExpandDirection::Either,
        };
        plan = match rel.hop_range {
            None => LogicalPlan::Expand {
                input: Box::new(plan),
                from_var: from_var.clone(),
                to_var: to_var.clone(),
                rel_var: rel.var.clone(),
                rel_label: rel.rel_type.clone(),
                direction,
            },
            Some((min_hops, max_hops)) => {
                if rel.var.is_some() {
                    // Real Cypher binds a *list* of relationships for a
                    // variable-length pattern's rel_var; v1 doesn't support
                    // that value shape, so reject rather than silently bind
                    // just the last hop's edge (wrong, not just incomplete).
                    return Err(QueryError::Parse(
                        "binding a variable name to a variable-length relationship (e.g. \
                         [r:TYPE*1..3]) isn't supported — omit the variable name"
                            .into(),
                    ));
                }
                LogicalPlan::VarExpand {
                    input: Box::new(plan),
                    from_var: from_var.clone(),
                    to_var: to_var.clone(),
                    rel_label: rel.rel_type.clone(),
                    direction,
                    min_hops,
                    max_hops,
                }
            }
        };
        // Hop nodes reach this point via Expand/VarExpand, which don't
        // pre-filter by label at all (unlike the start node's
        // NodeByLabelScan) — every listed label must be Filter-checked
        // here, not just the extras beyond the first.
        plan = wrap_labels_and_props(plan, &to_var, node, 0);
        from_var = to_var;
    }
    if let Some(expr) = where_clause {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: expr.clone(),
        };
    }
    Ok(plan)
}

fn scan_for(var: &str, node: &NodePattern) -> LogicalPlan {
    // The first label (if any) narrows the scan; any additional labels
    // (`(n:Post:Message)`) become extra HasLabel filters — a node must
    // have ALL listed labels, matching Cypher's multi-label AND semantics.
    let base = match node.labels.first() {
        Some(label) => LogicalPlan::NodeByLabelScan {
            var: var.to_string(),
            label: label.clone(),
        },
        None => LogicalPlan::AllNodesScan { var: var.to_string() },
    };
    // Skip the first label — NodeByLabelScan above already selected for it.
    wrap_labels_and_props(base, var, node, 1)
}

/// Inline node-pattern properties (`(a:Person {name:'Alice'})`) and any
/// labels not already handled by a preceding scan (`skip` labels from the
/// front) compile to the same Filter machinery as a WHERE clause, just
/// synthesized from the pattern.
fn wrap_labels_and_props(plan: LogicalPlan, var: &str, node: &NodePattern, skip: usize) -> LogicalPlan {
    let mut plan = plan;
    for label in node.labels.iter().skip(skip) {
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate: Expr::HasLabel(var.to_string(), label.clone()),
        };
    }
    for (key, lit) in &node.props {
        let predicate = Expr::Compare(
            PropAccess {
                var: var.to_string(),
                prop: key.clone(),
            },
            CompareOp::Eq,
            lit.clone(),
        );
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate,
        };
    }
    plan
}
