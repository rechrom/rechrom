use crate::src::core::SkImageInfo::SkImageInfo;
// Source: src/image/SkSurface_Raster.cpp:39-42
// Source: src/image/SkSurface_Raster.cpp:44-46
// Source: src/image/SkSurface_Raster.cpp:48-50
// Source: src/image/SkSurface_Raster.cpp:52-56
// Source: src/image/SkSurface_Raster.cpp:58-59
pub fn SkSurfaceValidateRasterInfo(info: &SkImageInfo, rowBytes: usize) -> bool {
    if !crate::src::core::SkImageInfoPriv::SkImageInfoIsValid(info) {
        return false;
    }
    if crate::src::core::SkSurfacePriv::kIgnoreRowBytesValue == rowBytes {
        return true;
    }
    if !info.validRowBytes(rowBytes) {
        return false;
    }
    // Native sk_64_mul takes two int64_t arguments before converting to uint64_t.
    // Signed-overflow inputs remain outside the original function's defined domain.
    let size =
        crate::include::private::SkMath::sk_64_mul(info.height() as i64, rowBytes as i64) as u64;
    let kMaxTotalSize = crate::include::private::SkMath::SK_MaxS32 as usize;
    if size > kMaxTotalSize as u64 {
        return false;
    }
    true
}

use super::{
    SkImage::SkImage,
    SkSurface::{ContentChangeMode, SkSurface, SkSurfaceState},
    SkSurface_Base::{SkSurface_Base, SurfaceDispatch},
};
use crate::include::core::{
    SkRefCnt::{sk_sp, SkRefCntObject},
    SkSurfaceProps::SkSurfaceProps,
};
use crate::src::core::{
    SkBitmap::SkBitmap,
    SkCPURecorderImpl::RecorderImpl,
    SkCanvas::{SkCanvas, SkCanvasOwnerHandle},
    SkPixelRef::SkPixelRef,
    SkPixelStorage::SkPixelStorageObject,
    SkRect::SkIRect,
};
use core::{pin::Pin, ptr::NonNull};
use std::sync::Arc;

