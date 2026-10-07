// C++ BasicHeapHashMap/Set select on-stack or GC-allocated ownership while
// tracing live keys and values. Rust owns backing storage inside the wrapper;
// the Visitor walks that storage when the wrapper is traced.
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::{AtomicString, HashMap, Member, Traceable, Visitor, WeakMember, WtfSizeT};

// A weak map key needs ephemeron tracing and removal of cleared buckets.
// Until that backing is connected, only strong keys may be traced as maps.
pub trait StrongHeapMapKey: Traceable {}
impl StrongHeapMapKey for AtomicString {}
impl<T: ?Sized> StrongHeapMapKey for Member<T> {}
impl StrongHeapMapKey for std::string::String {}
impl StrongHeapMapKey for crate::BlinkString {}
macro_rules! impl_strong_heap_map_key {
    ($($type:ty),* $(,)?) => { $(impl StrongHeapMapKey for $type {})* };
}
impl_strong_heap_map_key!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

// cpp: foundation/blink_base/heap/collection_support/heap_hash_map.h:16-37,77-102
pub struct BasicHeapHashMap<K, V, const GCED: bool, KeyTraits = (), MappedTraits = ()> {
    entries: HashMap<K, V, KeyTraits>,
    _mapped_traits: PhantomData<MappedTraits>,
}

pub type HeapHashMap<K, V, KeyTraits = (), MappedTraits = ()> =
    BasicHeapHashMap<K, V, false, KeyTraits, MappedTraits>;
pub type GCedHeapHashMap<K, V, KeyTraits = (), MappedTraits = ()> =
    BasicHeapHashMap<K, V, true, KeyTraits, MappedTraits>;

// C++ HeapHashMap<WeakMember<K>, V> uses ephemeron tracing. A cleared weak
// pointer cannot remain in a Rust HashMap bucket because its hash changes, so
// this specialization keeps entries in a compact sequence and skips/evicts
// cleared keys during lookups and mutations.
// cpp: foundation/blink_base/heap/collection_support/heap_hash_map.h:16-40
// cpp: foundation/blink_base/heap/trace_traits.h:185-205
pub struct WeakHeapHashMap<K: ?Sized, V> {
    entries: Vec<(WeakMember<K>, V)>,
}

impl<K: ?Sized, V> Default for WeakHeapHashMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K: ?Sized, V: Clone> Clone for WeakHeapHashMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: self
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        }
    }
}

impl<K: ?Sized, V: PartialEq> PartialEq for WeakHeapHashMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().count() == other.iter().count()
            && self.iter().all(|(key, value)| {
                other
                    .get(key)
                    .is_some_and(|other_value| value == other_value)
            })
    }
}

impl<K: ?Sized, V: Eq> Eq for WeakHeapHashMap<K, V> {}

#[allow(non_snake_case)]
impl<K: ?Sized, V> WeakHeapHashMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.iter().count()).expect("WTF weak heap map size exceeds 32 bits")
    }

    pub fn empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&WeakMember<K>, &V)> {
        self.entries
            .iter()
            .filter(|(key, _)| key.GetNonNull().is_some())
            .map(|(key, value)| (key, value))
    }

    pub fn get(&self, key: &WeakMember<K>) -> Option<&V> {
        if key.GetNonNull().is_none() {
            return None;
        }
        self.iter()
            .find_map(|(candidate, value)| (candidate == key).then_some(value))
    }

    pub fn get_mut(&mut self, key: &WeakMember<K>) -> Option<&mut V> {
        if key.GetNonNull().is_none() {
            return None;
        }
        self.entries.iter_mut().find_map(|(candidate, value)| {
            (candidate.GetNonNull().is_some() && candidate == key).then_some(value)
        })
    }

    // cpp: foundation/blink_base/wtf/hash_map.h:489-522
    pub fn Set(&mut self, key: WeakMember<K>, value: V) -> bool {
        assert!(key.GetNonNull().is_some(), "null weak hash-map key");
        if let Some(stored) = self.get_mut(&key) {
            *stored = value;
            return false;
        }
        self.entries
            .retain(|(candidate, _)| candidate.GetNonNull().is_some());
        self.entries.push((key, value));
        true
    }

    // cpp: foundation/blink_base/wtf/hash_map.h:489-522
    pub fn insert(&mut self, key: WeakMember<K>, value: V) -> bool {
        assert!(key.GetNonNull().is_some(), "null weak hash-map key");
        if self.get(&key).is_some() {
            return false;
        }
        self.entries
            .retain(|(candidate, _)| candidate.GetNonNull().is_some());
        self.entries.push((key, value));
        true
    }

    pub fn remove(&mut self, key: &WeakMember<K>) -> Option<V> {
        if key.GetNonNull().is_none() {
            return None;
        }
        let index = self
            .entries
            .iter()
            .position(|(candidate, _)| candidate.GetNonNull().is_some() && candidate == key)?;
        Some(self.entries.remove(index).1)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<K: ?Sized, V: Traceable> Traceable for WeakHeapHashMap<K, V> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for (key, value) in self.iter() {
            visitor.Trace(key);
            visitor.TraceEphemeron(key, value);
        }
    }
}

