//! Local CPU readback frame storage; not the C++ SkSurface class.
use std::io;
use std::sync::Arc;

enum RasterPixels {
    Bytes(Vec<u8>),
    RgbaWords {
        words: Arc<Vec<u32>>,
        start: usize,
        len: usize,
    },
}

pub struct RasterSurface {
    width: u32,
    height: u32,
    pixels: RasterPixels,
}

impl RasterSurface {
    pub fn new(width: u32, height: u32) -> io::Result<Self> {
        let bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|count| count.checked_mul(4))
            .filter(|_| width > 0 && height > 0)
            .ok_or_else(|| io::Error::other("invalid raster surface dimensions"))?;
        Ok(Self {
            width,
            height,
            pixels: RasterPixels::Bytes(vec![255; bytes]),
        })
    }

    /// Take ownership of an already rendered CPU frame without initializing
    /// another full-sized blank target first.
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> io::Result<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|count| count.checked_mul(4))
            .filter(|_| width > 0 && height > 0)
            .ok_or_else(|| io::Error::other("invalid raster surface dimensions"))?;
        if pixels.len() != expected {
            return Err(io::Error::other("surface readback size differs"));
        }
        Ok(Self {
            width,
            height,
            pixels: RasterPixels::Bytes(pixels),
        })
    }

    /// Borrow a rectangular row range from a completed RGBA8888 compositor
    /// target. The shared word allocation keeps its original alignment and
    /// ownership while screenshot readers observe native RGBA byte order.
    pub fn from_rgba_words(
        width: u32,
        height: u32,
        words: Arc<Vec<u32>>,
        start: usize,
    ) -> io::Result<Self> {
        let len = (width as usize)
            .checked_mul(height as usize)
            .filter(|_| width > 0 && height > 0)
            .ok_or_else(|| io::Error::other("invalid raster surface dimensions"))?;
        if start.checked_add(len).is_none_or(|end| end > words.len()) {
            return Err(io::Error::other("surface readback range differs"));
        }
        Ok(Self {
            width,
            height,
            pixels: RasterPixels::RgbaWords { words, start, len },
        })
    }

    /// Replace CPU readback with a frame produced for this target.
    pub fn replace_pixels(&mut self, pixels: Vec<u8>) {
        assert_eq!(
            pixels.len(),
            self.width as usize * self.height as usize * 4,
            "surface readback size differs"
        );
        self.pixels = RasterPixels::Bytes(pixels);
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// CPU readback for screenshots and software presentation.
    pub fn pixels(&self) -> &[u8] {
        match &self.pixels {
            RasterPixels::Bytes(pixels) => pixels,
            RasterPixels::RgbaWords { words, start, len } => unsafe {
                // RGBA8888 is defined in byte order. The word allocation stays
                // alive through Arc and this view never changes its layout.
                std::slice::from_raw_parts(words.as_ptr().add(*start).cast::<u8>(), len * 4)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rendered_pixels_transfer_ownership_and_validate_dimensions() {
        let pixels = vec![12; 8 * 7 * 4];
        let pointer = pixels.as_ptr();
        let surface = RasterSurface::from_pixels(8, 7, pixels).unwrap();
        assert_eq!(surface.pixels().as_ptr(), pointer);
        assert!(RasterSurface::from_pixels(0, 7, Vec::new()).is_err());
        assert!(RasterSurface::from_pixels(u32::MAX, u32::MAX, Vec::new()).is_err());
        assert!(RasterSurface::from_pixels(8, 7, vec![0; 8]).is_err());
    }
}
