// Copyright 2023 Google LLC. BSD-style license.
use super::SkColorFilterBase::Type;
use crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType;
use crate::src::core::{
    SkColorSpace::SkColorSpace, SkColorSpaceXformSteps::SkColorSpaceXformSteps,
};
use std::sync::Arc;

// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:20-20
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:43-45
// Rust declaration order is the C++ destruction order: steps, destination, source.
pub struct SkColorSpaceXformColorFilter {
    fSteps: SkColorSpaceXformSteps,
    fDst: Option<Arc<SkColorSpace>>,
    fSrc: Option<Arc<SkColorSpace>>,
}
impl SkColorSpaceXformColorFilter {
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:22-22
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:27-35
    pub fn new(src: Option<Arc<SkColorSpace>>, dst: Option<Arc<SkColorSpace>>) -> Self {
        let steps = SkColorSpaceXformSteps::new(
            src.as_deref(),
            kUnpremul_SkAlphaType,
            dst.as_deref(),
            kUnpremul_SkAlphaType,
        );
        Self {
            fSrc: src,
            fDst: dst,
            fSteps: steps,
        }
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:26-28
    pub fn r#type(&self) -> Type {
        Type::kColorSpaceXform
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:30-31
    pub fn src(&self) -> Option<Arc<SkColorSpace>> {
        self.fSrc.clone()
    }
    pub fn dst(&self) -> Option<Arc<SkColorSpace>> {
        self.fDst.clone()
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:36-36
    pub fn onIsAlphaUnchanged(&self) -> bool {
        true
    }
}

use super::SkColorFilterBase::SkColorFilterBase;
use crate::src::core::{
    SkColorFilter::SkColorFilter, SkEffectPriv::SkStageRec,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
impl SkColorFilterBase for SkColorSpaceXformColorFilter {
    fn r#type(&self) -> Type {
        SkColorSpaceXformColorFilter::r#type(self)
    }
    fn onIsAlphaUnchanged(&self) -> bool {
        true
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:24-24
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:37-48
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            if !shaderIsOpaque {
                (*rec.fPipeline).append(Op::unpremul, std::ptr::null_mut());
            }
            self.fSteps.applyPipeline(&mut *rec.fPipeline);
            if !shaderIsOpaque {
                (*rec.fPipeline).append(Op::premul, std::ptr::null_mut());
            }
        }
        Ok(true)
    }
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.h:40-40
impl crate::src::core::SkFlattenable::SkFlattenableView for SkColorSpaceXformColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkColorSpaceXformColorFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkColorFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:50-53
impl SkColorSpaceXformColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        let src = self
            .fSrc
            .as_ref()
            .expect("native serialized source color space is nonnull")
            .serialize();
        buffer.writeDataAsByteArray(Some(&src));
        let dst = self
            .fDst
            .as_ref()
            .expect("native serialized destination color space is nonnull")
            .serialize();
        buffer.writeDataAsByteArray(Some(&dst));

        Ok(())
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:55-64
    pub fn LegacyGammaOnlyCreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let direction = buffer.read32() as u32;
        if !buffer.validate(direction <= 1) {
            return Ok(None);
        }
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter(if direction == 0 {
                LinearToSRGBGamma()
            } else {
                SRGBToLinearGamma()
            }),
        ))
    }
    // Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:66-80
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut spaces: [Option<Arc<SkColorSpace>>; 2] = [None, None];
        for space in &mut spaces {
            let data = buffer.readByteArrayAsData();
            if !buffer.validate(data.is_some()) {
                return Ok(None);
            }
            let data = data.unwrap();
            *space = SkColorSpace::Deserialize(unsafe {
                std::slice::from_raw_parts(data.data(), data.size())
            });
            if !buffer.validate(space.is_some()) {
                return Ok(None);
            }
        }
        let src = spaces[0].take();
        let dst = spaces[1].take();
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter(Arc::new(Self::new(
                src, dst,
            ))),
        ))
    }
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:82-86
pub fn LinearToSRGBGamma() -> Arc<dyn SkColorFilter> {
    static SINGLETON: std::sync::OnceLock<Arc<SkColorSpaceXformColorFilter>> =
        std::sync::OnceLock::new();
    SINGLETON
        .get_or_init(|| {
            Arc::new(SkColorSpaceXformColorFilter::new(
                Some(SkColorSpace::MakeSRGBLinear()),
                Some(SkColorSpace::MakeSRGB()),
            ))
        })
        .clone()
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:88-92
pub fn SRGBToLinearGamma() -> Arc<dyn SkColorFilter> {
    static SINGLETON: std::sync::OnceLock<Arc<SkColorSpaceXformColorFilter>> =
        std::sync::OnceLock::new();
    SINGLETON
        .get_or_init(|| {
            Arc::new(SkColorSpaceXformColorFilter::new(
                Some(SkColorSpace::MakeSRGB()),
                Some(SkColorSpace::MakeSRGBLinear()),
            ))
        })
        .clone()
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:94-97
pub fn MakeColorSpaceXform(
    src: Option<Arc<SkColorSpace>>,
    dst: Option<Arc<SkColorSpace>>,
) -> Arc<dyn SkColorFilter> {
    Arc::new(SkColorSpaceXformColorFilter::new(src, dst))
}
// Source: src/effects/colorfilters/SkColorSpaceXformColorFilter.cpp:99-106
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterSkColorSpaceXformColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkColorSpaceXformColorFilter",
            SkColorSpaceXformColorFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"ColorSpaceXformColorFilter",
            SkColorSpaceXformColorFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkSRGBGammaColorFilter",
            SkColorSpaceXformColorFilter::LegacyGammaOnlyCreateProc,
        );
    }
}
