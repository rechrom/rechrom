// Copyright 2016 Google Inc.; Copyright 2017 Google LLC. BSD-style license.
// Source: include/core/SkBlendMode.h:38-81
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum SkBlendMode {
    kClear,
    kSrc,
    kDst,
    kSrcOver,
    kDstOver,
    kSrcIn,
    kDstIn,
    kSrcOut,
    kDstOut,
    kSrcATop,
    kDstATop,
    kXor,
    kPlus,
    kModulate,
    kScreen,
    kOverlay,
    kDarken,
    kLighten,
    kColorDodge,
    kColorBurn,
    kHardLight,
    kSoftLight,
    kDifference,
    kExclusion,
    kMultiply,
    kHue,
    kSaturation,
    kColor,
    kLuminosity,
}
impl SkBlendMode {
    pub const kLastCoeffMode: Self = Self::kScreen;
    pub const kLastSeparableMode: Self = Self::kMultiply;
    pub const kLastMode: Self = Self::kLuminosity;
}
pub const kSkBlendModeCount: usize = 29;
// Source: include/core/SkBlendMode.h:89-102
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkBlendModeCoeff {
    kZero,
    kOne,
    kSC,
    kISC,
    kDC,
    kIDC,
    kSA,
    kISA,
    kDA,
    kIDA,
    kCoeffCount,
}
// Source: src/core/SkBlendMode.cpp:20-50
pub fn SkBlendMode_ShouldPreScaleCoverage(mode: SkBlendMode, rgb_coverage: bool) -> bool {
    use SkBlendMode::*;
    match mode {
        kDst | kDstOver | kPlus => true,
        kDstOut | kSrcATop | kSrcOver | kXor => !rgb_coverage,
        _ => false,
    }
}
// Source: src/core/SkBlendMode.cpp:53-55
pub fn SkBlendMode_SupportsCoverageAsAlpha(mode: SkBlendMode) -> bool {
    SkBlendMode_ShouldPreScaleCoverage(mode, false)
}
// Source: src/core/SkBlendMode.cpp:57-98
pub fn SkBlendMode_AsCoeff(mode: SkBlendMode) -> Option<(SkBlendModeCoeff, SkBlendModeCoeff)> {
    use SkBlendModeCoeff::*;
    const COEFFS: [(SkBlendModeCoeff, SkBlendModeCoeff); 15] = [
        (kZero, kZero),
        (kOne, kZero),
        (kZero, kOne),
        (kOne, kISA),
        (kIDA, kOne),
        (kDA, kZero),
        (kZero, kSA),
        (kIDA, kZero),
        (kZero, kISA),
        (kDA, kISA),
        (kIDA, kSA),
        (kIDA, kISA),
        (kOne, kOne),
        (kZero, kSC),
        (kOne, kISC),
    ];
    if mode > SkBlendMode::kScreen {
        return None;
    }
    Some(COEFFS[mode as usize])
}
// Source: src/core/SkBlendMode.cpp:97-133
// Rust typed enum exhausts valid modes. The native invalid-mode default and
// SkUNREACHABLE have no valid-enum execution path; these2lines are adapted,
// not credited as a literal default branch. All validmode cases retain ops.
/// Safety: pipeline's arena must satisfy its native lifetime contract.
pub unsafe fn SkBlendMode_AppendStages(
    mode: SkBlendMode,
    p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
) {
    use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
    let op = match mode {
        SkBlendMode::kSrc => return,
        SkBlendMode::kClear => Op::clear,
        SkBlendMode::kDst => Op::move_dst_src,
        SkBlendMode::kSrcOver => Op::srcover,
        SkBlendMode::kDstOver => Op::dstover,
        SkBlendMode::kSrcIn => Op::srcin,
        SkBlendMode::kDstIn => Op::dstin,
        SkBlendMode::kSrcOut => Op::srcout,
        SkBlendMode::kDstOut => Op::dstout,
        SkBlendMode::kSrcATop => Op::srcatop,
        SkBlendMode::kDstATop => Op::dstatop,
        SkBlendMode::kXor => Op::xor_,
        SkBlendMode::kPlus => Op::plus_,
        SkBlendMode::kModulate => Op::modulate,
        SkBlendMode::kScreen => Op::screen,
        SkBlendMode::kOverlay => Op::overlay,
        SkBlendMode::kDarken => Op::darken,
        SkBlendMode::kLighten => Op::lighten,
        SkBlendMode::kColorDodge => Op::colordodge,
        SkBlendMode::kColorBurn => Op::colorburn,
        SkBlendMode::kHardLight => Op::hardlight,
        SkBlendMode::kSoftLight => Op::softlight,
        SkBlendMode::kDifference => Op::difference,
        SkBlendMode::kExclusion => Op::exclusion,
        SkBlendMode::kMultiply => Op::multiply,
        SkBlendMode::kHue => Op::hue,
        SkBlendMode::kSaturation => Op::saturation,
        SkBlendMode::kColor => Op::color,
        SkBlendMode::kLuminosity => Op::luminosity,
    };
    p.append(op, core::ptr::null_mut());
}
// Source: src/core/SkBlendMode.cpp:135-164
// MissingStage Result is a Rust error-boundary adaptation; no missing operation
// is replaced by a default color. All original fast paths and pipeline ops remain.
pub fn SkBlendMode_Apply(
    mode: SkBlendMode,
    src: &crate::src::core::SkColorData::SkPMColor4f,
    dst: &crate::src::core::SkColorData::SkPMColor4f,
) -> Result<
    crate::src::core::SkColorData::SkPMColor4f,
    crate::src::core::SkRasterPipeline::MissingStage,
