use std::borrow::Borrow;

use redb::{
    AccessGuard, Key, MultimapRange, MultimapTable, MultimapTableDefinition, MultimapValue, Range,
    ReadOnlyMultimapTable, ReadOnlyTable, ReadTransaction, ReadableMultimapTable, ReadableTable,
    ReadableTableMetadata, Table, TableDefinition, Value, WriteTransaction,
};

use crate::error::StorageError;

/// Either kind of redb transaction, so a read-only function can run
/// against a real `WriteTransaction` or a `ReadTransaction` (which avoids
/// contending for redb's single-writer lock). redb gives `open_table` on
/// two unrelated structs returning different concrete types (`Table`/
/// `ReadOnlyTable`, `MultimapTable`/`ReadOnlyMultimapTable`) with no
/// shared trait, so this and `TableHandle`/`MultimapTableHandle` below
/// paper over that.
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

/// Exposes `get`/`iter`/`range` as plain inherent methods rather than
/// implementing the full `redb::ReadableTable` trait, since nothing calls
/// the rest of it (`first`/`last`/...).
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

    /// Total entry count — O(1), redb tracks it per table. Used for the
    /// planner's start-point cardinality comparisons (an `AllNodesScan`
    /// candidate's cost is this count for `NODES`).
    // Fallible len can't back a conventional is_empty; no caller needs one.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Result<u64, StorageError> {
        Ok(match self {
            TableHandle::Write(t) => t.len()?,
            TableHandle::Read(t) => t.len()?,
        })
    }

    /// Key-ordered scan over a sub-range — backs composite-key prefix
    /// reads (`ADJ_OUT`/`ADJ_IN`'s `node ++ label` expansion).
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

    #[allow(dead_code)] // kept for parity with TableHandle::iter
    pub fn iter(&self) -> Result<MultimapRange<'_, K, V>, StorageError> {
        Ok(match self {
            MultimapTableHandle::Write(t) => t.iter()?,
            MultimapTableHandle::Read(t) => t.iter()?,
        })
    }

    /// Key-ordered scan over a sub-range of keys — the multimap
    /// counterpart of `TableHandle::range`, backing `PROPERTY_INDEX`
    /// range predicates.
    pub fn range<'k, KR: Borrow<K::SelfType<'k>> + 'k>(
        &self,
        range: impl std::ops::RangeBounds<KR> + 'k,
    ) -> Result<MultimapRange<'_, K, V>, StorageError> {
        Ok(match self {
            MultimapTableHandle::Write(t) => t.range(range)?,
            MultimapTableHandle::Read(t) => t.range(range)?,
        })
    }
}
