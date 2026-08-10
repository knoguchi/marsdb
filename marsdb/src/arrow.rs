//! Arrow export of query results (`Database::query_arrow`) — the
//! core-owned columnar boundary. The transpose from the row-oriented
//! `QueryResult` happens exactly once, here; from the produced
//! `RecordBatch`es outward (the Rust `RecordBatchReader`, the C Data
//! Interface stream in marsdb-capi, the PyCapsule protocol in
//! marsdb-python), every consumer is zero-copy. When v2 columnar
//! storage lands, this API is the interface that stays while the
//! transpose disappears.
//!
//! Type inference is strict, per the precision discipline everywhere
//! else in this codebase: a column must be one consistent Arrow type
//! over every row or the export fails with a typed error naming the
//! column — no silent Int→Float promotion (it corrupts |int| > 2^53),
//! no stringifying entities. Rules:
//!
//! | Cypher values (whole column)      | Arrow type                |
//! |-----------------------------------|---------------------------|
//! | integers                          | `Int64` (exact)           |
//! | floats                            | `Float64`                 |
//! | strings                           | `Utf8`                    |
//! | booleans                          | `Boolean`                 |
//! | dates                             | `Date32` (per-value error outside i32 days) |
//! | durations                         | `Interval(MonthDayNano)`  |
//! | other temporals (time, datetime…) | `Utf8`, canonical ISO text |
//! | homogeneous lists of the above    | `List<child>`             |
//! | only nulls                        | `Null`                    |
//! | mixed numerics / mixed types      | error                     |
//! | nodes/edges/maps/paths            | error ("project properties") |
//!
//! Nulls (Cypher `null`, absent properties) become Arrow validity — a
//! column stays typed by its non-null values.

use std::sync::Arc;

use arrow_array::builder::{
    BooleanBuilder, Date32Builder, Float64Builder, Int64Builder, IntervalMonthDayNanoBuilder,
    ListBuilder, StringBuilder,
};
use arrow_array::{ArrayRef, NullArray, RecordBatch, RecordBatchReader};
use arrow_buffer::IntervalMonthDayNano;
use arrow_schema::{ArrowError, DataType, Field, Schema, SchemaRef};

use crate::{Error, Literal, PropertyValue, QueryError, QueryResult, Value};

// Re-exported so downstream crates (marsdb-capi, marsdb-python) can
// build C Data Interface streams without their own arrow-rs dependency
// (which would risk a version split against our arrays).
pub use arrow_array::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};
pub use {arrow_array, arrow_schema};

/// The inferred scalar kind of one column (pre-Arrow-mapping).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Int,
    Float,
    Text,
    Bool,
    Date,
    Duration,
    /// Time/DateTime/LocalTime/LocalDateTime — exported as ISO text.
    IsoText,
}

impl Kind {
    fn data_type(self) -> DataType {
        match self {
            Kind::Int => DataType::Int64,
            Kind::Float => DataType::Float64,
            Kind::Text | Kind::IsoText => DataType::Utf8,
            Kind::Bool => DataType::Boolean,
            Kind::Date => DataType::Date32,
            Kind::Duration => DataType::Interval(arrow_schema::IntervalUnit::MonthDayNano),
        }
    }
}

/// One column's inferred shape: scalar, list-of-scalar, or all-null.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ColumnShape {
    AllNull,
    Scalar(Kind),
    List(Kind),
}

fn type_error(column: &str, detail: impl std::fmt::Display) -> Error {
    Error::Query(QueryError::Type(format!(
        "column '{column}' cannot be exported to Arrow: {detail}"
    )))
}

