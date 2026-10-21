use crate::include::core::SkRefCnt::{sk_sp, SkRefCntBase, SkRefCntObject};
use crate::src::core::{
    SkBBHFactory::{Metadata, SkBBHFactory, SkBBoxHierarchy},
    SkBigPicture::{SkBigPicture, SnapshotArray},
    SkCanvas::SkCanvas,
    SkDrawable::SkDrawable,
    SkPicture::{AbortCallback, SkPicture, SkPictureState},
    SkRecord::SkRecord,
    SkRecordCanvas::SkRecordCanvas,
    SkRecordedDrawable::SkRecordedDrawable,
    SkRect::SkRect,
};
use core::{mem::ManuallyDrop, pin::Pin, ptr::NonNull};
// Source: include/core/SkPictureRecorder.h:31-32
// Source: include/core/SkPictureRecorder.h:105-113
// Original native fields are contiguous; explicit destruction retains reverse
// member order (Record, recorder, BBH), independent of Rust's default order.
pub struct SkPictureRecorder<'device> {
    fBBH: ManuallyDrop<sk_sp<dyn SkBBoxHierarchy>>,
    fRecorder: ManuallyDrop<Pin<Box<SkRecordCanvas<'device>>>>,
    fRecord: ManuallyDrop<sk_sp<SkRecord>>,
    fCullRect: SkRect,
    fActivelyRecording: bool,
}
impl<'device> SkPictureRecorder<'device> {
    // Source: include/core/SkPictureRecorder.h:33-33
    // Source: src/core/SkPictureRecorder.cpp:27-30
    pub fn new() -> Self {
        Self {
            fBBH: ManuallyDrop::new(sk_sp::default()),
            fRecorder: ManuallyDrop::new(SkRecordCanvas::new(
                core::ptr::null_mut(),
                &SkRect::MakeEmpty(),
            )),
            fRecord: ManuallyDrop::new(sk_sp::default()),
            fCullRect: SkRect::MakeEmpty(),
            fActivelyRecording: false,
        }
    }
    // Source: include/core/SkPictureRecorder.h:43-43
    // Source: src/core/SkPictureRecorder.cpp:34-47
    pub unsafe fn beginRecording(
        &mut self,
        bounds: &SkRect,
        bbh: sk_sp<dyn SkBBoxHierarchy>,
    ) -> Result<NonNull<SkCanvas<'device>>, crate::RasterError> {
        let cull = if bounds.isEmpty() {
            SkRect::MakeEmpty()
        } else {
            *bounds
        };
        self.fCullRect = cull;
        drop(core::mem::replace(&mut *self.fBBH, bbh));
        if self.fRecord.is_none() {
            self.fRecord = ManuallyDrop::new(sk_sp::new(SkRecord::new()));
        }
        let ptr = self.fRecord.as_ptr().expect("native record owner").as_ptr();
        unsafe {
            self.fRecorder
                .as_mut()
                .get_unchecked_mut()
                .reset(ptr, &cull)
        }?;
        self.fActivelyRecording = true;
        Ok(self
            .getRecordingCanvas()
            .expect("active recording owns its canvas"))
    }
    // Source: include/core/SkPictureRecorder.h:45-45
    // Source: src/core/SkPictureRecorder.cpp:49-51
    pub unsafe fn beginRecordingWithFactory(
        &mut self,
        bounds: &SkRect,
        factory: Option<&dyn SkBBHFactory>,
    ) -> Result<NonNull<SkCanvas<'device>>, crate::RasterError> {
        unsafe {
            self.beginRecording(
                bounds,
                factory.map_or_else(sk_sp::default, |factory| factory.make()),
            )
        }
    }
    // Source: include/core/SkPictureRecorder.h:47-50
    pub unsafe fn beginRecordingWH(
        &mut self,
        width: f32,
        height: f32,
        factory: Option<&dyn SkBBHFactory>,
    ) -> Result<NonNull<SkCanvas<'device>>, crate::RasterError> {
        unsafe { self.beginRecordingWithFactory(&SkRect::MakeWH(width, height), factory) }
    }
    // Source: include/core/SkPictureRecorder.h:56-56
    // Source: src/core/SkPictureRecorder.cpp:54-56
    pub fn getRecordingCanvas(&mut self) -> Option<NonNull<SkCanvas<'device>>> {
        if self.fActivelyRecording {
            let base = unsafe { self.fRecorder.as_mut().canvas().get_unchecked_mut() };
            Some(NonNull::from(base))
        } else {
            None
        }
    }
}
// Source: src/core/SkPictureRecorder.cpp:57-64
struct SkEmptyPicture {
    state: SkPictureState,
}
unsafe impl SkRefCntObject for SkEmptyPicture {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.state.ref_count
    }
    fn ref_(&self) {
        self.state.ref_count.ref_();
    }
}
impl SkPicture for SkEmptyPicture {
    fn picture_state(&self) -> &SkPictureState {
        &self.state
    }
    unsafe fn playback<'device>(
        &self,
        _: NonNull<SkCanvas<'device>>,
        _: Option<&mut dyn AbortCallback>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    fn approximateBytesUsed(&self) -> usize {
        core::mem::size_of::<Self>()
    }
    fn approximateOpCount(&self, _: bool) -> i32 {
        0
    }
    fn cullRect(&self) -> SkRect {
        SkRect::MakeEmpty()
    }
}
impl<'device> SkPictureRecorder<'device> {
    // Source: include/core/SkPictureRecorder.h:69-69
    // Source: src/core/SkPictureRecorder.cpp:67-109
    pub unsafe fn finishRecordingAsPicture(
        &mut self,
    ) -> Result<sk_sp<dyn SkPicture>, crate::RasterError> {
        self.fActivelyRecording = false;
        let canvas = unsafe { NonNull::from(self.fRecorder.as_mut().canvas().get_unchecked_mut()) };
        unsafe { SkCanvas::restoreToCount_raw(canvas, 1) }?;
        if self.fRecord.count() == 0 {
            let empty: Box<dyn SkPicture> = Box::new(SkEmptyPicture {
                state: SkPictureState::new(),
            });
            return Ok(unsafe { sk_sp::from_raw(Box::into_raw(empty)) });
        }
        unsafe {
            crate::src::core::SkRecordOpts::SkRecordOptimize(
                &mut *self.fRecord.as_ptr().expect("native record owner").as_ptr(),
            )
        };
        let pictures = if let Some(list) = self.fRecorder.getDrawableList() {
            unsafe { list.newDrawableSnapshot() }?
        } else {
            None
        };
        if let Some(bbh) = self.fBBH.get() {
            let count = self.fRecord.count();
            let mut bounds = unsafe {
                crate::include::private::SkTemplates::AutoTArray::<SkRect>::new_uninit(
                    count as usize,
                )
            };
            let mut meta =
                crate::include::private::SkTemplates::AutoTMalloc::<Metadata>::with_count(
                    count as usize,
                );
            unsafe {
                crate::src::core::SkRecordDraw::SkRecordFillBounds(
                    &self.fCullRect,
                    &self.fRecord,
                    bounds.data_mut(),
                    meta.data_mut(),
                )
            }?;
            unsafe {
                bounds.assume_init();
                bbh.insert_with_metadata(
                    bounds.as_slice().expect("native completed op bounds"),
                    core::slice::from_raw_parts(meta.get(), count as usize),
                    count,
                );
            }
            let mut bbh_bound = SkRect::MakeEmpty();
            for i in 0..count {
                bbh_bound.join(bounds.at(i as usize));
            }
            debug_assert!(
                bbh_bound.isEmpty()
                    || self.fCullRect.contains(&bbh_bound)
                    || (bbh_bound.isEmpty() && self.fCullRect.isEmpty())
            );
            self.fCullRect = bbh_bound;
        }
        let mut bytes = self.fRecorder.approxBytesUsedBySubPictures();
        if let Some(pictures) = pictures.as_ref() {
            for picture in pictures.begin() {
                bytes = bytes.wrapping_add(picture.approximateBytesUsed());
            }
        }
        let record = core::mem::replace(&mut *self.fRecord, sk_sp::default());
        let bbh = core::mem::replace(&mut *self.fBBH, sk_sp::default());
        Ok(SkBigPicture::new(
            &self.fCullRect,
            record,
            pictures,
            bbh,
            bytes,
        ))
    }
    // Source: include/core/SkPictureRecorder.h:80-80
    // Source: src/core/SkPictureRecorder.cpp:111-114
    pub unsafe fn finishRecordingAsPictureWithCull(
        &mut self,
        cull: &SkRect,
    ) -> Result<sk_sp<dyn SkPicture>, crate::RasterError> {
        self.fCullRect = *cull;
        unsafe { self.finishRecordingAsPicture() }
    }
    // Source: include/core/SkPictureRecorder.h:103-103
    // Source: src/core/SkPictureRecorder.cpp:117-130
    pub unsafe fn partialReplay(
        &self,
        canvas: Option<NonNull<SkCanvas<'device>>>,
    ) -> Result<(), crate::RasterError> {
        let Some(canvas) = canvas else {
            return Ok(());
        };
        let (drawables, count) = if let Some(list) = self.fRecorder.getDrawableList() {
            (
                Some(unsafe { core::slice::from_raw_parts(list.begin(), list.count() as usize) }),
                list.count(),
            )
        } else {
            (None, 0)
        };
        unsafe {
            crate::src::core::SkRecordDraw::SkRecordDraw(
                &self.fRecord,
                canvas,
                None,
                drawables,
                count,
                None,
                None,
            )
        }
    }
    // Source: include/core/SkPictureRecorder.h:94-94
    // Source: src/core/SkPictureRecorder.cpp:132-150
    pub unsafe fn finishRecordingAsDrawable(
        &mut self,
    ) -> Result<sk_sp<dyn SkDrawable>, crate::RasterError> {
        self.fActivelyRecording = false;
        let canvas = unsafe { NonNull::from(self.fRecorder.as_mut().canvas().get_unchecked_mut()) };
        unsafe { SkCanvas::restoreToCount_raw(canvas, 1) }?;
        unsafe {
            crate::src::core::SkRecordOpts::SkRecordOptimize(
                &mut *self.fRecord.as_ptr().expect("native record owner").as_ptr(),
            )
        };
        if let Some(bbh) = self.fBBH.get() {
            let count = self.fRecord.count();
            let mut bounds = unsafe {
                crate::include::private::SkTemplates::AutoTArray::<SkRect>::new_uninit(
                    count as usize,
                )
            };
            let mut meta =
                crate::include::private::SkTemplates::AutoTMalloc::<Metadata>::with_count(
                    count as usize,
                );
            unsafe {
                crate::src::core::SkRecordDraw::SkRecordFillBounds(
                    &self.fCullRect,
                    &self.fRecord,
                    bounds.data_mut(),
                    meta.data_mut(),
                )
            }?;
            unsafe {
                bounds.assume_init();
                bbh.insert_with_metadata(
                    bounds.as_slice().expect("native completed op bounds"),
                    core::slice::from_raw_parts(meta.get(), count as usize),
                    count,
                );
            }
        }
        let record = core::mem::replace(&mut *self.fRecord, sk_sp::default());
        let bbh = core::mem::replace(&mut *self.fBBH, sk_sp::default());
        let list = unsafe { self.fRecorder.as_mut().get_unchecked_mut() }.detachDrawableList();
        let drawable: Box<dyn SkDrawable> =
            Box::new(SkRecordedDrawable::new(record, bbh, list, &self.fCullRect));
        Ok(unsafe { sk_sp::from_raw(Box::into_raw(drawable)) })
    }
}
// Source: include/core/SkPictureRecorder.h:34-34
// Source: src/core/SkPictureRecorder.cpp:32-32
impl Drop for SkPictureRecorder<'_> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fRecord);
            ManuallyDrop::drop(&mut self.fRecorder);
            ManuallyDrop::drop(&mut self.fBBH);
        }
    }
}
