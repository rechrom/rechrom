#![allow(non_snake_case)]
use crate::include::core::SkTileMode::SkTileMode;
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, ParameterSpace},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:31-39
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:93-95
// Native crop/source DAG, validated factory and real tiled FilterResult dispatch.
pub(crate) struct SkCropImageFilter {
    base: SkImageFilter_Base,
    fCropRect: ParameterSpace<SkRect>,
    fTileMode: SkTileMode,
}
impl SkCropImageFilter {
    pub(crate) fn new(
        rect: SkRect,
        tile: SkTileMode,
        input: Option<Arc<dyn SkImageFilter>>,
    ) -> Self {
        let value = Self {
            base: SkImageFilter_Base::new(&[input], None),
            fCropRect: ParameterSpace::new(rect),
            fTileMode: tile,
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkCropImageFilter.cpp\0".as_ptr(),37; @condition b"cropRect.isFinite()\0".as_ptr(); rect.isFinite());
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkCropImageFilter.cpp\0".as_ptr(),38; @condition b"cropRect.isSorted()\0".as_ptr(); rect.isSorted());
        value
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:74-84
    fn cropRect(&self, mapping: &Mapping) -> LayerSpace<SkIRect> {
        let crop = mapping.paramToLayer(&self.fCropRect);
        if self.fTileMode == SkTileMode::kDecal {
            crop.roundOut()
        } else {
            crop.roundIn()
        }
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:88-91
    fn requiredInput(&self, mapping: &Mapping, output: LayerSpace<SkIRect>) -> LayerSpace<SkIRect> {
        self.cropRect(mapping)
            .relevantSubset(output, self.fTileMode)
    }
}
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:99-106
pub fn Crop(
    rect: SkRect,
    tile: SkTileMode,
    input: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    if !crate::src::core::SkValidationUtils::SkIsValidRect(&rect) {
        return None;
    }
    Some(Arc::new(SkCropImageFilter::new(rect, tile, input)))
}
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:110-112
pub fn Empty() -> Option<Arc<dyn SkImageFilter>> {
    Crop(SkRect::MakeEmpty(), SkTileMode::kDecal, None)
}
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:114-122
pub fn Tile(
    src: SkRect,
    dst: SkRect,
    input: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    let filter = Crop(src, SkTileMode::kRepeat, input);
    Crop(dst, SkTileMode::kDecal, filter)
}
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:48-48
impl crate::src::core::SkFlattenable::SkFlattenableView for SkCropImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkCropImageFilter")
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
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:124-129
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterCropImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkCropImageFilter",
            SkCropImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkTileImageFilter",
            SkCropImageFilter::LegacyTileCreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkTileImageFilterImpl",
            SkCropImageFilter::LegacyTileCreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkCropImageFilter.cpp:131-137
impl SkCropImageFilter {
    pub fn LegacyTileCreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let src = buffer.readRect();
        let dst = buffer.readRect();
        Ok(Tile(src, dst, common.getInput(0))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:139-152
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let rect = buffer.readRect();
        if !buffer.isValid()
            || !buffer.validate(crate::src::core::SkValidationUtils::SkIsValidRect(&rect))
        {
            return Ok(None);
        }
        let mut tile = SkTileMode::kDecal;
        if !buffer.isVersionLT(crate::src::core::SkPicturePriv::kCropImageFilterSupportsTiling) {
            let value = buffer.read32LE(SkTileMode::kDecal as u32);
            tile = match value {
                0 => SkTileMode::kClamp,
                1 => SkTileMode::kRepeat,
                2 => SkTileMode::kMirror,
                _ => SkTileMode::kDecal,
            };
        }
        Ok(Crop(rect, tile, common.getInput(0))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:154-158
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeRect(self.fCropRect.geometry());
        buffer.writeInt(self.fTileMode as i32);
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkCropImageFilter {
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:51-51
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok({ self.fTileMode != SkTileMode::kDecal })
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:55-55
    fn ignoreInputsAffectsTransparentBlack(&self) -> bool {
        true
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:162-171
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let crop_input = self.requiredInput(context.mapping(), *context.desiredOutput());
        let child_output = self.getChildOutput(0, &context.withNewDesiredOutput(crop_input))?;
        child_output.applyCrop(context, self.cropRect(context.mapping()), self.fTileMode)
    }

    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:179-190
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let required = self.requiredInput(mapping, *desired);
            self.getChildInputLayerBounds(0, mapping, &required, content)?
        })
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:192-214
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            let child = self.getChildOutputLayerBounds(0, mapping, content)?;
            let mut crop = self.cropRect(mapping);
            if child.is_some_and(|bounds| !crop.intersect(&bounds)) {
                Some(LayerSpace::<SkIRect>::Empty())
            } else if self.fTileMode == SkTileMode::kDecal {
                Some(crop)
            } else {
                LayerSpace::<SkIRect>::Unbounded()
            }
        })
    }
}
impl SkImageFilter for SkCropImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkCropImageFilter.cpp:216-227
    fn computeFastBounds(&self, bounds: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            let mut input = match self.getInput(0) {
                Some(child) => child.computeFastBounds(bounds)?,
                None => *bounds,
            };
            if !input.intersect(self.fCropRect.geometry()) {
                return Ok(SkRect::MakeEmpty());
            }
            if self.fTileMode == SkTileMode::kDecal {
                input
            } else {
                crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32()
            }
        })
    }
}
