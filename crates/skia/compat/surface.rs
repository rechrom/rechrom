//! Local CPU readback frame storage; not the C++ SkSurface class.
use std::io;
pub struct RasterSurface {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
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
            pixels: vec![255; bytes],
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
            pixels,
        })
    }

    /// Replace CPU readback with a frame produced for this target.
    pub fn replace_pixels(&mut self, pixels: Vec<u8>) {
        assert_eq!(
            pixels.len(),
            self.pixels.len(),
            "surface readback size differs"
        );
        self.pixels = pixels;
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// CPU readback for screenshots and software presentation.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
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
