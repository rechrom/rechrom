//! Picture-layer tilings, tile priority, coverage, and tile lifecycle.
//!
//! Raster playback and resource allocation are injected boundaries; they are
//! not owned by this package.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub use compositor_tree::{WhichTree, LAST_TREE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PictureLayerTilingSetId(pub u64);

pub mod eviction_tile_priority_queue;
pub mod index_rect;
pub mod mipmap_util;
pub mod picture_layer_tiling;
pub mod picture_layer_tiling_set;
pub mod picture_layer_tiling_types;
pub mod picture_layer_update;
pub mod prepaint_trim;
pub mod prioritized_tile;
pub mod prioritized_tile_types;
pub mod raster_tile_priority_queue;
pub mod raster_tile_priority_queue_all;
pub mod raster_tile_priority_queue_required;
pub mod reverse_spiral_iterator;
pub mod spiral_iterator;
pub mod tile;
pub mod tile_assignment;
pub mod tile_draw_info;
pub mod tile_manager_client;
pub mod tile_manager_policy;
pub mod tile_manager_settings;
pub mod tile_priority;
pub mod tile_size_calculator;
pub mod tile_task_manager;
pub mod tile_types;
pub mod tiles_with_resource_iterator;
pub mod tiling_coverage_iterator;
pub mod tiling_data;
pub mod tiling_internal;
pub mod tiling_set_coverage_iterator;
pub mod tiling_set_eviction_queue;
pub mod tiling_set_raster_queue_all;
pub mod tiling_set_raster_queue_required;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileMemoryLimitPolicy {
    ALLOW_NOTHING = 0,
    ALLOW_ABSOLUTE_MINIMUM = 1,
    ALLOW_PREPAINT_ONLY = 2,
    ALLOW_ANYTHING = 3,
}

pub fn TileMemoryLimitPolicyToString(policy: TileMemoryLimitPolicy) -> &'static str {
    match policy {
        TileMemoryLimitPolicy::ALLOW_NOTHING => "ALLOW_NOTHING",
        TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM => "ALLOW_ABSOLUTE_MINIMUM",
        TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY => "ALLOW_PREPAINT_ONLY",
        TileMemoryLimitPolicy::ALLOW_ANYTHING => "ALLOW_ANYTHING",
    }
}

pub fn IsTileMemoryLimitPolicyMoreRestictive(
    policy1: TileMemoryLimitPolicy,
    policy2: TileMemoryLimitPolicy,
) -> bool {
    policy1 < policy2
}

// Source: cc/tiles/tile_priority.h:86-93.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TreePriority {
    SAME_PRIORITY_FOR_BOTH_TREES,
    SMOOTHNESS_TAKES_PRIORITY,
    NEW_CONTENT_TAKES_PRIORITY,
}

pub const LAST_TREE_PRIORITY: TreePriority = TreePriority::NEW_CONTENT_TAKES_PRIORITY;

pub fn TreePriorityToString(priority: TreePriority) -> &'static str {
    match priority {
        TreePriority::SAME_PRIORITY_FOR_BOTH_TREES => "SAME_PRIORITY_FOR_BOTH_TREES",
        TreePriority::SMOOTHNESS_TAKES_PRIORITY => "SMOOTHNESS_TAKES_PRIORITY",
        TreePriority::NEW_CONTENT_TAKES_PRIORITY => "NEW_CONTENT_TAKES_PRIORITY",
    }
}

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

pub mod tile_index {
    use std::hash::{Hash, Hasher};

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TileIndex {
        pub i: i32,
        pub j: i32,
    }

    impl TileIndex {
        pub const fn new(i: i32, j: i32) -> Self {
            Self { i, j }
        }
    }

    impl From<(i32, i32)> for TileIndex {
        fn from(index: (i32, i32)) -> Self {
            Self::new(index.0, index.1)
        }
    }

    impl Hash for TileIndex {
        fn hash<H: Hasher>(&self, state: &mut H) {
            let value1 = self.i as u16;
            let value2 = self.j as u16;
            state.write_u32(((value1 as u32) << 16) | value2 as u32);
        }
    }
}