// Source: src/image/SkSurface_Raster.h:69-71
// Native derived fields destroy bitmap before Base. Recorder/context remain
// BORROWED with their real lifetimes; Base pins the entire complete allocation.
pub struct SkSurface_Raster<'rec, 'ctx> {
    // Native reverse member destruction, followed by parent last. These are
    // original fields; recorder is a borrow, never a retained replacement owner.
    // Source: src/image/SkSurface_Raster.h:71.
    fWeOwnThePixels: bool,
    // Source: src/image/SkSurface_Raster.h:70.
    fBitmap: SkBitmap,
    // Source: src/image/SkSurface_Raster.h:69.
    fRecorder: &'rec RecorderImpl<'ctx>,
    base: SkSurface_Base<'rec>,
}
impl SkSurface_Raster<'static, 'static> {
    // Source: src/image/SkSurface_Raster.cpp:61-68
    /// Safety: preserve native exclusive external backing/release-context lease.
    pub unsafe fn from_pixels_default(
        info: &SkImageInfo,
        pixels: *mut core::ffi::c_void,
        row_bytes: usize,
        release: Option<super::SkSurface::SkSurfaces::PixelsReleaseProc>,
        context: *mut core::ffi::c_void,
        props: Option<&SkSurfaceProps>,
    ) -> Pin<Box<Self>> {
        unsafe {
            Self::from_pixels(
                crate::src::core::SkCPURecorder::Recorder::TODO(),
                info,
                pixels,
                row_bytes,
                release,
                context,
                props,
            )
        }
    }
    // Source: src/image/SkSurface_Raster.cpp:70-73
    /// Safety: pixel reference retains the original writable native pixel owner.
    pub unsafe fn from_pixel_ref_default(
        info: &SkImageInfo,
        pixel_ref: Arc<SkPixelRef>,
        props: Option<&SkSurfaceProps>,
    ) -> Pin<Box<Self>> {
        unsafe {
            Self::from_pixel_ref(
                crate::src::core::SkCPURecorder::Recorder::TODO(),
                info,
                pixel_ref,
                props,
            )
        }
    }
}
impl<'rec, 'ctx> SkSurface_Raster<'rec, 'ctx> {
    // Source: src/image/SkSurface_Raster.cpp:75-87
    /// Safety: pixels/release/context establish the complete exclusive native
    /// Surface backing lease until release; the actual recorder outlives Surface,
    /// every cached Canvas operation, and its real borrowed device/context views.
    pub unsafe fn from_pixels(
        recorder: &'rec RecorderImpl<'ctx>,
        info: &SkImageInfo,
        pixels: *mut core::ffi::c_void,
        row_bytes: usize,
        release: Option<super::SkSurface::SkSurfaces::PixelsReleaseProc>,
        context: *mut core::ffi::c_void,
        props: Option<&SkSurfaceProps>,
    ) -> Pin<Box<Self>> {
        // Native Base and recorder initialize before bitmap installation (and
        // any actual release callback), independent of Rust reverse field order.
        let mut base = SkSurface_Base::from_info(info, props, None);
        let fRecorder = recorder;
        let mut bitmap = SkBitmap::default();
        // Native constructor intentionally does not branch on installPixels' result.
        unsafe {
            bitmap.installPixels(info, pixels, row_bytes, release, context);
        }
        base.fPixelStorage = bitmap
            .pixelRef()
            .cloned()
            .map(|owner| owner as Arc<dyn SkPixelStorageObject>);
        let fWeOwnThePixels = false;
        let owner = Box::pin(Self {
            fBitmap: bitmap,
            fRecorder,
            fWeOwnThePixels,
            base,
        });
        unsafe { Self::bind_complete(owner) }
    }
    // Source: src/image/SkSurface_Raster.cpp:89-97
    /// Safety: the pixel reference supplies the valid native image layout and
    /// exclusive live mutable storage; recorder/context remain truly borrowed.
    pub unsafe fn from_pixel_ref(
        recorder: &'rec RecorderImpl<'ctx>,
        info: &SkImageInfo,
        pixel_ref: Arc<SkPixelRef>,
        props: Option<&SkSurfaceProps>,
    ) -> Pin<Box<Self>> {
        let base = SkSurface_Base::new(
            pixel_ref.width(),
            pixel_ref.height(),
            props,
            Some(pixel_ref.clone() as Arc<dyn SkPixelStorageObject>),
        );
        let fRecorder = recorder;
        let mut bitmap = SkBitmap::default();
        bitmap.setInfo(info, pixel_ref.rowBytes());
        bitmap.setPixelRef(Some(pixel_ref), 0, 0);
        let fWeOwnThePixels = true;
        let owner = Box::pin(Self {
            fBitmap: bitmap,
            fRecorder,
            fWeOwnThePixels,
            base,
        });
        unsafe { Self::bind_complete(owner) }
    }
    unsafe fn bind_complete(mut owner: Pin<Box<Self>>) -> Pin<Box<Self>> {
        // This is the original complete allocation's raw provenance, obtained
        // AFTER pinning. No mutable field/base projection escapes independently.
        let whole = unsafe { owner.as_mut().get_unchecked_mut() } as *mut Self;
        let base = unsafe { NonNull::new_unchecked(core::ptr::addr_of_mut!((*whole).base)) };
        unsafe {
            SkSurface_Base::bind_raw(
                base,
                SurfaceDispatch {
                    // Actual inherited native Base null/false ordinary virtuals.
                    on_get_recording_context: None,
                    on_get_graphite_recorder: None,
                    on_wait: None,
                    on_characterize: None,
                    on_is_compatible: None,
                    on_replace_backend_texture: None,
                    on_capabilities: Self::capabilities_callback,
                    on_get_base_recorder: Some(Self::recorder_callback),
                    type_: Self::type_callback,
                    on_new_canvas: Self::canvas_callback,
                    on_copy_on_write: Self::cow_callback,
                    on_new_surface: Self::new_surface_callback,
                    on_write_pixels: Self::write_pixels_callback,
                    on_async_rescale_pixels: None, // Actual inherited native RGB rescale implementation.
                    on_async_rescale_yuv420: None, // Actual inherited native base callback.
                    on_draw: Some(Self::onDraw_raw),
                    on_make_temporary_image: None, // Native inherited real snapshot default.
                    on_restore_backing_mutability: Some(Self::restore_callback),
                    on_discard: None,
                    on_new_image_snapshot: Some(Self::snapshot_callback),
                },
            );
        }
        owner
    }
    unsafe fn whole_from_base(base: NonNull<SkSurface_Base<'rec>>) -> *mut Self {
        // Native inheritance adjustment uses Rust's ACTUAL concrete field offset;
        // no C++ ABI offset, fat-pointer reinterpretation or erased lifetime.
        unsafe {
            base.as_ptr()
                .cast::<u8>()
                .sub(core::mem::offset_of!(Self, base))
                .cast()
        }
    }
    // Source: src/image/SkSurface_Raster.h:56-56
    unsafe fn type_callback(_base: NonNull<SkSurface_Base<'rec>>) -> super::SkSurface_Base::Type {
        super::SkSurface_Base::Type::kRaster
    }
    unsafe fn new_surface_callback(
        base: NonNull<SkSurface_Base<'rec>>,
        info: &SkImageInfo,
    ) -> Option<sk_sp<dyn SkSurface<'rec> + 'rec>> {
        unsafe { Self::onNewSurface_raw(base, info) }
    }
    unsafe fn write_pixels_callback(
        base: NonNull<SkSurface_Base<'rec>>,
        source: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::onWritePixels_raw(Self::whole_from_base(base), source, x, y) }
    }
    unsafe fn canvas_callback(
        base: NonNull<SkSurface_Base<'rec>>,
    ) -> Option<SkCanvasOwnerHandle<'rec>> {
        unsafe { Self::onNewCanvas_raw(Self::whole_from_base(base)) }
    }
    unsafe fn cow_callback(base: NonNull<SkSurface_Base<'rec>>, mode: ContentChangeMode) -> bool {
        unsafe { Self::onCopyOnWrite_raw(Self::whole_from_base(base), mode) }
    }
    unsafe fn restore_callback(base: NonNull<SkSurface_Base<'rec>>) {
        unsafe {
            Self::onRestoreBackingMutability_raw(Self::whole_from_base(base));
        }
    }
    unsafe fn snapshot_callback(
        base: NonNull<SkSurface_Base<'rec>>,
        subset: Option<&SkIRect>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        unsafe { Self::onNewImageSnapshot_raw(Self::whole_from_base(base), subset) }
    }
    // Source: src/image/SkSurface_Raster.cpp:99-102
    unsafe fn onNewCanvas_raw(whole: *mut Self) -> Option<SkCanvasOwnerHandle<'rec>> {
        let recorder = unsafe { core::ptr::addr_of!((*whole).fRecorder).read() };
        let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
        let props = unsafe { &*core::ptr::addr_of!((*whole).base) }
            .surface
            .props();
        let device = unsafe {
            crate::src::core::SkBitmapDevice::SkBitmapDevice::with_recorder_props(
                Some(recorder),
                bitmap,
                props,
                core::ptr::null_mut(),
            )
        };
        let device = unsafe { crate::src::core::SkDevice::adopt_pinned_device(device) };
        let owner: SkCanvasOwnerHandle<'rec> =
            unsafe { SkCanvas::with_device_props_allocator(device, *props, None) };
        Some(owner)
    }
    // Source: src/image/SkSurface_Raster.cpp:104-107
    unsafe fn onNewSurface_raw(
        base: NonNull<SkSurface_Base<'rec>>,
        info: &SkImageInfo,
    ) -> Option<sk_sp<dyn SkSurface<'rec> + 'rec>> {
        let whole = unsafe { Self::whole_from_base(base) };
        let recorder = unsafe { core::ptr::addr_of!((*whole).fRecorder).read() };
        let props = unsafe { (*base.as_ptr()).surface.props().clone() };
        recorder.makeBitmapSurfaceRowBytes(info, 0, Some(&props))
    }
    // Source: src/image/SkSurface_Raster.cpp:108-111
    unsafe fn onDraw_raw<'target>(
        base: NonNull<SkSurface_Base<'rec>>,
        canvas: NonNull<SkCanvas<'target>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let whole = unsafe { Self::whole_from_base(base) };
        // Release the source bitmap field projection before destination Canvas
        // callbacks: asImage retains/copies the original backing as native does.
        let image = {
            let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
            unsafe { bitmap.asImage() }
        };
        unsafe { SkCanvas::drawImage_raw(canvas, image.as_ref(), x, y, sampling, paint) }
    }
    // Source: src/image/SkSurface_Raster.cpp:114-137
    unsafe fn onNewImageSnapshot_raw(
        whole: *mut Self,
        subset: Option<&SkIRect>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
        if let Some(subset) = subset {
            crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),115; @condition b"SkIRect::MakeWH(fBitmap.width(), fBitmap.height()).contains(*subset)\0".as_ptr(); SkIRect::make_wh(bitmap.width(),bitmap.height()).contains_rect(subset));
            let mut dst = SkBitmap::default();
            dst.allocPixels(&bitmap.info().makeDimensions(subset.size()));
            crate::SkAssertResult!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),118; @condition b"fBitmap.readPixels(dst.pixmap(), subset->left(), subset->top())\0".as_ptr(); unsafe{bitmap.readPixels(dst.pixmap(),subset.fLeft,subset.fTop)}?);
            dst.setImmutable();
            return Ok(unsafe { dst.asImage() });
        }
        let mut mode = super::SkImage_Raster::SkCopyPixelsMode::kIfMutable;
        if unsafe { core::ptr::addr_of!((*whole).fWeOwnThePixels).read() } {
            if let Some(pixel_ref) = bitmap.pixelRef() {
                pixel_ref.setTemporarilyImmutable();
            }
        } else {
            mode = super::SkImage_Raster::SkCopyPixelsMode::kAlways;
        }
        Ok(
            unsafe { super::SkImage_Raster::SkImage_Raster::MakeFromBitmap(bitmap, mode, None) }
                .map(|image| image as Arc<dyn SkImage>),
        )
    }
    // Source: src/image/SkSurface_Raster.cpp:139-141
    // Called through the original Base write path only AFTER aboutToDraw /
    // notifyContentWillChange has performed the native COW decision. A borrowed
    // source Pixmap must still refer to live backing across that invalidation;
    // do not extend an escaped Surface peek view by inventing a retain/snapshot.
    unsafe fn onWritePixels_raw(
        whole: *mut Self,
        src: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<(), crate::RasterError> {
        let _ =
            unsafe { (&mut *core::ptr::addr_of_mut!((*whole).fBitmap)).writePixels(src, x, y) }?;
        Ok(())
    }
    // Source: src/image/SkSurface_Raster.cpp:143-148
    unsafe fn onRestoreBackingMutability_raw(whole: *mut Self) {
        crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),144; @condition b"!this->hasCachedImage()\0".as_ptr(); !unsafe{&*core::ptr::addr_of!((*whole).base)}.hasCachedImage());
        let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
        if let Some(pixel_ref) = bitmap.pixelRef() {
            pixel_ref.restoreMutability();
        }
    }
    // Source: src/image/SkSurface_Raster.cpp:150-179
    /// No whole Surface/Base/Canvas/Device or pixel reference crosses the actual
    /// Canvas backlink callback. Each bitmap lease ends before the next stage.
    unsafe fn onCopyOnWrite_raw(whole: *mut Self, mode: ContentChangeMode) -> bool {
        let base = unsafe { NonNull::new_unchecked(core::ptr::addr_of_mut!((*whole).base)) };
        // This retain is AFTER the Base's un-cloned unique decision.
        let cached = { unsafe { &*core::ptr::addr_of!((*whole).base.fCachedImage) }.clone() }
            .expect("native COW is called only for a cached image");
        let raster = cached
            .asImageBase()
            .asRaster()
            .expect("native cached Raster Surface snapshot is Raster");
        let shared = {
            let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
            match (raster.getPixelRef(), bitmap.pixelRef()) {
                (Some(image), Some(surface)) => Arc::ptr_eq(image, surface),
                (None, None) => true,
                _ => false,
            }
        };
        if shared {
            crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),156; @condition b"fWeOwnThePixels\0".as_ptr(); unsafe{core::ptr::addr_of!((*whole).fWeOwnThePixels).read()});
            let bitmap = unsafe { core::ptr::addr_of_mut!((*whole).fBitmap) };
            if mode == ContentChangeMode::kDiscard_ContentChangeMode {
                if !unsafe { (&mut *bitmap).tryAllocPixels_current() } {
                    return false;
                }
            } else {
                let previous = unsafe { &*bitmap }.clone();
                if !unsafe { (&mut *bitmap).tryAllocPixels_current() } {
                    return false;
                }
                {
                    let current = unsafe { &*bitmap };
                    crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),166; @condition b"prev.info() == fBitmap.info()\0".as_ptr(); previous.info()==current.info());
                    crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),167; @condition b"prev.rowBytes() == fBitmap.rowBytes()\0".as_ptr(); previous.rowBytes()==current.rowBytes());
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            previous.getPixels(),
                            current.getPixels(),
                            current.computeByteSize(),
                        );
                    }
                }
            }
            let canvas =
                unsafe { SkSurface_Base::getCachedCanvas_raw(base) }.expect("native cached Canvas");
            let device = unsafe { SkCanvas::root_device_raw(canvas) };
            // The Canvas was made ONLY by onNewCanvas above from this exact
            // SkBitmapDevice<'rec,'ctx>. Rust's concrete data-pointer projection
            // recovers that original complete allocation; no vtable is reinterpreted.
            let bitmap_device = device.as_ptr()
                as *mut crate::src::core::SkBitmapDevice::SkBitmapDevice<'rec, 'ctx>;
            {
                let lease = unsafe { Pin::new_unchecked(&mut *bitmap_device) };
                let bitmap = unsafe { &*core::ptr::addr_of!((*whole).fBitmap) };
                unsafe {
                    lease.replaceBitmapBackendForRasterSurface(bitmap);
                }
            }
        }
        true
    }
    /// Transfer the SAME pinned complete allocation and sole counter into native
    /// dynamic ownership. Never reconstruct Box<Base> or allocate a proxy view.
    pub unsafe fn into_surface(owner: Pin<Box<Self>>) -> sk_sp<dyn SkSurface<'rec> + 'rec> {
        assert!(owner.as_ref().get_ref().base.ref_count.unique());
        let owned = unsafe { Pin::into_inner_unchecked(owner) };
        let raw = Box::into_raw(owned);
        let dynamic: *mut (dyn SkSurface<'rec> + 'rec) = raw;
        unsafe { sk_sp::from_raw(dynamic) }
    }
}
unsafe impl<'rec, 'ctx> SkRefCntObject for SkSurface_Raster<'rec, 'ctx> {
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        &self.base.ref_count
    }
    fn ref_(&self) {
        self.base.ref_count.ref_();
    }
    // Adapted Source: src/image/SkSurface_Raster.h:33-33
    // Concrete destructor dispatch pairs this original complete allocation
    // with its actual Sized type; no trait metadata or owner is reconstructed here.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
}
unsafe impl<'rec, 'ctx> SkSurface<'rec> for SkSurface_Raster<'rec, 'ctx> {
    fn state(&self) -> &SkSurfaceState {
        &self.base.surface
    }
    fn base_address(&self) -> NonNull<SkSurface_Base<'rec>> {
        self.base.bound_address()
    }
    // Source: src/image/SkSurface_Raster.h:53-53
    fn imageInfo(&self) -> SkImageInfo {
        self.fBitmap.info().clone()
    }
}

