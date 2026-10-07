#![allow(non_snake_case)]

use crate::{heap_hash_containers::BasicHeapHashMap, heap_vector::BasicHeapVector};
use std::hash::Hash;

pub trait ClearCollection {
    fn ClearCollection(&mut self);
}

impl<K: Eq + Hash, V, const GCED: bool, KeyTraits, MappedTraits> ClearCollection
    for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    fn ClearCollection(&mut self) {
        *self = Self::default();
    }
}

impl<T, const N: usize, const GCED: bool> ClearCollection for BasicHeapVector<T, N, GCED> {
    fn ClearCollection(&mut self) {
        *self = Self::default();
    }
}

// The pointer permits the collection's owner to keep using it until scope
// exit, as in the C++ stack guard. It must outlive this guard and be unique.
// cpp: foundation/blink_base/heap/collection_support/clear_collection_scope.h:17-26
pub struct ClearCollectionScope<T: ClearCollection> {
    ptr_: *mut T,
}

impl<T: ClearCollection> ClearCollectionScope<T> {
    /// # Safety
    /// `ptr` must remain valid and uniquely writable until this guard drops.
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr_: ptr }
    }
}

impl<T: ClearCollection> Drop for ClearCollectionScope<T> {
    fn drop(&mut self) {
        unsafe { &mut *self.ptr_ }.ClearCollection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HeapVector;

    #[test]
    fn clears_a_stack_collection_when_scope_ends() {
        let mut values = HeapVector::<u32>::new();
        values.reserve(64);
        {
            let _guard = unsafe { ClearCollectionScope::new(&mut values as *mut _) };
            values.push_back(7);
            assert_eq!(values.size(), 1);
        }
        assert!(values.empty());
        assert_eq!(values.capacity(), 0);
    }
}
