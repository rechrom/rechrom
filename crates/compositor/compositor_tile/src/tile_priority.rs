//! Tile priority values and global memory/viewport policy.

use crate::{
    TileMemoryLimitPolicy, TileMemoryLimitPolicyToString, TreePriority, TreePriorityToString,
};
use compositor_tree::geometry::{Size, TracedValue};

pub trait TilePriorityTrace: TracedValue {
    fn SetTileString(&mut self, name: &str, value: &str);
}

impl TilePriorityTrace for compositor_tree::property_types::TracedValueJSON {
    fn SetTileString(&mut self, name: &str, value: &str) {
        compositor_tree::property_types::PropertyTrace::SetString(self, name, value);
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileResolution {
    HIGH_RESOLUTION = 1,
    NON_IDEAL_RESOLUTION = 2,
}

pub fn TileResolutionToString(resolution: TileResolution) -> &'static str {
    match resolution {
        TileResolution::HIGH_RESOLUTION => "HIGH_RESOLUTION",
        TileResolution::NON_IDEAL_RESOLUTION => "NON_IDEAL_RESOLUTION",
    }
}

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
    fn default() -> Self {
        Self {
            resolution: TileResolution::NON_IDEAL_RESOLUTION,
            priority_bin: PriorityBin::EVENTUALLY,
            distance_to_visible: f32::INFINITY,
        }
    }
}

impl TilePriority {
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

    pub fn IsHigherPriorityThan(&self, other: &Self) -> bool {
        self.priority_bin < other.priority_bin
            || (self.priority_bin == other.priority_bin
                && self.distance_to_visible < other.distance_to_visible)
    }

    pub fn AsValueInto(&self, state: &mut impl TilePriorityTrace) {
        state.SetTileString("resolution", TileResolutionToString(self.resolution));
        state.SetTileString("priority_bin", TilePriorityBinToString(self.priority_bin));
        state.SetDouble(
            "distance_to_visible",
            (self.distance_to_visible as f64).min(f64::MAX),
        );
    }
}

pub fn TilePriorityBinToString(bin: PriorityBin) -> &'static str {
    match bin {
        PriorityBin::NOW => "NOW",
        PriorityBin::SOON => "SOON",
        PriorityBin::EVENTUALLY => "EVENTUALLY",
    }
}

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
    pub fn AsValueInto(&self, state: &mut impl TilePriorityTrace) {
        state.SetTileString(
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
        state.SetTileString("tree_priority", TreePriorityToString(self.tree_priority));
        state.BeginDictionary("viewport_size");
        state.SetDouble("width", self.viewport_size.width() as f64);
        state.SetDouble("height", self.viewport_size.height() as f64);
        state.EndDictionary();
    }
}
