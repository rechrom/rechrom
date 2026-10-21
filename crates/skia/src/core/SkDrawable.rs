use crate::src::core::{SkCanvas::SkCanvas, SkMatrix::SkMatrix, SkRect::SkRect};
use core::ptr::NonNull;

// Source: include/core/SkDrawable.h:37-37
// Concrete objects embed the actual inherited counter and sole generation state.
// The native class hierarchy is projected through this abstract CPU protocol.
pub trait SkDrawable: crate::src::core::SkFlattenable::SkFlattenable + 'static {
    fn drawable_state(&self) -> &SkDrawableState;

    /// Safety: Canvas is pinned and has its native synchronous device/pixel lease.
    /// No whole-Canvas mutable reference may span onDraw or restoring a layer.
    /// Implementations must not unwind across this operation.
    // Source: include/core/SkDrawable.h:44-44
    unsafe fn draw(
        &self,
        canvas: NonNull<SkCanvas<'_>>,
        matrix: Option<&SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        drawable_draw(self, canvas, matrix)
    }

    /// Safety: the same raw operation lease as draw is required.
    // Source: include/core/SkDrawable.h:45-45
    unsafe fn draw_xy(
        &self,
        canvas: NonNull<SkCanvas<'_>>,
        x: f32,
        y: f32,
    ) -> Result<(), crate::RasterError> {
        drawable_draw_xy(self, canvas, x, y)
    }

    // Source: include/core/SkDrawable.h:99-99
    // Source: src/core/SkDrawable.cpp:60-62
    unsafe fn makePictureSnapshot(
        &self,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
        crate::RasterError,
    > {
        unsafe { self.onMakePictureSnapshot() }
    }
    // Source: include/core/SkDrawable.h:172-172
    unsafe fn onMakePictureSnapshot(
        &self,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
        crate::RasterError,
    > {
        unsafe { drawable_make_picture_snapshot(self) }
    }

    fn getGenerationID(&self) -> u32 {
        self.drawable_state().getGenerationID()
    }
    fn getBounds(&self) -> SkRect {
        drawable_bounds(self)
    }
    fn approximateBytesUsed(&self) -> usize {
        drawable_bytes(self)
    }
    fn notifyDrawingChanged(&self) {
        self.drawable_state().notifyDrawingChanged();
    }

    // Source: include/core/SkDrawable.h:130-132
    fn GetFlattenableType() -> crate::src::core::SkFlattenable::Type
    where
        Self: Sized,
    {
        crate::src::core::SkFlattenable::Type::kSkDrawable_Type
    }
    // Source: include/core/SkDrawable.h:134-136
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkDrawable_Type
    }
    // Source: include/core/SkDrawable.h:145-145
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        None
    }
    // Inherited native SkFlattenable default; subclasses may override this protocol.
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        crate::src::core::SkFlattenable::flatten_base(writer)?;
        Ok(())
    }
    // Source: include/core/SkDrawable.h:146-146
    fn getTypeName(&self) -> Option<&str> {
        None
    }
    // Source: include/core/SkDrawable.h:151-151
    fn onGetBounds(&self) -> SkRect;
    fn onApproximateBytesUsed(&self) -> usize {
        default_bytes()
    }
    // Source: include/core/SkDrawable.h:153-153
    /// Safety: retain the Canvas raw operation lease; no whole-Canvas mutable
    /// reference may remain active while calling an entry which can trigger COW.
    unsafe fn onDraw(&self, canvas: NonNull<SkCanvas<'_>>) -> Result<(), crate::RasterError>;
}

// Delegation of inherited virtual metadata; it carries no second counter/owner.
impl<T: SkDrawable + 'static> crate::src::core::SkFlattenable::SkFlattenable for T {
    unsafe fn project_drawable_owner(
        &self,
        allocation: NonNull<()>,
    ) -> Option<NonNull<dyn SkDrawable>> {
        debug_assert_eq!(allocation.as_ptr(), (self as *const T).cast_mut().cast());
        Some(NonNull::from(self as &dyn SkDrawable))
    }
    unsafe fn project_path_effect_owner(
        &self,
        _: std::ptr::NonNull<()>,
    ) -> Option<std::ptr::NonNull<dyn crate::src::core::SkPathEffect::SkPathEffect>> {
        None
    }
}
impl<T: SkDrawable + 'static> crate::src::core::SkFlattenable::SkFlattenableView for T {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        SkDrawable::getFactory(self)
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        SkDrawable::flatten(self, writer)?;
        Ok(())
    }

    fn getTypeName(&self) -> Option<&str> {
        SkDrawable::getTypeName(self)
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        SkDrawable::getFlattenableType(self)
    }
}

// Source: include/core/SkDrawable.h:175-175
// Cell models native non-const state updates without inventing Sync.
pub struct SkDrawableState {
    fGenerationID: std::cell::Cell<i32>,
}

// Source: src/core/SkDrawable.cpp:22-23
// Source: src/core/SkDrawable.cpp:25-30
fn next_generation_id() -> i32 {
    static NEXT_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
    loop {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if id != 0 {
            return id;
        }
    }
}
impl SkDrawableState {
    // Source: src/core/SkDrawable.cpp:32-32
    pub fn new() -> Self {
        Self {
            fGenerationID: std::cell::Cell::new(0),
        }
    }
}

