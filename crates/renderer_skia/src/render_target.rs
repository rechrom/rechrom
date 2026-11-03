//! Host-owned output for the concrete Skia renderer backend.

use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

/// Host-visible N32 byte layout. Concrete Skia implementations convert this
/// value inside the renderer package; the browser/window embedding does not
/// depend on either Skia crate's pixel-format vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8888,
    Bgra8888,
    Bgrx8888,
}

/// Storage supplied by the embedding. The renderer borrows its pixels while
/// drawing and returns the same owner after the synchronous Render call.
pub trait HostPixelStorage: Any {
    fn pixels(&self) -> &[u32];
    fn pixels_mut(&mut self) -> &mut [u32];
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// A render-pass backing can temporarily exchange its owned pixels with the
/// host's native target. This is the translated SoftwareRenderer root-target
/// handoff: object identity and allocation ownership stay with the host.
pub enum PixelStorage {
    Owned(Vec<u32>),
    Host(Box<dyn HostPixelStorage>),
}

impl PixelStorage {
    pub fn owned(pixels: Vec<u32>) -> Self {
        Self::Owned(pixels)
    }

    pub fn host(storage: impl HostPixelStorage) -> Self {
        Self::Host(Box::new(storage))
    }

    pub fn into_host<T: Any>(self) -> Result<Box<T>, Box<dyn Any>> {
        match self {
            Self::Host(storage) => storage.into_any().downcast::<T>(),
            Self::Owned(pixels) => Err(Box::new(pixels)),
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        self
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        self
    }
}

impl Deref for PixelStorage {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(pixels) => pixels,
            Self::Host(storage) => storage.pixels(),
        }
    }
}

impl DerefMut for PixelStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(pixels) => pixels,
            Self::Host(storage) => storage.pixels_mut(),
        }
    }
}

pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub format: PixelFormat,
    /// Move-only output storage bound directly as the root framebuffer.
    pub pixels: PixelStorage,
    /// The target still contains the pixels from this Renderer instance's
    /// previous completed aggregate at the same size and format.
    pub retained: bool,
}