impl<'rec, 'ctx> SkSurface_Raster<'rec, 'ctx> {
    // Source: src/image/SkSurface_Raster.h:65-65
    // Source: src/image/SkSurface_Raster.cpp:181-183
    unsafe fn capabilities_callback(
        _base: core::ptr::NonNull<SkSurface_Base<'rec>>,
    ) -> std::sync::Arc<crate::include::core::SkCapabilities::SkCapabilities> {
        crate::include::core::SkCapabilities::SkCapabilities::RasterBackend()
    }
    // Source: src/image/SkSurface_Raster.cpp:185-187
    unsafe fn recorder_callback(
        base: NonNull<SkSurface_Base<'rec>>,
    ) -> Option<&'rec dyn crate::include::core::SkRecorder::SkRecorder> {
        let whole = unsafe { Self::whole_from_base(base) };
        Some(unsafe { core::ptr::addr_of!((*whole).fRecorder).read() })
    }
}

// Source: src/image/SkSurface_Raster.cpp:191-208
/// Safety: pixels/release/context provide exclusive live backing for every
/// retained Surface/Canvas operation, and release occurs once at final ownership
/// loss. Snapshot sharing requires the native Surface COW protocol before writes.
pub unsafe fn WrapPixelsWithRelease(
    info: &SkImageInfo,
    pixels: *mut core::ffi::c_void,
    row_bytes: usize,
    release: Option<super::SkSurface::SkSurfaces::PixelsReleaseProc>,
    mut context: *mut core::ffi::c_void,
    props: Option<&SkSurfaceProps>,
) -> Option<sk_sp<dyn SkSurface<'static> + 'static>> {
    if release.is_none() {
        context = core::ptr::null_mut();
    }
    if !SkSurfaceValidateRasterInfo(info, row_bytes) {
        return None;
    }
    if pixels.is_null() {
        return None;
    }
    // Native legacy constructor uses the actual CPU Recorder::TODO singleton;
    // this is its real static allocation, never an extended borrowed recorder.
    Some(unsafe {
        SkSurface_Raster::into_surface(SkSurface_Raster::from_pixels_default(
            info, pixels, row_bytes, release, context, props,
        ))
    })
}
// Source: src/image/SkSurface_Raster.cpp:210-215
/// Safety: caller keeps the external pixels live/exclusive through final surface
/// release; all image snapshots follow the native copy/COW ownership protocol.
pub unsafe fn WrapPixels(
    info: &SkImageInfo,
    pixels: *mut core::ffi::c_void,
    row_bytes: usize,
    props: Option<&SkSurfaceProps>,
) -> Option<sk_sp<dyn SkSurface<'static> + 'static>> {
    unsafe { WrapPixelsWithRelease(info, pixels, row_bytes, None, core::ptr::null_mut(), props) }
}
// Source: src/image/SkSurface_Raster.cpp:217-230
pub fn Raster(
    info: &SkImageInfo,
    row_bytes: usize,
    props: Option<&SkSurfaceProps>,
) -> Option<sk_sp<dyn SkSurface<'static> + 'static>> {
    if !SkSurfaceValidateRasterInfo(info, crate::src::core::SkSurfacePriv::kIgnoreRowBytesValue) {
        return None;
    }
    let pixel_ref = crate::src::core::SkMallocPixelRef::MakeAllocate(info, row_bytes)?;
    if row_bytes != 0 {
        crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),227; @condition b"pr->rowBytes() == rowBytes\0".as_ptr(); pixel_ref.rowBytes()==row_bytes);
    }
    Some(unsafe {
        SkSurface_Raster::into_surface(SkSurface_Raster::from_pixel_ref_default(
            info, pixel_ref, props,
        ))
    })
}

