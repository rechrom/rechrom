// Source: src/image/SkSurface_Base.cpp:29-29
// Reexport the original incomplete identity, rather than declare another type.
pub use super::SkSurface::GrRecordingContext;
// Source: src/image/SkSurface_Base.cpp:32-32
pub use super::SkSurface::skgpu;
use super::{
    SkImage::SkImage,
    SkSurface::{ContentChangeMode, SkSurfaceState},
};
use crate::include::core::{SkRefCnt::SkRefCnt, SkSurfaceProps::SkSurfaceProps};
use crate::src::core::{
    SkCanvas::{SkCanvas, SkCanvasOwnerHandle},
    SkImageInfo::SkImageInfo,
    SkPixelStorage::SkPixelStorageObject,
    SkRect::SkIRect,
};
use core::{marker::PhantomPinned, pin::Pin, ptr::NonNull};
use std::sync::Arc;

// Source: src/image/SkSurface_Base.h:57-62
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kNull,
    kGanesh,
    kGraphite,
    kRaster,
}

/// Rust dispatch adaptation of the actual native CPU virtual slots. Each callback
/// projects the ORIGINAL pinned complete allocation from its embedded base; it
/// must not allocate a proxy owner, reinterpret a trait pointer, or retain a base
/// reference across a callback into the Canvas. This is not a backend service.
/// The concrete owner binds these slots once, after its whole allocation is pinned.
#[derive(Clone, Copy)]
pub(crate) struct SurfaceDispatch<'device> {
    // Structural vtable projections of actual native ordinary virtual slots.
    pub on_get_recording_context: Option<
        unsafe fn(NonNull<SkSurface_Base<'device>>) -> *mut super::SkSurface::GrRecordingContext,
    >,
    pub on_get_graphite_recorder: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
        ) -> *mut super::SkSurface::skgpu::graphite::Recorder,
    >,
    pub on_wait: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            i32,
            *const super::SkSurface::GrBackendSemaphore,
            bool,
        ) -> bool,
    >,
    pub on_characterize: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            *mut super::SkSurface::GrSurfaceCharacterization,
        ) -> bool,
    >,
    pub on_is_compatible: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            *const super::SkSurface::GrSurfaceCharacterization,
        ) -> bool,
    >,
    pub on_replace_backend_texture: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            *const super::SkSurface::GrBackendTexture,
            super::SkSurface::GrSurfaceOrigin,
            ContentChangeMode,
            Option<super::SkSurface::TextureReleaseProc>,
            super::SkSurface::ReleaseContext,
        ) -> bool,
    >,

    pub on_capabilities: unsafe fn(
        NonNull<SkSurface_Base<'device>>,
    )
        -> Arc<crate::include::core::SkCapabilities::SkCapabilities>,
    pub on_get_base_recorder: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
        ) -> Option<&'device dyn crate::include::core::SkRecorder::SkRecorder>,
    >,
    pub type_: unsafe fn(NonNull<SkSurface_Base<'device>>) -> Type,
    pub on_new_canvas:
        unsafe fn(NonNull<SkSurface_Base<'device>>) -> Option<SkCanvasOwnerHandle<'device>>,
    pub on_new_surface: unsafe fn(
        NonNull<SkSurface_Base<'device>>,
        &SkImageInfo,
    ) -> Option<
        crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'device> + 'device>,
    >,
    pub on_write_pixels: unsafe fn(
        NonNull<SkSurface_Base<'device>>,
        &crate::src::core::SkPixmap::SkPixmap,
        i32,
        i32,
    ) -> Result<(), crate::RasterError>,
    // Source: src/image/SkSurface_Base.h:105-110
    pub on_async_rescale_pixels: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            &SkImageInfo,
            SkIRect,
            super::SkSurface::RescaleGamma,
            super::SkSurface::RescaleMode,
            super::SkSurface::ReadPixelsCallback,
            super::SkSurface::ReadPixelsContext,
        ) -> Result<(), crate::RasterError>,
    >,
    // Source: src/image/SkSurface_Base.h:114-122
    pub on_async_rescale_yuv420: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            crate::src::core::SkImageInfo::SkYUVColorSpace,
            bool,
            Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
            SkIRect,
            crate::include::core::SkSize::SkISize,
            super::SkSurface::RescaleGamma,
            super::SkSurface::RescaleMode,
            super::SkSurface::ReadPixelsCallback,
            super::SkSurface::ReadPixelsContext,
        ),
    >,
    pub on_make_temporary_image: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
        ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError>,
    >,
    pub on_draw: Option<
        for<'target> unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            NonNull<SkCanvas<'target>>,
            f32,
            f32,
            &crate::include::core::SkSamplingOptions::SkSamplingOptions,
            Option<&crate::src::core::SkPaint::SkPaint>,
        ) -> Result<(), crate::RasterError>,
    >,
    pub on_copy_on_write: unsafe fn(NonNull<SkSurface_Base<'device>>, ContentChangeMode) -> bool,
    pub on_restore_backing_mutability: Option<unsafe fn(NonNull<SkSurface_Base<'device>>)>,
    pub on_discard: Option<unsafe fn(NonNull<SkSurface_Base<'device>>)>,
    pub on_new_image_snapshot: Option<
        unsafe fn(
            NonNull<SkSurface_Base<'device>>,
            Option<&SkIRect>,
        ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError>,
    >,
}

// Rust field destruction is ordered as the native reverse member destruction:
// storage, cached image, sole owned Canvas, then public Surface metadata/counter.
// There is exactly one embedded counter for the eventual complete Surface owner.
// The owning Canvas handle preserves its real complete derived allocation and
// destructor. fCachedCanvas is only the actual base/capture projection.
// Dispatch staging/pinning and Arc handles are adapted, not literal ABI/layout.
pub struct SkSurface_Base<'device> {
    // Source: src/image/SkSurface_Base.h:208-208
    pub(crate) fPixelStorage: Option<Arc<dyn SkPixelStorageObject>>,
    // Source: src/image/SkSurface_Base.h:194-194
    pub(crate) fCachedImage: Option<Arc<dyn SkImage>>,
    // Source: src/image/SkSurface_Base.h:193-193
    fOwnedBaseCanvas: Option<SkCanvasOwnerHandle<'device>>,
    // Source: src/image/SkSurface_Base.h:190-190
    fCachedCanvas: Option<NonNull<SkCanvas<'device>>>,
    dispatch: Option<SurfaceDispatch<'device>>,
    bound_address: Option<NonNull<Self>>,
    pub(crate) surface: SkSurfaceState,
    pub(crate) ref_count: SkRefCnt,
    pinned: PhantomPinned,
}
impl<'device> SkSurface_Base<'device> {
    // Source: src/image/SkSurface_Base.cpp:34-36
    pub(crate) fn new(
        width: i32,
        height: i32,
        props: Option<&SkSurfaceProps>,
        storage: Option<Arc<dyn SkPixelStorageObject>>,
    ) -> Self {
        let ref_count = SkRefCnt::new();
        let surface = SkSurfaceState::new(width, height, props);
        Self {
            fPixelStorage: storage,
            fCachedImage: None,
            fOwnedBaseCanvas: None,
            fCachedCanvas: None,
            dispatch: None,
            bound_address: None,
            surface,
            ref_count,
            pinned: PhantomPinned,
        }
    }
    // Source: src/image/SkSurface_Base.cpp:38-40
    pub(crate) fn from_info(
        info: &SkImageInfo,
        props: Option<&SkSurfaceProps>,
        storage: Option<Arc<dyn SkPixelStorageObject>>,
    ) -> Self {
        let ref_count = SkRefCnt::new();
        let surface = SkSurfaceState::from_info(info, props);
        Self {
            fPixelStorage: storage,
            fCachedImage: None,
            fOwnedBaseCanvas: None,
            fCachedCanvas: None,
            dispatch: None,
            bound_address: None,
            surface,
            ref_count,
            pinned: PhantomPinned,
        }
    }
}
// Source: src/image/SkSurface_Base.cpp:42-49
impl Drop for SkSurface_Base<'_> {
    fn drop(&mut self) {
        if let Some(canvas) = self.fCachedCanvas {
            // Detach before the genuine selected base-Canvas deletion hook.
            unsafe {
                SkCanvas::set_surface_base_raw(canvas, None);
                SkCanvas::on_surface_delete_raw(canvas);
            }
        }
        // Fields then drop in the explicit native reverse order above.
    }
}

impl<'device> SkSurface_Base<'device> {
    pub(crate) fn bound_address(&self) -> NonNull<Self> {
        self.bound_address
            .expect("complete Surface must bind after pinning")
    }
    /// Safety: base is embedded in its FINAL pinned complete owner. Its callbacks
    /// are that owner's real virtuals and the owner outlives all raw Canvas links.
    /// Binding neither creates nor transfers a second owned reference.
    pub(crate) unsafe fn bind_raw(base: NonNull<Self>, dispatch: SurfaceDispatch<'device>) {
        let slot = unsafe { core::ptr::addr_of_mut!((*base.as_ptr()).dispatch) };
        debug_assert!(unsafe { (*slot).is_none() });
        unsafe {
            slot.write(Some(dispatch));
            core::ptr::addr_of_mut!((*base.as_ptr()).bound_address).write(Some(base));
        }
    }
    unsafe fn dispatch_raw(base: NonNull<Self>) -> SurfaceDispatch<'device> {
        unsafe { core::ptr::addr_of!((*base.as_ptr()).dispatch).read() }
            .expect("complete pinned Surface owner must bind its native virtual slots")
    }
    // Source: src/image/SkSurface_Base.h:85-85
    pub(crate) unsafe fn new_surface_raw(
        base: NonNull<Self>,
        info: &SkImageInfo,
    ) -> Option<
        crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'device> + 'device>,
    > {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.on_new_surface)(base, info) }
    }
    // Source: src/image/SkSurface_Base.h:100-100
    pub(crate) unsafe fn write_pixels_raw(
        base: NonNull<Self>,
        source: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<(), crate::RasterError> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.on_write_pixels)(base, source, x, y) }
    }
    // Source: src/image/SkSurface_Base.h:67-67
    pub(crate) unsafe fn isRasterBacked_raw(base: NonNull<Self>) -> bool {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.type_)(base) == Type::kRaster }
    }
    // Source: src/image/SkSurface_Base.h:69-69
    pub(crate) unsafe fn isGaneshBacked_raw(base: NonNull<Self>) -> bool {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.type_)(base) == Type::kGanesh }
    }
    // Source: src/image/SkSurface_Base.h:71-71
    pub(crate) unsafe fn isGraphiteBacked_raw(base: NonNull<Self>) -> bool {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.type_)(base) == Type::kGraphite }
    }
    // Source: src/image/SkSurface_Base.cpp:54-54
    pub(crate) unsafe fn baseRecorder_raw(
        base: NonNull<Self>,
    ) -> Option<&'device dyn crate::include::core::SkRecorder::SkRecorder> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        match dispatch.on_get_base_recorder {
            Some(callback) => unsafe { callback(base) },
            None => None,
        }
    }
    // Source: src/image/SkSurface_Base.cpp:56-62
    /// Safety: base and destination Canvas remain actual live pinned owners;
    /// snapshot/COW callbacks must not overlap whole-Surface/Canvas references.
    unsafe fn onDraw_default_raw<'target>(
        base: NonNull<Self>,
        canvas: NonNull<SkCanvas<'target>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let image = unsafe { Self::make_temporary_image_raw(base) }?;
        if let Some(image) = image {
            unsafe { SkCanvas::drawImage_raw(canvas, Some(&image), x, y, sampling, paint) }?;
        }
        Ok(())
    }
    // Source: src/image/SkSurface_Base.h:96-96
    pub(crate) unsafe fn new_image_snapshot_raw(
        base: NonNull<Self>,
        subset: Option<&SkIRect>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        match dispatch.on_new_image_snapshot {
            Some(callback) => unsafe { callback(base, subset) },
            None => Ok(None), // Original base virtual returns nullptr.
        }
    }
    // Source: src/image/SkSurface_Base.h:98-98
    // Absent override selects the actual native makeImageSnapshot default;
    // it never means an empty image or an unavailable backend result.
    pub(crate) unsafe fn make_temporary_image_raw(
        base: NonNull<Self>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(callback) = dispatch.on_make_temporary_image {
            unsafe { callback(base) }
        } else {
            unsafe { Self::refCachedImage_raw(base) }
        }
    }
    // Source: src/image/SkSurface_Base.h:133-133
    // Raster supplies its actual override. No override selects the complete
    // source-derived temporary-image draw implementation above, never a no-op.
    pub(crate) unsafe fn draw_raw<'target>(
        base: NonNull<Self>,
        canvas: NonNull<SkCanvas<'target>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(callback) = dispatch.on_draw {
            unsafe { callback(base, canvas, x, y, sampling, paint) }
        } else {
            unsafe { Self::onDraw_default_raw(base, canvas, x, y, sampling, paint) }
        }
    }
    // Source: src/image/SkSurface_Base.h:139-139
    unsafe fn discard_raw(base: NonNull<Self>) {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(callback) = dispatch.on_discard {
            unsafe {
                callback(base);
            }
        }
        // Otherwise use the original base virtual's empty body.
    }
    // Source: src/image/SkSurface_Base.h:154-154
    unsafe fn restore_backing_mutability_raw(base: NonNull<Self>) {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(callback) = dispatch.on_restore_backing_mutability {
            unsafe {
                callback(base);
            }
        }
    }
    // Source: src/image/SkSurface_Base.h:171-171
    pub(crate) fn getPixelStorage(&self) -> Option<Arc<dyn SkPixelStorageObject>> {
        self.fPixelStorage.clone()
    }
    // Source: src/image/SkSurface_Base.h:182-182
    pub(crate) fn hasCachedImage(&self) -> bool {
        self.fCachedImage.is_some()
    }

    // Source: src/image/SkSurface_Base.h:211-226
    pub(crate) unsafe fn getCachedCanvas_raw(
        base: NonNull<Self>,
    ) -> Option<NonNull<SkCanvas<'device>>> {
        let cached = unsafe { core::ptr::addr_of!((*base.as_ptr()).fCachedCanvas).read() };
        if cached.is_none() {
            let dispatch = unsafe { Self::dispatch_raw(base) };
            let owned = unsafe { (dispatch.on_new_canvas)(base) };
            let slot = unsafe { core::ptr::addr_of_mut!((*base.as_ptr()).fOwnedBaseCanvas) };
            unsafe {
                let old = slot.replace(owned);
                drop(old);
            }
            let canvas = {
                let owned = unsafe { &mut *slot };
                owned.as_mut().map(|owner| unsafe {
                    NonNull::from(owner.as_mut().canvas().get_unchecked_mut())
                })
            };
            if let Some(canvas) = canvas {
                unsafe {
                    SkCanvas::set_surface_base_raw(canvas, Some(base));
                }
            }
            let capture = if let Some(recorder) = unsafe { Self::baseRecorder_raw(base) } {
                recorder.makeCaptureCanvas(canvas)
            } else {
                None
            };
            unsafe {
                core::ptr::addr_of_mut!((*base.as_ptr()).fCachedCanvas).write(capture.or(canvas));
            }
        }
        unsafe { core::ptr::addr_of!((*base.as_ptr()).fCachedCanvas).read() }
    }

    // Source: src/image/SkSurface_Base.h:228-239
    pub(crate) unsafe fn refCachedImage_raw(
        base: NonNull<Self>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let existing = { unsafe { &*core::ptr::addr_of!((*base.as_ptr()).fCachedImage) }.clone() };
        if existing.is_some() {
            return Ok(existing);
        }
        unsafe {
            Self::createCaptureBreakpoint_raw(base);
        }
        let image = unsafe { Self::new_image_snapshot_raw(base, None) }?;
        let old = unsafe { core::ptr::addr_of_mut!((*base.as_ptr()).fCachedImage).replace(image) };
        drop(old);
        unsafe {
            Self::validate_canvas_identity_raw(base, b"src/image/SkSurface_Base.h\0".as_ptr(), 236);
        }
        Ok(unsafe { &*core::ptr::addr_of!((*base.as_ptr()).fCachedImage) }.clone())
    }
    // Source: src/image/SkSurface_Base.cpp:64-87
    /// Safety: base and owner project the same original pinned Surface. Short
    /// Canvas/info projections end before callbacks or owner mutation; backing
    /// and callback leases survive the complete synchronous rescale operation.
    pub(crate) unsafe fn async_rescale_pixels_raw(
        base: NonNull<Self>,
        owner: NonNull<dyn super::SkSurface::SkSurface<'device> + 'device>,
        info: &SkImageInfo,
        origSrcRect: SkIRect,
        rescaleGamma: super::SkSurface::RescaleGamma,
        rescaleMode: super::SkSurface::RescaleMode,
        callback: super::SkSurface::ReadPixelsCallback,
        context: super::SkSurface::ReadPixelsContext,
    ) -> Result<(), crate::RasterError> {
        let dispatch = Self::dispatch_raw(base);
        if let Some(on_read) = dispatch.on_async_rescale_pixels {
            return on_read(
                base,
                info,
                origSrcRect,
                rescaleGamma,
                rescaleMode,
                callback,
                context,
            );
        }
        let mut src = crate::src::core::SkBitmap::SkBitmap::default();
        let mut peek = crate::src::core::SkPixmap::SkPixmap::default();
        let srcRect;
        let mut canvas = Self::getCachedCanvas_raw(base).expect("native complete Surface canvas");
        if canvas.as_ref().peekPixels(Some(&mut peek)) {
            src.installPixmap(&peek);
            srcRect = origSrcRect;
        } else {
            let sourceInfo = owner
                .as_ref()
                .imageInfo()
                .makeDimensions(origSrcRect.size());
            src.setInfo(&sourceInfo, 0);
            src.allocPixels_current();
            if !Pin::new_unchecked(canvas.as_mut()).readPixels_bitmap(
                &src,
                origSrcRect.x(),
                origSrcRect.y(),
            )? {
                callback(context, None);
                return Ok(());
            }
            srcRect = SkIRect::make_size(src.dimensions());
        }
        super::SkRescaleAndReadPixels::SkRescaleAndReadPixels(
            src,
            info,
            &srcRect,
            rescaleGamma,
            rescaleMode,
            callback,
            context,
        )
    }

    // Source: src/image/SkSurface_Base.cpp:89-96
    /// Safety: the complete pinned owner and callback/context remain valid.
    /// Native Raster inherits this exact base failure callback. An override is
    /// invoked on its actual complete owner, without a whole-owner borrow.
    pub(crate) unsafe fn async_rescale_yuv420_raw(
        base: NonNull<Self>,
        yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
        read_alpha: bool,
        dst_color_space: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        src_rect: SkIRect,
        dst_size: crate::include::core::SkSize::SkISize,
        rescale_gamma: super::SkSurface::RescaleGamma,
        rescale_mode: super::SkSurface::RescaleMode,
        callback: super::SkSurface::ReadPixelsCallback,
        context: super::SkSurface::ReadPixelsContext,
    ) {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(on_read) = dispatch.on_async_rescale_yuv420 {
            unsafe {
                on_read(
                    base,
                    yuv_color_space,
                    read_alpha,
                    dst_color_space,
                    src_rect,
                    dst_size,
                    rescale_gamma,
                    rescale_mode,
                    callback,
                    context,
                );
            }
        } else {
            unsafe {
                callback(context, None);
            }
        }
    }

    // Source: src/image/SkSurface_Base.cpp:98-100
    pub(crate) unsafe fn outstandingImageSnapshot_raw(base: NonNull<Self>) -> bool {
        let image = unsafe { &*core::ptr::addr_of!((*base.as_ptr()).fCachedImage) };
        image
            .as_ref()
            .is_some_and(|image| Arc::strong_count(image) != 1)
    }
    // Source: src/image/SkSurface_Base.cpp:102-132
    /// Safety: enter from the actual raw Surface/Canvas operation, with NO live
    /// whole Canvas, Surface, Device, Pixmap or pixel reference across COW. A
    /// complete pinned owner and its sole Canvas remain live for this operation.
    /// Every raw callback must end its temporary projections before returning.
    pub(crate) unsafe fn aboutToDraw_raw(base: NonNull<Self>, mode: ContentChangeMode) -> bool {
        unsafe {
            (&*core::ptr::addr_of!((*base.as_ptr()).surface)).dirtyGenerationID();
        }
        unsafe {
            Self::validate_canvas_identity_raw(
                base,
                b"src/image/SkSurface_Base.cpp\0".as_ptr(),
                105,
            );
        }
        // Test uniqueness WITHOUT cloning the cached image. End the field borrow
        // before any virtual call, so the concrete COW may mutate its real fields.
        let unique = {
            let image = unsafe { &*core::ptr::addr_of!((*base.as_ptr()).fCachedImage) };
            image.as_ref().map(|image| Arc::strong_count(image) == 1)
        };
        let dispatch = unsafe { Self::dispatch_raw(base) };
        if let Some(unique) = unique {
            if !unique && !unsafe { (dispatch.on_copy_on_write)(base, mode) } {
                return false;
            }
            let image = unsafe { core::ptr::addr_of_mut!((*base.as_ptr()).fCachedImage) };
            // Publish null before releasing the old retained image, as native reset().
            drop(unsafe { (*image).take() });
            if unique {
                unsafe {
                    Self::restore_backing_mutability_raw(base);
                }
            }
        } else if mode == ContentChangeMode::kDiscard_ContentChangeMode {
            unsafe {
                Self::discard_raw(base);
            }
        }
        true
    }
    unsafe fn validate_canvas_identity_raw(
        base: NonNull<Self>,
        native_file: *const u8,
        native_line: u32,
    ) {
        #[cfg(sk_debug)]
        {
            let canvas = {
                let owned = unsafe { &*core::ptr::addr_of!((*base.as_ptr()).fOwnedBaseCanvas) };
                owned
                    .as_ref()
                    .map(|canvas| NonNull::from(canvas.as_ref().get_ref().canvas_ref()))
            };
            if let Some(canvas) = canvas {
                crate::SkASSERT!(@source native_file,native_line; @condition b"!fOwnedBaseCanvas || fOwnedBaseCanvas->getSurface() == this\0".as_ptr(); unsafe{SkCanvas::get_surface_base_raw(canvas)}==Some(base));
            }
        }
    }
    // Source: src/image/SkSurface_Base.cpp:134-138
    pub(crate) unsafe fn newGenerationID_raw(base: NonNull<Self>) -> u32 {
        unsafe {
            Self::validate_canvas_identity_raw(
                base,
                b"src/image/SkSurface_Base.cpp\0".as_ptr(),
                135,
            );
        }
        static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl<'device> SkSurface_Base<'device> {
    // Source: src/image/SkSurface_Base.cpp:140-142
    pub(crate) fn onCapabilities() -> Arc<crate::include::core::SkCapabilities::SkCapabilities> {
        crate::include::core::SkCapabilities::SkCapabilities::RasterBackend()
    }
    // Source: src/image/SkSurface_Base.cpp:144-148
    unsafe fn createCaptureBreakpoint_raw(base: NonNull<Self>) {
        if let Some(recorder) = unsafe { Self::baseRecorder_raw(base) } {
            recorder.createCaptureBreakpoint(base);
        }
    }
}

impl<'device> SkSurface_Base<'device> {
    // Source: src/image/SkSurface_Base.h:169-169
    pub(crate) unsafe fn capabilities_raw(
        base: NonNull<Self>,
    ) -> Arc<crate::include::core::SkCapabilities::SkCapabilities> {
        let dispatch = unsafe { Self::dispatch_raw(base) };
        unsafe { (dispatch.on_capabilities)(base) }
    }
}

impl<'d> SkSurface_Base<'d> {
    // Source: src/image/SkSurface_Base.h:73-73
    // Source: src/image/SkSurface_Base.cpp:51-51
    pub(crate) unsafe fn get_recording_context_raw(
        base: NonNull<Self>,
    ) -> *mut super::SkSurface::GrRecordingContext {
        match unsafe { Self::dispatch_raw(base) }.on_get_recording_context {
            Some(f) => unsafe { f(base) },
            None => core::ptr::null_mut(),
        }
    }
    // Source: src/image/SkSurface_Base.h:74-74
    // Source: src/image/SkSurface_Base.cpp:53-53
    pub(crate) unsafe fn get_graphite_recorder_raw(
        base: NonNull<Self>,
    ) -> *mut super::SkSurface::skgpu::graphite::Recorder {
        match unsafe { Self::dispatch_raw(base) }.on_get_graphite_recorder {
            Some(f) => unsafe { f(base) },
            None => core::ptr::null_mut(),
        }
    }
    // Source: src/image/SkSurface_Base.h:161-164
    pub(crate) unsafe fn wait_raw(
        base: NonNull<Self>,
        n: i32,
        semaphores: *const super::SkSurface::GrBackendSemaphore,
        delete_after_wait: bool,
    ) -> bool {
        match unsafe { Self::dispatch_raw(base) }.on_wait {
            Some(f) => unsafe { f(base, n, semaphores, delete_after_wait) },
            None => false,
        }
    }
    // Source: src/image/SkSurface_Base.h:166-166
    pub(crate) unsafe fn characterize_raw(
        base: NonNull<Self>,
        characterization: *mut super::SkSurface::GrSurfaceCharacterization,
    ) -> bool {
        match unsafe { Self::dispatch_raw(base) }.on_characterize {
            Some(f) => unsafe { f(base, characterization) },
            None => false,
        }
    }
    // Source: src/image/SkSurface_Base.h:167-167
    pub(crate) unsafe fn is_compatible_raw(
        base: NonNull<Self>,
        characterization: *const super::SkSurface::GrSurfaceCharacterization,
    ) -> bool {
        match unsafe { Self::dispatch_raw(base) }.on_is_compatible {
            Some(f) => unsafe { f(base, characterization) },
            None => false,
        }
    }
    // Source: src/image/SkSurface_Base.h:49-55
    // This is the real inherited Base override, not a default for Surface's pure virtual.
    pub(crate) unsafe fn replace_backend_texture_raw(
        base: NonNull<Self>,
        texture: *const super::SkSurface::GrBackendTexture,
        origin: super::SkSurface::GrSurfaceOrigin,
        mode: ContentChangeMode,
        release: Option<super::SkSurface::TextureReleaseProc>,
        context: super::SkSurface::ReleaseContext,
    ) -> bool {
        match unsafe { Self::dispatch_raw(base) }.on_replace_backend_texture {
            Some(f) => unsafe { f(base, texture, origin, mode, release, context) },
            None => false,
        }
    }
}
