//! Typed template-parameter families of the main cc layers (the Rechrom
//! projection of cc::Layer's virtuals and its LayerImpl push targets).

pub mod content_layer_client_impl;
pub mod main_layer;
pub mod paint_artifact_compositor;
pub mod paint_chunks_to_cc_layer;
pub mod paint_geometry;
pub mod pending_layer;
pub mod property_tree_manager;
pub mod raster_invalidator;
pub mod painted_scrollbar_layer;
pub mod picture_layer;
pub mod recording_source;
pub mod scrollbar_layer_base;
pub mod solid_color_layer;
pub mod solid_color_scrollbar_layer;
pub mod surface_layer;

// Module paths of the shared value roots under which the moved translations
// name them (the compositor keeps the same paths). Aliases only.
pub(crate) mod draw_property {
    pub use crate::commit::layer_value_backend::DrawFilters;
}
pub(crate) mod geometry {
    pub use crate::commit::geometry::*;
}
pub(crate) mod input_types {
    pub use crate::commit::input_types::*;
}
pub(crate) mod layer_tree_host_impl {
    pub use crate::commit::ContentColorUsage;
}
pub(crate) mod layer_tree_impl {
    pub use crate::commit::{values::UIResourceId, PropertyTreesFor};
    pub use compositor_input::layer_selection_bound::SelectionBoundType;
    pub use compositor_animation::ElementListType;
    pub use compositor_draw::Color4f;
}
pub(crate) mod occlusion {
    pub use compositor_draw::occlusion_context::SkBlendMode;
}
pub(crate) mod prioritized_tile {
    pub use crate::commit::ScrollOffsetMap;
}
pub(crate) mod property_tree {
    pub use crate::commit::property_tree_services::*;
}
pub(crate) mod property_types {
    pub use crate::commit::property_types::*;
}
pub(crate) mod region_backend {
    pub use crate::commit::region_backend::*;
}
pub(crate) mod commit_state {
    pub use crate::commit::commit_state::*;
}
pub(crate) mod display_item_list {
    pub use crate::commit::paint::display_item_list_types::*;
}
pub(crate) mod picture_layer_impl {
    pub use crate::commit::paint::display_item_list_types::DirectlyCompositedImageInfo;
}
