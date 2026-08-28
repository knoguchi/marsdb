//! Slot-index compilation for the read path's hottest plan shapes.
//!
//! `executor::BindingRow` (`HashMap<String, Binding>`) is the row
//! representation used everywhere in the executor. This module is a
//! read-only, post-hoc analysis pass over an already-built `LogicalPlan`
//! (run after `planner::apply_index_seeks`, which never introduces new
//! variable names) that tries to resolve every variable name the plan
//! touches to a fixed `usize` slot index, producing a `SlottedPlan`/
//! `SlotExpr` mirror the executor can run against `Vec<Binding>` rows
//! instead — no per-row string hashing for row construction or predicate
//! evaluation.
//!
//! This never changes `LogicalPlan`/`Expr` construction and never touches
//! `planner.rs`: compilation either succeeds (`Some`) or the executor
//! falls back to the unmodified `HashMap`-based path (`None`). A plan
//! containing `VarExpand`/`MatchRelList` anywhere, an `IndexSeek` whose
//! value depends on the current row (`IndexSeekValue::RowExpr`), or a
//! predicate built from anything other than the nine simple/combinator
//! `Expr` leaves (`GeneralCompare`/`GeneralIsNull`/`GeneralBare`/
//! `Pattern`/`Exists`/`ExistsSubquery` all disqualify) is never eligible.

use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{CompareOp, Expr, Literal};
use crate::ir::{ExpandDirection, IndexSeekValue, LogicalPlan};

/// name -> slot index for one compiled plan invocation, plus the reverse
/// mapping for boundary conversion back to a `BindingRow` and for
/// diagnostic messages. Built once per top-level `stream_plan_auto` call,
/// not per row.
#[derive(Debug)]
pub(crate) struct SlotTable {
    index: HashMap<String, usize>,
    names: Vec<String>,
}

impl SlotTable {
    fn new() -> Self {
        SlotTable {
            index: HashMap::new(),
            names: Vec::new(),
        }
    }

    fn get_or_insert(&mut self, name: &str) -> usize {
        if let Some(&slot) = self.index.get(name) {
            return slot;
        }
        let slot = self.names.len();
        self.names.push(name.to_string());
        self.index.insert(name.to_string(), slot);
        slot
    }

    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    pub(crate) fn names(&self) -> &[String] {
        &self.names
    }

    pub(crate) fn name_of(&self, slot: usize) -> &str {
        &self.names[slot]
    }
}

/// Mirrors the `LogicalPlan` variants eligible for the slotted engine.
/// No `VarExpand`/`MatchRelList` variant exists here at all — a plan
/// containing either is fully ineligible, never partially compiled.
/// `input`/predicate subtrees are `Rc`-shared (not deep-cloned) so the
/// same compiled plan can be reused across a dispatcher call's seed
/// conversion and its recursive per-node streaming.
#[derive(Debug, Clone)]
pub(crate) enum SlottedPlan {
    AllNodesScan {
        slot: usize,
    },
    NodeByLabelScan {
        slot: usize,
        label: String,
    },
    /// `IndexSeekValue::Fixed` only — `RowExpr` disqualifies the whole
    /// plan during compilation.
    IndexSeek {
        slot: usize,
        label: String,
        prop: String,
        value: marsdb_graph::PropertyValue,
    },
    IndexRangeSeek {
        slot: usize,
        label: String,
        prop: String,
        lo: Option<(marsdb_graph::PropertyValue, bool)>,
        hi: Option<(marsdb_graph::PropertyValue, bool)>,
    },
    /// `rel_predicate` stays a plain `Expr`, unconverted: it's evaluated
    /// directly against the swept edge record's raw bytes
    /// (`executor::eval_scan_predicate`), never against a row, so slot
    /// resolution has nothing to do here. `planner::apply_index_seeks`
    /// only ever produces `And`/`Not`/`Compare`/`IsNull` in this
    /// position (enforced at runtime by
    /// `executor::collect_scan_prop_ids`), so this is always eligible.
    EdgeTypeScan {
        src_slot: usize,
        rel_slot: usize,
        dst_slot: usize,
        rel_types: Vec<String>,
        src_label: Option<String>,
        dst_label: Option<String>,
        rel_predicate: Option<Expr>,
    },
    Seed {
        slot: usize,
    },
    Expand {
        input: Rc<SlottedPlan>,
        from_slot: usize,
        to_slot: usize,
        rel_slot: Option<usize>,
        rel_labels: Vec<String>,
        direction: ExpandDirection,
    },
    Filter {
        input: Rc<SlottedPlan>,
        predicate: SlotExpr,
    },
}

