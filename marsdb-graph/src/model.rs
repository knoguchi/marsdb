use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub u64);

/// A node/edge property, as persisted to redb (via `postcard`, see
/// `encode.rs`) and used directly as MarsDB's runtime scalar type -- there
/// is no separate "wire" representation. New variants append at the end
/// (postcard's derive encodes an enum discriminant by declaration order),
/// never reorder/remove an existing one, or every already-stored property
/// silently decodes as the wrong variant.
///
/// `Date`/`Duration` are Cypher's `DATE`/`DURATION` temporal types, added
/// as first-class variants rather than reusing `Int`/`String` -- e.g.
/// stashing a date as `Int(epoch_day)` would round-trip through storage
/// fine, but a plain `Int` and a `Date` would then be indistinguishable
/// once read back (Temporal4's "store a date, read it back, it must
/// still print/compare/access-components as a date" scenarios need that
/// distinction to survive the storage boundary). `LocalTime`/`Time`/
/// `LocalDateTime`/`DateTime` (Cypher's other four temporal types) follow
/// the same reasoning below. `Time` only accepts a *fixed* UTC offset --
/// it carries no calendar date, so a named zone's DST-dependent offset
/// has nothing to resolve against; `DateTime` accepts either a fixed
/// offset or a named zone (`TzId`).
///
/// `Map` exists here for exactly one reason: a `$parameter`'s value can be
/// map-shaped (`{name: 'Apa'}`, TCK's Map2/Map3), and query-time
/// parameters flow in as `PropertyValue` (this is the one place a
/// non-storable shape has to travel through). A node/edge *property*
/// value is never actually a `Map` though -- real Cypher forbids storing
/// one (`marsdb-query::executor::value_to_storable_property` rejects it
/// outright before anything reaches `GraphStore`), so this variant is
/// only ever constructed on the parameter-passing path, never persisted.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    /// A calendar date with no time-of-day or timezone, stored as the
    /// number of days since the Unix epoch (1970-01-01), proleptic
    /// Gregorian. Plain `i32` (not a `chrono` type) -- keeps this crate's
    /// storage format independent of any date library's own internal
    /// representation (which is free to change across `chrono` versions),
    /// and keeps comparison a plain integer compare. Conversion to/from
    /// calendar year/month/day and ISO-8601 text lives in `marsdb-query`
    /// (`temporal.rs`), not here -- this crate only stores the value, it
    /// doesn't know Cypher's date grammar/semantics.
    Date(i32),
    /// An ISO-8601 duration (Cypher's `DURATION` type), kept in Neo4j's
    /// own four-component normalized form rather than as a single scalar
    /// -- months and days are *not* fungible with each other or with
    /// seconds (a month is 28-31 days depending which month; without a
    /// reference date, "3 months" has no fixed length in days at all), so
    /// collapsing `duration({months: 1})` and `duration({days: 30})` into
    /// one comparable number would silently be wrong once added to some
    /// starting date. `nanos` always has the same sign as `seconds` (or is
    /// `0`) -- i.e. `seconds*1_000_000_000 + nanos` is `total_nanoseconds`
    /// truncated-towards-zero the same way Rust's integer division/`%`
    /// already works, never a separately-signed remainder -- so
    /// "-1.999 seconds" is `seconds: -1, nanos: -999_000_000`, not
    /// `seconds: -2, nanos: 1_000_000`, which would make the same
    /// duration representable two different ways.
    Duration {
        months: i64,
        days: i64,
        seconds: i64,
        nanos: i32,
    },
    /// A time-of-day with no date or timezone, stored as nanoseconds since
    /// midnight (`0..86_400_000_000_000`, always non-negative -- there's no
    /// sign to carry the way `Date`'s epoch-day has). Cypher's `LOCAL TIME`.
    LocalTime(i64),
    /// A time-of-day with a *fixed* UTC offset (Cypher's `TIME`) -- named
    /// timezones (`Europe/Stockholm`) aren't supported, only literal
    /// `+HH:MM`-style offsets (see `marsdb-query::temporal`'s module docs
    /// for the exact scope). `nanos_of_day` is the wall-clock reading (same
    /// representation as `LocalTime`); `offset_seconds` is seconds *east*
    /// of UTC. Comparison/equality use the UTC-equivalent instant-of-day
    /// (`nanos_of_day - offset_seconds`), not the raw wall-clock reading --
    /// two `Time`s at different offsets can represent the same instant.
    Time {
        nanos_of_day: i64,
        offset_seconds: i32,
    },
    /// A calendar date + time-of-day with no timezone (Cypher's `LOCAL
    /// DATETIME`), stored as a naive (zone-less) instant: whole seconds
    /// since the Unix epoch (`epoch_seconds`, signed -- a pre-1970 value is
    /// negative) plus a `0..999_999_999` nanosecond remainder that always
    /// stays non-negative (the sign lives entirely in `epoch_seconds`,
    /// mirroring `Duration`'s "no separately-signed remainder" invariant).
    LocalDateTime {
        epoch_seconds: i64,
        nanos: i32,
    },
    /// A calendar date + time-of-day with a timezone (Cypher's
    /// `DATETIME`) -- either a *fixed* UTC offset or a named IANA zone
    /// (`Europe/Stockholm`). `epoch_seconds`/`nanos` are the *UTC
    /// instant* (same convention as `LocalDateTime`); `zone` is kept
    /// only for display/round-tripping the original wall-clock reading
    /// -- comparison/equality use the instant alone, matching real
    /// Cypher (two `DateTime`s at the same instant but different zones
    /// are equal, even though they print differently). A `Named` zone's
    /// real offset at this instant is *not* cached here (the same zone
    /// has different offsets across a DST transition) -- it's re-derived
    /// on demand via `chrono-tz` (`marsdb-query::temporal::resolve_
    /// offset`), this crate only stores the value, it doesn't know
    /// Cypher's timezone-resolution semantics.
    DateTime {
        epoch_seconds: i64,
        nanos: i32,
        zone: TzId,
    },
    /// A homogeneous array of scalars (real Cypher/Neo4j's own property
    /// restriction: a stored list property can hold any of the scalar
    /// variants above, all the same variant, never `Null`-mixed-with-a-
    /// type, another `List`, or a map -- enforced where a `Value::List`
    /// is converted to a storable `PropertyValue`, in `marsdb-query`, not
    /// here; this crate just stores whatever `Vec<PropertyValue>` it's
    /// given). Appended last (see this enum's own doc comment on why
    /// variant order is a real, one-way storage-compat constraint).
    List(Vec<PropertyValue>),
    /// See this enum's own doc comment -- parameter-passing only, never a
    /// real stored property value.
    Map(BTreeMap<String, PropertyValue>),
}

