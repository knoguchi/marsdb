use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use marsdb_storage::{
    ReadTransaction, ReadableMultimapTable, ReadableTable, ReadableTableMetadata, StorageEngine,
    Txn, WriteTransaction,
};

use crate::encode::{
    decode_edge, decode_node, edge_header, encode_edge, encode_node, node_label_ids, EdgeRecord,
    NodeRecord,
};
use crate::error::GraphError;
use crate::id::next_id;
use crate::labels::{intern_label, lookup_label_id, resolve_label};
use crate::model::{AdjEntry, Direction, Edge, EdgeId, Node, NodeId, PropertyValue};
use crate::props::{intern_prop, prop_resolver};
use crate::write_ctx::WriteCtx;

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
        let store = Self {
            storage: StorageEngine::open_file(path)?,
        };
        store.backfill_rel_type_counts()?;
        Ok(store)
    }

    pub fn open_memory() -> Result<Self, GraphError> {
        Ok(Self {
            storage: StorageEngine::open_memory()?,
        })
    }

    /// One-time `REL_TYPE_COUNTS` rebuild for a file written by a build
    /// that predates the table: counts empty while `EDGES` isn't can only
    /// mean the maintaining writes never ran, so scan every edge header
    /// once (no property decode) and commit the tallies. A fresh or
    /// up-to-date file exits on the first check without writing anything.
    /// A file this build writes and an *older* build later mutates would
    /// go stale with no way to detect it here -- tolerable by
    /// construction, since the table is a planner statistic that can cost
    /// a suboptimal plan but never a wrong result (see its definition).
    fn backfill_rel_type_counts(&self) -> Result<(), GraphError> {
        let write_txn = self.begin_write()?;
        let up_to_date = {
            let counts = write_txn.open_table(marsdb_storage::tables::REL_TYPE_COUNTS)?;
            let edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            !counts.is_empty()? || edges.is_empty()?
        };
        if up_to_date {
            write_txn.abort()?;
            return Ok(());
        }
        let mut tallies: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
        {
            let edges = write_txn.open_table(marsdb_storage::tables::EDGES)?;
            for entry in edges.iter()? {
                let (_, value) = entry?;
                let (label_id, _, _) = edge_header(value.value())?;
                *tallies.entry(label_id).or_insert(0) += 1;
            }
        }
        {
            let mut counts = write_txn.open_table(marsdb_storage::tables::REL_TYPE_COUNTS)?;
            for (label_id, count) in tallies {
                counts.insert(label_id, count)?;
            }
        }
        write_txn.commit()?;
        Ok(())
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
                // Header-only read -- integrity checks node labels here,
                // never properties, so the directory stays untouched.
                let label_ids = node_label_ids(value.value())?;
                for label_id in &label_ids {
                    if !labels_by_id.contains_key(label_id) {
                        return Err(GraphError::CorruptData(format!(
                            "node {} references unknown label {}",
                            id.value(),
                            label_id
                        )));
                    }
                }
                nodes.insert(id.value(), label_ids);
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
                // Header-only, same reasoning as the node loop above.
                let (label_id, src, dst) = edge_header(value.value())?;
                if !labels_by_id.contains_key(&label_id) {
                    return Err(GraphError::CorruptData(format!(
                        "edge {} references unknown label {}",
                        id.value(),
                        label_id
                    )));
                }
                if !nodes.contains_key(&src) || !nodes.contains_key(&dst) {
                    return Err(GraphError::CorruptData(format!(
                        "edge {} references missing endpoint {} -> {}",
                        id.value(),
                        src,
                        dst
                    )));
                }
                edges.insert(id.value(), (label_id, src, dst));
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
        definition: marsdb_storage::TableDefinition<(u64, u32, u64), u64>,
        nodes: &BTreeMap<u64, Vec<u32>>,
        edges: &BTreeMap<u64, (u32, u64, u64)>,
        outgoing: bool,
    ) -> Result<BTreeSet<(u64, u64, u64, u32)>, GraphError> {
        let table = read.open_table(definition)?;
        let mut found = BTreeSet::new();
        for entry in table.iter()? {
            let (key, value) = entry?;
            let (owner, key_label_id, edge_id) = key.value();
            let other = value.value();
            if !nodes.contains_key(&owner) {
                return Err(GraphError::CorruptData(format!(
                    "adjacency references missing owner node {owner}"
                )));
            }
            let Some(&(label_id, src, dst)) = edges.get(&edge_id) else {
                return Err(GraphError::CorruptData(format!(
                    "adjacency references missing edge {edge_id}"
                )));
            };
            let expected = if outgoing { (src, dst) } else { (dst, src) };
            if owner != expected.0 || other != expected.1 || key_label_id != label_id {
                return Err(GraphError::CorruptData(format!(
                    "adjacency entry for edge {edge_id} does not match the edge record"
                )));
            }
            found.insert((owner, edge_id, other, key_label_id));
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
        let mut ctx = WriteCtx::open(write_txn);
        Self::create_node_ctx(&mut ctx, labels, props)
    }

    fn create_node_ctx(
        ctx: &mut WriteCtx,
        labels: &[&str],
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<NodeId, GraphError> {
        let label_ids = labels
            .iter()
            .map(|l| intern_label(ctx, l))
            .collect::<Result<Vec<_>, _>>()?;
        let id = next_id(ctx, "next_node_id")?;
        let record = NodeRecord {
            label_ids: label_ids.clone(),
            props,
        };
        let bytes = encode_node(&record, |name| intern_prop(ctx, name))?;
        ctx.nodes()?.insert(id, bytes.as_slice())?;
        for &label_id in &label_ids {
            ctx.node_label_index()?.insert(label_id, id)?;
        }
        crate::index::on_node_created(ctx, id, &label_ids, &record.props)?;
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
                Some(guard) => {
                    let mut resolve = prop_resolver(txn)?;
                    Some(decode_node(guard.value(), &mut resolve)?)
                }
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

    /// The id interned for a property name, if any -- `None` means the
    /// name has never been written anywhere, so no record can hold it.
    /// Exposed for the query layer's per-property read path: names resolve
    /// to ids once per statement there, then every row access goes through
    /// `get_node_prop_in_txn`/`get_edge_prop_in_txn` by id.
    pub fn lookup_prop_id_in_txn(txn: Txn, prop: &str) -> Result<Option<u32>, GraphError> {
        crate::props::lookup_prop_id(txn, prop)
    }

    /// One property of one node, by interned prop id, without decoding the
    /// rest of the record or resolving any names — a directory binary
    /// search plus one value decode (the v2 read fast path; the codec
    /// mechanism measured 79x over whole-record decode at 1-of-20 props).
    ///
    /// Nested `Option` distinguishes the two kinds of missing the executor
    /// must not collapse (`lookup_prop`'s own docs): outer `None` = the
    /// node record doesn't exist (deleted-entity error at the call site),
    /// inner `None` = node exists, property absent (legal null).
    pub fn get_node_prop_in_txn(
        txn: Txn,
        id: NodeId,
        prop_id: u32,
    ) -> Result<Option<Option<PropertyValue>>, GraphError> {
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        let Some(guard) = nodes.get(id.0)? else {
            return Ok(None);
        };
        match crate::encode::node_prop_raw(guard.value(), prop_id)? {
            Some(raw) => Ok(Some(Some(crate::encode::decode_value(raw)?))),
            None => Ok(Some(None)),
        }
    }

    /// Edge counterpart of `get_node_prop_in_txn`, same nested-`Option`
    /// contract.
    pub fn get_edge_prop_in_txn(
        txn: Txn,
        id: EdgeId,
        prop_id: u32,
    ) -> Result<Option<Option<PropertyValue>>, GraphError> {
        let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
        let Some(guard) = edges.get(id.0)? else {
            return Ok(None);
        };
        match crate::encode::edge_prop_raw(guard.value(), prop_id)? {
            Some(raw) => Ok(Some(Some(crate::encode::decode_value(raw)?))),
            None => Ok(Some(None)),
        }
    }

    /// Per-property reader over ONE pre-opened `NODES` handle -- for a
    /// caller probing many nodes' properties in a loop, where
    /// `get_node_prop_in_txn`'s per-call table open would dominate (the
    /// mars-3va lesson: opens measured 23.67% of a bulk load). Same
    /// nested-`Option` contract as `get_node_prop_in_txn`.
    #[allow(clippy::type_complexity)] // the nested Option IS the contract (see get_node_prop_in_txn)
    pub fn node_prop_reader(
        txn: Txn<'_>,
    ) -> Result<
        impl FnMut(NodeId, u32) -> Result<Option<Option<PropertyValue>>, GraphError> + '_,
        GraphError,
    > {
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        Ok(move |id: NodeId, prop_id: u32| {
            let Some(guard) = nodes.get(id.0)? else {
                return Ok(None);
            };
            match crate::encode::node_prop_raw(guard.value(), prop_id)? {
                Some(raw) => Ok(Some(Some(crate::encode::decode_value(raw)?))),
                None => Ok(Some(None)),
            }
        })
    }

    /// Record-existence check without any decoding — for the per-property
    /// read path when the property name was never interned (the value is
    /// necessarily absent on every record, but a *deleted* node must still
    /// error, not read as null).
    pub fn node_exists_in_txn(txn: Txn, id: NodeId) -> Result<bool, GraphError> {
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        let exists = nodes.get(id.0)?.is_some();
        Ok(exists)
    }

    /// Edge counterpart of `node_exists_in_txn`.
    pub fn edge_exists_in_txn(txn: Txn, id: EdgeId) -> Result<bool, GraphError> {
        let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
        let exists = edges.get(id.0)?.is_some();
        Ok(exists)
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
        let mut ctx = WriteCtx::open(write_txn);
        Self::create_edge_ctx(&mut ctx, label, src, dst, props)
    }

    fn create_edge_ctx(
        ctx: &mut WriteCtx,
        label: &str,
        src: NodeId,
        dst: NodeId,
        props: BTreeMap<String, PropertyValue>,
    ) -> Result<EdgeId, GraphError> {
        if ctx.nodes()?.get(src.0)?.is_none() {
            return Err(GraphError::NodeNotFound(src));
        }
        if ctx.nodes()?.get(dst.0)?.is_none() {
            return Err(GraphError::NodeNotFound(dst));
        }
        let label_id = intern_label(ctx, label)?;
        let id = next_id(ctx, "next_edge_id")?;
        let record = EdgeRecord {
            label_id,
            src: src.0,
            dst: dst.0,
            props,
        };
        let bytes = encode_edge(&record, |name| intern_prop(ctx, name))?;
        ctx.edges()?.insert(id, bytes.as_slice())?;

        ctx.adj_out()?
            .insert(crate::model::adj_key(src.0, label_id, id), dst.0)?;
        ctx.adj_in()?
            .insert(crate::model::adj_key(dst.0, label_id, id), src.0)?;
        Self::bump_rel_type_count(ctx, label_id, 1)?;
        Ok(EdgeId(id))
    }

    /// Adjust `REL_TYPE_COUNTS` for one edge born (`+1`) or dying (`-1`)
    /// -- called from the only two such places, `create_edge_ctx` and
    /// `delete_edge_ctx`. Saturating on the way down: a file written by
    /// a build that predates the table (or the backfill racing nothing
    /// -- see `backfill_rel_type_counts`) must degrade to a wrong
    /// *estimate*, never an underflow panic.
    fn bump_rel_type_count(
        ctx: &mut WriteCtx,
        label_id: u32,
        delta: i64,
    ) -> Result<(), GraphError> {
        let table = ctx.rel_type_counts()?;
        let current = table.get(label_id)?.map(|g| g.value()).unwrap_or(0);
        let next = current.saturating_add_signed(delta);
        table.insert(label_id, next)?;
        Ok(())
    }

    pub fn get_edge(&self, id: EdgeId) -> Result<Option<Edge>, GraphError> {
        let read_txn = self.begin_read()?;
        Self::get_edge_in_txn(Txn::Read(&read_txn), id)
    }

    pub fn get_edge_in_txn(txn: Txn, id: EdgeId) -> Result<Option<Edge>, GraphError> {
        let record: Option<EdgeRecord> = {
            let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
            let found = match edges.get(id.0)? {
                Some(guard) => {
                    let mut resolve = prop_resolver(txn)?;
                    Some(decode_edge(guard.value(), &mut resolve)?)
                }
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
        // Typed expansion narrows the key range itself (`node ++ label`
        // prefix) instead of post-filtering a full entry scan -- the
        // O(matching degree) fix this composite key layout exists for.
        let (lo, hi) = match label_filter {
            Some(l) => match lookup_label_id(txn, l)? {
                Some(lid) => crate::model::adj_label_bounds(node.0, lid),
                None => return Ok(Vec::new()),
            },
            None => crate::model::adj_node_bounds(node.0),
        };
        let mut result = Vec::new();
        let table_def = match dir {
            Direction::Out => marsdb_storage::tables::ADJ_OUT,
            Direction::In => marsdb_storage::tables::ADJ_IN,
        };
        let table = txn.open_table(table_def)?;
        for item in table.range(lo..=hi)? {
            let (key, value) = item?;
            let (_, label_id, edge_id) = key.value();
            result.push(AdjEntry {
                edge_id: EdgeId(edge_id),
                other: NodeId(value.value()),
                label_id,
            });
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
        let mut ctx = WriteCtx::open(write_txn);
        Ok(Self::delete_edge_ctx(&mut ctx, id)?.is_some())
    }

    /// Batch form of `delete_edge_in_txn`: one `WriteCtx` across every
    /// id instead of a fresh one (and its table opens) per edge, and the
    /// deleted edges' label names resolved here — once per distinct
    /// label id, while the ctx is already open — instead of a separate
    /// whole-edge fetch per id on the caller's side. Measured ~neutral
    /// on wall time vs the per-edge path (a scattered bulk delete's cost
    /// lives in the executor's match phase, not here — sorting the ids
    /// into per-table passes was tried too and moved nothing), so this
    /// exists for the API shape: one call for a `DELETE r` statement's
    /// whole edge set, doing strictly less redundant work. Returns
    /// `(id, label name)` for each edge that actually existed — an id
    /// already gone (a duplicate in `ids`, or deleted by an earlier
    /// statement) is silently skipped, same contract as the single-edge
    /// form's `false`.
    pub fn delete_edges_in_txn(
        write_txn: &WriteTransaction,
        ids: &[EdgeId],
    ) -> Result<Vec<(EdgeId, String)>, GraphError> {
        let mut ctx = WriteCtx::open(write_txn);
        let mut label_names: HashMap<u32, String> = HashMap::new();
        let mut deleted = Vec::with_capacity(ids.len());
        for &id in ids {
            let Some(label_id) = Self::delete_edge_ctx(&mut ctx, id)? else {
                continue;
            };
            let name = match label_names.entry(label_id) {
                std::collections::hash_map::Entry::Occupied(e) => e.get().clone(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    let name = ctx
                        .id_to_label()?
                        .get(label_id)?
                        .ok_or_else(|| {
                            GraphError::CorruptData(format!(
                                "label id {label_id} has no interned string"
                            ))
                        })?
                        .value()
                        .to_string();
                    e.insert(name).clone()
                }
            };
            deleted.push((id, name));
        }
        Ok(deleted)
    }

    /// Internal, `WriteCtx`-based logic -- `delete_node_in_txn` calls this
    /// directly (not the public `delete_edge_in_txn` wrapper) for each of a
    /// deleted node's incident edges, since it already has its own `ctx`
    /// open for the same transaction; opening a second `WriteCtx` on top of
    /// it would try to open every table twice and hit redb's
    /// `TableAlreadyOpen`. Returns the deleted edge's label id, `None` if
    /// the edge didn't exist.
    fn delete_edge_ctx(ctx: &mut WriteCtx, id: EdgeId) -> Result<Option<u32>, GraphError> {
        let Some(record_bytes) = ctx
            .edges()?
            .remove(id.0)?
            .map(|guard| guard.value().to_vec())
        else {
            return Ok(None);
        };
        // Header-only read: adjacency cleanup needs (label, src, dst),
        // never the edge's properties -- skips every prop-name resolution.
        let (label_id, src, dst) = edge_header(&record_bytes)?;
        ctx.adj_out()?
            .remove(crate::model::adj_key(src, label_id, id.0))?;
        ctx.adj_in()?
            .remove(crate::model::adj_key(dst, label_id, id.0))?;
        Self::bump_rel_type_count(ctx, label_id, -1)?;
        Ok(Some(label_id))
    }

    /// Delete a node. If `detach` is false and the node has incident edges,
    /// returns `GraphError::NodeHasEdges` instead of deleting anything.
    pub fn delete_node(&self, id: NodeId, detach: bool) -> Result<bool, GraphError> {
        let write_txn = self.begin_write()?;
        let existed = Self::delete_node_in_txn(&write_txn, id, detach)?.is_some();
        write_txn.commit()?;
        Ok(existed)
    }

    /// Returns `None` when the node didn't exist, else
    /// `Some(incident edges actually deleted)` — the caller-visible
    /// count a `DETACH DELETE` needs for its statement stats (a
    /// self-loop appears in both adjacency directions but deletes
    /// once, so the count comes from the deletions, not the scan).
    pub fn delete_node_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        detach: bool,
    ) -> Result<Option<u64>, GraphError> {
        let mut ctx = WriteCtx::open(write_txn);
        let mut incident: Vec<EdgeId> = Vec::new();
        let (lo, hi) = crate::model::adj_node_bounds(id.0);
        for item in ctx.adj_out()?.range(lo..=hi)? {
            let (key, _) = item?;
            let (_, _, edge_id) = key.value();
            incident.push(EdgeId(edge_id));
        }
        for item in ctx.adj_in()?.range(lo..=hi)? {
            let (key, _) = item?;
            let (_, _, edge_id) = key.value();
            incident.push(EdgeId(edge_id));
        }
        if !incident.is_empty() && !detach {
            return Err(GraphError::NodeHasEdges(id));
        }
        let mut edges_deleted: u64 = 0;
        for edge_id in incident {
            if Self::delete_edge_ctx(&mut ctx, edge_id)?.is_some() {
                edges_deleted += 1;
            }
        }
        let Some(removed_bytes) = ctx
            .nodes()?
            .remove(id.0)?
            .map(|guard| guard.value().to_vec())
        else {
            return Ok(None);
        };
        let record = decode_node(&removed_bytes, |pid| {
            crate::index::resolve_prop_ctx(&mut ctx, pid)
        })?;
        for &label_id in &record.label_ids {
            ctx.node_label_index()?.remove(label_id, id.0)?;
        }
        crate::index::on_node_deleted(&mut ctx, id.0, &record.label_ids, &record.props)?;
        Ok(Some(edges_deleted))
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
        let mut ctx = WriteCtx::open(write_txn);
        crate::index::create_index(&mut ctx, label, prop, unique)
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
    /// Range counterpart of `lookup_by_index_in_txn` — every node whose
    /// indexed value falls within the bounds (`(value, inclusive)` per
    /// side, either side open). Returns a SUPERSET for numeric bounds
    /// (int/float regions both scanned, lossy conversions widened
    /// outward) — callers must re-check the original predicate; see
    /// `index::lookup_range`.
    pub fn lookup_by_index_range_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
        lo: Option<(&PropertyValue, bool)>,
        hi: Option<(&PropertyValue, bool)>,
        limit: Option<usize>,
    ) -> Result<Vec<NodeId>, GraphError> {
        crate::index::lookup_range(txn, label, prop, lo, hi, limit)
    }

    /// Resumable form of `lookup_by_index_range_in_txn` — see
    /// `index::IndexRangeCursor` for the demand-driven contract.
    pub fn index_range_cursor_in_txn(
        txn: Txn,
        label: &str,
        prop: &str,
        lo: Option<(&PropertyValue, bool)>,
        hi: Option<(&PropertyValue, bool)>,
    ) -> Result<Option<crate::index::IndexRangeCursor>, GraphError> {
        crate::index::IndexRangeCursor::new(txn, label, prop, lo, hi)
    }

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
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.nodes()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        let mut record = decode_node(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        let old_value = record.props.insert(key.to_string(), value.clone());
        let new_bytes = encode_node(&record, |name| intern_prop(&mut ctx, name))?;
        ctx.nodes()?.insert(id.0, new_bytes.as_slice())?;
        crate::index::on_node_prop_changed(
            &mut ctx,
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
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.edges()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        let mut record = decode_edge(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        record.props.insert(key.to_string(), value);
        let new_bytes = encode_edge(&record, |name| intern_prop(&mut ctx, name))?;
        ctx.edges()?.insert(id.0, new_bytes.as_slice())?;
        Ok(true)
    }

    pub fn remove_node_prop_in_txn(
        write_txn: &WriteTransaction,
        id: NodeId,
        key: &str,
    ) -> Result<bool, GraphError> {
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.nodes()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        let mut record = decode_node(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        let old_value = record.props.remove(key);
        let new_bytes = encode_node(&record, |name| intern_prop(&mut ctx, name))?;
        ctx.nodes()?.insert(id.0, new_bytes.as_slice())?;
        crate::index::on_node_prop_changed(
            &mut ctx,
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
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.edges()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        let mut record = decode_edge(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        record.props.remove(key);
        let new_bytes = encode_edge(&record, |name| intern_prop(&mut ctx, name))?;
        ctx.edges()?.insert(id.0, new_bytes.as_slice())?;
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
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.nodes()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        let mut record = decode_node(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        let label_id = intern_label(&mut ctx, label)?;
        if !record.label_ids.contains(&label_id) {
            record.label_ids.push(label_id);
            let new_bytes = encode_node(&record, |name| intern_prop(&mut ctx, name))?;
            ctx.nodes()?.insert(id.0, new_bytes.as_slice())?;
            ctx.node_label_index()?.insert(label_id, id.0)?;
            crate::index::on_node_created(&mut ctx, id.0, &[label_id], &record.props)?;
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
        let mut ctx = WriteCtx::open(write_txn);
        let Some(bytes) = ctx.nodes()?.get(id.0)?.map(|g| g.value().to_vec()) else {
            return Ok(false);
        };
        // Same lookup as `labels::lookup_label_id`, but reading directly
        // from the already-open `ctx.label_to_id` -- a second `Txn`-based
        // open of the same table would be `TableAlreadyOpen`.
        let Some(label_id) = ctx.label_to_id()?.get(label)?.map(|g| g.value()) else {
            return Ok(true);
        };
        let mut record = decode_node(&bytes, |pid| crate::index::resolve_prop_ctx(&mut ctx, pid))?;
        if let Some(pos) = record.label_ids.iter().position(|&l| l == label_id) {
            record.label_ids.remove(pos);
            let new_bytes = encode_node(&record, |name| intern_prop(&mut ctx, name))?;
            ctx.nodes()?.insert(id.0, new_bytes.as_slice())?;
            ctx.node_label_index()?.remove(label_id, id.0)?;
            crate::index::on_node_deleted(&mut ctx, id.0, &[label_id], &record.props)?;
        }
        Ok(true)
    }

    /// Total node count — O(1) (redb tracks table entry counts). For the
    /// query planner's start-point cardinality comparison: the cost of an
    /// `AllNodesScan` leaf, never for fetching anything.
    pub fn node_count_in_txn(txn: Txn) -> Result<u64, GraphError> {
        let nodes = txn.open_table(marsdb_storage::tables::NODES)?;
        Ok(nodes.len()?)
    }

    /// Number of nodes carrying `label` — O(1) via the label index's
    /// per-key entry count (same mechanism as `index_match_count_in_txn`).
    /// An unknown label reads as 0, same as everywhere else. Planner
    /// cardinality use only, like `node_count_in_txn`.
    pub fn label_count_in_txn(txn: Txn, label: &str) -> Result<u64, GraphError> {
        let Some(label_id) = lookup_label_id(txn, label)? else {
            return Ok(0);
        };
        let index = txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        let count = index.get(label_id)?.len();
        Ok(count)
    }

    /// Every interned name currently carried by at least one node, as
    /// `(label, node count)` sorted by label — the substance behind
    /// `CALL db.labels()`. Node labels and relationship types share one
    /// intern namespace (`intern_label` serves both), so membership here
    /// is decided by live *use* (a nonzero label-index count), not by
    /// interning: a name whose nodes were all deleted drops out, same as
    /// a name only ever used as a relationship type never appears.
    /// O(interned names), each with an O(1) count read.
    pub fn list_node_labels_in_txn(txn: Txn) -> Result<Vec<(String, u64)>, GraphError> {
        let l2i = txn.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
        let index = txn.open_multimap_table(marsdb_storage::tables::NODE_LABEL_INDEX)?;
        let mut out = Vec::new();
        for entry in l2i.iter()? {
            let (name, id) = entry?;
            let count = index.get(id.value())?.len();
            if count > 0 {
                out.push((name.value().to_string(), count));
            }
        }
        Ok(out)
    }

    /// Relationship-type counterpart of `list_node_labels_in_txn`:
    /// `(type, live edge count)` sorted by type, counts from
    /// `REL_TYPE_COUNTS`. Same live-use membership rule.
    pub fn list_rel_types_in_txn(txn: Txn) -> Result<Vec<(String, u64)>, GraphError> {
        let l2i = txn.open_table(marsdb_storage::tables::LABEL_TO_ID)?;
        let counts = txn.open_table(marsdb_storage::tables::REL_TYPE_COUNTS)?;
        let mut out = Vec::new();
        for entry in l2i.iter()? {
            let (name, id) = entry?;
            let count = counts.get(id.value())?.map(|g| g.value()).unwrap_or(0);
            if count > 0 {
                out.push((name.value().to_string(), count));
            }
        }
        Ok(out)
    }

    /// Every interned property name, sorted — `CALL db.propertyKeys()`.
    /// Interning is permanent (there is no un-intern on last use, unlike
    /// the liveness rule above, which has cheap per-name counts to
    /// consult), so this lists every key that has ever appeared.
    pub fn list_property_keys_in_txn(txn: Txn) -> Result<Vec<String>, GraphError> {
        let p2i = txn.open_table(marsdb_storage::tables::PROP_TO_ID)?;
        let mut out = Vec::new();
        for entry in p2i.iter()? {
            let (name, _) = entry?;
            out.push(name.value().to_string());
        }
        Ok(out)
    }

    /// Every declared index as `(label, property, unique)` —
    /// `CALL db.indexes()`. Full `INDEX_DEFS` scan; the number of
    /// declared indexes is small by nature.
    pub fn list_indexes_in_txn(txn: Txn) -> Result<Vec<(String, String, bool)>, GraphError> {
        let mut resolve_prop = prop_resolver(txn)?;
        let defs = txn.open_table(marsdb_storage::tables::INDEX_DEFS)?;
        let mut out = Vec::new();
        for entry in defs.iter()? {
            let (key, value) = entry?;
            let key_bytes = key.value();
            let label_id = u32::from_be_bytes(key_bytes[0..4].try_into().map_err(|_| {
                GraphError::CorruptData("index key prefix shorter than 8 bytes".into())
            })?);
            let prop_id = u32::from_be_bytes(key_bytes[4..8].try_into().map_err(|_| {
                GraphError::CorruptData("index key prefix shorter than 8 bytes".into())
            })?);
            let def: crate::IndexDef = postcard::from_bytes(value.value())
                .map_err(|e| GraphError::CorruptData(format!("undecodable index def: {e}")))?;
            out.push((
                resolve_label(txn, label_id)?,
                resolve_prop(prop_id)?,
                def.unique,
            ));
        }
        Ok(out)
    }

    /// Resolve a label/relationship-type name to its interned id —
    /// `None` if never interned. Scan-support API (`EdgeScanCursor`
    /// consumers pre-resolve type names once per scan).
    pub fn label_id_for(txn: Txn, name: &str) -> Result<Option<u32>, GraphError> {
        lookup_label_id(txn, name)
    }

    /// Header fields `(label_id, src, dst)` of a raw edge record as
    /// returned by `EdgeScanCursor` — no property decode.
    pub fn edge_record_header(bytes: &[u8]) -> Result<(u32, u64, u64), GraphError> {
        edge_header(bytes)
    }

    /// One property's value from a raw edge record, by interned prop
    /// id — a directory-entry read from in-hand bytes, no storage
    /// access. `Ok(None)` = property absent on this edge.
    pub fn edge_record_prop(
        bytes: &[u8],
        prop_id: u32,
    ) -> Result<Option<PropertyValue>, GraphError> {
        match crate::encode::edge_prop_raw(bytes, prop_id)? {
            Some(raw) => Ok(Some(crate::encode::decode_value(raw)?)),
            None => Ok(None),
        }
    }

    /// Resumable chunked sweep over the whole `EDGES` table in id
    /// order — the sequential-scan primitive behind the planner's
    /// `EdgeTypeScan`. Same demand-driven shape as `IndexRangeCursor`:
    /// each `next_chunk` re-seeks past the last returned id (O(log n))
    /// and copies at most `chunk_size` raw records out, so a `LIMIT`ed
    /// consumer that stops early never pays for the rest of the table.
    pub fn edge_scan_cursor() -> EdgeScanCursor {
        EdgeScanCursor { resume_after: None }
    }

    /// Total edge count — O(1) (redb tracks table entry counts), the
    /// edge counterpart of `node_count_in_txn`. Planner cardinality use
    /// only.
    pub fn edge_count_in_txn(txn: Txn) -> Result<u64, GraphError> {
        let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
        Ok(edges.len()?)
    }

    /// Number of live edges of relationship type `rel_type` — O(1) via
    /// `REL_TYPE_COUNTS` (see its definition in `tables.rs` for the
    /// maintenance/backfill story). An unknown type reads as 0, same as
    /// `label_count_in_txn`. Planner cardinality use only.
    pub fn rel_type_count_in_txn(txn: Txn, rel_type: &str) -> Result<u64, GraphError> {
        let Some(label_id) = lookup_label_id(txn, rel_type)? else {
            return Ok(0);
        };
        let counts = txn.open_table(marsdb_storage::tables::REL_TYPE_COUNTS)?;
        let count = counts.get(label_id)?.map(|g| g.value()).unwrap_or(0);
        Ok(count)
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
            // Resolver hoisted out of the loop: one ID_TO_PROP open for the
            // whole scan, not one per record (table opens were themselves a
            // measured hot cost -- mars-3va).
            let mut resolve = prop_resolver(txn)?;
            for item in nodes.iter()? {
                if result.len() >= limit {
                    break;
                }
                let (key, value) = item?;
                let record = decode_node(value.value(), &mut resolve)?;
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
        // Same loop-hoisted resolver as the unfiltered scan above.
        let mut resolve = prop_resolver(txn)?;
        for id in node_ids {
            let guard = nodes.get(id)?.ok_or_else(|| {
                GraphError::CorruptData(format!("node label index references missing node {}", id))
            })?;
            let record = decode_node(guard.value(), &mut resolve)?;
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

/// See `GraphStore::edge_scan_cursor`.
pub struct EdgeScanCursor {
    resume_after: Option<u64>,
}

impl EdgeScanCursor {
    /// At most `chunk_size` `(edge_id, raw record bytes)` pairs, id
    /// order, starting after the previous chunk's last id. Empty vec =
    /// table exhausted.
    pub fn next_chunk(
        &mut self,
        txn: Txn,
        chunk_size: usize,
    ) -> Result<Vec<(u64, Vec<u8>)>, GraphError> {
        if chunk_size == 0 {
            return Ok(Vec::new());
        }
        let edges = txn.open_table(marsdb_storage::tables::EDGES)?;
        let mut out = Vec::with_capacity(chunk_size.min(1024));
        let iter = match self.resume_after {
            Some(last) => {
                edges.range::<u64>((std::ops::Bound::Excluded(last), std::ops::Bound::Unbounded))?
            }
            None => edges.iter()?,
        };
        for entry in iter {
            let (id, value) = entry?;
            let id = id.value();
            out.push((id, value.value().to_vec()));
            self.resume_after = Some(id);
            if out.len() >= chunk_size {
                break;
            }
        }
        Ok(out)
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
        write
            .open_table(marsdb_storage::tables::ADJ_OUT)
            .unwrap()
            .insert(crate::model::adj_key(node.0, 0, 999), node.0)
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
