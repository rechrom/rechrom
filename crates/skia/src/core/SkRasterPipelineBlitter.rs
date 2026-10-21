// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2016 Google Inc. BSD-style license.
use crate::src::core::SkBlendMode::SkBlendMode;
use crate::src::core::SkColor::SkColor4f;
use crate::src::core::SkPaint::SkPaint;
use crate::src::core::SkPixmap::SkPixmap;
// Source: src/core/SkRasterPipelineBlitter.cpp:54-70
pub fn can_direct_blit(paint: &SkPaint) -> bool {
    can_direct_blit_with_shader_presence(paint, paint.getShader().is_some())
}
fn can_direct_blit_with_shader_presence(paint: &SkPaint, shader_present: bool) -> bool {
    if shader_present || paint.getColorFilter().is_some() || paint.isDither() {
        return false;
    }
    if let Some(mode) = paint.asBlendMode() {
        match mode {
            SkBlendMode::kClear | SkBlendMode::kSrc => return true,
            SkBlendMode::kSrcOver => return paint.getAlphaf() == 1.0,
            _ => {}
        }
    }
    false
}
impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:84-93
    /// Safety: alloc and destination backing store must remain live throughout
    /// all blitter use. The value must be pinned before appending self-contexts.
    pub unsafe fn new(
        dst: SkPixmap,
        paint: &SkPaint,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Self {
        Self {
            fDst: dst,
            fAlloc: alloc,
            fColorPipeline: crate::src::core::SkRasterPipeline::SkRasterPipeline::new(alloc),
            fBlendPipeline: crate::src::core::SkRasterPipeline::SkRasterPipeline::new(alloc),
            fBlendMode: None,
            fClipShaderBuffer: core::ptr::null_mut(),
            fCanDirectBlit: can_direct_blit(paint),
            fDirectBlitPaintColor: paint.getColor4f(),
            fDirectBlitValue: None,
            fDstPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx {
                pixels: core::ptr::null_mut(),
                stride: 0,
            },
            fMaskPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx {
                pixels: core::ptr::null_mut(),
                stride: 0,
            },
            fEmbossCtx: core::mem::MaybeUninit::uninit(),
            fMemset2D: None,
            fMemsetColor: 0,
            fBlitRect: None,
            fBlitAntiH: None,
            fBlitMaskA8: None,
            fBlitMaskLCD16: None,
            fBlitMask3D: None,
            fCurrentCoverage: 0.0,
            fDitherRate: 0.0,
            _pin: core::marker::PhantomPinned,
            fBlitterStorage: crate::src::core::SkBlitter::SkBlitterStorage::new(),
        }
    }
}
// Source: src/core/SkRasterPipelineBlitter.cpp:112-144
// Canonical arena-backed CompiledPipeline plus Option retains the native
// callable/null state. Its lifetime belongs to the external arena/context lease,
// not the temporary assembly pipeline. Pin stabilizes self-context pointers.
// Emboss is genuinely uninitialized until the native mask branch sets it.
type BlitProc<'arena> = crate::src::core::SkRasterPipeline::CompiledPipeline<'arena>;
// Rust fields drop in declaration order. Reverse the native member order so
// cached callable thunks and pipelines die before the retained destination,
// and keep the inherited scratch-allocation owner last, as in the C++ base.
pub struct SkRasterPipelineBlitter<'arena> {
    _pin: core::marker::PhantomPinned,
    fDitherRate: f32,
    fCurrentCoverage: f32,
    fBlitMask3D: Option<BlitProc<'arena>>,
    fBlitMaskLCD16: Option<BlitProc<'arena>>,
    fBlitMaskA8: Option<BlitProc<'arena>>,
    fBlitAntiH: Option<BlitProc<'arena>>,
    fBlitRect: Option<BlitProc<'arena>>,
    fMemsetColor: u64,
    fMemset2D: Option<unsafe fn(*mut SkPixmap, i32, i32, i32, i32, u64)>,
    fEmbossCtx: core::mem::MaybeUninit<crate::src::core::SkRasterPipelineOpContexts::EmbossCtx>,
    fMaskPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx,
    fDstPtr: crate::src::core::SkRasterPipelineOpContexts::MemoryCtx,
    fDirectBlitValue: Option<u64>,
    fDirectBlitPaintColor: SkColor4f,
    fCanDirectBlit: bool,
    fClipShaderBuffer: *mut core::ffi::c_void,
    fBlendMode: Option<SkBlendMode>,
    fBlendPipeline: crate::src::core::SkRasterPipeline::SkRasterPipeline,
    fColorPipeline: crate::src::core::SkRasterPipeline::SkRasterPipeline,
    fAlloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    fDst: SkPixmap,
    fBlitterStorage: crate::src::core::SkBlitter::SkBlitterStorage,
}

