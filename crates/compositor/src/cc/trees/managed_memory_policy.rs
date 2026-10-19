// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/managed_memory_policy.h:1-38
// Source: cc/trees/managed_memory_policy.cc:1-63
use crate::adapter::resource_types::{MemoryAllocation, PriorityCutoff, TileMemoryLimitPolicy};

// Source: cc/trees/managed_memory_policy.h:16-34
#[derive(Clone, Copy, Debug)]
pub struct ManagedMemoryPolicy {
    // Source: cc/trees/managed_memory_policy.h:28-30
    pub bytes_limit_when_visible: usize,
    pub priority_cutoff_when_visible: PriorityCutoff,
    pub num_resources_limit: usize,
}

impl ManagedMemoryPolicy {
    // Source: cc/trees/managed_memory_policy.cc:13
    pub const kDefaultNumResourcesLimit: usize = 10 * 1000 * 1000;

    // Source: cc/trees/managed_memory_policy.h:19
    // Source: cc/trees/managed_memory_policy.cc:17-20
    pub fn new(bytes_limit_when_visible: usize) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_EVERYTHING,
            num_resources_limit: Self::kDefaultNumResourcesLimit,
        }
    }

    // Source: cc/trees/managed_memory_policy.h:20
    // Source: cc/trees/managed_memory_policy.cc:22-26
    pub fn FromAllocation(allocation: &MemoryAllocation) -> Self {
        Self {
            bytes_limit_when_visible: allocation.bytes_limit_when_visible as usize,
            priority_cutoff_when_visible: allocation.priority_cutoff_when_visible,
            num_resources_limit: Self::kDefaultNumResourcesLimit,
        }
    }

    // Source: cc/trees/managed_memory_policy.h:21-24
    // Source: cc/trees/managed_memory_policy.cc:28-34
    pub fn WithLimits(
        bytes_limit_when_visible: usize,
        priority_cutoff_when_visible: PriorityCutoff,
        num_resources_limit: usize,
    ) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible,
            num_resources_limit,
        }
    }

    // Source: cc/trees/managed_memory_policy.h:32-33
    // Source: cc/trees/managed_memory_policy.cc:45-61
    pub fn PriorityCutoffToTileMemoryLimitPolicy(
        priority_cutoff: PriorityCutoff,
    ) -> TileMemoryLimitPolicy {
        match priority_cutoff {
            PriorityCutoff::CUTOFF_ALLOW_NOTHING => TileMemoryLimitPolicy::ALLOW_NOTHING,
            PriorityCutoff::CUTOFF_ALLOW_REQUIRED_ONLY => {
                TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM
            }
            PriorityCutoff::CUTOFF_ALLOW_NICE_TO_HAVE => TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY,
            PriorityCutoff::CUTOFF_ALLOW_EVERYTHING => TileMemoryLimitPolicy::ALLOW_ANYTHING,
        }
        // The exhaustive Rust enum makes the source NOTREACHED() unreachable.
    }
}

impl PartialEq for ManagedMemoryPolicy {
    // Source: cc/trees/managed_memory_policy.h:25
    // Source: cc/trees/managed_memory_policy.cc:36-40
    fn eq(&self, other: &Self) -> bool {
        self.bytes_limit_when_visible == other.bytes_limit_when_visible
            && self.priority_cutoff_when_visible == other.priority_cutoff_when_visible
            && self.num_resources_limit == other.num_resources_limit
    }
    // Source: cc/trees/managed_memory_policy.h:26
    // Source: cc/trees/managed_memory_policy.cc:42-44
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for ManagedMemoryPolicy {}
