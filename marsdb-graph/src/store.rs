use std::collections::BTreeMap;
use std::path::Path;

use marsdb_storage::{ReadableMultimapTable, ReadableTable, StorageEngine, WriteTransaction};

use crate::encode::{decode, encode, EdgeRecord, NodeRecord};
use crate::error::GraphError;
use crate::id::next_id;
use crate::labels::{intern_label, lookup_label_id, resolve_label};
use crate::model::{AdjEntry, Direction, Edge, EdgeId, Node, NodeId, PropertyValue};

pub struct GraphStore {
    storage: StorageEngine,
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
        let mut label_index = write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        for label_id in label_ids {
            label_index.insert(label_id, id)?;
        }
        Ok(NodeId(id))
    }

    pub fn get_node(&self, id: NodeId) -> Result<Option<Node>, GraphError> {
        let write_txn = self.begin_write()?;
        let node = Self::get_node_in_txn(&write_txn, id)?;
        write_txn.abort()?;
        Ok(node)
    }

    pub fn get_node_in_txn(write_txn: &WriteTransaction, id: NodeId) -> Result<Option<Node>, GraphError> {
        let record: Option<NodeRecord> = {
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            let found = match nodes.get(id.0)? {
                Some(guard) => Some(decode(guard.value())?),
                None => None,
            };
            found
        };
        let Some(record) = record else { return Ok(None) };
        let labels = record
            .label_ids
            .iter()
            .map(|&lid| resolve_label(write_txn, lid))
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
        let write_txn = self.begin_write()?;
        let edge = Self::get_edge_in_txn(&write_txn, id)?;
        write_txn.abort()?;
        Ok(edge)
    }

    pub fn get_edge_in_txn(write_txn: &WriteTransaction, id: EdgeId) -> Result<Option<Edge>, GraphError> {
        let record: Option<EdgeRecord> = {
            let edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            let found = match edges.get(id.0)? {
                Some(guard) => Some(decode(guard.value())?),
                None => None,
            };
            found
        };
        let Some(record) = record else { return Ok(None) };
        let label = resolve_label(write_txn, record.label_id)?;
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
        let write_txn = self.begin_write()?;
        let result = Self::neighbors_in_txn(&write_txn, node, dir, label_filter)?;
        write_txn.abort()?;
        Ok(result)
    }

    pub fn neighbors_in_txn(
        write_txn: &WriteTransaction,
        node: NodeId,
        dir: Direction,
        label_filter: Option<&str>,
    ) -> Result<Vec<AdjEntry>, GraphError> {
        let label_id_filter = match label_filter {
            Some(l) => match lookup_label_id(write_txn, l)? {
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
        let table = write_txn.open_multimap_table(table_def)?;
        for item in table.get(node.0)? {
            let entry = AdjEntry::decode(item?.value());
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

    pub fn delete_edge_in_txn(write_txn: &WriteTransaction, id: EdgeId) -> Result<bool, GraphError> {
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

    pub fn delete_node_in_txn(write_txn: &WriteTransaction, id: NodeId, detach: bool) -> Result<bool, GraphError> {
        let mut incident: Vec<EdgeId> = Vec::new();
        {
            let adj_out = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_OUT)?;
            for item in adj_out.get(id.0)? {
                incident.push(AdjEntry::decode(item?.value()).edge_id);
            }
            let adj_in = write_txn.open_multimap_table(marsdb_storage::tables::ADJ_IN)?;
            for item in adj_in.get(id.0)? {
                incident.push(AdjEntry::decode(item?.value()).edge_id);
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
        let mut label_index = write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        for label_id in record.label_ids {
            label_index.remove(label_id, id.0)?;
        }
        Ok(true)
    }

    pub fn set_node_prop(&self, id: NodeId, key: &str, value: PropertyValue) -> Result<bool, GraphError> {
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
        let Some(bytes) = bytes_opt else { return Ok(false) };
        let mut record: NodeRecord = decode(&bytes)?;
        record.props.insert(key.to_string(), value);
        let new_bytes = encode(&record)?;
        let mut nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        nodes.insert(id.0, new_bytes.as_slice())?;
        Ok(true)
    }

    pub fn set_edge_prop(&self, id: EdgeId, key: &str, value: PropertyValue) -> Result<bool, GraphError> {
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
        let Some(bytes) = bytes_opt else { return Ok(false) };
        let mut record: EdgeRecord = decode(&bytes)?;
        record.props.insert(key.to_string(), value);
        let new_bytes = encode(&record)?;
        let mut edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
        edges.insert(id.0, new_bytes.as_slice())?;
        Ok(true)
    }

    /// Full scan of all nodes, optionally filtered by label. v1 has no
    /// secondary index on label, so this is a linear scan of the table.
    pub fn all_nodes(&self, label_filter: Option<&str>) -> Result<Vec<Node>, GraphError> {
        let write_txn = self.begin_write()?;
        let result = Self::all_nodes_in_txn(&write_txn, label_filter)?;
        write_txn.abort()?;
        Ok(result)
    }

    pub fn all_nodes_in_txn(write_txn: &WriteTransaction, label_filter: Option<&str>) -> Result<Vec<Node>, GraphError> {
        // A label filter goes through NODE_LABEL_INDEX (label_id -> node_ids)
        // plus a point lookup per match, instead of scanning every row in
        // NODES — cost scales with the number of matching rows, not the
        // table size. No filter means every row is wanted anyway, so a full
        // scan is already optimal; the index wouldn't help.
        let Some(label_filter) = label_filter else {
            let mut result = Vec::new();
            let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
            for item in nodes.iter()? {
                let (key, value) = item?;
                let record: NodeRecord = decode(value.value())?;
                let labels = record
                    .label_ids
                    .iter()
                    .map(|&lid| resolve_label(write_txn, lid))
                    .collect::<Result<Vec<_>, _>>()?;
                result.push(Node {
                    id: NodeId(key.value()),
                    labels,
                    props: record.props,
                });
            }
            return Ok(result);
        };
        let Some(label_id) = lookup_label_id(write_txn, label_filter)? else {
            return Ok(Vec::new());
        };
        let node_ids: Vec<u64> = {
            let label_index = write_txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
            let ids: Vec<u64> = label_index
                .get(label_id)?
                .map(|item| item.map(|g| g.value()))
                .collect::<Result<_, _>>()?;
            ids
        };
        let mut result = Vec::with_capacity(node_ids.len());
        let nodes = write_txn.open_table(marsdb_storage::tables::NODES)?;
        for id in node_ids {
            let guard = nodes
                .get(id)?
                .expect("node_label_index entry must reference a live node");
            let record: NodeRecord = decode(guard.value())?;
            drop(guard);
            let labels = record
                .label_ids
                .iter()
                .map(|&lid| resolve_label(write_txn, lid))
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
