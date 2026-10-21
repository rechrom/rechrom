#![allow(non_snake_case)]
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkRect::SkIRect,
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkMergeImageFilter.cpp:24-29
// Native nonempty input DAG, Merge factory, bounds and real Builder merge.
pub(crate) struct SkMergeImageFilter {
    base: SkImageFilter_Base,
}
impl SkMergeImageFilter {
    pub(crate) fn new(inputs: &[Option<Arc<dyn SkImageFilter>>]) -> Self {
        let value = Self {
            base: SkImageFilter_Base::new(inputs, None),
        };
        crate::SkASSERT!(@source b"src/effects/imagefilters/SkMergeImageFilter.cpp\0".as_ptr(),28; @condition b"filters && count > 0\0".as_ptr(); !inputs.is_empty());
        value
    }
}
// Source: src/effects/imagefilters/SkMergeImageFilter.cpp:55-66
// Option/slice preserves native nullable array and count without accepting out-of-bounds counts.
pub fn Merge(
    filters: Option<&[Option<Arc<dyn SkImageFilter>>]>,
    crop: Option<crate::src::core::SkRect::SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let Some(filters) = filters else {
        return super::SkCropImageFilter::Empty();
    };
    if filters.is_empty() {
        return super::SkCropImageFilter::Empty();
    }
    let filter: Arc<dyn SkImageFilter> = Arc::new(SkMergeImageFilter::new(filters));
    if let Some(crop) = crop {
        super::SkCropImageFilter::Crop(
            crop,
            crate::include::core::SkTileMode::SkTileMode::kDecal,
            Some(filter),
        )
    } else {
        Some(filter)
    }
}
// Source: src/effects/imagefilters/SkMergeImageFilter.cpp:37-37
impl crate::src::core::SkFlattenable::SkFlattenableView for SkMergeImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMergeImageFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkImageFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(writer)?;
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkMergeImageFilter.cpp:68-72
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMergeImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkMergeImageFilter",
            SkMergeImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkMergeImageFilterImpl",
            SkMergeImageFilter::CreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkMergeImageFilter.cpp:74-80
impl SkMergeImageFilter {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, -1)? || !buffer.isValid() {
            return Ok(None);
        }
        // Read the independent optional rectangle before borrowing the native
        // input array mutably; the same retained input handles reach Merge.
        let crop = common.cropRect();
        Ok(Merge(Some(common.inputs()), crop)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
}
impl SkImageFilterBaseHooks for SkMergeImageFilter {
    // Source: src/effects/imagefilters/SkMergeImageFilter.cpp:39-39
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkMergeImageFilter.cpp:84-91
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let input_count = self.countInputs();
        let mut builder = crate::src::core::SkImageFilterTypes::Builder::new(context);
        for i in 0..input_count {
            builder.add(self.getChildOutput(i, context)?);
        }
        builder.merge()
    }

    // Source: src/effects/imagefilters/SkMergeImageFilter.cpp:93-104
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            LayerSpace::<SkIRect>::TryUnion(self.countInputs() as i32, |i| {
                self.getChildInputLayerBounds(i as usize, mapping, desired, content)
            })?
        })
    }
    // Source: src/effects/imagefilters/SkMergeImageFilter.cpp:106-129
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            let mut unbounded = false;
            let child = LayerSpace::<SkIRect>::TryUnion(self.countInputs() as i32, |i| {
                Ok(
                    match self.getChildOutputLayerBounds(i as usize, mapping, content)? {
                        Some(out) => out,
                        None => {
                            unbounded = true;
                            LayerSpace::<SkIRect>::Empty()
                        }
                    },
                )
            })?;
            if unbounded {
                LayerSpace::<SkIRect>::Unbounded()
            } else {
                Some(child)
            }
        })
    }
}
impl SkImageFilter for SkMergeImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkMergeImageFilter.cpp:131-138
    fn computeFastBounds(
        &self,
        rect: &crate::src::core::SkRect::SkRect,
    ) -> Result<crate::src::core::SkRect::SkRect, crate::RasterError> {
        crate::src::core::SkImageFilter::compute_fast_bounds_base(self, rect)
    }
}
