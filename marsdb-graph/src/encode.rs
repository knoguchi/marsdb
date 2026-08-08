//! On-disk record encoding: directory format (v1.5, format version 2).
//!
//! Replaces the v1 whole-blob postcard encoding (`BTreeMap<String,
//! PropertyValue>` serialized as one unit) with a directory layout using
//! interned `u32` prop-id keys and per-property offsets:
//!
//! ```text
//! node:  [label_count: u8][label_id: u32 LE × n]
//!        [prop_count: u16 LE]
//!        [(prop_id: u32 LE, offset: u32 LE) × m]     <- sorted by prop_id
//!        [values: postcard-encoded PropertyValue, packed in dir order]
//! edge:  [label_id: u32 LE][src: u64 LE][dst: u64 LE]
//!        [prop_count/directory/values as above]
//! ```
//!
//! `offset` is relative to the start of the values region; value `i`'s
//! length is `offset[i+1] - offset[i]` (last value runs to the end), which
//! is why values must be packed in directory order. Reading one property is
//! a binary search over the directory plus one postcard decode of just that
//! value — no `BTreeMap` build, no per-property string allocation, no
//! touching sibling properties (measured 79x @ 1-of-20 props, 7x even at
//! full materialization, `marsdb-storage/examples/v1_vs_v2_bench.rs`).
//! Property *names* appear nowhere in the record: encode interns them to
//! ids (`props::intern_prop`), decode resolves ids back only when a caller
//! actually needs the name-keyed `NodeRecord`/`EdgeRecord` shape.
//!
//! Encode/decode take interning/resolution as closures rather than a
//! transaction type: write paths intern via `&mut WriteCtx`, read paths
//! resolve via `Txn` — same functions serve both without this module
//! depending on either.

use std::collections::BTreeMap;

use crate::error::GraphError;
use crate::model::PropertyValue;

pub(crate) struct NodeRecord {
    pub label_ids: Vec<u32>,
    pub props: BTreeMap<String, PropertyValue>,
}

pub(crate) struct EdgeRecord {
    pub label_id: u32,
    pub src: u64,
    pub dst: u64,
    pub props: BTreeMap<String, PropertyValue>,
}

const DIR_ENTRY_SIZE: usize = 4 + 4; // prop_id + offset

fn corrupt(what: &str) -> GraphError {
    GraphError::CorruptData(format!("record decode: {what}"))
}

/// Encode the shared `[prop_count][directory][values]` tail. Props are
/// sorted by their interned id (not by name) so the directory is
/// binary-searchable; values are packed in that same order so each value's
/// length is recoverable from the next entry's offset.
fn encode_props(
    out: &mut Vec<u8>,
    props: &BTreeMap<String, PropertyValue>,
    mut intern: impl FnMut(&str) -> Result<u32, GraphError>,
) -> Result<(), GraphError> {
    let mut entries: Vec<(u32, &PropertyValue)> = props
        .iter()
        .map(|(name, value)| Ok((intern(name)?, value)))
        .collect::<Result<_, GraphError>>()?;
    entries.sort_unstable_by_key(|&(id, _)| id);

    let count = u16::try_from(entries.len())
        .map_err(|_| GraphError::CorruptData("more than 65535 properties on one record".into()))?;
    out.extend_from_slice(&count.to_le_bytes());

    let mut values = Vec::new();
    for (prop_id, value) in entries {
        let offset = u32::try_from(values.len())
            .map_err(|_| GraphError::CorruptData("record property data exceeds 4 GiB".into()))?;
        out.extend_from_slice(&prop_id.to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        values = postcard::to_extend(value, values)?;
    }
    out.extend_from_slice(&values);
    Ok(())
}

/// Decode the shared tail back into a name-keyed map. The inverse of
/// `encode_props` — resolves each prop id to its name, so this is the
/// full-materialization path; single-property access should use
/// `prop_raw_in` instead and skip names entirely.
fn decode_props(
    bytes: &[u8],
    mut resolve: impl FnMut(u32) -> Result<String, GraphError>,
) -> Result<BTreeMap<String, PropertyValue>, GraphError> {
    let count = u16::from_le_bytes(
        bytes
            .get(0..2)
            .ok_or_else(|| corrupt("missing prop count"))?
            .try_into()
            .unwrap(),
    ) as usize;
    let dir_end = 2 + count * DIR_ENTRY_SIZE;
    let dir = bytes
        .get(2..dir_end)
        .ok_or_else(|| corrupt("directory out of bounds"))?;
    let values = &bytes[dir_end..];

    let mut props = BTreeMap::new();
    for i in 0..count {
        let entry = &dir[i * DIR_ENTRY_SIZE..(i + 1) * DIR_ENTRY_SIZE];
        let prop_id = u32::from_le_bytes(entry[0..4].try_into().unwrap());
        let value_bytes = value_slice(dir, values, count, i)?;
        let value: PropertyValue = postcard::from_bytes(value_bytes)?;
        props.insert(resolve(prop_id)?, value);
    }
    Ok(props)
}

/// Value byte range for directory entry `i`, bounds-checked.
fn value_slice<'a>(
    dir: &[u8],
    values: &'a [u8],
    count: usize,
    i: usize,
) -> Result<&'a [u8], GraphError> {
    let offset_at = |j: usize| -> usize {
        u32::from_le_bytes(
            dir[j * DIR_ENTRY_SIZE + 4..j * DIR_ENTRY_SIZE + 8]
                .try_into()
                .unwrap(),
        ) as usize
    };
    let start = offset_at(i);
    let end = if i + 1 < count {
        offset_at(i + 1)
    } else {
        values.len()
    };
    values
        .get(start..end)
        .ok_or_else(|| corrupt("value offset out of bounds"))
}

