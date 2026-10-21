use crate::include::core::{
    SkRefCnt::{sk_sp, SkRef, SkUnref},
    SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
};
use crate::src::core::{
    SkCanvas::{PointMode, QuadAAFlags, SaveLayerStrategy, SrcRectConstraint},
    SkColor::{SkColor, SkColor4f},
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRecord::SkRecord,
    SkRecords::{self, Optional, PODArray, RecordCommand, TypedMatrix},
    SkRect::SkRect,
    SkRegion::SkRegion,
};
use crate::src::{
    core::{SkDrawable::SkDrawable, SkPicture::SkPicture},
    image::SkImage::SkImage,
};
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
use std::sync::Arc;
// Source: src/core/SkRecordCanvas.h:62-79
pub struct SkDrawableList {
    // Raw retained intrusive pointers use the actual native SkTDArray POD storage.
    fArray: crate::src::core::SkTDArray::SkTDArray<NonNull<dyn SkDrawable>>,
}
impl SkDrawableList {
    // Source: src/core/SkRecordCanvas.h:64-64
    pub fn new() -> Self {
        Self {
            fArray: crate::src::core::SkTDArray::SkTDArray::new(),
        }
    }
    // Source: src/core/SkRecordCanvas.h:67-69
    pub fn count(&self) -> i32 {
        self.fArray.size()
    }
    pub fn begin(&self) -> *const NonNull<dyn SkDrawable> {
        self.fArray.begin()
    }
    pub fn end(&self) -> *const NonNull<dyn SkDrawable> {
        self.fArray.end()
    }
    // Source: src/core/SkRecordCanvas.cpp:62-72
    pub unsafe fn newDrawableSnapshot(
        &self,
    ) -> Result<Option<Box<crate::src::core::SkBigPicture::SnapshotArray>>, crate::RasterError>
    {
        let count = self.fArray.size();
        if count == 0 {
            return Ok(None);
        }
        // Intrusive handles adopt the native released picture references exactly once.
        let mut pictures = Box::<[sk_sp<dyn SkPicture>]>::new_uninit_slice(count as usize);
        for slot in &mut pictures {
            slot.write(sk_sp::default());
        }
        let mut pictures = unsafe { pictures.assume_init() };
        for i in 0..count {
            pictures[i as usize] = unsafe {
                self.fArray.as_slice()[i as usize]
                    .as_ref()
                    .makePictureSnapshot()
            }?;
        }
        Ok(Some(Box::new(
            crate::src::core::SkBigPicture::SnapshotArray::new(pictures),
        )))
    }
    // Source: src/core/SkRecordCanvas.cpp:74-74
    pub unsafe fn append(&mut self, drawable: NonNull<dyn SkDrawable>) {
        let retained = unsafe { SkRef(drawable) };
        self.fArray.append(&[retained]);
    }
}
// Source: src/core/SkRecordCanvas.cpp:55-60
impl Drop for SkDrawableList {
    fn drop(&mut self) {
        for &p in self.fArray.as_slice() {
            unsafe { SkUnref(p) }
        }
        self.fArray.reset();
    }
}
// Source: src/core/SkRecordCanvas.cpp:78-90
pub fn safe_picture_bounds(
    bounds: &crate::src::core::SkRect::SkRect,
) -> crate::src::core::SkRect::SkIRect {
    let mut picture_bounds = bounds.roundOut();
    const SAFE_EDGE: i32 = crate::src::core::SkFloatingPoint::SK_MaxS32FitsInFloat / 2 - 1;
    const SAFE_BOUNDS: crate::src::core::SkRect::SkIRect = crate::src::core::SkRect::SkIRect {
        fLeft: -SAFE_EDGE,
        fTop: -SAFE_EDGE,
        fRight: SAFE_EDGE,
        fBottom: SAFE_EDGE,
    };
    const _: () = assert!(
        SAFE_BOUNDS.fRight - SAFE_BOUNDS.fLeft >= 0 && SAFE_BOUNDS.fBottom - SAFE_BOUNDS.fTop >= 0
    );
    if !picture_bounds.intersect(&SAFE_BOUNDS) {
        picture_bounds = crate::src::core::SkRect::SkIRect::make_ltrb(0, 0, 0, 0);
    }
    picture_bounds
}
// Source: src/core/SkRecordCanvas.h:84-85
// Source: src/core/SkRecordCanvas.h:194-197
// Actual native fields and actual inherited Canvas owner; no substitute recorder
// callback or display-command container is introduced. The complete pinned
// owner installs native state/clip and draw dispatch on its actual Canvas base.
struct RecordStorage {
    fApproxBytesUsedBySubPictures: usize,
    fRecord: *mut SkRecord,
    fDrawableList: Option<Box<SkDrawableList>>,
}
pub struct SkRecordCanvas<'device> {
    storage: core::cell::UnsafeCell<RecordStorage>,
    inherited: Pin<Box<crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'device>>>,
    pinned: core::marker::PhantomPinned,
}
impl<'device> SkRecordCanvas<'device> {
    // Source: src/core/SkRecordCanvas.cpp:92-104
    pub fn new(record: *mut SkRecord, bounds: &SkRect) -> Pin<Box<Self>> {
        Self::bind(
            crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas::with_bounds(
                &safe_picture_bounds(bounds),
            ),
            record,
        )
    }
    pub fn new_wh(record: *mut SkRecord, width: i32, height: i32) -> Pin<Box<Self>> {
        Self::bind(
            crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas::new(width, height),
            record,
        )
    }
    fn bind(
        inherited: Pin<Box<crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'device>>>,
        record: *mut SkRecord,
    ) -> Pin<Box<Self>> {
        let mut owner = Box::pin(Self {
            storage: core::cell::UnsafeCell::new(RecordStorage {
                fApproxBytesUsedBySubPictures: 0,
                fRecord: record,
                fDrawableList: None,
            }),
            inherited,
            pinned: core::marker::PhantomPinned,
        });
        let object = unsafe { owner.as_mut().get_unchecked_mut() };
        let raw = object as *mut Self;
        let hooks:*mut(dyn crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer<'device>+'device)=raw;
        unsafe {
            object
                .inherited
                .as_mut()
                .canvas()
                .get_unchecked_mut()
                .install_derived_hooks(NonNull::new_unchecked(hooks));
        }
        debug_assert!(
            object
                .inherited
                .as_ref()
                .get_ref()
                .inherited()
                .imageInfo()
                .width()
                >= 0
                && object
                    .inherited
                    .as_ref()
                    .get_ref()
                    .inherited()
                    .imageInfo()
                    .height()
                    >= 0
        );
        owner
    }

