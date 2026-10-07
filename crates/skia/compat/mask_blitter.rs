//! Drawing-color adapters around Skia pixel blitters.

// Paint-color adapter for the owned Skia CPU blitters.
use crate::compat::commands::Color;
use crate::include::core::SkColor::SkColor4f;
pub(crate) use crate::src::core::SkBlitRow_D32::blend_span;
pub(crate) use crate::src::core::SkColor::mul_div_255_round;
pub(crate) use crate::src::opts::SkBlitMask_opts::blend_premultiplied;
fn color(c: Color) -> SkColor4f {
    SkColor4f::new(c.red, c.green, c.blue, c.alpha)
}
pub(crate) fn premultiply(c: Color) -> [u8; 4] {
    crate::src::core::SkColor::premultiply(color(c))
}
pub(crate) fn blend_mask(dst: &mut [u8], c: Color, coverage: u8) {
    if coverage != 0 {
        blend_premultiplied(dst, premultiply(c), coverage);
    }
}
pub(crate) fn blend_anti_h2(dst: &mut [u8], c: Color, coverage: u8) {
    if coverage != 0 {
        crate::src::core::SkBlitter_ARGB32::blend_anti_h2_prepared(dst, premultiply(c), coverage);
    }
}

pub(crate) fn blend_mask_format(
    dst: &mut [u8],
    c: Color,
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    blend_prepared_format(dst, premultiply(c), coverage, false, format);
}
pub(crate) fn blend_anti_h2_format(
    dst: &mut [u8],
    c: Color,
    coverage: u8,
    format: crate::raster::PixelFormat,
) {
    blend_prepared_format(dst, premultiply(c), coverage, true, format);
}
pub(crate) fn blend_prepared_format(
    dst: &mut [u8],
    source: [u8; 4],
    coverage: u8,
    pair: bool,
    format: crate::raster::PixelFormat,
) {
    crate::src::core::SkBlitRow_D32::blend_span_format(dst, source, coverage, pair, format);
}
