//! Restricted color-layout subset of SkColorType, with explicit alpha metadata.
//! Official raster pipelines load/store BGRA via swap_rb / swap_rb_dst around
//! load/store_8888. Bgrx8888 additionally represents an opaque host target whose
//! unused byte must be zero (local softbuffer storage adapter, not a Skia enum).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PixelFormat {
    #[default]
    Rgba8888,
    Bgra8888,
    Bgrx8888,
}
impl PixelFormat {
    #[inline(always)]
    pub(crate) fn channel(self, canonical: usize) -> usize {
        if self != Self::Rgba8888 && canonical < 3 {
            2 - canonical
        } else {
            canonical
        }
    }
    #[inline(always)]
    pub(crate) fn swizzle(self, rgba: [u8; 4]) -> [u8; 4] {
        if self != Self::Rgba8888 {
            [rgba[2], rgba[1], rgba[0], rgba[3]]
        } else {
            rgba
        }
    }
    #[inline(always)]
    pub(crate) fn encode(self, rgba: [u8; 4]) -> [u8; 4] {
        let mut bytes = self.swizzle(rgba);
        if self == Self::Bgrx8888 {
            bytes[3] = 0;
        }
        bytes
    }
    #[inline(always)]
    pub(crate) fn decode(self, bytes: [u8; 4]) -> [u8; 4] {
        let mut rgba = self.swizzle(bytes);
        if self == Self::Bgrx8888 {
            rgba[3] = 255;
        }
        rgba
    }
}
