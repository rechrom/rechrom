// Source: src/image/SkSurface.cpp:33-33
pub use crate::src::core::SkPaint::SkPaint;
// Source: src/image/SkSurface_Base.h:26-28
// Source: src/image/SkSurface_Base.h:33-33
// Incomplete native classes are usable as pointers/references only.
unsafe extern "C" {
    // Source: src/image/SkSurface.cpp:31-31
    pub type GrBackendSemaphore;
    pub type GrBackendTexture;
    // Source: src/image/SkSurface.cpp:32-32
    pub type GrRecordingContext;
    // Source: src/image/SkSurface.cpp:34-34
    pub type GrSurfaceCharacterization;
}
// Source: src/image/SkSurface_Base.h:35-35
// The opaque native enum explicitly fixes its underlying representation to int.
// Preserve its complete value bits without inventing any enumerator.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GrSurfaceOrigin(pub i32);
// Source: src/image/SkSurface_Base.h:37-37
// Source: src/image/SkSurface.cpp:35-35
pub mod skgpu {
    pub mod graphite {
        unsafe extern "C" {
            pub type Recorder;
        }
    }
}
// Copyright 2012 Google Inc. BSD-style license.
use crate::include::core::SkSurfaceProps::{SkPixelGeometry, SkSurfaceProps};
use crate::include::core::SkTypes::{SK_GAMMA_CONTRAST, SK_GAMMA_EXPONENT};
// Source: include/core/SkSurface.h:43-43
pub mod SkSurfaces {
    // Source: include/core/SkSurface.h:45-47
    #[repr(i32)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BackendSurfaceAccess {
        kNoAccess,
        kPresent,
    }

    // Source: include/core/SkSurface.h:59-59
    pub use super::super::SkSurface_Null::Null;
    // Existing actual factories retain their canonical intrusive Surface owner.
    // Source: include/core/SkSurface.h:78-80
    // Source: include/core/SkSurface.h:109-112
    // Source: include/core/SkSurface.h:143-148
    pub use super::super::SkSurface_Raster::{Raster, WrapPixels, WrapPixelsWithRelease};
    // Source: include/core/SkSurface.h:81-84
    pub fn Raster_default(
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface<'static> + 'static>>
    {
        Raster(info, 0, props)
    }
    // Source: include/core/SkSurface.h:113-115
    /// The external pixmap pixels remain live/exclusive through final Surface
    /// release, including all Canvas work and the real snapshot/COW protocol.
    pub unsafe fn WrapPixels_pixmap(
        pm: &crate::src::core::SkPixmap::SkPixmap,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface<'static> + 'static>>
    {
        unsafe { WrapPixels(pm.info(), pm.writable_addr().cast(), pm.rowBytes(), props) }
    }
    // Source: include/core/SkSurface.h:117-117
    // Native function type becomes a procedure pointer; nullability belongs to
    // the owning WrapPixels argument, not to this callback type.
    pub type PixelsReleaseProc =
        unsafe extern "C" fn(pixels: *mut core::ffi::c_void, context: *mut core::ffi::c_void);
}
// Native protected Surface base storage only; this does not implement a concrete Surface.
// The actual CPU virtual contract and derived ownership remain separate from this base state.
// Cell preserves native non-atomic mutation through surface methods; no Sync promise is added.
// Source: include/core/SkSurface.h:179-179
impl SkSurfaceState {
    pub fn width(&self) -> i32 {
        self.fWidth
    }
}
// Source: include/core/SkSurface.h:185-185
impl SkSurfaceState {
    pub fn height(&self) -> i32 {
        self.fHeight
    }
}
// Source: include/core/SkSurface.h:204-207
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ContentChangeMode {
    kDiscard_ContentChangeMode,
    kRetain_ContentChangeMode,
}
// Source: include/core/SkSurface.h:488-488
pub use super::SkImage::AsyncReadResult;
// Source: include/core/SkSurface.h:491-491
pub type ReadPixelsContext = *mut std::ffi::c_void;
// Source: include/core/SkSurface.h:496-496
pub type ReadPixelsCallback = super::SkImage::ReadPixelsCallback;
// Source: include/core/SkSurface.h:501-502
pub use super::SkImage::{RescaleGamma, RescaleMode};
// Source: include/core/SkSurface.h:634-634
impl SkSurfaceState {
    pub fn props(&self) -> &SkSurfaceProps {
        &self.fProps
    }
}
// Source: include/core/SkSurface.h:676-678
impl SkSurfaceState {
    pub(crate) fn dirtyGenerationID(&self) {
        self.fGenerationID.set(0);
    }
}
// Source: include/core/SkSurface.h:681-684
pub struct SkSurfaceState {
    fProps: SkSurfaceProps,
    fWidth: i32,
    fHeight: i32,
    fGenerationID: std::cell::Cell<u32>,
}
// Source: include/core/SkSurfaceProps.h:68-68
// Source: src/image/SkSurface.cpp:37-41
impl Default for SkSurfaceProps {
    fn default() -> Self {
        Self {
            fFlags: 0,
            fPixelGeometry: SkPixelGeometry::kUnknown_SkPixelGeometry,
            fTextContrast: SK_GAMMA_CONTRAST,
            fTextGamma: SK_GAMMA_EXPONENT,
        }
    }
}
// Source: include/core/SkSurfaceProps.h:70-70
// Source: src/image/SkSurface.cpp:43-47
impl SkSurfaceProps {
    pub fn with_flags(flags: u32, pg: SkPixelGeometry) -> Self {
        Self {
            fFlags: flags,
            fPixelGeometry: pg,
            fTextContrast: SK_GAMMA_CONTRAST,
            fTextGamma: SK_GAMMA_EXPONENT,
        }
    }
}
// Source: include/core/SkSurfaceProps.h:72-72
// Source: src/image/SkSurface.cpp:49-53
impl SkSurfaceProps {
    pub fn new(flags: u32, pg: SkPixelGeometry, textContrast: f32, textGamma: f32) -> Self {
        Self {
            fFlags: flags,
            fPixelGeometry: pg,
            fTextContrast: textContrast,
            fTextGamma: textGamma,
        }
    }
}

// Source: src/image/SkSurface.cpp:55-61
impl SkSurfaceState {
    pub(crate) fn new(width: i32, height: i32, props: Option<&SkSurfaceProps>) -> Self {
        let fProps = crate::src::core::SkSurfacePriv::SkSurfacePropsCopyOrDefault(props);
        let fWidth = width;
        let fHeight = height;
        crate::SkASSERT!(@source b"src/image/SkSurface.cpp\0".as_ptr(),58; @condition b"fWidth > 0\0".as_ptr(); fWidth>0);
        crate::SkASSERT!(@source b"src/image/SkSurface.cpp\0".as_ptr(),59; @condition b"fHeight > 0\0".as_ptr(); fHeight>0);
        Self {
            fProps,
            fWidth,
            fHeight,
            fGenerationID: std::cell::Cell::new(0),
        }
    }
}
// Source: src/image/SkSurface.cpp:63-69
// Source: src/image/SkSurface.cpp:71-71
impl SkSurfaceState {
    pub(crate) fn from_info(
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: Option<&SkSurfaceProps>,
    ) -> Self {
        let fProps = crate::src::core::SkSurfacePriv::SkSurfacePropsCopyOrDefault(props);
        let fWidth = info.width();
        let fHeight = info.height();
        crate::SkASSERT!(@source b"src/image/SkSurface.cpp\0".as_ptr(),66; @condition b"fWidth > 0\0".as_ptr(); fWidth>0);
        crate::SkASSERT!(@source b"src/image/SkSurface.cpp\0".as_ptr(),67; @condition b"fHeight > 0\0".as_ptr(); fHeight>0);
        Self {
            fProps,
            fWidth,
            fHeight,
            fGenerationID: std::cell::Cell::new(0),
        }
    }
}

// Native abstract public Surface is carried by a trait object whose dynamic
// metadata denotes the COMPLETE pinned concrete allocation. The sole counter is
// in its actual embedded Base; this is not a separately allocated surface view.
/// Safety: base_address returns the original whole-owner-derived raw base pointer
/// saved AFTER pinning, not a writable pointer derived from a shared reference.
/// The single allocation and counter obey SkRefCntObject's disposal contract.
pub unsafe trait SkSurface<'device>: crate::include::core::SkRefCnt::SkRefCntObject {
    fn state(&self) -> &SkSurfaceState;
    fn base_address(&self) -> core::ptr::NonNull<super::SkSurface_Base::SkSurface_Base<'device>>;
    // Source: include/core/SkSurface.h:189-189
    fn imageInfo(&self) -> crate::src::core::SkImageInfo::SkImageInfo {
        crate::src::core::SkImageInfo::SkImageInfo::MakeUnknown(
            self.state().width(),
            self.state().height(),
        )
    }
}
/// Safety: owner remains live/pinned and exclusively callable throughout the raw
/// operation; no whole Surface/Canvas/device/pixel borrow spans native COW.
pub(crate) unsafe fn base_address_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> core::ptr::NonNull<super::SkSurface_Base::SkSurface_Base<'device>> {
    // Read the stored original provenance; end the temporary shared owner lease
    // BEFORE any callback may mutate the embedded base or complete allocation.
    unsafe { owner.as_ref() }.base_address()
}
// Source: src/image/SkSurface.cpp:70-76
/// Safety: original complete pinned Surface is live and exclusively callable.
pub unsafe fn generationID_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> u32 {
    let base = unsafe { base_address_raw(owner) };
    let field = unsafe { core::ptr::addr_of!((*base.as_ptr()).surface.fGenerationID) };
    let mut generation = unsafe { (&*field).get() };
    if generation == 0 {
        generation = unsafe { super::SkSurface_Base::SkSurface_Base::newGenerationID_raw(base) };
        unsafe {
            (&*field).set(generation);
        }
    }
    generation
}

