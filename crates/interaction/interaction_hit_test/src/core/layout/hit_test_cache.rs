// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/hit_test_cache.h:1-80
// Source: third_party/blink/renderer/core/layout/hit_test_cache.cc:1-82

use super::hit_test_location::HitTestLocation;
use super::hit_test_result::HitTestResult;

// Source: hit_test_cache.h:32-36
pub const HIT_TEST_CACHE_SIZE: usize = 2;

// Source: hit_test_cache.h:38-46
pub struct HitTestCacheEntry<L: ?Sized + 'static> {
    pub location: HitTestLocation,
    pub result: HitTestResult<'static, L>,
}

// Source: hit_test_cache.h:48-73
// CachedValues clears borrowed stop-node/callback fields in the source.
// The cache roots result owners, never a request-scoped Layout reference.
pub struct HitTestCache<L: ?Sized + 'static> {
    update_index_: usize,
    items_: Vec<HitTestCacheEntry<L>>,
    dom_tree_version_: u64,
}

impl<L: ?Sized + 'static> Default for HitTestCache<L> {
    // Source: hit_test_cache.h:50
    fn default() -> Self {
        Self {
            update_index_: 0,
            items_: Vec::new(),
            dom_tree_version_: 0,
        }
    }
}
impl<L: ?Sized + 'static> HitTestCache<L> {
    pub fn new() -> Self {
        Self::default()
    }

    // Source: hit_test_cache.cc:11-31
    pub fn LookupCachedResult(
        &self,
        location: &HitTestLocation,
        result: &mut HitTestResult<'_, L>,
        dom_tree_version: u64,
    ) -> bool {
        let mut found = false;
        if result.GetHitTestRequest().AvoidCache() {
            // The source leaves this branch empty.
        } else if dom_tree_version == self.dom_tree_version_ && !location.IsRectBasedTest() {
            for cached in &self.items_ {
                if cached.location.Point() == location.Point()
                    && result
                        .GetHitTestRequest()
                        .EqualForCacheability(cached.result.GetHitTestRequest())
                {
                    found = true;
                    result.AssignCachedValues(&cached.result);
                    break;
                }
            }
        }
        found
    }
}

// Source: hit_test_cache.cc:33-40
impl<L: ?Sized + 'static> HitTestCacheEntry<L> {
    // Rc result fields and request borrows retain the source Trace edges.
    pub fn CacheValues(&mut self, other: &Self) {
        self.location = other.location;
        self.result.Assign(&other.result);
        self.result.CacheValues(&other.result);
    }
}

impl<L: ?Sized + 'static> HitTestCache<L> {
    // Source: hit_test_cache.cc:42-71
    pub fn AddCachedResult(
        &mut self,
        location: &HitTestLocation,
        result: &HitTestResult<'_, L>,
        dom_tree_version: u64,
    ) {
        if !result.IsCacheable() {
            return;
        }
        if result.IsOverEmbeddedContentView()
            && result.GetHitTestRequest().AllowsChildFrameContent()
        {
            return;
        }
        if location.IsRectBasedTest() || result.GetHitTestRequest().ListBased() {
            return;
        }
        if dom_tree_version != self.dom_tree_version_ {
            self.Clear();
        }
        if self.items_.len() < HIT_TEST_CACHE_SIZE {
            self.items_
                .resize_with(self.update_index_ + 1, || HitTestCacheEntry {
                    location: HitTestLocation::default(),
                    result: HitTestResult::default(),
                });
        }
        let entry = HitTestCacheEntry {
            location: *location,
            result: result.OwnedCachedValues(),
        };
        self.items_[self.update_index_].CacheValues(&entry);
        self.dom_tree_version_ = dom_tree_version;
        self.update_index_ += 1;
        if self.update_index_ >= HIT_TEST_CACHE_SIZE {
            self.update_index_ = 0;
        }
    }
    // Source: hit_test_cache.cc:73-76
    pub fn Clear(&mut self) {
        self.update_index_ = 0;
        self.items_.clear();
    }
    // Source: hit_test_cache.cc:78-80
    // Vec/Rc destruction and borrowed request lifetimes supply the GC edges.
}