    // Source: src/core/SkRecordCanvas.h:92-96
    pub fn approxBytesUsedBySubPictures(&self) -> usize {
        unsafe { &mut *self.storage.get() }.fApproxBytesUsedBySubPictures
    }
    pub fn getDrawableList(&self) -> Option<&SkDrawableList> {
        unsafe { &*self.storage.get() }.fDrawableList.as_deref()
    }
    pub fn detachDrawableList(&mut self) -> Option<Box<SkDrawableList>> {
        unsafe { &mut *self.storage.get() }.fDrawableList.take()
    }
    // Source: src/core/SkRecordCanvas.cpp:106-111
    pub unsafe fn reset(
        &mut self,
        record: *mut SkRecord,
        bounds: &SkRect,
    ) -> Result<(), crate::RasterError> {
        self.forgetRecord();
        unsafe { &mut *self.storage.get() }.fRecord = record;
        unsafe {
            self.inherited
                .as_mut()
                .resetCanvas_rect(&safe_picture_bounds(bounds))
        }?;
        debug_assert!(
            self.inherited
                .as_ref()
                .get_ref()
                .inherited()
                .imageInfo()
                .width()
                >= 0
                && self
                    .inherited
                    .as_ref()
                    .get_ref()
                    .inherited()
                    .imageInfo()
                    .height()
                    >= 0
        );
        Ok(())
    }
    // Source: src/core/SkRecordCanvas.cpp:113-117
    pub fn forgetRecord(&self) {
        unsafe { &mut *self.storage.get() }.fDrawableList = None;
        unsafe { &mut *self.storage.get() }.fApproxBytesUsedBySubPictures = 0;
        unsafe { &mut *self.storage.get() }.fRecord = core::ptr::null_mut();
    }
    // Source: src/core/SkRecordCanvas.cpp:120-122
    unsafe fn append<T: RecordCommand>(&self, value: T) {
        unsafe {
            (*unsafe { &mut *self.storage.get() }.fRecord)
                .append::<T>()
                .write(value);
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:129-134
    unsafe fn copy<T: Clone>(&self, src: Option<&T>) -> Optional<T> {
        match src {
            None => Optional::new(),
            Some(src) => {
                let dst = unsafe { (*unsafe { &mut *self.storage.get() }.fRecord).alloc::<T>(1) };
                unsafe {
                    dst.write(src.clone());
                    Optional::from_raw(dst)
                }
            }
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:138-147
    unsafe fn copy_array<T: Clone>(&self, src: *const T, count: usize) -> PODArray<T> {
        if src.is_null() {
            return unsafe { PODArray::from_raw(core::ptr::null_mut()) };
        }
        let dst = unsafe { (*unsafe { &mut *self.storage.get() }.fRecord).alloc::<T>(count) };
        for i in 0..count {
            unsafe {
                dst.add(i).write((&*src.add(i)).clone());
            }
        }
        unsafe { PODArray::from_raw(dst) }
    }
    // Source: src/core/SkRecordCanvas.cpp:152-159
    unsafe fn copy_chars(&self, src: *const u8, count: usize) -> PODArray<u8> {
        if src.is_null() {
            return unsafe { PODArray::from_raw(core::ptr::null_mut()) };
        }
        let dst = unsafe { (*unsafe { &mut *self.storage.get() }.fRecord).alloc::<u8>(count) };
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, count);
            PODArray::from_raw(dst)
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:162-162
    unsafe fn copy_string(&self, src: *const core::ffi::c_char) -> PODArray<u8> {
        unsafe {
            self.copy_chars(
                src.cast(),
                core::ffi::CStr::from_ptr(src).to_bytes().len() + 1,
            )
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:164-166
    pub unsafe fn onDrawPaint(&self, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawPaint::new(paint.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:168-170
    pub unsafe fn onDrawBehind(&self, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawBehind::new(paint.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:172-177
    pub unsafe fn onDrawPoints(
        &self,
        mode: PointMode,
        count: usize,
        pts: *const SkPoint,
        paint: &SkPaint,
    ) {
        let points = unsafe { self.copy_array(pts, count) };
        unsafe {
            self.append(SkRecords::DrawPoints::new(
                paint.clone(),
                mode,
                count as u32,
                points,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:179-181
    pub unsafe fn onDrawRect(&self, rect: &SkRect, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawRect::new(paint.clone(), *rect));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:183-185
    pub unsafe fn onDrawRegion(&self, region: &SkRegion, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawRegion::new(paint.clone(), region.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:187-189
    pub unsafe fn onDrawOval(&self, oval: &SkRect, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawOval::new(paint.clone(), *oval));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:191-197
    pub unsafe fn onDrawArc(
        &self,
        oval: &SkRect,
        start: f32,
        sweep: f32,
        center: bool,
        paint: &SkPaint,
    ) {
        unsafe {
            self.append(SkRecords::DrawArc::new(
                paint.clone(),
                *oval,
                start,
                sweep,
                center as u32,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:199-201
    pub unsafe fn onDrawRRect(&self, rrect: &SkRRect, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawRRect::new(paint.clone(), *rrect));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:203-207
    pub unsafe fn onDrawDRRect(&self, outer: &SkRRect, inner: &SkRRect, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawDRRect::new(paint.clone(), *outer, *inner));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:209-216
    pub unsafe fn onDrawDrawable(
        &self,
        drawable: NonNull<dyn SkDrawable>,
        matrix: Option<&SkMatrix>,
    ) {
        if unsafe { &mut *self.storage.get() }.fDrawableList.is_none() {
            unsafe { &mut *self.storage.get() }.fDrawableList =
                Some(Box::new(SkDrawableList::new()));
        }
        unsafe {
            unsafe { &mut *self.storage.get() }
                .fDrawableList
                .as_mut()
                .unwrap()
                .append(drawable);
        }
        let matrix = unsafe { self.copy(matrix) };
        let bounds = unsafe { drawable.as_ref() }.getBounds();
        let index = unsafe { &mut *self.storage.get() }
            .fDrawableList
            .as_ref()
            .unwrap()
            .count()
            - 1;
        unsafe {
            self.append(SkRecords::DrawDrawable::new(matrix, bounds, index));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:218-220
    pub unsafe fn onDrawPath(&self, path: &SkPath, paint: &SkPaint) {
        unsafe {
            self.append(SkRecords::DrawPath::new(paint.clone(), path.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:222-228
    pub unsafe fn onDrawImage2(
        &self,
        image: Option<&Arc<dyn SkImage>>,
        x: f32,
        y: f32,
        sampling: &SkSamplingOptions,
        paint: Option<&SkPaint>,
    ) {
        let paint = unsafe { self.copy(paint) };
        unsafe {
            self.append(SkRecords::DrawImage::new(
                paint,
                image.cloned(),
                x,
                y,
                sampling.clone(),
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:230-238
    pub unsafe fn onDrawImageRect2(
        &self,
        image: &Arc<dyn SkImage>,
        src: &SkRect,
        dst: &SkRect,
        sampling: &SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: SrcRectConstraint,
    ) {
        let paint = unsafe { self.copy(paint) };
        unsafe {
            self.append(SkRecords::DrawImageRect::new(
                paint,
                Some(image.clone()),
                *src,
                *dst,
                sampling.clone(),
                constraint,
            ));
        }
    }
}
impl<'device> SkRecordCanvas<'device> {
    // Source: src/core/SkRecordCanvas.cpp:240-259
    pub unsafe fn onDrawImageLattice2(
        &self,
        image: &Arc<dyn SkImage>,
        lattice: &crate::src::core::SkCanvas::Lattice,
        dst: &SkRect,
        filter: SkFilterMode,
        paint: Option<&SkPaint>,
    ) {
        let count = if !lattice.fRectTypes.is_null() {
            (lattice.fXCount + 1) * (lattice.fYCount + 1)
        } else {
            0
        };
        crate::SkASSERT!(@source b"src/core/SkRecordCanvas.cpp\0".as_ptr(),246; @condition b"lattice.fBounds\0".as_ptr(); !lattice.fBounds.is_null());
        let paint = unsafe { self.copy(paint) };
        let x = unsafe { self.copy_array(lattice.fXDivs, lattice.fXCount as usize) };
        let y = unsafe { self.copy_array(lattice.fYDivs, lattice.fYCount as usize) };
        let flags = unsafe { self.copy_array(lattice.fRectTypes, count as usize) };
        let colors = unsafe { self.copy_array(lattice.fColors, count as usize) };
        unsafe {
            self.append(SkRecords::DrawImageLattice::new(
                paint,
                Some(image.clone()),
                lattice.fXCount,
                x,
                lattice.fYCount,
                y,
                count,
                flags,
                colors,
                *lattice.fBounds,
                *dst,
                filter,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:261-266
    pub unsafe fn onDrawTextBlob(
        &self,
        blob: NonNull<crate::src::core::SkTextBlob::SkTextBlob>,
        x: f32,
        y: f32,
        paint: &SkPaint,
    ) {
        let blob = unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(blob)) };
        unsafe {
            self.append(SkRecords::DrawTextBlob::new(paint.clone(), blob, x, y));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:268-270
    pub unsafe fn onDrawSlug(
        &self,
        slug: NonNull<dyn crate::include::private::chromium::Slug::Slug>,
        paint: &SkPaint,
    ) {
        let slug = unsafe { sk_sp::from_raw(SkRef(slug).as_ptr()) };
        unsafe {
            self.append(SkRecords::DrawSlug::new(paint.clone(), slug));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:272-280
    pub unsafe fn onDrawGlyphRunList(
        &self,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &SkPaint,
    ) {
        let raw = list.blob();
        let blob = if raw.is_null() {
            unsafe { list.makeBlob() }
        } else {
            unsafe {
                crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::new_unchecked(
                    raw.cast_mut(),
                )))
            }
        };
        unsafe {
            self.onDrawTextBlob(
                blob.as_ptr().expect("native blob owner"),
                list.origin().fX,
                list.origin().fY,
                paint,
            );
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:282-288
    pub unsafe fn onDrawPicture(
        &self,
        picture: NonNull<dyn SkPicture>,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
    ) {
        unsafe { &mut *self.storage.get() }.fApproxBytesUsedBySubPictures =
            unsafe { &mut *self.storage.get() }
                .fApproxBytesUsedBySubPictures
                .wrapping_add(unsafe { picture.as_ref() }.approximateBytesUsed());
        let paint = unsafe { self.copy(paint) };
        let pic = unsafe { sk_sp::from_raw(SkRef(picture).as_ptr()) };
        let matrix = TypedMatrix::new(matrix.unwrap_or(&SkMatrix::I()));
        unsafe {
            self.append(SkRecords::DrawPicture::new(paint, pic, matrix));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:290-295
    pub unsafe fn onDrawVerticesObject(
        &self,
        vertices: NonNull<crate::src::core::SkVertices::SkVertices>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) {
        let vertices = unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(vertices)) };
        unsafe {
            self.append(SkRecords::DrawVertices::new(paint.clone(), vertices, mode));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:297-301
    pub unsafe fn onDrawMesh(
        &self,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        paint: &SkPaint,
    ) {
        unsafe {
            self.append(SkRecords::DrawMesh::new(
                paint.clone(),
                mesh.clone(),
                blender,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:303-314
    pub unsafe fn onDrawPatch(
        &self,
        cubics: *const SkPoint,
        colors: *const SkColor,
        tex: *const SkPoint,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) {
        let cubics = unsafe { self.copy_array(cubics, 12) };
        let colors = unsafe { self.copy_array(colors, 4) };
        let tex = unsafe { self.copy_array(tex, 4) };
        unsafe {
            self.append(SkRecords::DrawPatch::new(
                paint.clone(),
                cubics,
                colors,
                tex,
                mode,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:316-334
    pub unsafe fn onDrawAtlas2(
        &self,
        atlas: Option<&Arc<dyn SkImage>>,
        xforms: *const crate::src::core::SkRSXform::SkRSXform,
        tex: *const SkRect,
        colors: *const SkColor,
        count: i32,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &SkSamplingOptions,
        cull: Option<&SkRect>,
        paint: Option<&SkPaint>,
    ) {
        let paint = unsafe { self.copy(paint) };
        let xforms = unsafe { self.copy_array(xforms, count as usize) };
        let tex = unsafe { self.copy_array(tex, count as usize) };
        let colors = unsafe { self.copy_array(colors, count as usize) };
        let cull = unsafe { self.copy(cull) };
        unsafe {
            self.append(SkRecords::DrawAtlas::new(
                paint,
                atlas.cloned(),
                xforms,
                tex,
                colors,
                count as u32,
                mode,
                sampling.clone(),
                cull,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:336-338
    pub unsafe fn onDrawShadowRec(
        &self,
        path: &SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) {
        unsafe {
            self.append(SkRecords::DrawShadowRec::new(path.clone(), *rec));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:340-342
    pub unsafe fn onDrawAnnotation(
        &self,
        rect: &SkRect,
        key: &core::ffi::CStr,
        value: Option<&Arc<crate::src::core::SkData::SkData>>,
    ) {
        let key = unsafe { crate::src::core::SkString::SkString::from_cstr(key.as_ptr().cast()) };
        unsafe {
            self.append(SkRecords::DrawAnnotation::new(*rect, key, value.cloned()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:344-350
    pub unsafe fn onDrawEdgeAAQuad(
        &self,
        rect: &SkRect,
        clip: *const SkPoint,
        aa: QuadAAFlags,
        color: &SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) {
        let clip = unsafe { self.copy_array(clip, 4) };
        unsafe {
            self.append(SkRecords::DrawEdgeAAQuad::new(
                *rect, clip, aa, *color, mode,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:352-374
    pub unsafe fn onDrawEdgeAAImageSet2(
        &self,
        set: &[crate::src::core::SkCanvas::ImageSetEntry],
        count: i32,
        clips: *const SkPoint,
        matrices: *const SkMatrix,
        sampling: &SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: SrcRectConstraint,
    ) {
        let (mut clip_count, mut matrix_count) = (0, 0);
        crate::src::core::SkCanvasPriv::GetDstClipAndMatrixCounts(
            &set[..count as usize],
            &mut clip_count,
            &mut matrix_count,
        );
        let mut i = 0;
        let set_copy = crate::include::private::SkTemplates::AutoTArray::with_constructor(
            count as usize,
            || {
                let entry = set[i].clone();
                i += 1;
                entry
            },
        );
        let paint = unsafe { self.copy(paint) };
        let clips = unsafe { self.copy_array(clips, clip_count as usize) };
        let matrices = unsafe { self.copy_array(matrices, matrix_count as usize) };
        unsafe {
            self.append(SkRecords::DrawEdgeAAImageSet::new(
                paint,
                set_copy,
                count,
                clips,
                matrices,
                sampling.clone(),
                constraint,
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:376-376
    pub unsafe fn willSave(&self) {
        unsafe {
            self.append(SkRecords::Save::new());
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:378-392
    pub unsafe fn getSaveLayerStrategy(
        &self,
        rec: &crate::src::core::SkCanvas::SaveLayerRec<'_>,
    ) -> SaveLayerStrategy {
        let mut i = 0;
        let filters = crate::include::private::SkTemplates::AutoTArray::with_constructor(
            rec.fFilters.len(),
            || {
                let filter = rec.fFilters[i].clone();
                i += 1;
                filter
            },
        );
        let bounds = unsafe { self.copy(rec.fBounds) };
        let paint = unsafe { self.copy(rec.fPaint) };
        unsafe {
            self.append(SkRecords::SaveLayer::new(
                bounds,
                paint,
                rec.fBackdrop.cloned(),
                rec.fSaveLayerFlags,
                crate::src::core::SkCanvasPriv::GetBackdropScaleFactor(rec),
                rec.fBackdropTileMode,
                filters,
            ));
        }
        SaveLayerStrategy::kNoLayer_SaveLayerStrategy
    }
    // Source: src/core/SkRecordCanvas.cpp:394-397
    pub unsafe fn onDoSaveBehind(&self, subset: Option<&SkRect>) -> bool {
        let subset = unsafe { self.copy(subset) };
        unsafe {
            self.append(SkRecords::SaveBehind::new(subset));
        }
        false
    }
    // Source: src/core/SkRecordCanvas.cpp:399-399
    pub unsafe fn didRestore(&self) {
        let matrix = self
            .inherited
            .as_ref()
            .get_ref()
            .inherited()
            .getTotalMatrix();
        unsafe {
            self.append(SkRecords::Restore::new(TypedMatrix::new(&matrix)));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:401-401
    pub unsafe fn didConcat44(&self, matrix: &SkM44) {
        unsafe {
            self.append(SkRecords::Concat44::new(matrix.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:403-403
    pub unsafe fn didSetM44(&self, matrix: &SkM44) {
        unsafe {
            self.append(SkRecords::SetM44::new(matrix.clone()));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:405-405
    pub unsafe fn didScale(&self, sx: f32, sy: f32) {
        unsafe {
            self.append(SkRecords::Scale::new(sx, sy));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:407-409
    pub unsafe fn didTranslate(&self, dx: f32, dy: f32) {
        unsafe {
            self.append(SkRecords::Translate::new(dx, dy));
        }
    }
}

// Rust pinned projection to the actual inherited Canvas; no new native credit.
impl<'device> SkRecordCanvas<'device> {
    pub fn canvas(self: Pin<&mut Self>) -> Pin<&mut crate::src::core::SkCanvas::SkCanvas<'device>> {
        unsafe { self.get_unchecked_mut() }
            .inherited
            .as_mut()
            .canvas()
    }
}
// Source: src/core/SkRecordCanvas.cpp:444-446
impl<'device> SkRecordCanvas<'device> {
    pub fn onNewSurface(
        &self,
        _: &crate::src::core::SkImageInfo::SkImageInfo,
        _: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> sk_sp<dyn crate::src::image::SkSurface::SkSurface<'device> + 'device> {
        sk_sp::default()
    }
}

impl Drop for SkRecordCanvas<'_> {
    fn drop(&mut self) {
        unsafe { self.inherited.as_mut().canvas().get_unchecked_mut() }.clear_derived_hooks();
    }
}
impl<'device> crate::src::core::SkCanvas::SkCanvasOwner<'device> for SkRecordCanvas<'device> {
    fn canvas(self: Pin<&mut Self>) -> Pin<&mut crate::src::core::SkCanvas::SkCanvas<'device>> {
        SkRecordCanvas::canvas(self)
    }
    fn canvas_ref(&self) -> &crate::src::core::SkCanvas::SkCanvas<'device> {
        self.inherited.as_ref().get_ref().inherited()
    }
}
use crate::include::core::{
    SkCanvasVirtualEnforcer::{SkCanvasVirtual, SkCanvasVirtualEnforcer},
    SkClipOp::SkClipOp,
};
use crate::src::core::SkCanvas::{ClipEdgeStyle, SkCanvas};
impl<'device> SkCanvasVirtual<'device> for SkRecordCanvas<'device> {
    unsafe fn onDrawSlug(
        &self,
        _: NonNull<SkCanvas<'device>>,
        slug: &dyn crate::include::private::chromium::Slug::Slug,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawSlug(self, NonNull::from(slug), paint) };
        Ok(())
    }

    unsafe fn onDrawMesh(
        &self,
        _: NonNull<SkCanvas<'device>>,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawMesh(self, mesh, blender, paint) };
        Ok(())
    }
    unsafe fn onDrawAtlas2(
        &self,
        _: NonNull<SkCanvas<'device>>,
        atlas: &Arc<dyn SkImage>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[SkRect],
        colors: &[SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &SkSamplingOptions,
        cull: Option<&SkRect>,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkRecordCanvas::onDrawAtlas2(
                self,
                Some(atlas),
                xform.as_ptr(),
                tex.as_ptr(),
                if colors.is_empty() {
                    core::ptr::null()
                } else {
                    colors.as_ptr()
                },
                xform.len() as i32,
                mode,
                sampling,
                cull,
                paint,
            )
        };
        Ok(())
    }

    unsafe fn willSave(&self, _: NonNull<SkCanvas<'device>>) {
        unsafe { SkRecordCanvas::willSave(self) }
    }
    fn getSaveLayerStrategy(
        &self,
        r: &crate::src::core::SkCanvas::SaveLayerRec<'_>,
    ) -> SaveLayerStrategy {
        unsafe { SkRecordCanvas::getSaveLayerStrategy(self, r) }
    }
    fn onDoSaveBehind(&self, r: Option<&SkRect>) -> bool {
        unsafe { SkRecordCanvas::onDoSaveBehind(self, r) }
    }
    unsafe fn didRestore(&self, _: NonNull<SkCanvas<'device>>) {
        unsafe { SkRecordCanvas::didRestore(self) }
    }
    unsafe fn didConcat44(&self, _: NonNull<SkCanvas<'device>>, m: &SkM44) {
        unsafe { SkRecordCanvas::didConcat44(self, m) }
    }
    unsafe fn didSetM44(&self, _: NonNull<SkCanvas<'device>>, m: &SkM44) {
        unsafe { SkRecordCanvas::didSetM44(self, m) }
    }
    unsafe fn didScale(&self, _: NonNull<SkCanvas<'device>>, x: f32, y: f32) {
        unsafe { SkRecordCanvas::didScale(self, x, y) }
    }
    unsafe fn didTranslate(&self, _: NonNull<SkCanvas<'device>>, x: f32, y: f32) {
        unsafe { SkRecordCanvas::didTranslate(self, x, y) }
    }
    // Source: src/core/SkRecordCanvas.cpp:411-415
    unsafe fn onClipRect(
        &self,
        c: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        op: SkClipOp,
        e: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipRect_base_raw(c, r, op, e);
            self.append(SkRecords::ClipRect::new(
                *r,
                SkRecords::ClipOpAndAA::new(op, e == ClipEdgeStyle::kSoft_ClipEdgeStyle),
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:417-421
    unsafe fn onClipRRect(
        &self,
        c: NonNull<SkCanvas<'device>>,
        r: &SkRRect,
        op: SkClipOp,
        e: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipRRect_base_raw(c, r, op, e);
            self.append(SkRecords::ClipRRect::new(
                *r,
                SkRecords::ClipOpAndAA::new(op, e == ClipEdgeStyle::kSoft_ClipEdgeStyle),
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:423-427
    unsafe fn onClipPath(
        &self,
        c: NonNull<SkCanvas<'device>>,
        p: &SkPath,
        op: SkClipOp,
        e: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipPath_base_raw(c, p, op, e);
            self.append(SkRecords::ClipPath::new(
                p.clone(),
                SkRecords::ClipOpAndAA::new(op, e == ClipEdgeStyle::kSoft_ClipEdgeStyle),
            ));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:429-432
    unsafe fn onClipShader(
        &self,
        c: NonNull<SkCanvas<'device>>,
        shader: Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: SkClipOp,
    ) {
        unsafe {
            SkCanvas::onClipShader_base_raw(c, shader.clone(), op);
            self.append(SkRecords::ClipShader::new(Some(shader), op));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:434-437
    unsafe fn onClipRegion(&self, c: NonNull<SkCanvas<'device>>, r: &SkRegion, op: SkClipOp) {
        unsafe {
            SkCanvas::onClipRegion_base_raw(c, r, op);
            self.append(SkRecords::ClipRegion::new(r.clone(), op));
        }
    }
    // Source: src/core/SkRecordCanvas.cpp:439-442
    unsafe fn onResetClip(&self, c: NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::onResetClip_base_raw(c);
            self.append(SkRecords::ResetClip::new());
        }
    }
    unsafe fn onDrawImage2(
        &self,
        _: NonNull<SkCanvas<'device>>,
        i: Option<&Arc<dyn SkImage>>,
        x: f32,
        y: f32,
        s: &SkSamplingOptions,
        p: Option<&SkPaint>,
    ) {
        unsafe { SkRecordCanvas::onDrawImage2(self, i, x, y, s, p) }
    }
    unsafe fn onDrawImageRect2(
        &self,
        _: NonNull<SkCanvas<'device>>,
        i: &Arc<dyn SkImage>,
        src: &SkRect,
        dst: &SkRect,
        s: &SkSamplingOptions,
        p: Option<&SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawImageRect2(self, i, src, dst, s, p, constraint) };
        Ok(())
    }
    unsafe fn onDrawImageLattice2(
        &self,
        _: NonNull<SkCanvas<'device>>,
        i: &Arc<dyn SkImage>,
        l: &crate::src::core::SkCanvas::Lattice,
        d: &SkRect,
        f: SkFilterMode,
        p: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawImageLattice2(self, i, l, d, f, p) };
        Ok(())
    }
    unsafe fn onDrawGlyphRunList(
        &self,
        _: NonNull<SkCanvas<'device>>,
        l: &crate::src::text::GlyphRun::GlyphRunList,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawGlyphRunList(self, l, p) };
        Ok(())
    }
    unsafe fn onDrawVerticesObject(
        &self,
        _: NonNull<SkCanvas<'device>>,
        v: &crate::src::core::SkVertices::SkVertices,
        m: crate::src::core::SkBlendMode::SkBlendMode,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawVerticesObject(self, NonNull::from(v), m, p) };
        Ok(())
    }
    unsafe fn onDrawEdgeAAImageSet2(
        &self,
        _: NonNull<SkCanvas<'device>>,
        s: &[crate::src::core::SkCanvas::ImageSetEntry],
        c: &[SkPoint],
        m: &[SkMatrix],
        sampling: &SkSamplingOptions,
        p: Option<&SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkRecordCanvas::onDrawEdgeAAImageSet2(
                self,
                s,
                s.len() as i32,
                c.as_ptr(),
                m.as_ptr(),
                sampling,
                p,
                constraint,
            )
        };
        Ok(())
    }
}
impl<'device> SkCanvasVirtualEnforcer<'device> for SkRecordCanvas<'device> {
    unsafe fn onDrawPaint(
        &self,
        _: NonNull<SkCanvas<'device>>,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawPaint(self, p) };
        Ok(())
    }
    unsafe fn onDrawBehind(
        &self,
        _: NonNull<SkCanvas<'device>>,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawBehind(self, p) };
        Ok(())
    }
    unsafe fn onDrawRect(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawRect(self, r, p) };
        Ok(())
    }
    unsafe fn onDrawRRect(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRRect,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawRRect(self, r, p) };
        Ok(())
    }
    unsafe fn onDrawDRRect(
        &self,
        _: NonNull<SkCanvas<'device>>,
        o: &SkRRect,
        i: &SkRRect,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawDRRect(self, o, i, p) };
        Ok(())
    }
    unsafe fn onDrawOval(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawOval(self, r, p) };
        Ok(())
    }
    unsafe fn onDrawArc(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        s: f32,
        w: f32,
        c: bool,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawArc(self, r, s, w, c, p) };
        Ok(())
    }
    unsafe fn onDrawPath(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkPath,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawPath(self, r, p) };
        Ok(())
    }
    unsafe fn onDrawRegion(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRegion,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawRegion(self, r, p) };
        Ok(())
    }
    unsafe fn onDrawTextBlob(
        &self,
        _: NonNull<SkCanvas<'device>>,
        b: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawTextBlob(self, NonNull::from(b), x, y, p) };
        Ok(())
    }
    unsafe fn onDrawPatch(
        &self,
        _: NonNull<SkCanvas<'device>>,
        c: &[SkPoint; 12],
        colors: Option<&[SkColor; 4]>,
        t: Option<&[SkPoint; 4]>,
        m: crate::src::core::SkBlendMode::SkBlendMode,
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkRecordCanvas::onDrawPatch(
                self,
                c.as_ptr(),
                colors.map_or(core::ptr::null(), |x| x.as_ptr()),
                t.map_or(core::ptr::null(), |x| x.as_ptr()),
                m,
                p,
            )
        };
        Ok(())
    }
    unsafe fn onDrawPoints(
        &self,
        _: NonNull<SkCanvas<'device>>,
        m: PointMode,
        pts: &[SkPoint],
        p: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawPoints(self, m, pts.len(), pts.as_ptr(), p) };
        Ok(())
    }
    unsafe fn onDrawEdgeAAQuad(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        c: Option<&[SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &SkColor4f,
        m: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkRecordCanvas::onDrawEdgeAAQuad(
                self,
                r,
                c.map_or(core::ptr::null(), |x| x.as_ptr()),
                aa,
                color,
                m,
            )
        };
        Ok(())
    }
    unsafe fn onDrawAnnotation(
        &self,
        _: NonNull<SkCanvas<'device>>,
        r: &SkRect,
        k: &core::ffi::CStr,
        v: Option<&Arc<crate::src::core::SkData::SkData>>,
    ) {
        unsafe { SkRecordCanvas::onDrawAnnotation(self, r, k, v) }
    }
    unsafe fn onDrawShadowRec(
        &self,
        _: NonNull<SkCanvas<'device>>,
        p: &SkPath,
        r: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawShadowRec(self, p, r) };
        Ok(())
    }
    unsafe fn onDrawDrawable(
        &self,
        _: NonNull<SkCanvas<'device>>,
        d: &dyn SkDrawable,
        m: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawDrawable(self, NonNull::from(d), m) };
        Ok(())
    }
    unsafe fn onDrawPicture(
        &self,
        _: NonNull<SkCanvas<'device>>,
        p: &dyn SkPicture,
        m: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkRecordCanvas::onDrawPicture(self, NonNull::from(p), m, paint) };
        Ok(())
    }
}
