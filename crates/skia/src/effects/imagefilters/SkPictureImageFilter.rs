// Copyright 2013 The Android Open Source Project. BSD-style license. UNVERIFIED.
use crate::{
    include::core::SkRefCnt::sk_sp,
    src::core::{
        SkImageFilter::SkImageFilter,
        SkImageFilterTypes::{
            Context, FilterResult, LayerSpace, Mapping, MatrixCapability, ParameterSpace,
        },
        SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
        SkPicture::SkPicture,
        SkRect::{SkIRect, SkRect},
    },
};
use std::sync::Arc;
type PictureOwner = sk_sp<dyn SkPicture>;
type Filter = Option<Arc<dyn SkImageFilter>>;
// Source: src/effects/imagefilters/SkPictureImageFilter.cpp:28-28
// Source: src/effects/imagefilters/SkPictureImageFilter.cpp:60-61
// Original intrusive picture reference; no Arc/proxy picture owner is introduced.
struct SkPictureImageFilter {
    fPicture: PictureOwner,
    fCullRect: ParameterSpace<SkRect>,
    base: SkImageFilter_Base,
}
impl SkPictureImageFilter {
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:30-36
    fn new(picture: PictureOwner, cull: &SkRect) -> Self {
        let result = Self {
            base: SkImageFilter_Base::new(&[], None),
            fPicture: picture,
            fCullRect: ParameterSpace::new(*cull),
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkPictureImageFilter.cpp\0".as_ptr(),35; @condition b"fPicture && fPicture->cullRect().contains(cullRect)\0".as_ptr(); result.fPicture.is_some()&&result.fPicture.cullRect().contains(cull));
        result
    }
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:38-38
    fn header_computeFastBounds(&self) -> SkRect {
        *self.fCullRect.geometry()
    }
}
// Source: src/effects/imagefilters/SkPictureImageFilter.cpp:45-45
impl crate::src::core::SkFlattenable::SkFlattenableView for SkPictureImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkPictureImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(buffer)
    }
}
impl SkPictureImageFilter {
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:47-47
    fn header_onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
}
// Source: src/effects/imagefilters/SkPictureImageFilter.cpp:66-74
pub fn Picture(pic: PictureOwner, target: &SkRect) -> Filter {
    if pic.is_some() {
        let mut cull = pic.cullRect();
        if cull.intersect(target) {
            return Some(Arc::new(SkPictureImageFilter::new(pic, &cull)));
        }
    }
    super::SkCropImageFilter::Empty()
}
// Source: src/effects/imagefilters/SkPictureImageFilter.cpp:76-80
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterPictureImageFilterFlattenable() {
    unsafe {
        use crate::src::core::SkFlattenable::Register;
        Register(b"SkPictureImageFilter", SkPictureImageFilter::CreateProc);
        Register(
            b"SkPictureImageFilterImpl",
            SkPictureImageFilter::CreateProc,
        );
    }
}
impl SkPictureImageFilter {
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:82-91
    fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut picture = PictureOwner::default();
        if buffer.readBool() {
            picture = crate::src::core::SkPicturePriv::MakeFromBuffer(buffer)?.unwrap_or_default();
        }
        let cull = buffer.readRect();
        Ok(Picture(picture, &cull)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:93-99
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeBool(self.fPicture.is_some());
        if self.fPicture.is_some() {
            crate::src::core::SkPicturePriv::Flatten(self.fPicture.clone(), buffer)?;
        }
        buffer.writeRect(self.fCullRect.geometry());
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkPictureImageFilter {
    fn onGetCTMCapability(&self) -> MatrixCapability {
        self.header_onGetCTMCapability()
    }
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:103-105
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:49-49
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        FilterResult::MakeFromPicture(ctx, self.fPicture.clone(), self.fCullRect)
    }
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:107-113
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:51-54
    fn onGetInputLayerBounds(
        &self,
        _mapping: &Mapping,
        _desired: &LayerSpace<SkIRect>,
        _content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok(LayerSpace::<SkIRect>::Empty())
    }
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:115-120
    // Source: src/effects/imagefilters/SkPictureImageFilter.cpp:56-58
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        _content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        let bounds: LayerSpace<SkRect> = mapping.paramToLayer(&self.fCullRect);
        Ok(Some(bounds.roundOut()))
    }
}
impl SkImageFilter for SkPictureImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    fn computeFastBounds(&self, _src: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok(self.header_computeFastBounds())
    }
}
