//! A write statement's table handles, opened lazily on first access and
//! cached for the rest of the call instead of every `*_in_txn` helper
//! reopening its own tables on every call.
//!
//! redb errors at runtime (`TableAlreadyOpen`) on a second live handle to
//! the same table from one `WriteTransaction`. `WriteCtx` avoids that by
//! ensuring each table it touches is opened at most once and reused for
//! the rest of the call, even when several helper functions in that call
//! each need the same table (e.g. `create_node_in_txn` needs `NODES` for
//! the insert and `index::on_node_created` needs `INDEX_DEFS`,
//! `PROPERTY_INDEX`, `ID_TO_LABEL`, `ID_TO_PROP` on top).
//!
//! Lazy (`Option` per table, opened on first access), not eager: most
//! calls only touch a handful of the 12 tables (`set_edge_prop_in_txn`
//! only needs `edges`), and a bulk-load benchmark showed eagerly opening
//! all 12 up front costs more than the redundant opens it would save.
//!
//! Scoped to one `*_in_txn` call, not a whole transaction: extending the
//! cache across a transaction would require every read that can happen
//! mid-write (property lookups, list ops, `existsPattern` subqueries in
//! the executor's `eval_return_expr` tree) to also route through these
//! same cached handles, or a `Txn::Write` read elsewhere in the same
//! transaction hits the same `TableAlreadyOpen` this exists to avoid --
//! a larger redesign of the read+write path across `marsdb-graph` and
//! `marsdb-query`'s executor.

use marsdb_storage::WriteTransaction;
use redb::{MultimapTable, Table};

use crate::error::GraphError;

pub(crate) struct WriteCtx<'txn> {
    write_txn: &'txn WriteTransaction,
    meta: Option<Table<'txn, &'static str, u64>>,
    label_to_id: Option<Table<'txn, &'static str, u32>>,
    id_to_label: Option<Table<'txn, u32, &'static str>>,
    nodes: Option<Table<'txn, u64, &'static [u8]>>,
    edges: Option<Table<'txn, u64, &'static [u8]>>,
    adj_out: Option<Table<'txn, (u64, u32, u64), u64>>,
    adj_in: Option<Table<'txn, (u64, u32, u64), u64>>,
    rel_type_counts: Option<Table<'txn, u32, u64>>,
    node_label_index: Option<MultimapTable<'txn, u32, u64>>,
    prop_to_id: Option<Table<'txn, &'static str, u32>>,
    id_to_prop: Option<Table<'txn, u32, &'static str>>,
    index_defs: Option<Table<'txn, &'static [u8], &'static [u8]>>,
    property_index: Option<MultimapTable<'txn, &'static [u8], u64>>,
}

macro_rules! table_accessor {
    ($name:ident, $field:ident, $def:expr, Table<$k:ty, $v:ty>) => {
        pub(crate) fn $name(&mut self) -> Result<&mut Table<'txn, $k, $v>, GraphError> {
            if self.$field.is_none() {
                self.$field = Some(self.write_txn.open_table($def)?);
            }
            Ok(self.$field.as_mut().unwrap())
        }
    };
}

macro_rules! multimap_accessor {
    ($name:ident, $field:ident, $def:expr, MultimapTable<$k:ty, $v:ty>) => {
        pub(crate) fn $name(&mut self) -> Result<&mut MultimapTable<'txn, $k, $v>, GraphError> {
            if self.$field.is_none() {
                self.$field = Some(self.write_txn.open_multimap_table($def)?);
            }
            Ok(self.$field.as_mut().unwrap())
        }
    };
}

impl<'txn> WriteCtx<'txn> {
    pub(crate) fn open(write_txn: &'txn WriteTransaction) -> Self {
        Self {
            write_txn,
            meta: None,
            label_to_id: None,
            id_to_label: None,
            nodes: None,
            edges: None,
            adj_out: None,
            adj_in: None,
            rel_type_counts: None,
            node_label_index: None,
            prop_to_id: None,
            id_to_prop: None,
            index_defs: None,
            property_index: None,
        }
    }

    table_accessor!(
        meta,
        meta,
        marsdb_storage::tables::META,
        Table<&'static str, u64>
    );
    table_accessor!(
        label_to_id,
        label_to_id,
        marsdb_storage::tables::LABEL_TO_ID,
        Table<&'static str, u32>
    );
    table_accessor!(id_to_label, id_to_label, marsdb_storage::tables::ID_TO_LABEL, Table<u32, &'static str>);
    table_accessor!(nodes, nodes, marsdb_storage::tables::NODES, Table<u64, &'static [u8]>);
    table_accessor!(edges, edges, marsdb_storage::tables::EDGES, Table<u64, &'static [u8]>);
    table_accessor!(
        adj_out,
        adj_out,
        marsdb_storage::tables::ADJ_OUT,
        Table<(u64, u32, u64), u64>
    );
    table_accessor!(
        adj_in,
        adj_in,
        marsdb_storage::tables::ADJ_IN,
        Table<(u64, u32, u64), u64>
    );
    table_accessor!(
        rel_type_counts,
        rel_type_counts,
        marsdb_storage::tables::REL_TYPE_COUNTS,
        Table<u32, u64>
    );
    multimap_accessor!(node_label_index, node_label_index, marsdb_storage::tables::NODE_LABEL_INDEX, MultimapTable<u32, u64>);
    table_accessor!(
        prop_to_id,
        prop_to_id,
        marsdb_storage::tables::PROP_TO_ID,
        Table<&'static str, u32>
    );
    table_accessor!(id_to_prop, id_to_prop, marsdb_storage::tables::ID_TO_PROP, Table<u32, &'static str>);
    table_accessor!(
        index_defs,
        index_defs,
        marsdb_storage::tables::INDEX_DEFS,
        Table<&'static [u8], &'static [u8]>
    );
    multimap_accessor!(
        property_index,
        property_index,
        marsdb_storage::tables::PROPERTY_INDEX,
        MultimapTable<&'static [u8], u64>
    );
}