// Source: src/image/SkSurface.cpp:78-80
/// Safety: original complete Surface remains live/pinned with exclusive backing
/// access; no Canvas/Device/Surface reference survives the raw COW callback.
pub unsafe fn notifyContentWillChange_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    mode: ContentChangeMode,
) {
    let base = unsafe { base_address_raw(owner) };
    // Native explicitly ignores this boolean; no RasterError is discarded here.
    let _ = unsafe { super::SkSurface_Base::SkSurface_Base::aboutToDraw_raw(base, mode) };
}
// Source: src/image/SkSurface.cpp:82-84
/// Safety: returned Canvas is borrowed from this original pinned Surface. Caller
/// keeps the Surface alive and uses raw drawing entries without overlapping
/// Surface/Canvas/device/pixel references across a possible COW callback.
pub unsafe fn getCanvas_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> Option<core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::getCachedCanvas_raw(base) }
}

// Source: include/core/SkSurface.h:295-295
// Source: src/image/SkSurface.cpp:86-88
/// Safety: original complete Surface remains pinned/live through virtual dispatch.
pub unsafe fn capabilities_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> std::sync::Arc<crate::include::core::SkCapabilities::SkCapabilities> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::capabilities_raw(base) }
}

// Source: src/image/SkSurface.cpp:90-92
pub unsafe fn makeImageSnapshot_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> Result<Option<std::sync::Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::refCachedImage_raw(base) }
}
// Source: src/image/SkSurface.cpp:94-106
pub unsafe fn makeImageSnapshot_subset_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    subset: &crate::src::core::SkRect::SkIRect,
) -> Result<Option<std::sync::Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
    let bounds = {
        let surface = unsafe { owner.as_ref() };
        crate::src::core::SkRect::SkIRect::make_wh(
            surface.state().width(),
            surface.state().height(),
        )
    };
    let mut subset = *subset;
    if !subset.intersect(&bounds) {
        return Ok(None);
    }
    crate::SkASSERT!(@source b"src/image/SkSurface.cpp\0".as_ptr(),100; @condition b"!bounds.isEmpty()\0".as_ptr(); !subset.is_empty());
    if subset == bounds {
        unsafe { makeImageSnapshot_raw(owner) }
    } else {
        let base = unsafe { base_address_raw(owner) };
        unsafe {
            super::SkSurface_Base::SkSurface_Base::new_image_snapshot_raw(base, Some(&subset))
        }
    }
}

