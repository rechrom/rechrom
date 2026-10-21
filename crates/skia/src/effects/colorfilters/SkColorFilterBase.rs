// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::{SkBlendMode::SkBlendMode, SkColor::SkColor, SkEffectPriv::SkStageRec};
// Source: src/effects/colorfilters/SkColorFilterBase.h:26-33
// Source: src/effects/colorfilters/SkColorFilterBase.h:43-43
// Source: src/effects/colorfilters/SkColorFilterBase.h:45-45
// Source: src/effects/colorfilters/SkColorFilterBase.h:47-47
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kNoop,
    kBlendMode,
    kColorSpaceXform,
    kCompose,
    kGaussian,
    kMatrix,
    kRuntime,
    kTable,
    kWorkingFormat,
}
// Source: src/effects/colorfilters/SkColorFilterBase.h:35-35
// Source: src/effects/colorfilters/SkColorFilterBase.h:37-37
// Source: src/effects/colorfilters/SkColorFilterBase.h:52-52
// Public source contract requires filters to be shareable and reentrant across threads.
// Source: src/effects/colorfilters/SkColorFilterBase.h:79-79
// Adapted native base construction: Rust trait composition has no separately
// constructible base subobject. Real concrete factories initialize Arc owners;
// this replaces the implicit SkFlattenable/SkRefCntBase count=1 construction.
pub trait SkColorFilterBase:
    Send + Sync + crate::src::core::SkFlattenable::SkFlattenableView
{
    // Source: src/effects/colorfilters/SkColorFilterBase.h:59-59
    fn asRuntimeEffect(&self) -> Option<&crate::src::core::SkRuntimeEffect::SkRuntimeEffect> {
        None
    }
    /// Caller keeps the filter, pipeline and arena alive through pipeline execution.
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError>;
    fn r#type(&self) -> Type;
    // Source: src/effects/colorfilters/SkColorFilterBase.h:41-41
    fn onIsAlphaUnchanged(&self) -> bool {
        false
    }
    // Source: src/effects/colorfilters/SkColorFilterBase.h:81-82
    // Source: src/effects/colorfilters/SkColorFilterBase.cpp:25-26
    // Source: src/effects/colorfilters/SkColorFilterBase.cpp:29-30
    fn onAsAColorMode(
        &self,
        _color: Option<&mut SkColor>,
        _mode: Option<&mut SkBlendMode>,
    ) -> bool {
        false
    }
    fn onAsAColorMatrix(&self, _matrix: Option<&mut [f32; 20]>) -> bool {
        false
    }
    // Source: src/effects/colorfilters/SkColorFilterBase.h:54-57
    // Adapted: propagate the same explicit incomplete-stage result as scalar filtering.
    fn affectsTransparentBlack(&self) -> Result<bool, ColorFilterEvaluationError> {
        let transparent = crate::src::core::SkColor::SkColor4f {
            fR: 0.0,
            fG: 0.0,
            fB: 0.0,
            fA: 0.0,
        };
        Ok(
            crate::src::core::SkColorFilter::SkColorFilter::filterColor4f(
                self,
                &transparent,
                None,
                None,
            )? != transparent,
        )
    }
    // Source: src/effects/colorfilters/SkColorFilterBase.h:76-76
    // Source: src/effects/colorfilters/SkColorFilterBase.cpp:33-57
    // Adapted: execution remains fallible while canonical pipeline stage translations are incomplete.
    // Missing stages and rejected append are surfaced, never converted into a fabricated color.
    fn onFilterColor4f(
        &self,
        color: &crate::src::core::SkColorData::SkPMColor4f,
        dstCS: Option<&std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Result<crate::src::core::SkColorData::SkPMColor4f, ColorFilterEvaluationError> {
        use crate::src::core::{
            SkArenaAlloc::SkSTArenaAlloc, SkColorData::SkPMColor4f,
            SkRasterPipeline::SkRasterPipeline, SkRasterPipelineOpContexts::MemoryCtx,
            SkRasterPipelineOpList::SkRasterPipelineOp as Op, SkRect::SkRect,
        };
        let mut storage = std::pin::pin!(SkSTArenaAlloc::<2048>::new(0));
        // The inline arena never moves, and pipeline plus every stack context die before storage.
        let alloc = unsafe { storage.as_mut().arena() } as *mut _;
        let mut pipeline = unsafe { SkRasterPipeline::new(alloc) };
        unsafe {
            pipeline.appendConstantColor(alloc, color.vec());
        }
        let props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        let rec = SkStageRec {
            fPipeline: &mut pipeline,
            fAlloc: alloc,
            fDstColorType: crate::include::core::SkColorType::SkColorType::kRGBA_F32_SkColorType,
            fDstCS: dstCS,
            fPaintColor: color.unpremul(),
            fSurfaceProps: &props,
            fDstBounds: SkRect::MakeEmpty(),
        };
        if unsafe { self.appendStages(&rec, color.fA == 1.0) }
            .map_err(ColorFilterEvaluationError::from)?
        {
            let mut dst = core::mem::MaybeUninit::<SkPMColor4f>::uninit();
            let mut dstPtr = MemoryCtx {
                pixels: dst.as_mut_ptr().cast(),
                stride: 0,
            };
            unsafe {
                pipeline.append(Op::store_f32, (&mut dstPtr as *mut MemoryCtx).cast());
                pipeline
                    .run(0, 0, 1, 1)
                    .map_err(ColorFilterEvaluationError::MissingStage)?;
                // Successful execution includes our final store_f32 of this one destination pixel.
                return Ok(dst.assume_init());
            }
        }
        Err(ColorFilterEvaluationError::RejectedStages)
    }
}

// Explicit translation-boundary error; not an upstream color-filter type or fallback color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorFilterEvaluationError {
    MissingStage(crate::src::core::SkRasterPipeline::MissingStage),
    RejectedStages,
}

