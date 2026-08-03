use std::collections::{BTreeMap, HashMap, HashSet};

use marsdb_graph::{EdgeId, GraphStore, NodeId, PropertyValue, WriteTransaction};

use crate::ast::{CompareOp, Expr, Literal, Pattern, PropAccess, RelDirection, ReturnExpr, ReturnItem, Statement, Tail};
use crate::error::QueryError;
use crate::ir::LogicalPlan;
use crate::planner::build_match_plan;
use crate::result::QueryResult;
use crate::value::Value;

#[derive(Debug, Clone, Copy)]
enum Binding {
    Node(NodeId),
    Edge(EdgeId),
}

type BindingRow = HashMap<String, Binding>;

pub struct Executor<'a> {
    store: &'a GraphStore,
}

impl<'a> Executor<'a> {
    pub fn new(store: &'a GraphStore) -> Self {
        Self { store }
    }

    /// Runs the whole statement inside a single write transaction — the
    /// crash-safety boundary from the plan (one statement = one commit).
    /// Every graph access below this point must go through `write_txn` and
    /// the `*_in_txn` GraphStore methods, never the standalone
    /// `self.store.*` methods, which open (and would deadlock trying to
    /// re-open) their own transaction.
    pub fn execute(&self, stmt: &Statement) -> Result<QueryResult, QueryError> {
        let write_txn = self.store.begin_write()?;
        let outcome = match stmt {
            Statement::Create(patterns) => self.execute_create(&write_txn, patterns),
            Statement::Match {
                pattern,
                where_clause,
                tail,
                limit,
            } => self.execute_match(&write_txn, pattern, where_clause, tail, *limit),
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
        for pattern in patterns {
            let start_labels = pattern_labels(&pattern.start.labels);
            let start_props = literal_props_to_values(&pattern.start.props);
            let mut prev_id = GraphStore::create_node_in_txn(write_txn, &start_labels, start_props)?;

            for (rel, node) in &pattern.hops {
                let labels = pattern_labels(&node.labels);
                let props = literal_props_to_values(&node.props);
                let node_id = GraphStore::create_node_in_txn(write_txn, &labels, props)?;

                let rel_label = rel.rel_type.clone().unwrap_or_else(|| "REL".to_string());
                let rel_props = literal_props_to_values(&rel.props);
                let (src, dst) = match rel.direction {
                    RelDirection::Right => (prev_id, node_id),
                    RelDirection::Left => (node_id, prev_id),
                };
                GraphStore::create_edge_in_txn(write_txn, &rel_label, src, dst, rel_props)?;
                prev_id = node_id;
            }
        }
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }

    fn execute_match(
        &self,
        write_txn: &WriteTransaction,
        pattern: &Pattern,
        where_clause: &Option<Expr>,
        tail: &Tail,
        limit: Option<i64>,
    ) -> Result<QueryResult, QueryError> {
        let mut plan = build_match_plan(pattern, where_clause);
        if let Some(count) = limit {
            plan = LogicalPlan::Limit {
                input: Box::new(plan),
                count,
            };
        }
        let rows = self.eval_plan(write_txn, &plan)?;
        match tail {
            Tail::Return(items) => self.materialize_return(write_txn, items, &rows),
            Tail::Delete(vars) => self.materialize_delete(write_txn, vars, &rows, false),
            Tail::DetachDelete(vars) => self.materialize_delete(write_txn, vars, &rows, true),
            Tail::Set(items) => self.materialize_set(write_txn, items, &rows),
        }
    }

    fn eval_plan(&self, write_txn: &WriteTransaction, plan: &LogicalPlan) -> Result<Vec<BindingRow>, QueryError> {
        match plan {
            LogicalPlan::AllNodesScan { var } => self.scan(write_txn, var, None),
            LogicalPlan::NodeByLabelScan { var, label } => self.scan(write_txn, var, Some(label)),
            LogicalPlan::Expand {
                input,
                from_var,
                to_var,
                rel_var,
                rel_label,
                direction,
            } => {
                let base_rows = self.eval_plan(write_txn, input)?;
                let mut out = Vec::new();
                for row in base_rows {
                    let Some(Binding::Node(from_id)) = row.get(from_var).copied() else {
                        return Err(QueryError::UnboundVariable(from_var.clone()));
                    };
                    let entries =
                        GraphStore::neighbors_in_txn(write_txn, from_id, *direction, rel_label.as_deref())?;
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
            LogicalPlan::Filter { input, predicate } => {
                let rows = self.eval_plan(write_txn, input)?;
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    if self.eval_expr(write_txn, predicate, &row)? {
                        out.push(row);
                    }
                }
                Ok(out)
            }
            LogicalPlan::Limit { input, count } => {
                let mut rows = self.eval_plan(write_txn, input)?;
                rows.truncate((*count).max(0) as usize);
                Ok(rows)
            }
        }
    }

    fn scan(&self, write_txn: &WriteTransaction, var: &str, label: Option<&str>) -> Result<Vec<BindingRow>, QueryError> {
        let nodes = GraphStore::all_nodes_in_txn(write_txn, label)?;
        Ok(nodes
            .into_iter()
            .map(|n| {
                let mut row = BindingRow::new();
                row.insert(var.to_string(), Binding::Node(n.id));
                row
            })
            .collect())
    }

    fn eval_expr(&self, write_txn: &WriteTransaction, expr: &Expr, row: &BindingRow) -> Result<bool, QueryError> {
        Ok(match expr {
            Expr::And(l, r) => self.eval_expr(write_txn, l, row)? && self.eval_expr(write_txn, r, row)?,
            Expr::Or(l, r) => self.eval_expr(write_txn, l, row)? || self.eval_expr(write_txn, r, row)?,
            Expr::Not(e) => !self.eval_expr(write_txn, e, row)?,
            Expr::Compare(pa, op, lit) => {
                let prop_value = self.lookup_prop(write_txn, pa, row)?;
                compare(&prop_value, *op, lit)
            }
            Expr::HasLabel(var, label) => {
                let binding = row.get(var).ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                let Binding::Node(id) = binding else {
                    return Err(QueryError::UnboundVariable(var.clone()));
                };
                let node = GraphStore::get_node_in_txn(write_txn, *id)?;
                node.is_some_and(|n| n.labels.iter().any(|l| l == label))
            }
        })
    }

    fn lookup_prop(
        &self,
        write_txn: &WriteTransaction,
        pa: &PropAccess,
        row: &BindingRow,
    ) -> Result<Option<PropertyValue>, QueryError> {
        let binding = row
            .get(&pa.var)
            .ok_or_else(|| QueryError::UnboundVariable(pa.var.clone()))?;
        match binding {
            Binding::Node(id) => {
                let node = GraphStore::get_node_in_txn(write_txn, *id)?;
                Ok(node.and_then(|n| n.props.get(&pa.prop).cloned()))
            }
            Binding::Edge(id) => {
                let edge = GraphStore::get_edge_in_txn(write_txn, *id)?;
                Ok(edge.and_then(|e| e.props.get(&pa.prop).cloned()))
            }
        }
    }

    fn materialize_return(
        &self,
        write_txn: &WriteTransaction,
        items: &[ReturnItem],
        rows: &[BindingRow],
    ) -> Result<QueryResult, QueryError> {
        let columns = items
            .iter()
            .enumerate()
            .map(|(i, item)| item.alias.clone().unwrap_or_else(|| default_column_name(&item.expr, i)))
            .collect();
        let mut out_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let mut out_row = Vec::with_capacity(items.len());
            for item in items {
                out_row.push(self.eval_return_expr(write_txn, &item.expr, row)?);
            }
            out_rows.push(out_row);
        }
        Ok(QueryResult {
            columns,
            rows: out_rows,
        })
    }

    fn eval_return_expr(
        &self,
        write_txn: &WriteTransaction,
        expr: &ReturnExpr,
        row: &BindingRow,
    ) -> Result<Value, QueryError> {
        match expr {
            ReturnExpr::Var(var) => {
                let binding = row.get(var).ok_or_else(|| QueryError::UnboundVariable(var.clone()))?;
                match binding {
                    Binding::Node(id) => {
                        let node = GraphStore::get_node_in_txn(write_txn, *id)?
                            .expect("bound node exists within this statement's transaction");
                        Ok(Value::Node(node))
                    }
                    Binding::Edge(id) => {
                        let edge = GraphStore::get_edge_in_txn(write_txn, *id)?
                            .expect("bound edge exists within this statement's transaction");
                        Ok(Value::Edge(edge))
                    }
                }
            }
            ReturnExpr::Prop(pa) => {
                let value = self.lookup_prop(write_txn, pa, row)?;
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
            ReturnExpr::Call(name, args) => {
                let arg_values = args
                    .iter()
                    .map(|a| self.eval_return_expr(write_txn, a, row))
                    .collect::<Result<Vec<_>, _>>()?;
                call_builtin(name, &arg_values)
            }
            ReturnExpr::Case { test, whens, else_ } => {
                let test_value = match test {
                    Some(t) => Some(self.eval_return_expr(write_txn, t, row)?),
                    None => None,
                };
                for (when, then) in whens {
                    let when_value = self.eval_return_expr(write_txn, when, row)?;
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
                        return self.eval_return_expr(write_txn, then, row);
                    }
                }
                match else_ {
                    Some(e) => self.eval_return_expr(write_txn, e, row),
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
                }
            }
        }
        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
        })
    }
}

fn default_column_name(expr: &ReturnExpr, idx: usize) -> String {
    match expr {
        ReturnExpr::Var(v) => v.clone(),
        ReturnExpr::Prop(pa) => format!("{}.{}", pa.var, pa.prop),
        ReturnExpr::Lit(_) => format!("col{idx}"),
        ReturnExpr::Call(name, _) => format!("{name}(...)"),
        ReturnExpr::Case { .. } => format!("case{idx}"),
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

fn pattern_labels(labels: &[String]) -> Vec<&str> {
    if labels.is_empty() {
        vec!["Node"]
    } else {
        labels.iter().map(|s| s.as_str()).collect()
    }
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

/// Value equality for CASE's WHEN-comparison. Null == Null -> true here
/// deliberately, matching `compare()`'s convention above, not standard
/// three-valued NULL logic.
fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Null, _) | (_, Value::Null) => false,
        (Value::Property(pa), Value::Property(pb)) => pa == pb,
        (Value::Literal(la), Value::Literal(lb)) => la == lb,
        (Value::Property(pa), Value::Literal(lb)) => *pa == literal_to_value(lb),
        (Value::Literal(la), Value::Property(pb)) => literal_to_value(la) == *pb,
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
