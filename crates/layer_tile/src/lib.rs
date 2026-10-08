//! Persistent layer/tiling/tile planning, independent of raster backends.
//!
//! Sources: Blink PendingLayer / PaintArtifactCompositor and cc
//! PictureLayerTiling{,Set} / TileManager. This implements a conservative
//! chunk merging and a conservative same-transform/effect subset of Chromium's
//! overlap layerization, source chunk matching, compatible tiling reuse and
//! invalidation. It does not implement general effect decompositing, GPU
//! priorities or raster execution.
//!
//! LayerTileEngine owns resident metadata and lowers records into a complete
//! FramePlan. Raster owns pixels and backend record compilation; compositor
//! owns final frame construction. Raster completion and resource retirement
//! return through explicit messages, never through callbacks hidden in a plan.

mod engine;
mod geometry;
mod manager;
mod plan;
#[doc(hidden)]
pub mod recording;

pub use engine::LayerTileEngine;
pub use geometry::{
    compositor_transform, compositor_transform_with_scroll, needs_transparent_backing,
    raster_properties, resolved_compositor_properties, resolved_compositor_properties_with_scroll,
    resolved_raster_properties, resolved_record_clip_bounds,
};
pub use plan::*;
