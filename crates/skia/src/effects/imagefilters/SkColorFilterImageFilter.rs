use crate::src::core::SkColorFilter::SkColorFilter;
use crate::src::core::SkImageFilter::SkImageFilter;
use crate::src::core::SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability};
use crate::src::core::SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base};
use crate::src::core::SkRect::{SkIRect, SkRect};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:27-31
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:67-68
// Derived color-filter ownership drops before native base input ownership.
pub struct SkColorFilterImageFilter {
    fColorFilter: Arc<dyn SkColorFilter>,
    base: SkImageFilter_Base,
}
impl SkColorFilterImageFilter {
    fn new(cf: Arc<dyn SkColorFilter>, input: Option<Arc<dyn SkImageFilter>>) -> Self {
        Self {
            base: SkImageFilter_Base::new(&[input], None),
            fColorFilter: cf,
        }
    }
}
impl SkImageFilterBaseHooks for SkColorFilterImageFilter {
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:53-53
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:55-57
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok(self.fColorFilter.affectsTransparentBlack()?)
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:59-65
    fn onIsColorFilterNode(&self, filter: &mut Option<Arc<dyn SkColorFilter>>) -> bool {
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkColorFilterImageFilter.cpp\0".as_ptr(),60; @condition b"1 == this->countInputs()\0".as_ptr(); self.countInputs()==1);
        *filter = Some(self.fColorFilter.clone());
        true
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:117-119
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        self.getChildOutput(0, context)?
            .applyColorFilter(context, Some(self.fColorFilter.clone()))
    }

    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:121-126
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desiredOutput: &LayerSpace<SkIRect>,
        contentBounds: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        self.getChildInputLayerBounds(0, mapping, desiredOutput, contentBounds)
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:128-139
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        contentBounds: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        if self.fColorFilter.affectsTransparentBlack()? {
            Ok(LayerSpace::<SkIRect>::Unbounded())
        } else {
            self.getChildOutputLayerBounds(0, mapping, contentBounds)
        }
    }
}
impl SkImageFilter for SkColorFilterImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:141-150
    fn computeFastBounds(&self, bounds: &SkRect) -> Result<SkRect, crate::RasterError> {
        if self.fColorFilter.affectsTransparentBlack()? {
            Ok(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32())
        } else if let Some(input) = self.getInput(0) {
            input.computeFastBounds(bounds)
        } else {
            Ok(*bounds)
        }
    }
}
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:72-96
pub fn ColorFilter(
    mut cf: Option<Arc<dyn SkColorFilter>>,
    mut input: Option<Arc<dyn SkImageFilter>>,
    cropRect: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    if let Some(color) = cf.as_ref() {
        if let Some(child) = input.as_ref() {
            let mut inputCF = None;
            if child.isColorFilterNode(&mut inputCF) {
                cf = Some(color.makeComposed(inputCF));
                input = child.getInput(0).cloned();
            }
        }
    }
    let mut filter = input;
    if let Some(cf) = cf {
        filter = Some(Arc::new(SkColorFilterImageFilter::new(cf, filter)));
    }
    if let Some(crop) = cropRect {
        filter = super::SkCropImageFilter::Crop(
            crop,
            crate::include::core::SkTileMode::SkTileMode::kDecal,
            filter,
        );
    }
    filter
}
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:40-40
impl crate::src::core::SkFlattenable::SkFlattenableView for SkColorFilterImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkColorFilterImageFilter")
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
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:98-102
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterColorFilterImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkColorFilterImageFilter",
            SkColorFilterImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkColorFilterImageFilterImpl",
            SkColorFilterImageFilter::CreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:104-108
impl SkColorFilterImageFilter {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let cf = buffer.readColorFilter()?;
        Ok(ColorFilter(cf, common.getInput(0), common.cropRect())
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkColorFilterImageFilter.cpp:110-113
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeFlattenable(Some(&*self.fColorFilter))?;
        Ok(())
    }
}
