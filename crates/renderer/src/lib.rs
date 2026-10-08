//! Execute immutable Viz aggregate output into a caller-owned pixel target.
//!
//! Paint-record replay and tile rasterization live in `raster`; surface
//! scheduling and render-pass aggregation live in `viz`. This crate owns only
//! final render-pass execution and retained output-pass backing.

mod clip_mask;
mod render_pass;
mod renderer;

pub use renderer::{
    RasterResourceRegistry, RenderTarget, RenderUpdate, Renderer, SingleSurfaceResources,
};
pub use skia::PixelFormat;