/// A `Value` reduced to (kind, is-list) for inference; `None` = null.
/// Entities and maps are inference errors.
fn value_kind(column: &str, value: &Value) -> Result<Option<(Kind, bool)>, Error> {
    let prop_kind = |p: &PropertyValue| -> Result<Option<Kind>, Error> {
        Ok(Some(match p {
            PropertyValue::Null => return Ok(None),
            PropertyValue::Int(_) => Kind::Int,
            PropertyValue::Float(_) => Kind::Float,
            PropertyValue::String(_) => Kind::Text,
            PropertyValue::Bool(_) => Kind::Bool,
            PropertyValue::Date(_) => Kind::Date,
            PropertyValue::Duration { .. } => Kind::Duration,
            PropertyValue::LocalTime(_)
            | PropertyValue::Time { .. }
            | PropertyValue::LocalDateTime { .. }
            | PropertyValue::DateTime { .. } => Kind::IsoText,
            PropertyValue::List(_) | PropertyValue::Map(_) => {
                return Err(type_error(column, "nested list/map property value"))
            }
        }))
    };
    match value {
        Value::Null => Ok(None),
        Value::Literal(Literal::Null) | Value::Literal(Literal::Param(_)) => Ok(None),
        Value::Literal(Literal::Int(_)) => Ok(Some((Kind::Int, false))),
        Value::Literal(Literal::Float(_)) => Ok(Some((Kind::Float, false))),
        Value::Literal(Literal::String(_)) => Ok(Some((Kind::Text, false))),
        Value::Literal(Literal::Bool(_)) => Ok(Some((Kind::Bool, false))),
        Value::Property(p) => Ok(prop_kind(p)?.map(|k| (k, false))),
        Value::List(items) => {
            let mut child: Option<Kind> = None;
            for item in items {
                let Some((kind, is_list)) = value_kind(column, item)? else {
                    continue;
                };
                if is_list {
                    return Err(type_error(column, "nested lists"));
                }
                match child {
                    None => child = Some(kind),
                    Some(existing) if existing == kind => {}
                    Some(existing) => {
                        return Err(type_error(
                            column,
                            format!("mixed list element types ({existing:?} and {kind:?})"),
                        ))
                    }
                }
            }
            // An empty/all-null list contributes a list-ness signal
            // with no element kind; Int is a placeholder that unifies
            // with whatever other rows establish (see unify()).
            Ok(Some((child.unwrap_or(Kind::Int), true)))
        }
        Value::Node(_) | Value::Edge(_) | Value::Path(_) | Value::Map(_) => Err(type_error(
            column,
            "node/edge/path/map values -- project scalar properties instead of whole entities",
        )),
    }
}

fn unify(column: &str, a: ColumnShape, b: (Kind, bool)) -> Result<ColumnShape, Error> {
    let (kind, is_list) = b;
    let incoming = if is_list {
        ColumnShape::List(kind)
    } else {
        ColumnShape::Scalar(kind)
    };
    match (a, incoming) {
        (ColumnShape::AllNull, shape) => Ok(shape),
        (shape, ColumnShape::AllNull) => Ok(shape),
        (ColumnShape::Scalar(x), ColumnShape::Scalar(y)) if x == y => Ok(a),
        (ColumnShape::List(x), ColumnShape::List(y)) if x == y => Ok(a),
        // An empty list carried a placeholder Int child; a list column
        // unifies with any established list child in either direction.
        (ColumnShape::List(_), ColumnShape::List(_)) => {
            Err(type_error(column, "mixed list element types across rows"))
        }
        (ColumnShape::Scalar(Kind::Int), ColumnShape::Scalar(Kind::Float))
        | (ColumnShape::Scalar(Kind::Float), ColumnShape::Scalar(Kind::Int)) => Err(type_error(
            column,
            "mixed Int and Float values -- exporting both as Float64 would silently lose \
             precision for integers beyond 2^53; cast explicitly in the query (toFloat/toInteger)",
        )),
        (x, y) => Err(type_error(
            column,
            format!("mixed value shapes ({x:?} and {y:?})"),
        )),
    }
}

/// True list-child unification, aware of the empty-list placeholder:
/// prefer whichever side has real evidence.
fn unify_list_children(
    column: &str,
    shapes: &mut [ColumnShape],
    col: usize,
    kind: Kind,
    had_elems: bool,
) -> Result<(), Error> {
    match shapes[col] {
        ColumnShape::AllNull => {
            shapes[col] = ColumnShape::List(kind);
            Ok(())
        }
        ColumnShape::List(existing) => {
            if existing == kind || !had_elems {
                Ok(())
            } else if matches!(existing, Kind::Int) && had_elems {
                // Existing might itself be a placeholder from an
                // earlier empty list -- overwrite with real evidence.
                shapes[col] = ColumnShape::List(kind);
                Ok(())
            } else {
                Err(type_error(column, "mixed list element types across rows"))
            }
        }
        ColumnShape::Scalar(_) => Err(type_error(column, "mix of list and scalar values")),
    }
}