> {
    use crate::src::core::SkColorData::{SK_PMColor4fTRANSPARENT, SkPMColor4f};
    match mode {
        SkBlendMode::kClear => return Ok(SK_PMColor4fTRANSPARENT),
        SkBlendMode::kSrc => return Ok(*src),
        SkBlendMode::kDst => return Ok(*dst),
        SkBlendMode::kSrcOver => {
            let mut out = core::mem::MaybeUninit::<SkPMColor4f>::uninit();
            let result = out.as_mut_ptr().cast::<f32>();
            for i in 0..4 {
                unsafe {
                    result
                        .add(i)
                        .write(src.vec()[i] + dst.vec()[i] * (1.0 - src.fA));
                }
            }
            return Ok(unsafe { out.assume_init() });
        }
        _ => {}
    }
    let mut storage = core::pin::pin!(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<256>::new(
        0
    ));
    let mut source = *src;
    let mut destination = *dst;
    let mut result = core::mem::MaybeUninit::<SkPMColor4f>::uninit();
    use crate::src::core::SkRasterPipelineOpContexts::MemoryCtx;
    let mut source_ctx = MemoryCtx {
        pixels: source.vec_mut().as_mut_ptr().cast(),
        stride: 0,
    };
    let mut destination_ctx = MemoryCtx {
        pixels: destination.vec_mut().as_mut_ptr().cast(),
        stride: 0,
    };
    let mut result_ctx = MemoryCtx {
        pixels: result.as_mut_ptr().cast(),
        stride: 0,
    };
    // Original synchronous one-pixel program: all contexts and spans remain
    // live/exclusive through run, and store_f32 initializes all result components.
    unsafe {
        let mut pipeline =
            crate::src::core::SkRasterPipeline::SkRasterPipeline::new(storage.as_mut().arena());
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        pipeline.append(
            Op::load_f32,
            (&mut destination_ctx as *mut MemoryCtx).cast(),
        );
        pipeline.append(Op::move_src_dst, core::ptr::null_mut());
        pipeline.append(Op::load_f32, (&mut source_ctx as *mut MemoryCtx).cast());
        SkBlendMode_AppendStages(mode, &mut pipeline);
        pipeline.append(Op::store_f32, (&mut result_ctx as *mut MemoryCtx).cast());
        pipeline.run(0, 0, 1, 1)?;
        Ok(result.assume_init())
    }
}
// Source: src/core/SkBlendMode.cpp:166-201
pub fn SkBlendMode_Name(mode: SkBlendMode) -> &'static str {
    match mode {
        SkBlendMode::kClear => "Clear",
        SkBlendMode::kSrc => "Src",
        SkBlendMode::kDst => "Dst",
        SkBlendMode::kSrcOver => "SrcOver",
        SkBlendMode::kDstOver => "DstOver",
        SkBlendMode::kSrcIn => "SrcIn",
        SkBlendMode::kDstIn => "DstIn",
        SkBlendMode::kSrcOut => "SrcOut",
        SkBlendMode::kDstOut => "DstOut",
        SkBlendMode::kSrcATop => "SrcATop",
        SkBlendMode::kDstATop => "DstATop",
        SkBlendMode::kXor => "Xor",
        SkBlendMode::kPlus => "Plus",
        SkBlendMode::kModulate => "Modulate",
        SkBlendMode::kScreen => "Screen",
        SkBlendMode::kOverlay => "Overlay",
        SkBlendMode::kDarken => "Darken",
        SkBlendMode::kLighten => "Lighten",
        SkBlendMode::kColorDodge => "ColorDodge",
        SkBlendMode::kColorBurn => "ColorBurn",
        SkBlendMode::kHardLight => "HardLight",
        SkBlendMode::kSoftLight => "SoftLight",
        SkBlendMode::kDifference => "Difference",
        SkBlendMode::kExclusion => "Exclusion",
        SkBlendMode::kMultiply => "Multiply",
        SkBlendMode::kHue => "Hue",
        SkBlendMode::kSaturation => "Saturation",
        SkBlendMode::kColor => "Color",
        SkBlendMode::kLuminosity => "Luminosity",
    }
}
// Source: src/core/SkBlendMode.cpp:203-205
fn just_solid_color(p: &crate::src::core::SkPaint::SkPaint) -> bool {
    p.getAlpha() == 255 && p.getColorFilter().is_none() && p.getShader().is_none()
}
// Source: src/core/SkBlendMode.cpp:207-240
pub fn CheckFastPath(
    paint: &crate::src::core::SkPaint::SkPaint,
    dstIsOpaque: bool,
) -> crate::src::core::SkBlendModePriv::SkBlendFastPath {
    use crate::src::core::SkBlendModePriv::SkBlendFastPath::*;
    let bm = paint.asBlendMode();
    let Some(mode) = bm else {
        return kNormal;
    };
    match mode {
        SkBlendMode::kSrcOver => return kSrcOver,
        SkBlendMode::kSrc => {
            if just_solid_color(paint) {
                return kSrcOver;
            }
            return kNormal;
        }
        SkBlendMode::kDst => return kSkipDrawing,
        SkBlendMode::kDstOver => {
            if dstIsOpaque {
                return kSkipDrawing;
            }
            return kNormal;
        }
        SkBlendMode::kSrcIn => {
            if dstIsOpaque && just_solid_color(paint) {
                return kSrcOver;
            }
            return kNormal;
        }
        SkBlendMode::kDstIn => {
            if just_solid_color(paint) {
                return kSkipDrawing;
            }
            return kNormal;
        }
        _ => return kNormal,
    }
}
