//! cc-selected raster execution.
//!
//! The compositor selects tile work and consumes only opaque resource
//! identities. This crate executes immutable paint records through its private
//! backend and owns the resulting resources. It has no render-pass, window, swap or
//! presentation responsibilities.

// Chromium identifiers keep their source spelling.
#![allow(non_snake_case)]

pub mod convert;
mod png;
mod reflection_mask;
pub(crate) mod raster_commands {
    pub mod commands;
    pub mod filter_graph;
}
pub mod svg_filter_graph;

pub use png::EncodeRgbaPng;
pub use reflection_mask::CompileReflectionMaskLayers;

pub mod execution;
#[cfg(any(target_vendor = "apple", windows))]
pub use execution::SkiaFontBackendFactory;
pub use execution::{
    Canvas2DRasterBackendFactory, ImageBitmapRespectingOrientation, MapResourceForRead,
    MapVideoResourceForRead, Nv12ReadView, RasterEngine, RasterReadView,
    DocumentPaintRecord, RasterizeDocumentDragImage, RasterizeDocumentImageBitmap, ResizeAndOrientDragImage,
    ResizeDragImageBitmap, ScaleCachedDragImage, ScaleDocumentDragImage,
};
pub use execution::{
    RasterizePaintOps, RasterizePaintOpsTransparent, RasterizePaintOpsWithScale,
};
