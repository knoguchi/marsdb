//! Property indexes: `(label, property) -> node_ids` keyed by an
//! order-preserving encoding of the property's value. See
//! `marsdb_storage::tables::{INDEX_DEFS, PROPERTY_INDEX}` for the on-disk
//! layout this module builds keys for.

use std::collections::BTreeMap;

use marsdb_storage::{ReadableMultimapTable, ReadableTable, Txn, WriteTransaction};
use serde::{Deserialize, Serialize};

use crate::error::GraphError;
use crate::labels::{lookup_label_id, resolve_label};
use crate::model::{NodeId, PropertyValue};
use crate::props::{lookup_prop_id, resolve_prop};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexDef {
    pub unique: bool,
}

/// `label_id(4 bytes BE) ++ property_id(4 bytes BE)` — the key `INDEX_DEFS`
/// uses, and the fixed prefix every `PROPERTY_INDEX` entry for this
/// (label, property) pair starts with.
fn index_prefix(label_id: u32, prop_id: u32) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&label_id.to_be_bytes());
    out[4..8].copy_from_slice(&prop_id.to_be_bytes());
    out
}

/// Order-preserving byte encoding of a single `PropertyValue`, for use as
/// a `PROPERTY_INDEX` key suffix. Lexicographic byte comparison matches
/// real value ordering *within one type* (needed for a future range scan,
/// not used yet — MVP only does exact-match lookups) — a leading type tag
/// keeps different types from ever comparing as equal or interleaving.
/// `Duration` has no meaningful total order (see `PropertyValue::Duration`'s
/// own doc comment) — its encoding is only guaranteed consistent for
/// equality, not real ordering, which is fine since nothing orders by it.
pub(crate) fn encode_index_value(v: &PropertyValue) -> Vec<u8> {
    match v {
        PropertyValue::Null => vec![0x00],
        PropertyValue::Bool(b) => vec![0x01, u8::from(*b)],
        // Flip the sign bit so two's-complement ordering becomes correct
        // unsigned big-endian byte ordering (the standard trick: the most
        // negative i64 maps to all-zero bytes, the most positive to
        // all-one bytes).
        PropertyValue::Int(i) => {
            let mut out = vec![0x02];
            out.extend_from_slice(&((*i as u64) ^ 0x8000_0000_0000_0000).to_be_bytes());
            out
        }
        // Standard sortable-float transform: flip the sign bit for a
        // non-negative float (so it sorts above all negatives), flip every
        // bit for a negative float (so more-negative sorts lower).
        PropertyValue::Float(f) => {
            let bits = f.to_bits();
            let sortable = if bits & 0x8000_0000_0000_0000 != 0 {
                !bits
            } else {
                bits | 0x8000_0000_0000_0000
            };
            let mut out = vec![0x03];
            out.extend_from_slice(&sortable.to_be_bytes());
            out
        }
        // Raw UTF-8 bytes compare correctly by codepoint for ASCII and
        // "close enough" (not full Unicode collation) in general -- same
        // tradeoff most embedded databases make without pulling in ICU.
        PropertyValue::String(s) => {
            let mut out = vec![0x04];
            out.extend_from_slice(s.as_bytes());
            out
        }
        PropertyValue::Date(days) => {
            let mut out = vec![0x05];
            out.extend_from_slice(&((*days as i64 as u64) ^ 0x8000_0000_0000_0000).to_be_bytes());
            out
        }
        PropertyValue::Duration {
            months,
            days,
            seconds,
            nanos,
        } => {
            let mut out = vec![0x06];
            out.extend_from_slice(&months.to_be_bytes());
            out.extend_from_slice(&days.to_be_bytes());
            out.extend_from_slice(&seconds.to_be_bytes());
            out.extend_from_slice(&nanos.to_be_bytes());
            out
        }
        // Always non-negative by construction (`0..86_400_000_000_000`) --
        // plain BE bytes already sort correctly, no sign-flip needed.
        PropertyValue::LocalTime(nanos_of_day) => {
            let mut out = vec![0x07];
            out.extend_from_slice(&nanos_of_day.to_be_bytes());
            out
        }
        // Keyed by the UTC-equivalent instant-of-day (`nanos_of_day -
        // offset_seconds`), not the raw wall-clock fields -- matches
        // `Time`'s own equality/ordering rule (see its doc comment), so
        // two structurally-different `Time`s that represent the same
        // instant correctly collapse to the same index key.
        PropertyValue::Time {
            nanos_of_day,
            offset_seconds,
        } => {
            let instant = nanos_of_day - *offset_seconds as i64 * 1_000_000_000;
            let mut out = vec![0x08];
            out.extend_from_slice(&((instant as u64) ^ 0x8000_0000_0000_0000).to_be_bytes());
            out
        }
        PropertyValue::LocalDateTime {
            epoch_seconds,
            nanos,
        } => {
            let mut out = vec![0x09];
            out.extend_from_slice(&((*epoch_seconds as u64) ^ 0x8000_0000_0000_0000).to_be_bytes());
            out.extend_from_slice(&nanos.to_be_bytes());
            out
        }
        // `offset_seconds` deliberately excluded -- `DateTime`'s equality/
        // ordering is instant-only (see its doc comment), same reasoning
        // as `Time` above.
        PropertyValue::DateTime {
            epoch_seconds,
            nanos,
            ..
        } => {
            let mut out = vec![0x0A];
            out.extend_from_slice(&((*epoch_seconds as u64) ^ 0x8000_0000_0000_0000).to_be_bytes());
            out.extend_from_slice(&nanos.to_be_bytes());
            out
        }
        // No real ordering across two lists is defined/needed (same
        // "consistent for equality, not real ordering" carve-out
        // `Duration` above already has) -- MVP indexing only does exact-
        // match lookups. Each element's own encoding is length-prefixed
        // so two different lists can never collide onto the same byte
        // string (e.g. `["ab", "c"]` vs `["a", "bc"]`, which otherwise
        // concatenate to visually-different but genuinely ambiguous byte
        // runs once strings' own raw-UTF-8, non-length-prefixed encoding
        // is stacked back to back).
        PropertyValue::List(items) => {
            let mut out = vec![0x0B];
            for item in items {
                let encoded = encode_index_value(item);
                out.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
                out.extend_from_slice(&encoded);
            }
            out
        }
        // Never reaches here: `Map` is only ever constructed on the
        // parameter-passing path (`PropertyValue`'s own doc comment), and
        // nothing ever stores -- so nothing ever indexes -- a real node/
        // edge property this way.
        PropertyValue::Map(_) => {
            unreachable!("PropertyValue::Map is never a real stored/indexed property value")
        }
    }
}

