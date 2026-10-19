//! Execute native Viz aggregate output into a caller-owned pixel target.
pub mod native;
pub use native::{RenderTarget, RenderUpdate, Renderer};
pub use skia::PixelFormat;
