// Copyright 2015 Google Inc. BSD-style license.
use super::SkPicture::SkPicture;
use crate::include::core::SkRefCnt::sk_sp;
// Source: src/core/SkBigPicture.h:30-40
// Boxed contiguous intrusive handles own precisely the native pre-retained
// picture references. The allocation length carries the native array count.
// The type is neither Clone nor Copy; it does not clone/ref pictures on adoption.
pub struct SnapshotArray {
    fPics: Box<[sk_sp<dyn SkPicture>]>,
}
impl SnapshotArray {
    // Source: src/core/SkBigPicture.h:32-32
    pub fn new(pics: Box<[sk_sp<dyn SkPicture>]>) -> Self {
        Self { fPics: pics }
    }
    // Source: src/core/SkBigPicture.h:33-33
    // Rust drops boxed slice elements from first to last, unref'ing each actual
    // picture once, then frees the pointer array, as the native destructor does.
    // No separate explicit unref is added, which would release every owner twice.
    // Source: src/core/SkBigPicture.h:35-35
    pub fn begin(&self) -> &[sk_sp<dyn SkPicture>] {
        &self.fPics
    }
    // Source: src/core/SkBigPicture.h:36-36
    pub fn count(&self) -> i32 {
        self.fPics.len() as i32
    }
}
use super::{
    SkCanvas::SkCanvas,
    SkPicture::{AbortCallback, SkPictureState},
    SkRecord::{RecordVisitor, SkRecord},
    SkRecords,
    SkRect::SkRect,
};
use crate::include::core::SkBBHFactory::SkBBoxHierarchy;
use crate::include::core::SkRefCnt::{SkRefCntBase, SkRefCntObject};
use core::{mem::ManuallyDrop, ptr::NonNull};
impl SkBigPicture {
    // Source: src/core/SkBigPicture.h:54-54
    pub fn asSkBigPicture(&self) -> &Self {
        self
    }
    // Source: src/core/SkBigPicture.h:57-58
    pub fn bbh(&self) -> Option<&dyn SkBBoxHierarchy> {
        self.fBBH.get()
    }
    pub fn record(&self) -> &SkRecord {
        &self.fRecord
    }
}
// Source: src/core/SkBigPicture.h:64-68
// Native declaration order is retained. Explicit reverse member destruction
// runs before the embedded Picture base's cache purge/counter teardown.
// Source: src/core/SkBigPicture.h:27-27
pub struct SkBigPicture {
    // Source: src/core/SkBigPicture.h:64-64
    fCullRect: SkRect,
    // Source: src/core/SkBigPicture.h:65-65
    fApproxBytesUsedBySubPictures: usize,
    // Source: src/core/SkBigPicture.h:66-66
    fRecord: ManuallyDrop<sk_sp<SkRecord>>,
    // Source: src/core/SkBigPicture.h:67-67
    fDrawablePicts: ManuallyDrop<Option<Box<SnapshotArray>>>,
    // Source: src/core/SkBigPicture.h:68-68
    fBBH: ManuallyDrop<sk_sp<dyn SkBBoxHierarchy>>,
    base: SkPictureState,
}
impl Drop for SkBigPicture {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fBBH);
            ManuallyDrop::drop(&mut self.fDrawablePicts);
            ManuallyDrop::drop(&mut self.fRecord);
        }
    }
}
// Native immutable recorded commands, snapshots and BBH retain their original
// thread-safe read protocol. No mutable record handle escapes this owner.
unsafe impl Send for SkBigPicture {}
unsafe impl Sync for SkBigPicture {}
unsafe impl SkRefCntObject for SkBigPicture {
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base.ref_count
    }
    fn ref_(&self) {
        self.base.ref_count.ref_();
    } // Adapted Source: src/core/SkBigPicture.h:27-27
      // Concrete destructor dispatch pairs this original complete allocation
      // with its actual Sized type; no trait metadata or owner is reconstructed here.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
}
// Source: src/core/SkBigPicture.h:42-46
// Source: src/core/SkBigPicture.cpp:19-29
impl SkBigPicture {
    pub fn new(
        cull: &SkRect,
        record: sk_sp<SkRecord>,
        drawable_picts: Option<Box<SnapshotArray>>,
        bbh: sk_sp<dyn SkBBoxHierarchy>,
        approx_bytes: usize,
    ) -> sk_sp<dyn SkPicture> {
        let owner = Box::new(Self {
            fCullRect: *cull,
            fApproxBytesUsedBySubPictures: approx_bytes,
            fRecord: ManuallyDrop::new(record),
            fDrawablePicts: ManuallyDrop::new(drawable_picts),
            fBBH: ManuallyDrop::new(bbh),
            base: SkPictureState::new(),
        });
        let dynamic: *mut dyn SkPicture = Box::into_raw(owner);
        unsafe { sk_sp::from_raw(dynamic) }
    }
    // Source: src/core/SkBigPicture.h:50-50
    // Source: src/core/SkBigPicture.cpp:31-44
    pub unsafe fn playback<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        callback: Option<&mut dyn AbortCallback>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            let useBBH = !canvas
                .as_ref()
                .getLocalClipBounds()
                .contains(&self.fCullRect);
            super::SkRecordDraw::SkRecordDraw(
                &self.fRecord,
                canvas,
                self.drawablePicts(),
                None,
                self.drawableCount(),
                if useBBH { self.fBBH.get() } else { None },
                callback,
            )
        }
    }
}
// Source: src/core/SkBigPicture.cpp:46-55
struct NestedApproxOpCounter {
    fCount: i32,
}
// C++'s generic overload is instantiated for every REAL native record tag by
// the same canonical macro list as SkRecord::visit. DrawPicture keeps its
// explicit recursive overload; no missing command receives a replay fallback.
macro_rules! nested_count_method {
    (DrawPicture) => {
        unsafe fn DrawPicture(&mut self, op: &SkRecords::DrawPicture) {
            self.fCount += op.picture.approximateOpCount(true);
        }
    };
    ($name:ident) => {
        unsafe fn $name(&mut self, _op: &SkRecords::$name) {
            self.fCount += 1;
        }
    };
}
macro_rules! nested_count_visitor {($($name:ident:$variant:ident),*)=>{
    impl RecordVisitor for NestedApproxOpCounter {type Output=();$(nested_count_method!($name);)*}
};}
SkRecords::SK_RECORD_TYPES!(nested_count_visitor);
impl SkBigPicture {
    // Source: src/core/SkBigPicture.h:51-51
    // Source: src/core/SkBigPicture.cpp:57-57
    pub fn cullRect(&self) -> SkRect {
        self.fCullRect
    }
    // Source: src/core/SkBigPicture.h:52-52
    // Source: src/core/SkBigPicture.cpp:58-68
    pub fn approximateOpCount(&self, nested: bool) -> i32 {
        if nested {
            let mut visitor = NestedApproxOpCounter { fCount: 0 };
            for i in 0..self.fRecord.count() {
                unsafe {
                    self.fRecord.visit(i, &mut visitor);
                }
            }
            visitor.fCount
        } else {
            self.fRecord.count()
        }
    }
    // Source: src/core/SkBigPicture.h:53-53
    // Source: src/core/SkBigPicture.cpp:69-73
    pub fn approximateBytesUsed(&self) -> usize {
        let mut bytes = std::mem::size_of::<Self>()
            + self.fRecord.bytesUsed()
            + self.fApproxBytesUsedBySubPictures;
        if let Some(bbh) = self.fBBH.get() {
            bytes += bbh.bytesUsed();
        }
        bytes
    }
    // Source: src/core/SkBigPicture.h:61-61
    // Source: src/core/SkBigPicture.cpp:75-77
    fn drawableCount(&self) -> i32 {
        self.fDrawablePicts.as_ref().map_or(0, |pics| pics.count())
    }
    // Source: src/core/SkBigPicture.h:62-62
    // Source: src/core/SkBigPicture.cpp:79-81
    fn drawablePicts(&self) -> Option<&[sk_sp<dyn SkPicture>]> {
        self.fDrawablePicts.as_ref().map(|pics| pics.begin())
    }
}
impl SkPicture for SkBigPicture {
    fn picture_state(&self) -> &SkPictureState {
        &self.base
    }
    fn asSkBigPicture(&self) -> Option<&Self> {
        Some(self)
    }
    unsafe fn playback<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        callback: Option<&mut dyn AbortCallback>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBigPicture::playback(self, canvas, callback) }
    }
    fn cullRect(&self) -> SkRect {
        SkBigPicture::cullRect(self)
    }
    fn approximateOpCount(&self, nested: bool) -> i32 {
        SkBigPicture::approximateOpCount(self, nested)
    }
    fn approximateBytesUsed(&self) -> usize {
        SkBigPicture::approximateBytesUsed(self)
    }
}
