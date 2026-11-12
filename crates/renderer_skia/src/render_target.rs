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

/// Convert native little-endian N32/BGRA pixels to RGBA bytes for host
/// readback. Chromium's Skia swizzle is involutive: the same translated
/// `RGBA_to_BGRA` kernel exchanges red and blue in either direction.
///
/// The renderer package owns the concrete Skia call so embedders keep using
/// backend-neutral byte and pixel slices.
pub fn swizzle_bgra_to_rgba(source: &[u32], output: &mut [u8]) -> bool {
    let Some(required) = source.len().checked_mul(4) else {
        return false;
    };
    if output.len() != required || source.len() > i32::MAX as usize {
        return false;
    }

    #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
    unsafe {
        // Source: third_party/skia/src/core/SkSwizzlePriv.h:18-25 and
        // third_party/skia/src/opts/SkSwizzler_opts.inc:400-401.
        skia::src::core::SkSwizzler_opts::RGBA_to_BGRA(
            output.as_mut_ptr().cast(),
            source.as_ptr(),
            source.len() as i32,
        );
    }

    #[cfg(not(all(target_arch = "aarch64", target_endian = "little")))]
    for (pixel, target) in source.iter().zip(output.chunks_exact_mut(4)) {
        let bytes = pixel.to_ne_bytes();
        target.copy_from_slice(&[bytes[2], bytes[1], bytes[0], bytes[3]]);
    }
    true
}

#[cfg(test)]
mod swizzle_tests {
    use super::swizzle_bgra_to_rgba;

    #[test]
    fn translated_swizzle_matches_scalar_bytes_across_vector_tail() {
        let source = (0..17_u32)
            .map(|index| {
                let red = index.wrapping_mul(17) & 255;
                let green = index.wrapping_mul(29) & 255;
                let blue = index.wrapping_mul(43) & 255;
                let alpha = index.wrapping_mul(61) & 255;
                alpha << 24 | red << 16 | green << 8 | blue
            })
            .collect::<Vec<_>>();
        let expected = source
            .iter()
            .flat_map(|pixel| {
                let bytes = pixel.to_ne_bytes();
                [bytes[2], bytes[1], bytes[0], bytes[3]]
            })
            .collect::<Vec<_>>();
        let mut output = vec![0; expected.len()];
        assert!(swizzle_bgra_to_rgba(&source, &mut output));
        assert_eq!(output, expected);
        assert!(!swizzle_bgra_to_rgba(&source, &mut output[..4]));
    }
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
