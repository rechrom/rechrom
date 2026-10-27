//! PaintArtifact layerization and Paint-property to cc-property conversion.
//!
//! Inputs are immutable Paint values. Outputs are compositor layer/tree
//! mutations; this package does not own Paint production or Viz submission.

#![allow(non_snake_case)]

pub mod background_color;
pub mod chunk_policy;
pub mod chunk_to_layer_mapper;
pub mod content_layer_client;
pub mod effect_decomposition;
pub mod fast_path_update;
pub mod full_update;
pub mod layer_search;
pub mod layerize_walk;
pub mod paint_geometry;
pub mod pending_layer_policy;
pub mod property_tree_policy;
pub mod repaint_policy;
pub mod stats;
pub mod transform_decomposition;

#[derive(Clone, Copy)]
pub enum CompositingReason {
    OverflowScrolling,
    FixedAttachmentBackground,
    Caret,
    Scrollbar,
    Canvas,
    DevToolsOverlay,
    Plugin,
    Video,
    IFrame,
    LinkHighlight,
    Viewport,
    ViewTransitionContent,
    BackfaceVisibilityHidden,
    Overlap,
}

pub trait CompositingReasons: Default + Clone {
    fn empty(&self) -> bool;
    fn Put(&mut self, reason: CompositingReason);
    fn PutAll(&mut self, reasons: Self);
}