/// Mirrors `Expr`'s 9 simple/combinator variants only — every operand
/// naming a row variable resolves to a slot index; property names stay
/// plain strings (never row-bound). The other 6 `Expr` variants
/// (`GeneralCompare`/`GeneralIsNull`/`GeneralBare`/`Pattern`/`Exists`/
/// `ExistsSubquery`) have no `SlotExpr` counterpart at all — encountering
/// one anywhere in a predicate tree disqualifies the whole plan.
#[derive(Debug, Clone)]
pub(crate) enum SlotExpr {
    And(Box<SlotExpr>, Box<SlotExpr>),
    Or(Box<SlotExpr>, Box<SlotExpr>),
    Not(Box<SlotExpr>),
    Compare(usize, String, CompareOp, Literal),
    PropCompare(usize, String, CompareOp, usize, String),
    IsNull(usize, String),
    HasLabel(usize, String),
    VarEq(usize, usize),
    /// `edge_set_var` is always an internal `VarExpand`-produced name
    /// (see `Expr::EdgeNotInSet`'s docs); it resolves through the same
    /// `SlotTable` as any other name. In v1 this variant is reachable in
    /// compiled form but never executed: `EdgeNotInSet` only appears
    /// alongside a `VarExpand` hop, and any plan containing `VarExpand`
    /// is already ineligible — see `slots_disqualify_var_expand_plan`
    /// below. Implemented anyway so lifting the `VarExpand` exclusion
    /// later doesn't need to revisit `SlotExpr`.
    EdgeNotInSet(usize, usize),
}

/// Tries to compile `plan` into a `SlottedPlan`, seeding `table` from
/// `seed_keys` (the incoming row's full existing key set) before
/// assigning slots to any variable the plan itself introduces. Seeding
/// from the incoming row first — rather than only from names the plan
/// references — is what keeps a "passenger" binding (one the current
/// plan never touches but that must survive to the tail, e.g.
/// `OPTIONAL MATCH`'s internal `__seed_idx` tag, or any variable carried
/// from an earlier clause) from being silently dropped on the
/// `BindingRow` <-> `SlotRow` boundary conversion: every seeded name
/// gets a slot up front, and a slot with no plan-node writer is simply
/// copied through unchanged on every row clone, exactly mirroring
/// today's `HashMap::clone()` behavior.
///
/// Returns `None` at the first disqualifying node/expression — the
/// caller falls back to the unmodified `HashMap`-based execution path.
pub(crate) fn try_compile_slotted<'a>(
    plan: &LogicalPlan,
    seed_keys: impl Iterator<Item = &'a str>,
) -> Option<(SlottedPlan, SlotTable)> {
    let mut table = SlotTable::new();
    for key in seed_keys {
        table.get_or_insert(key);
    }
    let slotted = compile_plan(plan, &mut table)?;
    Some((slotted, table))
}

fn compile_plan(plan: &LogicalPlan, table: &mut SlotTable) -> Option<SlottedPlan> {
    match plan {
        LogicalPlan::AllNodesScan { var } => Some(SlottedPlan::AllNodesScan {
            slot: table.get_or_insert(var),
        }),
        LogicalPlan::NodeByLabelScan { var, label } => Some(SlottedPlan::NodeByLabelScan {
            slot: table.get_or_insert(var),
            label: label.clone(),
        }),
        LogicalPlan::IndexSeek {
            var,
            label,
            prop,
            value,
        } => match value {
            IndexSeekValue::Fixed(value) => Some(SlottedPlan::IndexSeek {
                slot: table.get_or_insert(var),
                label: label.clone(),
                prop: prop.clone(),
                value: value.clone(),
            }),
            // Row-dependent lookup value -- the whole plan needs the
            // current `BindingRow` to re-evaluate it per seed row, so
            // it can't be planned once at compile time.
            IndexSeekValue::RowExpr(_) => None,
        },
        LogicalPlan::IndexRangeSeek {
            var,
            label,
            prop,
            lo,
            hi,
        } => Some(SlottedPlan::IndexRangeSeek {
            slot: table.get_or_insert(var),
            label: label.clone(),
            prop: prop.clone(),
            lo: lo.clone(),
            hi: hi.clone(),
        }),
        LogicalPlan::EdgeTypeScan {
            src_var,
            rel_var,
            dst_var,
            rel_types,
            src_label,
            dst_label,
            rel_predicate,
        } => Some(SlottedPlan::EdgeTypeScan {
            src_slot: table.get_or_insert(src_var),
            rel_slot: table.get_or_insert(rel_var),
            dst_slot: table.get_or_insert(dst_var),
            rel_types: rel_types.clone(),
            src_label: src_label.clone(),
            dst_label: dst_label.clone(),
            rel_predicate: rel_predicate.clone(),
        }),
        LogicalPlan::Seed { var } => Some(SlottedPlan::Seed {
            slot: table.get_or_insert(var),
        }),
        LogicalPlan::Expand {
            input,
            from_var,
            to_var,
            rel_var,
            rel_labels,
            direction,
        } => {
            let input = compile_plan(input, table)?;
            Some(SlottedPlan::Expand {
                input: Rc::new(input),
                from_slot: table.get_or_insert(from_var),
                to_slot: table.get_or_insert(to_var),
                rel_slot: rel_var.as_deref().map(|v| table.get_or_insert(v)),
                rel_labels: rel_labels.clone(),
                direction: *direction,
            })
        }
        // BFS emits a variable number of rows per input row with
        // `Path`/`List`-valued payloads and cross-hop exclude-set
        // tracking -- a genuinely separate design, not v1 scope.
        LogicalPlan::VarExpand { .. } => None,
        // Row-dependent already-bound relationship-list walk -- not v1
        // scope.
        LogicalPlan::MatchRelList { .. } => None,
        LogicalPlan::Filter { input, predicate } => {
            let input = compile_plan(input, table)?;
            let predicate = compile_expr(predicate, table)?;
            Some(SlottedPlan::Filter {
                input: Rc::new(input),
                predicate,
            })
        }
    }
}

