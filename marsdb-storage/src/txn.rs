use std::borrow::Borrow;

use redb::{
    AccessGuard, Key, MultimapRange, MultimapTable, MultimapTableDefinition, MultimapValue, Range,
    ReadOnlyMultimapTable, ReadOnlyTable, ReadTransaction, ReadableMultimapTable, ReadableTable,
    ReadableTableMetadata, Table, TableDefinition, Value, WriteTransaction,
};

use crate::error::StorageError;

/// Either kind of redb transaction — lets a function that only ever reads
/// (`.get()`/`.iter()`, never `.insert()`/`.remove()`) run against a real
/// `WriteTransaction` (the crash-safety boundary for a write statement) or
/// a `ReadTransaction` (so a read-only statement doesn't have to contend
/// for redb's single-writer lock at all). `WriteTransaction`/
/// `ReadTransaction` share no common trait in redb itself — `open_table`/
/// `open_multimap_table` are inherent methods on two unrelated structs,
/// returning different concrete types (`Table`/`ReadOnlyTable`,
/// `MultimapTable`/`ReadOnlyMultimapTable`) — so this (and `TableHandle`/
/// `MultimapTableHandle` below) is a small local abstraction over that,
/// not something redb provides.
#[derive(Clone, Copy)]
pub enum Txn<'a> {
    Write(&'a WriteTransaction),
    Read(&'a ReadTransaction),
}

impl<'a> Txn<'a> {
    pub fn open_table<K: Key + 'static, V: Value + 'static>(
        &self,
        def: TableDefinition<K, V>,
    ) -> Result<TableHandle<'a, K, V>, StorageError> {
        Ok(match self {
            Txn::Write(w) => TableHandle::Write(w.open_table(def)?),
            Txn::Read(r) => TableHandle::Read(r.open_table(def)?),
        })
    }

    pub fn open_multimap_table<K: Key + 'static, V: Key + 'static>(
        &self,
        def: MultimapTableDefinition<K, V>,
    ) -> Result<MultimapTableHandle<'a, K, V>, StorageError> {
        Ok(match self {
            Txn::Write(w) => MultimapTableHandle::Write(w.open_multimap_table(def)?),
            Txn::Read(r) => MultimapTableHandle::Read(r.open_multimap_table(def)?),
        })
    }
}

/// Exposes `get`/`iter`/`range` as plain inherent methods, not a full
/// `redb::ReadableTable` trait impl — matching the whole trait (`first`/
/// `last`/...) would be boilerplate for methods nothing calls. `range`
/// was deliberately left out until a real call site needed it; v1.5's
/// composite-key adjacency (prefix scans over `ADJ_OUT`/`ADJ_IN`) is that
/// call site.
pub enum TableHandle<'a, K: Key + 'static, V: Value + 'static> {
    Write(Table<'a, K, V>),
    Read(ReadOnlyTable<K, V>),
}

impl<'a, K: Key + 'static, V: Value + 'static> TableHandle<'a, K, V> {
    pub fn get<'k>(
        &self,
        key: impl Borrow<K::SelfType<'k>>,
    ) -> Result<Option<AccessGuard<'_, V>>, StorageError> {
        Ok(match self {
            TableHandle::Write(t) => t.get(key)?,
            TableHandle::Read(t) => t.get(key)?,
        })
    }

    pub fn iter(&self) -> Result<Range<'_, K, V>, StorageError> {
        Ok(match self {
            TableHandle::Write(t) => t.iter()?,
            TableHandle::Read(t) => t.iter()?,
        })
    }

    /// Total entry count — O(1), redb tracks it per table. Added for the
    /// planner's start-point cardinality comparisons (an `AllNodesScan`
    /// candidate's cost is exactly this count for `NODES`), same
    /// "cheap count, never walk the entries" contract as
    /// `MultimapValue::len()` in `index::match_count`.
    // Fallible len can't back a conventional is_empty; no caller wants
    // one (the planner compares counts, never emptiness).
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<u64, StorageError> {
        Ok(match self {
            TableHandle::Write(t) => t.len()?,
            TableHandle::Read(t) => t.len()?,
        })
    }

    /// Key-ordered scan over a sub-range — the primitive behind composite-
    /// key prefix reads (`ADJ_OUT`/`ADJ_IN`'s `node ++ label` expansion)
    /// and, eventually, indexed range predicates over `PROPERTY_INDEX`
    /// (whose order-preserving value encoding has been range-ready since
    /// it was written).
    pub fn range<'k, KR: Borrow<K::SelfType<'k>> + 'k>(
        &self,
        range: impl std::ops::RangeBounds<KR> + 'k,
    ) -> Result<Range<'_, K, V>, StorageError> {
        Ok(match self {
            TableHandle::Write(t) => t.range(range)?,
            TableHandle::Read(t) => t.range(range)?,
        })
    }
}

pub enum MultimapTableHandle<'a, K: Key + 'static, V: Key + 'static> {
    Write(MultimapTable<'a, K, V>),
    Read(ReadOnlyMultimapTable<K, V>),
}

impl<'a, K: Key + 'static, V: Key + 'static> MultimapTableHandle<'a, K, V> {
    pub fn get<'k>(
        &self,
        key: impl Borrow<K::SelfType<'k>>,
    ) -> Result<MultimapValue<'_, V>, StorageError> {
        Ok(match self {
            MultimapTableHandle::Write(t) => t.get(key)?,
            MultimapTableHandle::Read(t) => t.get(key)?,
        })
    }

    #[allow(dead_code)] // not called by any current read path, kept for parity with TableHandle::iter
    pub fn iter(&self) -> Result<MultimapRange<'_, K, V>, StorageError> {
        Ok(match self {
            MultimapTableHandle::Write(t) => t.iter()?,
            MultimapTableHandle::Read(t) => t.iter()?,
        })
    }
}