// Source: src/image/SkSurface.cpp:108-110
/// Safety: original pinned Surface remains live/exclusively callable through
/// snapshot creation and any recorder capture breakpoint. No whole-owner borrow
/// crosses that callback; returned image retains its original backing.
pub unsafe fn makeTemporaryImage_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> Result<Option<std::sync::Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::make_temporary_image_raw(base) }
}

// Source: src/image/SkSurface.cpp:112-114
/// Safety: the original pinned owner remains live and exclusively callable;
/// returned owner borrows the real recorder, never a temporary parent view.
pub unsafe fn makeSurface_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    info: &crate::src::core::SkImageInfo::SkImageInfo,
) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn SkSurface<'device> + 'device>> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::new_surface_raw(base, info) }
}
// Source: src/image/SkSurface.cpp:116-118
pub unsafe fn makeSurfaceWH_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    width: i32,
    height: i32,
) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn SkSurface<'device> + 'device>> {
    let info = unsafe { owner.as_ref() }.imageInfo().makeWH(width, height);
    unsafe { makeSurface_raw(owner, &info) }
}

// Source: include/core/SkSurface.h:370-372
/// Same pinned Surface/Canvas provenance and source/destination COW leases as
/// draw_raw; neither owner is projected to a complete borrow across dispatch.
pub unsafe fn draw_default_raw<'device, 'target>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'target>>,
    x: f32,
    y: f32,
    paint: Option<&crate::src::core::SkPaint::SkPaint>,
) -> Result<(), crate::RasterError> {
    unsafe {
        draw_raw(
            owner,
            canvas,
            x,
            y,
            &crate::include::core::SkSamplingOptions::SkSamplingOptions::default(),
            paint,
        )
    }
}