/// Binary-search the tail for one property's raw (still postcard-encoded)
/// value bytes, without resolving any names or touching other values —
/// the per-property fast path (v1.5 step 1b's `get_node_prop_in_txn`).
fn prop_raw_in(bytes: &[u8], prop_id: u32) -> Result<Option<&[u8]>, GraphError> {
    let count = u16::from_le_bytes(
        bytes
            .get(0..2)
            .ok_or_else(|| corrupt("missing prop count"))?
            .try_into()
            .unwrap(),
    ) as usize;
    let dir_end = 2 + count * DIR_ENTRY_SIZE;
    let dir = bytes
        .get(2..dir_end)
        .ok_or_else(|| corrupt("directory out of bounds"))?;
    let values = &bytes[dir_end..];

    let mut lo = 0usize;
    let mut hi = count;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let entry_id = u32::from_le_bytes(
            dir[mid * DIR_ENTRY_SIZE..mid * DIR_ENTRY_SIZE + 4]
                .try_into()
                .unwrap(),
        );
        match entry_id.cmp(&prop_id) {
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Equal => return Ok(Some(value_slice(dir, values, count, mid)?)),
            std::cmp::Ordering::Greater => hi = mid,
        }
    }
    Ok(None)
}

pub(crate) fn encode_node(
    record: &NodeRecord,
    intern: impl FnMut(&str) -> Result<u32, GraphError>,
) -> Result<Vec<u8>, GraphError> {
    let label_count = u8::try_from(record.label_ids.len())
        .map_err(|_| GraphError::CorruptData("more than 255 labels on one node".into()))?;
    let mut out = Vec::with_capacity(1 + record.label_ids.len() * 4 + 2);
    out.push(label_count);
    for id in &record.label_ids {
        out.extend_from_slice(&id.to_le_bytes());
    }
    encode_props(&mut out, &record.props, intern)?;
    Ok(out)
}

/// Byte offset where a node record's props tail begins.
fn node_props_start(bytes: &[u8]) -> Result<usize, GraphError> {
    let label_count = *bytes.first().ok_or_else(|| corrupt("empty node record"))? as usize;
    Ok(1 + label_count * 4)
}

pub(crate) fn decode_node(
    bytes: &[u8],
    resolve: impl FnMut(u32) -> Result<String, GraphError>,
) -> Result<NodeRecord, GraphError> {
    let start = node_props_start(bytes)?;
    let label_count = bytes[0] as usize;
    let mut label_ids = Vec::with_capacity(label_count);
    for i in 0..label_count {
        label_ids.push(u32::from_le_bytes(
            bytes
                .get(1 + i * 4..1 + (i + 1) * 4)
                .ok_or_else(|| corrupt("label ids out of bounds"))?
                .try_into()
                .unwrap(),
        ));
    }
    let props = decode_props(
        bytes.get(start..).ok_or_else(|| corrupt("node tail"))?,
        resolve,
    )?;
    Ok(NodeRecord { label_ids, props })
}

