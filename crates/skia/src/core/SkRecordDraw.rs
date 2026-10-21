use crate::include::core::SkRefCnt::sk_sp;
use crate::src::core::SkBBHFactory::SkBBoxHierarchy;
use crate::src::core::{
    SkCanvas::{SkAutoCanvasRestore, SkCanvas},
    SkDrawable::SkDrawable,
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPicture::{AbortCallback, SkPicture},
    SkRecord::{RecordVisitor, SkRecord},
    SkRecords,
};
use core::{pin::Pin, ptr::NonNull};
// Source: src/core/SkRecordDraw.h:26-28
// Source: src/core/SkRecordDraw.cpp:47-91
pub unsafe fn SkRecordDraw<'device>(
    record: &SkRecord,
    canvas: NonNull<SkCanvas<'device>>,
    drawable_picts: Option<&[sk_sp<dyn SkPicture>]>,
    drawables: Option<&[NonNull<dyn SkDrawable>]>,
    drawable_count: i32,
    bbh: Option<&dyn SkBBoxHierarchy>,
    mut callback: Option<&mut dyn AbortCallback>,
) -> Result<(), crate::RasterError> {
    let mut save_restore = unsafe { SkAutoCanvasRestore::new(Some(canvas), true) };
    if let Some(bbh) = bbh {
        let query = unsafe { canvas.as_ref() }.getLocalClipBounds();
        let mut ops = Vec::new();
        bbh.search(&query, &mut ops);
        let mut draw =
            unsafe { Draw::new(canvas, drawable_picts, drawables, drawable_count, None) };
        for &op in &ops {
            if callback.as_mut().is_some_and(|callback| callback.abort()) {
                return unsafe { save_restore.restore() };
            }
            unsafe { record.visit(op, &mut draw) }?;
        }
    } else {
        let mut draw =
            unsafe { Draw::new(canvas, drawable_picts, drawables, drawable_count, None) };
        for i in 0..record.count() {
            if callback.as_mut().is_some_and(|callback| callback.abort()) {
                return unsafe { save_restore.restore() };
            }
            unsafe { record.visit(i, &mut draw) }?;
        }
    }
    unsafe { save_restore.restore() }
}
// Source: src/core/SkRecordDraw.h:33-43
// Source: src/core/SkRecordDraw.h:61-65
pub struct Draw<'a, 'device> {
    fInitialCTM: SkM44,
    fCanvas: NonNull<SkCanvas<'device>>,
    fDrawablePicts: Option<&'a [sk_sp<dyn SkPicture>]>,
    fDrawables: Option<&'a [NonNull<dyn SkDrawable>]>,
    fDrawableCount: i32,
}
impl<'a, 'device> Draw<'a, 'device> {
    // Source: src/core/SkRecordDraw.h:35-43
    pub unsafe fn new(
        canvas: NonNull<SkCanvas<'device>>,
        pictures: Option<&'a [sk_sp<dyn SkPicture>]>,
        drawables: Option<&'a [NonNull<dyn SkDrawable>]>,
        count: i32,
        initial: Option<&SkM44>,
    ) -> Self {
        Self {
            fInitialCTM: initial
                .copied()
                .unwrap_or_else(|| unsafe { canvas.as_ref() }.getLocalToDevice()),
            fCanvas: canvas,
            fDrawablePicts: pictures,
            fDrawables: drawables,
            fDrawableCount: count,
        }
    }
    // Source: src/core/SkRecordDraw.h:53-54
    pub fn drawablePicts(&self) -> Option<&[sk_sp<dyn SkPicture>]> {
        self.fDrawablePicts
    }
    pub fn drawableCount(&self) -> i32 {
        self.fDrawableCount
    }
    unsafe fn canvas_pin(&mut self) -> Pin<&mut SkCanvas<'device>> {
        unsafe { Pin::new_unchecked(self.fCanvas.as_mut()) }
    }
}
impl RecordVisitor for Draw<'_, '_> {
    type Output = Result<(), crate::RasterError>;
    // Source: src/core/SkRecordDraw.cpp:96-96
    unsafe fn NoOp(&mut self, _: &SkRecords::NoOp) -> Self::Output {
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:99-100
    unsafe fn Restore(&mut self, _: &SkRecords::Restore) -> Self::Output {
        unsafe { SkCanvas::restore_raw(self.fCanvas) }
    }
    unsafe fn Save(&mut self, _: &SkRecords::Save) -> Self::Output {
        unsafe {
            self.canvas_pin().save();
        }
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:101-110
    unsafe fn SaveLayer(&mut self, r: &SkRecords::SaveLayer) -> Self::Output {
        let rec = crate::src::core::SkCanvasPriv::ScaledBackdropLayerWithTileMode(
            unsafe { r.bounds.as_ref() },
            unsafe { r.paint.as_ref() },
            r.backdrop.as_ref(),
            *r.backdropScale,
            *r.backdropTileMode,
            *r.saveLayerFlags,
            r.filters
                .as_slice()
                .expect("native initialized filter array"),
        );
        unsafe { self.canvas_pin().saveLayer(&rec) }?;
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:112-114
    unsafe fn SaveBehind(&mut self, r: &SkRecords::SaveBehind) -> Self::Output {
        unsafe { crate::src::core::SkCanvasPriv::SaveBehind(self.fCanvas, r.subset.as_ref()) }?;
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:116-118
    unsafe fn DrawBehind(&mut self, r: &SkRecords::DrawBehind) -> Self::Output {
        unsafe { crate::src::core::SkCanvasPriv::DrawBehind(self.fCanvas, &r.paint) }
    }
    // Source: src/core/SkRecordDraw.cpp:120-125
    unsafe fn SetMatrix(&mut self, r: &SkRecords::SetMatrix) -> Self::Output {
        let m = &self.fInitialCTM.asM33() * &r.matrix.matrix;
        unsafe {
            self.canvas_pin().setMatrix(&m);
        }
        Ok(())
    }
    unsafe fn SetM44(&mut self, r: &SkRecords::SetM44) -> Self::Output {
        let m = SkM44::concat(&self.fInitialCTM, &r.matrix);
        unsafe {
            self.canvas_pin().setMatrix44(&m);
        }
        Ok(())
    }
    unsafe fn Concat44(&mut self, r: &SkRecords::Concat44) -> Self::Output {
        unsafe {
            self.canvas_pin().concat44(&r.matrix);
        }
        Ok(())
    }
    unsafe fn Concat(&mut self, r: &SkRecords::Concat) -> Self::Output {
        unsafe {
            self.canvas_pin().concat(&r.matrix.matrix);
        }
        Ok(())
    }
    unsafe fn Translate(&mut self, r: &SkRecords::Translate) -> Self::Output {
        unsafe {
            self.canvas_pin().translate(*r.dx, *r.dy);
        }
        Ok(())
    }
    unsafe fn Scale(&mut self, r: &SkRecords::Scale) -> Self::Output {
        unsafe {
            self.canvas_pin().scale(*r.sx, *r.sy);
        }
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:127-131
    unsafe fn ClipPath(&mut self, r: &SkRecords::ClipPath) -> Self::Output {
        unsafe {
            self.canvas_pin()
                .clipPath(&r.path, r.opAA.op(), r.opAA.aa());
        }
        Ok(())
    }
    unsafe fn ClipRRect(&mut self, r: &SkRecords::ClipRRect) -> Self::Output {
        unsafe {
            self.canvas_pin()
                .clipRRect(&r.rrect, r.opAA.op(), r.opAA.aa());
        }
        Ok(())
    }
    unsafe fn ClipRect(&mut self, r: &SkRecords::ClipRect) -> Self::Output {
        unsafe {
            self.canvas_pin()
                .clipRect(&r.rect, r.opAA.op(), r.opAA.aa());
        }
        Ok(())
    }
    unsafe fn ClipRegion(&mut self, r: &SkRecords::ClipRegion) -> Self::Output {
        unsafe {
            self.canvas_pin().clipRegion(&r.region, *r.op);
        }
        Ok(())
    }
    unsafe fn ClipShader(&mut self, r: &SkRecords::ClipShader) -> Self::Output {
        unsafe {
            self.canvas_pin().clipShader((*r.shader).clone(), *r.op);
        }
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:133-135
    unsafe fn ResetClip(&mut self, _: &SkRecords::ResetClip) -> Self::Output {
        unsafe { crate::src::core::SkCanvasPriv::ResetClip(self.fCanvas) };
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:137-139
    unsafe fn DrawArc(&mut self, r: &SkRecords::DrawArc) -> Self::Output {
        unsafe {
            SkCanvas::drawArc_raw(
                self.fCanvas,
                &r.oval,
                *r.startAngle,
                *r.sweepAngle,
                *r.useCenter != 0,
                &r.paint,
            )
        }
    }
    unsafe fn DrawDRRect(&mut self, r: &SkRecords::DrawDRRect) -> Self::Output {
        unsafe { SkCanvas::drawDRRect_raw(self.fCanvas, &r.outer, &r.inner, &r.paint) }
    }
    unsafe fn DrawImage(&mut self, r: &SkRecords::DrawImage) -> Self::Output {
        unsafe {
            SkCanvas::drawImage_raw(
                self.fCanvas,
                r.image.as_ref(),
                *r.left,
                *r.top,
                &r.sampling,
                r.paint.as_ref(),
            )
        }
    }
    // Source: src/core/SkRecordDraw.cpp:141-152
    unsafe fn DrawImageLattice(&mut self, r: &SkRecords::DrawImageLattice) -> Self::Output {
        let lattice = crate::src::core::SkCanvas::Lattice {
            fXCount: *r.xCount,
            fXDivs: r.xDivs.data(),
            fYCount: *r.yCount,
            fYDivs: r.yDivs.data(),
            fRectTypes: if *r.flagCount == 0 {
                core::ptr::null()
            } else {
                r.flags.data()
            },
            fColors: if *r.flagCount == 0 {
                core::ptr::null()
            } else {
                r.colors.data()
            },
            fBounds: &*r.src,
        };
        unsafe {
            SkCanvas::drawImageLattice_raw(
                self.fCanvas,
                r.image.as_ref(),
                &lattice,
                &r.dst,
                *r.filter,
                r.paint.as_ref(),
            )
        }
    }
    // Source: src/core/SkRecordDraw.cpp:154-171
    unsafe fn DrawImageRect(&mut self, r: &SkRecords::DrawImageRect) -> Self::Output {
        unsafe {
            SkCanvas::drawImageRect_raw(
                self.fCanvas,
                r.image.as_ref().expect("native retained image"),
                Some(&r.src),
                &r.dst,
                &r.sampling,
                r.paint.as_ref(),
                *r.constraint,
            )
        }
    }
    unsafe fn DrawOval(&mut self, r: &SkRecords::DrawOval) -> Self::Output {
        unsafe { SkCanvas::drawOval_raw(self.fCanvas, &r.oval, &r.paint) }
    }
    unsafe fn DrawPaint(&mut self, r: &SkRecords::DrawPaint) -> Self::Output {
        unsafe { SkCanvas::drawPaint_raw(self.fCanvas, &r.paint) }
    }
    unsafe fn DrawPath(&mut self, r: &SkRecords::DrawPath) -> Self::Output {
        unsafe { SkCanvas::drawPath_raw(self.fCanvas, &r.path, &r.paint) }
    }
    unsafe fn DrawPicture(&mut self, r: &SkRecords::DrawPicture) -> Self::Output {
        unsafe {
            SkCanvas::drawPicture_raw(
                self.fCanvas,
                r.picture.get(),
                Some(&r.matrix.matrix),
                r.paint.as_ref(),
            )
        }
    }
    unsafe fn DrawPoints(&mut self, r: &SkRecords::DrawPoints) -> Self::Output {
        let points = if *r.count == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.pts.data(), *r.count as usize) }
        };
        unsafe { SkCanvas::drawPoints_raw(self.fCanvas, *r.mode, points, &r.paint) }
    }
    unsafe fn DrawRRect(&mut self, r: &SkRecords::DrawRRect) -> Self::Output {
        unsafe { SkCanvas::drawRRect_raw(self.fCanvas, &r.rrect, &r.paint) }
    }
    unsafe fn DrawRect(&mut self, r: &SkRecords::DrawRect) -> Self::Output {
        unsafe { SkCanvas::drawRect_raw(self.fCanvas, &r.rect, &r.paint) }
    }
    unsafe fn DrawRegion(&mut self, r: &SkRecords::DrawRegion) -> Self::Output {
        unsafe { SkCanvas::drawRegion_raw(self.fCanvas, &r.region, &r.paint) }
    }
    unsafe fn DrawTextBlob(&mut self, r: &SkRecords::DrawTextBlob) -> Self::Output {
        unsafe { SkCanvas::drawTextBlob_raw(self.fCanvas, r.blob.get(), *r.x, *r.y, &r.paint) }
    }
    // Source: src/core/SkRecordDraw.cpp:166-168
    unsafe fn DrawMesh(&mut self, r: &SkRecords::DrawMesh) -> Self::Output {
        unsafe { SkCanvas::drawMesh_raw(self.fCanvas, &r.mesh, (*r.blender).clone(), &r.paint) }
    }
    unsafe fn DrawPatch(&mut self, r: &SkRecords::DrawPatch) -> Self::Output {
        unsafe {
            SkCanvas::drawPatch_raw(
                self.fCanvas,
                r.cubics
                    .data()
                    .cast::<[crate::src::core::SkPoint::SkPoint; 12]>()
                    .as_ref(),
                r.colors
                    .data()
                    .cast::<[crate::src::core::SkColor::SkColor; 4]>()
                    .as_ref(),
                r.texCoords
                    .data()
                    .cast::<[crate::src::core::SkPoint::SkPoint; 4]>()
                    .as_ref(),
                *r.bmode,
                &r.paint,
            )
        }
    }
    unsafe fn DrawAtlas(&mut self, r: &SkRecords::DrawAtlas) -> Self::Output {
        let n = *r.count as usize;
        let xforms = if n == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.xforms.data(), n) }
        };
        let tex = if n == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.texs.data(), n) }
        };
        let colors = if r.colors.data().is_null() || n == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.colors.data(), n) }
        };
        unsafe {
            SkCanvas::drawAtlas_raw(
                self.fCanvas,
                r.atlas.as_ref(),
                xforms,
                tex,
                colors,
                *r.mode,
                &r.sampling,
                r.cull.as_ref(),
                r.paint.as_ref(),
            )
        }
    }
    unsafe fn DrawShadowRec(&mut self, r: &SkRecords::DrawShadowRec) -> Self::Output {
        unsafe { SkCanvas::private_draw_shadow_rec_raw(self.fCanvas, &r.path, &r.rec) }
    }
    // Source: src/core/SkRecordDraw.cpp:164-164
    unsafe fn DrawSlug(&mut self, r: &SkRecords::DrawSlug) -> Self::Output {
        unsafe { SkCanvas::drawSlug_raw(self.fCanvas, r.slug.get(), &r.paint) }
    }
    unsafe fn DrawVertices(&mut self, r: &SkRecords::DrawVertices) -> Self::Output {
        unsafe { SkCanvas::drawVertices_sp_raw(self.fCanvas, &r.vertices, *r.bmode, &r.paint) }
    }
    unsafe fn DrawAnnotation(&mut self, r: &SkRecords::DrawAnnotation) -> Self::Output {
        let key = unsafe { core::ffi::CStr::from_ptr(r.key.c_str().cast()) };
        unsafe {
            SkCanvas::drawAnnotation_raw(self.fCanvas, &r.rect, Some(key), r.value.as_ref());
        }
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:176-180
    unsafe fn DrawEdgeAAQuad(&mut self, r: &SkRecords::DrawEdgeAAQuad) -> Self::Output {
        let clip = unsafe {
            r.clip
                .data()
                .cast::<[crate::src::core::SkPoint::SkPoint; 4]>()
                .as_ref()
        };
        unsafe {
            SkCanvas::experimental_DrawEdgeAAQuad_raw(
                self.fCanvas,
                &r.rect,
                clip,
                *r.aa,
                &r.color,
                *r.mode,
            )
        }
    }
    unsafe fn DrawEdgeAAImageSet(&mut self, r: &SkRecords::DrawEdgeAAImageSet) -> Self::Output {
        let set = r
            .set
            .as_slice()
            .expect("native initialized ImageSetEntry array");
        let (mut clips, mut matrices) = (0, 0);
        crate::src::core::SkCanvasPriv::GetDstClipAndMatrixCounts(set, &mut clips, &mut matrices);
        let clip_slice = if clips == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.dstClips.data(), clips as usize) }
        };
        let matrix_slice = if matrices == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(r.preViewMatrices.data(), matrices as usize) }
        };
        unsafe {
            SkCanvas::experimental_DrawEdgeAAImageSet_raw(
                self.fCanvas,
                set,
                clip_slice,
                matrix_slice,
                &r.sampling,
                r.paint.as_ref(),
                *r.constraint,
            )
        }
    }
    // Source: src/core/SkRecordDraw.cpp:184-193
    unsafe fn DrawDrawable(&mut self, r: &SkRecords::DrawDrawable) -> Self::Output {
        debug_assert!(*r.index >= 0);
        debug_assert!(*r.index < self.fDrawableCount);
        if let Some(drawables) = self.fDrawables {
            crate::SkASSERT!(@source b"src/core/SkRecordDraw.cpp\0".as_ptr(),186; @condition b"nullptr == fDrawablePicts\0".as_ptr(); self.fDrawablePicts.is_none());
            unsafe {
                SkCanvas::drawDrawable_raw(
                    self.fCanvas,
                    Some(drawables[*r.index as usize].as_ref()),
                    r.matrix.as_ref(),
                )
            }
        } else {
            unsafe {
                SkCanvas::drawPicture_raw(
                    self.fCanvas,
                    self.fDrawablePicts.expect("native snapshot array")[*r.index as usize].get(),
                    r.matrix.as_ref(),
                    None,
                )
            }
        }
    }
}
use crate::src::core::SkBBHFactory::Metadata;
use crate::src::core::{
    SkPaint::SkPaint,
    SkRect::SkRect,
    SkTDArray::{SkTDArray, SkTDStorage},
};
use core::mem::ManuallyDrop;
// Source: src/core/SkRecordDraw.cpp:277-283
#[derive(Clone)]
struct SaveBounds {
    controlOps: i32,
    bounds: SkRect,
    paint: *const SkPaint,
    ctm: SkMatrix,
}
// The actual SkTDStorage owns/grows/moves the same native array bytes. A narrow
// typed projection is required because SkMatrix's Cell cache forbids Rust Copy;
// native SkTDArray does byte relocation, not element destruction. No replacement
// vector, alternate growth policy, secondary matrix state or extra heap is used.
struct SaveStack(SkTDStorage);
impl SaveStack {
    fn new() -> Self {
        Self(SkTDStorage::new(core::mem::size_of::<SaveBounds>() as i32))
    }
    fn empty(&self) -> bool {
        self.0.empty()
    }
    fn size(&self) -> i32 {
        self.0.size()
    }
    unsafe fn at(&self, i: i32) -> &SaveBounds {
        unsafe { &*self.0.data().cast::<SaveBounds>().add(i as usize) }
    }
    unsafe fn at_mut(&mut self, i: i32) -> &mut SaveBounds {
        unsafe { &mut *self.0.data().cast::<SaveBounds>().add(i as usize) }
    }
    fn push(&mut self, value: SaveBounds) {
        let index = self.0.size();
        self.0.append_one();
        unsafe {
            self.0
                .data()
                .cast::<SaveBounds>()
                .add(index as usize)
                .write(value);
        }
    }
    fn pop(&mut self) -> SaveBounds {
        let i = self.0.size() - 1;
        let value = unsafe { self.at(i) }.clone();
        self.0.pop_back();
        value
    }
}
// Source: src/core/SkRecordDraw.cpp:212-239
// Source: src/core/SkRecordDraw.cpp:562-582
pub struct FillBounds {
    fCullRect: SkRect,
    fBounds: *mut SkRect,
    fMeta: *mut Metadata,
    fCurrentOp: i32,
    fCTM: SkMatrix,
    fSaveStack: ManuallyDrop<SaveStack>,
    fControlIndices: ManuallyDrop<SkTDArray<i32>>,
}
impl FillBounds {
    // Source: src/core/SkRecordDraw.cpp:214-224
    pub unsafe fn new(
        cull: &SkRect,
        _record: &SkRecord,
        bounds: *mut SkRect,
        meta: *mut Metadata,
    ) -> Self {
        let ctm = (*SkMatrix::I()).clone();
        let mut stack = SaveStack::new();
        stack.push(SaveBounds {
            controlOps: 0,
            bounds: SkRect::MakeEmpty(),
            paint: core::ptr::null(),
            ctm: ctm.clone(),
        });
        Self {
            fCullRect: *cull,
            fBounds: bounds,
            fMeta: meta,
            fCurrentOp: 0,
            fCTM: ctm,
            fSaveStack: ManuallyDrop::new(stack),
            fControlIndices: ManuallyDrop::new(SkTDArray::new()),
        }
    }
    // Source: src/core/SkRecordDraw.cpp:239-239
    pub fn setCurrentOp(&mut self, i: i32) {
        self.fCurrentOp = i;
    }
    // Source: src/core/SkRecordDraw.cpp:250-274
    unsafe fn adjustAndMap(
        &self,
        mut rect: SkRect,
        paint: Option<&SkPaint>,
    ) -> Result<SkRect, crate::RasterError> {
        rect.sort();
        if !Self::AdjustForPaint(paint, &mut rect)? {
            return Ok(self.fCullRect);
        }
        if !unsafe { self.adjustForSaveLayerPaints(&mut rect, 0) }? {
            return Ok(self.fCullRect);
        }
        rect = self.fCTM.mapRect_value(&rect);
        if !rect.intersect(&self.fCullRect) {
            return Ok(SkRect::MakeEmpty());
        }
        Ok(rect)
    }
    // Source: src/core/SkRecordDraw.cpp:332-347
    unsafe fn pushSaveBlock(
        &mut self,
        paint: Option<&SkPaint>,
        backdrop: bool,
    ) -> Result<(), crate::RasterError> {
        let full = backdrop || Self::PaintMayAffectTransparentBlack(paint)?;
        self.fSaveStack.push(SaveBounds {
            controlOps: 0,
            bounds: if full {
                self.fCullRect
            } else {
                SkRect::MakeEmpty()
            },
            paint: paint.map_or(core::ptr::null(), |p| p),
            ctm: self.fCTM.clone(),
        });
        self.pushControl();
        Ok(())
    }
    // Source: src/core/SkRecordDraw.cpp:349-388
    fn PaintMayAffectTransparentBlack(paint: Option<&SkPaint>) -> Result<bool, crate::RasterError> {
        if let Some(paint) = paint {
            if let Some(filter) = paint.getImageFilter() {
                if filter.affectsTransparentBlack()? {
                    return Ok(true);
                }
            }
            if let Some(filter) = paint.getColorFilter() {
                if filter.affectsTransparentBlack()? {
                    return Ok(true);
                }
            }
            let Some(mode) = paint.asBlendMode() else {
                return Ok(true);
            };
            use crate::src::core::SkBlendMode::SkBlendMode::*;
            if matches!(
                mode,
                kClear | kSrc | kSrcIn | kDstIn | kSrcOut | kDstATop | kModulate
            ) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    // Source: src/core/SkRecordDraw.cpp:390-406
    unsafe fn popSaveBlock(&mut self) -> SkRect {
        let mut sb = self.fSaveStack.pop();
        while sb.controlOps > 0 {
            sb.controlOps -= 1;
            unsafe {
                self.popControl(&sb.bounds);
            }
        }
        unsafe {
            self.updateSaveBounds(&sb.bounds);
        }
        sb.bounds
    }
    // Source: src/core/SkRecordDraw.cpp:408-413
    fn pushControl(&mut self) {
        self.fControlIndices.push_back(self.fCurrentOp);
        if !self.fSaveStack.empty() {
            let i = self.fSaveStack.size() - 1;
            unsafe { self.fSaveStack.at_mut(i) }.controlOps += 1;
        }
    }
    // Source: src/core/SkRecordDraw.cpp:415-419
    unsafe fn popControl(&mut self, bounds: &SkRect) {
        let i = *self.fControlIndices.back();
        unsafe {
            self.fBounds.add(i as usize).write(*bounds);
            (*self.fMeta.add(i as usize)).isDraw = false;
        }
        self.fControlIndices.pop_back();
    }
    // Source: src/core/SkRecordDraw.cpp:421-426
    unsafe fn updateSaveBounds(&mut self, bounds: &SkRect) {
        if !self.fSaveStack.empty() {
            let i = self.fSaveStack.size() - 1;
            unsafe { self.fSaveStack.at_mut(i) }.bounds.join(bounds);
        }
    }
    // Source: src/core/SkRecordDraw.cpp:316-330
    unsafe fn trackDraw(&mut self, bounds: SkRect) {
        unsafe {
            self.fBounds.add(self.fCurrentOp as usize).write(bounds);
            (*self.fMeta.add(self.fCurrentOp as usize)).isDraw = true;
            self.updateSaveBounds(&bounds);
        }
    }
    // Source: src/core/SkRecordDraw.cpp:533-545
    fn AdjustForPaint(
        paint: Option<&SkPaint>,
        rect: &mut SkRect,
    ) -> Result<bool, crate::RasterError> {
        if let Some(paint) = paint {
            if paint.canComputeFastBounds()? {
                let original = *rect;
                *rect = *paint.computeFastBounds(&original, rect)?;
                return Ok(true);
            }
            return Ok(false);
        }
        Ok(true)
    }
    // Source: src/core/SkRecordDraw.cpp:547-560
    unsafe fn adjustForSaveLayerPaints(
        &self,
        rect: &mut SkRect,
        ignore: i32,
    ) -> Result<bool, crate::RasterError> {
        let mut i = self.fSaveStack.size() - 1 - ignore;
        while i >= 0 {
            let save = unsafe { self.fSaveStack.at(i) };
            let Some(inverse) = save.ctm.invert() else {
                return Ok(false);
            };
            *rect = inverse.mapRect_value(rect);
            if !Self::AdjustForPaint(unsafe { save.paint.as_ref() }, rect)? {
                return Ok(false);
            }
            *rect = save.ctm.mapRect_value(rect);
            i -= 1;
        }
        Ok(true)
    }
}
// Source: src/core/SkRecordDraw.cpp:226-237
impl Drop for FillBounds {
    fn drop(&mut self) {
        while !self.fSaveStack.empty() {
            unsafe {
                self.popSaveBlock();
            }
        }
        while !self.fControlIndices.empty() {
            let cull = self.fCullRect;
            unsafe {
                self.popControl(&cull);
            }
        }
        unsafe {
            ManuallyDrop::drop(&mut self.fControlIndices);
            ManuallyDrop::drop(&mut self.fSaveStack);
        }
    }
}
// Source: src/core/SkRecordDraw.cpp:241-246
// Source: src/core/SkRecordDraw.cpp:285-294
trait FillBoundsOp {
    fn update_ctm(&self, _: &mut FillBounds) {}
    unsafe fn track(&self, visitor: &mut FillBounds) -> Result<(), crate::RasterError>;
}
macro_rules! fill_visitor {($($name:ident:$variant:ident),*)=>{impl RecordVisitor for FillBounds{type Output=Result<(),crate::RasterError>;$(unsafe fn $name(&mut self,op:&SkRecords::$name)->Self::Output{op.update_ctm(self);unsafe{op.track(self)}})*}};}
SkRecords::SK_RECORD_TYPES!(fill_visitor);
// Source: src/core/SkRecordDraw.cpp:297-314
macro_rules! control {($($name:ident),*)=>{$(impl FillBoundsOp for SkRecords::$name {unsafe fn track(&self,v:&mut FillBounds)->Result<(),crate::RasterError>{v.pushControl();Ok(())}})*};}
control!(ClipRect, ClipRRect, ClipPath, ClipRegion, ClipShader, ResetClip);
impl FillBoundsOp for SkRecords::Save {
    unsafe fn track(&self, v: &mut FillBounds) -> Result<(), crate::RasterError> {
        unsafe { v.pushSaveBlock(None, false) }
    }
}
impl FillBoundsOp for SkRecords::SaveBehind {
    unsafe fn track(&self, v: &mut FillBounds) -> Result<(), crate::RasterError> {
        unsafe { v.pushSaveBlock(None, false) }
    }
}
impl FillBoundsOp for SkRecords::SaveLayer {
    unsafe fn track(&self, v: &mut FillBounds) -> Result<(), crate::RasterError> {
        unsafe { v.pushSaveBlock(self.paint.as_ref(), self.backdrop.is_some()) }
    }
}
impl FillBoundsOp for SkRecords::Restore {
    fn update_ctm(&self, v: &mut FillBounds) {
        v.fCTM = self.matrix.matrix.clone();
    }
    unsafe fn track(&self, v: &mut FillBounds) -> Result<(), crate::RasterError> {
        let layer = unsafe { v.fSaveStack.at(v.fSaveStack.size() - 1) }
            .paint
            .is_null()
            == false;
        let bounds = unsafe { v.popSaveBlock() };
        unsafe {
            v.fBounds.add(v.fCurrentOp as usize).write(bounds);
            (*v.fMeta.add(v.fCurrentOp as usize)).isDraw = layer;
        }
        Ok(())
    }
}
// Source: src/core/SkRecordDraw.cpp:286-294
macro_rules! matrix_control {($name:ident,$op:ident,$v:ident,$body:block)=>{impl FillBoundsOp for SkRecords::$name{fn update_ctm(&$op,$v:&mut FillBounds)$body unsafe fn track(&self,v:&mut FillBounds)->Result<(),crate::RasterError>{v.pushControl();Ok(())}}};}
matrix_control!(SetMatrix, self, v, {
    v.fCTM = self.matrix.matrix.clone();
});
matrix_control!(SetM44, self, v, {
    v.fCTM = self.matrix.asM33();
});
matrix_control!(Concat44, self, v, {
    v.fCTM.preConcat(&self.matrix.asM33());
});
matrix_control!(Concat, self, v, {
    v.fCTM.preConcat(&self.matrix.matrix);
});
matrix_control!(Scale, self, v, {
    v.fCTM.preScale(*self.sx, *self.sy);
});
matrix_control!(Translate, self, v, {
    v.fCTM.preTranslate(*self.dx, *self.dy);
});
// Source: src/core/SkRecordDraw.cpp:428-531
macro_rules! draw_bounds {($name:ident,$op:ident,$v:ident,$body:block)=>{impl FillBoundsOp for SkRecords::$name{unsafe fn track(&$op,$v:&mut FillBounds)->Result<(),crate::RasterError>{let bounds:SkRect=$body;unsafe{$v.trackDraw(bounds);}Ok(())}}};}
draw_bounds!(DrawPaint, self, v, { v.fCullRect });
draw_bounds!(DrawBehind, self, v, { v.fCullRect });
draw_bounds!(NoOp, self, v, { SkRect::MakeEmpty() });
draw_bounds!(DrawRect, self, v, {
    unsafe { v.adjustAndMap(*self.rect, Some(&self.paint)) }?
});
draw_bounds!(DrawRegion, self, v, {
    unsafe { v.adjustAndMap(SkRect::Make(self.region.get_bounds()), Some(&self.paint)) }?
});
draw_bounds!(DrawOval, self, v, {
    unsafe { v.adjustAndMap(*self.oval, Some(&self.paint)) }?
});
draw_bounds!(DrawArc, self, v, {
    unsafe { v.adjustAndMap(*self.oval, Some(&self.paint)) }?
});
draw_bounds!(DrawRRect, self, v, {
    unsafe { v.adjustAndMap(*self.rrect.rect(), Some(&self.paint)) }?
});
draw_bounds!(DrawDRRect, self, v, {
    unsafe { v.adjustAndMap(*self.outer.rect(), Some(&self.paint)) }?
});
draw_bounds!(DrawImage, self, v, {
    let image = self.image.as_ref().expect("native retained image");
    let rect = SkRect::MakeXYWH(
        *self.left,
        *self.top,
        image.width() as f32,
        image.height() as f32,
    );
    unsafe { v.adjustAndMap(rect, self.paint.as_ref()) }?
});
draw_bounds!(DrawImageLattice, self, v, {
    unsafe { v.adjustAndMap(*self.dst, self.paint.as_ref()) }?
});
draw_bounds!(DrawImageRect, self, v, {
    unsafe { v.adjustAndMap(*self.dst, self.paint.as_ref()) }?
});
draw_bounds!(DrawPath, self, v, {
    if self.path.isInverseFillType() {
        v.fCullRect
    } else {
        unsafe { v.adjustAndMap(*self.path.getBounds(), Some(&self.paint)) }?
    }
});
draw_bounds!(DrawPoints, self, v, {
    let points = if *self.count == 0 {
        &[]
    } else {
        unsafe { core::slice::from_raw_parts(self.pts.data(), *self.count as usize) }
    };
    let mut rect = SkRect::BoundsOrEmpty(points);
    let stroke = self.paint.getStrokeWidth().max(0.01);
    rect.outset(stroke / 2.0, stroke / 2.0);
    unsafe { v.adjustAndMap(rect, Some(&self.paint)) }?
});
draw_bounds!(DrawPatch, self, v, {
    let rect =
        SkRect::BoundsOrEmpty(unsafe { core::slice::from_raw_parts(self.cubics.data(), 12) });
    unsafe { v.adjustAndMap(rect, Some(&self.paint)) }?
});
draw_bounds!(DrawVertices, self, v, {
    unsafe { v.adjustAndMap(*self.vertices.bounds(), Some(&self.paint)) }?
});
// Source: src/core/SkRecordDraw.cpp:474-476
draw_bounds!(DrawMesh, self, v, {
    unsafe { v.adjustAndMap(self.mesh.bounds(), Some(&self.paint)) }?
});
draw_bounds!(DrawAtlas, self, v, {
    if let Some(cull) = unsafe { self.cull.as_ref() } {
        unsafe { v.adjustAndMap(*cull, self.paint.as_ref()) }?
    } else {
        v.fCullRect
    }
});
draw_bounds!(DrawShadowRec, self, v, {
    let mut bounds = SkRect::MakeEmpty();
    crate::src::core::SkDrawShadowInfo::SkDrawShadowMetrics::GetLocalBounds(
        &self.path,
        &self.rec,
        &v.fCTM,
        &mut bounds,
    );
    unsafe { v.adjustAndMap(bounds, None) }?
});
draw_bounds!(DrawPicture, self, v, {
    let mut rect = self.picture.cullRect();
    rect = self.matrix.mapRect_value(&rect);
    unsafe { v.adjustAndMap(rect, self.paint.as_ref()) }?
});
draw_bounds!(DrawTextBlob, self, v, {
    let mut rect = *self.blob.bounds();
    rect.offset(*self.x, *self.y);
    unsafe { v.adjustAndMap(rect, Some(&self.paint)) }?
});
// Source: src/core/SkRecordDraw.cpp:504-507
draw_bounds!(DrawSlug, self, v, {
    unsafe { v.adjustAndMap(self.slug.sourceBoundsWithOrigin(), Some(&self.paint)) }?
});
draw_bounds!(DrawDrawable, self, v, {
    unsafe { v.adjustAndMap(*self.worstCaseBounds, None) }?
});
draw_bounds!(DrawAnnotation, self, v, {
    unsafe { v.adjustAndMap(*self.rect, None) }?
});
draw_bounds!(DrawEdgeAAQuad, self, v, {
    let rect = if self.clip.data().is_null() {
        *self.rect
    } else {
        SkRect::BoundsOrEmpty(unsafe { core::slice::from_raw_parts(self.clip.data(), 4) })
    };
    unsafe { v.adjustAndMap(rect, None) }?
});
draw_bounds!(DrawEdgeAAImageSet, self, v, {
    let mut rect = SkRect::MakeEmpty();
    let mut clip_index = 0;
    for i in 0..*self.count as usize {
        let entry = self.set.at(i);
        let mut bounds = unsafe { entry.fDstRect.assume_init() };
        if entry.fHasClip {
            bounds = SkRect::BoundsOrEmpty(unsafe {
                core::slice::from_raw_parts(self.dstClips.data().add(clip_index), 4)
            });
            clip_index += 4;
        }
        if entry.fMatrixIndex >= 0 {
            bounds = unsafe { &*self.preViewMatrices.data().add(entry.fMatrixIndex as usize) }
                .mapRect_value(&bounds);
        }
        rect.join(&unsafe { v.adjustAndMap(bounds, None) }?);
    }
    rect
});
// Source: src/core/SkRecordDraw.h:22-23
// Source: src/core/SkRecordDraw.cpp:587-595
pub unsafe fn SkRecordFillBounds(
    cull: &SkRect,
    record: &SkRecord,
    bounds: *mut SkRect,
    meta: *mut Metadata,
) -> Result<(), crate::RasterError> {
    let mut visitor = unsafe { FillBounds::new(cull, record, bounds, meta) };
    for i in 0..record.count() {
        visitor.setCurrentOp(i);
        unsafe { record.visit(i, &mut visitor) }?;
    }
    drop(visitor);
    Ok(())
}
