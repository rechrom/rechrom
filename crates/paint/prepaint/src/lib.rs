#![allow(non_snake_case)]

//! PrePaint property, coordinate, and invalidation state.
//!
//! The retained tree walker is being migrated behind this boundary. Shared
//! property values remain owned by `paint_model`; traversal and culling are
//! delegated to `paint_walk`, and equality proofs to `paint_reuse`.

pub use paint_model::cull;
pub use paint_model::paint_property_tree;
pub use paint_model::property_node_geometry;
pub use paint_reuse::pre_paint_state_compare;

pub mod background_geometry;
pub mod client_rects;
pub mod nine_piece_geometry;
pub mod nine_piece_image_grid;
pub mod paint_shader_resolver;
pub mod pre_paint_tree_walk;

pub use pre_paint_tree_walk::{PaintFragmentSource, PaintTreeNode, PrePaintTreeWalk};

#[derive(Clone, Copy, Debug, Default)]
pub struct PrePaintUpdateStats {
    pub computed_nodes: usize,
    pub reused_nodes: usize,
    pub invalidation_only_nodes: usize,
    pub geometry_only_nodes: usize,
    pub same_geometry_rebuilds: usize,
    pub property_geometry_changes: usize,
    pub property_geometry_changes_with_own_keys: usize,
    pub property_geometry_changes_needing_property_update: usize,
    pub skipped_subtrees: usize,
    pub post_walk: bool,
}
