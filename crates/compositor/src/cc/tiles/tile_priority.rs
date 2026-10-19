// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_priority.h:1-137
// Source: cc/tiles/tile_priority.cc:1-116

use crate::adapter::{geometry::Size, property_types::PropertyTrace};
use crate::cc::base::math_util::{MathTraceGeometry, MathUtil};

// Source: cc/tiles/tile_priority.h:21-27
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhichTree {
    ACTIVE_TREE = 0,
    PENDING_TREE = 1,
}
pub const LAST_TREE: WhichTree = WhichTree::PENDING_TREE;

// Source: cc/tiles/tile_priority.h:29-33
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileResolution {
    HIGH_RESOLUTION = 1,
    NON_IDEAL_RESOLUTION = 2,
}

// Source: cc/tiles/tile_priority.cc:14-24
pub fn TileResolutionToString(resolution: TileResolution) -> &'static str {
    match resolution {
        TileResolution::HIGH_RESOLUTION => "HIGH_RESOLUTION",
        TileResolution::NON_IDEAL_RESOLUTION => "NON_IDEAL_RESOLUTION",
    }
}

// Source: cc/tiles/tile_priority.h:36-60
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PriorityBin {
    NOW,
    SOON,
    EVENTUALLY,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TilePriority {
    pub resolution: TileResolution,
    pub priority_bin: PriorityBin,
    pub distance_to_visible: f32,
}

impl Default for TilePriority {
    // Source: cc/tiles/tile_priority.h:39-42
    fn default() -> Self {
        Self {
            resolution: TileResolution::NON_IDEAL_RESOLUTION,
            priority_bin: PriorityBin::EVENTUALLY,
            distance_to_visible: f32::INFINITY,
        }
    }
}

impl TilePriority {
    // Source: cc/tiles/tile_priority.h:44-50
    pub const fn new(
        resolution: TileResolution,
        priority_bin: PriorityBin,
        distance_to_visible: f32,
    ) -> Self {
        Self {
            resolution,
            priority_bin,
            distance_to_visible,
        }
    }

    // Source: cc/tiles/tile_priority.h:54-58
    pub fn IsHigherPriorityThan(&self, other: &Self) -> bool {
        self.priority_bin < other.priority_bin
            || (self.priority_bin == other.priority_bin
                && self.distance_to_visible < other.distance_to_visible)
    }

    // Source: cc/tiles/tile_priority.cc:38-44
    pub fn AsValueInto(&self, state: &mut impl PropertyTrace) {
        state.SetString("resolution", TileResolutionToString(self.resolution));
        state.SetString("priority_bin", TilePriorityBinToString(self.priority_bin));
        state.SetDouble(
            "distance_to_visible",
            MathUtil::AsDoubleSafely(self.distance_to_visible as f64),
        );
    }
}

// Source: cc/tiles/tile_priority.cc:26-36
pub fn TilePriorityBinToString(bin: PriorityBin) -> &'static str {
    match bin {
        PriorityBin::NOW => "NOW",
        PriorityBin::SOON => "SOON",
        PriorityBin::EVENTUALLY => "EVENTUALLY",
    }
}

// Source: cc/tiles/tile_priority.h:66-80
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileMemoryLimitPolicy {
    ALLOW_NOTHING = 0,
    ALLOW_ABSOLUTE_MINIMUM = 1,
    ALLOW_PREPAINT_ONLY = 2,
    ALLOW_ANYTHING = 3,
}

// Source: cc/tiles/tile_priority.cc:46-61
pub fn TileMemoryLimitPolicyToString(policy: TileMemoryLimitPolicy) -> &'static str {
    match policy {
        TileMemoryLimitPolicy::ALLOW_NOTHING => "ALLOW_NOTHING",
        TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM => "ALLOW_ABSOLUTE_MINIMUM",
        TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY => "ALLOW_PREPAINT_ONLY",
        TileMemoryLimitPolicy::ALLOW_ANYTHING => "ALLOW_ANYTHING",
    }
}

// Source: cc/tiles/tile_priority.h:82-84
// Source: cc/tiles/tile_priority.cc:63-73
pub fn IsTileMemoryLimitPolicyMoreRestictive(
    policy1: TileMemoryLimitPolicy,
    policy2: TileMemoryLimitPolicy,
) -> bool {
    policy1 < policy2
}

