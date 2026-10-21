// C++ HashMap/HashSet delegate storage to HashTable. This first Rust slice
// preserves ordinary map/set lookup, replacement, insertion, removal, and
// equality. The Blink hash-table bucket layout, complete key traits, and
// translator APIs are not represented here; GC collections remain separate.
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

use crate::WtfSizeT;

// cpp: foundation/blink_base/wtf/hash_traits.h:260-267
pub struct IntWithZeroKeyHashTraits<T>(PhantomData<T>);

pub trait HashKeyTraits<K> {
    fn is_valid_key(key: &K) -> bool;
}

impl<K> HashKeyTraits<K> for () {
    fn is_valid_key(_key: &K) -> bool {
        true
    }
}

impl<K: IntegralKey, T: IntegralKey> HashKeyTraits<K> for IntWithZeroKeyHashTraits<T> {
    fn is_valid_key(key: &K) -> bool {
        let key = key.to_i128();
        key != T::max_value().to_i128() && key != T::max_minus_one().to_i128()
    }
}

impl<T: IntegralKey> IntWithZeroKeyHashTraits<T> {
    pub fn IsValidKey(value: T) -> bool {
        value != T::max_value() && value != T::max_minus_one()
    }
}

pub trait IntegralKey: Copy + Eq {
    fn max_value() -> Self;
    fn max_minus_one() -> Self;
    fn to_i128(self) -> i128;
}

macro_rules! impl_integral_key {
    ($($type:ty),* $(,)?) => {
        $(impl IntegralKey for $type {
            fn max_value() -> Self { <$type>::MAX }
            fn max_minus_one() -> Self { <$type>::MAX - 1 }
            fn to_i128(self) -> i128 { self as i128 }
        })*
    };
}
impl_integral_key!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

// cpp: foundation/blink_base/wtf/hash_map.h:81-102,121-145,230-232
pub struct HashMap<K, V, Traits = ()> {
    entries: StdHashMap<K, V>,
    _traits: PhantomData<Traits>,
}

impl<K, V, Traits> Default for HashMap<K, V, Traits> {
    fn default() -> Self {
        Self {
            entries: StdHashMap::new(),
            _traits: PhantomData,
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone, Traits> Clone for HashMap<K, V, Traits> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            _traits: PhantomData,
        }
    }
}

// cpp: foundation/blink_base/wtf/hash_map.h:568-585
impl<K: Eq + Hash, V: PartialEq, Traits> PartialEq for HashMap<K, V, Traits> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<K: Eq + Hash, V: Eq, Traits> Eq for HashMap<K, V, Traits> {}

impl<K, V, Traits> Deref for HashMap<K, V, Traits> {
    type Target = StdHashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

#[allow(non_snake_case)]
impl<K: Eq + Hash, V, Traits> HashMap<K, V, Traits> {
    // cpp: foundation/blink_base/wtf/hash_map.h:122,409-421,460-462
    pub fn new() -> Self {
        Self::default()
    }
    pub fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.entries.len()).expect("WTF hash map size exceeds 32 bits")
    }
    pub fn empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn Contains(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }
    // cpp: foundation/blink_base/wtf/hash_map.h:175-176
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.entries.get_mut(key)
    }
    pub fn Capacity(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.entries.capacity()).expect("WTF hash map capacity exceeds 32 bits")
    }
    pub fn ReserveCapacityForSize(&mut self, size: WtfSizeT) {
        if self.entries.capacity() < size as usize {
            self.entries.reserve(size as usize - self.entries.len());
        }
    }

    // The Rust translation of C++ insert(...).stored_value->value uses entry.
    pub fn entry(&mut self, key: K) -> std::collections::hash_map::Entry<'_, K, V>
    where
        Traits: HashKeyTraits<K>,
    {
        assert!(Traits::is_valid_key(&key), "invalid Blink hash-map key");
        self.entries.entry(key)
    }

    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, V> {
        self.entries.values_mut()
    }

    // cpp: foundation/blink_base/wtf/hash_map.h:489-522
    // The return value is the C++ AddResult::is_new_entry bit. The full
    // iterator-bearing AddResult is still pending.
    pub fn Set(&mut self, key: K, value: V) -> bool
    where
        Traits: HashKeyTraits<K>,
    {
        assert!(Traits::is_valid_key(&key), "invalid Blink hash-map key");
        self.entries.insert(key, value).is_none()
    }
    pub fn insert(&mut self, key: K, value: V) -> bool
    where
        Traits: HashKeyTraits<K>,
    {
        assert!(Traits::is_valid_key(&key), "invalid Blink hash-map key");
        match self.entries.entry(key) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                true
            }
            std::collections::hash_map::Entry::Occupied(_) => false,
        }
    }

    // cpp: foundation/blink_base/wtf/hash_map.h:536-549
    pub fn erase(&mut self, key: &K) {
        self.entries.remove(key);
    }

    // cpp: foundation/blink_base/wtf/hash_map.h:552-559
    pub fn Take(&mut self, key: &K) -> V
    where
        V: Default,
    {
        self.entries.remove(key).unwrap_or_default()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// cpp: foundation/blink_base/wtf/hash_set.h:45-79,168-169
pub struct HashSet<T, Traits = ()> {
    entries: StdHashSet<T>,
    _traits: PhantomData<Traits>,
}

impl<T, Traits> Default for HashSet<T, Traits> {
    fn default() -> Self {
        Self {
            entries: StdHashSet::new(),
            _traits: PhantomData,
        }
    }
}

impl<T: Clone + Eq + Hash, Traits> Clone for HashSet<T, Traits> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            _traits: PhantomData,
        }
    }
}

