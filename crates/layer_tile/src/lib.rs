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
//! FramePlan. Renderer owns pixels, backend record compilation and composition.
//! Validated raster completion updates tile readiness independently of final
//! composition. Retirement notifications remain pending until consumed, and
//! resource loss invalidates readiness without recycling tile IDs. Rejected
//! replay topologies are explicit; a plan is not proof a backend supports it.

mod engine;
mod geometry;
mod manager;
mod plan;
#[doc(hidden)]
pub mod recording;

pub use engine::{LayerTileEngine, TileResourceOwner};
pub use geometry::{
    compositor_transform, compositor_transform_with_scroll, needs_transparent_backing,
    raster_properties, resolved_compositor_properties, resolved_compositor_properties_with_scroll,
    resolved_raster_properties, resolved_record_clip_bounds,
};
pub use plan::*;