// Source: cc/tiles/tile_priority.h:86-93
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreePriority {
    SAME_PRIORITY_FOR_BOTH_TREES,
    SMOOTHNESS_TAKES_PRIORITY,
    NEW_CONTENT_TAKES_PRIORITY,
}
pub const LAST_TREE_PRIORITY: TreePriority = TreePriority::NEW_CONTENT_TAKES_PRIORITY;

// Source: cc/tiles/tile_priority.cc:75-88
pub fn TreePriorityToString(priority: TreePriority) -> &'static str {
    match priority {
        TreePriority::SAME_PRIORITY_FOR_BOTH_TREES => "SAME_PRIORITY_FOR_BOTH_TREES",
        TreePriority::SMOOTHNESS_TAKES_PRIORITY => "SMOOTHNESS_TAKES_PRIORITY",
        TreePriority::NEW_CONTENT_TAKES_PRIORITY => "NEW_CONTENT_TAKES_PRIORITY",
    }
}

// Source: cc/tiles/tile_priority.cc:90-107. This typed boundary is the Rust
// counterpart of the generated Perfetto enum returned by the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtozeroTreePriority {
    TREE_PRIORITY_UNSPECIFIED,
    TREE_PRIORITY_SAME_PRIORITY_FOR_BOTH_TREES,
    TREE_PRIORITY_SMOOTHNESS_TAKES_PRIORITY,
    TREE_PRIORITY_NEW_CONTENT_TAKES_PRIORITY,
}

pub fn TreePriorityToProtozeroEnum(priority: TreePriority) -> ProtozeroTreePriority {
    match priority {
        TreePriority::SAME_PRIORITY_FOR_BOTH_TREES => {
            ProtozeroTreePriority::TREE_PRIORITY_SAME_PRIORITY_FOR_BOTH_TREES
        }
        TreePriority::SMOOTHNESS_TAKES_PRIORITY => {
            ProtozeroTreePriority::TREE_PRIORITY_SMOOTHNESS_TAKES_PRIORITY
        }
        TreePriority::NEW_CONTENT_TAKES_PRIORITY => {
            ProtozeroTreePriority::TREE_PRIORITY_NEW_CONTENT_TAKES_PRIORITY
        }
    }
}

// Source: cc/tiles/tile_priority.h:101-133
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalStateThatImpactsTilePriority {
    pub memory_limit_policy: TileMemoryLimitPolicy,
    pub soft_memory_limit_in_bytes: usize,
    pub hard_memory_limit_in_bytes: usize,
    pub num_resources_limit: usize,
    pub tree_priority: TreePriority,
    pub viewport_size: Size,
}

impl Default for GlobalStateThatImpactsTilePriority {
    fn default() -> Self {
        Self {
            memory_limit_policy: TileMemoryLimitPolicy::ALLOW_NOTHING,
            soft_memory_limit_in_bytes: 0,
            hard_memory_limit_in_bytes: 0,
            num_resources_limit: 0,
            tree_priority: TreePriority::SAME_PRIORITY_FOR_BOTH_TREES,
            viewport_size: Size::default(),
        }
    }
}

impl GlobalStateThatImpactsTilePriority {
    // Source: cc/tiles/tile_priority.cc:109-116
    pub fn AsValueInto(&self, state: &mut impl PropertyTrace) {
        state.SetString(
            "memory_limit_policy",
            TileMemoryLimitPolicyToString(self.memory_limit_policy),
        );
        state.SetInteger(
            "soft_memory_limit_in_bytes",
            self.soft_memory_limit_in_bytes.min(i32::MAX as usize) as i32,
        );
        state.SetInteger(
            "hard_memory_limit_in_bytes",
            self.hard_memory_limit_in_bytes.min(i32::MAX as usize) as i32,
        );
        state.SetInteger(
            "num_resources_limit",
            self.num_resources_limit.min(i32::MAX as usize) as i32,
        );
        state.SetString("tree_priority", TreePriorityToString(self.tree_priority));
        self.viewport_size.AddToTracedValue("viewport_size", state);
    }
}

// Source: cc/tiles/tile_priority.h:5-19,34,62-64,96-99,135-137;
// Source: cc/tiles/tile_priority.cc:5-12.
// Includes, namespaces, export decoration, and include guards map to Rust
// modules and typed adapter boundaries.
