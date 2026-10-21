#![allow(non_snake_case)]
use crate::include::core::SkSamplingOptions::SkSamplingOptions;
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability, ParameterSpace},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkMatrix::{ScaleToFit, SkMatrix},
    SkRect::{SkIRect, SkRect},
};
use crate::src::image::SkImage::SkImage;
use std::sync::Arc;
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:67-73
struct SkImageImageFilter {
    fSampling: SkSamplingOptions,
    fDstRect: ParameterSpace<SkRect>,
    fSrcRect: SkRect,
    fImage: Arc<dyn SkImage>,
    base: SkImageFilter_Base,
}
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:32-43
impl SkImageImageFilter {
    fn new(image: Arc<dyn SkImage>, src: SkRect, dst: SkRect, sampling: SkSamplingOptions) -> Self {
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkImageImageFilter.cpp\0".as_ptr(),42; @condition b"fImage && !dstRect.isEmpty()\0".as_ptr(); !dst.isEmpty());
        Self {
            base: SkImageFilter_Base::new(&[], None),
            fImage: image,
            fSrcRect: src,
            fDstRect: ParameterSpace::new(dst),
            fSampling: sampling,
        }
    }
}
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:78-107
pub fn Image(
    image: Option<Arc<dyn SkImage>>,
    src: SkRect,
    dst: SkRect,
    sampling: SkSamplingOptions,
) -> Option<Arc<dyn SkImageFilter>> {
    if src.isEmpty() || dst.isEmpty() || image.is_none() {
        return super::SkCropImageFilter::Empty();
    }
    let image = image.unwrap();
    let size = image.dimensions();
    let mut bounds = SkRect::MakeWH(size.fWidth as f32, size.fHeight as f32);
    if bounds.contains(&src) {
        return Some(Arc::new(SkImageImageFilter::new(image, src, dst, sampling)));
    }
    let matrix = SkMatrix::RectToRectOrIdentity(&src, &dst, ScaleToFit::kFill_ScaleToFit);
    if !bounds.intersect(&src) {
        return super::SkCropImageFilter::Empty();
    }
    let mut mapped = SkRect::MakeEmpty();
    matrix.mapRect(&mut mapped, &bounds);
    if mapped.isEmpty() {
        return super::SkCropImageFilter::Empty();
    }
    Some(Arc::new(SkImageImageFilter::new(
        image, bounds, mapped, sampling,
    )))
}
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:52-52
impl crate::src::core::SkFlattenable::SkFlattenableView for SkImageImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkImageImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:108-112
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterImageImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkImageImageFilter",
            SkImageImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkImageSourceImpl",
            SkImageImageFilter::CreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkImageImageFilter.cpp:114-132
impl SkImageImageFilter {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let sampling = if buffer
            .isVersionLT(crate::src::core::SkPicturePriv::kImageFilterImageSampling_Version)
        {
            crate::src::core::SkSamplingPriv::SkSamplingPriv::FromFQ(
                buffer.checkFilterQuality(),
                crate::src::core::SkSamplingPriv::SkMediumAs::kLinear_SkMediumAs,
            )
        } else {
            buffer.readSampling()
        };
        let src = buffer.readRect();
        let dst = buffer.readRect();
        let Some(image) = buffer.readImage()? else {
            return Ok(None);
        };
        Ok(Image(Some(image), src, dst, sampling)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:134-139
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:48
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeSampling(&self.fSampling);
        buffer.writeRect(&self.fSrcRect);
        buffer.writeRect(self.fDstRect.geometry()); // The retained image is the original immutable native image owner;
                                                    // its producer's backing lease also covers synchronous serialization.
        unsafe {
            buffer.writeImage(&*self.fImage);
        }
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkImageImageFilter {
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:54-54
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:143-145
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        crate::src::core::SkImageFilterTypes::FilterResult::MakeFromImage(
            context,
            self.fImage.clone(),
            self.fSrcRect,
            self.fDstRect,
            self.fSampling,
        )
    }

    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:147-153
    fn onGetInputLayerBounds(
        &self,
        _: &Mapping,
        _: &LayerSpace<SkIRect>,
        _: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({ LayerSpace::<SkIRect>::Empty() })
    }
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:155-160
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        _: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({ Some(mapping.paramToLayer(&self.fDstRect).roundOut()) })
    }
}
impl SkImageFilter for SkImageImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkImageImageFilter.cpp:45-45
    fn computeFastBounds(&self, _: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({ *self.fDstRect.geometry() })
    }
}