// Source: src/core/SkRasterPipelineBlitter.cpp:149-154
pub fn paint_color_to_dst(paint: &SkPaint, dst: &SkPixmap) -> SkColor4f {
    use crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType;
    let mut paintColor = paint.getColor4f();
    crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
        Some(crate::src::core::SkColorSpace::sk_srgb_singleton().as_ref()),
        kUnpremul_SkAlphaType,
        dst.colorSpace(),
        kUnpremul_SkAlphaType,
    )
    .apply(paintColor.vec_mut());
    paintColor
}
// Source: src/core/SkRasterPipelineBlitter.cpp:156-193
/// Safety: source Paint shader pointer must be valid for this call, and alloc
/// plus every appended context must remain valid through pipeline execution.
/// shaderPipeline must be exclusively accessible while stages are appended.
pub unsafe fn create_pipeline_for_blitter(
    dst: &SkPixmap,
    paint: &SkPaint,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    shaderPipeline: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    dstPaintColor: &mut SkColor4f,
    isOpaqueOut: &mut bool,
    isConstantOut: &mut bool,
    devBounds: &crate::src::core::SkRect::SkRect,
) -> Result<bool, crate::RasterError> {
    let view = paint
        .getShader()
        .map(crate::src::shaders::SkBlendShader::BorrowedShaderStage::Shader);
    unsafe {
        create_pipeline_for_blitter_with_view(
            dst,
            paint,
            ctm,
            alloc,
            props,
            shaderPipeline,
            dstPaintColor,
            isOpaqueOut,
            isConstantOut,
            devBounds,
            view.as_ref()
                .map(|v| v as &dyn crate::src::shaders::SkBlendShader::ShaderStageView),
        )
    }
}
// Source: src/core/SkRasterPipelineBlitter.cpp:156-193
pub unsafe fn create_pipeline_for_blitter_with_view(
    dst: &SkPixmap,
    paint: &SkPaint,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    alloc: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    shaderPipeline: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    dstPaintColor: &mut SkColor4f,
    isOpaqueOut: &mut bool,
    isConstantOut: &mut bool,
    devBounds: &crate::src::core::SkRect::SkRect,
    shader: Option<&dyn crate::src::shaders::SkBlendShader::ShaderStageView>,
) -> Result<bool, crate::RasterError> {
    *dstPaintColor = paint_color_to_dst(paint, dst);
    let Some(shader) = shader else {
        shaderPipeline.appendConstantColor(alloc, dstPaintColor.premul().vec());
        *isOpaqueOut = dstPaintColor.fA == 1.0;
        *isConstantOut = true;
        return Ok(true);
    };
    let dstCS = dst.refColorSpace();
    let dstCT = dst.colorType();
    *isOpaqueOut = shader.stage_is_opaque() && dstPaintColor.fA == 1.0;
    *isConstantOut = shader.stage_is_constant(None);
    let rec = crate::src::core::SkEffectPriv::SkStageRec {
        fPipeline: shaderPipeline,
        fAlloc: alloc,
        fDstColorType: dstCT,
        fDstCS: dstCS.as_ref(),
        fPaintColor: *dstPaintColor,
        fSurfaceProps: props,
        fDstBounds: *devBounds,
    };
    if shader.append_root_stage_view(&rec, ctm)? {
        if dstPaintColor.fA != 1.0 {
            shaderPipeline.append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::scale_1_float,
                alloc.make(dstPaintColor.fA).cast(),
            );
        }
        return Ok(true);
    }
    Ok(false)
}

// Source: src/core/SkRasterPipelineBlitter.cpp:195-228
/// Safety: the destination, arena, and stage contexts must satisfy Create's
/// lifetime contract. The returned arena-owned blitter must remain pinned.
pub unsafe fn SkCreateRasterPipelineBlitter<'arena>(
    dst: &SkPixmap,
    paint: &SkPaint,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    alloc: &'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    devBounds: &crate::src::core::SkRect::SkRect,
) -> Result<
    Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    crate::RasterError,
> {
    let view = paint
        .getShader()
        .map(crate::src::shaders::SkBlendShader::BorrowedShaderStage::Shader);
    unsafe {
        SkCreateRasterPipelineBlitter_with_view(
            dst,
            paint,
            ctm,
            alloc,
            clipShader,
            props,
            devBounds,
            view.as_ref()
                .map(|v| v as &dyn crate::src::shaders::SkBlendShader::ShaderStageView),
        )
    }
}
// Source: src/core/SkRasterPipelineBlitter.cpp:195-228
pub unsafe fn SkCreateRasterPipelineBlitter_with_view<'arena>(
    dst: &SkPixmap,
    paint: &SkPaint,
    ctm: &crate::src::core::SkMatrix::SkMatrix,
    alloc: &'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    devBounds: &crate::src::core::SkRect::SkRect,
    shader: Option<&dyn crate::src::shaders::SkBlendShader::ShaderStageView>,
) -> Result<
    Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    crate::RasterError,
> {
    let mut builtin = core::pin::pin!(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<256>::new(
        0
    ));
    let mut shaderPipeline =
        crate::src::core::SkRasterPipeline::SkRasterPipeline::new(builtin.as_mut().arena());
    // Rust output references require initialized values. All three are
    // overwritten by create_pipeline_for_blitter before a successful return.
    let mut dstPaintColor = crate::src::core::SkColor::SkColors::kTransparent;
    let mut is_opaque = false;
    let mut is_constant = false;
    if !create_pipeline_for_blitter_with_view(
        dst,
        paint,
        ctm,
        alloc,
        props,
        &mut shaderPipeline,
        &mut dstPaintColor,
        &mut is_opaque,
        &mut is_constant,
        devBounds,
        shader,
    )? {
        return Ok(None);
    }
    SkRasterPipelineBlitter::Create_with_shader_presence(
        dst,
        paint,
        &dstPaintColor,
        alloc,
        &shaderPipeline,
        is_opaque,
        is_constant,
        clipShader.as_deref(),
        shader.is_some(),
    )
}