fn index_key(label_id: u32, prop_id: u32, value: &PropertyValue) -> Vec<u8> {
    let mut out = index_prefix(label_id, prop_id).to_vec();
    out.extend_from_slice(&encode_index_value(value));
    out
}

/// Declares an index on `(label, prop)` and backfills it from every
/// existing node carrying `label`. Errors (without creating the index) if
/// `unique` is requested and two existing nodes already share a value.
/// Idempotent by (label, prop) identity, not by `unique`-ness — calling
/// this again on an already-indexed pair is an error, same as most
/// databases' `CREATE INDEX` (no silent redefinition).
pub fn create_index(
    write_txn: &WriteTransaction,
    label: &str,
    prop: &str,
    unique: bool,
) -> Result<(), GraphError> {
    let label_id = crate::labels::intern_label(write_txn, label)?;
    let prop_id = crate::props::intern_prop(write_txn, prop)?;
    let prefix = index_prefix(label_id, prop_id);
    {
        let defs = write_txn.open_table(marsdb_storage::tables::INDEX_DEFS)?;
        if defs.get(prefix.as_slice())?.is_some() {
            return Err(GraphError::CorruptData(format!(
                "index on label {label:?} property {prop:?} already exists"
            )));
        }
    }

    // Backfill: walk every node with this label (via the existing
    // NODE_LABEL_INDEX secondary index, not a full NODES scan) and index
    // whatever value it currently has for `prop` (skipping nodes missing
    // it entirely -- a missing property never appears in the index, same
    // as `IS NULL`/absence being indistinguishable elsewhere in this
    // codebase).
    let label_index = write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
    let node_ids: Vec<u64> = label_index
        .get(label_id)?
        .map(|entry| entry.map(|value| value.value()).map_err(GraphError::from))
        .collect::<Result<Vec<_>, GraphError>>()?;
    drop(label_index);
    let mut entries: Vec<(Vec<u8>, u64)> = Vec::with_capacity(node_ids.len());
    {
        let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        for node_id in &node_ids {
            let Some(guard) = nodes.get(*node_id)? else {
                continue;
            };
            let record: crate::encode::NodeRecord = crate::encode::decode(guard.value())?;
            if let Some(value) = record.props.get(prop) {
                entries.push((index_key(label_id, prop_id, value), *node_id));
            }
        }
    }
    if unique {
        let mut seen = std::collections::HashSet::with_capacity(entries.len());
        for (key, _) in &entries {
            if !seen.insert(key.clone()) {
                return Err(GraphError::UniqueConstraintViolation {
                    label: label.to_string(),
                    property: prop.to_string(),
                });
            }
        }
    }

    {
        let mut defs = write_txn.open_table(marsdb_storage::tables::INDEX_DEFS)?;
        let encoded = postcard::to_allocvec(&IndexDef { unique })?;
        defs.insert(prefix.as_slice(), encoded.as_slice())?;
    }
    {
        let mut index = write_txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
        for (key, node_id) in entries {
            index.insert(key.as_slice(), node_id)?;
        }
    }
    Ok(())
}