impl<K, V, const GCED: bool, KeyTraits, MappedTraits> Default
    for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    fn default() -> Self {
        Self {
            entries: HashMap::default(),
            _mapped_traits: PhantomData,
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone, const GCED: bool, KeyTraits, MappedTraits> Clone
    for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            _mapped_traits: PhantomData,
        }
    }
}

impl<K, V, const GCED: bool, KeyTraits, MappedTraits> Deref
    for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    type Target = HashMap<K, V, KeyTraits>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl<K, V, const GCED: bool, KeyTraits, MappedTraits> DerefMut
    for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

// cpp: foundation/blink_base/heap/collection_support/heap_hash_map.h:38-40
impl<K: Eq + Hash + StrongHeapMapKey, V: Traceable, const GCED: bool, KeyTraits, MappedTraits>
    Traceable for BasicHeapHashMap<K, V, GCED, KeyTraits, MappedTraits>
{
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for (key, value) in self.entries.iter() {
            visitor.Trace(key);
            visitor.Trace(value);
        }
    }
}

// Heap hash sets contain only Member/WeakMember keys in the supplied source.
// Weak keys are filtered after GC clears their slots; vector storage keeps a
// cleared weak pointer from invalidating a hash-table bucket.
pub trait HeapSetEntry: Traceable + PartialEq {
    fn is_live(&self) -> bool;
}

impl<T: ?Sized> HeapSetEntry for Member<T> {
    fn is_live(&self) -> bool {
        self.GetNonNull().is_some()
    }
}

impl<T: ?Sized> HeapSetEntry for WeakMember<T> {
    fn is_live(&self) -> bool {
        self.GetNonNull().is_some()
    }
}

// cpp: foundation/blink_base/heap/collection_support/heap_hash_set.h:15-35,74-89
pub struct BasicHeapHashSet<T: HeapSetEntry, const GCED: bool> {
    entries: Vec<T>,
}

pub type HeapHashSet<T> = BasicHeapHashSet<T, false>;
pub type GCedHeapHashSet<T> = BasicHeapHashSet<T, true>;

impl<T: HeapSetEntry, const GCED: bool> Default for BasicHeapHashSet<T, GCED> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T: HeapSetEntry + Clone, const GCED: bool> Clone for BasicHeapHashSet<T, GCED> {
    fn clone(&self) -> Self {
        Self {
            entries: self.iter().cloned().collect(),
        }
    }
}

#[allow(non_snake_case)]
impl<T: HeapSetEntry, const GCED: bool> BasicHeapHashSet<T, GCED> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.iter().count()).expect("WTF heap hash set size exceeds 32 bits")
    }

    pub fn empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn Contains(&self, value: &T) -> bool {
        self.iter().any(|candidate| candidate == value)
    }

    // cpp: foundation/blink_base/wtf/hash_set.h:319-324,341-355
    pub fn insert(&mut self, value: T) -> bool {
        assert!(value.is_live(), "empty and deleted GC keys are invalid");
        self.entries.retain(HeapSetEntry::is_live);
        if self.Contains(&value) {
            return false;
        }
        assert!(self.entries.len() < WtfSizeT::MAX as usize);
        self.entries.push(value);
        true
    }

    pub fn erase(&mut self, value: &T) {
        self.entries
            .retain(|candidate| candidate.is_live() && candidate != value);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().filter(|entry| entry.is_live())
    }
}

