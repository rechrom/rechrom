use crate::src::core::SkBlitter::SkBlitter;
use crate::src::core::SkVertState::VertState;
use crate::src::core::{
    SkArenaAlloc::SkArenaAlloc, SkColor::SkColor, SkColorData::SkPMColor4f, SkMatrix::SkMatrix,
    SkPoint::SkPoint, SkPoint3::SkPoint3, SkRasterClip::SkRasterClip,
};
use core::pin::Pin;

// Source: src/core/SkDraw_vertices.cpp:51-62
pub unsafe fn texture_to_matrix(
    state: &VertState,
    verts: &[SkPoint],
    texs: &[SkPoint],
) -> Option<SkMatrix> {
    let src = [
        verts[unsafe { state.f0.assume_init() } as usize],
        verts[unsafe { state.f1.assume_init() } as usize],
        verts[unsafe { state.f2.assume_init() } as usize],
    ];
    let dst = [
        texs[unsafe { state.f0.assume_init() } as usize],
        texs[unsafe { state.f1.assume_init() } as usize],
        texs[unsafe { state.f2.assume_init() } as usize],
    ];
    SkMatrix::PolyToPoly(&src, &dst)
}
// Source: src/core/SkDraw_vertices.cpp:73-88
/// Safety: original arena remains pinned/live through every returned-color use;
/// source colors are initialized and do not overlap the allocated destination.
unsafe fn convert_colors(
    src: &[SkColor],
    device_cs: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    alloc: &mut SkArenaAlloc,
    skip_color_xform: bool,
) -> Result<*mut SkPMColor4f, crate::RasterError> {
    use crate::include::core::{SkAlphaType::SkAlphaType, SkColorType::SkColorType};
    use crate::src::core::{SkColorSpace::SkColorSpace, SkImageInfo::SkImageInfo};
    let dst = alloc.makeArrayDefault::<SkPMColor4f>(src.len());
    let dst_cs = if skip_color_xform { None } else { device_cs };
    let src_info = SkImageInfo::MakeWithColorSpace(
        src.len() as i32,
        1,
        SkColorType::kBGRA_8888_SkColorType,
        SkAlphaType::kUnpremul_SkAlphaType,
        Some(SkColorSpace::MakeSRGB()),
    );
    let dst_info = SkImageInfo::MakeWithColorSpace(
        src.len() as i32,
        1,
        SkColorType::kRGBA_F32_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
        dst_cs,
    );
    let converted = unsafe {
        crate::src::core::SkConvertPixels::SkConvertPixels(
            &dst_info,
            dst.cast(),
            0,
            &src_info,
            src.as_ptr().cast(),
            0,
        )
    }?;
    crate::SkAssertResult!(@source b"src/core/SkDraw_vertices.cpp\0".as_ptr(), 86; @condition b"SkConvertPixels(dstInfo, dst, 0, srcInfo, src, 0)\0".as_ptr(); converted);
    Ok(dst)
}
// Source: src/core/SkDraw_vertices.cpp:90-96
fn compute_is_opaque(colors: &[SkColor]) -> bool {
    let mut c = !0u32;
    for color in colors {
        c &= *color;
    }
    (c >> 24) == 0xFF
}
// Source: src/core/SkDraw_vertices.cpp:98-104
unsafe fn fill_triangle_2(
    state: &VertState,
    blitter: Pin<&mut dyn SkBlitter>,
    rc: &SkRasterClip,
    dev2: &[SkPoint],
) {
    let tmp = [
        dev2[unsafe { state.f0.assume_init() } as usize],
        dev2[unsafe { state.f1.assume_init() } as usize],
        dev2[unsafe { state.f2.assume_init() } as usize],
    ];
    unsafe {
        crate::src::core::SkScan::SkScan::FillTriangle(&tmp, rc, blitter);
    }
}
// Source: src/core/SkDraw_vertices.cpp:106-106
const kMaxClippedTrianglePointCount: usize = 4;
// Source: src/core/SkDraw_vertices.cpp:107-176
unsafe fn fill_triangle_3(
    state: &VertState,
    mut blitter: Pin<&mut dyn SkBlitter>,
    rc: &SkRasterClip,
    dev3: &[SkPoint3],
) {
    let compute_t = |curr: f32, next: f32| {
        crate::SkASSERT!(@source b"src/core/SkDraw_vertices.cpp\0".as_ptr(), 113; @condition b"(next <= 0 && 0 < curr) || (curr <= 0 && 0 < next)\0".as_ptr(); (next<=0.0&&0.0<curr)||(curr<=0.0&&0.0<next));
        let t = curr / (curr - next);
        crate::SkASSERT!(@source b"src/core/SkDraw_vertices.cpp\0".as_ptr(), 115; @condition b"0 <= t && t <= 1\0".as_ptr(); 0.0<=t&&t<=1.0);
        t
    };
    let lerp = |curr: SkPoint3, next: SkPoint3, t: f32| curr + t * (next - curr);
    let tol = 0.05f32;
    let clip =
        |curr: SkPoint3, next: SkPoint3| lerp(curr, next, compute_t(curr.fZ - tol, next.fZ - tol));
    let clip_triangle = |dst: &mut [SkPoint; kMaxClippedTrianglePointCount],
                         idx: [usize; 3],
                         pts: &[SkPoint3]| {
        // Only the populated prefix is read, matching the native outP cursor.
        let mut out_points =
            [core::mem::MaybeUninit::<SkPoint3>::uninit(); kMaxClippedTrianglePointCount];
        let mut out_p = 0;
        for i in 0..3 {
            let curr = idx[i];
            let next = idx[(i + 1) % 3];
            if pts[curr].fZ > tol {
                out_points[out_p].write(pts[curr]);
                out_p += 1;
                if pts[next].fZ <= tol {
                    out_points[out_p].write(clip(pts[curr], pts[next]));
                    out_p += 1;
                }
            } else if pts[next].fZ > tol {
                out_points[out_p].write(clip(pts[curr], pts[next]));
                out_p += 1;
            }
        }
        let count = out_p;
        crate::SkASSERT!(@source b"src/core/SkDraw_vertices.cpp\0".as_ptr(), 156; @condition b"count == 0 || count == 3 || count == 4\0".as_ptr(); count==0||count==3||count==4);
        for i in 0..count {
            let p = unsafe { out_points[i].assume_init() };
            let scale = 1.0f32 / p.fZ;
            dst[i].set(p.fX * scale, p.fY * scale);
        }
        count
    };
    let mut tmp = [SkPoint::default(); kMaxClippedTrianglePointCount];
    let idx = [
        unsafe { state.f0.assume_init() } as usize,
        unsafe { state.f1.assume_init() } as usize,
        unsafe { state.f2.assume_init() } as usize,
    ];
    let n = clip_triangle(&mut tmp, idx, dev3);
    if n != 0 {
        crate::SkASSERT!(@source b"src/core/SkDraw_vertices.cpp\0".as_ptr(), 168; @condition b"n == 3 || n == 4\0".as_ptr(); n==3||n==4);
        let triangle = [tmp[0], tmp[1], tmp[2]];
        unsafe {
            crate::src::core::SkScan::SkScan::FillTriangle(&triangle, rc, blitter.as_mut());
        }
        if n == 4 {
            tmp[1] = tmp[2];
            tmp[2] = tmp[3];
            let triangle = [tmp[0], tmp[1], tmp[2]];
            unsafe {
                crate::src::core::SkScan::SkScan::FillTriangle(&triangle, rc, blitter.as_mut());
            }
        }
    }
}
// Source: src/core/SkDraw_vertices.cpp:178-185
unsafe fn fill_triangle(
    state: &VertState,
    blitter: Pin<&mut dyn SkBlitter>,
    rc: &SkRasterClip,
    dev2: Option<&[SkPoint]>,
    dev3: Option<&[SkPoint3]>,
) {
    if let Some(dev3) = dev3 {
        unsafe {
            fill_triangle_3(state, blitter, rc, dev3);
        }
    } else {
        unsafe {
            fill_triangle_2(
                state,
                blitter,
                rc,
                dev2.expect("native dev2 when dev3 is null"),
            );
        }
    }
}