// Source: src/core/SkDrawable.cpp:43-54
unsafe fn drawable_draw(
    drawable: &(impl SkDrawable + ?Sized),
    mut canvas: NonNull<SkCanvas<'_>>,
    matrix: Option<&SkMatrix>,
) -> Result<(), crate::RasterError> {
    // Each Pin projection ends before callbacks. Save and concat do not perform
    // drawing/COW; raw restoration splits its own device/state leases internally.
    let save_count = core::pin::Pin::new_unchecked(canvas.as_mut()).save();
    if let Some(matrix) = matrix {
        core::pin::Pin::new_unchecked(canvas.as_mut()).concat(matrix);
    }
    let drawn = drawable.onDraw(canvas);
    let restored = SkCanvas::restoreToCount_raw(canvas, save_count);
    drawn.and(restored)
}

// Source: src/core/SkDrawable.cpp:55-59
unsafe fn drawable_draw_xy(
    drawable: &(impl SkDrawable + ?Sized),
    canvas: NonNull<SkCanvas<'_>>,
    x: f32,
    y: f32,
) -> Result<(), crate::RasterError> {
    drawable.draw(canvas, Some(&SkMatrix::Translate(x, y)))
}

impl SkDrawableState {
    // Source: src/core/SkDrawable.cpp:64-69
    pub fn getGenerationID(&self) -> u32 {
        if self.fGenerationID.get() == 0 {
            self.fGenerationID.set(next_generation_id());
        }
        self.fGenerationID.get() as u32
    }
}

// Source: src/core/SkDrawable.cpp:71-73
fn drawable_bounds(drawable: &(impl SkDrawable + ?Sized)) -> SkRect {
    drawable.onGetBounds()
}
// Source: src/core/SkDrawable.cpp:75-77
fn drawable_bytes(drawable: &(impl SkDrawable + ?Sized)) -> usize {
    drawable.onApproximateBytesUsed()
}
// Source: src/core/SkDrawable.cpp:78-80
fn default_bytes() -> usize {
    0
}

impl SkDrawableState {
    // Source: src/core/SkDrawable.cpp:82-84
    pub fn notifyDrawingChanged(&self) {
        self.fGenerationID.set(0);
    }
}

// Native ownership upcast: retain the same complete allocation and transfer the
// sole intrusive reference. No wrapper allocation or second counter is added.
pub(crate) fn into_flattenable(
    mut drawable: crate::include::core::SkRefCnt::sk_sp<dyn SkDrawable>,
) -> Option<crate::src::core::SkFlattenable::FlattenableOwner> {
    let raw = drawable.release()?;
    Some(
        crate::src::core::SkFlattenable::FlattenableOwner::Intrusive(unsafe {
            crate::include::core::SkRefCnt::sk_sp::from_raw(
                raw.as_ptr() as *mut dyn crate::src::core::SkFlattenable::SkFlattenable
            )
        }),
    )
}

// Source: src/core/SkDrawable.cpp:88-98
unsafe fn drawable_make_picture_snapshot(
    drawable: &(impl SkDrawable + ?Sized),
) -> Result<
    crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
    crate::RasterError,
> {
    let mut recorder = crate::src::core::SkPictureRecorder::SkPictureRecorder::new();
    let bounds = drawable.getBounds();
    let canvas = unsafe {
        recorder.beginRecording(&bounds, crate::include::core::SkRefCnt::sk_sp::default())
    }?;
    unsafe { drawable.draw(canvas, None) }?;
    // Native compile-time false debug bounding-box branch performs no calls.
    unsafe { recorder.finishRecordingAsPicture() }
}

// Source: include/core/SkDrawable.h:138-143
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<Option<crate::include::core::SkRefCnt::sk_sp<dyn SkDrawable>>, crate::RasterError> {
    use crate::src::core::SkFlattenable::{self, FlattenableOwner, Type};
    let Some(owner) = SkFlattenable::Deserialize(Type::kSkDrawable_Type, data, procs)? else {
        return Ok(None);
    };
    let FlattenableOwner::Intrusive(mut owner) = owner else {
        return Ok(None);
    };
    let Some(raw) = owner.as_ptr() else {
        return Ok(None);
    };
    let projected = unsafe { raw.as_ref().project_drawable_owner(raw.cast()) };
    let Some(projected) = projected else {
        return Ok(None);
    };
    owner.release();
    Ok(Some(unsafe {
        crate::include::core::SkRefCnt::sk_sp::from_raw(projected.as_ptr())
    }))
}
// Source: include/core/SkDrawable.h:139-139
pub fn Deserialize_default(
    data: &[u8],
) -> Result<Option<crate::include::core::SkRefCnt::sk_sp<dyn SkDrawable>>, crate::RasterError> {
    Deserialize(data, None)
}

// Source: src/core/SkDrawable.cpp:34-41
unsafe fn draw_bbox(canvas: NonNull<SkCanvas<'_>>, r: &SkRect) -> Result<(), crate::RasterError> {
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setStyle(crate::src::core::SkPaint::Style::kStroke_Style);
    paint.setColor(0xFFFF7088);
    unsafe {
        SkCanvas::drawRect_raw(canvas, r, &paint)?;
        SkCanvas::drawLine_raw(canvas, r.fLeft, r.fTop, r.fRight, r.fBottom, &paint)?;
        SkCanvas::drawLine_raw(canvas, r.fLeft, r.fBottom, r.fRight, r.fTop, &paint)?;
    }
    Ok(())
}