fn infer_shapes(result: &QueryResult) -> Result<Vec<ColumnShape>, Error> {
    let mut shapes = vec![ColumnShape::AllNull; result.columns.len()];
    for row in &result.rows {
        for (col, value) in row.iter().enumerate() {
            let column = &result.columns[col];
            let Some((kind, is_list)) = value_kind(column, value)? else {
                continue;
            };
            if is_list {
                let had_elems = match value {
                    Value::List(items) => items.iter().any(|i| !matches!(i, Value::Null)),
                    _ => false,
                };
                unify_list_children(column, &mut shapes, col, kind, had_elems)?;
            } else {
                shapes[col] = unify(column, shapes[col], (kind, false))?;
            }
        }
    }
    Ok(shapes)
}

fn shape_field(name: &str, shape: ColumnShape) -> Field {
    match shape {
        ColumnShape::AllNull => Field::new(name, DataType::Null, true),
        ColumnShape::Scalar(kind) => Field::new(name, kind.data_type(), true),
        ColumnShape::List(kind) => Field::new(
            name,
            DataType::List(Arc::new(Field::new("item", kind.data_type(), true))),
            true,
        ),
    }
}

/// The scalar payload of a value, normalized to `PropertyValue`-ish
/// access; `None` = null. Callers only reach this after inference, so
/// entity variants are unreachable.
fn scalar_of(value: &Value) -> Option<PropertyValue> {
    match value {
        Value::Null | Value::Literal(Literal::Null) | Value::Literal(Literal::Param(_)) => None,
        Value::Literal(Literal::Int(i)) => Some(PropertyValue::Int(*i)),
        Value::Literal(Literal::Float(f)) => Some(PropertyValue::Float(*f)),
        Value::Literal(Literal::String(s)) => Some(PropertyValue::String(s.clone())),
        Value::Literal(Literal::Bool(b)) => Some(PropertyValue::Bool(*b)),
        Value::Property(PropertyValue::Null) => None,
        Value::Property(p) => Some(p.clone()),
        _ => unreachable!("inference rejected entity values before building"),
    }
}

fn iso_text(p: &PropertyValue) -> String {
    match p {
        PropertyValue::LocalTime(nanos_of_day) => crate::temporal::format_local_time(*nanos_of_day),
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => crate::temporal::format_time(*nanos_of_day, *offset_seconds),
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => crate::temporal::format_local_date_time(*epoch_seconds, *nanos),
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            zone,
        } => crate::temporal::format_date_time(*epoch_seconds, *nanos, &to_temporal_tz(zone)),
        PropertyValue::String(s) => s.clone(),
        other => unreachable!("iso_text only for temporal/text kinds, got {other:?}"),
    }
}

/// Map a stored duration onto Arrow's `IntervalMonthDayNano` fields --
/// months/days narrow i64 -> i32 and seconds fold into the nanosecond
/// field, each with a typed error on overflow rather than wrapping.
fn interval_fields(
    column: &str,
    months: i64,
    days: i64,
    seconds: i64,
    nanos: i32,
) -> Result<(i32, i32, i64), Error> {
    let m = i32::try_from(months)
        .map_err(|_| type_error(column, "duration months exceed Arrow's i32 field"))?;
    let d = i32::try_from(days)
        .map_err(|_| type_error(column, "duration days exceed Arrow's i32 field"))?;
    let total = seconds
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(i64::from(nanos)))
        .ok_or_else(|| type_error(column, "duration seconds overflow Arrow's nanosecond field"))?;
    Ok((m, d, total))
}

fn to_temporal_tz(zone: &marsdb_graph::TzId) -> crate::temporal::TzId {
    match zone {
        marsdb_graph::TzId::Offset(o) => crate::temporal::TzId::Offset(*o),
        marsdb_graph::TzId::Named(name) => crate::temporal::TzId::Named(name.clone()),
    }
}

enum AnyBuilder {
    Int(Int64Builder),
    Float(Float64Builder),
    Text(StringBuilder),
    Bool(BooleanBuilder),
    Date(Date32Builder),
    Duration(IntervalMonthDayNanoBuilder),
}

