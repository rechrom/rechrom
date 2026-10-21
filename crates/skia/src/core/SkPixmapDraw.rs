use super::SkPixmap::SkPixmap;

// Source: src/core/SkPixmapDraw.cpp:29-87
impl SkPixmap {
    /// Safety: source pixels are initialized/readable for the complete shader
    /// draw and destination covers its exclusive writable native layout. Scaling
    /// source/destination allocations and all other aliases must not conflict.
    /// Raw wrapped pixels remain live until the temporary Surface/Canvas drops.
    pub unsafe fn scalePixels(
        &self,
        actual_dst: &Self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<bool, crate::RasterError> {
        use crate::include::core::SkAlphaType::SkAlphaType;
        use crate::src::core::{
            SkBitmap::SkBitmap, SkMatrix::SkMatrix, SkPaint::SkPaint, SkRect::SkRect,
        };
        let mut src = self.clone();
        let mut dst = actual_dst.clone();
        if src.width() <= 0 || src.height() <= 0 || dst.width() <= 0 || dst.height() <= 0 {
            return Ok(false);
        }
        if src.width() == dst.width() && src.height() == dst.height() {
            return unsafe {
                src.readPixels_raw(dst.info(), dst.writable_addr(), dst.rowBytes(), 0, 0)
            };
        }
        let mut clamp_as_if_unpremul = false;
        if src.alphaType() == SkAlphaType::kUnpremul_SkAlphaType
            && dst.alphaType() == SkAlphaType::kUnpremul_SkAlphaType
        {
            src.reset_with(
                &src.info().makeAlphaType(SkAlphaType::kPremul_SkAlphaType),
                src.addr(),
                src.rowBytes(),
            );
            dst.reset_with(
                &dst.info().makeAlphaType(SkAlphaType::kOpaque_SkAlphaType),
                dst.addr(),
                dst.rowBytes(),
            );
            clamp_as_if_unpremul = true;
        }
        let mut bitmap = SkBitmap::default();
        if !bitmap.installPixmap(&src) {
            return Ok(false);
        }
        bitmap.setImmutable();
        // Native SkMatrix.h1120-1123 defaults the omitted argument to kFill.
        let scale = SkMatrix::RectToRectOrIdentity(
            &SkRect::Make(&src.bounds()),
            &SkRect::Make(&dst.bounds()),
            crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
        );
        let shader = crate::src::shaders::SkImageShader::SkImageShader::Make(
            unsafe { bitmap.asImage() },
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            Some(&scale),
            clamp_as_if_unpremul,
        );
        let surface = unsafe {
            crate::src::image::SkSurface_Raster::WrapPixels(
                dst.info(),
                dst.writable_addr().cast(),
                dst.rowBytes(),
                None,
            )
        };
        if shader.is_none() || surface.is_none() {
            return Ok(false);
        }
        let surface = surface.expect("native successful WrapPixels branch");
        let mut paint = SkPaint::default();
        paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
        paint.setShader(shader);
        let canvas = unsafe {
            crate::src::image::SkSurface::getCanvas_raw(
                surface.as_ptr().expect("native live Surface owner"),
            )
        }
        .expect("native complete Surface has Canvas");
        unsafe { crate::src::core::SkCanvas::SkCanvas::drawPaint_raw(canvas, &paint) }?;
        Ok(true)
    }
}