// Source: src/core/SkRasterPipelineBlitter.cpp:289-299
// Rust has no overloads; the original pipeline overload has an explicit suffix.
/// Safety: the destination, arena, and stage contexts must satisfy Create's
/// lifetime contract. The returned arena-owned blitter must remain pinned.
pub unsafe fn SkCreateRasterPipelineBlitter_from_pipeline<'arena>(
    dst: &SkPixmap,
    paint: &SkPaint,
    shaderPipeline: &crate::src::core::SkRasterPipeline::SkRasterPipeline,
    is_opaque: bool,
    alloc: &'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    clipShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
) -> Result<
    Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
    crate::RasterError,
> {
    let is_constant = false;
    SkRasterPipelineBlitter::Create(
        dst,
        paint,
        &paint_color_to_dst(paint, dst),
        alloc,
        shaderPipeline,
        is_opaque,
        is_constant,
        clipShader.as_deref(),
    )
}

impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:301-499
    /// Safety: alloc must remain live and unmoved for all blitter calls. Shader,
    /// filter and blender append contracts must retain/copy all needed context
    /// into that arena. Returned pointer is arena-owned and must stay pinned.
    /// Only this completed constructor publishes the private void-dispatch adapter.
    /// Frozen dynamic stage lists are validated before any successful publication.
    pub unsafe fn Create(
        dst: &SkPixmap,
        paint: &SkPaint,
        dstPaintColor: &SkColor4f,
        alloc: &'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        shaderPipeline: &crate::src::core::SkRasterPipeline::SkRasterPipeline,
        mut is_opaque: bool,
        mut is_constant: bool,
        clipShader: Option<&dyn crate::src::shaders::SkShader::SkShader>,
    ) -> Result<
        Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
        crate::RasterError,
    > {
        unsafe {
            Self::Create_with_shader_presence(
                dst,
                paint,
                dstPaintColor,
                alloc,
                shaderPipeline,
                is_opaque,
                is_constant,
                clipShader,
                paint.getShader().is_some(),
            )
        }
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:301-499
    pub unsafe fn Create_with_shader_presence(
        dst: &SkPixmap,
        paint: &SkPaint,
        dstPaintColor: &SkColor4f,
        alloc: &'arena mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        shaderPipeline: &crate::src::core::SkRasterPipeline::SkRasterPipeline,
        mut is_opaque: bool,
        mut is_constant: bool,
        clipShader: Option<&dyn crate::src::shaders::SkShader::SkShader>,
        shader_present: bool,
    ) -> Result<
        Option<core::ptr::NonNull<dyn crate::src::core::SkBlitter::SkBlitter + 'arena>>,
        crate::RasterError,
    > {
        use crate::include::core::SkBlender::SkBlender;
        use crate::include::core::SkColorType::SkColorType::*;
        use crate::include::core::SkSurfaceProps::SkSurfaceProps;
        use crate::src::core::SkBlenderBase::as_BB;
        use crate::src::core::SkColorFilter::SkColorFilter;
        use crate::src::core::SkEffectPriv::SkStageRec;
        use crate::src::core::SkRasterPipelineOpContexts::{kMaxStride, MemoryCtx};
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let arena_ptr = alloc as *mut crate::src::core::SkArenaAlloc::SkArenaAlloc;
        // new does not depend on the final address. Complete paint/blender
        // queries before make_with installs its destructor footer: safe virtual
        // callbacks may panic, and arena construction permits no unwinding after
        // footer registration. The placement closure performs only a pure move.
        let mut value = Self::new(dst.clone(), paint, arena_ptr);
        value.fCanDirectBlit = can_direct_blit_with_shader_presence(paint, shader_present);
        let blitter = core::ptr::NonNull::new(alloc.make_with(|_, _| value)).unwrap();
        let b = blitter.as_ptr();
        if let Some(clipShader) = clipShader {
            let props = SkSurfaceProps::default();
            let dst_cs_owner = dst.refColorSpace();
            let rec = SkStageRec {
                fPipeline: &mut (*b).fColorPipeline,
                fAlloc: arena_ptr,
                fDstColorType: kRGBA_8888_SkColorType,
                fDstCS: None,
                fPaintColor: crate::src::core::SkColor::SkColors::kBlack,
                fSurfaceProps: &props,
                fDstBounds: crate::src::core::SkRect::SkRect::MakeEmpty(),
            };
            if clipShader.appendRootStages(&rec, crate::src::core::SkMatrix::SkMatrix::I())? {
                #[derive(Clone, Copy)]
                struct Storage {
                    fA: [f32; kMaxStride],
                }
                // Native make<Storage>() selects the POD uninitialized-byte
                // branch (SkArenaAlloc.h:158-162), not value initialization.
                let storage = alloc.makeUninit::<Storage>();
                let alpha = core::ptr::addr_of_mut!((*storage).fA).cast::<f32>();
                (*b).fColorPipeline.append(Op::store_src_a, alpha.cast());
                (*b).fClipShaderBuffer = alpha.cast();
                is_constant = false;
            } else {
                return Ok(None);
            }
        }
        (*b).fColorPipeline.extend(shaderPipeline);
        if let Some(colorFilter) = paint.getColorFilter() {
            let props = SkSurfaceProps::default();
            let dst_cs_owner = dst.refColorSpace();
            let rec = SkStageRec {
                fPipeline: &mut (*b).fColorPipeline,
                fAlloc: arena_ptr,
                fDstColorType: dst.colorType(),
                fDstCS: dst_cs_owner.as_ref(),
                fPaintColor: *dstPaintColor,
                fSurfaceProps: &props,
                fDstBounds: crate::src::core::SkRect::SkRect::MakeEmpty(),
            };
            // The canonical public trait inherits the native CPU base directly.
            if !colorFilter.appendStages(&rec, is_opaque)? {
                return Ok(None);
            }
            is_opaque = is_opaque && colorFilter.isAlphaUnchanged();
        }
        if paint.isDither() && !is_constant {
            (*b).fDitherRate = match dst.info().colorType() {
                kARGB_4444_SkColorType => 1.0 / 15.0,
                kRGB_565_SkColorType => 1.0 / 63.0,
                kGray_8_SkColorType
                | kRGB_888x_SkColorType
                | kRGBA_8888_SkColorType
                | kBGRA_8888_SkColorType
                | kSRGBA_8888_SkColorType
                | kR8_unorm_SkColorType => 1.0 / 255.0,
                kRGB_101010x_SkColorType
                | kRGBA_1010102_SkColorType
                | kBGR_101010x_SkColorType
                | kBGRA_1010102_SkColorType
                | kBGRA_10101010_XR_SkColorType
                | kRGBA_10x6_SkColorType => 1.0 / 1023.0,
                kUnknown_SkColorType
                | kAlpha_8_SkColorType
                | kBGR_101010x_XR_SkColorType
                | kRGBA_F16_SkColorType
                | kRGB_F16F16F16x_SkColorType
                | kRGBA_F16Norm_SkColorType
                | kRGBA_F32_SkColorType
                | kR8G8_unorm_SkColorType
                | kA16_float_SkColorType
                | kA16_unorm_SkColorType
                | kR16G16_float_SkColorType
                | kR16_unorm_SkColorType
                | kR16_float_SkColorType
                | kR16G16_unorm_SkColorType
                | kR16G16B16A16_unorm_SkColorType => 0.0,
            };
            if (*b).fDitherRate > 0.0 {
                (*b).fColorPipeline
                    .append(Op::dither, core::ptr::addr_of_mut!((*b).fDitherRate).cast());
            }
        }
        if is_constant {
            let mut constantColor = core::mem::MaybeUninit::<SkColor4f>::uninit();
            let mut ptr = MemoryCtx {
                pixels: constantColor.as_mut_ptr().cast(),
                stride: 0,
            };
            (*b).fColorPipeline.appendClampIfNormalized(dst.info());
            (*b).fColorPipeline
                .append(Op::store_f32, (&mut ptr as *mut MemoryCtx).cast());
            // MissingStage must return before reading the uninitialized color.
            (*b).fColorPipeline.run(0, 0, 1, 1)?;
            (*b).fColorPipeline.reset();
            let constantColor = constantColor.assume_init();
            (*b).fColorPipeline
                .appendConstantColor(arena_ptr, constantColor.vec());
            is_opaque = constantColor.fA == 1.0;
        }
        let mut blender = paint.refBlender();
        if !blender.is_some() {
            blender = <dyn SkBlender>::Mode(SkBlendMode::kSrcOver);
        }
        if is_opaque && as_BB(blender.get().unwrap()).asBlendMode() == Some(SkBlendMode::kSrcOver) {
            blender = <dyn SkBlender>::Mode(SkBlendMode::kSrc);
        }
        if is_constant
            && as_BB(blender.get().unwrap()).asBlendMode() == Some(SkBlendMode::kSrc)
            && dst.info().bytesPerPixel() <= core::mem::size_of::<u64>() as i32
        {
            // Native SkRasterPipeline_<256>: retain the real inline arena and
            // establish its final address before creating a borrowing pipeline.
            let mut builtin = core::pin::pin!(
                crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<256>::new(0)
            );
            let mut p =
                crate::src::core::SkRasterPipeline::SkRasterPipeline::new(builtin.as_mut().arena());
            p.extend(&(*b).fColorPipeline);
            (*b).fDstPtr = MemoryCtx {
                pixels: core::ptr::addr_of_mut!((*b).fMemsetColor).cast(),
                stride: 0,
            };
            (*b).appendStore(&mut p);
            p.run(0, 0, 1, 1)?;
            (*b).fMemset2D = match (*b).fDst.shiftPerPixel() {
                0 => Some(|dst, x, y, w, mut h, c| {
                    let dst = &mut *dst;
                    let mut p = dst.writable_addr_xy(x, y);
                    loop {
                        let old = h;
                        h -= 1;
                        if old <= 0 {
                            break;
                        }
                        p.write_bytes(c as u8, w as usize);
                        p = p.wrapping_add(dst.rowBytes());
                    }
                }),
                1 => Some(|dst, x, y, w, h, c| {
                    let dst = &mut *dst;
                    crate::src::opts::SkMemset_opts::rect_memsetT_raw(
                        dst.writable_addr16(x, y),
                        c as u16,
                        w as usize,
                        dst.rowBytes(),
                        h as usize,
                    );
                }),
                2 => Some(|dst, x, y, w, h, c| {
                    let dst = &mut *dst;
                    crate::src::opts::SkMemset_opts::rect_memset32_raw(
                        dst.writable_addr32(x, y),
                        c as u32,
                        w as usize,
                        dst.rowBytes(),
                        h as usize,
                    );
                }),
                3 => Some(|dst, x, y, w, h, c| {
                    let dst = &mut *dst;
                    crate::src::opts::SkMemset_opts::rect_memsetT_raw(
                        dst.writable_addr64(x, y),
                        c,
                        w as usize,
                        dst.rowBytes(),
                        h as usize,
                    );
                }),
                _ => None,
            };
        }
        {
            let props = SkSurfaceProps::default();
            let dst_cs_owner = dst.refColorSpace();
            let rec = SkStageRec {
                fPipeline: &mut (*b).fBlendPipeline,
                fAlloc: arena_ptr,
                fDstColorType: dst.colorType(),
                fDstCS: dst_cs_owner.as_ref(),
                fPaintColor: *dstPaintColor,
                fSurfaceProps: &props,
                fDstBounds: crate::src::core::SkRect::SkRect::MakeEmpty(),
            };
            if !as_BB(blender.get().unwrap()).appendStages(&rec)? {
                return Ok(None);
            }
            (*b).fBlendMode = as_BB(blender.get().unwrap()).asBlendMode();
        }
        (*b).fDstPtr = MemoryCtx {
            pixels: (*b).fDst.writable_addr().cast(),
            stride: (*b).fDst.rowBytesAsPixels(),
        };
        // Rust integration gate, after the final actual append/reset/extend.
        // Private lists stay frozen after this point. All native lazy suffixes
        // are among the 65 operations verified in the immutable stage table.
        (*b).fColorPipeline.validate_executable()?;
        (*b).fBlendPipeline.validate_executable()?;
        // Both public factories enter this common final gate. Propagate the
        // one-pixel conversion error before a Validated wrapper can escape.
        (*b).prepareDirectBlitValue()?;
        // The arena retains the native object and destroys this pointer-only
        // adapter before it. No adapter is available for an unfinished new().
        // Publish the finalized object through one actual pinned borrow. The
        // arena never relocates this allocation; its footer destroys the adapter
        // before the pointee. Projection cannot safely extract/move !Unpin Self.
        let pinned: core::pin::Pin<&'arena mut Self> = core::pin::Pin::new_unchecked(&mut *b);
        let adapter = alloc.make_with(|_, _| ValidatedPipelineBlitter {
            blitter: pinned,
            _pin: core::marker::PhantomPinned,
        });
        Ok(Some(core::ptr::NonNull::new_unchecked(
            adapter as *mut (dyn crate::src::core::SkBlitter::SkBlitter + 'arena),
        )))
    }
}

impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:501-506
    unsafe fn appendLoadDst(
        &mut self,
        p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline,
    ) {
        p.appendLoadDst(
            self.fDst.info().colorType(),
            core::ptr::addr_of_mut!(self.fDstPtr),
        );
        if self.fDst.info().alphaType()
            == crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
        {
            p.append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::premul_dst,
                core::ptr::null_mut(),
            );
        }
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:508-513
    unsafe fn appendStore(&mut self, p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline) {
        if self.fDst.info().alphaType()
            == crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
        {
            p.append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::unpremul,
                core::ptr::null_mut(),
            );
        }
        p.appendStore(
            self.fDst.info().colorType(),
            core::ptr::addr_of_mut!(self.fDstPtr),
        );
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:515-519
    unsafe fn appendClipScale(&self, p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline) {
        if !self.fClipShaderBuffer.is_null() {
            p.append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::scale_native,
                self.fClipShaderBuffer,
            );
        }
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:521-525
    unsafe fn appendClipLerp(&self, p: &mut crate::src::core::SkRasterPipeline::SkRasterPipeline) {
        if !self.fClipShaderBuffer.is_null() {
            p.append(
                crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp::lerp_native,
                self.fClipShaderBuffer,
            );
        }
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:527-529
    /// Safety: pinned arena-backed instance must have completed native Create
    /// stage assembly, including fDstPtr and its color/blend pipeline state.
    pub unsafe fn blitH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        self.blitRect(x, y, w, 1)
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:531-567
    /// Safety: same initialized-state and context lifetime contract as blitH.
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        use crate::include::core::SkColorType::SkColorType::{
            kBGRA_8888_SkColorType, kRGBA_8888_SkColorType,
        };
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let this = self.get_unchecked_mut();
        if let Some(memset) = this.fMemset2D {
            memset(&mut this.fDst, x, y, w, h, this.fMemsetColor);
            return Ok(());
        }
        if this.fBlitRect.is_none() {
            let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
            p.extend(&this.fColorPipeline);
            p.appendClampIfNormalized(this.fDst.info());
            if this.fBlendMode == Some(SkBlendMode::kSrcOver)
                && matches!(
                    this.fDst.info().colorType(),
                    kRGBA_8888_SkColorType | kBGRA_8888_SkColorType
                )
                && this.fDst.colorSpace().is_none()
                && this.fDst.info().alphaType()
                    != crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
                && this.fDitherRate == 0.0
            {
                if this.fDst.info().colorType() == kBGRA_8888_SkColorType {
                    p.append(Op::swap_rb, core::ptr::null_mut());
                }
                this.appendClipScale(&mut p);
                p.append(
                    Op::srcover_rgba_8888,
                    core::ptr::addr_of_mut!(this.fDstPtr).cast(),
                );
            } else {
                if this.fBlendMode != Some(SkBlendMode::kSrc) {
                    this.appendLoadDst(&mut p);
                    p.extend(&this.fBlendPipeline);
                    this.appendClipLerp(&mut p);
                } else if !this.fClipShaderBuffer.is_null() {
                    this.appendLoadDst(&mut p);
                    this.appendClipLerp(&mut p);
                }
                this.appendStore(&mut p);
            }
            this.fBlitRect = Some(p.compile_for_arena(this.fAlloc)?);
        }
        this.fBlitRect
            .as_mut()
            .expect("compiled above")
            .run(x as usize, y as usize, w as usize, h as usize);
        Ok(())
    }
}

impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:569-603
    /// Safety: instance must have completed native Create, and sparse runs must
    /// address valid destination pixels. Pin keeps coverage context stable.
    /// Cached coverage/mask contexts use raw place projections and raw writes,
    /// without creating an exclusive field reference that outlives this call.
    pub unsafe fn blitAntiH(
        mut self: core::pin::Pin<&mut Self>,
        mut x: i32,
        y: i32,
        aa: &[u8],
        runs: &[i16],
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let this = self.as_mut().get_unchecked_mut();
        if this.fBlitAntiH.is_none() {
            let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
            p.extend(&this.fColorPipeline);
            p.appendClampIfNormalized(this.fDst.info());
            if this.fBlendMode.is_some_and(|m| {
                crate::src::core::SkBlendMode::SkBlendMode_ShouldPreScaleCoverage(m, false)
            }) {
                p.append(
                    Op::scale_1_float,
                    core::ptr::addr_of_mut!(this.fCurrentCoverage).cast(),
                );
                this.appendClipScale(&mut p);
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
            } else {
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
                p.append(
                    Op::lerp_1_float,
                    core::ptr::addr_of_mut!(this.fCurrentCoverage).cast(),
                );
                this.appendClipLerp(&mut p);
            }
            this.appendStore(&mut p);
            this.fBlitAntiH = Some(p.compile_for_arena(this.fAlloc)?);
        }
        let mut offset = 0;
        while runs[offset] > 0 {
            let run = runs[offset];
            match aa[offset] {
                0 => {}
                255 => self.as_mut().blitRect(x, y, run as i32, 1)?,
                coverage => {
                    let this = self.as_mut().get_unchecked_mut();
                    core::ptr::addr_of_mut!(this.fCurrentCoverage)
                        .write(coverage as f32 * (1.0 / 255.0));
                    this.fBlitAntiH.as_mut().expect("compiled above").run(
                        x as usize,
                        y as usize,
                        run as usize,
                        1,
                    );
                }
            }
            x += run as i32;
            offset += run as usize;
        }
        Ok(())
    }
}

impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:605-610
    /// Safety: successful native Create and a writable two-pixel destination
    /// span are required. Pin keeps all retained stage contexts stable.
    pub unsafe fn blitAntiH2(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a0: u32,
        a1: u32,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        let clip = crate::src::core::SkRect::SkIRect {
            fLeft: x,
            fTop: y,
            fRight: x + 2,
            fBottom: y + 1,
        };
        let coverage = [a0 as u8, a1 as u8];
        let mask = crate::src::core::SkMask::SkMask::new(
            coverage.as_ptr(),
            clip,
            2,
            crate::src::core::SkMask::Format::kA8_Format,
        );
        // This native descriptor is always A8, so full blitMask's BW branch is
        // unreachable. Coverage stays live until this synchronous run returns.
        self.blitMask_non_bw(&mask, &clip)
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:612-617
    /// Safety: successful native Create and the two vertical destination pixels
    /// must satisfy the pixel lease and pinned-context contracts.
    pub unsafe fn blitAntiV2(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        a0: u32,
        a1: u32,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        let clip = crate::src::core::SkRect::SkIRect {
            fLeft: x,
            fTop: y,
            fRight: x + 1,
            fBottom: y + 2,
        };
        let coverage = [a0 as u8, a1 as u8];
        let mask = crate::src::core::SkMask::SkMask::new(
            coverage.as_ptr(),
            clip,
            1,
            crate::src::core::SkMask::Format::kA8_Format,
        );
        self.blitMask_non_bw(&mask, &clip)
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:619-625
    /// Safety: successful native Create and all height destination pixels must
    /// remain valid for the synchronous run; owner/stage contexts stay pinned.
    pub unsafe fn blitV(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        height: i32,
        alpha: u8,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        let clip = crate::src::core::SkRect::SkIRect {
            fLeft: x,
            fTop: y,
            fRight: x + 1,
            fBottom: y + height,
        };
        let mask = crate::src::core::SkMask::SkMask::new(
            &alpha,
            clip,
            0,
            crate::src::core::SkMask::Format::kA8_Format,
        );
        self.blitMask_non_bw(&mask, &clip)
    }
}

// Rust virtual-inheritance adaptation, deliberately private. Construction occurs
// only after Create freezes and validates both actual dynamic lists. new() alone
// cannot publish this adapter. The native object is pinned in the same real arena.
struct ValidatedPipelineBlitter<'arena> {
    _pin: core::marker::PhantomPinned,
    blitter: core::pin::Pin<&'arena mut SkRasterPipelineBlitter<'arena>>,
}

