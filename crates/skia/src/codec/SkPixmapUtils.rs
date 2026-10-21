use crate::include::codec::SkEncodedOrigin::*;
use crate::src::core::{SkImageInfo::SkImageInfo, SkPixmap::SkPixmap};
// Source: src/codec/SkPixmapUtils.cpp:24-40
unsafe fn draw_orientation(
    dst: &SkPixmap,
    src: &SkPixmap,
    origin: SkEncodedOrigin,
) -> Result<bool, crate::RasterError> {
    let surf = unsafe {
        crate::src::image::SkSurface_Raster::WrapPixels(
            dst.info(),
            dst.writable_addr().cast(),
            dst.rowBytes(),
            None,
        )
    };
    let Some(surf) = surf else {
        return Ok(false);
    };
    let mut bm = crate::src::core::SkBitmap::SkBitmap::default();
    bm.installPixmap(src);
    let m = SkEncodedOriginToMatrix(origin, dst.width(), dst.height());
    let mut p = crate::src::core::SkPaint::SkPaint::default();
    p.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
    let canvas = unsafe {
        crate::src::image::SkSurface::getCanvas_raw(
            surf.as_ptr().expect("native nonnull wrapped surface"),
        )
    }
    .expect("native raster surface canvas");
    unsafe {
        core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).concat(&m);
    }
    let image = unsafe { crate::src::image::SkImage_RasterFactories::RasterFromBitmap(&bm) };
    unsafe {
        crate::src::core::SkCanvas::SkCanvas::drawImage_raw(
            canvas,
            image.as_ref(),
            0.0,
            0.0,
            &crate::include::core::SkSamplingOptions::SkSamplingOptions::default(),
            Some(&p),
        )
    }?;
    Ok(true)
}
// Source: include/codec/SkPixmapUtils.h:17-22
// Source: src/codec/SkPixmapUtils.cpp:44-69
/// Safety: descriptors cover initialized source pixels and exclusive writable
/// destination pixels throughout the real surface/Canvas operation. Distinct
/// addresses must not overlap. The local wrapping Surface cannot escape.
pub unsafe fn Orient(
    dst: &SkPixmap,
    src: &SkPixmap,
    origin: SkEncodedOrigin,
) -> Result<bool, crate::RasterError> {
    if src.colorType() != dst.colorType() {
        return Ok(false);
    }
    let mut w = src.width();
    let mut h = src.height();
    if SkEncodedOriginSwapsWidthHeight(origin) {
        core::mem::swap(&mut w, &mut h);
    }
    if dst.width() != w || dst.height() != h {
        return Ok(false);
    }
    if w == 0 || h == 0 {
        return Ok(true);
    }
    if src.addr() == dst.addr() {
        return Ok(kTopLeft_SkEncodedOrigin == origin);
    }
    unsafe { draw_orientation(dst, src, origin) }
}
// Source: include/codec/SkPixmapUtils.h:24-27
// Source: src/codec/SkPixmapUtils.cpp:71-73
pub fn SwapWidthHeight(info: &SkImageInfo) -> SkImageInfo {
    info.makeWH(info.height(), info.width())
}