// Source: src/image/SkSurface.cpp:120-123
/// Safety: owner and destination are live pinned native objects; their source
/// read/destination write leases exclude conflicting access through every alias.
/// No whole Surface/Canvas reference survives into the concrete drawing callback.
pub unsafe fn draw_raw<'device, 'target>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'target>>,
    x: f32,
    y: f32,
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    paint: Option<&crate::src::core::SkPaint::SkPaint>,
) -> Result<(), crate::RasterError> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::draw_raw(base, canvas, x, y, sampling, paint) }
}

// Source: src/image/SkSurface.cpp:125-127
/// Safety: original Surface/Canvas remain live/exclusive during cache creation;
/// returned raw pixel view borrows its real backing, which caller must keep live.
pub unsafe fn peekPixels_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    pixmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
) -> bool {
    let canvas = unsafe { getCanvas_raw(owner) }.expect("native complete surface has canvas");
    unsafe { canvas.as_ref() }.peekPixels(pixmap)
}
// Source: src/image/SkSurface.cpp:129-131
/// Safety: source backing is initialized/live; destination pixmap covers its
/// declared writable layout exclusively for the actual complete pixel transfer.
pub unsafe fn readPixels_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    pixmap: &crate::src::core::SkPixmap::SkPixmap,
    x: i32,
    y: i32,
) -> Result<bool, crate::RasterError> {
    let mut canvas = unsafe { getCanvas_raw(owner) }.expect("native complete surface has canvas");
    // Read-only transfer performs no Surface COW callback; this short Canvas
    // projection ends before the Surface owner may be called again.
    unsafe { core::pin::Pin::new_unchecked(canvas.as_mut()).readPixels(pixmap, x, y) }
}
// Source: src/image/SkSurface.cpp:133-136
pub unsafe fn readPixels_info_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    info: &crate::src::core::SkImageInfo::SkImageInfo,
    pixels: *mut u8,
    row_bytes: usize,
    x: i32,
    y: i32,
) -> Result<bool, crate::RasterError> {
    let pixmap = crate::src::core::SkPixmap::SkPixmap::new(info, pixels, row_bytes);
    unsafe { readPixels_raw(owner, &pixmap, x, y) }
}
// Source: src/image/SkSurface.cpp:138-141
pub unsafe fn readPixels_bitmap_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    bitmap: &crate::src::core::SkBitmap::SkBitmap,
    x: i32,
    y: i32,
) -> Result<bool, crate::RasterError> {
    let mut pixmap = crate::src::core::SkPixmap::SkPixmap::default();
    if !bitmap.peekPixels(Some(&mut pixmap)) {
        return Ok(false);
    }
    unsafe { readPixels_raw(owner, &pixmap, x, y) }
}
// Source: include/core/SkSurface.h:537-542
// Source: src/image/SkSurface.cpp:143-156
/// Safety: original pinned Surface and exclusive native operation/backing lease
/// remain valid through synchronous dispatch and callback; errors propagate.
pub unsafe fn asyncRescaleAndReadPixels_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    info: &crate::src::core::SkImageInfo::SkImageInfo,
    srcRect: &crate::src::core::SkRect::SkIRect,
    rescaleGamma: RescaleGamma,
    rescaleMode: RescaleMode,
    callback: ReadPixelsCallback,
    context: ReadPixelsContext,
) -> Result<(), crate::RasterError> {
    let bounds = {
        let surface = owner.as_ref();
        crate::src::core::SkRect::SkIRect::make_wh(
            surface.state().width(),
            surface.state().height(),
        )
    };
    if !bounds.contains_rect(srcRect)
        || !crate::src::core::SkImageInfoPriv::SkImageInfoIsValid(info)
    {
        callback(context, None);
        return Ok(());
    }
    let base = base_address_raw(owner);
    super::SkSurface_Base::SkSurface_Base::async_rescale_pixels_raw(
        base,
        owner,
        info,
        *srcRect,
        rescaleGamma,
        rescaleMode,
        callback,
        context,
    )
}

