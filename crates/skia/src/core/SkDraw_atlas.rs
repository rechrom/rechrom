// Source: src/core/SkDraw_atlas.cpp:1-6
// Copyright 2019 Google LLC. BSD-style license.
// Source: src/core/SkDraw_atlas.cpp:8-48
// Native includes/forward declarations bind existing canonical Rust types;
// there are no additional objects, data producers or first-writing credits.
use crate::src::core::{
    SkBlitter::SkBlitter, SkMatrix::SkMatrix, SkRasterClip::SkRasterClip,
    SkRasterPipelineOpContexts::UniformColorCtx, SkRect::SkRect,
};
use core::pin::Pin;
// Source: src/core/SkDraw_atlas.cpp:50-62
unsafe fn fill_rect(
    ctm: &SkMatrix,
    rc: &SkRasterClip,
    r: &SkRect,
    blitter: Pin<&mut dyn SkBlitter>,
) {
    if ctm.rectStaysRect() {
        unsafe {
            crate::src::core::SkScan::SkScan::FillRectRasterClip(
                &ctm.mapRect_value(r),
                rc,
                blitter,
            );
        }
    } else {
        let mut owner = crate::src::core::SkPathRawShapes::Rect::new(
            r,
            crate::include::core::SkPathTypes::SkPathDirection::kCW,
            0,
        );
        ctm.mapPointsInPlace(&mut owner.fStorage);
        if let Some(bounds) = SkRect::Bounds(&owner.fStorage) {
            // raw() lends the actual owner storage; its by-value metadata view
            // carries the updated native fBounds without copying path geometry.
            let mut raw = owner.raw();
            raw.fBounds = bounds;
            unsafe {
                crate::src::core::SkScan::SkScan::FillPathRasterClip(&raw, rc, blitter);
            }
        }
    }
}
// Source: src/core/SkDraw_atlas.cpp:64-70
unsafe fn load_color(ctx: *mut UniformColorCtx, rgba: &[f32; 4]) {
    use crate::src::core::SkScalar::SkScalarRoundToInt;
    unsafe {
        (*ctx).rgba[0] = SkScalarRoundToInt(rgba[0] * 255.0) as u16;
        (*ctx).r = rgba[0];
        (*ctx).rgba[1] = SkScalarRoundToInt(rgba[1] * 255.0) as u16;
        (*ctx).g = rgba[1];
        (*ctx).rgba[2] = SkScalarRoundToInt(rgba[2] * 255.0) as u16;
        (*ctx).b = rgba[2];
        (*ctx).rgba[3] = SkScalarRoundToInt(rgba[3] * 255.0) as u16;
        (*ctx).a = rgba[3];
    }
}
// Source: src/core/SkDraw_atlas.cpp:72-72
// The native namespace is this crate module, not a runtime owner.
// Source: src/core/SkDraw.h:207-208
// Source: src/core/SkDraw_atlas.cpp:73-159
impl crate::src::core::SkDraw::Draw<'_> {
    /// Safety: one exclusive operation lease covers the original Draw, pinned
    /// arena, actual TransformShader and uniform context through every blit.
    pub unsafe fn drawAtlas(
        &self,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        textures: &[SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::{
            SkArenaAlloc::{SkArenaAlloc, SkSTArenaAlloc},
            SkColor::SkColor4f,
            SkPaint::Style,
            SkRasterPipeline::SkRasterPipeline,
            SkRasterPipelineOpList::SkRasterPipelineOp as Op,
        };
        use crate::src::shaders::{
            SkShader::SkShader, SkShaderBase::SkShaderBase, SkTransformShader::SkTransformShader,
        };
        crate::SkASSERT!(@source b"src/core/SkDraw_atlas.cpp\0".as_ptr(), 78; @condition b"xform.size() == textures.size()\0".as_ptr(); (xform.len()) == (textures.len()));
        crate::SkASSERT!(@source b"src/core/SkDraw_atlas.cpp\0".as_ptr(), 79; @condition b"colors.empty() || (xform.size() == colors.size())\0".as_ptr(); colors.is_empty()||xform.len()==colors.len());
        let Some(atlas_shader) = paint.refShader() else {
            return Ok(());
        };
        if xform.is_empty() {
            return Ok(());
        }
        let mut alloc = core::pin::pin!(SkSTArenaAlloc::<256>::new(0));
        let arena = unsafe { alloc.as_mut().arena() } as *mut SkArenaAlloc;
        let mut p = paint.clone();
        p.setAntiAlias(false);
        p.setStyle(Style::kFill_Style);
        p.setShader(None);
        p.setMaskFilter(None);
        let ctm = self.fCTM.expect("native configured CTM");
        let perspective = ctm.hasPerspective();
        let transform = unsafe {
            (*arena).make_with(|_, _| SkTransformShader::new(atlas_shader.as_ref(), perspective))
        };
        let mut pipeline = unsafe { SkRasterPipeline::new(arena) };
        let default_props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        let props = self.fProps.unwrap_or(&default_props);
        let dst_cs = self.fDst.refColorSpace();
        let rec = crate::src::core::SkEffectPriv::SkStageRec {
            fPipeline: &mut pipeline,
            fAlloc: arena,
            fDstColorType: self.fDst.colorType(),
            fDstCS: dst_cs.as_ref(),
            fPaintColor: p.getColor4f(),
            fSurfaceProps: props,
            fDstBounds: SkRect::MakeEmpty(),
        };
        if !unsafe { (&*transform).appendRootStages(&rec, &SkMatrix::default()) }? {
            return Ok(());
        }
        let mut uniform_ctx = core::ptr::null_mut();
        let srgb = crate::src::core::SkColorSpace::sk_srgb_singleton();
        let steps = crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
            Some(srgb.as_ref()),
            crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType,
            self.fDst.colorSpace(),
            crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType,
        );
        if !colors.is_empty() {
            // All fields are overwritten before first execution; initialized Rust
            // storage permits references while preserving native late binding.
            uniform_ctx = unsafe {
                (*arena).make(UniformColorCtx {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                    rgba: [0; 4],
                })
            };
            unsafe {
                (*rec.fPipeline).append(Op::uniform_color_dst, uniform_ctx.cast());
            }
            let Some(mode) = blender
                .get()
                .expect("native non-null blender")
                .asBlendMode()
            else {
                return Ok(());
            };
            unsafe {
                crate::src::core::SkBlendMode::SkBlendMode_AppendStages(mode, &mut *rec.fPipeline);
            }
        }
        let mut is_opaque = colors.is_empty() && unsafe { (&*transform).isOpaque() };
        if p.getAlphaf() != 1.0 {
            let alpha = unsafe { (*arena).make(p.getAlphaf()) };
            unsafe {
                (*rec.fPipeline).append(Op::scale_1_float, alpha.cast());
            }
            is_opaque = false;
        }
        let rc = self.fRC.expect("native configured clip");
        let Some(mut blitter) = (unsafe {
            crate::src::core::SkRasterPipelineBlitter::SkCreateRasterPipelineBlitter_from_pipeline(
                &self.fDst,
                &p,
                &pipeline,
                is_opaque,
                &mut *arena,
                rc.clip_shader(),
            )
        })?
        else {
            return Ok(());
        };
        for i in 0..xform.len() {
            if !colors.is_empty() {
                let mut c4 = SkColor4f::FromColor(colors[i]);
                steps.apply(c4.vec_mut());
                unsafe {
                    load_color(uniform_ctx, c4.premul().vec());
                }
            }
            let mut mx = SkMatrix::default();
            mx.setRSXform(&xform[i]);
            mx.preTranslate(-textures[i].fLeft, -textures[i].fTop);
            mx.postConcat(ctm);
            if let Some(inverse) = mx.invert() {
                if unsafe { (&*transform).update(&inverse) } {
                    unsafe {
                        fill_rect(&mx, rc, &textures[i], Pin::new_unchecked(blitter.as_mut()));
                    }
                }
            } else {
                return Ok(());
            }
        }
        Ok(())
    }
}

// Source: src/core/SkDraw_atlas.cpp:160-160
// Closing native namespace; no runtime code.
