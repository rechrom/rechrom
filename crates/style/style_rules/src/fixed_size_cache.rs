#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2023 The Chromium Authors. BSD-style license; see Chromium LICENSE.

// KeyTraits is the C++ HashTraits policy: EmptyValue is reserved and GetHash
// supplies the same hash used by the explicit-hash overloads.
pub trait KeyTraits<Key> {
    fn EmptyValue() -> Key;
    fn GetHash(key: &Key) -> u32;
}

// cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:34-43,103-120
pub struct FixedSizeCache<Key, Value, Traits, const CACHE_SIZE: usize = 512> {
    prefilter: [u8; CACHE_SIZE],
    cache: Vec<(Key, Value)>,
    traits: std::marker::PhantomData<Traits>,
}

impl<Key, Value, Traits, const CACHE_SIZE: usize> FixedSizeCache<Key, Value, Traits, CACHE_SIZE>
where
    Key: PartialEq + Clone,
    Value: Default + Clone,
    Traits: KeyTraits<Key>,
{
    const CHECK_SIZE: () = {
        assert!(CACHE_SIZE >= 2);
        assert!(CACHE_SIZE.is_power_of_two());
        assert!(CACHE_SIZE <= u32::MAX as usize);
    };

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:46-51
    pub fn new() -> Self {
        let () = Self::CHECK_SIZE;
        let mut cache = Vec::with_capacity(CACHE_SIZE);
        for _ in 0..CACHE_SIZE {
            cache.push((Traits::EmptyValue(), Value::default()));
        }
        Self {
            prefilter: [0; CACHE_SIZE],
            cache,
            traits: std::marker::PhantomData,
        }
    }

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:55
    pub fn Find(&mut self, key: &Key) -> Option<&mut Value> {
        self.FindWithHash(key, Traits::GetHash(key))
    }

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:58-78
    pub fn FindWithHash(&mut self, key: &Key, hash: u32) -> Option<&mut Value> {
        debug_assert!(Traits::EmptyValue() != *key);
        debug_assert!(Traits::GetHash(key) == hash);
        let bucket_set = (hash as usize % CACHE_SIZE) & !1;
        let prefilter_hash = Self::GetPrefilterHash(hash);
        if self.prefilter[bucket_set] == prefilter_hash && self.cache[bucket_set].0 == *key {
            return Some(&mut self.cache[bucket_set].1);
        }
        if self.prefilter[bucket_set + 1] == prefilter_hash && self.cache[bucket_set + 1].0 == *key
        {
            self.prefilter.swap(bucket_set, bucket_set + 1);
            self.cache.swap(bucket_set, bucket_set + 1);
            return Some(&mut self.cache[bucket_set].1);
        }
        None
    }

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:80-82
    pub fn Insert(&mut self, key: &Key, value: &Value) -> &mut Value {
        self.InsertWithHash(key, value, Traits::GetHash(key))
    }

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:85-101
    pub fn InsertWithHash(&mut self, key: &Key, value: &Value, hash: u32) -> &mut Value {
        debug_assert!(Traits::EmptyValue() != *key);
        debug_assert!(Traits::GetHash(key) == hash);
        let mut slot = (hash as usize % CACHE_SIZE) & !1;
        debug_assert!(self.cache[slot].0 != *key);
        debug_assert!(self.cache[slot + 1].0 != *key);
        if self.prefilter[slot] != 0 {
            slot += 1;
        }
        self.prefilter[slot] = Self::GetPrefilterHash(hash);
        self.cache[slot] = (key.clone(), value.clone());
        &mut self.cache[slot].1
    }

    // cpp: third_party/blink/renderer/core/css/fixed_size_cache.h:104-107
    fn GetPrefilterHash(hash: u32) -> u8 {
        (((hash as usize / CACHE_SIZE) & 0xff) | 1) as u8
    }
}

impl<Key, Value, Traits, const CACHE_SIZE: usize> Default
    for FixedSizeCache<Key, Value, Traits, CACHE_SIZE>
where
    Key: PartialEq + Clone,
    Value: Default + Clone,
    Traits: KeyTraits<Key>,
{
    fn default() -> Self {
        Self::new()
    }
}
// cpp: fixed_size_cache.h:53 is Oilpan tracing only; Rust owns its Vec.

#[cfg(test)]
mod tests {
    use crate::fixed_size_cache;
    struct Traits;
    impl fixed_size_cache::KeyTraits<u32> for Traits {
        fn EmptyValue() -> u32 {
            u32::MAX
        }
        fn GetHash(k: &u32) -> u32 {
            *k
        }
    }
    #[test]
    fn cache_hit_preserves_recent_value() {
        let mut c = fixed_size_cache::FixedSizeCache::<u32, u32, Traits, 2>::new();
        c.Insert(&2, &20);
        c.Insert(&4, &40);
        assert_eq!(c.Find(&4), Some(&mut 40));
        c.Insert(&6, &60);
        assert!(c.Find(&2).is_none());
        assert_eq!(c.Find(&4), Some(&mut 40));
        assert_eq!(c.Find(&6), Some(&mut 60));
        *c.Find(&4).unwrap() = 41;
        assert_eq!(c.Find(&4), Some(&mut 41));
        assert!(c.FindWithHash(&8, 8).is_none());
    }
}
