use std::borrow::Borrow;

use redb::{
    AccessGuard, Key, MultimapRange, MultimapTable, MultimapTableDefinition, MultimapValue, ReadOnlyMultimapTable,
    ReadOnlyTable, ReadTransaction, ReadableMultimapTable, ReadableTable, Range, Table, TableDefinition, Value,
    WriteTransaction,
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

/// Deliberately exposes only `get`/`iter` as plain inherent methods, not a
/// full `redb::ReadableTable` trait impl — every read-only call site in
/// this codebase today only ever calls those two (no `.range()`/
/// `.first()`/`.last()` anywhere), so matching the full trait (including
/// its lifetime-heavy generic `range`) would be boilerplate for methods
/// nothing calls.
pub enum TableHandle<'a, K: Key + 'static, V: Value + 'static> {
    Write(Table<'a, K, V>),
    Read(ReadOnlyTable<K, V>),
}

impl<'a, K: Key + 'static, V: Value + 'static> TableHandle<'a, K, V> {
    pub fn get<'k>(&self, key: impl Borrow<K::SelfType<'k>>) -> Result<Option<AccessGuard<'_, V>>, StorageError> {
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
}

pub enum MultimapTableHandle<'a, K: Key + 'static, V: Key + 'static> {
    Write(MultimapTable<'a, K, V>),
    Read(ReadOnlyMultimapTable<K, V>),
}

impl<'a, K: Key + 'static, V: Key + 'static> MultimapTableHandle<'a, K, V> {
    pub fn get<'k>(&self, key: impl Borrow<K::SelfType<'k>>) -> Result<MultimapValue<'_, V>, StorageError> {
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
