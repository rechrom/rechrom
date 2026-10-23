//! Borrowed host output for the concrete Skia renderer backend.

/// Host-visible N32 byte layout. Concrete Skia implementations convert this
/// value inside the renderer package; the browser/window embedding does not
/// depend on either Skia crate's pixel-format vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8888,
    Bgra8888,
    Bgrx8888,
}

/// Borrowed host output; no reference escapes Render.
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub format: PixelFormat,
    /// Move-only output storage bound directly as the root framebuffer.
    pub pixels: Vec<u32>,
    /// The target still contains the pixels from this Renderer instance's
    /// previous completed aggregate at the same size and format.
    pub retained: bool,
}