/// `None` means no index is declared on `(label, prop)`.
pub fn lookup_index_def(txn: Txn, label: &str, prop: &str) -> Result<Option<IndexDef>, GraphError> {
    let Some(label_id) = lookup_label_id(txn, label)? else {
        return Ok(None);
    };
    let Some(prop_id) = lookup_prop_id(txn, prop)? else {
        return Ok(None);
    };
    let prefix = index_prefix(label_id, prop_id);
    let defs = txn.open_table(marsdb_storage::tables::INDEX_DEFS)?;
    let found = defs
        .get(prefix.as_slice())?
        .map(|guard| guard.value().to_vec());
    drop(defs);
    match found {
        Some(bytes) => Ok(Some(postcard::from_bytes(&bytes)?)),
        None => Ok(None),
    }
}

/// Exact-match lookup: every node currently indexed under `(label, prop) =
/// value`, up to `limit` of them if given. Caller (the planner, in a
/// later change) is responsible for checking `lookup_index_def` first —
/// this returns an empty result, not an error, if no such index exists
/// (matching a genuinely-empty index would look the same, and this
/// function has no way to tell those apart itself without the same
/// lookup its caller likely already did). `limit` bounds the underlying
/// multimap iterator itself (`.take(limit)` before collecting, not a
/// truncate after) — same real fix this same class of bug needed for
/// `NODE_LABEL_INDEX` (see `GraphStore::all_nodes_limited_in_txn`'s
/// history): truncating *after* `collect()` would still walk every
/// matching entry first, defeating the point of a `LIMIT` push-down.
pub fn lookup_exact(
    txn: Txn,
    label: &str,
    prop: &str,
    value: &PropertyValue,
    limit: Option<usize>,
) -> Result<Vec<NodeId>, GraphError> {
    let Some(label_id) = lookup_label_id(txn, label)? else {
        return Ok(Vec::new());
    };
    let Some(prop_id) = lookup_prop_id(txn, prop)? else {
        return Ok(Vec::new());
    };
    let key = index_key(label_id, prop_id, value);
    let index = txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
    let iter = index.get(key.as_slice())?;
    let ids: Vec<NodeId> = match limit {
        Some(limit) => iter
            .take(limit)
            .map(|entry| {
                entry
                    .map(|value| NodeId(value.value()))
                    .map_err(GraphError::from)
            })
            .collect::<Result<Vec<_>, GraphError>>()?,
        None => iter
            .map(|entry| {
                entry
                    .map(|value| NodeId(value.value()))
                    .map_err(GraphError::from)
            })
            .collect::<Result<Vec<_>, GraphError>>()?,
    };
    drop(index);
    Ok(ids)
}

