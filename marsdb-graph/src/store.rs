use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use marsdb_storage::{
    ReadTransaction, ReadableMultimapTable, ReadableTable, StorageEngine, Txn, WriteTransaction,
};

use crate::encode::{decode, encode, EdgeRecord, NodeRecord};
use crate::error::GraphError;
use crate::id::next_id;
use crate::labels::{intern_label, lookup_label_id, resolve_label};
use crate::model::{AdjEntry, Direction, Edge, EdgeId, Node, NodeId, PropertyValue};

pub struct GraphStore {
    storage: StorageEngine,
}

/// Successful physical and logical integrity-check summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    /// `false` means redb detected physical damage and repaired it before
    /// MarsDB's logical checks ran.
    pub physical_was_clean: bool,
    pub labels: u64,
    pub nodes: u64,
    pub edges: u64,
}

impl GraphStore {
    pub fn open_file(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Ok(Self {
            storage: StorageEngine::open_file(path)?,
        })
    }

    pub fn open_memory() -> Result<Self, GraphError> {
        Ok(Self {
            storage: StorageEngine::open_memory()?,
        })
    }

    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), GraphError> {
        self.storage.backup_to(path)?;
        Ok(())
    }

    /// Check physical storage plus MarsDB's graph invariants. This requires
    /// exclusive mutable access because redb may repair physical metadata.
    pub fn check_integrity(&mut self) -> Result<IntegrityReport, GraphError> {
        let physical_was_clean = self.storage.check_integrity()?;
        let read = self.storage.begin_read()?;

        let mut labels_by_id = BTreeMap::new();
        {
            let table = read.open_table(marsdb_storage::tables::ID_TO_LABEL)?;
            for entry in table.iter()? {
                let (id, label) = entry?;
                labels_by_id.insert(id.value(), label.value().to_owned());
            }
        }
        {
            let table = read.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
            let mut count = 0usize;
            for entry in table.iter()? {
                let (label, id) = entry?;
                count += 1;
                if labels_by_id.get(&id.value()).map(String::as_str) != Some(label.value()) {
                    return Err(GraphError::CorruptData(format!(
                        "label mapping {:?} -> {} has no matching reverse mapping",
                        label.value(),
                        id.value()
                    )));
                }
            }
            if count != labels_by_id.len() {
                return Err(GraphError::CorruptData(
                    "label mapping tables have different entry counts".into(),
                ));
            }
        }

        let mut nodes = BTreeMap::<u64, Vec<u32>>::new();
        {
            let table = read.open_table(marsdb_storage::tables::NODES)?;
            for entry in table.iter()? {
                let (id, value) = entry?;
                let record: NodeRecord = decode(value.value())?;
                for label_id in &record.label_ids {
                    if !labels_by_id.contains_key(label_id) {
                        return Err(GraphError::CorruptData(format!(
                            "node {} references unknown label {}",
                            id.value(),
                            label_id
                        )));
                    }
                }
                nodes.insert(id.value(), record.label_ids);
            }
        }

        let mut indexed_labels = BTreeSet::new();
        {
            let table = read.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
            for entry in table.iter()? {
                let (label_id, values) = entry?;
                let label_id = label_id.value();
                if !labels_by_id.contains_key(&label_id) {
                    return Err(GraphError::CorruptData(format!(
                        "node label index references unknown label {label_id}"
                    )));
                }
                for node_id in values {
                    let node_id = node_id?.value();
                    let Some(node_labels) = nodes.get(&node_id) else {
                        return Err(GraphError::CorruptData(format!(
                            "node label index references missing node {node_id}"
                        )));
                    };
                    if !node_labels.contains(&label_id) {
                        return Err(GraphError::CorruptData(format!(
                            "node label index has label {label_id} for node {node_id}, but the node does not"
                        )));
                    }
                    indexed_labels.insert((label_id, node_id));
                }
            }
        }
        for (node_id, label_ids) in &nodes {
            for label_id in label_ids {
                if !indexed_labels.contains(&(*label_id, *node_id)) {
                    return Err(GraphError::CorruptData(format!(
                        "node {node_id} has label {label_id} but is missing from the label index"
                    )));
                }
            }
        }

        let mut edges = BTreeMap::<u64, (u32, u64, u64)>::new();
        {
            let table = read.open_table(marsdb_storage::tables::EDGES)?;
            for entry in table.iter()? {
                let (id, value) = entry?;
                let record: EdgeRecord = decode(value.value())?;
                if !labels_by_id.contains_key(&record.label_id) {
                    return Err(GraphError::CorruptData(format!(
                        "edge {} references unknown label {}",
                        id.value(),
                        record.label_id
                    )));
                }
                if !nodes.contains_key(&record.src) || !nodes.contains_key(&record.dst) {
                    return Err(GraphError::CorruptData(format!(
                        "edge {} references missing endpoint {} -> {}",
                        id.value(),
                        record.src,
                        record.dst
                    )));
                }
                edges.insert(id.value(), (record.label_id, record.src, record.dst));
            }
        }

        let outgoing =
            Self::check_adjacency(&read, marsdb_storage::tables::ADJ_OUT, &nodes, &edges, true)?;
        let incoming =
            Self::check_adjacency(&read, marsdb_storage::tables::ADJ_IN, &nodes, &edges, false)?;
        for (&edge_id, &(label_id, src, dst)) in &edges {
            if !outgoing.contains(&(src, edge_id, dst, label_id)) {
                return Err(GraphError::CorruptData(format!(
                    "edge {edge_id} is missing from outgoing adjacency"
                )));
            }
            if !incoming.contains(&(dst, edge_id, src, label_id)) {
                return Err(GraphError::CorruptData(format!(
                    "edge {edge_id} is missing from incoming adjacency"
                )));
            }
        }

        let meta = read.open_table(marsdb_storage::tables::META)?;
        for (counter, maximum) in [
            ("next_node_id", nodes.keys().next_back().copied()),
            ("next_edge_id", edges.keys().next_back().copied()),
        ] {
            if let Some(maximum) = maximum {
                let stored = meta.get(counter)?.map(|value| value.value()).unwrap_or(0);
                if stored < maximum {
                    return Err(GraphError::CorruptData(format!(
                        "{counter} counter {stored} is below maximum allocated id {maximum}"
                    )));
                }
            }
        }

        Ok(IntegrityReport {
            physical_was_clean,
            labels: labels_by_id.len() as u64,
            nodes: nodes.len() as u64,
            edges: edges.len() as u64,
        })
    }

    fn check_adjacency(
        read: &ReadTransaction,
        definition: marsdb_storage::MultimapTableDefinition<u64, &[u8]>,
        nodes: &BTreeMap<u64, Vec<u32>>,
        edges: &BTreeMap<u64, (u32, u64, u64)>,
        outgoing: bool,
    ) -> Result<BTreeSet<(u64, u64, u64, u32)>, GraphError> {
        let table = read.open_multimap_table(definition)?;
        let mut found = BTreeSet::new();
        for entry in table.iter()? {
            let (owner, values) = entry?;
            let owner = owner.value();
            if !nodes.contains_key(&owner) {
                return Err(GraphError::CorruptData(format!(
                    "adjacency references missing owner node {owner}"
                )));
            }
            for value in values {
                let adjacency = AdjEntry::decode(value?.value())?;
                let Some(&(label_id, src, dst)) = edges.get(&adjacency.edge_id.0) else {
                    return Err(GraphError::CorruptData(format!(
                        "adjacency references missing edge {}",
                        adjacency.edge_id.0
                    )));
                };
                let expected = if outgoing { (src, dst) } else { (dst, src) };
                if owner != expected.0
                    || adjacency.other.0 != expected.1
                    || adjacency.label_id != label_id
                {
                    return Err(GraphError::CorruptData(format!(
                        "adjacency entry for edge {} does not match the edge record",
                        adjacency.edge_id.0
                    )));
                }
                found.insert((
                    owner,
                    adjacency.edge_id.0,
                    adjacency.other.0,
                    adjacency.label_id,
                ));
            }
        }
        Ok(found)
    }

    /// Open a write transaction spanning multiple graph operations. Callers
    /// (e.g. the query executor) drive an entire Cypher statement through
    /// the `*_in_txn` methods below using this one transaction, then call
    /// `write_txn.commit()` themselves — this is the crash-safety boundary
    /// from the plan: one statement = one transaction, not one transaction
    /// per individual node/edge write.
    ///
    /// v1 uses a write transaction even for pure-read statements (rather
    /// than a separate read-only path) to keep one code path and guarantee
    /// every statement — reads included — sees one consistent snapshot.
    /// Trade-off: this serializes concurrent readers behind redb's
    /// single-writer lock instead of allowing true concurrent reads; a
    /// read-only transaction path is the natural follow-up if read
    /// concurrency becomes a bottleneck.
    pub fn begin_write(&self) -> Result<WriteTransaction, GraphError> {
        Ok(self.storage.begin_write()?)
    }

    /// Open a read transaction for a statement that never mutates
    /// anything (`MATCH ... RETURN`) — a consistent point-in-time
    /// snapshot that runs alongside any concurrent readers or a
    /// concurrent writer without contending for redb's single-writer
    /// lock. No commit/abort: a read transaction has nothing to roll
    /// back, it just releases on drop.
    pub fn begin_read(&self) -> Result<ReadTransaction, GraphError> {
        Ok(self.storage.begin_read()?)
    }

    /// Commit a transaction obtained from [`begin_write`](Self::begin_write).
    pub fn commit(write_txn: WriteTransaction) -> Result<(), GraphError> {
        write_txn.commit()?;
        Ok(())
    }

    /// Abort (roll back) a transaction obtained from
    /// [`begin_write`](Self::begin_write), discarding any writes made
    /// through it.
    pub fn abort(write_txn: WriteTransaction) -> Result<(), GraphError> {
        write_txn.abort()?;
        Ok(())
    }

    pub fn create_node(
        &self,
        labels: &[&str],
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<NodeId, GraphError> {
        let write_txn = self.begin_write()?;
        let id = Self::create_node_in_txn(&write_txn, labels, props)?;
        write_txn.commit()?;
        Ok(id)
    }

    pub fn create_node_in_txn(
        write_txn: &WriteTransaction,
        labels: &[&str],
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<NodeId, GraphError> {
        let label_ids = labels
            .iter()
            .map(|l| intern_label(write_txn, l))
            .collect::<Result<Vec<_>, _>>()?;
        let id = next_id(write_txn, "next_node_id")?;
        let record = NodeRecord {
            label_ids: label_ids.clone(),
            props,
        };
        let bytes = encode(&record)?;
        let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        nodes.insert(id, bytes.as_slice())?;
        drop(nodes);
        let mut label_index =
            write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        for &label_id in &label_ids {
            label_index.insert(label_id, id)?;
        }
        drop(label_index);
        crate::index::on_node_created(write_txn, id, &label_ids, &record.props)?;
        Ok(NodeId(id))
    }

    pub fn get_node(&self, id: NodeId) -> Result<Option<Node>, GraphError> {
        let read_txn = self.begin_read()?;
        Self::get_node_in_txn(Txn::Read(&read_txn), id)
    }

    pub fn get_node_in_txn(txn: Txn, id: NodeId) -> Result<Option<Node>, GraphError> {
        let record: Option<NodeRecord> = {
            let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
            let found = match nodes.get(id.0)? {
                Some(guard) => Some(decode(guard.value())?),
                None => None,
            };
            found
        };
        let Some(record) = record else {
            return Ok(None);
        };
        let labels = record
            .label_ids
            .iter()
            .map(|&lid| resolve_label(txn, lid))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(Node {
            id,
            labels,
            props: record.props,
        }))
    }

    pub fn create_edge(
        &self,
        label: &str,
        src: NodeId,
        dst: NodeId,
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<EdgeId, GraphError> {
        let write_txn = self.begin_write()?;
        let id = Self::create_edge_in_txn(&write_txn, label, src, dst, props)?;
        write_txn.commit()?;
        Ok(id)
    }

    pub fn create_edge_in_txn(
        write_txn: &WriteTransaction,
        label: &str,
        src: NodeId,
        dst: NodeId,
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<EdgeId, GraphError> {
        {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            if nodes.get(src.0)?.is_none() {
                return Err(GraphError::NodeNotFound(src));
            }
            if nodes.get(dst.0)?.is_none() {
                return Err(GraphError::NodeNotFound(dst));
            }
        }
        let label_id = intern_label(write_txn, label)?;
        let id = next_id(write_txn, "next_edge_id")?;
        let record = EdgeRecord {
            label_id,
            src: src.0,
            dst: dst.0,
            props,
        };
        let bytes = encode(&record)?;
        let mut edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
        edges.insert(id, bytes.as_slice())?;

        let out_entry = AdjEntry {
            edge_id: EdgeId(id),
            other: dst,
            label_id,
        }
        .encode();
        let in_entry = AdjEntry {
            edge_id: EdgeId(id),
            other: src,
            label_id,
        }
        .encode();
        let mut adj_out = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_OUT)?;
        adj_out.insert(src.0, out_entry.as_slice())?;
        let mut adj_in = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_IN)?;
        adj_in.insert(dst.0, in_entry.as_slice())?;
        Ok(EdgeId(id))
    }

    pub fn get_edge(&self, id: EdgeId) -> Result<Option<Edge>, GraphError> {
        let read_txn = self.begin_read()?;
        Self::get_edge_in_txn(Txn::Read(&read_txn), id)
    }

    pub fn get_edge_in_txn(txn: Txn, id: EdgeId) -> Result<Option<Edge>, GraphError> {
        let record: Option<EdgeRecord> = {
            let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
            let found = match edges.get(id.0)? {
                Some(guard) => Some(decode(guard.value())?),
                None => None,
            };
            found
        };
        let Some(record) = record else {
            return Ok(None);
        };
        let label = resolve_label(txn, record.label_id)?;
        Ok(Some(Edge {
            id,
            label,
            src: NodeId(record.src),
            dst: NodeId(record.dst),
            props: record.props,
        }))
    }

    /// Neighbors of `node` in `dir`, optionally filtered by edge label.
    /// Reads directly from the adjacency multimap without touching `edges`.
    pub fn neighbors(
        &self,
        node: NodeId,
        dir: Direction,
        label_filter: Option<&str>,
    ) -> Result<Vec<AdjEntry>, GraphError> {
        let read_txn = self.begin_read()?;
        Self::neighbors_in_txn(Txn::Read(&read_txn), node, dir, label_filter)
    }

    pub fn neighbors_in_txn(
        txn: Txn,
        node: NodeId,
        dir: Direction,
        label_filter: Option<&str>,
    ) -> Result<Vec<AdjEntry>, GraphError> {
        let label_id_filter = match label_filter {
            Some(l) => match lookup_label_id(txn, l)? {
                Some(id) => Some(id),
                None => return Ok(Vec::new()),
            },
            None => None,
        };
        let mut result = Vec::new();
        let table_def = match dir {
            Direction::Out => marsdb_storage::tables::ADJ_OUT,
            Direction::In => marsdb_storage::tables::ADJ_IN,
        };
        let table = txn.open_multimap_table(table_def)?;
        for item in table.get(node.0)? {
            let entry = AdjEntry::decode(item?.value())?;
            if label_id_filter.is_none_or(|lid| lid == entry.label_id) {
                result.push(entry);
            }
        }
        Ok(result)
    }

    pub fn delete_edge(&self, id: EdgeId) -> Result<bool, GraphError> {
        let write_txn = self.begin_write()?;
        let removed = Self::delete_edge_in_txn(&write_txn, id)?;
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn delete_edge_in_txn(
        write_txn: &WriteTransaction,
        id: EdgeId,
    ) -> Result<bool, GraphError> {
        let record_bytes: Option<Vec<u8>> = {
            let mut edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            let removed = edges.remove(id.0)?.map(|guard| guard.value().to_vec());
            removed
        };
        let Some(record_bytes) = record_bytes else {
            return Ok(false);
        };
        let record: EdgeRecord = decode(&record_bytes)?;
        let out_entry = AdjEntry {
            edge_id: id,
            other: NodeId(record.dst),
            label_id: record.label_id,
        }
        .encode();
        let in_entry = AdjEntry {
            edge_id: id,
            other: NodeId(record.src),
            label_id: record.label_id,
        }
        .encode();
        {
            let mut adj_out = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_OUT)?;
            adj_out.remove(record.src, out_entry.as_slice())?;
        }
        {
            let mut adj_in = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_IN)?;
            adj_in.remove(record.dst, in_entry.as_slice())?;
        }
        Ok(true)
    }

    /// Delete a node. If `detach` is false and the node has incident edges,
    /// returns `GraphError::NodeHasEdges` instead of deleting anything.
    pub fn delete_node(&self, id: NodeId, detach: bool) -> Result<bool, GraphError> {
        let write_txn = self.begin_write()?;
        let existed = Self::delete_node_in_txn(&write_txn, id, detach)?;
        write_txn.commit()?;
        Ok(existed)
    }

    pub fn delete_node_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        detach: bool,
    ) -> Result<bool, GraphError> {
        let mut incident: Vec<EdgeId> = Vec::new();
        {
            let adj_out = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_OUT)?;
            for item in adj_out.get(id.0)? {
                incident.push(AdjEntry::decode(item?.value())?.edge_id);
            }
            let adj_in = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_IN)?;
            for item in adj_in.get(id.0)? {
                incident.push(AdjEntry::decode(item?.value())?.edge_id);
            }
        }
        if !incident.is_empty() && !detach {
            return Err(GraphError::NodeHasEdges(id));
        }
        for edge_id in incident {
            Self::delete_edge_in_txn(write_txn, edge_id)?;
        }
        let removed_bytes: Option<Vec<u8>> = {
            let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let removed = nodes.remove(id.0)?.map(|guard| guard.value().to_vec());
            removed
        };
        let Some(removed_bytes) = removed_bytes else {
            return Ok(false);
        };
        let record: NodeRecord = decode(&removed_bytes)?;
        let mut label_index =
            write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        for &label_id in &record.label_ids {
            label_index.remove(label_id, id.0)?;
        }
        drop(label_index);
        crate::index::on_node_deleted(write_txn, id.0, &record.label_ids, &record.props)?;
        Ok(true)
    }

    /// Declares an index on `(label, prop)`, backfilling it from every
    /// existing node with `label` — see `index::create_index`'s own docs
    /// for the exact semantics (idempotency, unique-violation behavior).
    pub fn create_index(&self, label: &str, prop: &str, unique: bool) -> Result<(), GraphError> {
        let write_txn = self.begin_write()?;
        Self::create_index_in_txn(&write_txn, label, prop, unique)?;
        write_txn.commit()?;
        Ok(())
    }

    /// Same as `create_index`, but against an already-open
    /// `WriteTransaction` — for a caller (`CREATE INDEX` as a Cypher
    /// statement) that's already inside one transaction and must commit
    /// or abort it as a whole, not open a second one (redb allows only one
    /// writer at a time; opening a second would deadlock).
    pub fn create_index_in_txn(
        write_txn: &WriteTransaction,
        label: &str,
        prop: &str,
        unique: bool,
    ) -> Result<(), GraphError> {
        crate::index::create_index(write_txn, label, prop, unique)
    }

    /// `None` means no index is declared on `(label, prop)`.
    pub fn index_def(
        &self,
        label: &str,
        prop: &str,
    ) -> Result<Option<crate::IndexDef>, GraphError> {
        let read_txn = self.begin_read()?;
        crate::index::lookup_index_def(Txn::Read(&read_txn), label, prop)
    }

    /// Same as `index_def`, but against an already-open `Txn` — for a
    /// caller (the query planner/executor) that's already inside one
    /// transaction and needs a consistent view, not a fresh snapshot.
    pub fn index_def_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
    ) -> Result<Option<crate::IndexDef>, GraphError> {
        crate::index::lookup_index_def(txn, label, prop)
    }

    /// Same as `lookup_by_index`, but against an already-open `Txn`.
    pub fn lookup_by_index_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
        value: &PropertyValue,
    ) -> Result<Vec<NodeId>, GraphError> {
        crate::index::lookup_exact(txn, label, prop, value, None)
    }

    /// Same as `lookup_by_index_in_txn`, but stops once `limit` nodes are
    /// found — the storage-level end of `LIMIT` push-down through an
    /// `IndexSeek` (see `marsdb_query::planner`/`executor::stream_index_seek`).
    pub fn lookup_by_index_limited_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
        value: &PropertyValue,
        limit: usize,
    ) -> Result<Vec<NodeId>, GraphError> {
        crate::index::lookup_exact(txn, label, prop, value, Some(limit))
    }

    /// Cheap, exact count of nodes under `(label, prop) = value` — for the
    /// query planner to compare selectivity between several indexed
    /// equality candidates, not for fetching the nodes themselves (see
    /// `lookup_by_index_in_txn`). O(1), same contract as `lookup_by_index`
    /// re: "no index" vs "index, no match" both reading as `0`.
    pub fn index_match_count_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
        value: &PropertyValue,
    ) -> Result<u64, GraphError> {
        crate::index::match_count(txn, label, prop, value)
    }

    /// Every node currently indexed under `(label, prop) = value`. Empty
    /// (not an error) if no such index exists — check `index_def` first if
    /// the caller needs to distinguish "no index" from "index, no match".
    pub fn lookup_by_index(
        &self,
        label: &str,
        prop: &str,
        value: &PropertyValue,
    ) -> Result<Vec<NodeId>, GraphError> {
        let read_txn = self.begin_read()?;
        crate::index::lookup_exact(Txn::Read(&read_txn), label, prop, value, None)
    }

    pub fn set_node_prop(
        &self,
        id: NodeId,
        key: &str,
        value: PropertyValue,
    ) -> Result<bool, GraphError> {
        let write_txn = self.begin_write()?;
        let updated = Self::set_node_prop_in_txn(&write_txn, id, key, value)?;
        if updated {
            write_txn.commit()?;
        } else {
            write_txn.abort()?;
        }
        Ok(updated)
    }

    pub fn set_node_prop_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        key: &str,
        value: PropertyValue,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let found = nodes.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let mut record: NodeRecord = decode(&bytes)?;
        let old_value = record.props.insert(key.to_string(), value.clone());
        let new_bytes = encode(&record)?;
        let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        nodes.insert(id.0, new_bytes.as_slice())?;
        drop(nodes);
        crate::index::on_node_prop_changed(
            write_txn,
            id.0,
            &record.label_ids,
            key,
            old_value.as_ref(),
            Some(&value),
        )?;
        Ok(true)
    }

    pub fn set_edge_prop(
        &self,
        id: EdgeId,
        key: &str,
        value: PropertyValue,
    ) -> Result<bool, GraphError> {
        let write_txn = self.begin_write()?;
        let updated = Self::set_edge_prop_in_txn(&write_txn, id, key, value)?;
        if updated {
            write_txn.commit()?;
        } else {
            write_txn.abort()?;
        }
        Ok(updated)
    }

    pub fn set_edge_prop_in_txn(
        write_txn: &WriteTransaction,
        id: EdgeId,
        key: &str,
        value: PropertyValue,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            let found = edges.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let mut record: EdgeRecord = decode(&bytes)?;
        record.props.insert(key.to_string(), value);
        let new_bytes = encode(&record)?;
        let mut edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
        edges.insert(id.0, new_bytes.as_slice())?;
        Ok(true)
    }

    pub fn remove_node_prop_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        key: &str,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let found = nodes.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let mut record: NodeRecord = decode(&bytes)?;
        let old_value = record.props.remove(key);
        let new_bytes = encode(&record)?;
        let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        nodes.insert(id.0, new_bytes.as_slice())?;
        drop(nodes);
        crate::index::on_node_prop_changed(
            write_txn,
            id.0,
            &record.label_ids,
            key,
            old_value.as_ref(),
            None,
        )?;
        Ok(true)
    }

    pub fn remove_edge_prop_in_txn(
        write_txn: &WriteTransaction,
        id: EdgeId,
        key: &str,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            let found = edges.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let mut record: EdgeRecord = decode(&bytes)?;
        record.props.remove(key);
        let new_bytes = encode(&record)?;
        let mut edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
        edges.insert(id.0, new_bytes.as_slice())?;
        Ok(true)
    }

    /// Adds `label` to `id`'s label set -- a no-op (not an error) if it's
    /// already there, same idempotent-add semantics real Cypher's `SET
    /// n:Label` has.
    pub fn add_node_label_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        label: &str,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let found = nodes.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let mut record: NodeRecord = decode(&bytes)?;
        let label_id = intern_label(write_txn, label)?;
        if !record.label_ids.contains(&label_id) {
            record.label_ids.push(label_id);
            let new_bytes = encode(&record)?;
            let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            nodes.insert(id.0, new_bytes.as_slice())?;
            let mut label_index =
                write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
            label_index.insert(label_id, id.0)?;
            drop(label_index);
            crate::index::on_node_created(write_txn, id.0, &[label_id], &record.props)?;
        }
        Ok(true)
    }

    /// Removes `label` from `id`'s label set -- a no-op (not an error) if
    /// it's not there (label unknown entirely, or known but not on this
    /// node), same as real Cypher's `REMOVE n:Label`.
    pub fn remove_node_label_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        label: &str,
    ) -> Result<bool, GraphError> {
        let bytes_opt: Option<Vec<u8>> = {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let found = nodes.get(id.0)?.map(|g| g.value().to_vec());
            found
        };
        let Some(bytes) = bytes_opt else {
            return Ok(false);
        };
        let Some(label_id) = lookup_label_id(Txn::Write(write_txn), label)? else {
            return Ok(true);
        };
        let mut record: NodeRecord = decode(&bytes)?;
        if let Some(pos) = record.label_ids.iter().position(|&l| l == label_id) {
            record.label_ids.remove(pos);
            let new_bytes = encode(&record)?;
            let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            nodes.insert(id.0, new_bytes.as_slice())?;
            let mut label_index =
                write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
            label_index.remove(label_id, id.0)?;
            drop(label_index);
            crate::index::on_node_deleted(write_txn, id.0, &[label_id], &record.props)?;
        }
        Ok(true)
    }

    /// Full scan of all nodes, optionally filtered by label. v1 has no
    /// secondary index on label, so this is a linear scan of the table.
    pub fn all_nodes(&self, label_filter: Option<&str>) -> Result<Vec<Node>, GraphError> {
        let read_txn = self.begin_read()?;
        Self::all_nodes_in_txn(Txn::Read(&read_txn), label_filter)
    }

    /// Scan only graph identities, without decoding node records. Query
    /// pipelines use this to defer record/property loading until a filter or
    /// projection actually needs it.
    pub fn all_node_ids_limited_in_txn(
        txn: Txn,
        label_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<NodeId>, GraphError> {
        let Some(label_filter) = label_filter else {
            let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
            return nodes
                .iter()?
                .take(limit)
                .map(|entry| {
                    entry
                        .map(|(key, _)| NodeId(key.value()))
                        .map_err(Into::into)
                })
                .collect();
        };
        let Some(label_id) = lookup_label_id(txn, label_filter)? else {
            return Ok(Vec::new());
        };
        let label_index = txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        let ids = label_index
            .get(label_id)?
            .take(limit)
            .map(|entry| entry.map(|value| NodeId(value.value())).map_err(Into::into))
            .collect::<Result<Vec<_>, GraphError>>()?;
        drop(label_index);
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        for id in &ids {
            if nodes.get(id.0)?.is_none() {
                return Err(GraphError::CorruptData(format!(
                    "node label index references missing node {}",
                    id.0
                )));
            }
        }
        Ok(ids)
    }

    pub fn all_nodes_in_txn(txn: Txn, label_filter: Option<&str>) -> Result<Vec<Node>, GraphError> {
        Self::all_nodes_limited_in_txn(txn, label_filter, usize::MAX)
    }

    /// Same as `all_nodes_in_txn`, but stops once `limit` nodes are found --
    /// the storage-level end of `LIMIT` push-down (see the executor's
    /// `scan()`/`eval_plan` docs for the query-level half): a query whose
    /// entire plan is a bare scan feeding straight into a `LIMIT` doesn't
    /// need to touch rows past the first `limit`, whether or not a label
    /// filter narrows it first.
    pub fn all_nodes_limited_in_txn(
        txn: Txn,
        label_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Node>, GraphError> {
        // A label filter goes through NODE_LABEL_INDEX (label_id -> node_ids)
        // plus a point lookup per match, instead of scanning every row in
        // NODES — cost scales with the number of matching rows, not the
        // table size. No filter means every row is wanted anyway, so a full
        // scan is already optimal; the index wouldn't help.
        let Some(label_filter) = label_filter else {
            let mut result = Vec::new();
            let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
            for item in nodes.iter()? {
                if result.len() >= limit {
                    break;
                }
                let (key, value) = item?;
                let record: NodeRecord = decode(value.value())?;
                let labels = record
                    .label_ids
                    .iter()
                    .map(|&lid| resolve_label(txn, lid))
                    .collect::<Result<Vec<_>, _>>()?;
                result.push(Node {
                    id: NodeId(key.value()),
                    labels,
                    props: record.props,
                });
            }
            return Ok(result);
        };
        let Some(label_id) = lookup_label_id(txn, label_filter)? else {
            return Ok(Vec::new());
        };
        let node_ids: Vec<u64> = {
            let label_index = txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
            // `.take(limit)` here, not a `.truncate()` after collecting --
            // stops walking the multimap's own entries past `limit`, not
            // just the (more expensive) per-id NODES point-reads below.
            // Measured difference: without this, a labeled LIMIT query's
            // cost still scaled with the *matching* row count, not `limit`
            // (see BENCHMARKS.md's `execute_scan_limit_pushdown` numbers).
            let ids: Vec<u64> = label_index
                .get(label_id)?
                .take(limit)
                .map(|item| item.map(|g| g.value()))
                .collect::<Result<_, _>>()?;
            ids
        };
        let mut result = Vec::with_capacity(node_ids.len());
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        for id in node_ids {
            let guard = nodes.get(id)?.ok_or_else(|| {
                GraphError::CorruptData(format!("node label index references missing node {}", id))
            })?;
            let record: NodeRecord = decode(guard.value())?;
            drop(guard);
            let labels = record
                .label_ids
                .iter()
                .map(|&lid| resolve_label(txn, lid))
                .collect::<Result<Vec<_>, _>>()?;
            result.push(Node {
                id: NodeId(id),
                labels,
                props: record.props,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_check_rejects_missing_node_label_index_entry() {
        let mut store = GraphStore::open_memory().unwrap();
        let node = store.create_node(&["Person"], BTreeMap::new()).unwrap();

        let write = store.begin_write().unwrap();
        let label_id = {
            let labels = write
                .open_table(marsdb_storage::tables::LABEL_TO_ID)
                .unwrap();
            let id = labels.get("Person").unwrap().unwrap().value();
            id
        };
        write
            .open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)
            .unwrap()
            .remove(label_id, node.0)
            .unwrap();
        write.commit().unwrap();

        let error = store.check_integrity().unwrap_err();
        assert!(
            matches!(error, GraphError::CorruptData(message) if message.contains("missing from the label index"))
        );
    }

    #[test]
    fn integrity_check_rejects_dangling_adjacency_entry() {
        let mut store = GraphStore::open_memory().unwrap();
        let node = store.create_node(&[], BTreeMap::new()).unwrap();

        let write = store.begin_write().unwrap();
        let bytes = AdjEntry {
            edge_id: EdgeId(999),
            other: node,
            label_id: 0,
        }
        .encode();
        write
            .open_multimap_table(marsdb_storage::tables::ADJ_OUT)
            .unwrap()
            .insert(node.0, bytes.as_slice())
            .unwrap();
        write.commit().unwrap();

        let error = store.check_integrity().unwrap_err();
        assert!(
            matches!(error, GraphError::CorruptData(message) if message.contains("missing edge 999"))
        );
    }

    #[test]
    fn create_index_backfills_existing_nodes() {
        let store = GraphStore::open_memory().unwrap();
        let mut alice_props = BTreeMap::new();
        alice_props.insert(
            "email".to_string(),
            PropertyValue::String("alice@x.com".to_string()),
        );
        let alice = store.create_node(&["Person"], alice_props).unwrap();
        let mut bob_props = BTreeMap::new();
        bob_props.insert(
            "email".to_string(),
            PropertyValue::String("bob@x.com".to_string()),
        );
        store.create_node(&["Person"], bob_props).unwrap();
        // A Person with no email at all -- must not show up under any lookup.
        store.create_node(&["Person"], BTreeMap::new()).unwrap();

        store.create_index("Person", "email", false).unwrap();

        let found = store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("alice@x.com".to_string()),
            )
            .unwrap();
        assert_eq!(found, vec![alice]);
    }

    #[test]
    fn create_index_rejects_duplicate_unique_value() {
        let store = GraphStore::open_memory().unwrap();
        let mut props1 = BTreeMap::new();
        props1.insert(
            "email".to_string(),
            PropertyValue::String("same@x.com".to_string()),
        );
        store.create_node(&["Person"], props1).unwrap();
        let mut props2 = BTreeMap::new();
        props2.insert(
            "email".to_string(),
            PropertyValue::String("same@x.com".to_string()),
        );
        store.create_node(&["Person"], props2).unwrap();

        let error = store.create_index("Person", "email", true).unwrap_err();
        assert!(matches!(
            error,
            GraphError::UniqueConstraintViolation { .. }
        ));

        // A rejected unique index must not partially exist.
        assert!(store.index_def("Person", "email").unwrap().is_none());
    }

    #[test]
    fn lookup_by_index_on_undeclared_index_is_empty_not_an_error() {
        let store = GraphStore::open_memory().unwrap();
        store.create_node(&["Person"], BTreeMap::new()).unwrap();
        let found = store
            .lookup_by_index("Person", "email", &PropertyValue::String("x".to_string()))
            .unwrap();
        assert_eq!(found, Vec::new());
        assert!(store.index_def("Person", "email").unwrap().is_none());
    }

    #[test]
    fn index_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.db");
        {
            let store = GraphStore::open_file(&path).unwrap();
            let mut props = BTreeMap::new();
            props.insert(
                "email".to_string(),
                PropertyValue::String("x@x.com".to_string()),
            );
            store.create_node(&["Person"], props).unwrap();
            store.create_index("Person", "email", false).unwrap();
        }
        let store = GraphStore::open_file(&path).unwrap();
        assert!(store.index_def("Person", "email").unwrap().is_some());
        let found = store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("x@x.com".to_string()),
            )
            .unwrap();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn create_node_after_index_declared_is_indexed_immediately() {
        let store = GraphStore::open_memory().unwrap();
        store.create_index("Person", "email", false).unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("new@x.com".to_string()),
        );
        let node = store.create_node(&["Person"], props).unwrap();

        let found = store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("new@x.com".to_string()),
            )
            .unwrap();
        assert_eq!(found, vec![node]);
    }

    #[test]
    fn set_node_prop_moves_the_index_entry() {
        let store = GraphStore::open_memory().unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("old@x.com".to_string()),
        );
        let node = store.create_node(&["Person"], props).unwrap();
        store.create_index("Person", "email", false).unwrap();

        store
            .set_node_prop(
                node,
                "email",
                PropertyValue::String("new@x.com".to_string()),
            )
            .unwrap();

        assert!(store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("old@x.com".to_string())
            )
            .unwrap()
            .is_empty());
        assert_eq!(
            store
                .lookup_by_index(
                    "Person",
                    "email",
                    &PropertyValue::String("new@x.com".to_string())
                )
                .unwrap(),
            vec![node]
        );
    }

    #[test]
    fn set_node_prop_enforces_unique_index() {
        let store = GraphStore::open_memory().unwrap();
        let mut props1 = BTreeMap::new();
        props1.insert(
            "email".to_string(),
            PropertyValue::String("a@x.com".to_string()),
        );
        store.create_node(&["Person"], props1).unwrap();
        let mut props2 = BTreeMap::new();
        props2.insert(
            "email".to_string(),
            PropertyValue::String("b@x.com".to_string()),
        );
        let node2 = store.create_node(&["Person"], props2).unwrap();
        store.create_index("Person", "email", true).unwrap();

        let error = store
            .set_node_prop(node2, "email", PropertyValue::String("a@x.com".to_string()))
            .unwrap_err();
        assert!(matches!(
            error,
            GraphError::UniqueConstraintViolation { .. }
        ));
    }

    #[test]
    fn remove_node_prop_removes_the_index_entry() {
        let store = GraphStore::open_memory().unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("gone@x.com".to_string()),
        );
        let node = store.create_node(&["Person"], props).unwrap();
        store.create_index("Person", "email", false).unwrap();

        let write = store.begin_write().unwrap();
        GraphStore::remove_node_prop_in_txn(&write, node, "email").unwrap();
        write.commit().unwrap();

        assert!(store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("gone@x.com".to_string())
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn delete_node_removes_its_index_entries() {
        let store = GraphStore::open_memory().unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("deleted@x.com".to_string()),
        );
        let node = store.create_node(&["Person"], props).unwrap();
        store.create_index("Person", "email", false).unwrap();

        store.delete_node(node, false).unwrap();

        assert!(store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("deleted@x.com".to_string())
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn add_node_label_indexes_existing_props_under_the_new_label() {
        let store = GraphStore::open_memory().unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("multi@x.com".to_string()),
        );
        let node = store.create_node(&["Contact"], props).unwrap();
        store.create_index("Person", "email", false).unwrap();

        // Not indexed yet -- the node isn't a Person.
        assert!(store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("multi@x.com".to_string())
            )
            .unwrap()
            .is_empty());

        let write = store.begin_write().unwrap();
        GraphStore::add_node_label_in_txn(&write, node, "Person").unwrap();
        write.commit().unwrap();

        assert_eq!(
            store
                .lookup_by_index(
                    "Person",
                    "email",
                    &PropertyValue::String("multi@x.com".to_string())
                )
                .unwrap(),
            vec![node]
        );
    }

    #[test]
    fn remove_node_label_removes_index_entries_under_that_label() {
        let store = GraphStore::open_memory().unwrap();
        let mut props = BTreeMap::new();
        props.insert(
            "email".to_string(),
            PropertyValue::String("dual@x.com".to_string()),
        );
        let node = store.create_node(&["Person", "Contact"], props).unwrap();
        store.create_index("Person", "email", false).unwrap();
        assert_eq!(
            store
                .lookup_by_index(
                    "Person",
                    "email",
                    &PropertyValue::String("dual@x.com".to_string())
                )
                .unwrap(),
            vec![node]
        );

        let write = store.begin_write().unwrap();
        GraphStore::remove_node_label_in_txn(&write, node, "Person").unwrap();
        write.commit().unwrap();

        assert!(store
            .lookup_by_index(
                "Person",
                "email",
                &PropertyValue::String("dual@x.com".to_string())
            )
            .unwrap()
            .is_empty());
    }
}
