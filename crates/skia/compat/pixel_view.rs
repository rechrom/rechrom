//! Checked mutable packed-RGBA view for local blur-pass adapters.
//! The caller owns storage; this convenience API is not an official Skia class.
pub struct PixelView<'a> {
    pixels: &'a mut [u8],
    width: u32,
    height: u32,
}
impl<'a> PixelView<'a> {
    pub fn from_rgba(pixels: &'a mut [u8], width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let len = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if pixels.len() != len {
            return None;
        }
        Some(Self {
            pixels,
            width,
            height,
        })
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn data(&self) -> &[u8] {
        self.pixels
    }
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.pixels
    }
}
