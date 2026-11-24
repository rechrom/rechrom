//! cc property trees and their nodes, ids and shared tree values
//! (cc/trees/property_tree.cc and friends), named by both the main and the
//! impl side. Impl-only pending/active synchronization lives in
//! `compositor_tree_sync`. This package owns no lifecycle or engine state.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod clip_node;
pub mod commit_types;
pub mod damage_reason;
pub mod effect_node;
pub mod effect_tree_update;
pub mod enum_set;
pub mod geometry;
pub mod input_geometry;
pub mod math_util;
pub mod property_ids;
pub mod property_tree;
pub mod property_types;
pub mod region;
pub mod region_backend;
pub mod render_surface_lifecycle;
pub mod scroll_delta_collection;
pub mod snap_geometry;
pub mod sticky_position_constraint;
pub mod subnormal_float;
pub mod synced_property;
pub mod target_property;
pub mod transform;
pub mod transform_node;
pub mod viewport_property_ids;

pub use scroll_snap::{ElementId, RemapElementIdToCcNamespace};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeHostId(pub u64);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhichTree {
    ACTIVE_TREE = 0,
    PENDING_TREE = 1,
}

pub const LAST_TREE: WhichTree = WhichTree::PENDING_TREE;