/// Just the label ids from an encoded node record — for paths
/// (`check_integrity`, label maintenance) that never touch properties, so
/// they skip the directory walk and every name resolution entirely.
pub(crate) fn node_label_ids(bytes: &[u8]) -> Result<Vec<u32>, GraphError> {
    let label_count = *bytes.first().ok_or_else(|| corrupt("empty node record"))? as usize;
    let mut label_ids = Vec::with_capacity(label_count);
    for i in 0..label_count {
        label_ids.push(u32::from_le_bytes(
            bytes
                .get(1 + i * 4..1 + (i + 1) * 4)
                .ok_or_else(|| corrupt("label ids out of bounds"))?
                .try_into()
                .unwrap(),
        ));
    }
    Ok(label_ids)
}

/// One property's raw value bytes from an encoded node record, by interned
/// prop id — no name resolution, no map build. `None` = property absent.
pub(crate) fn node_prop_raw(bytes: &[u8], prop_id: u32) -> Result<Option<&[u8]>, GraphError> {
    let start = node_props_start(bytes)?;
    prop_raw_in(
        bytes.get(start..).ok_or_else(|| corrupt("node tail"))?,
        prop_id,
    )
}

/// Decode one raw value slice produced by `node_prop_raw`/`edge_prop_raw`.
pub(crate) fn decode_value(bytes: &[u8]) -> Result<PropertyValue, GraphError> {
    Ok(postcard::from_bytes(bytes)?)
}

const EDGE_HEADER: usize = 4 + 8 + 8; // label_id + src + dst

pub(crate) fn encode_edge(
    record: &EdgeRecord,
    intern: impl FnMut(&str) -> Result<u32, GraphError>,
) -> Result<Vec<u8>, GraphError> {
    let mut out = Vec::with_capacity(EDGE_HEADER + 2);
    out.extend_from_slice(&record.label_id.to_le_bytes());
    out.extend_from_slice(&record.src.to_le_bytes());
    out.extend_from_slice(&record.dst.to_le_bytes());
    encode_props(&mut out, &record.props, intern)?;
    Ok(out)
}

pub(crate) fn decode_edge(
    bytes: &[u8],
    resolve: impl FnMut(u32) -> Result<String, GraphError>,
) -> Result<EdgeRecord, GraphError> {
    let header = bytes
        .get(0..EDGE_HEADER)
        .ok_or_else(|| corrupt("edge header"))?;
    let props = decode_props(&bytes[EDGE_HEADER..], resolve)?;
    Ok(EdgeRecord {
        label_id: u32::from_le_bytes(header[0..4].try_into().unwrap()),
        src: u64::from_le_bytes(header[4..12].try_into().unwrap()),
        dst: u64::from_le_bytes(header[12..20].try_into().unwrap()),
        props,
    })
}

/// Just an edge record's fixed header `(label_id, src, dst)` — for paths
/// (adjacency maintenance on delete, `check_integrity`) that never touch
/// properties.
pub(crate) fn edge_header(bytes: &[u8]) -> Result<(u32, u64, u64), GraphError> {
    let header = bytes
        .get(0..EDGE_HEADER)
        .ok_or_else(|| corrupt("edge header"))?;
    Ok((
        u32::from_le_bytes(header[0..4].try_into().unwrap()),
        u64::from_le_bytes(header[4..12].try_into().unwrap()),
        u64::from_le_bytes(header[12..20].try_into().unwrap()),
    ))
}

