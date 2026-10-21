use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::SkBBHFactory::SkBBoxHierarchy;
use crate::src::core::{
    SkCanvas::SkCanvas, SkDrawable::SkDrawableState, SkPicture::SkPicture, SkRecord::SkRecord,
    SkRecordCanvas::SkDrawableList, SkRect::SkRect,
};
use core::{mem::ManuallyDrop, ptr::NonNull};
// Source: src/core/SkRecordedDrawable.h:27-35
// Source: src/core/SkRecordedDrawable.h:48-53
pub struct SkRecordedDrawable {
    fRecord: ManuallyDrop<sk_sp<SkRecord>>,
    fBBH: ManuallyDrop<sk_sp<dyn SkBBoxHierarchy>>,
    fDrawableList: ManuallyDrop<Option<Box<SkDrawableList>>>,
    fBounds: SkRect,
    fDrawableState: SkDrawableState,
    fRefCnt: SkRefCnt,
}
unsafe impl SkRefCntObject for SkRecordedDrawable {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.fRefCnt.ref_cnt_base()
    }
    fn ref_(&self) {
        self.fRefCnt.ref_();
    }
}
impl SkRecordedDrawable {
    // Source: src/core/SkRecordedDrawable.h:29-35
    pub fn new(
        record: sk_sp<SkRecord>,
        bbh: sk_sp<dyn SkBBoxHierarchy>,
        drawables: Option<Box<SkDrawableList>>,
        bounds: &SkRect,
    ) -> Self {
        Self {
            fRecord: ManuallyDrop::new(record),
            fBBH: ManuallyDrop::new(bbh),
            fDrawableList: ManuallyDrop::new(drawables),
            fBounds: *bounds,
            fDrawableState: SkDrawableState::new(),
            fRefCnt: SkRefCnt::new(),
        }
    }
    // Source: src/core/SkRecordedDrawable.h:40-40
    pub fn onGetBounds(&self) -> SkRect {
        self.fBounds
    }
    // Source: src/core/SkRecordedDrawable.cpp:23-34
    pub unsafe fn onApproximateBytesUsed(&self) -> usize {
        let mut drawables_size = 0usize;
        if let Some(list) = self.fDrawableList.as_ref() {
            for &drawable in
                unsafe { core::slice::from_raw_parts(list.begin(), list.count() as usize) }
            {
                drawables_size = drawables_size
                    .wrapping_add(unsafe { drawable.as_ref() }.approximateBytesUsed());
            }
        }
        core::mem::size_of::<Self>()
            .wrapping_add(self.fRecord.get().map_or(0, |record| record.bytesUsed()))
            .wrapping_add(self.fBBH.get().map_or(0, |bbh| bbh.bytesUsed()))
            .wrapping_add(drawables_size)
    }
    // Source: src/core/SkRecordedDrawable.cpp:36-44
    pub unsafe fn onDraw(&self, canvas: NonNull<SkCanvas<'_>>) -> Result<(), crate::RasterError> {
        let (drawables, count) = if let Some(list) = self.fDrawableList.as_ref() {
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
                self.fBBH.get(),
                None,
            )
        }
    }
    // Source: src/core/SkRecordedDrawable.cpp:46-58
    pub unsafe fn onMakePictureSnapshot(&self) -> Result<sk_sp<dyn SkPicture>, crate::RasterError> {
        let pictures = if let Some(list) = self.fDrawableList.as_ref() {
            unsafe { list.newDrawableSnapshot() }?
        } else {
            None
        };
        let mut bytes = 0usize;
        if let Some(pictures) = pictures.as_ref() {
            for picture in pictures.begin() {
                bytes = bytes.wrapping_add(picture.approximateBytesUsed());
            }
        }
        Ok(crate::src::core::SkBigPicture::SkBigPicture::new(
            &self.fBounds,
            (*self.fRecord).clone(),
            pictures,
            (*self.fBBH).clone(),
            bytes,
        ))
    }
}
// Native members destroy drawable list, BBH, Record, then inherited state.
// ManuallyDrop preserves that exact order without an extra owner or counter.
impl Drop for SkRecordedDrawable {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fDrawableList);
            ManuallyDrop::drop(&mut self.fBBH);
            ManuallyDrop::drop(&mut self.fRecord);
        }
    }
}
impl SkRecordedDrawable {
    // Source: src/core/SkRecordedDrawable.cpp:61-86
    pub unsafe fn flatten_recorded(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeRect(&self.fBounds);
        let info = crate::src::core::SkPictureData::SkPictInfo::new();
        let dimensions = crate::src::core::SkRect::SkIRect::make_wh(
            self.fBounds.width() as i32,
            self.fBounds.height() as i32,
        );
        let mut pictureRecord =
            crate::src::core::SkPictureRecord::SkPictureRecord::new(&dimensions, 0);
        let canvas = unsafe { NonNull::from(pictureRecord.as_mut().canvas().get_unchecked_mut()) };
        let bbh = if unsafe { canvas.as_ref() }
            .getLocalClipBounds()
            .contains(&self.fBounds)
        {
            None
        } else {
            self.fBBH.get()
        };
        let (drawables, count) = if let Some(list) = self.fDrawableList.as_ref() {
            (
                Some(unsafe { core::slice::from_raw_parts(list.begin(), list.count() as usize) }),
                list.count(),
            )
        } else {
            (None, 0)
        };
        unsafe { pictureRecord.as_mut().beginRecording() }?;
        unsafe {
            crate::src::core::SkRecordDraw::SkRecordDraw(
                &self.fRecord,
                canvas,
                None,
                drawables,
                count,
                bbh,
                None,
            )
        }?;
        unsafe { pictureRecord.as_mut().endRecording() }?;
        let data = crate::src::core::SkPictureData::SkPictureData::from_record(
            pictureRecord.as_ref().get_ref(),
            &info,
        );
        data.flatten(buffer)
    }
    // Source: src/core/SkRecordedDrawable.cpp:88-110
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        if !buffer.isValid() {
            return Ok(None);
        }
        let bounds = buffer.readRect();
        let mut info = crate::src::core::SkPictureData::SkPictInfo::new();
        info.setVersion(buffer.getVersion());
        info.fCullRect = bounds;
        let Some(data) =
            crate::src::core::SkPictureData::SkPictureData::CreateFromBuffer(buffer, &info)?
        else {
            return Ok(None);
        };
        let playback = crate::src::core::SkPicturePlayback::SkPicturePlayback::new(&data);
        let mut recorder = crate::src::core::SkPictureRecorder::SkPictureRecorder::new();
        let canvas = unsafe { recorder.beginRecording(&bounds, sk_sp::default()) }?;
        unsafe { playback.draw(canvas, None, Some(buffer)) }?;
        Ok(crate::src::core::SkDrawable::into_flattenable(unsafe {
            recorder.finishRecordingAsDrawable()
        }?))
    }
}
impl crate::src::core::SkDrawable::SkDrawable for SkRecordedDrawable {
    fn drawable_state(&self) -> &SkDrawableState {
        &self.fDrawableState
    }
    fn onGetBounds(&self) -> SkRect {
        SkRecordedDrawable::onGetBounds(self)
    }
    fn onApproximateBytesUsed(&self) -> usize {
        unsafe { SkRecordedDrawable::onApproximateBytesUsed(self) }
    }
    unsafe fn onDraw(&self, canvas: NonNull<SkCanvas<'_>>) -> Result<(), crate::RasterError> {
        unsafe { SkRecordedDrawable::onDraw(self, canvas) }
    }
    unsafe fn onMakePictureSnapshot(&self) -> Result<sk_sp<dyn SkPicture>, crate::RasterError> {
        unsafe { SkRecordedDrawable::onMakePictureSnapshot(self) }
    }
    fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.flatten_recorded(buffer) }
    }
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkRecordedDrawable")
    }
}
