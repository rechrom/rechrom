//! Native cc-selected CPU raster execution and external Skia replay helpers.
//!
//! The compositor selects tile work and consumes only opaque resource
//! identities. This crate executes immutable paint records through Skia and
//! owns the resulting CPU resources. It has no render-pass, window, swap or
//! presentation responsibilities.

#[cfg(all(test, feature = "pure_replay", feature = "source_replay"))]
mod analytic_aa;
mod basic_replay;
pub mod convert;
#[cfg(feature = "font_backend")]
pub mod font_backend;
#[cfg(feature = "profiling")]
pub mod profiling_snapshot;
#[cfg(feature = "pure_replay")]
pub mod pure_replay;
mod reflection_mask;
#[cfg(feature = "source_replay")]
pub mod source_replay;
#[cfg(feature = "pure_replay")]
pub mod surface;
pub mod svg_filter_graph;

pub use basic_replay::{RasterizeDisplayItemList, WriteDisplayItemListPng};
pub use reflection_mask::CompileReflectionMaskLayers;
pub use skia::compat::png::EncodeRgbaPng;
#[cfg(feature = "pure_replay")]
pub use skia::cpu::hvgl;
pub use skia::PixelFormat;

#[cfg(feature = "pure_replay")]
pub mod native;

pub mod native_filters;