// Source: src/core/SkDraw_vertices.cpp:188-309
impl crate::src::core::SkDraw::Draw<'_> {
    /// Safety: configured Draw and all native operation borrows remain valid;
    /// the pinned arena and its actual shader owners outlive every blit.
    // Source: src/core/SkDraw.h:238-245
    unsafe fn drawFixedVertices(
        &self,
        vertices: &crate::src::core::SkVertices::SkVertices,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
        ctm_inverse: &SkMatrix,
        dev2: Option<&[SkPoint]>,
        dev3: Option<&[SkPoint3]>,
        arena: *mut SkArenaAlloc,
        skip_color_xform: bool,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkBlendMode::SkBlendMode;
        use crate::src::shaders::{
            SkBlendShader::{BorrowedBlendStage, BorrowedShaderStage, ShaderStageView},
            SkShader::SkShader,
            SkTransformShader::SkTransformShader,
            SkTriColorShader::SkTriColorShader,
        };
        let info = vertices.priv_();
        let vertex_count = info.vertexCount();
        let index_count = info.indexCount();
        let positions =
            unsafe { core::slice::from_raw_parts(info.positions(), vertex_count as usize) };
        let mut tex = if info.texCoords().is_null() {
            None
        } else {
            Some(unsafe { core::slice::from_raw_parts(info.texCoords(), vertex_count as usize) })
        };
        let indices = if info.indices().is_null() {
            None
        } else {
            Some(unsafe { core::slice::from_raw_parts(info.indices(), index_count as usize) })
        };
        let mut colors = if info.colors().is_null() {
            None
        } else {
            Some(unsafe { core::slice::from_raw_parts(info.colors(), vertex_count as usize) })
        };
        let mut paint_shader = paint.getShader();
        if paint_shader.is_some() {
            if tex.is_none() {
                tex = Some(positions);
            }
        } else {
            tex = None;
        }
        let mut blender_is_dst = false;
        if colors.is_some() {
            if let Some(mode) = blender
                .get()
                .expect("native non-null blender")
                .asBlendMode()
            {
                match mode {
                    SkBlendMode::kSrc => colors = None,
                    SkBlendMode::kDst => {
                        blender_is_dst = true;
                        tex = None;
                        paint_shader = None;
                    }
                    _ => {}
                }
            }
        }
        debug_assert_eq!(tex.is_some(), paint_shader.is_some());
        let ctm = self.fCTM.expect("native configured CTM");
        let use_perspective = ctm.hasPerspective();
        let mut tri = None;
        let mut dst_colors = None;
        if let Some(colors) = colors {
            let dst = unsafe {
                convert_colors(
                    colors,
                    self.fDst.refColorSpace(),
                    &mut *arena,
                    skip_color_xform,
                )
            }?;
            dst_colors = Some(unsafe { core::slice::from_raw_parts(dst, vertex_count as usize) });
            tri = Some(unsafe {
                (*arena).make(SkTriColorShader::new(
                    compute_is_opaque(colors),
                    use_perspective,
                ))
            });
        }
        let mut transform = None;
        let identity = SkMatrix::default();
        let mut draw_ctm = ctm;
        if let Some(tex) = tex {
            if !core::ptr::eq(tex.as_ptr(), positions.as_ptr()) {
                let child = paint_shader.expect("native texture shader");
                let owner = unsafe {
                    (*arena).make_with(|_, _| SkTransformShader::new(child, use_perspective))
                };
                paint_shader = Some(unsafe { &*owner });
                transform = Some(owner);
                draw_ctm = &identity;
            }
        }
        // The native retained opaque-color factory is still the real factory.
        // Arena shaders are borrowed stage views, never fabricated Arc owners.
        let opaque_color = if colors.is_some() && !blender_is_dst && paint_shader.is_none() {
            crate::src::shaders::SkColorShader::Color4f(&paint.getColor4f().makeOpaque(), None)
        } else {
            None
        };
        let stage = if colors.is_none() {
            paint_shader.map(BorrowedShaderStage::Shader)
        } else if blender_is_dst {
            Some(BorrowedShaderStage::Shader(unsafe { &*tri.unwrap() }))
        } else {
            paint_shader.or(opaque_color.as_deref()).map(|src| {
                BorrowedShaderStage::Blend(BorrowedBlendStage {
                    blender: blender.get().expect("native non-null blender"),
                    dst: unsafe { &*tri.unwrap() },
                    src,
                })
            })
        };
        let mut final_paint = paint.clone();
        final_paint.setShader(None);
        let mut state = VertState::new(vertex_count, indices, index_count);
        let proc = state
            .chooseProc(info.mode())
            .expect("native valid vertex mode");
        let default_props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        let props = self.fProps.unwrap_or(&default_props);
        let rc = self.fRC.expect("native configured clip");
        let Some(mut blitter) = (unsafe {
            crate::src::core::SkRasterPipelineBlitter::SkCreateRasterPipelineBlitter_with_view(
                &self.fDst,
                &final_paint,
                draw_ctm,
                &mut *arena,
                rc.clip_shader(),
                props,
                &crate::src::core::SkRect::SkRect::MakeEmpty(),
                stage.as_ref().map(|s| s as &dyn ShaderStageView),
            )
        })?
        else {
            return Ok(());
        };
        while proc(&mut state) {
            let (i0, i1, i2) = unsafe {
                (
                    state.f0.assume_init(),
                    state.f1.assume_init(),
                    state.f2.assume_init(),
                )
            };
            if let Some(tri) = tri {
                if !unsafe {
                    (&*tri).update(ctm_inverse, positions, dst_colors.unwrap(), i0, i1, i2)
                } {
                    continue;
                }
            }
            let draw = if let Some(transform) = transform {
                if let Some(local) = unsafe { texture_to_matrix(&state, positions, tex.unwrap()) } {
                    unsafe { (&*transform).update(&SkMatrix::Concat(&local, ctm_inverse)) }
                } else {
                    false
                }
            } else {
                true
            };
            if draw {
                unsafe {
                    fill_triangle(&state, Pin::new_unchecked(blitter.as_mut()), rc, dev2, dev3);
                }
            }
        }
        Ok(())
    }
    // Source: src/core/SkDraw_vertices.cpp:311-354
    /// Safety: destination, CTM, clip and all shader operation leases are valid.
    // Source: src/core/SkDraw.h:203-206
    pub unsafe fn drawVertices(
        &self,
        vertices: &crate::src::core::SkVertices::SkVertices,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
        skip_color_xform: bool,
    ) -> Result<(), crate::RasterError> {
        let info = vertices.priv_();
        let count = info.vertexCount() as usize;
        let index_count = info.indexCount() as usize;
        let rc = self.fRC.expect("native configured clip");
        if count < 3 || (index_count > 0 && index_count < 3) || rc.is_empty() {
            return Ok(());
        }
        let ctm = self.fCTM.expect("native configured CTM");
        let Some(inverse) = ctm.invert() else {
            return Ok(());
        };
        const OUTER_SIZE: usize = core::mem::size_of::<
            crate::src::shaders::SkTriColorShader::SkTriColorShader,
        >() + (2 * core::mem::size_of::<SkPoint>()
            + core::mem::size_of::<crate::src::core::SkColor::SkColor4f>())
            * 16;
        let mut outer =
            core::pin::pin!(crate::src::core::SkArenaAlloc::SkSTArenaAlloc::<OUTER_SIZE>::new(0));
        let arena = unsafe { outer.as_mut().arena() } as *mut SkArenaAlloc;
        let positions = unsafe { core::slice::from_raw_parts(info.positions(), count) };
        let mut dev2 = None;
        let mut dev3 = None;
        if ctm.hasPerspective() {
            let points = unsafe { (*arena).makeArray::<SkPoint3>(count) };
            let points = unsafe { core::slice::from_raw_parts_mut(points, count) };
            ctm.mapPointsToHomogeneous(points, positions);
            if points
                .iter()
                .any(|p| !p.fX.is_finite() || !p.fY.is_finite() || !p.fZ.is_finite())
            {
                return Ok(());
            }
            dev3 = Some(&*points);
        } else {
            let points = unsafe { (*arena).makeArray::<SkPoint>(count) };
            let points = unsafe { core::slice::from_raw_parts_mut(points, count) };
            ctm.mapPoints(points, positions);
            if crate::src::core::SkRect::SkRect::BoundsOrEmpty(points).isEmpty() {
                return Ok(());
            }
            dev2 = Some(&*points);
        }
        unsafe {
            self.drawFixedVertices(
                vertices,
                blender,
                paint,
                &inverse,
                dev2,
                dev3,
                arena,
                skip_color_xform,
            )
        }
    }
}
