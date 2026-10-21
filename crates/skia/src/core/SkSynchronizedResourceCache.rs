// Copyright 2025 Google LLC. BSD-style license.
use super::SkCachedData::SkCachedData;
use super::SkResourceCache::{DiscardableFactory, FindVisitor, Key, Rec, SkResourceCache, Visitor};
use crate::include::private::SkMutex::{
    SkAutoMutexExclusive, SkAutoMutexExclusiveStorage, SkMutex,
};
use core::{cell::UnsafeCell, pin::Pin};
use std::ffi::c_void;
// Native derived member is destroyed before the actual base cache. UnsafeCell
// permits precisely the mutations protected by the one original native mutex.
// No Arc/Weak owner or independent cache allocation is introduced.
// Source: src/core/SkSynchronizedResourceCache.h:47-47
pub struct SkSynchronizedResourceCache {
    fMutex: SkMutex,
    base: UnsafeCell<SkResourceCache>,
}
unsafe impl Send for SkSynchronizedResourceCache {}
unsafe impl Sync for SkSynchronizedResourceCache {}
impl SkSynchronizedResourceCache {
    // Source: src/core/SkSynchronizedResourceCache.cpp:10-11
    /// Safety: factory returns real locked allocations or null, never unwinds
    /// or reenters this cache, and obeys its contract for every requested size.
    pub unsafe fn with_discardable_factory(fact: Option<DiscardableFactory>) -> Self {
        let base = unsafe { SkResourceCache::with_discardable_factory(fact) };
        Self {
            fMutex: SkMutex::new(),
            base: UnsafeCell::new(base),
        }
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:13-14
    pub fn with_byte_limit(byteLimit: usize) -> Self {
        let base = SkResourceCache::with_byte_limit(byteLimit);
        Self {
            fMutex: SkMutex::new(),
            base: UnsafeCell::new(base),
        }
    }
}
// Source: src/core/SkSynchronizedResourceCache.cpp:16-16
impl Drop for SkSynchronizedResourceCache {
    fn drop(&mut self) {} // Native default body, followed by owned-field/base teardown.
}
impl SkSynchronizedResourceCache {
    // Source: src/core/SkSynchronizedResourceCache.cpp:18-21
    pub fn getTotalBytesUsed(self: Pin<&Self>) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.getTotalBytesUsed()
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:23-26
    pub fn getTotalByteLimit(self: Pin<&Self>) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.getTotalByteLimit()
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:28-31
    pub fn setTotalByteLimit(self: Pin<&Self>, newLimit: usize) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        am.setTotalByteLimit(newLimit)
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:33-36
    pub fn discardableFactory(self: Pin<&Self>) -> Option<DiscardableFactory> {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.discardableFactory()
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:38-41
    pub fn newCachedData(self: Pin<&Self>, bytes: usize) -> Option<SkCachedData<()>> {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        am.newCachedData(bytes)
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:43-46
    pub fn dump(self: Pin<&Self>) {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.dump();
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:48-51
    pub fn setSingleAllocationByteLimit(self: Pin<&Self>, size: usize) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        am.setSingleAllocationByteLimit(size)
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:53-56
    pub fn getSingleAllocationByteLimit(self: Pin<&Self>) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.getSingleAllocationByteLimit()
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:58-61
    pub fn getEffectiveSingleAllocationByteLimit(self: Pin<&Self>) -> usize {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        am.getEffectiveSingleAllocationByteLimit()
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:63-66
    pub fn purgeAll(self: Pin<&Self>) {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        am.purgeAll();
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:68-71
    /// Safety: visitor/context obey the actual base contract. Lock is retained
    /// through message cleanup, visitor, LRU mutation and any stale-record Drop.
    pub unsafe fn find(
        self: Pin<&Self>,
        key: &Key,
        visitor: FindVisitor,
        context: *mut c_void,
    ) -> bool {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        unsafe { am.find(key, visitor, context) }
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:73-76
    /// Safety: transfer stable record ownership and valid payload; post-install
    /// callbacks/record destructors cannot unwind or reenter this same cache.
    pub unsafe fn add(self: Pin<&Self>, rec: Box<dyn Rec>, payload: *mut c_void) {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &mut *self.base.get() };
        unsafe {
            am.add(rec, payload);
        }
    }
    // Source: src/core/SkSynchronizedResourceCache.cpp:78-81
    /// Safety: visitor/context obey the actual base contract. Lock is held
    /// until every callback returns; borrowed records cannot escape callbacks.
    pub unsafe fn visitAll(self: Pin<&Self>, visitor: Visitor, context: *mut c_void) {
        let mut guard_storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { self.map_unchecked(|cache| &cache.fMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(guard_storage.as_mut(), mutex) };
        let am = unsafe { &*self.base.get() };
        unsafe {
            am.visitAll(visitor, context);
        }
    }
    // Source: src/core/SkResourceCache.cpp:515-517
    /// Safety: uphold the native unguarded base-call lease: no concurrent cache
    /// operation/record callbacks or outstanding base references. This method
    /// deliberately does not acquire the derived mutex absent from native code.
    pub(crate) unsafe fn checkMessages_unlocked(self: Pin<&Self>) {
        unsafe {
            (&mut *self.base.get()).checkMessages();
        }
    }
}

impl SkSynchronizedResourceCache {
    /// Safety: final pinned allocation, before publication/first cache operation.
    /// Construct/register the same native embedded base Inbox exactly once.
    pub unsafe fn init_owner(self: Pin<&mut Self>) {
        unsafe {
            SkResourceCache::init_owner(Pin::new_unchecked(&mut *self.base.get()));
        }
    }
}