// Source: include/core/SkSurface.h:573-580
// Source: src/image/SkSurface.cpp:158-180
/// Safety: the original pinned Surface remains live and exclusively callable.
/// Callback/context remain valid through dispatch, without overlapping a
/// whole-Surface or Canvas borrow with callback access to the native owner.
pub unsafe fn asyncRescaleAndReadPixelsYUV420_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
    dst_color_space: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    src_rect: &crate::src::core::SkRect::SkIRect,
    dst_size: &crate::include::core::SkSize::SkISize,
    rescale_gamma: RescaleGamma,
    rescale_mode: RescaleMode,
    callback: ReadPixelsCallback,
    context: ReadPixelsContext,
) {
    let bounds = {
        let surface = unsafe { owner.as_ref() };
        crate::src::core::SkRect::SkIRect::make_wh(
            surface.state().width(),
            surface.state().height(),
        )
    };
    if !bounds.contains_rect(src_rect)
        || dst_size.isZero()
        || (dst_size.width() & 1) != 0
        || (dst_size.height() & 1) != 0
    {
        unsafe {
            callback(context, None);
        }
        return;
    }
    let base = unsafe { base_address_raw(owner) };
    unsafe {
        super::SkSurface_Base::SkSurface_Base::async_rescale_yuv420_raw(
            base,
            yuv_color_space,
            false,
            dst_color_space,
            *src_rect,
            *dst_size,
            rescale_gamma,
            rescale_mode,
            callback,
            context,
        );
    }
}