fn compile_expr(expr: &Expr, table: &mut SlotTable) -> Option<SlotExpr> {
    match expr {
        Expr::And(l, r) => Some(SlotExpr::And(
            Box::new(compile_expr(l, table)?),
            Box::new(compile_expr(r, table)?),
        )),
        Expr::Or(l, r) => Some(SlotExpr::Or(
            Box::new(compile_expr(l, table)?),
            Box::new(compile_expr(r, table)?),
        )),
        Expr::Not(e) => Some(SlotExpr::Not(Box::new(compile_expr(e, table)?))),
        Expr::Compare(pa, op, lit) => Some(SlotExpr::Compare(
            table.get_or_insert(&pa.var),
            pa.prop.clone(),
            *op,
            lit.clone(),
        )),
        Expr::PropCompare(left, op, right) => Some(SlotExpr::PropCompare(
            table.get_or_insert(&left.var),
            left.prop.clone(),
            *op,
            table.get_or_insert(&right.var),
            right.prop.clone(),
        )),
        Expr::IsNull(pa) => Some(SlotExpr::IsNull(
            table.get_or_insert(&pa.var),
            pa.prop.clone(),
        )),
        Expr::HasLabel(var, label) => {
            Some(SlotExpr::HasLabel(table.get_or_insert(var), label.clone()))
        }
        Expr::VarEq(a, b) => Some(SlotExpr::VarEq(
            table.get_or_insert(a),
            table.get_or_insert(b),
        )),
        Expr::EdgeNotInSet {
            edge_var,
            edge_set_var,
        } => Some(SlotExpr::EdgeNotInSet(
            table.get_or_insert(edge_var),
            table.get_or_insert(edge_set_var),
        )),
        Expr::GeneralCompare(..)
        | Expr::GeneralIsNull(_)
        | Expr::GeneralBare(_)
        | Expr::Pattern(_)
        | Expr::Exists { .. }
        | Expr::ExistsSubquery(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{QueryClause, Statement};
    use crate::planner::{apply_index_seeks, build_match_plan};
    use marsdb_graph::{GraphStore, Txn};
    use std::collections::HashSet;

    fn plan_for(store: &GraphStore, cypher: &str) -> LogicalPlan {
        let Statement::Match { clauses, .. } = crate::antlr_visitor::parse_antlr(cypher).unwrap()
        else {
            panic!("expected a Match statement");
        };
        let QueryClause::Match(part) = clauses.into_iter().next().unwrap() else {
            panic!("expected a Match clause");
        };
        let carried_vars = HashSet::new();
        let read_txn = store.begin_read().expect("begin_read");
        let txn = Txn::Read(&read_txn);
        let plan = build_match_plan(&part.pattern, &part.where_clause, &carried_vars)
            .expect("build_match_plan");
        apply_index_seeks(plan, txn).expect("apply_index_seeks")
    }

    #[test]
    fn var_expand_pattern_is_never_eligible() {
        let store = GraphStore::open_memory().expect("open_memory");
        let plan = plan_for(&store, "MATCH (a)-[:R*1..3]->(b) RETURN a, b");
        assert!(try_compile_slotted(&plan, std::iter::empty()).is_none());
    }

    #[test]
    fn simple_hop_with_label_filter_is_eligible() {
        let store = GraphStore::open_memory().expect("open_memory");
        let plan = plan_for(
            &store,
            "MATCH (a:Person)-[:KNOWS]->(b:Person) WHERE b.age > 30 RETURN a, b",
        );
        let (slotted, table) =
            try_compile_slotted(&plan, std::iter::empty()).expect("should compile");
        assert!(table.names().contains(&"a".to_string()));
        assert!(table.names().contains(&"b".to_string()));
        assert!(matches!(slotted, SlottedPlan::Filter { .. }));
    }
}
