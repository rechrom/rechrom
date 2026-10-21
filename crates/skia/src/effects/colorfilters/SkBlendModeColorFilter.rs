// Copyright 2023 Google LLC. BSD-style license.
use super::SkColorFilterBase::Type;
use crate::src::core::{
    SkBlendMode::SkBlendMode,
    SkColor::{SkColor, SkColor4f},
};

// Source: src/effects/colorfilters/SkBlendModeColorFilter.h:19-19
// Source: src/effects/colorfilters/SkBlendModeColorFilter.h:39-40
pub struct SkBlendModeColorFilter {
    fColor: SkColor4f,
    fMode: SkBlendMode,
}
impl SkBlendModeColorFilter {
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:21-21
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:27-28
    pub fn new(color: &SkColor4f, mode: SkBlendMode) -> Self {
        Self {
            fColor: *color,
            fMode: mode,
        }
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:37-37
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:30-38
    pub fn onAsAColorMode(
        &self,
        color: Option<&mut SkColor>,
        mode: Option<&mut SkBlendMode>,
    ) -> bool {
        if let Some(color) = color {
            *color = self.fColor.toSkColor();
        }
        if let Some(mode) = mode {
            *mode = self.fMode;
        }
        true
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:25-25
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:40-49
    pub fn onIsAlphaUnchanged(&self) -> bool {
        match self.fMode {
            SkBlendMode::kDst | SkBlendMode::kSrcATop => true,
            _ => false,
        }
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:27-27
    pub fn r#type(&self) -> Type {
        Type::kBlendMode
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:29-30
    pub fn color(&self) -> SkColor4f {
        self.fColor
    }
    pub fn mode(&self) -> SkBlendMode {
        self.fMode
    }
}

use super::SkColorFilterBase::SkColorFilterBase;
use crate::include::core::SkAlphaType::SkAlphaType::{kPremul_SkAlphaType, kUnpremul_SkAlphaType};
use crate::src::core::{
    SkBlendMode::SkBlendMode_AppendStages, SkColorFilter::SkColorFilter,
    SkColorSpace::SkColorSpace, SkColorSpaceXformSteps::SkColorSpaceXformSteps,
    SkEffectPriv::SkStageRec, SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use std::sync::Arc;
// Source: src/effects/colorfilters/SkBlendModeColorFilter.h:34-34
impl crate::src::core::SkFlattenable::SkFlattenableView for SkBlendModeColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkBlendModeColorFilter")
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
// Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:51-54
impl SkBlendModeColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeColor4f(&self.fColor);
        buffer.writeUInt(self.fMode as u32);
        Ok(())
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:56-69
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let color = if buffer.isVersionLT(crate::src::core::SkPicturePriv::kBlend4fColorFilter) {
            SkColor4f::FromColor(buffer.readColor())
        } else {
            let mut color = SkColor4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            };
            buffer.readColor4f(&mut color);
            color
        };
        let mode = buffer.readUInt();
        // The native Blend factory rejects these integers without invalidating the stream.
        if mode > SkBlendMode::kLastMode as u32 {
            return Ok(None);
        }
        let mode: SkBlendMode = unsafe { std::mem::transmute(mode as u8) };
        Ok(Blend(&color, None, mode)
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter))
    }
}
impl SkColorFilterBase for SkBlendModeColorFilter {
    fn r#type(&self) -> Type {
        SkBlendModeColorFilter::r#type(self)
    }
    fn onIsAlphaUnchanged(&self) -> bool {
        SkBlendModeColorFilter::onIsAlphaUnchanged(self)
    }
    fn onAsAColorMode(&self, color: Option<&mut SkColor>, mode: Option<&mut SkBlendMode>) -> bool {
        SkBlendModeColorFilter::onAsAColorMode(self, color, mode)
    }
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.h:23-23
    // Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:71-79
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        _shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            (*rec.fPipeline).append(Op::move_src_dst, std::ptr::null_mut());
            let mut color = self.fColor;
            let srgb = SkColorSpace::MakeSRGB();
            SkColorSpaceXformSteps::new(
                Some(&srgb),
                kUnpremul_SkAlphaType,
                rec.fDstCS.map(std::sync::Arc::as_ref),
                kPremul_SkAlphaType,
            )
            .apply(color.vec_mut());
            (*rec.fPipeline).appendConstantColor(rec.fAlloc, color.vec());
            SkBlendMode_AppendStages(self.fMode, &mut *rec.fPipeline);
        }
        Ok(true)
    }
}
// Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:83-123
pub fn Blend(
    color: &SkColor4f,
    colorSpace: Option<Arc<SkColorSpace>>,
    mut mode: SkBlendMode,
) -> Option<Arc<dyn SkColorFilter>> {
    // Rust's closed enum already enforces SkIsValidMode; invalid discriminants cannot be supplied.
    let mut srgb = color.pinAlpha();
    let srgbSpace = SkColorSpace::MakeSRGB();
    SkColorSpaceXformSteps::new(
        colorSpace.as_deref(),
        kUnpremul_SkAlphaType,
        Some(&srgbSpace),
        kUnpremul_SkAlphaType,
    )
    .apply(srgb.vec_mut());
    let alpha = srgb.fA;
    if mode == SkBlendMode::kClear {
        srgb = SkColor4f {
            fR: 0.0,
            fG: 0.0,
            fB: 0.0,
            fA: 0.0,
        };
        mode = SkBlendMode::kSrc;
    } else if mode == SkBlendMode::kSrcOver {
        if alpha == 0.0 {
            mode = SkBlendMode::kDst;
        } else if alpha == 1.0 {
            mode = SkBlendMode::kSrc;
        }
    }
    if mode == SkBlendMode::kDst
        || (alpha == 0.0
            && matches!(
                mode,
                SkBlendMode::kSrcOver
                    | SkBlendMode::kDstOver
                    | SkBlendMode::kDstOut
                    | SkBlendMode::kSrcATop
                    | SkBlendMode::kXor
                    | SkBlendMode::kDarken
            ))
        || (alpha == 1.0 && mode == SkBlendMode::kDstIn)
    {
        return None;
    }
    Some(Arc::new(SkBlendModeColorFilter::new(&srgb, mode)))
}
// Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:125-127
pub fn BlendColor(color: SkColor, mode: SkBlendMode) -> Option<Arc<dyn SkColorFilter>> {
    Blend(&SkColor4f::FromColor(color), None, mode)
}
// Source: src/effects/colorfilters/SkBlendModeColorFilter.cpp:129-133
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterModeColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkBlendModeColorFilter",
            SkBlendModeColorFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkModeColorFilter",
            SkBlendModeColorFilter::CreateProc,
        );
    }
}
