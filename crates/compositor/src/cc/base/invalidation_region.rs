// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/invalidation_region.h:1-38
// Source: cc/base/invalidation_region.cc:1-58

use super::region::Region;
use crate::adapter::{geometry::Rect, region_backend::SkRegionBackend};

// Source: cc/base/invalidation_region.cc:9-13
const kMaxInvalidationRectCount: usize = 256;

// Source: cc/base/invalidation_region.h:15-34
pub struct InvalidationRegion<B: SkRegionBackend> {
    region_: Region<B>,
    pending_rects_: Vec<Rect>,
}

impl<B: SkRegionBackend> Default for InvalidationRegion<B> {
    // Source: cc/base/invalidation_region.cc:17-19
    fn default() -> Self {
        Self {
            region_: Region::default(),
            pending_rects_: Vec::new(),
        }
    }
}

impl<B: SkRegionBackend> InvalidationRegion<B> {
    // Source: cc/base/invalidation_region.h:19-21
    pub fn new() -> Self {
        Self::default()
    }

    // Source: cc/base/invalidation_region.cc:21-24
    pub fn Swap(&mut self, region: &mut Region<B>) {
        self.FinalizePendingRects();
        self.region_.Swap(region);
    }

    // Source: cc/base/invalidation_region.cc:26-29
    pub fn Clear(&mut self) {
        self.pending_rects_.clear();
        self.region_.Clear();
    }

    // Source: cc/base/invalidation_region.cc:31-36
    pub fn Union(&mut self, rect: Rect) {
        if self.pending_rects_.len() >= kMaxInvalidationRectCount {
            self.pending_rects_[0].Union(rect);
        } else {
            self.pending_rects_.push(rect);
        }
    }

    // Source: cc/base/invalidation_region.h:25-25
    pub fn IsEmpty(&self) -> bool {
        self.pending_rects_.is_empty() && self.region_.IsEmpty()
    }

    // Source: cc/base/invalidation_region.cc:38-55
    fn FinalizePendingRects(&mut self) {
        if self.pending_rects_.is_empty() {
            return;
        }
        if self.region_.GetRegionComplexity() as usize + self.pending_rects_.len()
            > kMaxInvalidationRectCount
        {
            let mut pending_bounds = self.region_.bounds();
            let mut rects_bounds = self.pending_rects_[0];
            for rect in &self.pending_rects_[1..] {
                rects_bounds.Union(*rect);
            }
            pending_bounds.Union(rects_bounds);
            self.region_.AssignRect(pending_bounds);
        } else {
            for rect in &self.pending_rects_ {
                self.region_.Union(rect);
            }
        }
        self.pending_rects_.clear();
    }
}

// Source: cc/base/invalidation_region.h:36-38; cc/base/invalidation_region.cc:5-7,15,57-58.
// Includes, namespaces, and include guards are represented by Rust modules.
