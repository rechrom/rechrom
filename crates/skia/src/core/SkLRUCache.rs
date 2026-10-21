//! Native THashTable plus intrusive recency list. Pinning expresses the native
//! nonmoving cache and preserves node debug back-pointers; entries are actual owned nodes.
use super::SkTHash::{HashTraits, NativeHash, THashTable};
use super::SkTInternalLList::{Iter, IterStart, Linked, Links, SkTInternalLList};
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::ptr::NonNull;

pub unsafe trait PurgeCallback<K, V>: Default {
    /// Context and borrows have the native synchronous lifetime; do not retain
    /// the entry or mutate this cache from the callback.
    unsafe fn purge(&self, context: *mut c_void, key: &K, value: &V);
}
// Source: src/core/SkLRUCache.h:15-18
#[derive(Default)]
pub struct SkNoOpPurge;
unsafe impl<K, V> PurgeCallback<K, V> for SkNoOpPurge {
    unsafe fn purge(&self, _: *mut c_void, _: &K, _: &V) {}
}
// Source: src/core/SkLRUCache.h:26-36
struct Entry<K, V> {
    fKey: ManuallyDrop<K>,
    fValue: ManuallyDrop<V>,
    links: Links<Self>,
}
impl<K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            fKey: ManuallyDrop::new(key),
            fValue: ManuallyDrop::new(value),
            links: Links::default(),
        }
    }
}
impl<K, V> Drop for Entry<K, V> {
    fn drop(&mut self) {
        // Native implicit member destruction is reverse declaration order.
        // Rust's default field drop order would destroy the key first.
        unsafe {
            ManuallyDrop::drop(&mut self.fValue);
            ManuallyDrop::drop(&mut self.fKey);
        }
    }
}
unsafe impl<K, V> Linked for Entry<K, V> {
    unsafe fn links(node: *mut Self) -> *mut Links<Self> {
        std::ptr::addr_of_mut!((*node).links)
    }
}
impl<K: PartialEq, V, H: NativeHash<K>, P: PurgeCallback<K, V>> SkLRUCache<K, V, H, P> {
    // Source: src/core/SkLRUCache.h:39-42
    /// The callback context must remain valid through cache destruction, including
    /// on another thread when the cache is transferred. Native insertion requires
    /// a nonnegative capacity; zero capacity returns an already-evicted raw pointer.
    pub unsafe fn new(maxCount: i32, context: *mut c_void) -> Pin<Box<Self>> {
        Box::pin(Self {
            fMaxCount: maxCount,
            fMap: THashTable::default(),
            fLRU: SkTInternalLList::new(),
            fContext: context,
            _types: PhantomData,
            _pin: PhantomPinned,
        })
    }
}
// Source: src/core/SkLRUCache.h:44-51
impl<K, V, H, P> Drop for SkLRUCache<K, V, H, P> {
    fn drop(&mut self) {
        unsafe {
            let mut node = self.fLRU.head();
            while !node.is_null() {
                self.fLRU.remove(node);
                drop(Box::from_raw(node));
                node = self.fLRU.head();
            }
        }
        // Native destructor does not invoke PurgeCB and destroys fMap afterwards.
    }
}
impl<K: PartialEq, V, H: NativeHash<K>, P: PurgeCallback<K, V>> SkLRUCache<K, V, H, P> {
    // Source: src/core/SkLRUCache.h:57-68
    pub fn find(self: Pin<&mut Self>, key: &K) -> Option<&mut V> {
        unsafe {
            let this = self.get_unchecked_mut();
            let entry = this.fMap.find(key)?.as_ptr();
            if entry != this.fLRU.head() {
                this.fLRU.remove(entry);
                this.fLRU.addToHead(entry);
            }
            Some(&mut (*entry).fValue)
        }
    }
    // Source: src/core/SkLRUCache.h:70-77
    unsafe fn insert_entry(mut self: Pin<&mut Self>, entry: *mut Entry<K, V>) -> *mut V {
        {
            let this = self.as_mut().get_unchecked_mut();
            this.fMap.set(NonNull::new_unchecked(entry));
            this.fLRU.addToHead(entry);
        }
        while self.fMap.count() > self.fMaxCount {
            let tail = self.fLRU.tail();
            self.as_mut().remove(&(*tail).fKey);
        }
        // addr_of_mut does not create a reference: maxCount=0 has native dangling-pointer semantics.
        std::ptr::addr_of_mut!((*entry).fValue).cast()
    }
    // Source: src/core/SkLRUCache.h:79-83
    pub unsafe fn insert(mut self: Pin<&mut Self>, key: K, value: V) -> *mut V {
        crate::SkASSERT!(@source b"src/core/SkLRUCache.h\0".as_ptr(),81; @condition b"!this->find(key)\0".as_ptr(); self.as_mut().find(&key).is_none());
        self.insert_entry(Box::into_raw(Box::new(Entry::new(key, value))))
    }
    // Source: src/core/SkLRUCache.h:85-92
    pub unsafe fn insert_or_update(mut self: Pin<&mut Self>, key: K, value: V) -> *mut V {
        if let Some(found) = self.as_mut().find(&key) {
            *found = value;
            return found;
        }
        self.insert_entry(Box::into_raw(Box::new(Entry::new(key, value))))
    }
    // Source: src/core/SkLRUCache.h:94-96
    pub fn count(&self) -> i32 {
        self.fMap.count()
    }
    // Source: src/core/SkLRUCache.h:98-105
    pub fn foreach(self: Pin<&mut Self>, mut f: impl FnMut(&K, &mut V)) {
        unsafe {
            let this = self.get_unchecked_mut();
            let mut iter = Iter::new();
            let mut entry = iter.init(&this.fLRU, IterStart::kHead_IterStart);
            while !entry.is_null() {
                f(&(*entry).fKey, &mut (*entry).fValue);
                entry = iter.next();
            }
        }
    }
    // Source: src/core/SkLRUCache.h:107-113
    pub fn reset(self: Pin<&mut Self>) {
        unsafe {
            let this = self.get_unchecked_mut();
            this.fMap.reset();
            let mut entry = this.fLRU.head();
            while !entry.is_null() {
                this.fLRU.remove(entry);
                drop(Box::from_raw(entry));
                entry = this.fLRU.head();
            }
        }
    }
    // Source: src/core/SkLRUCache.h:115-124
    pub unsafe fn remove(self: Pin<&mut Self>, key: &K) {
        let this = self.get_unchecked_mut();
        let value = this.fMap.find(key);
        crate::SkASSERT!(@source b"src/core/SkLRUCache.h\0".as_ptr(),117; @condition b"value\0".as_ptr(); value.is_some());
        let entry = value.expect("native SkASSERT(value)").as_ptr();
        crate::SkASSERT!(@source b"src/core/SkLRUCache.h\0".as_ptr(),119; @condition b"key == entry->fKey\0".as_ptr(); key==&*(*entry).fKey);
        P::default().purge(this.fContext, key, &(*entry).fValue);
        this.fMap.remove(key);
        this.fLRU.remove(entry);
        drop(Box::from_raw(entry));
    }
}
// Source: src/core/SkLRUCache.h:127-135
struct Traits<H>(PhantomData<H>);
impl<K, V, H: NativeHash<K>> HashTraits<NonNull<Entry<K, V>>, K> for Traits<H> {
    type Key<'a>
        = &'a K
    where
        K: 'a,
        V: 'a;
    fn GetKey<'a>(entry: &'a NonNull<Entry<K, V>>) -> Self::Key<'a>
    where
        K: 'a,
    {
        unsafe { &entry.as_ref().fKey }
    }
    fn Hash(key: &K) -> u32 {
        H::default().hash(key)
    }
}
// Source: src/core/SkLRUCache.h:137-141
pub struct SkLRUCache<K, V, H = super::SkChecksum::SkGoodHash, P = SkNoOpPurge> {
    fMaxCount: i32,
    fMap: THashTable<NonNull<Entry<K, V>>, K, Traits<H>>,
    fLRU: SkTInternalLList<Entry<K, V>>,
    fContext: *mut c_void,
    _types: PhantomData<P>,
    _pin: PhantomPinned,
}
// Native nodes stay pinned and exclusively owned. new's context contract extends
// to the receiving thread. Synchronization remains the cache user's responsibility.
unsafe impl<K: Send, V: Send, H: Send, P: Send> Send for SkLRUCache<K, V, H, P> {}