// This is an internal invariant assertion, not an error-transport boundary.
// An actual unsupported dynamic opcode has already returned Err from Create;
// frozen lists plus the immutable supported 65-op suffix set prove lazy compile
// cannot return MissingStage here. No typed panic/error slot/fallback is used.
fn require_validated_lazy_compile(
    result: Result<(), crate::src::core::SkRasterPipeline::MissingStage>,
) {
    if result.is_err() {
        panic!("validated immutable blitter stage invariant violated");
    }
}

// Safety: this adapter exists only for completed Create objects. Scratch memory
// is the actual base owner; methods retain canonical unsafe coordinate/run/lease
// contracts. Field contexts and the pointed-to owner remain pinned until arena
// teardown; there is no independent ownership or lifetime extension.
unsafe impl crate::src::core::SkBlitter::SkBlitter for ValidatedPipelineBlitter<'_> {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.blitter
                .as_mut()
                .get_unchecked_mut()
                .fBlitterStorage
                .allocBlitMemory(sz)
        }
    }
    fn canDirectBlit(
        self: core::pin::Pin<&mut Self>,
    ) -> Option<crate::src::core::SkBlitter::DirectBlit> {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blitter.as_mut().get_unchecked_mut().canDirectBlit() }
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitH(x, y, width));
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        aa: &mut [u8],
        runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitAntiH(x, y, aa, runs));
    }
    unsafe fn blitAntiH2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitAntiH2(x, y, a0, a1));
    }
    unsafe fn blitAntiV2(self: core::pin::Pin<&mut Self>, x: i32, y: i32, a0: u32, a1: u32) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitAntiV2(x, y, a0, a1));
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitV(x, y, height, alpha));
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        require_validated_lazy_compile(this.blitter.as_mut().blitRect(x, y, width, height));
    }
    // Source: src/core/SkRasterPipelineBlitter.cpp:627-631
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        if mask.fFormat == crate::src::core::SkMask::Format::kBW_Format {
            return crate::src::core::SkBlitter::blitMask(
                core::pin::Pin::new_unchecked(&mut *this),
                mask,
                clip,
            );
        }
        require_validated_lazy_compile(this.blitter.as_mut().blitMask_non_bw(mask, clip));
    }
}

