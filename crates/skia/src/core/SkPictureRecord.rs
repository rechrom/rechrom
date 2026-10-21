// Copyright 2011 Google Inc. BSD-style license.
use super::{
    SkCanvas::SkCanvas,
    SkData::SkData,
    SkDrawable::SkDrawable,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPicture::SkPicture,
    SkPictureFlat::*,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRect::{SkIRect, SkRect},
    SkRegion::SkRegion,
    SkTextBlob::SkTextBlob,
    SkVertices::SkVertices,
    SkWriter32::SkWriter32,
};
use crate::include::core::SkRefCnt::sk_sp;
use crate::include::private::chromium::Slug::Slug;
use crate::src::image::SkImage::SkImage;
use core::{cell::UnsafeCell, mem::ManuallyDrop, pin::Pin, ptr::NonNull};
use std::sync::Arc;
// Source: src/core/SkPictureRecord.h:59-64
pub const MASK_24: u32 = 0x00ffffff;
fn UNPACK_8_24(combined: u32) -> (u32, u32) {
    ((combined >> 24) & 0xff, combined & MASK_24)
}
fn PACK_8_24(small: u32, large: u32) -> u32 {
    (small << 24) | large
}
// Source: src/core/SkPictureRecord.h:275-277
#[derive(Default)]
pub(crate) struct PathHash;
impl super::SkTHash::NativeHash<SkPath> for PathHash {
    fn hash(&self, path: &SkPath) -> u32 {
        path.getGenerationID()
    }
}
// Source: src/core/SkPictureRecord.h:125-127
// Source: src/core/SkPictureRecord.h:273-290
pub(crate) struct RecordStorage {
    pub fRestoreOffsetStack: ManuallyDrop<Vec<i32>>,
    pub fCullOffsetStack: ManuallyDrop<Vec<u32>>,
    pub fPaints: ManuallyDrop<Vec<SkPaint>>,
    pub fPaths: ManuallyDrop<super::SkTHash::THashMap<SkPath, i32, PathHash>>,
    pub fWriter: ManuallyDrop<SkWriter32>,
    pub fImages: ManuallyDrop<Vec<Arc<dyn SkImage>>>,
    pub fPictures: ManuallyDrop<Vec<sk_sp<dyn SkPicture>>>,
    pub fDrawables: ManuallyDrop<Vec<sk_sp<dyn SkDrawable>>>,
    pub fTextBlobs: ManuallyDrop<Vec<sk_sp<SkTextBlob>>>,
    pub fVertices: ManuallyDrop<Vec<sk_sp<SkVertices>>>,
    pub fSlugs: ManuallyDrop<Vec<sk_sp<dyn Slug>>>,
    pub fRecordFlags: u32,
    pub fInitialSaveCount: i32,
}
impl Drop for RecordStorage {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fSlugs);
            ManuallyDrop::drop(&mut self.fVertices);
            ManuallyDrop::drop(&mut self.fTextBlobs);
            ManuallyDrop::drop(&mut self.fDrawables);
            ManuallyDrop::drop(&mut self.fPictures);
            ManuallyDrop::drop(&mut self.fImages);
            ManuallyDrop::drop(&mut self.fWriter);
            ManuallyDrop::drop(&mut self.fPaths);
            ManuallyDrop::drop(&mut self.fPaints);
            ManuallyDrop::drop(&mut self.fCullOffsetStack);
            ManuallyDrop::drop(&mut self.fRestoreOffsetStack);
        }
    }
}
// Complete derived owner. Its callback identity must point to this allocation;
// no secondary base, provider, or plain Canvas is substituted for this object.
pub struct SkPictureRecord<'device> {
    storage: UnsafeCell<ManuallyDrop<RecordStorage>>,
    inherited: ManuallyDrop<Pin<Box<SkCanvas<'device>>>>,
    pinned: core::marker::PhantomPinned,
}
impl<'device> SkPictureRecord<'device> {
    pub(crate) fn storage(&self) -> &RecordStorage {
        unsafe { &*self.storage.get() }
    }
    // All writer mutations require the native synchronous Canvas operation lease.
    // No storage borrow may span a Canvas call or reentrant user callback.
    unsafe fn storage_mut(&self) -> &mut RecordStorage {
        unsafe { &mut *self.storage.get() }
    }
    pub fn canvas(self: Pin<&mut Self>) -> Pin<&mut SkCanvas<'device>> {
        unsafe { self.get_unchecked_mut() }.inherited.as_mut()
    }
    // Source: src/core/SkPictureRecord.h:73-95
    pub fn getPictures(&self) -> &[sk_sp<dyn SkPicture>] {
        &self.storage().fPictures
    }
    pub fn getDrawables(&self) -> &[sk_sp<dyn SkDrawable>] {
        &self.storage().fDrawables
    }
    pub fn getTextBlobs(&self) -> &[sk_sp<SkTextBlob>] {
        &self.storage().fTextBlobs
    }
    pub fn getSlugs(&self) -> &[sk_sp<dyn Slug>] {
        &self.storage().fSlugs
    }
    pub fn getVertices(&self) -> &[sk_sp<SkVertices>] {
        &self.storage().fVertices
    }
    pub fn getImages(&self) -> &[Arc<dyn SkImage>] {
        &self.storage().fImages
    }
    // Source: src/core/SkPictureRecord.h:97-104
    pub fn opData(&self) -> Arc<SkData> {
        let storage = self.storage();
        storage.validate(storage.fWriter.bytesWritten(), 0);
        if storage.fWriter.bytesWritten() == 0 {
            SkData::MakeEmpty()
        } else {
            unsafe { storage.fWriter.snapshotAsData() }
        }
    }
    // Source: src/core/SkPictureRecord.h:106-112
    pub fn setFlags(self: Pin<&mut Self>, recordFlags: u32) {
        unsafe { self.storage_mut() }.fRecordFlags = recordFlags;
    }
    pub fn writeStream(&self) -> &SkWriter32 {
        &self.storage().fWriter
    }
}
impl Drop for SkPictureRecord<'_> {
    fn drop(&mut self) {
        unsafe {
            self.inherited
                .as_mut()
                .get_unchecked_mut()
                .clear_derived_hooks();
            // Native derived fields are destroyed before its Canvas base.
            ManuallyDrop::drop(&mut *self.storage.get());
            ManuallyDrop::drop(&mut self.inherited);
        }
    }
}
impl RecordStorage {
    // Source: src/core/SkPictureRecord.h:160-165
    fn addInt(&mut self, value: i32) {
        self.fWriter.writeInt(value);
    }
    fn addScalar(&mut self, scalar: f32) {
        self.fWriter.writeScalar(scalar);
    }
    // Source: src/core/SkPictureRecord.h:192-194
    fn validate(&self, initialOffset: usize, size: usize) {
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.h\0".as_ptr(),193; @condition b"fWriter.bytesWritten() == initialOffset + size\0".as_ptr(); self.fWriter.bytesWritten()==initialOffset+size);
    }
    // Source: src/core/SkPictureRecord.cpp:868-870
    fn addMatrix(&mut self, matrix: &SkMatrix) {
        self.fWriter.writeMatrix(matrix);
    }
    // Source: src/core/SkPictureRecord.cpp:872-879
    fn addPaintPtr(&mut self, paint: Option<&SkPaint>) {
        if let Some(paint) = paint {
            self.fPaints.push(paint.clone());
            self.addInt(self.fPaints.len() as i32);
        } else {
            self.addInt(0);
        }
    }
    // Source: src/core/SkPictureRecord.h:169-169
    fn addPaint(&mut self, paint: &SkPaint) {
        self.addPaintPtr(Some(paint));
    }
    // Source: src/core/SkPictureRecord.cpp:881-888
    fn addPathToHeap(&mut self, path: &SkPath) -> i32 {
        if let Some(n) = self.fPaths.find(path) {
            return *n;
        }
        let n = self.fPaths.count() + 1;
        self.fPaths.set(path.clone(), n);
        n
    }
    // Source: src/core/SkPictureRecord.cpp:890-892
    fn addPath(&mut self, path: &SkPath) {
        let n = self.addPathToHeap(path);
        self.addInt(n);
    }
    // Source: src/core/SkPictureRecord.cpp:894-896
    fn addPatch(&mut self, cubics: &[SkPoint; 12]) {
        unsafe {
            self.fWriter
                .write(cubics.as_ptr().cast(), 12 * core::mem::size_of::<SkPoint>());
        }
    }
    // Source: src/core/SkPictureRecord.cpp:908-910
    fn addPoint(&mut self, point: &SkPoint) {
        self.fWriter.writePoint(point);
    }
    // Source: src/core/SkPictureRecord.cpp:912-914
    fn addPoints(&mut self, pts: &[SkPoint]) {
        unsafe {
            self.fWriter
                .writeMul4(pts.as_ptr().cast(), core::mem::size_of_val(pts));
        }
    }
    // Source: src/core/SkPictureRecord.cpp:921-923
    fn addRect(&mut self, rect: &SkRect) {
        self.fWriter.writeRect(rect);
    }
    // Source: src/core/SkPictureRecord.cpp:925-929
    fn addRectPtr(&mut self, rect: Option<&SkRect>) {
        if self.fWriter.writeBool(rect.is_some()) {
            self.fWriter.writeRect(rect.unwrap());
        }
    }
    // Source: src/core/SkPictureRecord.cpp:931-933
    fn addIRect(&mut self, rect: &SkIRect) {
        unsafe {
            self.fWriter.write(
                (rect as *const SkIRect).cast(),
                core::mem::size_of::<SkIRect>(),
            );
        }
    }
    // Source: src/core/SkPictureRecord.cpp:935-939
    fn addIRectPtr(&mut self, rect: Option<&SkIRect>) {
        if self.fWriter.writeBool(rect.is_some()) {
            unsafe {
                self.fWriter
                    .reserve(core::mem::size_of::<SkIRect>())
                    .cast::<SkIRect>()
                    .write(*rect.unwrap());
            }
        }
    }
    // Source: src/core/SkPictureRecord.cpp:941-943
    fn addRRect(&mut self, rrect: &SkRRect) {
        self.fWriter.writeRRect(rrect);
    }
    // Source: src/core/SkPictureRecord.cpp:945-947
    fn addRegion(&mut self, region: &SkRegion) {
        self.fWriter.writeRegion(region);
    }
    // Source: src/core/SkPictureRecord.cpp:949-951
    fn addSampling(
        &mut self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) {
        self.fWriter.writeSampling(sampling);
    }
    // Source: src/core/SkPictureRecord.cpp:953-956
    unsafe fn addText(&mut self, text: *const u8, byteLength: usize) {
        self.addInt(byteLength as i32);
        unsafe {
            self.fWriter.writePad(text, byteLength);
        }
    }
}
// Source: src/core/SkPictureRecord.cpp:837-857
// Closures express native equals specialization and sk_ref_sp/Arc retention;
// retention executes only after the complete existing array has been searched.
fn find_or_append<R, T: ?Sized, E, F>(array: &mut Vec<R>, obj: &T, equals: E, retain: F) -> i32
where
    E: Fn(&R, &T) -> bool,
    F: FnOnce(&T) -> R,
{
    for (i, item) in array.iter().enumerate() {
        if equals(item, obj) {
            return i as i32;
        }
    }
    array.push(retain(obj));
    array.len() as i32 - 1
}
impl RecordStorage {
    // Source: src/core/SkPictureRecord.cpp:863-866
    fn addImage(&mut self, image: &Arc<dyn SkImage>) {
        let n = find_or_append(
            &mut self.fImages,
            image,
            |a, b| a.uniqueID() == b.uniqueID(),
            Arc::clone,
        );
        self.addInt(n);
    }
    // Source: src/core/SkPictureRecord.cpp:898-901
    fn addPicture(&mut self, picture: &dyn SkPicture) {
        let n = find_or_append(
            &mut self.fPictures,
            picture,
            |a, b| a.uniqueID() == b.uniqueID(),
            |p| unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::from(p))) },
        );
        self.addInt(n + 1);
    }
    // Source: src/core/SkPictureRecord.cpp:903-906
    fn addDrawable(&mut self, drawable: &dyn SkDrawable) {
        let n = find_or_append(
            &mut self.fDrawables,
            drawable,
            |a, b| core::ptr::addr_eq(&**a, b),
            |p| unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::from(p))) },
        );
        self.addInt(n + 1);
    }
    // Source: src/core/SkPictureRecord.cpp:958-961
    fn addTextBlob(&mut self, blob: &SkTextBlob) {
        let n = find_or_append(
            &mut self.fTextBlobs,
            blob,
            |a, b| a.uniqueID() == b.uniqueID(),
            |p| unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::from(p))) },
        );
        self.addInt(n + 1);
    }
    // Source: src/core/SkPictureRecord.cpp:963-966
    fn addSlug(&mut self, slug: &dyn Slug) {
        let n = find_or_append(
            &mut self.fSlugs,
            slug,
            |a, b| a.uniqueID() == b.uniqueID(),
            |p| unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::from(p))) },
        );
        self.addInt(n + 1);
    }
    // Source: src/core/SkPictureRecord.cpp:968-971
    fn addVertices(&mut self, vertices: &SkVertices) {
        let n = find_or_append(
            &mut self.fVertices,
            vertices,
            |a, b| a.uniqueID() == b.uniqueID(),
            |p| unsafe { crate::include::core::SkRefCnt::sk_ref_sp(Some(NonNull::from(p))) },
        );
        self.addInt(n + 1);
    }
    // Source: src/core/SkPictureRecord.cpp:283-300
    fn fillRestoreOffsetPlaceholdersForCurrentStackLevel(&mut self, restoreOffset: u32) {
        let mut offset = *self
            .fRestoreOffsetStack
            .last()
            .expect("native saved stack level");
        while offset > 0 {
            let peek = unsafe { *self.fWriter.readTAt::<u32>(offset as usize) };
            unsafe {
                self.fWriter.overwriteTAt(offset as usize, restoreOffset);
            }
            offset = peek as i32;
        }
        #[cfg(sk_debug)]
        if offset > 0 {
            let (op, _) = unsafe { peek_op_and_size(&self.fWriter, (-offset) as usize) };
            crate::SkASSERT!(@source b"src/core/SkPictureRecord.cpp\0".as_ptr(),297; @condition b"SAVE == drawOp || SAVE_LAYER_SAVELAYERREC == drawOp\0".as_ptr(); op==SAVE as u32||op==SAVE_LAYER_SAVELAYERREC as u32);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:314-330
    fn recordRestoreOffsetPlaceholder(&mut self) -> usize {
        if self.fRestoreOffsetStack.is_empty() {
            return usize::MAX;
        }
        let prevOffset = *self.fRestoreOffsetStack.last().unwrap();
        let offset = self.fWriter.bytesWritten();
        self.addInt(prevOffset);
        *self.fRestoreOffsetStack.last_mut().unwrap() = offset as u32 as i32;
        offset
    }
}
// Source: src/core/SkPictureRecord.cpp:184-194
#[cfg(sk_debug)]
unsafe fn peek_op_and_size(writer: &SkWriter32, offset: usize) -> (u32, u32) {
    let peek = unsafe { *writer.readTAt::<u32>(offset) };
    let (op, mut size) = UNPACK_8_24(peek);
    if size == MASK_24 {
        size = unsafe { *writer.readTAt::<u32>(offset + 4) };
    }
    (op, size)
}
impl<'device> SkPictureRecord<'device> {
    // Source: src/core/SkPictureRecord.h:141-158
    unsafe fn addDraw(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        drawType: DrawType,
        size: &mut usize,
    ) -> usize {
        let offset = self.storage().fWriter.bytesWritten();
        crate::SkASSERT_RELEASE!(@source b"src/core/SkPictureRecord.h\0".as_ptr(),144; @condition b"this->predrawNotify()\0".as_ptr(); unsafe{SkCanvas::predrawNotify_raw(canvas,false)});
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.h\0".as_ptr(),146; @condition b"0 != *size\0".as_ptr(); 0!=*size);
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.h\0".as_ptr(),147; @condition b"((uint8_t) drawType) == drawType\0".as_ptr(); drawType as u8 as u32==drawType as u32);
        let storage = unsafe { self.storage_mut() };
        if (*size & !(MASK_24 as usize)) != 0 || *size == MASK_24 as usize {
            storage
                .fWriter
                .writeInt(PACK_8_24(drawType as u32, MASK_24) as i32);
            *size += 1;
            storage.fWriter.writeInt(*size as u32 as i32);
        } else {
            storage
                .fWriter
                .writeInt(PACK_8_24(drawType as u32, *size as u32) as i32);
        }
        offset
    }
    // Source: src/core/SkPictureRecord.cpp:66-72
    unsafe fn recordSave(&self, canvas: NonNull<SkCanvas<'device>>) -> () {
        let mut size = 4;
        let offset = unsafe { self.addDraw(canvas, SAVE, &mut size) };
        self.storage().validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:214-221
    unsafe fn recordRestore(&self, canvas: NonNull<SkCanvas<'device>>, fillInSkips: bool) -> () {
        if fillInSkips {
            let storage = unsafe { self.storage_mut() };
            storage.fillRestoreOffsetPlaceholdersForCurrentStackLevel(
                storage.fWriter.bytesWritten() as u32,
            );
        }
        let mut size = 4;
        let offset = unsafe { self.addDraw(canvas, RESTORE, &mut size) };
        self.storage().validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:223-232
    unsafe fn recordTranslate(&self, canvas: NonNull<SkCanvas<'device>>, m: &SkMatrix) -> () {
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.cpp\0".as_ptr(),224; @condition b"SkMatrix::kTranslate_Mask == m.getType()\0".as_ptr(); SkMatrix::kTranslate_Mask==m.getType());
        let mut size = 12;
        let offset = unsafe { self.addDraw(canvas, TRANSLATE, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addScalar(m.getTranslateX());
        storage.addScalar(m.getTranslateY());
        storage.validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:234-243
    unsafe fn recordScale(&self, canvas: NonNull<SkCanvas<'device>>, m: &SkMatrix) -> () {
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.cpp\0".as_ptr(),235; @condition b"SkMatrix::kScale_Mask == m.getType()\0".as_ptr(); SkMatrix::kScale_Mask==m.getType());
        let mut size = 12;
        let offset = unsafe { self.addDraw(canvas, SCALE, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addScalar(m.getScaleX());
        storage.addScalar(m.getScaleY());
        storage.validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:274-281
    unsafe fn recordConcat(&self, canvas: NonNull<SkCanvas<'device>>, matrix: &SkMatrix) -> () {
        self.storage()
            .validate(self.storage().fWriter.bytesWritten(), 0);
        let mut size = 4 + super::SkMatrixPriv::SkMatrixPriv::WriteToMemory(matrix, None);
        let offset = unsafe { self.addDraw(canvas, CONCAT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addMatrix(matrix);
        storage.validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:337-352
    unsafe fn recordClipRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) -> usize {
        let mut size = 4 + core::mem::size_of::<SkRect>() + 4;
        if !self.storage().fRestoreOffsetStack.is_empty() {
            size += 4;
        }
        let initial = unsafe { self.addDraw(canvas, CLIP_RECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addRect(rect);
        storage.addInt(ClipParams_pack(op, aa) as i32);
        let offset = storage.recordRestoreOffsetPlaceholder();
        storage.validate(initial, size);
        offset
    }
    // Source: src/core/SkPictureRecord.cpp:359-373
    unsafe fn recordClipRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) -> usize {
        let mut size = 4 + SkRRect::kSizeInMemory + 4;
        if !self.storage().fRestoreOffsetStack.is_empty() {
            size += 4;
        }
        let initial = unsafe { self.addDraw(canvas, CLIP_RRECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addRRect(rect);
        storage.addInt(ClipParams_pack(op, aa) as i32);
        let offset = storage.recordRestoreOffsetPlaceholder();
        storage.validate(initial, size);
        offset
    }
    // Source: src/core/SkPictureRecord.cpp:381-395
    unsafe fn recordClipPath(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        pathID: i32,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) -> usize {
        let mut size = 12;
        if !self.storage().fRestoreOffsetStack.is_empty() {
            size += 4;
        }
        let initial = unsafe { self.addDraw(canvas, CLIP_PATH, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addInt(pathID);
        storage.addInt(ClipParams_pack(op, aa) as i32);
        let offset = storage.recordRestoreOffsetPlaceholder();
        storage.validate(initial, size);
        offset
    }
    // Source: src/core/SkPictureRecord.cpp:418-433
    unsafe fn recordClipRegion(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        region: &SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) -> usize {
        let mut size = 8 + unsafe { region.writeToMemory(core::ptr::null_mut()) };
        if !self.storage().fRestoreOffsetStack.is_empty() {
            size += 4;
        }
        let initial = unsafe { self.addDraw(canvas, CLIP_REGION, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addRegion(region);
        storage.addInt(ClipParams_pack(op, false) as i32);
        let offset = storage.recordRestoreOffsetPlaceholder();
        storage.validate(initial, size);
        offset
    }
    // Source: src/core/SkPictureRecord.cpp:448-454
    unsafe fn recordDrawPaint(&self, canvas: NonNull<SkCanvas<'device>>, paint: &SkPaint) -> () {
        let mut size = 8;
        let initial = unsafe { self.addDraw(canvas, DRAW_PAINT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.validate(initial, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:456-463
    unsafe fn recordDrawBehind(&self, canvas: NonNull<SkCanvas<'device>>, paint: &SkPaint) -> () {
        let mut size = 8;
        let initial = unsafe { self.addDraw(canvas, DRAW_BEHIND_PAINT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.validate(initial, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:465-476
    unsafe fn recordDrawPoints(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        mode: super::SkCanvas::PointMode,
        pts: &[SkPoint],
        paint: &SkPaint,
    ) -> () {
        let mut size = 16 + core::mem::size_of_val(pts);
        let initial = unsafe { self.addDraw(canvas, DRAW_POINTS, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addInt(mode as i32);
        storage.addInt(pts.len() as i32);
        storage.addPoints(pts);
        storage.validate(initial, size);
        ()
    }
}
// Source: src/core/SkPictureRecord.cpp:478-654
impl<'device> SkPictureRecord<'device> {
    unsafe fn recordDrawOval(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        oval: &SkRect,
        paint: &SkPaint,
    ) -> () {
        let mut size = 8 + core::mem::size_of::<SkRect>();
        let offset = unsafe { self.addDraw(canvas, DRAW_OVAL, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addRect(oval);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawArc(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        oval: &SkRect,
        start: f32,
        sweep: f32,
        center: bool,
        paint: &SkPaint,
    ) -> () {
        let mut size = 8 + core::mem::size_of::<SkRect>() + 12;
        let offset = unsafe { self.addDraw(canvas, DRAW_ARC, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addRect(oval);
        storage.addScalar(start);
        storage.addScalar(sweep);
        storage.addInt(center as i32);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        paint: &SkPaint,
    ) -> () {
        let mut size = 8 + core::mem::size_of::<SkRect>();
        let offset = unsafe { self.addDraw(canvas, DRAW_RECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addRect(rect);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawRegion(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        region: &SkRegion,
        paint: &SkPaint,
    ) -> () {
        let regionBytes = unsafe { region.writeToMemory(core::ptr::null_mut()) };
        let mut size = 8 + regionBytes;
        let offset = unsafe { self.addDraw(canvas, DRAW_REGION, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.fWriter.writeRegion(region);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRRect,
        paint: &SkPaint,
    ) -> () {
        let mut size = 8 + SkRRect::kSizeInMemory;
        let offset = unsafe { self.addDraw(canvas, DRAW_RRECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addRRect(rect);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawDRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        outer: &SkRRect,
        inner: &SkRRect,
        paint: &SkPaint,
    ) -> () {
        let mut size = 8 + 2 * SkRRect::kSizeInMemory;
        let offset = unsafe { self.addDraw(canvas, DRAW_DRRECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addRRect(outer);
        storage.addRRect(inner);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawPath(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        path: &SkPath,
        paint: &SkPaint,
    ) -> () {
        let mut size = 12;
        let offset = unsafe { self.addDraw(canvas, DRAW_PATH, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addPath(path);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawImage2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: &Arc<dyn SkImage>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
    ) -> () {
        let mut size = 20 + super::SkSamplingPriv::SkSamplingPriv::FlatSize(sampling);
        let offset = unsafe { self.addDraw(canvas, DRAW_IMAGE2, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaintPtr(paint);
        storage.addImage(image);
        storage.addScalar(x);
        storage.addScalar(y);
        storage.addSampling(sampling);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawImageRect2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: &Arc<dyn SkImage>,
        src: &SkRect,
        dst: &SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: super::SkCanvas::SrcRectConstraint,
    ) -> () {
        let mut size = 16
            + 2 * core::mem::size_of::<SkRect>()
            + super::SkSamplingPriv::SkSamplingPriv::FlatSize(sampling);
        let offset = unsafe { self.addDraw(canvas, DRAW_IMAGE_RECT2, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaintPtr(paint);
        storage.addImage(image);
        storage.addRect(src);
        storage.addRect(dst);
        storage.addSampling(sampling);
        storage.addInt(constraint as i32);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawImageLattice2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: &Arc<dyn SkImage>,
        lattice: &super::SkCanvas::Lattice,
        dst: &SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&SkPaint>,
    ) -> () {
        let latticeSize =
            unsafe { super::SkCanvasPriv::WriteLattice(core::ptr::null_mut(), lattice) };
        let mut size = 16 + latticeSize + core::mem::size_of::<SkRect>();
        let offset = unsafe { self.addDraw(canvas, DRAW_IMAGE_LATTICE2, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaintPtr(paint);
        storage.addImage(image);
        unsafe {
            super::SkCanvasPriv::WriteLattice(
                storage.fWriter.reservePad(latticeSize).cast(),
                lattice,
            );
        }
        storage.addRect(dst);
        storage.addInt(filter as i32);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawTextBlob(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        blob: &SkTextBlob,
        x: f32,
        y: f32,
        paint: &SkPaint,
    ) -> () {
        let mut size = 20;
        let offset = unsafe { self.addDraw(canvas, DRAW_TEXT_BLOB, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addTextBlob(blob);
        storage.addScalar(x);
        storage.addScalar(y);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawSlug(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        slug: &dyn Slug,
        paint: &SkPaint,
    ) -> () {
        let mut size = 12;
        let offset = unsafe { self.addDraw(canvas, DRAW_SLUG, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addSlug(slug);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawPicture(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        picture: &dyn SkPicture,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
    ) -> () {
        let mut size = 8;
        let offset;
        if matrix.is_none() && paint.is_none() {
            offset = unsafe { self.addDraw(canvas, DRAW_PICTURE, &mut size) };
            unsafe { self.storage_mut() }.addPicture(picture);
        } else {
            let identity = SkMatrix::default();
            let m = matrix.unwrap_or(&identity);
            size += super::SkMatrixPriv::SkMatrixPriv::WriteToMemory(m, None) + 4;
            offset = unsafe { self.addDraw(canvas, DRAW_PICTURE_MATRIX_PAINT, &mut size) };
            let storage = unsafe { self.storage_mut() };
            storage.addPaintPtr(paint);
            storage.addMatrix(m);
            storage.addPicture(picture);
        }
        self.storage().validate(offset, size);
        ()
    }
    unsafe fn recordDrawDrawable(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        drawable: &dyn SkDrawable,
        matrix: Option<&SkMatrix>,
    ) -> () {
        let mut size = 8;
        let offset;
        if let Some(matrix) = matrix {
            size += super::SkMatrixPriv::SkMatrixPriv::WriteToMemory(matrix, None);
            offset = unsafe { self.addDraw(canvas, DRAW_DRAWABLE_MATRIX, &mut size) };
            let storage = unsafe { self.storage_mut() };
            storage.addMatrix(matrix);
            storage.addDrawable(drawable);
        } else {
            offset = unsafe { self.addDraw(canvas, DRAW_DRAWABLE, &mut size) };
            unsafe { self.storage_mut() }.addDrawable(drawable);
        }
        self.storage().validate(offset, size);
        ()
    }
}
// Source: src/core/SkPictureRecord.cpp:656-831
impl<'device> SkPictureRecord<'device> {
    unsafe fn recordDrawVerticesObject(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        vertices: &SkVertices,
        mode: super::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) -> () {
        let mut size = 20;
        let offset = unsafe { self.addDraw(canvas, DRAW_VERTICES_OBJECT, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addVertices(vertices);
        storage.addInt(0);
        storage.addInt(mode as i32);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawPatch(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        cubics: &[SkPoint; 12],
        colors: Option<&[u32; 4]>,
        tex: Option<&[SkPoint; 4]>,
        mode: super::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) -> () {
        let mut size = 12 + 12 * core::mem::size_of::<SkPoint>();
        let mut flags = 0;
        if colors.is_some() {
            flags |= DRAW_VERTICES_HAS_COLORS;
            size += 16;
        }
        if tex.is_some() {
            flags |= DRAW_VERTICES_HAS_TEXS;
            size += 4 * core::mem::size_of::<SkPoint>();
        }
        if mode != super::SkBlendMode::SkBlendMode::kModulate {
            flags |= DRAW_VERTICES_HAS_XFER;
            size += 4;
        }
        let offset = unsafe { self.addDraw(canvas, DRAW_PATCH, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaint(paint);
        storage.addPatch(cubics);
        storage.addInt(flags as i32);
        if let Some(colors) = colors {
            unsafe {
                storage.fWriter.write(colors.as_ptr().cast(), 16);
            }
        }
        if let Some(tex) = tex {
            unsafe {
                storage
                    .fWriter
                    .write(tex.as_ptr().cast(), core::mem::size_of_val(tex));
            }
        }
        if flags & DRAW_VERTICES_HAS_XFER != 0 {
            storage.addInt(mode as i32);
        }
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawAtlas2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        atlas: &Arc<dyn SkImage>,
        xform: &[super::SkRSXform::SkRSXform],
        tex: &[SkRect],
        colors: Option<&[u32]>,
        mode: super::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&SkRect>,
        paint: Option<&SkPaint>,
    ) -> () {
        let count = xform.len();
        let mut size = 20
            + count * core::mem::size_of::<super::SkRSXform::SkRSXform>()
            + count * core::mem::size_of::<SkRect>()
            + super::SkSamplingPriv::SkSamplingPriv::FlatSize(sampling);
        let mut flags = 0;
        if colors.is_some() {
            flags |= DRAW_ATLAS_HAS_COLORS;
            size += count * 4 + 4;
        }
        if cull.is_some() {
            flags |= DRAW_ATLAS_HAS_CULL;
            size += core::mem::size_of::<SkRect>();
        }
        flags |= DRAW_ATLAS_HAS_SAMPLING;
        let offset = unsafe { self.addDraw(canvas, DRAW_ATLAS, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPaintPtr(paint);
        storage.addImage(atlas);
        storage.addInt(flags as i32);
        storage.addInt(count as i32);
        unsafe {
            storage.fWriter.write(
                xform.as_ptr().cast(),
                count * core::mem::size_of::<super::SkRSXform::SkRSXform>(),
            );
            storage
                .fWriter
                .write(tex.as_ptr().cast(), count * core::mem::size_of::<SkRect>());
        }
        if let Some(colors) = colors {
            unsafe {
                storage.fWriter.write(colors.as_ptr().cast(), count * 4);
            }
            storage.addInt(mode as i32);
        }
        if let Some(cull) = cull {
            unsafe {
                storage.fWriter.write(
                    (cull as *const SkRect).cast(),
                    core::mem::size_of::<SkRect>(),
                );
            }
        }
        storage.addSampling(sampling);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawShadowRec(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        path: &SkPath,
        rec: &super::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> () {
        let mut size = 8 + 2 * core::mem::size_of::<super::SkPoint3::SkPoint3>() + 4 + 12;
        let offset = unsafe { self.addDraw(canvas, DRAW_SHADOW_REC, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addPath(path);
        storage.fWriter.writePoint3(&rec.fZPlaneParams);
        storage.fWriter.writePoint3(&rec.fLightPos);
        storage.fWriter.writeScalar(rec.fLightRadius);
        storage.fWriter.write32(rec.fAmbientColor as i32);
        storage.fWriter.write32(rec.fSpotColor as i32);
        storage.fWriter.write32(rec.fFlags as i32);
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawAnnotation(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        key: &core::ffi::CStr,
        value: Option<&Arc<SkData>>,
    ) -> () {
        let keyLen = unsafe { SkWriter32::WriteStringSize(key.as_ptr(), usize::MAX) };
        let valueLen = SkWriter32::WriteDataSize(value.map(|v| &**v));
        let mut size = 4 + core::mem::size_of::<SkRect>() + keyLen + valueLen;
        let offset = unsafe { self.addDraw(canvas, DRAW_ANNOTATION, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addRect(rect);
        unsafe {
            storage.fWriter.writeString(key.as_ptr(), usize::MAX);
        }
        storage.fWriter.writeData(value.map(|v| &**v));
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawEdgeAAQuad(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        clip: Option<&[SkPoint; 4]>,
        aa: super::SkCanvas::QuadAAFlags,
        color: &super::SkColor::SkColor4f,
        mode: super::SkBlendMode::SkBlendMode,
    ) -> () {
        let mut size = 16
            + core::mem::size_of::<super::SkColor::SkColor4f>()
            + core::mem::size_of::<SkRect>()
            + if clip.is_some() {
                4 * core::mem::size_of::<SkPoint>()
            } else {
                0
            };
        let offset = unsafe { self.addDraw(canvas, DRAW_EDGEAA_QUAD, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addRect(rect);
        storage.addInt(aa.0 as i32);
        unsafe {
            storage.fWriter.write(
                (color as *const super::SkColor::SkColor4f).cast(),
                core::mem::size_of_val(color),
            );
        }
        storage.addInt(mode as i32);
        storage.addInt(clip.is_some() as i32);
        if let Some(clip) = clip {
            storage.addPoints(clip);
        }
        storage.validate(offset, size);
        ()
    }
    unsafe fn recordDrawEdgeAAImageSet2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        set: &[super::SkCanvas::ImageSetEntry],
        dstClips: &[SkPoint],
        preViewMatrices: &[SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: super::SkCanvas::SrcRectConstraint,
    ) -> () {
        let (mut totalDstClipCount, mut totalMatrixCount) = (0, 0);
        super::SkCanvasPriv::GetDstClipAndMatrixCounts(
            set,
            &mut totalDstClipCount,
            &mut totalMatrixCount,
        );
        let mut size = 24
            + core::mem::size_of::<SkPoint>() * totalDstClipCount as usize
            + 36 * totalMatrixCount as usize
            + (16 + 2 * core::mem::size_of::<SkRect>() + 4) * set.len()
            + super::SkSamplingPriv::SkSamplingPriv::FlatSize(sampling);
        let offset = unsafe { self.addDraw(canvas, DRAW_EDGEAA_IMAGE_SET2, &mut size) };
        let storage = unsafe { self.storage_mut() };
        storage.addInt(set.len() as i32);
        storage.addPaintPtr(paint);
        storage.addSampling(sampling);
        storage.addInt(constraint as i32);
        for entry in set {
            storage.addImage(
                entry
                    .fImage
                    .as_ref()
                    .expect("native nonnull image set entry"),
            );
            storage.addRect(unsafe { entry.fSrcRect.assume_init_ref() });
            storage.addRect(unsafe { entry.fDstRect.assume_init_ref() });
            storage.addInt(entry.fMatrixIndex);
            storage.addScalar(entry.fAlpha);
            storage.addInt(entry.fAAFlags as i32);
            storage.addInt(entry.fHasClip as i32);
        }
        storage.addInt(totalDstClipCount);
        storage.addPoints(&dstClips[..totalDstClipCount as usize]);
        storage.addInt(totalMatrixCount);
        for i in 0..totalMatrixCount {
            storage.addMatrix(&preViewMatrices[i as usize]);
        }
        storage.validate(offset, size);
        ()
    }
    // Source: src/core/SkPictureRecord.cpp:916-919
    unsafe fn addNoOp(&self, canvas: NonNull<SkCanvas<'device>>) -> () {
        let mut size = 4;
        unsafe { self.addDraw(canvas, NOOP, &mut size) };
        ()
    }
}
impl<'device> SkPictureRecord<'device> {
    fn base_raw(&self) -> NonNull<SkCanvas<'device>> {
        NonNull::from(self.inherited.as_ref().get_ref())
    }
    // Source: src/core/SkPictureRecord.cpp:48-52
    pub fn new(dimensions: &SkIRect, flags: u32) -> Pin<Box<Self>> {
        let storage = RecordStorage {
            fRestoreOffsetStack: ManuallyDrop::new(Vec::new()),
            fCullOffsetStack: ManuallyDrop::new(Vec::new()),
            fPaints: ManuallyDrop::new(Vec::new()),
            fPaths: ManuallyDrop::new(Default::default()),
            fWriter: ManuallyDrop::new(unsafe { SkWriter32::new(core::ptr::null_mut(), 0) }),
            fImages: ManuallyDrop::new(Vec::new()),
            fPictures: ManuallyDrop::new(Vec::new()),
            fDrawables: ManuallyDrop::new(Vec::new()),
            fTextBlobs: ManuallyDrop::new(Vec::new()),
            fVertices: ManuallyDrop::new(Vec::new()),
            fSlugs: ManuallyDrop::new(Vec::new()),
            fRecordFlags: flags,
            fInitialSaveCount: -1,
        };
        let mut owner = Box::pin(Self {
            storage: UnsafeCell::new(ManuallyDrop::new(storage)),
            inherited: ManuallyDrop::new(SkCanvas::with_bounds(dimensions)),
            pinned: core::marker::PhantomPinned,
        });
        let object = unsafe { owner.as_mut().get_unchecked_mut() };
        let concrete = NonNull::from(&mut *object);
        let hooks:*mut(dyn crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer<'device>+'device)=concrete.as_ptr();
        unsafe {
            object
                .inherited
                .as_mut()
                .get_unchecked_mut()
                .install_derived_hooks(NonNull::new_unchecked(hooks));
        }
        owner
    }
    // Source: src/core/SkPictureRecord.cpp:54-55
    pub fn with_size(
        dimensions: &crate::include::core::SkSize::SkISize,
        flags: u32,
    ) -> Pin<Box<Self>> {
        Self::new(
            &SkIRect::make_wh(dimensions.width(), dimensions.height()),
            flags,
        )
    }
    // Source: src/core/SkPictureRecord.cpp:302-307
    pub unsafe fn beginRecording(self: Pin<&mut Self>) -> Result<(), crate::RasterError> {
        let owner = NonNull::from(unsafe { self.get_unchecked_mut() });
        let canvas = unsafe { owner.as_ref() }.base_raw();
        let count = unsafe { Pin::new_unchecked(&mut *canvas.as_ptr()).save() };
        unsafe { owner.as_ref().storage_mut() }.fInitialSaveCount = count;
        Ok(())
    }
    // Source: src/core/SkPictureRecord.cpp:309-312
    pub unsafe fn endRecording(self: Pin<&mut Self>) -> Result<(), crate::RasterError> {
        let owner = NonNull::from(unsafe { self.get_unchecked_mut() });
        let count = unsafe { owner.as_ref() }.storage().fInitialSaveCount;
        crate::SkASSERT!(@source b"src/core/SkPictureRecord.cpp\0".as_ptr(),310; @condition b"kNoInitialSave != fInitialSaveCount\0".as_ptr(); count!=-1);
        let canvas = unsafe { owner.as_ref() }.base_raw();
        unsafe { SkCanvas::restoreToCount_raw(canvas, count) }
    }
    // Source: src/core/SkPictureRecord.cpp:109-178
    unsafe fn recordSaveLayer(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rec: &super::SkCanvas::SaveLayerRec<'_>,
    ) {
        unsafe {
            let mut size = 8;
            let mut flags = 0;
            let filterCount = rec.fFilters.len() as u32;
            if rec.fBounds.is_some() {
                flags |= SAVELAYERREC_HAS_BOUNDS;
                size += core::mem::size_of::<SkRect>();
            }
            if rec.fPaint.is_some() {
                flags |= SAVELAYERREC_HAS_PAINT;
                size += 4;
            }
            if rec.fBackdrop.is_some() {
                flags |= SAVELAYERREC_HAS_BACKDROP;
                size += 4;
            }
            if rec.fSaveLayerFlags != 0 {
                flags |= SAVELAYERREC_HAS_FLAGS;
                size += 4;
            }
            if super::SkCanvasPriv::GetBackdropScaleFactor(rec) != 1.0 {
                flags |= SAVELAYERREC_HAS_BACKDROP_SCALE;
                size += 4;
            }
            if filterCount != 0 {
                flags |= SAVELAYERREC_HAS_MULTIPLE_FILTERS;
                size += 4 + 4 * filterCount as usize;
            }
            if rec.fBackdropTileMode != crate::include::core::SkTileMode::SkTileMode::kClamp {
                flags |= SAVELAYERREC_HAS_BACKDROP_TILEMODE;
                size += 4;
            }
            let offset = self.addDraw(canvas, SAVE_LAYER_SAVELAYERREC, &mut size);
            let storage = self.storage_mut();
            storage.addInt(flags as i32);
            if flags & SAVELAYERREC_HAS_BOUNDS != 0 {
                storage.addRect(rec.fBounds.unwrap());
            }
            if flags & SAVELAYERREC_HAS_PAINT != 0 {
                storage.addPaintPtr(rec.fPaint);
            }
            if flags & SAVELAYERREC_HAS_BACKDROP != 0 {
                let mut paint = SkPaint::default();
                paint.setImageFilter(rec.fBackdrop.cloned());
                storage.addPaint(&paint);
            }
            if flags & SAVELAYERREC_HAS_FLAGS != 0 {
                storage.addInt(rec.fSaveLayerFlags as i32);
            }
            if flags & SAVELAYERREC_HAS_BACKDROP_SCALE != 0 {
                storage.addScalar(super::SkCanvasPriv::GetBackdropScaleFactor(rec));
            }
            if flags & SAVELAYERREC_HAS_MULTIPLE_FILTERS != 0 {
                storage.addInt(filterCount as i32);
                for filter in rec.fFilters {
                    let mut paint = SkPaint::default();
                    paint.setImageFilter(filter.clone());
                    storage.addPaint(&paint);
                }
            }
            if flags & SAVELAYERREC_HAS_BACKDROP_TILEMODE != 0 {
                storage.addInt(rec.fBackdropTileMode as i32);
            }
            storage.validate(offset, size);
        }
    }
}
impl<'device> super::SkCanvas::SkCanvasOwner<'device> for SkPictureRecord<'device> {
    fn canvas(self: Pin<&mut Self>) -> Pin<&mut SkCanvas<'device>> {
        Self::canvas(self)
    }
    fn canvas_ref(&self) -> &SkCanvas<'device> {
        self.inherited.as_ref().get_ref()
    }
}
impl<'device> super::SkCanvas::SkCanvasVirtual<'device> for SkPictureRecord<'device> {
    // Source: src/core/SkPictureRecord.cpp:859-861
    // Source: src/core/SkPictureRecord.h:196-196
    // Source: src/core/SkPictureRecord.cpp:859-861
    unsafe fn onNewSurface(
        &self,
        _: NonNull<SkCanvas<'device>>,
        _: &super::SkImageInfo::SkImageInfo,
        _: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>> {
        sk_sp::default()
    }
    // Source: src/core/SkPictureRecord.h:197-197
    // Source: src/core/SkPictureRecord.h:197-197
    unsafe fn onPeekPixels(
        &self,
        _: NonNull<SkCanvas<'device>>,
        _: Option<&mut super::SkPixmap::SkPixmap>,
    ) -> bool {
        false
    }

    // Source: src/core/SkPictureRecord.cpp:609-617
    // Source: src/core/SkPictureRecord.h:213-213
    unsafe fn onDrawSlug(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        slug: &dyn Slug,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawSlug(canvas, slug, paint);
        }
        Ok(())
    }

    unsafe fn onDrawPatch(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        cubics: &[SkPoint; 12],
        colors: Option<&[u32; 4]>,
        tex: Option<&[SkPoint; 4]>,
        mode: super::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPatch(canvas, cubics, colors, tex, mode, paint);
        }
        Ok(())
    }
    unsafe fn onDrawAtlas2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        atlas: &Arc<dyn SkImage>,
        xform: &[super::SkRSXform::SkRSXform],
        tex: &[SkRect],
        colors: &[u32],
        mode: super::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&SkRect>,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawAtlas2(
                canvas,
                atlas,
                xform,
                tex,
                if colors.is_empty() {
                    None
                } else {
                    Some(colors)
                },
                mode,
                sampling,
                cull,
                paint,
            );
        }
        Ok(())
    }

    // Source: src/core/SkPictureRecord.cpp:57-64
    unsafe fn willSave(&self, canvas: NonNull<SkCanvas<'device>>) {
        unsafe {
            let storage = self.storage_mut();
            storage
                .fRestoreOffsetStack
                .push((storage.fWriter.bytesWritten() as i32).wrapping_neg());
            self.recordSave(canvas);
            SkCanvas::willSave_base_raw(canvas);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:74-87
    fn getSaveLayerStrategy(
        &self,
        rec: &super::SkCanvas::SaveLayerRec<'_>,
    ) -> super::SkCanvas::SaveLayerStrategy {
        unsafe {
            let storage = self.storage_mut();
            storage
                .fRestoreOffsetStack
                .push((storage.fWriter.bytesWritten() as i32).wrapping_neg());
            self.recordSaveLayer(self.base_raw(), rec);
        }
        let _ = self
            .inherited
            .as_ref()
            .get_ref()
            .inherited_getSaveLayerStrategy(rec);
        super::SkCanvas::SaveLayerStrategy::kNoLayer_SaveLayerStrategy
    }
    // Source: src/core/SkPictureRecord.cpp:89-107
    fn onDoSaveBehind(&self, subset: Option<&SkRect>) -> bool {
        unsafe {
            {
                let storage = self.storage_mut();
                storage
                    .fRestoreOffsetStack
                    .push((storage.fWriter.bytesWritten() as i32).wrapping_neg());
            }
            let mut size = 8;
            let mut flags = 0;
            if subset.is_some() {
                flags |= SAVEBEHIND_HAS_SUBSET;
                size += core::mem::size_of::<SkRect>();
            }
            let offset = self.addDraw(self.base_raw(), SAVE_BEHIND, &mut size);
            let storage = self.storage_mut();
            storage.addInt(flags as i32);
            if let Some(rect) = subset {
                storage.addRect(rect);
            }
            storage.validate(offset, size);
            false
        }
    }
    // Source: src/core/SkPictureRecord.cpp:197-212
    unsafe fn willRestore(&self, canvas: NonNull<SkCanvas<'device>>) {
        unsafe {
            if self.storage().fRestoreOffsetStack.is_empty() {
                return;
            }
            self.recordRestore(canvas, true);
            self.storage_mut().fRestoreOffsetStack.pop();
            SkCanvas::willRestore_base_raw(canvas);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:245-254
    unsafe fn didConcat44(&self, canvas: NonNull<SkCanvas<'device>>, m: &super::SkM44::SkM44) {
        unsafe {
            self.storage()
                .validate(self.storage().fWriter.bytesWritten(), 0);
            let mut size = 68;
            let offset = self.addDraw(canvas, CONCAT44, &mut size);
            {
                let mut col = [0.; 16];
                m.getColMajor(&mut col);
                let storage = self.storage_mut();
                storage.fWriter.write(col.as_ptr().cast(), 64);
                storage.validate(offset, size);
            }
            SkCanvas::didConcat44_base_raw(canvas, m);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:256-264
    unsafe fn didSetM44(&self, canvas: NonNull<SkCanvas<'device>>, m: &super::SkM44::SkM44) {
        unsafe {
            self.storage()
                .validate(self.storage().fWriter.bytesWritten(), 0);
            let mut size = 68;
            let offset = self.addDraw(canvas, SET_M44, &mut size);
            {
                let mut col = [0.; 16];
                m.getColMajor(&mut col);
                let storage = self.storage_mut();
                storage.fWriter.write(col.as_ptr().cast(), 64);
                storage.validate(offset, size);
            }
            SkCanvas::didSetM44_base_raw(canvas, m);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:266-268
    unsafe fn didScale(&self, canvas: NonNull<SkCanvas<'device>>, x: f32, y: f32) {
        unsafe {
            super::SkCanvas::SkCanvasVirtual::didConcat44(
                self,
                canvas,
                &super::SkM44::SkM44::Scale(x, y, 1.),
            );
        }
    }
    // Source: src/core/SkPictureRecord.cpp:270-272
    unsafe fn didTranslate(&self, canvas: NonNull<SkCanvas<'device>>, x: f32, y: f32) {
        unsafe {
            super::SkCanvas::SkCanvasVirtual::didConcat44(
                self,
                canvas,
                &super::SkM44::SkM44::Translate(x, y, 0.),
            );
        }
    }
    // Source: src/core/SkPictureRecord.cpp:332-335
    unsafe fn onClipRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: super::SkCanvas::ClipEdgeStyle,
    ) {
        unsafe {
            self.recordClipRect(
                canvas,
                rect,
                op,
                edge == super::SkCanvas::ClipEdgeStyle::kSoft_ClipEdgeStyle,
            );
            SkCanvas::onClipRect_base_raw(canvas, rect, op, edge);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:354-357
    unsafe fn onClipRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: super::SkCanvas::ClipEdgeStyle,
    ) {
        unsafe {
            self.recordClipRRect(
                canvas,
                rect,
                op,
                edge == super::SkCanvas::ClipEdgeStyle::kSoft_ClipEdgeStyle,
            );
            SkCanvas::onClipRRect_base_raw(canvas, rect, op, edge);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:375-379
    unsafe fn onClipPath(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        path: &SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: super::SkCanvas::ClipEdgeStyle,
    ) {
        unsafe {
            let id = self.storage_mut().addPathToHeap(path);
            self.recordClipPath(
                canvas,
                id,
                op,
                edge == super::SkCanvas::ClipEdgeStyle::kSoft_ClipEdgeStyle,
            );
            SkCanvas::onClipPath_base_raw(canvas, path, op, edge);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:397-411
    unsafe fn onClipShader(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        shader: Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            let mut paint = SkPaint::default();
            paint.setShader(Some(shader.clone()));
            let mut size = 12;
            let offset = self.addDraw(canvas, CLIP_SHADER_IN_PAINT, &mut size);
            {
                let storage = self.storage_mut();
                storage.addPaint(&paint);
                storage.addInt(op as i32);
                storage.validate(offset, size);
            }
            SkCanvas::onClipShader_base_raw(canvas, shader, op);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:413-416
    unsafe fn onClipRegion(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        region: &SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            self.recordClipRegion(canvas, region, op);
            SkCanvas::onClipRegion_base_raw(canvas, region, op);
        }
    }
    // Source: src/core/SkPictureRecord.cpp:435-446
    unsafe fn onResetClip(&self, canvas: NonNull<SkCanvas<'device>>) {
        unsafe {
            if !self.storage().fRestoreOffsetStack.is_empty() {
                self.storage_mut()
                    .fillRestoreOffsetPlaceholdersForCurrentStackLevel(0);
            }
            let mut size = 4;
            let offset = self.addDraw(canvas, RESET_CLIP, &mut size);
            self.storage().validate(offset, size);
            SkCanvas::onResetClip_base_raw(canvas);
        }
    }
    unsafe fn onDrawImage2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: Option<&Arc<dyn SkImage>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
    ) {
        unsafe {
            self.recordDrawImage2(
                canvas,
                image.expect("native recording image callback"),
                x,
                y,
                sampling,
                paint,
            );
        }
    }
    unsafe fn onDrawImageRect2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: &Arc<dyn SkImage>,
        src: &SkRect,
        dst: &SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: super::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawImageRect2(canvas, image, src, dst, sampling, paint, constraint);
        }
        Ok(())
    }
    unsafe fn onDrawImageLattice2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        image: &Arc<dyn SkImage>,
        lattice: &super::SkCanvas::Lattice,
        dst: &SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawImageLattice2(canvas, image, lattice, dst, filter, paint);
        }
        Ok(())
    }
    unsafe fn onDrawEdgeAAImageSet2(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        set: &[super::SkCanvas::ImageSetEntry],
        clips: &[SkPoint],
        matrices: &[SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&SkPaint>,
        constraint: super::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawEdgeAAImageSet2(
                canvas, set, clips, matrices, sampling, paint, constraint,
            );
        }
        Ok(())
    }
    unsafe fn onDrawVerticesObject(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        vertices: &SkVertices,
        mode: super::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawVerticesObject(canvas, vertices, mode, paint);
        }
        Ok(())
    }
}
// Rust vtable projections reuse the single original recording implementations;
// this is structural dispatch glue, with no second native-source credit.
impl<'device> crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer<'device>
    for SkPictureRecord<'device>
{
    unsafe fn onDrawPaint(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPaint(canvas, paint);
        }
        Ok(())
    }
    unsafe fn onDrawBehind(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawBehind(canvas, paint);
        }
        Ok(())
    }
    unsafe fn onDrawRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawRect(canvas, rect, paint);
        }
        Ok(())
    }
    unsafe fn onDrawRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRRect,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawRRect(canvas, rect, paint);
        }
        Ok(())
    }
    unsafe fn onDrawDRRect(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        outer: &SkRRect,
        inner: &SkRRect,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawDRRect(canvas, outer, inner, paint);
        }
        Ok(())
    }
    unsafe fn onDrawOval(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        oval: &SkRect,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawOval(canvas, oval, paint);
        }
        Ok(())
    }
    unsafe fn onDrawArc(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        oval: &SkRect,
        start: f32,
        sweep: f32,
        center: bool,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawArc(canvas, oval, start, sweep, center, paint);
        }
        Ok(())
    }
    unsafe fn onDrawPath(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        path: &SkPath,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPath(canvas, path, paint);
        }
        Ok(())
    }
    unsafe fn onDrawRegion(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        region: &SkRegion,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawRegion(canvas, region, paint);
        }
        Ok(())
    }
    unsafe fn onDrawTextBlob(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        blob: &SkTextBlob,
        x: f32,
        y: f32,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawTextBlob(canvas, blob, x, y, paint);
        }
        Ok(())
    }
    unsafe fn onDrawPatch(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        cubics: &[SkPoint; 12],
        colors: Option<&[u32; 4]>,
        tex: Option<&[SkPoint; 4]>,
        mode: super::SkBlendMode::SkBlendMode,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPatch(canvas, cubics, colors, tex, mode, paint);
        }
        Ok(())
    }
    unsafe fn onDrawPoints(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        mode: super::SkCanvas::PointMode,
        points: &[SkPoint],
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPoints(canvas, mode, points, paint);
        }
        Ok(())
    }
    unsafe fn onDrawShadowRec(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        path: &SkPath,
        rec: &super::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawShadowRec(canvas, path, rec);
        }
        Ok(())
    }
    unsafe fn onDrawEdgeAAQuad(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        clip: Option<&[SkPoint; 4]>,
        aa: super::SkCanvas::QuadAAFlags,
        color: &super::SkColor::SkColor4f,
        mode: super::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawEdgeAAQuad(canvas, rect, clip, aa, color, mode);
        }
        Ok(())
    }
    unsafe fn onDrawAnnotation(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        rect: &SkRect,
        key: &core::ffi::CStr,
        value: Option<&Arc<SkData>>,
    ) {
        unsafe {
            self.recordDrawAnnotation(canvas, rect, key, value);
        }
    }
    unsafe fn onDrawDrawable(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        drawable: &dyn SkDrawable,
        matrix: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawDrawable(canvas, drawable, matrix);
        }
        Ok(())
    }
    unsafe fn onDrawPicture(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        picture: &dyn SkPicture,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.recordDrawPicture(canvas, picture, matrix, paint);
        }
        Ok(())
    }
}

// Source: src/core/SkPictureRecord.cpp:40-46
const kNoInitialSave: i32 = -1;
const kUInt32Size: usize = 4;