/// Edge counterpart of `node_prop_raw`.
// Wired up by v1.5 step 1b (per-property executor read path) -- kept
// alongside `node_prop_raw` now so the two formats' accessors ship and
// get tested together.
#[allow(dead_code)]
pub(crate) fn edge_prop_raw(bytes: &[u8], prop_id: u32) -> Result<Option<&[u8]>, GraphError> {
    if bytes.len() < EDGE_HEADER {
        return Err(corrupt("edge header"));
    }
    prop_raw_in(&bytes[EDGE_HEADER..], prop_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Fake interner: names get sequential ids; resolution reverses it.
    /// Deliberately assigns ids in an order different from name sort order
    /// (reversed name order) so the tests prove the directory is sorted by
    /// id, not accidentally by name.
    struct FakeInterner {
        by_name: HashMap<String, u32>,
        by_id: HashMap<u32, String>,
    }

    impl FakeInterner {
        fn over(names: &[&str]) -> Self {
            let mut by_name = HashMap::new();
            let mut by_id = HashMap::new();
            for (i, name) in names.iter().rev().enumerate() {
                by_name.insert(name.to_string(), i as u32);
                by_id.insert(i as u32, name.to_string());
            }
            FakeInterner { by_name, by_id }
        }
        fn intern(&self, name: &str) -> Result<u32, GraphError> {
            Ok(self.by_name[name])
        }
        fn resolve(&self, id: u32) -> Result<String, GraphError> {
            Ok(self.by_id[&id].clone())
        }
    }

    fn sample_props() -> BTreeMap<String, PropertyValue> {
        let mut props = BTreeMap::new();
        props.insert("alpha".into(), PropertyValue::Int(42));
        props.insert("beta".into(), PropertyValue::String("hello".into()));
        props.insert(
            "gamma".into(),
            PropertyValue::List(vec![PropertyValue::Bool(true), PropertyValue::Bool(false)]),
        );
        props
    }

    #[test]
    fn node_round_trips_labels_and_props() {
        let interner = FakeInterner::over(&["alpha", "beta", "gamma"]);
        let record = NodeRecord {
            label_ids: vec![7, 3],
            props: sample_props(),
        };
        let bytes = encode_node(&record, |n| interner.intern(n)).unwrap();
        let back = decode_node(&bytes, |i| interner.resolve(i)).unwrap();
        assert_eq!(back.label_ids, vec![7, 3]);
        assert_eq!(back.props, sample_props());
    }

    #[test]
    fn edge_round_trips() {
        let interner = FakeInterner::over(&["alpha", "beta", "gamma"]);
        let record = EdgeRecord {
            label_id: 9,
            src: 100,
            dst: 200,
            props: sample_props(),
        };
        let bytes = encode_edge(&record, |n| interner.intern(n)).unwrap();
        let back = decode_edge(&bytes, |i| interner.resolve(i)).unwrap();
        assert_eq!(back.label_id, 9);
        assert_eq!(back.src, 100);
        assert_eq!(back.dst, 200);
        assert_eq!(back.props, sample_props());
    }

    #[test]
    fn empty_props_and_labels_round_trip() {
        let record = NodeRecord {
            label_ids: vec![],
            props: BTreeMap::new(),
        };
        let bytes = encode_node(&record, |_| unreachable!("no props to intern")).unwrap();
        let back = decode_node(&bytes, |_| unreachable!("no props to resolve")).unwrap();
        assert!(back.label_ids.is_empty());
        assert!(back.props.is_empty());
    }

    #[test]
    fn prop_raw_finds_each_property_without_names() {
        let interner = FakeInterner::over(&["alpha", "beta", "gamma"]);
        let record = NodeRecord {
            label_ids: vec![1],
            props: sample_props(),
        };
        let bytes = encode_node(&record, |n| interner.intern(n)).unwrap();
        for (name, expected) in sample_props() {
            let id = interner.intern(&name).unwrap();
            let raw = node_prop_raw(&bytes, id).unwrap().unwrap();
            assert_eq!(decode_value(raw).unwrap(), expected);
        }
        assert_eq!(node_prop_raw(&bytes, 999).unwrap(), None);
    }

    #[test]
    fn edge_prop_raw_mirrors_node_path() {
        let interner = FakeInterner::over(&["alpha", "beta", "gamma"]);
        let record = EdgeRecord {
            label_id: 1,
            src: 1,
            dst: 2,
            props: sample_props(),
        };
        let bytes = encode_edge(&record, |n| interner.intern(n)).unwrap();
        let id = interner.intern("beta").unwrap();
        let raw = edge_prop_raw(&bytes, id).unwrap().unwrap();
        assert_eq!(
            decode_value(raw).unwrap(),
            PropertyValue::String("hello".into())
        );
    }

    #[test]
    fn truncated_bytes_error_not_panic() {
        let interner = FakeInterner::over(&["alpha", "beta", "gamma"]);
        let record = NodeRecord {
            label_ids: vec![1, 2, 3],
            props: sample_props(),
        };
        let bytes = encode_node(&record, |n| interner.intern(n)).unwrap();
        for cut in [0, 1, 5, bytes.len() / 2, bytes.len() - 1] {
            let truncated = &bytes[..cut];
            // Any of these may error or (for value-region cuts) fail the
            // postcard decode -- but never panic.
            let _ = decode_node(truncated, |i| interner.resolve(i));
            let _ = node_prop_raw(truncated, 0);
        }
    }
}
