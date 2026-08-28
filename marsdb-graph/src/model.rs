use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub u64);

/// A node/edge property, persisted via `postcard` (see `encode.rs`) and
/// used directly as MarsDB's runtime scalar type. New variants must append
/// at the end (postcard encodes the enum discriminant by declaration
/// order); reordering or removing one makes every already-stored property
/// decode as the wrong variant.
///
/// `Date`/`Duration`/`LocalTime`/`Time`/`LocalDateTime`/`DateTime` are
/// Cypher's temporal types, each a distinct variant rather than reused
/// `Int`/`String` storage so a stored value round-trips as its own type
/// instead of colliding with a plain number. `Time` only carries a fixed
/// UTC offset (no calendar date to resolve a named zone's DST against);
/// `DateTime` accepts either a fixed offset or a named zone (`TzId`).
///
/// `Map` exists only because a `$parameter` value can be map-shaped
/// (`{name: 'Apa'}`); query-time parameters flow in as `PropertyValue`.
/// A real node/edge property is never a `Map` --
/// `marsdb-query::executor::value_to_storable_property` rejects that
/// before it reaches `GraphStore` -- so this variant is only ever
/// constructed on the parameter-passing path.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    /// A calendar date with no time-of-day or timezone: days since the
    /// Unix epoch (1970-01-01), proleptic Gregorian. Plain `i64` (not a
    /// `chrono` type, and not `i32`) so storage stays independent of any
    /// date library's representation and comparison is a plain integer
    /// compare; `i64` because Cypher's expanded year range (±999_999_999)
    /// reaches ±365 billion epoch days, past `i32`. Calendar conversion
    /// and ISO-8601 text live in `marsdb-query::temporal`, not here.
    Date(i64),
    /// Cypher's `DURATION`, kept as four independent components rather
    /// than one scalar: months and days aren't fungible with seconds or
    /// each other (a month's length in days depends which month), so
    /// collapsing them would silently misconvert once added to a date.
    /// `nanos` always shares `seconds`'s sign (or is `0`): "-1.999s" is
    /// `seconds: -1, nanos: -999_000_000`, never `seconds: -2, nanos:
    /// 1_000_000` -- one representation per duration.
    Duration {
        months: i64,
        days: i64,
        seconds: i64,
        nanos: i32,
    },
    /// Cypher's `LOCAL TIME`: nanoseconds since midnight
    /// (`0..86_400_000_000_000`, always non-negative).
    LocalTime(i64),
    /// Cypher's `TIME`: a fixed UTC offset only, no named timezone.
    /// `nanos_of_day` is the wall-clock reading; `offset_seconds` is
    /// seconds east of UTC. Comparison/equality use the UTC-equivalent
    /// instant (`nanos_of_day - offset_seconds`), not the raw wall-clock
    /// value, so two `Time`s at different offsets can be equal.
    Time {
        nanos_of_day: i64,
        offset_seconds: i32,
    },
    /// Cypher's `LOCAL DATETIME`: a naive (zone-less) instant as whole
    /// seconds since the Unix epoch (`epoch_seconds`, signed) plus a
    /// `0..999_999_999` nanosecond remainder that stays non-negative --
    /// the sign lives entirely in `epoch_seconds`.
    LocalDateTime {
        epoch_seconds: i64,
        nanos: i32,
    },
    /// Cypher's `DATETIME`: a fixed UTC offset or a named IANA zone.
    /// `epoch_seconds`/`nanos` are the UTC instant; `zone` is kept only
    /// for display -- comparison/equality use the instant alone, so two
    /// `DateTime`s at the same instant but different zones are equal. A
    /// named zone's real offset is not cached (it varies across DST
    /// transitions) and is re-derived on demand via `chrono-tz` in
    /// `marsdb-query::temporal::resolve_offset`.
    DateTime {
        epoch_seconds: i64,
        nanos: i32,
        zone: TzId,
    },
    /// A homogeneous scalar array: Cypher forbids mixing types, `Null`,
    /// nested `List`s, or maps inside a stored list, enforced where a
    /// `Value::List` converts to `PropertyValue` in `marsdb-query`, not
    /// here. Appended last -- see this enum's own doc comment on why
    /// variant order is a storage-compat constraint.
    List(Vec<PropertyValue>),
    /// Parameter-passing only, never a real stored property value -- see
    /// this enum's own doc comment.
    Map(BTreeMap<String, PropertyValue>),
}

/// A `DateTime`'s zone: a fixed UTC offset, or a named IANA timezone whose
/// real offset is resolved on demand, not stored -- see
/// `PropertyValue::DateTime`'s doc comment.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TzId {
    Offset(i32),
    Named(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub labels: Vec<String>,
    pub props: BTreeMap<String, PropertyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub label: String,
    pub src: NodeId,
    pub dst: NodeId,
    pub props: BTreeMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Out,
    In,
}

/// A traversal-hop candidate read directly from an adjacency multimap entry,
/// without touching the `edges`/`nodes` tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjEntry {
    pub edge_id: EdgeId,
    pub other: NodeId,
    pub label_id: u32,
}

/// `ADJ_OUT`/`ADJ_IN` key: `(owner_node, label_id, edge_id)` tuple ->
/// other node id as the value. redb orders a fixed-width tuple
/// component-wise, so one node's edges are contiguous and a label-typed
/// expansion is a sub-prefix range within them.
pub(crate) type AdjKey = (u64, u32, u64);

pub(crate) fn adj_key(owner: u64, label_id: u32, edge_id: u64) -> AdjKey {
    (owner, label_id, edge_id)
}

/// Inclusive key bounds covering every adjacency entry a node owns,
/// any label — the untyped-expansion prefix.
pub(crate) fn adj_node_bounds(owner: u64) -> (AdjKey, AdjKey) {
    ((owner, 0, 0), (owner, u32::MAX, u64::MAX))
}

/// Inclusive key bounds covering one node's entries under one label —
/// the typed-expansion prefix (`O(matching degree)`).
pub(crate) fn adj_label_bounds(owner: u64, label_id: u32) -> (AdjKey, AdjKey) {
    ((owner, label_id, 0), (owner, label_id, u64::MAX))
}