// cpp: foundation/blink_base/wtf/hash_set.h:250-264
impl<T: Eq + Hash, Traits> PartialEq for HashSet<T, Traits> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<T: Eq + Hash, Traits> Eq for HashSet<T, Traits> {}

impl<T, Traits> Deref for HashSet<T, Traits> {
    type Target = StdHashSet<T>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

#[allow(non_snake_case)]
impl<T: Eq + Hash, Traits> HashSet<T, Traits> {
    // cpp: foundation/blink_base/wtf/hash_set.h:75,266-302
    pub fn new() -> Self {
        Self::default()
    }
    pub fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.entries.len()).expect("WTF hash set size exceeds 32 bits")
    }
    pub fn empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn Contains(&self, value: &T) -> bool {
        self.entries.contains(value)
    }
    pub fn Capacity(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.entries.capacity()).expect("WTF hash set capacity exceeds 32 bits")
    }
    pub fn ReserveCapacityForSize(&mut self, size: WtfSizeT) {
        if self.entries.capacity() < size as usize {
            self.entries.reserve(size as usize - self.entries.len());
        }
    }

    // cpp: foundation/blink_base/wtf/hash_set.h:319-324,341-355
    pub fn insert(&mut self, value: T) -> bool
    where
        Traits: HashKeyTraits<T>,
    {
        assert!(Traits::is_valid_key(&value), "invalid Blink hash-set key");
        self.entries.insert(value)
    }
    pub fn erase(&mut self, value: &T) {
        self.entries.remove(value);
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_does_not_replace_but_set_does() {
        let mut map = HashMap::<u32, &str>::new();
        assert!(map.insert(7, "old"));
        assert!(!map.insert(7, "ignored"));
        assert_eq!(map.get(&7), Some(&"old"));
        assert!(!map.Set(7, "new"));
        assert_eq!(map.get(&7), Some(&"new"));
        assert_eq!(map.size(), 1);
        assert!(map == map.clone());
        map.erase(&7);
        assert!(map.empty());
    }

    #[test]
    fn zero_key_traits_and_set_membership() {
        assert!(IntWithZeroKeyHashTraits::<usize>::IsValidKey(0));
        assert!(!IntWithZeroKeyHashTraits::<usize>::IsValidKey(usize::MAX));
        assert!(!IntWithZeroKeyHashTraits::<usize>::IsValidKey(
            usize::MAX - 1
        ));
        let mut zero_key_map = HashMap::<usize, u32, IntWithZeroKeyHashTraits<usize>>::new();
        assert!(zero_key_map.insert(0, 7));
        assert_eq!(zero_key_map.get(&0), Some(&7));
        let mut set = HashSet::<u32>::new();
        assert!(set.insert(3));
        assert!(!set.insert(3));
        assert!(set.Contains(&3));
        set.erase(&3);
        assert!(set.empty());
    }

    #[test]
    #[should_panic(expected = "invalid Blink hash-map key")]
    fn zero_key_traits_reject_reserved_value() {
        let mut map = HashMap::<usize, u32, IntWithZeroKeyHashTraits<usize>>::new();
        map.insert(usize::MAX, 1);
    }
}
