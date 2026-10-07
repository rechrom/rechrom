// Copyright 2010 Google Inc.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Restricted translation of SkStrikeCache.cpp::internalPurge and the default
//! limits in SkStrikeCache.h. Images and paths are independently prepared once
//! per resident glyph; both share their strike's lifetime and budget. P defaults
//! to I for compatibility, or can independently store a caller's path payload.
//! The descriptor/glyph key types and image generator are Rust adapters. Callers
//! must include every raster-affecting font, matrix, color/gamma and smoothing
//! attribute in the descriptor, and exact subpixel coordinates in the glyph key.
//! Monotonic access stamps replace the upstream linked LRU list: hits are O(1),
//! and purge selects the same oldest-strike order by sorting current records.
//! No pinners, global singleton, font metrics, paths or GPU strikes are supplied.
//!
//! The budget counts live records, image objects, Arc control words and caller-
//! reported image allocation bytes. HashMap spare capacity/allocator overhead
//! and heap allocations within keys are not measured. Returned Arc handles may
//! keep evicted images alive, so this is not a hard process-memory upper bound.

use super::SkStrike::SkStrike;
use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;
use std::mem::size_of;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

pub const SK_DEFAULT_FONT_CACHE_LIMIT: usize = 2 * 1024 * 1024;
pub const SK_DEFAULT_FONT_CACHE_COUNT_LIMIT: usize = 2048;

