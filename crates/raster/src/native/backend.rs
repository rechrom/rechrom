//! Drawing backend behind [`super::RasterEngine`].
//!
//! The engine owns allocation identity, exclusive write admission, content
//! identity and publication. A backend receives one already-selected cc
//! playback request and one unpublished pixel allocation. It may mutate only
//! that allocation through an exclusive target view; success returns control
//! to the engine before it publishes completion.
//! No Page, layer, tiling, task graph, ResourcePool or presentation owner is
//! visible through this interface.

use super::*;

pub(super) type NativeRasterPlayback<'request, 'image> = RasterPlayback<
    'request,
    NativeRasterSource<SkRegion>,
    RasterTransform,
    RasterImageProviderRef<'image, SkRegion>,
    ScrollOffsetMap,
>;

/// Engine-owned storage admitted for one raster invocation. The bytes have no
/// immutable readers while this value exists; a backend receives only an
/// exclusive view and cannot replace the allocation metadata. RasterEngine
/// publishes the bytes only after `RasterBackend::Raster` succeeds.
pub(super) struct RasterBackendTarget {
    size: Size,
    format: SharedImageFormat,
    expected_len: usize,
    pixels: Vec<u8>,
}

impl RasterBackendTarget {
    pub fn new(size: Size, format: SharedImageFormat, pixels: Vec<u8>) -> io::Result<Self> {
        let expected_len = pixel_len(size)?;
        if pixels.len() != expected_len {
            return Err(io::Error::other(
                "raster storage does not match its allocation metadata",
            ));
        }
        Ok(Self {
            size,
            format,
            expected_len,
            pixels,
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn format(&self) -> SharedImageFormat {
        self.format
    }

    pub fn row_bytes(&self) -> usize {
        self.size.width() as usize * 4
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn into_pixels(self) -> io::Result<Vec<u8>> {
        if self.pixels.len() != self.expected_len {
            return Err(io::Error::other(
                "raster backend did not return the engine-owned allocation",
            ));
        }
        Ok(self.pixels)
    }
}

/// Skia execution boundary owned by the raster package.
///
/// This trait is deliberately package-private. Compositor injects and observes
/// `RasterEngine` through `CompositorRasterProvider`; it never selects a Skia
/// implementation or receives a Canvas.
pub(super) trait RasterBackend: Send + Sync {
    fn Raster(
        &self,
        request: NativeRasterPlayback<'_, '_>,
        target: &mut RasterBackendTarget,
    ) -> io::Result<()>;
}