// Source: src/image/SkSurface_Raster.cpp:236-240
impl<'ctx> RecorderImpl<'ctx> {
    /// The surface and its retained native Canvas borrow this real recorder.
    /// Allocation owns fresh pixels, so no external backing lease is required.
    pub fn makeBitmapSurface<'rec>(
        &'rec self,
        info: &SkImageInfo,
        props: Option<&SkSurfaceProps>,
    ) -> Option<sk_sp<dyn SkSurface<'rec> + 'rec>> {
        self.makeBitmapSurfaceRowBytes(info, 0, props)
    }
    // Source: src/image/SkSurface_Raster.cpp:242-257
    pub fn makeBitmapSurfaceRowBytes<'rec>(
        &'rec self,
        info: &SkImageInfo,
        row_bytes: usize,
        props: Option<&SkSurfaceProps>,
    ) -> Option<sk_sp<dyn SkSurface<'rec> + 'rec>> {
        if !SkSurfaceValidateRasterInfo(info, crate::src::core::SkSurfacePriv::kIgnoreRowBytesValue)
        {
            return None;
        }
        let pixel_ref = crate::src::core::SkMallocPixelRef::MakeAllocate(info, row_bytes)?;
        if row_bytes != 0 {
            crate::SkASSERT!(@source b"src/image/SkSurface_Raster.cpp\0".as_ptr(),253; @condition b"pr->rowBytes() == rowBytes\0".as_ptr(); pixel_ref.rowBytes()==row_bytes);
        }
        // Fresh allocation has no other mutable owner. The actual whole object
        // is pinned before its embedded Base/Canvas backlink can be published.
        Some(unsafe {
            SkSurface_Raster::into_surface(SkSurface_Raster::from_pixel_ref(
                self, info, pixel_ref, props,
            ))
        })
    }
}