impl AnyBuilder {
    fn new(kind: Kind) -> Self {
        match kind {
            Kind::Int => AnyBuilder::Int(Int64Builder::new()),
            Kind::Float => AnyBuilder::Float(Float64Builder::new()),
            Kind::Text | Kind::IsoText => AnyBuilder::Text(StringBuilder::new()),
            Kind::Bool => AnyBuilder::Bool(BooleanBuilder::new()),
            Kind::Date => AnyBuilder::Date(Date32Builder::new()),
            Kind::Duration => AnyBuilder::Duration(IntervalMonthDayNanoBuilder::new()),
        }
    }
    fn finish(self) -> ArrayRef {
        match self {
            AnyBuilder::Int(mut b) => Arc::new(b.finish()),
            AnyBuilder::Float(mut b) => Arc::new(b.finish()),
            AnyBuilder::Text(mut b) => Arc::new(b.finish()),
            AnyBuilder::Bool(mut b) => Arc::new(b.finish()),
            AnyBuilder::Date(mut b) => Arc::new(b.finish()),
            AnyBuilder::Duration(mut b) => Arc::new(b.finish()),
        }
    }
}

fn append_scalar(
    builder: &mut AnyBuilder,
    column: &str,
    scalar: Option<&PropertyValue>,
) -> Result<(), Error> {
    match builder {
        AnyBuilder::Int(b) => match scalar {
            None => b.append_null(),
            Some(PropertyValue::Int(i)) => b.append_value(*i),
            Some(other) => {
                return Err(type_error(
                    column,
                    format!("expected integer, got {other:?}"),
                ))
            }
        },
        AnyBuilder::Float(b) => match scalar {
            None => b.append_null(),
            Some(PropertyValue::Float(f)) => b.append_value(*f),
            Some(other) => {
                return Err(type_error(column, format!("expected float, got {other:?}")))
            }
        },
        AnyBuilder::Bool(b) => match scalar {
            None => b.append_null(),
            Some(PropertyValue::Bool(v)) => b.append_value(*v),
            Some(other) => {
                return Err(type_error(
                    column,
                    format!("expected boolean, got {other:?}"),
                ))
            }
        },
        AnyBuilder::Date(b) => match scalar {
            None => b.append_null(),
            Some(PropertyValue::Date(days)) => {
                let days32 = i32::try_from(*days).map_err(|_| {
                    type_error(
                        column,
                        format!("date {days} epoch-days exceeds Arrow Date32's i32 range"),
                    )
                })?;
                b.append_value(days32);
            }
            Some(other) => return Err(type_error(column, format!("expected date, got {other:?}"))),
        },
        AnyBuilder::Duration(b) => match scalar {
            None => b.append_null(),
            Some(PropertyValue::Duration {
                months,
                days,
                seconds,
                nanos,
            }) => {
                let (m, d, total_nanos) =
                    interval_fields(column, *months, *days, *seconds, *nanos)?;
                b.append_value(IntervalMonthDayNano::new(m, d, total_nanos));
            }
            Some(other) => {
                return Err(type_error(
                    column,
                    format!("expected duration, got {other:?}"),
                ))
            }
        },
        AnyBuilder::Text(b) => match scalar {
            None => b.append_null(),
            Some(
                p @ (PropertyValue::String(_)
                | PropertyValue::LocalTime(_)
                | PropertyValue::Time { .. }
                | PropertyValue::LocalDateTime { .. }
                | PropertyValue::DateTime { .. }),
            ) => b.append_value(iso_text(p)),
            Some(other) => {
                return Err(type_error(
                    column,
                    format!("expected string, got {other:?}"),
                ))
            }
        },
    }
    Ok(())
}

fn build_column(
    result: &QueryResult,
    col: usize,
    rows: std::ops::Range<usize>,
    shape: ColumnShape,
) -> Result<ArrayRef, Error> {
    let column = &result.columns[col];
    match shape {
        ColumnShape::AllNull => Ok(Arc::new(NullArray::new(rows.len()))),
        ColumnShape::Scalar(kind) => {
            let mut builder = AnyBuilder::new(kind);
            for r in rows {
                let scalar = scalar_of(&result.rows[r][col]);
                append_scalar(&mut builder, column, scalar.as_ref())?;
            }
            Ok(builder.finish())
        }
        ColumnShape::List(kind) => {
            // ListBuilder over the matching scalar builder; nulls at
            // the list level for null rows.
            let mut lb = ListBuilder::new(AnyBuilderList::new(kind));
            for r in rows {
                match &result.rows[r][col] {
                    Value::Null | Value::Literal(Literal::Null) => lb.append_null(),
                    Value::Property(PropertyValue::Null) => lb.append_null(),
                    Value::List(items) => {
                        for item in items {
                            lb.values().append(column, item, kind)?;
                        }
                        lb.append(true);
                    }
                    Value::Property(PropertyValue::List(items)) => {
                        for item in items {
                            lb.values().append_prop(column, item, kind)?;
                        }
                        lb.append(true);
                    }
                    other => {
                        return Err(type_error(column, format!("expected list, got {other:?}")))
                    }
                }
            }
            Ok(Arc::new(lb.finish()))
        }
    }
}

