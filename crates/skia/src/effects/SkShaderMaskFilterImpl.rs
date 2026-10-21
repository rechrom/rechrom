// Copyright 2018 Google LLC. BSD-style license. Direct translation, unverified.
use crate::src::{
    core::{
        SkFlattenable::{self as flat, FactoryResult, FlattenableOwner, SkFlattenableView},
        SkMask::{AllocType, Format, SkMask, SkMaskBuilder},
        SkMaskFilterBase::{BlurRec, SkMaskFilterBase, Type},
        SkMatrix::SkMatrix,
        SkPaint::SkPaint,
        SkPoint::SkIPoint,
        SkRect::SkRect,
    },
    shaders::SkShader::SkShader,
};
use std::sync::Arc;
// Source: src/effects/SkShaderMaskFilterImpl.h:27-29,55-55
pub struct SkShaderMaskFilterImpl {
    fShader: Arc<dyn SkShader>,
}
impl SkShaderMaskFilterImpl {
    pub fn new(shader: Arc<dyn SkShader>) -> Self {
        Self { fShader: shader }
    }
    // Source: src/effects/SkShaderMaskFilterImpl.h:48-48
    pub fn shader(&self) -> Arc<dyn SkShader> {
        self.fShader.clone()
    }
    // Source: src/effects/SkShaderMaskFilterImpl.cpp:35-37
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> FactoryResult {
        Ok(Make(buffer.readShader()?).map(FlattenableOwner::MaskFilter))
    }
}
impl SkFlattenableView for SkShaderMaskFilterImpl {
    fn getFactory(&self) -> Option<flat::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkShaderMaskFilterImpl")
    }
    fn getFlattenableType(&self) -> flat::Type {
        flat::Type::kSkMaskFilter_Type
    }
    // Source: src/effects/SkShaderMaskFilterImpl.cpp:39-41
    fn flatten(
        &self,
        w: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        w.writeFlattenable(Some(&*self.fShader))?;
        Ok(())
    }
}
// Source: src/effects/SkShaderMaskFilterImpl.cpp:43-50
unsafe fn rect_memcpy(
    mut dst: *mut u8,
    dstRB: usize,
    mut src: *const u8,
    srcRB: usize,
    copyBytes: usize,
    rows: i32,
) {
    for _ in 0..rows {
        core::ptr::copy_nonoverlapping(src, dst, copyBytes);
        dst = dst.add(dstRB);
        src = src.add(srcRB);
    }
}
impl SkMaskFilterBase for SkShaderMaskFilterImpl {
    // Source: src/effects/SkShaderMaskFilterImpl.h:31-32,43-48
    fn getFormat(&self) -> Format {
        Format::kA8_Format
    }
    fn r#type(&self) -> Type {
        Type::kShader
    }
    fn computeFastBounds(&self, src: SkRect, dst: &mut SkRect) -> Result<(), crate::RasterError> {
        *dst = src;
        Ok(())
    }
    fn asABlur(&self, _rec: Option<&mut BlurRec>) -> bool {
        false
    }
    // Source: src/effects/SkShaderMaskFilterImpl.cpp:52-105
    // The shared mask virtual propagates the actual Canvas RasterError boundary.
    unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        ctm: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        if src.fFormat != Format::kA8_Format {
            return Ok(false);
        }
        if let Some(m) = margin {
            m.set(0, 0);
        }
        *dst.bounds() = src.fBounds;
        *dst.rowBytes() = src.fBounds.width() as u32;
        *dst.format() = Format::kA8_Format;
        if src.fImage.is_null() {
            *dst.image() = core::ptr::null_mut();
            return Ok(true);
        }
        let size = dst.computeImageSize();
        if size == 0 {
            return Ok(false);
        }
        *dst.image() = SkMaskBuilder::AllocImage(size, AllocType::kUninit_Alloc);
        rect_memcpy(
            dst.fImage.cast_mut(),
            dst.fRowBytes as usize,
            src.fImage,
            src.fRowBytes as usize,
            src.fBounds.width() as usize,
            src.fBounds.height(),
        );
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        let info = crate::src::core::SkImageInfo::SkImageInfo::MakeA8(
            dst.fBounds.width(),
            dst.fBounds.height(),
        );
        if !bitmap.installPixels_no_release(
            &info,
            dst.fImage.cast_mut().cast(),
            dst.fRowBytes as usize,
        ) {
            return Ok(false);
        }
        let mut paint = SkPaint::default();
        paint.setShader(Some(self.fShader.clone()));
        paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrcIn);
        let mut canvas = crate::src::core::SkCanvas::SkCanvas::with_bitmap(&bitmap);
        canvas
            .as_mut()
            .translate(-(dst.fBounds.fLeft as f32), -(dst.fBounds.fTop as f32));
        canvas.as_mut().concat(ctm);
        let result = crate::src::core::SkCanvas::SkCanvas::drawPaint_raw(
            core::ptr::NonNull::from(canvas.as_mut().get_unchecked_mut()),
            &paint,
        );
        drop(canvas);
        if let Err(error) = result {
            SkMaskBuilder::FreeImage(dst.fImage.cast_mut().cast());
            *dst.image() = core::ptr::null_mut();
            return Err(error);
        }
        Ok(true)
    }
    // Source: src/effects/SkShaderMaskFilterImpl.cpp:107-112
    fn asImageFilter(
        &self,
        _ctm: &SkMatrix,
        _paint: &SkPaint,
    ) -> (
        Option<Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        bool,
    ) {
        let filter = crate::src::effects::imagefilters::SkShaderImageFilter::Shader(
            Some(self.fShader.clone()),
            false,
            None,
        );
        (
            crate::src::effects::imagefilters::SkBlendImageFilter::Blend(
                crate::src::core::SkBlendMode::SkBlendMode::kDstIn,
                filter,
                None,
                None,
            ),
            false,
        )
    }
}
// Source: src/effects/SkShaderMaskFilterImpl.cpp:114-116
pub fn Make(shader: Option<Arc<dyn SkShader>>) -> Option<Arc<dyn SkMaskFilterBase>> {
    shader.map(|shader| Arc::new(SkShaderMaskFilterImpl::new(shader)) as Arc<dyn SkMaskFilterBase>)
}
// Source: src/effects/SkShaderMaskFilterImpl.cpp:118-122
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        flat::Register(
            b"SkShaderMaskFilterImpl",
            SkShaderMaskFilterImpl::CreateProc,
        );
        flat::Register(b"SkShaderMF", SkShaderMaskFilterImpl::CreateProc);
    }
}
