// Copyright 2013 Google Inc. BSD-style license.
use super::SkTHash::{HashTraits, THashTable};
use core::marker::PhantomData;
// Native Traits static-method protocol. Raw entries are borrowed, never owned.
pub trait DynamicHashTraits<T, K: ?Sized> {
    fn GetKey(value: &T) -> &K;
    fn Hash(key: &K) -> u32;
}
// Source: src/core/SkTDynamicHash.h:51-54
struct AdaptedTraits<T, Traits>(PhantomData<(T, Traits)>);
impl<T, K: ?Sized, Traits: DynamicHashTraits<T, K>> HashTraits<*mut T, K>
    for AdaptedTraits<T, Traits>
{
    type Key<'a>
        = &'a K
    where
        T: 'a,
        K: 'a;
    fn GetKey<'a>(entry: &'a *mut T) -> Self::Key<'a>
    where
        K: 'a,
    {
        unsafe { Traits::GetKey(&**entry) }
    }
    fn Hash(key: &K) -> u32 {
        Traits::Hash(key)
    }
}
// Source: src/core/SkTDynamicHash.h:19-24
// Source: src/core/SkTDynamicHash.h:55-55
pub struct SkTDynamicHash<T, K: ?Sized, Traits = T> {
    fTable: THashTable<*mut T, K, AdaptedTraits<T, Traits>>,
}
impl<T, K: ?Sized, Traits> SkTDynamicHash<T, K, Traits> {
    // Source: src/core/SkTDynamicHash.h:24-24
    pub fn new() -> Self {
        Self {
            fTable: THashTable::default(),
        }
    }
    // Source: src/core/SkTDynamicHash.h:29-32
    pub fn foreach_mut(&mut self, mut f: impl FnMut(*mut T)) {
        self.fTable.foreach_mut(|entry| f(*entry));
    }
    // Source: src/core/SkTDynamicHash.h:33-36
    pub fn foreach(&self, mut f: impl FnMut(*mut T)) {
        self.fTable.foreach(|entry| f(*entry));
    }
    // Source: src/core/SkTDynamicHash.h:38-38
    pub fn count(&self) -> i32 {
        self.fTable.count()
    }
    // Source: src/core/SkTDynamicHash.h:40-40
    pub fn approxBytesUsed(&self) -> usize {
        self.fTable.approxBytesUsed()
    }
    // Source: src/core/SkTDynamicHash.h:47-48
    pub fn rewind(&mut self) {
        self.fTable.reset();
    }
    pub fn reset(&mut self) {
        self.fTable.reset();
    }
}
impl<T, K: PartialEq + ?Sized, Traits: DynamicHashTraits<T, K>> SkTDynamicHash<T, K, Traits> {
    // Source: src/core/SkTDynamicHash.h:42-42
    pub fn find(&self, key: &K) -> *mut T {
        self.fTable.findOrNull(key).unwrap_or(core::ptr::null_mut())
    }
    // Source: src/core/SkTDynamicHash.h:44-44
    /// Safety: entry is live, nonnull and its key stays immutable while indexed.
    /// Entries outlive all key lookups; no ownership is transferred to the table.
    pub unsafe fn add(&mut self, entry: *mut T) {
        self.fTable.set(entry);
    }
    // Source: src/core/SkTDynamicHash.h:45-45
    pub fn remove(&mut self, key: &K) {
        self.fTable.remove(key);
    }
}
