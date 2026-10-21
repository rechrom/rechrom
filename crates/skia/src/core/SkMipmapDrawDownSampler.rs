// Copyright 2023 Google LLC. BSD-style license.
use super::{
    SkBitmap::SkBitmap, SkBlendMode::SkBlendMode, SkDraw::Draw, SkMatrix::SkMatrix,
    SkPaint::SkPaint, SkPixmap::SkPixmap, SkRasterClip::SkRasterClip,
};
use crate::include::core::SkSamplingOptions::{
    SkCubicResampler, SkFilterMode, SkMipmapMode, SkSamplingOptions,
};

// Source: src/core/SkMipmapDrawDownSampler.cpp:26-26
// The selected native drawing implementation retains its actual Paint owner.
pub struct DrawDownSampler {
    // Source: src/core/SkMipmapDrawDownSampler.cpp:27-27
    fPaint: SkPaint,
}

impl DrawDownSampler {
    // Source: src/core/SkMipmapDrawDownSampler.cpp:29-33
    pub fn new() -> Self {
        let mut paint = SkPaint::default();
        paint.setBlendMode(SkBlendMode::kSrc);
        Self { fPaint: paint }
    }
}

// Source: src/core/SkMipmapDrawDownSampler.cpp:38-46
fn choose_options(dst: &SkPixmap, src: &SkPixmap) -> SkSamplingOptions {
    if dst.width() * 2 == src.width() && dst.height() * 2 == src.height() {
        return SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kNone);
    }
    let cubic = SkCubicResampler::CatmullRom();
    SkSamplingOptions::with_cubic(cubic)
}

impl super::SkMipmap::SkMipmapDownSampler for DrawDownSampler {
    // Source: src/core/SkMipmapDrawDownSampler.cpp:35-35
    // Source: src/core/SkMipmapDrawDownSampler.cpp:48-63
    /// Safety: src retains its original initialized readable backing, and dst
    /// retains exclusive writable backing of the declared dimensions/rowBytes.
    /// Both leases remain valid through drawing and all temporary shader uses.
    /// Native draw errors are preserved as Result rather than ignored.
    unsafe fn buildLevel(&self, dst: &SkPixmap, src: &SkPixmap) -> Result<(), crate::RasterError> {
        let clip = SkRasterClip::from_rect(dst.bounds());
        let matrix = SkMatrix::Scale(
            dst.width() as f32 / src.width() as f32,
            dst.height() as f32 / src.height() as f32,
        );
        let sampling = choose_options(dst, src);
        let mut draw = Draw::new();
        draw.fDst = dst.clone();
        draw.fCTM = Some(&matrix);
        draw.fRC = Some(&clip);

        let mut bitmap = SkBitmap::default();
        unsafe {
            bitmap.installPixels(
                src.info(),
                src.addr().cast_mut().cast(),
                src.rowBytes(),
                None,
                core::ptr::null_mut(),
            );
            draw.drawBitmap(
                &bitmap,
                &SkMatrix::default(),
                None,
                &sampling,
                &self.fPaint,
                None,
            )
        }
    }
}

// Source: src/core/SkMipmapDrawDownSampler.cpp:67-69
pub fn MakeDownSampler(_root: &SkPixmap) -> Option<Box<dyn super::SkMipmap::SkMipmapDownSampler>> {
    Some(Box::new(DrawDownSampler::new()))
}