/// A dynamically-typed child builder for lists -- wraps the same kinds
/// as `AnyBuilder` behind the `ArrayBuilder` object-safety `ListBuilder`
/// needs.
struct AnyBuilderList {
    kind: Kind,
    inner: Box<dyn arrow_array::builder::ArrayBuilder>,
}

impl AnyBuilderList {
    fn new(kind: Kind) -> Self {
        let inner: Box<dyn arrow_array::builder::ArrayBuilder> = match kind {
            Kind::Int => Box::new(Int64Builder::new()),
            Kind::Float => Box::new(Float64Builder::new()),
            Kind::Text | Kind::IsoText => Box::new(StringBuilder::new()),
            Kind::Bool => Box::new(BooleanBuilder::new()),
            Kind::Date => Box::new(Date32Builder::new()),
            Kind::Duration => Box::new(IntervalMonthDayNanoBuilder::new()),
        };
        Self { kind, inner }
    }

    fn append(&mut self, column: &str, item: &Value, kind: Kind) -> Result<(), Error> {
        let scalar = scalar_of(item);
        self.append_scalar_inner(column, scalar.as_ref(), kind)
    }

    fn append_prop(&mut self, column: &str, item: &PropertyValue, kind: Kind) -> Result<(), Error> {
        let scalar = if matches!(item, PropertyValue::Null) {
            None
        } else {
            Some(item.clone())
        };
        self.append_scalar_inner(column, scalar.as_ref(), kind)
    }

