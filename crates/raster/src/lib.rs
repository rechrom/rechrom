//! Native cc-selected raster execution and external replay helpers.
//!
//! The compositor selects tile work and consumes only opaque resource
//! identities. This crate executes immutable paint records through its private
//! backend and owns the resulting resources. It has no render-pass, window, swap or
//! presentation responsibilities.

mod basic_replay;
pub mod convert;
mod reflection_mask;
mod png;
pub mod svg_filter_graph;

pub use basic_replay::{RasterizeDisplayItemList, WriteDisplayItemListPng};
pub use png::EncodeRgbaPng;
pub use reflection_mask::CompileReflectionMaskLayers;

pub mod native;
pub use native::{
    MapResourceForRead, RasterEngine, RasterReadView, RasterizePaintArtifact, RasterizePaintArtifactTransparent,
    RasterizePaintArtifactWithScale,
};
