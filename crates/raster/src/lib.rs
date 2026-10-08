//! Tile rasterization and retained raster-resource ownership.
//!
//! The compositor selects tile work and consumes only opaque resource
//! identities. This crate executes immutable paint records through Skia and
//! owns the resulting CPU resources. It has no render-pass, window, swap or
//! presentation responsibilities.

#[cfg(all(test, feature = "pure_replay", feature = "source_replay"))]
mod analytic_aa;
mod basic_replay;
pub mod convert;
mod engine;
#[cfg(feature = "source_replay")]
pub mod font_backend;
mod layer_replay;
#[cfg(feature = "profiling")]
pub mod profiling_snapshot;
#[cfg(feature = "pure_replay")]
pub mod pure_replay;
mod solid;
mod solid_analysis;
#[cfg(feature = "source_replay")]
pub mod source_replay;
#[cfg(feature = "pure_replay")]
pub mod surface;
mod tile_worker;

pub use basic_replay::{RasterizeDisplayItemList, WriteDisplayItemListPng};
pub use engine::{
    RasterEngine, RasterResource, RasterResourceProvider, RasterRowSupport, RasterStats,
    RasterUpdate, RowSupport,
};
pub use skia::compat::png::EncodeRgbaPng;
#[cfg(feature = "pure_replay")]
pub use skia::cpu::hvgl;
pub use skia::PixelFormat;
pub use solid::RasterDrawMode;