    fn append_scalar_inner(
        &mut self,
        column: &str,
        scalar: Option<&PropertyValue>,
        _kind: Kind,
    ) -> Result<(), Error> {
        let any = self.inner.as_any_mut();
        match self.kind {
            Kind::Int => {
                let b = any.downcast_mut::<Int64Builder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(PropertyValue::Int(i)) => b.append_value(*i),
                    Some(other) => {
                        return Err(type_error(
                            column,
                            format!("expected integer, got {other:?}"),
                        ))
                    }
                }
            }
            Kind::Float => {
                let b = any.downcast_mut::<Float64Builder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(PropertyValue::Float(f)) => b.append_value(*f),
                    Some(other) => {
                        return Err(type_error(column, format!("expected float, got {other:?}")))
                    }
                }
            }
            Kind::Bool => {
                let b = any.downcast_mut::<BooleanBuilder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(PropertyValue::Bool(v)) => b.append_value(*v),
                    Some(other) => {
                        return Err(type_error(
                            column,
                            format!("expected boolean, got {other:?}"),
                        ))
                    }
                }
            }
            Kind::Date => {
                let b = any.downcast_mut::<Date32Builder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(PropertyValue::Date(days)) => {
                        let days32 = i32::try_from(*days).map_err(|_| {
                            type_error(column, "date exceeds Arrow Date32's i32 range")
                        })?;
                        b.append_value(days32);
                    }
                    Some(other) => {
                        return Err(type_error(column, format!("expected date, got {other:?}")))
                    }
                }
            }
            Kind::Duration => {
                let b = any.downcast_mut::<IntervalMonthDayNanoBuilder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(PropertyValue::Duration {
                        months,
                        days,
                        seconds,
                        nanos,
                    }) => {
                        let (m, d, total) =
                            interval_fields(column, *months, *days, *seconds, *nanos)?;
                        b.append_value(IntervalMonthDayNano::new(m, d, total));
                    }
                    Some(other) => {
                        return Err(type_error(
                            column,
                            format!("expected duration, got {other:?}"),
                        ))
                    }
                }
            }
            Kind::Text | Kind::IsoText => {
                let b = any.downcast_mut::<StringBuilder>().unwrap();
                match scalar {
                    None => b.append_null(),
                    Some(
                        p @ (PropertyValue::String(_)
                        | PropertyValue::LocalTime(_)
                        | PropertyValue::Time { .. }
                        | PropertyValue::LocalDateTime { .. }
                        | PropertyValue::DateTime { .. }),
                    ) => b.append_value(iso_text(p)),
                    Some(other) => {
                        return Err(type_error(
                            column,
                            format!("expected string, got {other:?}"),
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}

impl arrow_array::builder::ArrayBuilder for AnyBuilderList {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn finish(&mut self) -> ArrayRef {
        self.inner.finish()
    }
    fn finish_cloned(&self) -> ArrayRef {
        self.inner.finish_cloned()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn into_box_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/// Infer the Arrow schema a `QueryResult` would export as, without
/// building any arrays. Fails with the same typed errors as the full
/// export.
pub fn infer_schema(result: &QueryResult) -> Result<Schema, Error> {
    let shapes = infer_shapes(result)?;
    Ok(Schema::new(
        result
            .columns
            .iter()
            .zip(&shapes)
            .map(|(name, shape)| shape_field(name, *shape))
            .collect::<Vec<_>>(),
    ))
}

/// Transpose a materialized result into Arrow batches of `batch_rows`.
pub fn to_record_batches(
    result: &QueryResult,
    batch_rows: usize,
) -> Result<(SchemaRef, Vec<RecordBatch>), Error> {
    let shapes = infer_shapes(result)?;
    let schema: SchemaRef = Arc::new(Schema::new(
        result
            .columns
            .iter()
            .zip(&shapes)
            .map(|(name, shape)| shape_field(name, *shape))
            .collect::<Vec<_>>(),
    ));
    let batch_rows = batch_rows.max(1);
    let mut batches = Vec::new();
    let mut start = 0usize;
    while start < result.rows.len() {
        let end = (start + batch_rows).min(result.rows.len());
        let columns: Vec<ArrayRef> = (0..result.columns.len())
            .map(|col| build_column(result, col, start..end, shapes[col]))
            .collect::<Result<_, _>>()?;
        let batch = RecordBatch::try_new(schema.clone(), columns)
            .map_err(|e| Error::Query(QueryError::Type(format!("arrow batch: {e}"))))?;
        batches.push(batch);
        start = end;
    }
    Ok((schema, batches))
}

/// A finished Arrow export: schema + batches, iterable as a
/// `RecordBatchReader` (the trait Arrow's FFI stream export and every
/// Rust Arrow consumer accept).
pub struct ArrowResult {
    schema: SchemaRef,
    batches: std::vec::IntoIter<RecordBatch>,
    /// The statement's write counters, carried alongside (Arrow has no
    /// slot for them).
    pub stats: crate::QueryStats,
}

impl ArrowResult {
    /// Convert an already-executed result. `Database::query_arrow` is
    /// the one-call form; this exists for callers that run statements
    /// through their own path (e.g. the C ABI's prepared statements).
    pub fn from_result(result: &QueryResult, batch_rows: usize) -> Result<Self, Error> {
        let (schema, batches) = to_record_batches(result, batch_rows)?;
        Ok(ArrowResult {
            schema,
            batches: batches.into_iter(),
            stats: result.stats,
        })
    }
}

impl Iterator for ArrowResult {
    type Item = Result<RecordBatch, ArrowError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.batches.next().map(Ok)
    }
}

impl RecordBatchReader for ArrowResult {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}

impl crate::Database {
    /// Execute one statement and export the result as Arrow — see this
    /// module's docs for the inference rules and the (single,
    /// core-side) transpose this implies on v1's row-oriented storage.
    /// The returned reader plugs into anything accepting
    /// `RecordBatchReader`, including the C Data Interface stream
    /// export the C ABI and Python PyCapsule protocol build on.
    pub fn query_arrow(
        &self,
        cypher: &str,
        params: &std::collections::HashMap<String, PropertyValue>,
        options: &crate::ExecutionOptions,
        batch_rows: usize,
    ) -> Result<ArrowResult, Error> {
        let result = self.execute_with_params_and_options(cypher, params, options)?;
        ArrowResult::from_result(&result, batch_rows)
    }
}