pub struct SkStrikeCache<D, G, I, P = I> {
    strikes: HashMap<D, SkStrike<G, I, P>>,
    bytes_limit: usize,
    count_limit: usize,
    memory_used: usize,
    clock: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl<D: Clone + Eq + Hash, G: Clone + Eq + Hash, I, P> Default for SkStrikeCache<D, G, I, P> {
    fn default() -> Self {
        Self::new(
            SK_DEFAULT_FONT_CACHE_LIMIT,
            SK_DEFAULT_FONT_CACHE_COUNT_LIMIT,
        )
    }
}

impl<D: Clone + Eq + Hash, G: Clone + Eq + Hash, I, P> SkStrikeCache<D, G, I, P> {
    pub fn new(bytes_limit: usize, count_limit: usize) -> Self {
        Self {
            strikes: HashMap::new(),
            bytes_limit,
            count_limit,
            memory_used: 0,
            clock: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Looks up prepared data without generating or admitting anything.
    /// None means missing; Some(None) means a prepared empty glyph. A resident
    /// strike is touched even if its glyph is missing, like upstream findStrike.
    /// Successful record lookups increment hits; misses count only generators
    /// run by prepare_image, avoiding double-counting a probe then preparation.
    /// This allows a caller to reject offscreen glyphs after a miss without
    /// incorrectly installing an empty-glyph negative record.
    pub fn find_image(&mut self, descriptor: &D, glyph: &G) -> Option<Option<Arc<I>>> {
        let stamp = self.next_stamp();
        let strike = self.strikes.get_mut(descriptor)?;
        strike.last_used = stamp;
        let image = strike.find_image(glyph)?;
        self.hits = self.hits.saturating_add(1);
        Some(image)
    }

    /// Runs generate only when the exact resident key has not been prepared.
    /// image_size reports separately allocated image payload (e.g. A8 Vec
    /// capacity); size_of::<I>() and Arc control words are already accounted.
    /// Both successful images and None results are reused until strike eviction.
    /// Over-budget images are returned normally but can immediately be evicted.
    pub fn prepare_image(
        &mut self,
        descriptor: D,
        glyph: G,
        generate: impl FnOnce() -> Option<I>,
        image_size: impl Fn(&I) -> usize,
    ) -> Option<Arc<I>> {
        self.with_strike(descriptor, |strike| {
            strike.prepare_image(glyph, generate, image_size)
        })
    }

    /// Three-state path lookup, independent of image records. Miss probes do
    /// not count as misses; generators executed by prepare_path do.
    pub fn find_path(&mut self, descriptor: &D, glyph: &G) -> Option<Option<Arc<P>>> {
        let stamp = self.next_stamp();
        let strike = self.strikes.get_mut(descriptor)?;
        strike.last_used = stamp;
        let path = strike.find_path(glyph)?;
        self.hits = self.hits.saturating_add(1);
        Some(path)
    }

    /// Prepares a path lazily, with the same accounting and purge as images.
    /// Prefer with_strike for runs so descriptor hashing is performed once.
    pub fn prepare_path(
        &mut self,
        descriptor: D,
        glyph: G,
        generate: impl FnOnce() -> Option<P>,
        path_size: impl Fn(&P) -> usize,
    ) -> Option<Arc<P>> {
        self.with_strike(descriptor, |strike| {
            strike.prepare_path(glyph, generate, path_size)
        })
    }

    /// Borrows one strike for an entire run, corresponding to upstream callers
    /// holding a strike across prepareImages. The descriptor is hashed and its
    /// LRU position touched only at run entry; glyph operations reuse this borrow.
    /// Growth and counters are merged once, and budgets purged after the run.
    /// A run can temporarily exceed its budget. If the closure unwinds, growth
    /// is still accounted and purged before resuming that same panic.
    pub fn with_strike<R>(
        &mut self,
        descriptor: D,
        f: impl FnOnce(&mut SkStrike<G, I, P>) -> R,
    ) -> R {
        let stamp = self.next_stamp();
        let base_bytes = size_of::<D>() + size_of::<SkStrike<G, I, P>>();
        // Only relevant near usize::MAX: make room for possible new metadata
        // without ever allowing the tracked total to wrap.
        if self.memory_used.checked_add(base_bytes).is_none() {
            self.purge_all();
        }
        let (mut entry, new_strike) = match self.strikes.entry(descriptor) {
            Entry::Occupied(entry) => (entry, false),
            Entry::Vacant(entry) => {
                self.memory_used += base_bytes;
                (entry.insert_entry(SkStrike::new(base_bytes, stamp)), true)
            }
        };
        let strike = entry.get_mut();
        strike.last_used = stamp;
        let before_bytes = strike.memory_used;
        let before_hits = strike.hits();
        let before_misses = strike.misses();
        let outcome = catch_unwind(AssertUnwindSafe(|| f(strike)));
        let growth = strike.memory_used - before_bytes;
        self.hits = self
            .hits
            .saturating_add(strike.hits().saturating_sub(before_hits));
        self.misses = self
            .misses
            .saturating_add(strike.misses().saturating_sub(before_misses));
        // Don't retain a newly-created empty strike after an offscreen miss,
        // unrepresentable image-size report, or panic before any admission.
        if new_strike && strike.is_empty() {
            entry.remove();
            self.memory_used -= before_bytes;
        } else if let Some(total) = self.memory_used.checked_add(growth) {
            self.memory_used = total;
        } else {
            // The image handles already returned by the closure remain valid;
            // remove the overflowing strike rather than undercount its storage.
            entry.remove();
            self.memory_used -= before_bytes;
            self.evictions = self.evictions.saturating_add(1);
        }
        self.internal_purge();
        match outcome {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }

    /// Changes limits and immediately applies upstream's coarse LRU purge.
    /// Returns accounted bytes freed; outstanding image handles remain valid.
    pub fn set_limits(&mut self, bytes_limit: usize, count_limit: usize) -> usize {
        self.bytes_limit = bytes_limit;
        self.count_limit = count_limit;
        self.internal_purge()
    }

    /// Removes every resident strike without changing budgets or hit counters.
    pub fn purge_all(&mut self) {
        self.evictions = self.evictions.saturating_add(self.strikes.len() as u64);
        self.strikes.clear();
        self.memory_used = 0;
    }

    pub fn memory_used(&self) -> usize {
        self.memory_used
    }
    pub fn count(&self) -> usize {
        self.strikes.len()
    }
    pub fn bytes_limit(&self) -> usize {
        self.bytes_limit
    }
    pub fn count_limit(&self) -> usize {
        self.count_limit
    }
    pub fn hits(&self) -> u64 {
        self.hits
    }
    pub fn misses(&self) -> u64 {
        self.misses
    }
    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    fn next_stamp(&mut self) -> u64 {
        // Preserve strict LRU order even after an artificial u64 clock wrap.
        if self.clock == u64::MAX {
            let mut order: Vec<_> = self.strikes.values_mut().collect();
            order.sort_unstable_by_key(|strike| strike.last_used);
            for (index, strike) in order.iter_mut().enumerate() {
                strike.last_used = index as u64;
            }
            self.clock = order.len() as u64;
        }
        self.clock += 1;
        self.clock
    }

    fn internal_purge(&mut self) -> usize {
        // Corresponds to upstream internalPurge(minBytesNeeded=0), without
        // pinned strikes: both targets must be satisfied by whole-strike LRU.
        let mut bytes_needed = self.memory_used.saturating_sub(self.bytes_limit);
        if bytes_needed != 0 {
            bytes_needed = bytes_needed.max(self.memory_used >> 2);
        }
        let mut count_needed = self.strikes.len().saturating_sub(self.count_limit);
        if count_needed != 0 {
            count_needed = count_needed.max(self.strikes.len() >> 2);
        }
        if bytes_needed == 0 && count_needed == 0 {
            return 0;
        }
        let mut order: Vec<_> = self
            .strikes
            .iter()
            .map(|(descriptor, strike)| (strike.last_used, descriptor.clone()))
            .collect();
        order.sort_unstable_by_key(|entry| entry.0);
        let mut bytes_freed = 0;
        let mut count_freed = 0;
        for (_, descriptor) in order {
            if bytes_freed >= bytes_needed && count_freed >= count_needed {
                break;
            }
            let strike = self.strikes.remove(&descriptor).unwrap();
            bytes_freed += strike.memory_used;
            count_freed += 1;
        }
        self.memory_used -= bytes_freed;
        self.evictions = self.evictions.saturating_add(count_freed as u64);
        bytes_freed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    type Cache = SkStrikeCache<u8, u8, Vec<u8>>;
    fn image(cache: &mut Cache, descriptor: u8, glyph: u8, bytes: usize) -> Arc<Vec<u8>> {
        cache
            .prepare_image(
                descriptor,
                glyph,
                || Some(vec![glyph; bytes]),
                Vec::capacity,
            )
            .unwrap()
    }

    #[test]
    fn paths_and_images_have_independent_lazy_records_and_payload_types() {
        let mut cache = SkStrikeCache::<u8, u8, Vec<u8>, Vec<u16>>::default();
        cache.with_strike(1, |strike| {
            assert!(strike.find_path(&1).is_none());
            let path = strike
                .prepare_path(
                    1,
                    || Some(vec![13u16; 4]),
                    |p| p.capacity() * size_of::<u16>(),
                )
                .unwrap();
            let again = strike
                .prepare_path(
                    1,
                    || panic!("path regenerated"),
                    |_| panic!("path size recomputed"),
                )
                .unwrap();
            assert!(Arc::ptr_eq(&path, &again));
            assert!(strike.find_image(&1).is_none());
            assert!(strike.prepare_image(1, || None, Vec::capacity).is_none());
            assert!(matches!(strike.find_image(&1), Some(None)));
            assert_eq!(strike.find_path(&1).unwrap().unwrap().as_slice(), &[13; 4]);
            assert!(strike.prepare_path(2, || None, |_| 0).is_none());
            assert!(matches!(strike.find_path(&2), Some(None)));
            assert!(strike.find_image(&2).is_none());
        });
        assert_eq!((cache.hits(), cache.misses()), (4, 3));
        assert_eq!(
            cache.memory_used(),
            cache.strikes.values().map(|s| s.memory_used).sum()
        );
        assert_eq!(
            cache.find_path(&1, &1).unwrap().unwrap().as_slice(),
            &[13; 4]
        );
        assert!(matches!(cache.find_path(&1, &2), Some(None)));
    }

    #[test]
    fn path_only_strikes_survive_runs_and_purge_as_a_unit_with_images() {
        let mut cache = Cache::new(usize::MAX, 1);
        let path = cache
            .prepare_path(1, 1, || Some(vec![7; 31]), Vec::capacity)
            .unwrap();
        assert_eq!(cache.count(), 1);
        cache.prepare_image(1, 1, || Some(vec![8; 19]), Vec::capacity);
        let before = cache.memory_used();
        assert_eq!(before, cache.strikes.values().map(|s| s.memory_used).sum());
        cache.prepare_path(2, 1, || Some(vec![9; 11]), Vec::capacity);
        assert_eq!(cache.count(), 1);
        assert!(cache.find_path(&1, &1).is_none());
        assert!(cache.find_image(&1, &1).is_none());
        assert_eq!(path.as_slice(), &[7; 31]);
        cache.set_limits(1, 1);
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
    }

    #[test]
    fn path_size_overflow_and_panics_preserve_accounting() {
        let mut cache = Cache::default();
        assert!(cache
            .prepare_path(1, 1, || Some(vec![1]), |_| usize::MAX)
            .is_some());
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
        let result = catch_unwind(AssertUnwindSafe(|| {
            cache.with_strike(2, |strike| {
                strike.prepare_path(1, || Some(vec![1; 29]), Vec::capacity);
                strike.prepare_path(2, || Some(vec![2]), |_| panic!("path accounting callback"));
            });
        }));
        assert!(result.is_err());
        assert_eq!(cache.count(), 1);
        assert_eq!(
            cache.memory_used(),
            cache.strikes.values().map(|s| s.memory_used).sum()
        );
        assert!(cache.find_path(&2, &2).is_none());
        assert_eq!(cache.find_path(&2, &1).unwrap().unwrap().len(), 29);
        cache.set_limits(0, 0);
        let result = catch_unwind(AssertUnwindSafe(|| {
            cache.with_strike(3, |strike| {
                strike.prepare_path(1, || Some(vec![3; 29]), Vec::capacity);
                panic!("path run overbudget");
            });
        }));
        assert!(result.is_err());
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
    }

    #[test]
    fn run_borrow_prepares_multiple_glyphs_and_merges_lazy_counters() {
        let mut cache = Cache::default();
        cache.with_strike(1, |strike| {
            for glyph in 0..6 {
                let first = strike
                    .prepare_image(glyph, || Some(vec![glyph; 3]), Vec::capacity)
                    .unwrap();
                let second = strike
                    .prepare_image(glyph, || panic!("run hit regenerated"), Vec::capacity)
                    .unwrap();
                assert!(Arc::ptr_eq(&first, &second));
            }
            assert!(strike.find_image(&9).is_none());
            assert!(strike.prepare_image(9, || None, Vec::capacity).is_none());
            assert!(matches!(strike.find_image(&9), Some(None)));
        });
        assert_eq!((cache.count(), cache.hits(), cache.misses()), (1, 7, 7));
        cache.with_strike(1, |strike| {
            assert_eq!(strike.find_image(&2).unwrap().unwrap().as_slice(), &[2; 3]);
        });
        assert_eq!((cache.hits(), cache.misses()), (8, 7));
    }

    #[test]
    fn resident_run_hashes_descriptor_once_for_many_glyphs() {
        use std::rc::Rc;
        #[derive(Clone)]
        struct Descriptor(Rc<Cell<usize>>);
        impl PartialEq for Descriptor {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }
        impl Eq for Descriptor {}
        impl Hash for Descriptor {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.set(self.0.get() + 1);
                0u8.hash(state);
            }
        }
        let hashes = Rc::new(Cell::new(0));
        let descriptor = Descriptor(hashes.clone());
        let mut cache = SkStrikeCache::<Descriptor, u8, Vec<u8>>::default();
        cache.with_strike(descriptor.clone(), |strike| {
            for glyph in 0..4 {
                strike.prepare_image(glyph, || Some(vec![glyph; 2]), Vec::capacity);
            }
        });
        hashes.set(0);
        cache.with_strike(descriptor, |strike| {
            for glyph in 0..64 {
                assert!(strike.find_image(&(glyph % 4)).unwrap().is_some());
            }
        });
        assert_eq!(hashes.get(), 1);
        assert_eq!((cache.hits(), cache.misses()), (64, 4));
    }

    #[test]
    fn run_pins_whole_strike_until_return_then_purges() {
        let mut cache = Cache::new(1, 0);
        let image = cache.with_strike(1, |strike| {
            let first = strike
                .prepare_image(1, || Some(vec![3; 32]), Vec::capacity)
                .unwrap();
            strike.prepare_image(2, || Some(vec![4; 32]), Vec::capacity);
            assert!(Arc::ptr_eq(
                &first,
                &strike.find_image(&1).unwrap().unwrap()
            ));
            first
        });
        assert_eq!(image.as_slice(), &[3; 32]);
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
        assert_eq!((cache.hits(), cache.misses(), cache.evictions()), (1, 2, 1));
    }

    #[test]
    fn run_panic_accounts_insertions_and_applies_purge_before_unwinding() {
        let mut cache = Cache::default();
        let result = catch_unwind(AssertUnwindSafe(|| {
            cache.with_strike(1, |strike| {
                strike.prepare_image(1, || Some(vec![1; 17]), Vec::capacity);
                panic!("after admission");
            });
        }));
        assert!(result.is_err());
        assert_eq!(
            cache.memory_used(),
            cache.strikes.values().map(|s| s.memory_used).sum()
        );
        assert_eq!((cache.count(), cache.misses()), (1, 1));
        assert_eq!(cache.find_image(&1, &1).unwrap().unwrap().len(), 17);
        cache.set_limits(0, 0);
        let result = catch_unwind(AssertUnwindSafe(|| {
            cache.with_strike(2, |strike| {
                strike.prepare_image(1, || Some(vec![2; 17]), Vec::capacity);
                panic!("overbudget admission");
            });
        }));
        assert!(result.is_err());
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
        let result = catch_unwind(AssertUnwindSafe(|| {
            cache.with_strike(3, |_| panic!("no admission"));
        }));
        assert!(result.is_err());
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
    }

    #[test]
    fn image_is_prepared_once_and_shared_until_evicted() {
        let mut cache = Cache::default();
        let first = image(&mut cache, 1, 3, 17);
        let bytes = cache.memory_used();
        let second = cache
            .prepare_image(
                1,
                3,
                || panic!("resident image regenerated"),
                |_| panic!("resident size recomputed"),
            )
            .unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.memory_used(), bytes);
        assert_eq!((cache.hits(), cache.misses()), (1, 1));
        cache.purge_all();
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
        assert_eq!(first.as_slice(), &[3; 17]);
        let third = image(&mut cache, 1, 3, 17);
        assert!(!Arc::ptr_eq(&first, &third));
    }

    #[test]
    fn lookup_distinguishes_missing_empty_and_nonempty_without_admitting() {
        let mut cache = Cache::default();
        assert!(cache.find_image(&1, &1).is_none());
        assert_eq!((cache.count(), cache.hits(), cache.misses()), (0, 0, 0));
        image(&mut cache, 1, 1, 3);
        assert!(cache.find_image(&1, &2).is_none());
        assert_eq!((cache.count(), cache.hits(), cache.misses()), (1, 0, 1));
        assert!(cache.prepare_image(1, 2, || None, Vec::capacity).is_none());
        assert!(matches!(cache.find_image(&1, &2), Some(None)));
        assert_eq!(cache.find_image(&1, &1).unwrap().unwrap().len(), 3);
        assert_eq!((cache.hits(), cache.misses()), (2, 2));
    }

    #[test]
    fn empty_image_is_negative_cached_and_charged_for_its_record() {
        let mut cache = Cache::default();
        let calls = Cell::new(0);
        for _ in 0..3 {
            assert!(cache
                .prepare_image(
                    4,
                    0,
                    || {
                        calls.set(calls.get() + 1);
                        None
                    },
                    Vec::capacity
                )
                .is_none());
        }
        assert_eq!(calls.get(), 1);
        assert!(cache.memory_used() >= size_of::<u8>() + size_of::<Option<Arc<Vec<u8>>>>());
        assert_eq!((cache.hits(), cache.misses()), (2, 1));
    }

    #[test]
    fn descriptor_and_glyph_keys_separate_images() {
        let mut cache = Cache::default();
        let a = image(&mut cache, 1, 1, 3);
        let b = image(&mut cache, 2, 1, 4);
        let c = image(&mut cache, 1, 2, 5);
        assert_eq!((a.len(), b.len(), c.len()), (3, 4, 5));
        assert_eq!((cache.count(), cache.hits(), cache.misses()), (2, 0, 3));
    }

    #[test]
    fn count_purge_removes_whole_oldest_strike_and_touch_protects_it() {
        let mut cache = Cache::new(usize::MAX, 2);
        image(&mut cache, 1, 1, 3);
        image(&mut cache, 1, 2, 3);
        image(&mut cache, 2, 1, 3);
        image(&mut cache, 1, 1, 3);
        image(&mut cache, 3, 1, 3);
        assert!(cache.strikes.contains_key(&1));
        assert!(!cache.strikes.contains_key(&2));
        assert!(cache.strikes.contains_key(&3));
        image(&mut cache, 3, 1, 3);
        image(&mut cache, 2, 1, 3);
        assert!(!cache.strikes.contains_key(&1));
        assert_eq!(cache.evictions(), 2);
    }

    #[test]
    fn byte_purge_frees_at_least_a_quarter_and_accounts_whole_strikes() {
        let mut cache = Cache::new(usize::MAX, usize::MAX);
        for descriptor in 0..8 {
            image(&mut cache, descriptor, 1, 100);
        }
        let before = cache.memory_used();
        let per_strike = before / 8;
        let freed = cache.set_limits(before - 1, usize::MAX);
        assert_eq!(freed, per_strike * 2);
        assert_eq!(cache.count(), 6);
        assert_eq!(cache.memory_used(), before - freed);
        assert!(!cache.strikes.contains_key(&0));
        assert!(!cache.strikes.contains_key(&1));
    }

    #[test]
    fn count_purge_frees_at_least_a_quarter_and_budget_changes_apply() {
        let mut cache = Cache::new(usize::MAX, usize::MAX);
        for descriptor in 0..8 {
            image(&mut cache, descriptor, 1, 0);
        }
        let before = cache.memory_used();
        cache.set_limits(usize::MAX, 7);
        assert_eq!(cache.count(), 6);
        assert_eq!(cache.memory_used(), before * 6 / 8);
        cache.set_limits(usize::MAX, 1);
        assert_eq!(cache.count(), 1);
        assert!(cache.strikes.contains_key(&7));
        cache.set_limits(0, 0);
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
        assert_eq!((cache.bytes_limit(), cache.count_limit()), (0, 0));
    }

    #[test]
    fn zero_budgets_oversized_images_and_size_overflow_do_not_retain_images() {
        for (bytes, count) in [(0, 4), (4096, 0), (1, 4)] {
            let mut cache = Cache::new(bytes, count);
            assert_eq!(image(&mut cache, 1, 1, 200).len(), 200);
            assert_eq!((cache.count(), cache.memory_used()), (0, 0));
            image(&mut cache, 1, 1, 200);
            assert_eq!(cache.misses(), 2);
        }
        let mut cache = Cache::default();
        let result = cache
            .prepare_image(1, 1, || Some(vec![1]), |_| usize::MAX)
            .unwrap();
        assert_eq!(result.as_slice(), &[1]);
        assert_eq!((cache.count(), cache.memory_used()), (0, 0));
    }

    #[test]
    fn access_clock_wrap_preserves_lru_order_and_default_limits_match_upstream() {
        let mut cache = Cache::default();
        assert_eq!(cache.bytes_limit(), 2 * 1024 * 1024);
        assert_eq!(cache.count_limit(), 2048);
        image(&mut cache, 1, 1, 0);
        image(&mut cache, 2, 1, 0);
        cache.clock = u64::MAX;
        image(&mut cache, 1, 1, 0);
        cache.set_limits(usize::MAX, 1);
        assert!(cache.strikes.contains_key(&1));
        assert!(!cache.strikes.contains_key(&2));
    }
}