// Source: include/core/SkSurface.h:587-594
// Source: src/image/SkSurface.cpp:182-204
/// Safety: same original-owner and callback leases as the YUV entry.
pub unsafe fn asyncRescaleAndReadPixelsYUVA420_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
    dst_color_space: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    src_rect: &crate::src::core::SkRect::SkIRect,
    dst_size: &crate::include::core::SkSize::SkISize,
    rescale_gamma: RescaleGamma,
    rescale_mode: RescaleMode,
    callback: ReadPixelsCallback,
    context: ReadPixelsContext,
) {
    let bounds = {
        let surface = unsafe { owner.as_ref() };
        crate::src::core::SkRect::SkIRect::make_wh(
            surface.state().width(),
            surface.state().height(),
        )
    };
    if !bounds.contains_rect(src_rect)
        || dst_size.isZero()
        || (dst_size.width() & 1) != 0
        || (dst_size.height() & 1) != 0
    {
        unsafe {
            callback(context, None);
        }
        return;
    }
    let base = unsafe { base_address_raw(owner) };
    unsafe {
        super::SkSurface_Base::SkSurface_Base::async_rescale_yuv420_raw(
            base,
            yuv_color_space,
            true,
            dst_color_space,
            *src_rect,
            *dst_size,
            rescale_gamma,
            rescale_mode,
            callback,
            context,
        );
    }
}

// Source: src/image/SkSurface.cpp:206-223
/// Safety: source pixels remain live/initialized and do not conflict with the
/// exclusively writable Surface backing. No Surface/Canvas/device/pixel reference
/// crosses native COW; the whole owner is live/pinned until transfer completes.
pub unsafe fn writePixels_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    source: &crate::src::core::SkPixmap::SkPixmap,
    x: i32,
    y: i32,
) -> Result<(), crate::RasterError> {
    use crate::src::core::SkRect::SkIRect;
    if source.addr().is_null() || source.width() <= 0 || source.height() <= 0 {
        return Ok(());
    }
    let src = SkIRect::make_xywh(x, y, source.width(), source.height());
    let dst = {
        let surface = unsafe { owner.as_ref() };
        SkIRect::make_wh(surface.state().width(), surface.state().height())
    };
    if SkIRect::intersects(&src, &dst) {
        let mode = if src.contains_rect(&dst) {
            ContentChangeMode::kDiscard_ContentChangeMode
        } else {
            ContentChangeMode::kRetain_ContentChangeMode
        };
        let base = unsafe { base_address_raw(owner) };
        if !unsafe { super::SkSurface_Base::SkSurface_Base::aboutToDraw_raw(base, mode) } {
            return Ok(());
        }
        unsafe { super::SkSurface_Base::SkSurface_Base::write_pixels_raw(base, source, x, y) }?;
    }
    Ok(())
}
// Source: src/image/SkSurface.cpp:225-230
pub unsafe fn writePixels_bitmap_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
    bitmap: &crate::src::core::SkBitmap::SkBitmap,
    x: i32,
    y: i32,
) -> Result<(), crate::RasterError> {
    let mut pixmap = crate::src::core::SkPixmap::SkPixmap::default();
    if bitmap.peekPixels(Some(&mut pixmap)) {
        unsafe { writePixels_raw(owner, &pixmap, x, y) }?;
    }
    Ok(())
}

