#![allow(non_snake_case)]

//! PrePaint property, coordinate, cull, and invalidation lifecycle.
//!
//! Shared property values remain owned by `paint_model`; this package owns
//! `PrePaintTreeWalk`, property-tree construction, `CullRectUpdater`, and paint
//! invalidation. Recording traversal stays in `paint_walk`, and equality
//! proofs stay in `paint_reuse`.

pub use paint_model::paint_property_tree;
pub use paint_model::property_node_geometry;
pub use paint_reuse::pre_paint_state_compare;

pub mod background_geometry;
pub mod block_paint_invalidator;
pub mod box_paint_invalidator;
pub mod client_rects;
pub mod compositing_reason_finder;
pub mod cull_rect_updater;
pub mod nine_piece_geometry;
pub mod nine_piece_image_grid;
pub mod object_paint_invalidator;
pub mod paint_invalidator;
pub mod paint_shader_resolver;
pub mod pre_paint_tree_walk;
pub mod table_cell_paint_invalidator;

pub use block_paint_invalidator::BlockPaintInvalidator;
pub use box_paint_invalidator::BoxPaintInvalidator;
pub use compositing_reason_finder::CompositingReasonFinder;
pub use cull_rect_updater::CullRectUpdater;
pub use object_paint_invalidator::ObjectPaintInvalidator;
pub use paint_invalidator::{PaintInvalidator, PaintInvalidatorContext};
pub use pre_paint_tree_walk::{PaintFragmentSource, PaintTreeNode, PrePaintTreeWalk};
pub use table_cell_paint_invalidator::TableCellPaintInvalidator;

#[derive(Clone, Copy, Debug, Default)]
pub struct PrePaintUpdateStats {
    pub computed_nodes: usize,
    pub reused_nodes: usize,
    pub invalidation_only_nodes: usize,
    pub same_geometry_rebuilds: usize,
    pub property_geometry_changes: usize,
    pub property_geometry_changes_with_own_keys: usize,
    pub property_geometry_changes_needing_property_update: usize,
    pub skipped_subtrees: usize,
    pub post_walk: bool,
}
