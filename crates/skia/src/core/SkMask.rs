//! Restricted two-pixel A8 mask view used by SkBlitter anti-hairline calls.
//! Source: src/core/SkMask.h. The fixed array is this Rust subset
//! representation; official SkMask accepts an arbitrary borrowed image.
use crate::raster::geom::ScreenIntRect;
/// SkMask is used to describe alpha bitmaps.
pub struct SkMask {
    pub image: [u8; 2],
    pub bounds: ScreenIntRect,
    pub row_bytes: u32,
}
