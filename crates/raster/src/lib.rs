//! cc-selected raster execution.
//!
//! The compositor selects tile work and consumes only opaque resource
//! identities. This crate executes immutable paint records through its private
//! backend and owns the resulting resources. It has no render-pass, window, swap or
//! presentation responsibilities.

pub mod convert;
mod png;
mod reflection_mask;
pub mod svg_filter_graph;

pub use png::EncodeRgbaPng;
pub use reflection_mask::CompileReflectionMaskLayers;

pub mod execution;
#[cfg(target_vendor = "apple")]
pub use execution::SkiaFontBackendFactory;
pub use execution::{
    Canvas2DRasterBackendFactory, ImageBitmapRespectingOrientation, MapResourceForRead,
    MapVideoResourceForRead, Nv12ReadView, RasterEngine, RasterReadView,
    RasterizeDocumentDragImage, RasterizeDocumentImageBitmap, ResizeAndOrientDragImage,
    ResizeDragImageBitmap, ScaleCachedDragImage, ScaleDocumentDragImage,
};
#[cfg(any(test, feature = "artifact_test_support"))]
pub use execution::{
    RasterizePaintArtifact, RasterizePaintArtifactTransparent, RasterizePaintArtifactWithScale,
};