/// Cheap, exact cardinality of `(label, prop) = value` under a declared
/// index — the stat the query planner uses to pick the most selective
/// candidate when several indexed equality conjuncts are available for the
/// same scan (see `marsdb_query::planner::apply_index_seeks`). O(1): redb's
/// `MultimapValue::len()` reports a count it already tracks per key, so
/// this never walks the matching entries themselves, unlike `lookup_exact`.
/// Returns 0 if no such index/value exists (same "caller already checked
/// `lookup_index_def`" contract as `lookup_exact`).
pub fn match_count(
    txn: Txn,
    label: &str,
    prop: &str,
    value: &PropertyValue,
) -> Result<u64, GraphError> {
    let Some(label_id) = lookup_label_id(txn, label)? else {
        return Ok(0);
    };
    let Some(prop_id) = lookup_prop_id(txn, prop)? else {
        return Ok(0);
    };
    let key = index_key(label_id, prop_id, value);
    let index = txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
    let count = index.get(key.as_slice())?.len();
    Ok(count)
}

/// Every declared index whose label is in `label_ids`, as `(label_id,
/// prop_id, prop_name, IndexDef)`. `INDEX_DEFS` is scanned in full (not a
/// prefix-range query — `TableHandle` only exposes `get`/`iter`, and the
/// number of *declared indexes* is expected to be small, unlike node
/// counts) and filtered in memory.
fn indexes_for_labels(
    txn: Txn,
    label_ids: &[u32],
) -> Result<Vec<(u32, u32, String, IndexDef)>, GraphError> {
    let defs = match txn.open_table(marsdb_storage::tables::INDEX_DEFS) {
        Ok(table) => table,
        Err(marsdb_storage::StorageError::Table(redb::TableError::TableDoesNotExist(_))) => {
            return Ok(Vec::new())
        }
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in defs.iter()? {
        let (key, value) = entry?;
        let key_bytes = key.value();
        let label_id = u32::from_be_bytes(
            key_bytes[0..4]
                .try_into()
                .expect("index key prefix is 8 bytes"),
        );
        if !label_ids.contains(&label_id) {
            continue;
        }
        let prop_id = u32::from_be_bytes(
            key_bytes[4..8]
                .try_into()
                .expect("index key prefix is 8 bytes"),
        );
        let def: IndexDef = postcard::from_bytes(value.value())?;
        let prop_name = resolve_prop(txn, prop_id)?;
        out.push((label_id, prop_id, prop_name, def));
    }
    Ok(out)
}

/// Identifies one declared index, both by id (for the actual key/lookup)
/// and by name (only needed for a `UniqueConstraintViolation`'s message).
/// Bundled into one struct so `insert_entry` doesn't take 8 separate
/// arguments (clippy's `too_many_arguments`, capped at 7).
struct IndexTarget<'a> {
    label_id: u32,
    prop_id: u32,
    label: &'a str,
    prop: &'a str,
}

fn insert_entry(
    write_txn: &WriteTransaction,
    target: &IndexTarget<'_>,
    value: &PropertyValue,
    node_id: u64,
    unique: bool,
) -> Result<(), GraphError> {
    let key = index_key(target.label_id, target.prop_id, value);
    if unique {
        let index = write_txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
        let exists = index.get(key.as_slice())?.next().is_some();
        drop(index);
        if exists {
            return Err(GraphError::UniqueConstraintViolation {
                label: target.label.to_string(),
                property: target.prop.to_string(),
            });
        }
    }
    let mut index = write_txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
    index.insert(key.as_slice(), node_id)?;
    Ok(())
}

