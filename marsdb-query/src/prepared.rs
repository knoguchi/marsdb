use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use marsdb_graph::PropertyValue;

use crate::ast::Statement;

/// A statement parsed once and reused across many executions with
/// different bound `$parameter` values -- MarsDB's answer to a real
/// prepared-statement handle (like `sqlite3_stmt`). `Database::prepare`
/// creates one; `Database::execute_prepared` runs it.
///
/// Currently accelerates repeat executions by skipping semantic
/// validation when it's safe to (see `ParamFingerprint`'s doc comment
/// for the exact condition). Does not yet cache query planning
/// (`build_match_plan`/`apply_index_seeks` still rerun every call) --
/// see `crate::ParamSite`/`crate::PathStep`/`crate::IndexSeekOutcome`,
/// already in place as forward-compatible infrastructure for that
/// follow-up, which needs planner-level provenance tracking to safely
/// correlate a cached plan's embedded index-seek value back to the
/// parameter it came from (a value-equality heuristic isn't sound: two
/// different parameters can coincidentally share a value).
pub struct PreparedPlan {
    stmt: Statement,
    validated: RefCell<Option<ValidatedAt>>,
}

struct ValidatedAt {
    fingerprint: ParamFingerprint,
    schema_generation: u64,
}

impl PreparedPlan {
    pub fn new(stmt: Statement) -> Self {
        Self {
            stmt,
            validated: RefCell::new(None),
        }
    }

    pub fn statement(&self) -> &Statement {
        &self.stmt
    }

    /// Whether `validate_statement` can safely be skipped for this call:
    /// every previous validation recorded the same parameter-category
    /// fingerprint and the schema hasn't changed since (a new index
    /// might make a different plan available; see
    /// `GraphStore::schema_generation`'s doc comment).
    pub fn can_skip_validation(
        &self,
        params: &HashMap<String, PropertyValue>,
        schema_generation: u64,
    ) -> bool {
        match &*self.validated.borrow() {
            Some(v) => {
                v.schema_generation == schema_generation
                    && v.fingerprint == ParamFingerprint::of(params)
            }
            None => false,
        }
    }

    /// Records that `params` were just validated against this plan's
    /// statement, at the given schema generation -- call after a normal
    /// (validating) execution succeeds, so the *next* call with a
    /// matching fingerprint can skip validation.
    pub fn record_validated(
        &self,
        params: &HashMap<String, PropertyValue>,
        schema_generation: u64,
    ) {
        *self.validated.borrow_mut() = Some(ValidatedAt {
            fingerprint: ParamFingerprint::of(params),
            schema_generation,
        });
    }
}

/// Coarse shape of a parameter value, matching `semantic.rs`'s `Kind`
/// categories closely enough that validation only ever branches on this
/// distinction, never on the concrete value within a category (an `Int`
/// and a `String` both type as `Kind::Scalar` and can never make
/// `validate_statement` disagree; a `Null` types `Kind::Unknown`, a list
/// types `Kind::List`, a map types `Kind::Map`, and several checks --
/// `UNWIND`'s source, list-comprehension/quantifier sources, arithmetic/
/// boolean/slice/index operands, SET-target/HasLabel/VarEq targets fed
/// by a param-derived variable -- do branch on category).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParamCategory {
    Null,
    Scalar,
    List,
    Map,
}

fn category_of(v: &PropertyValue) -> ParamCategory {
    match v {
        PropertyValue::Null => ParamCategory::Null,
        PropertyValue::List(_) => ParamCategory::List,
        PropertyValue::Map(_) => ParamCategory::Map,
        _ => ParamCategory::Scalar,
    }
}

/// Every bound parameter's category, order-independent. Two calls with
/// equal fingerprints are guaranteed to produce the same
/// `validate_statement` outcome for the same `Statement`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParamFingerprint(BTreeMap<String, ParamCategory>);

impl ParamFingerprint {
    fn of(params: &HashMap<String, PropertyValue>) -> Self {
        Self(
            params
                .iter()
                .map(|(k, v)| (k.clone(), category_of(v)))
                .collect(),
        )
    }
}
