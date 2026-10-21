#![allow(non_snake_case)]
use crate::include::core::{SkSamplingOptions::SkSamplingOptions, SkSize::SkISize};
use crate::src::core::{
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability, ParameterSpace},
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkMatrix::SkMatrix,
    SkRect::{SkIRect, SkRect},
};
use std::sync::Arc;
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:32-43
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:71-73
// Real CPU matrix transform, bounds and FilterResult dispatch.
struct SkMatrixTransformImageFilter {
    fTransform: ParameterSpace<SkMatrix>,
    fSampling: SkSamplingOptions,
    base: SkImageFilter_Base,
}
impl SkMatrixTransformImageFilter {
    fn new(
        transform: &SkMatrix,
        sampling: SkSamplingOptions,
        input: Option<Arc<dyn SkImageFilter>>,
    ) -> Self {
        let value = Self {
            base: SkImageFilter_Base::new(&[input], None),
            fTransform: ParameterSpace::new(transform.clone()),
            fSampling: sampling,
        };
        value.fTransform.geometry().getType();
        value
    }
}
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:77-86
pub fn MatrixTransform(
    transform: &SkMatrix,
    sampling: SkSamplingOptions,
    input: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    if transform.invert().is_none() {
        return None;
    }
    Some(Arc::new(SkMatrixTransformImageFilter::new(
        transform, sampling, input,
    )))
}
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:88-102
pub fn Offset(
    dx: f32,
    dy: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let offset = MatrixTransform(
        &SkMatrix::Translate(dx, dy),
        SkSamplingOptions::with_filter(
            crate::include::core::SkSamplingOptions::SkFilterMode::kNearest,
        ),
        input,
    );
    if let Some(crop) = crop {
        super::SkCropImageFilter::Crop(
            crop,
            crate::include::core::SkTileMode::SkTileMode::kDecal,
            offset,
        )
    } else {
        offset
    }
}
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:52-52
impl crate::src::core::SkFlattenable::SkFlattenableView for SkMatrixTransformImageFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMatrixTransformImageFilter")
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
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:104-113
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMatrixTransformImageFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkMatrixTransformImageFilter",
            SkMatrixTransformImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkMatrixImageFilter",
            SkMatrixTransformImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkOffsetImageFilter",
            SkMatrixTransformImageFilter::LegacyOffsetCreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkOffsetImageFilterImpl",
            SkMatrixTransformImageFilter::LegacyOffsetCreateProc,
        );
    }
}
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:115-120
impl SkMatrixTransformImageFilter {
    pub fn LegacyOffsetCreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let p = buffer.readPoint();
        Ok(Offset(p.fX, p.fY, common.getInput(0), common.cropRect())
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:122-135
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let mut matrix = SkMatrix::default();
        buffer.readMatrix(&mut matrix);
        let sampling = if buffer
            .isVersionLT(crate::src::core::SkPicturePriv::kMatrixImageFilterSampling_Version)
        {
            let fq = buffer.read32LE(3);
            let fq = unsafe {
                std::mem::transmute::<u32, crate::src::core::SkSamplingPriv::SkLegacyFQ>(fq)
            };
            crate::src::core::SkSamplingPriv::SkSamplingPriv::FromFQ(
                fq,
                crate::src::core::SkSamplingPriv::SkMediumAs::kLinear_SkMediumAs,
            )
        } else {
            buffer.readSampling()
        };
        Ok(MatrixTransform(&matrix, sampling, common.getInput(0))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:137-141
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.base.flatten(buffer)?;
        buffer.writeMatrix(self.fTransform.geometry());
        buffer.writeSampling(&self.fSampling);
        Ok(())
    }
}
impl SkImageFilter for SkMatrixTransformImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:155-158
    fn computeFastBounds(&self, src: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok({
            let bounds = match self.getInput(0) {
                Some(child) => child.computeFastBounds(src)?,
                None => *src,
            };
            let mut out = SkRect::MakeEmpty();
            self.fTransform.geometry().mapRect(&mut out, &bounds);
            out
        })
    }
}
// Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:160-176
impl SkMatrixTransformImageFilter {
    fn requiredInput(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
    ) -> LayerSpace<SkIRect> {
        let mut required = LayerSpace::<SkIRect>::Empty();
        if !mapping
            .paramToLayer(&self.fTransform)
            .inverseMapIRect(desired, &mut required)
        {
            return LayerSpace::<SkIRect>::Empty();
        }
        if self.fSampling != SkSamplingOptions::default() {
            required.outset(LayerSpace::new(SkISize::Make(1, 1)));
        }
        required
    }
}
impl SkImageFilterBaseHooks for SkMatrixTransformImageFilter {
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:55-55
    fn onGetCTMCapability(&self) -> MatrixCapability {
        MatrixCapability::kComplex
    }
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:145-153
    // Adapted: the native synchronous CPU evaluation retains real child/image owners;
    // Result propagates failures from bounds, backend and raster operations.
    unsafe fn onFilterImage(
        &self,
        context: &crate::src::core::SkImageFilterTypes::Context<'_>,
    ) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
        let required_input = self.requiredInput(context.mapping(), context.desiredOutput());
        let child_output = self.getChildOutput(0, &context.withNewDesiredOutput(required_input))?;
        let transform = context.mapping().paramToLayer(&self.fTransform);
        child_output.applyTransform(context, transform, self.fSampling)
    }

    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:179-186
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        Ok({
            let required = self.requiredInput(mapping, desired);
            self.getChildInputLayerBounds(0, mapping, &required, content)?
        })
    }
    // Source: src/effects/imagefilters/SkMatrixTransformImageFilter.cpp:188-198
    fn onGetOutputLayerBounds(
        &self,
        mapping: &Mapping,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok({
            let child = self.getChildOutputLayerBounds(0, mapping, content)?;
            child.map(|out| mapping.paramToLayer(&self.fTransform).mapIRect(&out))
        })
    }
}
