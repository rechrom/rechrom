#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{kIndefiniteSize, LayoutUnit, Traceable, Visitor};

use super::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use super::recent_entries::RecentEntries;

// cpp: layoutng/internal/min_max_sizes_cache.h:37-41
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub sizes: MinMaxSizes,
    pub initial_block_size: LayoutUnit,
    pub depends_on_block_constraints: bool,
}

// cpp: layoutng/internal/min_max_sizes_cache.h:21-92
#[derive(Default)]
pub struct MinMaxSizesCache {
    cache_: RecentEntries<Entry, 8>,
}

impl MinMaxSizesCache {
    // cpp: layoutng/internal/min_max_sizes_cache.h:31-31
    pub const kMaxCacheEntries: usize = 8;

    // cpp: layoutng/internal/min_max_sizes_cache.h:33-33
    // The C++ GC trace method is intentionally empty: Entry has no GC edges.
    pub fn Trace(&self, _visitor: &mut Visitor) {}

    // cpp: layoutng/internal/min_max_sizes_cache.h:44-63
    pub fn Find(&mut self, initial_block_size: LayoutUnit) -> Option<MinMaxSizesResult> {
        debug_assert_ne!(initial_block_size, kIndefiniteSize);
        let copy = *self
            .cache_
            .touch_where(|entry| entry.initial_block_size == initial_block_size)?;
        Some(MinMaxSizesResult::new(
            copy.sizes,
            copy.depends_on_block_constraints,
        ))
    }

    // cpp: layoutng/internal/min_max_sizes_cache.h:66-86
    pub fn Add(
        &mut self,
        sizes: &MinMaxSizes,
        initial_block_size: LayoutUnit,
        depends_on_block_constraints: bool,
    ) {
        debug_assert!(self
            .cache_
            .iter()
            .all(|entry| entry.initial_block_size != initial_block_size));
        self.cache_.push(Entry {
            sizes: *sizes,
            initial_block_size,
            depends_on_block_constraints,
        });
    }

    // cpp: layoutng/internal/min_max_sizes_cache.h:88-88
    pub fn Clear(&mut self) {
        self.cache_.clear();
    }
}

impl Traceable for MinMaxSizesCache {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        MinMaxSizesCache::Trace(self, visitor);
    }
}