// Existing error protocol projection, preserving every actual RasterError variant.
impl From<crate::RasterError> for ColorFilterEvaluationError {
    fn from(error: crate::RasterError) -> Self {
        match error {
            crate::RasterError::MissingStage(stage) => Self::MissingStage(stage),
            crate::RasterError::RejectedStages => Self::RejectedStages,
        }
    }
}

// Source: src/effects/colorfilters/SkColorFilterBase.h:61-63
pub fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type {
    crate::src::core::SkFlattenable::Type::kSkColorFilter_Type
}

// Source: src/effects/colorfilters/SkColorFilterBase.h:69-74
// The native release/adopt cast becomes a move from the canonical typed owner.
// No filter is reconstructed and the original Arc counter is preserved.
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<
    Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
    crate::RasterError,
> {
    match crate::src::core::SkFlattenable::Deserialize(GetFlattenableType(), data, procs)? {
        None => Ok(None),
        Some(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter(owner)) => {
            Ok(Some(owner))
        }
        Some(_) => {
            unreachable!("color filter factory must return its canonical color filter owner")
        }
    }
}

// Source: src/effects/colorfilters/SkColorFilterBase.h:90-91
// UNVERIFIED: borrowed nullable trait upcast preserves the original object lease.
pub fn as_CFB<'a>(
    filter: Option<&'a mut (dyn crate::src::core::SkColorFilter::SkColorFilter + 'a)>,
) -> Option<&'a mut (dyn SkColorFilterBase + 'a)> {
    filter.map(|filter| {
        let base: &'a mut (dyn SkColorFilterBase + 'a) = filter;
        base
    })
}
// Source: src/effects/colorfilters/SkColorFilterBase.h:94-95
pub fn as_CFB_const<'a>(
    filter: Option<&'a (dyn crate::src::core::SkColorFilter::SkColorFilter + 'a)>,
) -> Option<&'a (dyn SkColorFilterBase + 'a)> {
    filter.map(|filter| {
        let base: &'a (dyn SkColorFilterBase + 'a) = filter;
        base
    })
}
// Source: src/effects/colorfilters/SkColorFilterBase.h:98-99
pub fn as_CFB_ref(
    filter: Option<&std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
) -> Option<&dyn SkColorFilterBase> {
    filter.map(|filter| {
        let base: &dyn SkColorFilterBase = filter.as_ref();
        base
    })
}
// Source: src/effects/colorfilters/SkColorFilterBase.h:102-103
// Native release/adopt is an Arc unsizing move: transfer this exact strong owner
// without cloning, retaining an additional reference, or prematurely dropping it.
pub fn as_CFB_sp(
    filter: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
) -> Option<std::sync::Arc<dyn SkColorFilterBase>> {
    filter.map(|filter| {
        let base: std::sync::Arc<dyn SkColorFilterBase> = filter;
        base
    })
}

// Native header declarations project the existing canonical implementations.
// These exports add no object, owner, counter, fallback or native body.
// Source: src/effects/colorfilters/SkColorFilterBase.h:106-106
pub use super::SkComposeColorFilter::SkRegisterComposeColorFilterFlattenable;
// Source: src/effects/colorfilters/SkColorFilterBase.h:107-107
pub use super::SkMatrixColorFilter::SkRegisterMatrixColorFilterFlattenable;
// Source: src/effects/colorfilters/SkColorFilterBase.h:108-108
pub use super::SkBlendModeColorFilter::SkRegisterModeColorFilterFlattenable;
// Source: src/effects/colorfilters/SkColorFilterBase.h:109-109
pub use super::SkColorSpaceXformColorFilter::SkRegisterSkColorSpaceXformColorFilterFlattenable;
// Source: src/effects/colorfilters/SkColorFilterBase.h:110-110
pub use super::SkTableColorFilter::SkRegisterTableColorFilterFlattenable;
// Source: src/effects/colorfilters/SkColorFilterBase.h:111-111
pub use super::SkWorkingFormatColorFilter::SkRegisterWorkingFormatColorFilterFlattenable;