impl<'arena> SkRasterPipelineBlitter<'arena> {
    // Source: src/core/SkRasterPipelineBlitter.cpp:633-736
    /// Safety: native Create must have initialized the pinned blitter. mask must
    /// be A8/LCD16/3D with live readable planes, and clip must address writable
    /// destination pixels. The BW superclass branch remains separately pending.
    pub unsafe fn blitMask_non_bw(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        use crate::src::core::SkMask::Format;
        use crate::src::core::SkRasterPipelineOpContexts::{EmbossCtx, MemoryCtx};
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        let this = self.get_unchecked_mut();
        crate::SkASSERT!(@source b"src/core/SkRasterPipelineBlitter.cpp\0".as_ptr(), 634; @condition b"mask.fFormat == SkMask::kA8_Format || mask.fFormat == SkMask::kLCD16_Format || mask.fFormat == SkMask::k3D_Format\0".as_ptr(); matches!(
            mask.fFormat,
            Format::kA8_Format | Format::kLCD16_Format | Format::k3D_Format
        ));
        let extract_mask_plane = |plane: usize| {
            let bpp = if mask.fFormat == Format::kLCD16_Format {
                2
            } else {
                1
            };
            let ptr =
                (mask.fImage as usize).wrapping_add(plane.wrapping_mul(mask.computeImageSize()));
            let rowBytes = mask.fRowBytes as usize;
            MemoryCtx {
                stride: (rowBytes / bpp) as i32,
                pixels: ptr
                    .wrapping_sub((mask.fBounds.fLeft as usize).wrapping_mul(bpp))
                    .wrapping_sub((mask.fBounds.fTop as usize).wrapping_mul(rowBytes))
                    as *mut core::ffi::c_void,
            }
        };
        core::ptr::addr_of_mut!(this.fMaskPtr).write(extract_mask_plane(0));
        if mask.fFormat == Format::k3D_Format {
            core::ptr::addr_of_mut!(this.fEmbossCtx)
                .cast::<EmbossCtx>()
                .write(EmbossCtx {
                    mul: extract_mask_plane(1),
                    add: extract_mask_plane(2),
                });
        }
        if mask.fFormat == Format::kA8_Format && this.fBlitMaskA8.is_none() {
            let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
            p.extend(&this.fColorPipeline);
            p.appendClampIfNormalized(this.fDst.info());
            if this.fBlendMode.is_some_and(|m| {
                crate::src::core::SkBlendMode::SkBlendMode_ShouldPreScaleCoverage(m, false)
            }) {
                p.append(Op::scale_u8, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipScale(&mut p);
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
            } else {
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
                p.append(Op::lerp_u8, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipLerp(&mut p);
            }
            this.appendStore(&mut p);
            this.fBlitMaskA8 = Some(p.compile_for_arena(this.fAlloc)?);
        }
        if mask.fFormat == Format::kLCD16_Format && this.fBlitMaskLCD16.is_none() {
            let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
            p.extend(&this.fColorPipeline);
            p.appendClampIfNormalized(this.fDst.info());
            if this.fBlendMode.is_some_and(|m| {
                crate::src::core::SkBlendMode::SkBlendMode_ShouldPreScaleCoverage(m, true)
            }) {
                this.appendLoadDst(&mut p);
                p.append(Op::scale_565, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipScale(&mut p);
                p.extend(&this.fBlendPipeline);
            } else {
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
                p.append(Op::lerp_565, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipLerp(&mut p);
            }
            this.appendStore(&mut p);
            this.fBlitMaskLCD16 = Some(p.compile_for_arena(this.fAlloc)?);
        }
        if mask.fFormat == Format::k3D_Format && this.fBlitMask3D.is_none() {
            let mut p = crate::src::core::SkRasterPipeline::SkRasterPipeline::new(this.fAlloc);
            p.extend(&this.fColorPipeline);
            p.append(
                Op::emboss,
                core::ptr::addr_of_mut!(this.fEmbossCtx)
                    .cast::<EmbossCtx>()
                    .cast(),
            );
            p.appendClampIfNormalized(this.fDst.info());
            if this.fBlendMode.is_some_and(|m| {
                crate::src::core::SkBlendMode::SkBlendMode_ShouldPreScaleCoverage(m, false)
            }) {
                p.append(Op::scale_u8, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipScale(&mut p);
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
            } else {
                this.appendLoadDst(&mut p);
                p.extend(&this.fBlendPipeline);
                p.append(Op::lerp_u8, core::ptr::addr_of_mut!(this.fMaskPtr).cast());
                this.appendClipLerp(&mut p);
            }
            this.appendStore(&mut p);
            this.fBlitMask3D = Some(p.compile_for_arena(this.fAlloc)?);
        }
        let blitter = match mask.fFormat {
            Format::kA8_Format => &mut this.fBlitMaskA8,
            Format::kLCD16_Format => &mut this.fBlitMaskLCD16,
            Format::k3D_Format => &mut this.fBlitMask3D,
            _ => {
                crate::SkASSERT!(@source b"src/core/SkRasterPipelineBlitter.cpp\0".as_ptr(), 730; @condition b"false\0".as_ptr(); false);
                return Ok(());
            }
        };
        crate::SkASSERT!(@source b"src/core/SkRasterPipelineBlitter.cpp\0".as_ptr(), 734; @condition b"blitter\0".as_ptr(); blitter.is_some());
        blitter.as_mut().expect("compiled above").run(
            clip.fLeft as usize,
            clip.fTop as usize,
            clip.width() as usize,
            clip.height() as usize,
        );
        Ok(())
    }
}

// Source: src/core/SkRasterPipelineBlitter.cpp:738-767
// Timing/error-boundary adaptation: compute only the original direct one-pixel
// value at the final Create gate, so MissingStage reaches the factory Result.
// No new mirror/cache field; immutable destination info/paint color permit the
// existing fDirectBlitValue cache. The primitive query remains Option/void-safe.
impl<'arena> SkRasterPipelineBlitter<'arena> {
    /// Safety: original image-info/source-color conversion contract holds and
    /// destination info/color are final; called before publishing the adapter.
    unsafe fn prepareDirectBlitValue(
        &mut self,
    ) -> Result<(), crate::src::core::SkRasterPipeline::MissingStage> {
        if !self.fCanDirectBlit || self.fDirectBlitValue.is_some() {
            return Ok(());
        }
        #[repr(C)]
        union DstBuffer {
            u8: [u64; 1],
            u4: [u32; 2],
            u2: [u16; 4],
            u1: [u8; 8],
        }
        let mut dstBuffer = core::mem::MaybeUninit::<DstBuffer>::uninit();
        let dst = crate::src::core::SkImageInfo::SkImageInfo::Make(
            1,
            1,
            self.fDst.info().colorType(),
            self.fDst.info().alphaType(),
        );
        let src = crate::src::core::SkImageInfo::SkImageInfo::Make(
            1,
            1,
            crate::include::core::SkColorType::SkColorType::kRGBA_F32_SkColorType,
            crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType,
        );
        // Conversion performs the native rowBytes/ValidConversion rejection:
        // a destination wider than this8-byte buffer returns Ok(false), and
        // no union bytes are read. A missing stage propagates before publishing.
        if !crate::src::core::SkConvertPixels::SkConvertPixels(
            &dst,
            dstBuffer.as_mut_ptr().cast(),
            core::mem::size_of::<DstBuffer>(),
            &src,
            (&self.fDirectBlitPaintColor as *const SkColor4f).cast(),
            core::mem::size_of::<SkColor4f>(),
        )? {
            self.fCanDirectBlit = false;
            return Ok(());
        }
        // Read only the initialized bytes of the successful one-pixel result;
        // never form a reference to an incompletely initialized union array.
        let buffer = dstBuffer.as_ptr();
        self.fDirectBlitValue = match dst.bytesPerPixel() {
            1 => Some(buffer.cast::<u8>().read() as u64),
            2 => Some(buffer.cast::<u16>().read() as u64),
            4 => Some(buffer.cast::<u32>().read() as u64),
            8 => Some(buffer.cast::<u64>().read()),
            _ => {
                self.fCanDirectBlit = false;
                None
            }
        };
        Ok(())
    }
    /// Safety: successful Create has prepared the direct-value cache. Raw new
    /// alone does not establish this invariant; only the validated adapter is
    /// published to a SkBlitter caller. This query accesses no pixel backing.
    pub unsafe fn canDirectBlit(&mut self) -> Option<crate::src::core::SkBlitter::DirectBlit> {
        if self.fCanDirectBlit {
            Some(crate::src::core::SkBlitter::DirectBlit {
                pm: self.fDst.clone(),
                value: self
                    .fDirectBlitValue
                    .expect("successful Create prepares direct color"),
            })
        } else {
            None
        }
    }
}