// cpp: foundation/blink_base/heap/collection_support/heap_hash_set.h:59-61
impl<T: HeapSetEntry, const GCED: bool> Traceable for BasicHeapHashSet<T, GCED> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        for entry in &self.entries {
            visitor.Trace(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomicString, LayoutHeapScope, MakeGarbageCollected, Persistent, WeakPersistent};

    struct Target;

    impl Traceable for Target {
        fn Trace(&self, _visitor: &mut Visitor<'_>) {}
    }

    #[test]
    fn weak_set_key_disappears_after_collection() {
        let root;
        {
            let _scope = LayoutHeapScope::new();
            let target = MakeGarbageCollected(Target);
            let set = MakeGarbageCollected(GCedHeapHashSet::<WeakMember<Target>>::default());
            assert!(unsafe { &mut *set }.insert(WeakMember::from_ptr(target)));
            assert_eq!(unsafe { &*set }.size(), 1);
            root = Persistent::from_ptr(set);
        }
        let set = unsafe { &*root.Get() };
        assert_eq!(set.size(), 0);
        assert!(set.empty());
    }

    #[test]
    fn heap_map_traces_member_values() {
        let root;
        let weak_target;
        {
            let _scope = LayoutHeapScope::new();
            let target = MakeGarbageCollected(Target);
            let map =
                MakeGarbageCollected(GCedHeapHashMap::<AtomicString, Member<Target>>::default());
            assert!(
                unsafe { &mut *map }.Set(AtomicString::from_str("box"), Member::from_ptr(target))
            );
            root = Persistent::from_ptr(map);
            weak_target = WeakPersistent::from_ptr(target);
        }
        assert!(!weak_target.Get().is_null());
        let map = unsafe { &*root.Get() };
        assert_eq!(map.size(), 1);
        assert_eq!(
            map.get(&AtomicString::from_str("box")).unwrap().Get(),
            weak_target.Get()
        );
    }

    #[test]
    fn weak_map_value_survives_only_while_key_is_live() {
        let map_root;
        let key_root;
        let value_weak;
        {
            let _scope = LayoutHeapScope::new();
            let key = MakeGarbageCollected(Target);
            let value = MakeGarbageCollected(Target);
            let map = MakeGarbageCollected(WeakHeapHashMap::<Target, Member<Target>>::default());
            assert!(unsafe { &mut *map }.Set(WeakMember::from_ptr(key), Member::from_ptr(value),));
            map_root = Persistent::from_ptr(map);
            key_root = Persistent::from_ptr(key);
            value_weak = WeakPersistent::from_ptr(value);
        }
        assert!(!value_weak.Get().is_null());
        assert_eq!(unsafe { &*map_root.Get() }.size(), 1);

        drop(key_root);
        {
            let _scope = LayoutHeapScope::new();
        }
        assert!(value_weak.Get().is_null());
        assert!(unsafe { &*map_root.Get() }.empty());
    }

    #[test]
    fn weak_map_does_not_keep_key_alive_through_its_value() {
        struct Dependent {
            key: Member<Target>,
        }
        impl Traceable for Dependent {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                visitor.Trace(&self.key);
            }
        }

        let map_root;
        let weak_key;
        let weak_value;
        {
            let _scope = LayoutHeapScope::new();
            let key = MakeGarbageCollected(Target);
            let value = MakeGarbageCollected(Dependent {
                key: Member::from_ptr(key),
            });
            let map = MakeGarbageCollected(WeakHeapHashMap::<Target, Member<Dependent>>::default());
            unsafe { &mut *map }.insert(WeakMember::from_ptr(key), Member::from_ptr(value));
            map_root = Persistent::from_ptr(map);
            weak_key = WeakPersistent::from_ptr(key);
            weak_value = WeakPersistent::from_ptr(value);
        }
        assert!(weak_key.Get().is_null());
        assert!(weak_value.Get().is_null());
        assert!(unsafe { &*map_root.Get() }.empty());
    }
}
