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
/// distinction to survive the storage boundary, not just live in the
/// query layer's own `Value` the way `Value::List` does -- see that
/// type's own doc comment for the contrasting case). MarsDB does *not*
/// yet have `LOCAL TIME`/`TIME`/`LOCAL DATETIME`/`DATETIME` variants (no
/// time-of-day, no timezone) -- see the README's "Cypher coverage"
/// section for exactly what that leaves unsupported.
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

impl AdjEntry {
    pub(crate) fn encode(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0..8].copy_from_slice(&self.edge_id.0.to_be_bytes());
        buf[8..16].copy_from_slice(&self.other.0.to_be_bytes());
        buf[16..20].copy_from_slice(&self.label_id.to_be_bytes());
        buf
    }

    pub(crate) fn decode(bytes: &[u8]) -> Self {
        let edge_id = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let other = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let label_id = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        AdjEntry {
            edge_id: EdgeId(edge_id),
            other: NodeId(other),
            label_id,
        }
    }
}