fn remove_entry(
    write_txn: &WriteTransaction,
    label_id: u32,
    prop_id: u32,
    value: &PropertyValue,
    node_id: u64,
) -> Result<(), GraphError> {
    let key = index_key(label_id, prop_id, value);
    let mut index = write_txn.open_multimap_table(marsdb_storage::tables::PROPERTY_INDEX)?;
    index.remove(key.as_slice(), node_id)?;
    Ok(())
}

/// Inserts index entries for `node_id` into every declared index whose
/// label is in `label_ids` and whose property `props` has a value for.
/// Called on node creation (`label_ids` = every label the node was just
/// given) and on `SET n:Label` (`label_ids` = just the one newly-added
/// label — indexes on labels the node already had are untouched, since
/// nothing about their entries changed).
pub fn on_node_created(
    write_txn: &WriteTransaction,
    node_id: u64,
    label_ids: &[u32],
    props: &BTreeMap<String, PropertyValue>,
) -> Result<(), GraphError> {
    for (label_id, prop_id, prop_name, def) in indexes_for_labels(Txn::Write(write_txn), label_ids)?
    {
        if let Some(value) = props.get(&prop_name) {
            let label = resolve_label(Txn::Write(write_txn), label_id)?;
            let target = IndexTarget {
                label_id,
                prop_id,
                label: &label,
                prop: &prop_name,
            };
            insert_entry(write_txn, &target, value, node_id, def.unique)?;
        }
    }
    Ok(())
}

/// Removes `node_id`'s index entries from every declared index whose label
/// is in `label_ids` and whose property `props` (the values *before* this
/// change) has a value for. Called on node deletion (`label_ids` = every
/// label the node had) and on `REMOVE n:Label` (`label_ids` = just the one
/// removed label).
pub fn on_node_deleted(
    write_txn: &WriteTransaction,
    node_id: u64,
    label_ids: &[u32],
    props: &BTreeMap<String, PropertyValue>,
) -> Result<(), GraphError> {
    for (label_id, prop_id, prop_name, _def) in
        indexes_for_labels(Txn::Write(write_txn), label_ids)?
    {
        if let Some(value) = props.get(&prop_name) {
            remove_entry(write_txn, label_id, prop_id, value, node_id)?;
        }
    }
    Ok(())
}

/// One property's value changed on an existing node (`SET n.prop = ..`/
/// `REMOVE n.prop`) — removes the old index entry (if `old_value` is
/// `Some` and an index covers `(label, prop)` for one of `label_ids`) and
/// inserts the new one (if `new_value` is `Some`). `new_value: None`
/// means the property was removed entirely, not set to `null` — a
/// `PropertyValue::Null` value is still `Some(&PropertyValue::Null)` here
/// and gets indexed like any other value (matches `create_index`'s own
/// backfill, which only skips a property that's *absent*, not one whose
/// value is `Null`).
pub fn on_node_prop_changed(
    write_txn: &WriteTransaction,
    node_id: u64,
    label_ids: &[u32],
    prop: &str,
    old_value: Option<&PropertyValue>,
    new_value: Option<&PropertyValue>,
) -> Result<(), GraphError> {
    for (label_id, prop_id, prop_name, def) in indexes_for_labels(Txn::Write(write_txn), label_ids)?
    {
        if prop_name != prop {
            continue;
        }
        if let Some(old) = old_value {
            remove_entry(write_txn, label_id, prop_id, old, node_id)?;
        }
        if let Some(new) = new_value {
            let label = resolve_label(Txn::Write(write_txn), label_id)?;
            let target = IndexTarget {
                label_id,
                prop_id,
                label: &label,
                prop: &prop_name,
            };
            insert_entry(write_txn, &target, new, node_id, def.unique)?;
        }
    }
    Ok(())
}