/// A `DateTime`'s zone: a fixed UTC offset, or a named IANA timezone
/// (`Europe/Stockholm`) whose real offset varies by instant (DST) and is
/// resolved on demand, not stored -- see `PropertyValue::DateTime`'s doc
/// comment.
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

/// Composite-key layout for `ADJ_OUT`/`ADJ_IN` (v1.5 step 2):
/// `owner_node(8B BE) ++ label_id(4B BE) ++ edge_id(8B BE)` -> other
/// node id as the value. All-BE so plain byte order sorts by (owner,
/// label, edge) — one node's edges are contiguous, and label-typed
/// expansion is a sub-prefix range within them. `AdjEntry` itself stays
/// the in-memory traversal-candidate type; only its storage layout moved
/// from a 20-byte multimap *value* to this key shape.
pub(crate) fn adj_key(owner: u64, label_id: u32, edge_id: u64) -> [u8; 20] {
    let mut buf = [0u8; 20];
    buf[0..8].copy_from_slice(&owner.to_be_bytes());
    buf[8..12].copy_from_slice(&label_id.to_be_bytes());
    buf[12..20].copy_from_slice(&edge_id.to_be_bytes());
    buf
}

/// Inclusive key bounds covering every adjacency entry a node owns,
/// any label — the untyped-expansion prefix.
pub(crate) fn adj_node_bounds(owner: u64) -> ([u8; 20], [u8; 20]) {
    (adj_key(owner, 0, 0), adj_key(owner, u32::MAX, u64::MAX))
}

/// Inclusive key bounds covering one node's entries under one label —
/// the typed-expansion prefix (`O(matching degree)`).
pub(crate) fn adj_label_bounds(owner: u64, label_id: u32) -> ([u8; 20], [u8; 20]) {
    (
        adj_key(owner, label_id, 0),
        adj_key(owner, label_id, u64::MAX),
    )
}

/// Parse a stored adjacency key back into `(owner, label_id, edge_id)`.
pub(crate) fn parse_adj_key(bytes: &[u8]) -> Result<(u64, u32, u64), crate::GraphError> {
    let bytes: &[u8; 20] = bytes.try_into().map_err(|_| {
        crate::GraphError::CorruptData(format!(
            "adjacency key has {} bytes; expected 20",
            bytes.len()
        ))
    })?;
    Ok((
        u64::from_be_bytes(bytes[0..8].try_into().expect("fixed-size slice")),
        u32::from_be_bytes(bytes[8..12].try_into().expect("fixed-size slice")),
        u64::from_be_bytes(bytes[12..20].try_into().expect("fixed-size slice")),
    ))
}
