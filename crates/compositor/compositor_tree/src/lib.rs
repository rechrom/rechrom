//! Pending/active compositor tree vocabulary and synchronization algorithms.
//!
//! Tree owners stay private to the compositor pipeline; this package exposes
//! no Page, Paint, Viz, renderer, or platform owner.

#![allow(non_camel_case_types)]

pub mod activation;
pub mod clip_node;
pub mod clip_tree_update;
pub mod commit_properties;
pub mod commit_sync;
pub mod commit_types;
pub mod damage_reason;
pub mod effect_node;
pub mod effect_tree_update;
pub mod enum_set;
pub mod geometry;
pub mod input_geometry;
pub mod layer_collections;
pub mod math_util;
pub mod property_ids;
pub mod property_push;
pub mod property_tree;
pub mod property_tree_update;
pub mod property_types;
pub mod region;
pub mod region_backend;
pub mod render_surface_lifecycle;
pub mod render_target_update;
pub mod scroll_delta_collection;
pub mod scrollbar_opacity_sync;
pub mod snap_geometry;
pub mod sticky_position_constraint;
pub mod subnormal_float;
pub mod synced_property;
pub mod target_property;
pub mod transform;
pub mod transform_node;
pub mod viewport_property_ids;

/// Opaque scrolling/animation identity from cc/paint/element_id.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId {
    id_: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeHostId(pub u64);

impl ElementId {
    pub const kInvalidElementId: u64 = 0;
    pub const kDeletedElementId: u64 = u64::MAX;

    pub const fn new(id: u64) -> Self {
        debug_assert!(id != Self::kInvalidElementId);
        debug_assert!(id != Self::kDeletedElementId);
        Self { id_: id }
    }

    pub const fn DeletedValue() -> Self {
        Self {
            id_: Self::kDeletedElementId,
        }
    }

    pub const fn GetInternalValue(self) -> u64 {
        self.id_
    }

    pub const fn IsValidInternalValue(value: u64) -> bool {
        value != Self::kInvalidElementId && value != Self::kDeletedElementId
    }

    pub const fn IsValid(self) -> bool {
        Self::IsValidInternalValue(self.id_)
    }

    pub fn ToString(self) -> String {
        format!("({})", self.id_)
    }
}

pub fn RemapElementIdToCcNamespace(element_id: ElementId) -> ElementId {
    assert!(element_id.GetInternalValue() & 1 == 0);
    ElementId::new(element_id.GetInternalValue() | 1)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhichTree {
    ACTIVE_TREE = 0,
    PENDING_TREE = 1,
}

pub const LAST_TREE: WhichTree = WhichTree::PENDING_TREE;