// Source: src/image/SkSurface.cpp:238-238
/// Safety: owner is the original live pinned Surface allocation. Its actual
/// borrowed recorder outlives every returned reference; no owner is fabricated
/// or retained and no whole Surface borrow crosses the native virtual dispatch.
pub unsafe fn baseRecorder_raw<'device>(
    owner: core::ptr::NonNull<dyn SkSurface<'device> + 'device>,
) -> Option<&'device dyn crate::include::core::SkRecorder::SkRecorder> {
    let base = unsafe { base_address_raw(owner) };
    unsafe { super::SkSurface_Base::SkSurface_Base::baseRecorder_raw(base) }
}

// Source: include/core/SkSurface.h:236-244
// Duplicate native enum values are associated aliases of the same value.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendHandleAccess {
    kFlushRead,
    kFlushWrite,
    kDiscardWrite,
}
impl BackendHandleAccess {
    pub const kFlushRead_BackendHandleAccess: Self = Self::kFlushRead;
    pub const kFlushWrite_BackendHandleAccess: Self = Self::kFlushWrite;
    pub const kDiscardWrite_BackendHandleAccess: Self = Self::kDiscardWrite;
}
// Source: include/core/SkSurface.h:248-253
pub const kFlushRead_BackendHandleAccess: BackendHandleAccess = BackendHandleAccess::kFlushRead;
pub const kFlushWrite_BackendHandleAccess: BackendHandleAccess = BackendHandleAccess::kFlushWrite;
pub const kDiscardWrite_BackendHandleAccess: BackendHandleAccess =
    BackendHandleAccess::kDiscardWrite;
// Source: include/core/SkSurface.h:256-256
pub type ReleaseContext = *mut core::ffi::c_void;
// Source: include/core/SkSurface.h:258-258
// Native C++ procedure protocol; an optional callback argument holds nullability.
pub type TextureReleaseProc = unsafe fn(ReleaseContext);

// Source: src/image/SkSurface.cpp:232-234
pub unsafe fn recordingContext_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
) -> *mut GrRecordingContext {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::get_recording_context_raw(base_address_raw(owner))
    }
}
// Source: src/image/SkSurface.cpp:236-236
pub unsafe fn recorder_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
) -> *mut skgpu::graphite::Recorder {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::get_graphite_recorder_raw(base_address_raw(owner))
    }
}
// Source: src/image/SkSurface.cpp:240-243
/// Safety: original live owner and native semaphore array obey the backend lease.
pub unsafe fn wait_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
    n: i32,
    semaphores: *const GrBackendSemaphore,
    delete_after_wait: bool,
) -> bool {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::wait_raw(
            base_address_raw(owner),
            n,
            semaphores,
            delete_after_wait,
        )
    }
}
// Source: src/image/SkSurface.cpp:245-247
pub unsafe fn characterize_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
    characterization: *mut GrSurfaceCharacterization,
) -> bool {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::characterize_raw(
            base_address_raw(owner),
            characterization,
        )
    }
}
// Source: src/image/SkSurface.cpp:249-251
/// Safety: characterization denotes a genuine live native object, never a null reference.
pub unsafe fn isCompatible_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
    characterization: core::ptr::NonNull<GrSurfaceCharacterization>,
) -> bool {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::is_compatible_raw(
            base_address_raw(owner),
            characterization.as_ptr(),
        )
    }
}
// Source: include/core/SkSurface.h:275-279
/// Safety: original pinned owner, live native texture reference and callback lease.
pub unsafe fn replaceBackendTexture_raw<'d>(
    owner: core::ptr::NonNull<dyn SkSurface<'d> + 'd>,
    texture: core::ptr::NonNull<GrBackendTexture>,
    origin: GrSurfaceOrigin,
    mode: ContentChangeMode,
    release: Option<TextureReleaseProc>,
    context: ReleaseContext,
) -> bool {
    unsafe {
        super::SkSurface_Base::SkSurface_Base::replace_backend_texture_raw(
            base_address_raw(owner),
            texture.as_ptr(),
            origin,
            mode,
            release,
            context,
        )
    }
}
