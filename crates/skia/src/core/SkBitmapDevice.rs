use crate::include::core::SkAlphaType::SkAlphaType;
use crate::include::core::SkColorType::SkColorType;
use crate::src::core::SkBitmap::SkBitmap;
use crate::src::core::SkImageInfo::{SkColorTypeIsAlwaysOpaque, SkImageInfo};

use crate::include::core::SkClipOp::SkClipOp;
use crate::src::core::SkDevice::{SkDevice, SkDeviceVirtual};
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::SkRegion;

// Source: src/core/SkBitmapDevice.h:124-124
impl SkBitmapDevice<'_, '_> {
    pub fn setImmutable(&self) {
        self.fBitmap.setImmutable();
    }
}
// Source: src/core/SkBitmapDevice.h:126-126
impl SkBitmapDevice<'_, '_> {
    pub fn getRasterHandle(&self) -> *mut core::ffi::c_void {
        self.fRasterHandle
    }
}
// Source: src/core/SkBitmapDevice.h:128-128
// Borrowed native recorder identity; Rust exposes the concrete canonical recorder base.
impl<'rec, 'ctx> SkBitmapDevice<'rec, 'ctx> {
    pub fn baseRecorder(
        &self,
    ) -> Option<&'rec crate::src::core::SkCPURecorderImpl::RecorderImpl<'ctx>> {
        self.fRecorder
    }
}
// Source: src/core/SkBitmapDevice.h:156-160
// Native inherited base uses composition. Recorder and external handle are non-owning;
// bitmap pixel ownership and clip/painter state remain canonical original types.
// Painter Option is construction staging only; every completed device owns Some
// canonical painter. Stack initializes at final Pin address before painter construction.
// Rust drops fields in declaration order; native derived members destroy in reverse
// declaration order, then the base. Keep painter alive only until before bitmap/base drop.
pub struct SkBitmapDevice<'rec, 'ctx> {
    fGlyphPainter: Option<crate::src::core::SkGlyphRunPainter::GlyphRunListPainter>,
    fRCStack: crate::src::core::SkRasterClipStack::SkRasterClipStack,
    fBitmap: crate::src::core::SkBitmap::SkBitmap,
    fRecorder: Option<&'rec crate::src::core::SkCPURecorderImpl::RecorderImpl<'ctx>>,
    fRasterHandle: *mut core::ffi::c_void,
    base: SkDevice,
}

// Source: src/core/SkBitmapDevice.cpp:47-49
struct Bounder {
    // MaybeUninit preserves native uninitialized fBounds on the false branch.
    fBounds: core::mem::MaybeUninit<crate::src::core::SkRect::SkRect>,
    fHasBounds: bool,
}

// Source: src/core/SkBitmapDevice.cpp:51-55
// Adaptation: initialized temporary output storage avoids forming a Rust reference to
// uninitialized native fBounds; only the true branch commits the resulting rectangle.
impl Bounder {
    fn new(
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<Self, crate::RasterError> {
        let mut bounder = Self {
            fBounds: core::mem::MaybeUninit::uninit(),
            fHasBounds: paint.canComputeFastBounds()?,
        };
        if bounder.fHasBounds {
            let mut storage = crate::src::core::SkRect::SkRect::default();
            let bounds = *paint.computeFastBounds(rect, &mut storage)?;
            bounder.fBounds.write(bounds);
        }
        Ok(bounder)
    }
}

// Source: src/core/SkBitmapDevice.cpp:57-59
impl Bounder {
    fn hasBounds(&self) -> bool {
        self.fHasBounds
    }
}
// Source: src/core/SkBitmapDevice.cpp:58-58
impl Bounder {
    fn bounds(&self) -> Option<&crate::src::core::SkRect::SkRect> {
        if self.fHasBounds {
            // SAFETY: only new() constructs this private type; true implies write above.
            Some(unsafe { self.fBounds.assume_init_ref() })
        } else {
            None
        }
    }
}
// Source: src/core/SkBitmapDevice.cpp:59-59
impl Bounder {
    fn bounds_ptr(&self) -> *const crate::src::core::SkRect::SkRect {
        self.bounds().map_or(core::ptr::null(), core::ptr::from_ref)
    }
}

// Source: src/core/SkBitmapDevice.cpp:64-64
const kMaxDim: i32 = 8192 - 1;

// Source: src/core/SkBitmapDevice.cpp:66-79
// Adapted: keep the complete-device provenance token without a spanning owner
// reference. Each tile's matrix/clip/props are owned value projections. next()
// lends only this helper's state, ending before the next setup/reentrant draw;
// callbacks may mutate/restore the Device without aliasing embedded CTM/clip.
// The original pixel/context leases remain caller obligations.
// Source: src/core/SkBitmapDevice.cpp:62-62
struct SkDrawTiler<'operation, 'rec, 'ctx> {
    fDevice: core::ptr::NonNull<SkBitmapDevice<'rec, 'ctx>>,
    fDeviceLease: core::marker::PhantomData<&'operation mut SkBitmapDevice<'rec, 'ctx>>,
    fProps: crate::include::core::SkSurfaceProps::SkSurfaceProps,
    fRootPixmap: crate::src::core::SkPixmap::SkPixmap,
    fSrcBounds: Option<SkIRect>,
    fDraw: crate::src::core::SkDraw::Draw<'operation>,
    fTileMatrix: Option<crate::src::core::SkMatrix::SkMatrix>,
    fTileRC: crate::src::core::SkRasterClip::SkRasterClip,
    fOrigin: crate::src::core::SkPoint::SkIPoint,
    fDone: bool,
    fNeedsTiling: bool,
}

// Source: src/core/SkBitmapDevice.cpp:81-83
// Adaptation: the independent test borrows the actual SkDevice base metadata;
// concrete SkBitmapDevice/SkDrawTiler state is not replaced by a partial shell.
fn needs_tiling(device: &crate::src::core::SkDevice::SkDevice) -> bool {
    device.width() > kMaxDim || device.height() > kMaxDim
}

// Source: src/core/SkBitmapDevice.cpp:85-141
impl<'operation, 'rec, 'ctx> SkDrawTiler<'operation, 'rec, 'ctx> {
    fn new(
        dev: core::pin::Pin<&'operation mut SkBitmapDevice<'rec, 'ctx>>,
        bounds: Option<&SkRect>,
    ) -> Self {
        unsafe { Self::new_raw(core::ptr::NonNull::from(dev.get_unchecked_mut()), bounds) }
    }
    /// Safety: original pinned device and pixel/recorder leases survive this tiler;
    /// callbacks may reenter, so no complete-device projection may span a draw.
    unsafe fn new_raw(
        mut raw: core::ptr::NonNull<SkBitmapDevice<'rec, 'ctx>>,
        bounds: Option<&SkRect>,
    ) -> Self {
        let mut dev = unsafe { core::pin::Pin::new_unchecked(raw.as_mut()) };
        let mut root = crate::src::core::SkPixmap::SkPixmap::default();
        if !dev.as_mut().accessPixels(Some(&mut root)) {
            root.reset_with(dev.base.image_info(), core::ptr::null(), 0);
        }
        let dev = dev.into_ref().get_ref();
        let clip = *dev.fRCStack.rc().get_bounds();
        let mut needs = clip.fRight > kMaxDim || clip.fBottom > kMaxDim;
        let mut src = None;
        let mut done = false;
        if needs {
            if let Some(bounds) = bounds {
                let mut mapped = dev.base.local_to_device().mapRect_value(bounds).roundOut();
                if mapped.intersect(&clip) {
                    needs = mapped.fRight > kMaxDim || mapped.fBottom > kMaxDim;
                } else {
                    needs = false;
                    done = true;
                }
                src = Some(mapped);
            } else {
                src = Some(clip);
            }
        }
        let mut draw = crate::src::core::SkDraw::Draw::new();
        let origin = if needs {
            let src = src.as_ref().expect("tiling initializes source bounds");
            crate::src::core::SkPoint::SkIPoint::make(src.fLeft - kMaxDim, src.fTop)
        } else {
            draw.fDst.clone_from(&root);
            crate::src::core::SkPoint::SkIPoint::make(0, 0)
        };
        let props = *dev.base.surface_props();
        if let Some(recorder) = dev.fRecorder {
            draw.fCtx = unsafe { recorder.ctx().as_ref() };
        }
        let matrix = if needs {
            None
        } else {
            Some(dev.base.local_to_device().clone())
        };
        let rc = if needs {
            crate::src::core::SkRasterClip::SkRasterClip::new()
        } else {
            dev.fRCStack.rc().clone()
        };
        Self {
            fDevice: raw,
            fDeviceLease: core::marker::PhantomData,
            fProps: props,
            fRootPixmap: root,
            fSrcBounds: src,
            fDraw: draw,
            fTileMatrix: matrix,
            fTileRC: rc,
            fOrigin: origin,
            fDone: done,
            fNeedsTiling: needs,
        }
    }
}
// Source: src/core/SkBitmapDevice.cpp:143-143
impl SkDrawTiler<'_, '_, '_> {
    fn needsTiling(&self) -> bool {
        self.fNeedsTiling
    }
}
// Source: src/core/SkBitmapDevice.cpp:145-163
impl SkDrawTiler<'_, '_, '_> {
    fn next(&mut self) -> Option<crate::src::core::SkDraw::Draw<'_>> {
        if self.fDone {
            return None;
        }
        if self.fNeedsTiling {
            loop {
                self.stepAndSetupTileDraw();
                if self.fDone || !self.fTileRC.is_empty() {
                    break;
                }
            }
            if self.fTileRC.is_empty() {
                crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 155; @condition b"fDone\0".as_ptr(); self.fDone);
                return None;
            }
            debug_assert!(!self.fTileRC.is_empty());
        } else {
            self.fDone = true;
        }
        let mut draw = self.fDraw.clone();
        draw.fCTM = Some(
            self.fTileMatrix
                .as_ref()
                .expect("tile setup initializes matrix"),
        );
        draw.fRC = Some(&self.fTileRC);
        draw.fProps = Some(&self.fProps);
        Some(draw)
    }
}
// Source: src/core/SkBitmapDevice.cpp:166-192
impl SkDrawTiler<'_, '_, '_> {
    fn stepAndSetupTileDraw(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 167; @condition b"!fDone\0".as_ptr(); !self.fDone);
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 168; @condition b"fNeedsTiling\0".as_ptr(); self.fNeedsTiling);
        let src = self
            .fSrcBounds
            .as_ref()
            .expect("tiling source bounds initialized");
        if self.fOrigin.fX >= src.fRight - kMaxDim {
            self.fOrigin.fX = src.fLeft;
            self.fOrigin.fY += kMaxDim;
        } else {
            self.fOrigin.fX += kMaxDim;
        }
        self.fDone =
            self.fOrigin.fX >= src.fRight - kMaxDim && self.fOrigin.fY >= src.fBottom - kMaxDim;
        let bounds = SkIRect::make_xywh(self.fOrigin.fX, self.fOrigin.fY, kMaxDim, kMaxDim);
        debug_assert!(!bounds.is_empty());
        if !self
            .fRootPixmap
            .extractSubset(&mut self.fDraw.fDst, &bounds)
        {
            std::process::abort();
        }
        let mut matrix = unsafe { self.fDevice.as_ref() }
            .base
            .local_to_device()
            .clone();
        matrix.postTranslate((-self.fOrigin.fX) as f32, (-self.fOrigin.fY) as f32);
        self.fTileMatrix = Some(matrix);
        unsafe { self.fDevice.as_ref() }.fRCStack.rc().translate_to(
            -self.fOrigin.fX,
            -self.fOrigin.fY,
            Some(&mut self.fTileRC),
        );
        self.fTileRC.op_irect(
            SkIRect::make_size(self.fDraw.fDst.dimensions()),
            SkClipOp::kIntersect,
        );
    }
}

// Source: src/core/SkBitmapDevice.cpp:199-203
// Raw Draw results propagate immediately; the pinned original Device remains
// leased by the real tiler for every tile, including early-error teardown.
macro_rules! LOOP_TILER{
    ($device:expr,$bounds:expr,$method:ident($($arg:expr),* $(,)?))=>{{
        let mut tiler=SkDrawTiler::new($device,$bounds);
        while let Some(draw)=tiler.next(){unsafe{draw.$method($($arg),*)}?;}
        Ok::<(),crate::RasterError>(())
    }};
}

// Source: src/core/SkBitmapDevice.cpp:205-217
// Derived Draw uses composition; references borrow the concrete device for the
// operation; the canonical inherited accessPixels wrapper dispatches to this device.
struct BDDraw<'operation> {
    draw: crate::src::core::SkDraw::Draw<'operation>,
}
impl<'operation> BDDraw<'operation> {
    fn new(mut dev: core::pin::Pin<&'operation mut SkBitmapDevice<'_, '_>>) -> Self {
        let mut draw = crate::src::core::SkDraw::Draw::new();
        if !dev.as_mut().accessPixels(Some(&mut draw.fDst)) {
            draw.fDst
                .reset_with(dev.base.image_info(), core::ptr::null(), 0);
        }
        let dev = dev.into_ref().get_ref();
        draw.fCTM = Some(dev.base.local_to_device());
        draw.fRC = Some(dev.fRCStack.rc());
        Self { draw }
    }
}

// Source: src/core/SkBitmapDevice.cpp:219-231
pub(crate) fn valid_for_bitmap_device(
    info: &SkImageInfo,
    newAlphaType: Option<&mut SkAlphaType>,
) -> bool {
    if info.width() < 0
        || info.height() < 0
        || info.colorType() == SkColorType::kUnknown_SkColorType
    {
        return false;
    }
    if let Some(newAlphaType) = newAlphaType {
        *newAlphaType = if SkColorTypeIsAlwaysOpaque(info.colorType()) {
            SkAlphaType::kOpaque_SkAlphaType
        } else {
            info.alphaType()
        };
    }
    true
}

// Source: src/core/SkBitmapDevice.cpp:233-234
impl SkBitmapDevice<'static, 'static> {
    /// Safety: bitmap backing obeys the constructor's original whole-device
    /// pixel lease until the complete pinned device's final destruction.
    pub unsafe fn new(bitmap: &SkBitmap) -> core::pin::Pin<Box<Self>> {
        unsafe {
            Self::with_recorder(
                Some(crate::src::core::SkCPURecorder::Recorder::TODO()),
                bitmap,
            )
        }
    }
    // Source: src/core/SkBitmapDevice.cpp:236-239
    pub unsafe fn with_props(
        bitmap: &SkBitmap,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        handle: *mut core::ffi::c_void,
    ) -> core::pin::Pin<Box<Self>> {
        unsafe {
            Self::with_recorder_props(
                Some(crate::src::core::SkCPURecorder::Recorder::TODO()),
                bitmap,
                props,
                handle,
            )
        }
    }
}

// Source: src/core/SkBitmapDevice.cpp:241-248
impl<'rec, 'ctx> SkBitmapDevice<'rec, 'ctx> {
    /// # Safety
    /// Establish the device's whole-lifetime pixel lease: every non-null bitmap
    /// backing is live, writable/initialized as required by blending, and has no
    /// overlapping access through other bitmap/pixel owners while this device
    /// lives. Recorder remains borrowed for its declared lifetime. Null backing
    /// retains the native no-pixels device behavior; no pointer validity is inferred
    /// from metadata or Pin. All backend replacement preserves this same invariant.
    /// Safe virtual drawing accepts arbitrary input shapes: the actual RCStack
    /// intersects replacement clips with root bounds, and every drawing branch
    /// must retain that clipping before accessing pixels.
    pub unsafe fn with_recorder(
        recorder: Option<&'rec crate::src::core::SkCPURecorderImpl::RecorderImpl<'ctx>>,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
    ) -> core::pin::Pin<Box<Self>> {
        let props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        unsafe { Self::with_recorder_props(recorder, bitmap, &props, core::ptr::null_mut()) }
    }
}
// Source: src/core/SkBitmapDevice.cpp:250-261
impl<'rec, 'ctx> SkBitmapDevice<'rec, 'ctx> {
    /// # Safety
    /// Establish the same whole-device pixel/recorder lease as with_recorder().
    /// The external raster handle is borrowed and remains valid for its native use.
    pub unsafe fn with_recorder_props(
        recorder: Option<&'rec crate::src::core::SkCPURecorderImpl::RecorderImpl<'ctx>>,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        handle: *mut core::ffi::c_void,
    ) -> core::pin::Pin<Box<Self>> {
        let mut device = Box::pin(Self {
            base: SkDevice::new(bitmap.info(), props),
            fRasterHandle: handle,
            fRecorder: recorder,
            fBitmap: bitmap.clone(),
            fRCStack: unsafe {
                crate::src::core::SkRasterClipStack::SkRasterClipStack::unbound(
                    bitmap.width(),
                    bitmap.height(),
                )
            },
            fGlyphPainter: None,
        });
        // Native clip stack constructs before glyph painter. Initialize only at
        // the complete owner's final pinned address, then construct the painter.
        unsafe { device.as_mut().map_unchecked_mut(|d| &mut d.fRCStack) }.initialize();
        let painter = crate::src::core::SkGlyphRunPainter::GlyphRunListPainter::new(
            device.base.surface_props(),
            device.fBitmap.colorType(),
            device.fBitmap.colorSpace(),
        );
        unsafe { device.as_mut().map_unchecked_mut(|d| &mut d.fGlyphPainter) }.set(Some(painter));
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 260; @condition b"valid_for_bitmap_device(bitmap.info(), nullptr)\0".as_ptr(); valid_for_bitmap_device(bitmap.info(),None));
        device
    }
}
// Source: src/core/SkBitmapDevice.cpp:263-298
impl<'rec, 'ctx> SkBitmapDevice<'rec, 'ctx> {
    /// # Safety
    /// Custom allocator allocations and returned handles must preserve the native
    /// whole-device backing/release lease. The device's private bitmap retains
    /// the resulting allocation until the last intrusive device reference drops.
    pub unsafe fn Create<'allocator>(
        orig_info: &SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
        allocator: Option<
            &mut (dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator
                      + 'allocator),
        >,
    ) -> crate::include::core::SkRefCnt::sk_sp<Self> {
        let mut alpha_type = orig_info.alphaType();
        if !valid_for_bitmap_device(orig_info, Some(&mut alpha_type)) {
            return crate::include::core::SkRefCnt::sk_sp::default();
        }
        let mut handle = core::ptr::null_mut();
        let info = orig_info.makeAlphaType(alpha_type);
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        if info.colorType() == SkColorType::kUnknown_SkColorType {
            if !bitmap.setInfo(&info, 0) {
                return crate::include::core::SkRefCnt::sk_sp::default();
            }
        } else if let Some(allocator) = allocator {
            handle = unsafe { allocator.allocBitmap(&info, &mut bitmap) };
            if handle.is_null() {
                return crate::include::core::SkRefCnt::sk_sp::default();
            }
        } else if info.isOpaque() {
            if !bitmap.tryAllocPixels(&info, 0) {
                return crate::include::core::SkRefCnt::sk_sp::default();
            }
        } else {
            // Native Bitmap.h kZeroPixels_AllocFlag=1<<0 is an obsolete no-op
            // flag (native allocations always zero); keep its exact call value.
            if !bitmap.tryAllocPixelsFlags(&info, 1 << 0) {
                return crate::include::core::SkRefCnt::sk_sp::default();
            }
        }
        let device = unsafe {
            Self::with_recorder_props(
                Some(crate::src::core::SkCPURecorder::Recorder::TODO()),
                &bitmap,
                props,
                handle,
            )
        };
        // Adopt this already-pinned complete concrete allocation's initial sole
        // embedded base counter. No second allocation, retain, or counter exists.
        let object = Box::into_raw(unsafe { core::pin::Pin::into_inner_unchecked(device) });
        unsafe { crate::include::core::SkRefCnt::sk_sp::from_raw(object) }
    }
}

// Source: src/core/SkBitmapDevice.cpp:301-305
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// New backing preserves the constructor's whole-device lifetime/exclusive pixel lease.
    pub unsafe fn replaceBitmapBackendForRasterSurface(
        self: core::pin::Pin<&mut Self>,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
    ) {
        debug_assert_eq!(bitmap.width(), self.fBitmap.width());
        debug_assert_eq!(bitmap.height(), self.fBitmap.height());
        // Native bitmap assignment publishes/relinquishes PixelRef before
        // assigning the descriptor; dropping/replacing a whole Bitmap reverses that.
        unsafe { self.get_unchecked_mut() }
            .fBitmap
            .clone_from(bitmap);
    }
}

// Source: src/core/SkBitmapDevice.cpp:307-321
impl SkBitmapDevice<'_, '_> {
    /// Safety: allocator backing/release contexts survive independently until
    /// the new child's final pixel release. The child borrows no parent owner.
    pub unsafe fn createDevice(
        &self,
        cinfo: &mut crate::src::core::SkDevice::CreateInfo<'_>,
        layer_paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn SkDeviceVirtual + 'static> {
        let props = self
            .base
            .surface_props()
            .cloneWithPixelGeometry(cinfo.pixel_geometry());
        let mut info = cinfo.info().clone();
        if layer_paint.is_some_and(|paint| paint.getImageFilter().is_some()) {
            info = info.makeColorType(crate::include::core::SkColorType::kN32_SkColorType);
        }
        let mut child =
            unsafe { SkBitmapDevice::<'static, 'static>::Create(&info, &props, cinfo.allocator()) };
        match child.release() {
            Some(child) => {
                let complete: *mut (dyn SkDeviceVirtual + 'static) = child.as_ptr();
                unsafe { crate::include::core::SkRefCnt::sk_sp::from_raw(complete) }
            }
            None => crate::include::core::SkRefCnt::sk_sp::default(),
        }
    }
}

// Source: src/core/SkBitmapDevice.cpp:323-329
impl SkBitmapDevice<'_, '_> {
    pub fn onAccessPixels(
        self: core::pin::Pin<&mut Self>,
        pixmap: &mut crate::src::core::SkPixmap::SkPixmap,
    ) -> bool {
        if self.onPeekPixels(pixmap) {
            self.fBitmap.notifyPixelsChanged();
            return true;
        }
        false
    }
}
// Source: src/core/SkBitmapDevice.cpp:331-338
impl SkBitmapDevice<'_, '_> {
    pub fn onPeekPixels(&self, pixmap: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        let info = self.fBitmap.info();
        if !self.fBitmap.getPixels().is_null()
            && info.colorType() != SkColorType::kUnknown_SkColorType
        {
            pixmap.reset_with(info, self.fBitmap.getPixels(), self.fBitmap.rowBytes());
            return true;
        }
        false
    }
}
// Source: src/core/SkBitmapDevice.cpp:340-351
// Uncompiled canonical Bitmap writePixels overload; generation notification remains native.
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Source and destination pixmap storage must be valid for their declared
    /// bounds/strides; caller excludes overlapping or concurrent pixel access.
    pub unsafe fn onWritePixels(
        self: core::pin::Pin<&mut Self>,
        pixmap: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        if self.fBitmap.getPixels().is_null() {
            return Ok(false);
        }
        let bitmap = unsafe { self.map_unchecked_mut(|d| &mut d.fBitmap) }.get_mut();
        if unsafe { bitmap.writePixels(pixmap, x, y) }? {
            bitmap.notifyPixelsChanged();
            return Ok(true);
        }
        Ok(false)
    }
}
// Source: src/core/SkBitmapDevice.cpp:353-355
// Uncompiled canonical Bitmap readPixels overload.
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Bitmap source and writable pixmap destination storage must remain live,
    /// appropriately initialized, and exclusively accessed for this operation.
    pub unsafe fn onReadPixels(
        &self,
        pixmap: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.fBitmap.readPixels(pixmap, x, y) }
    }
}

// Source: src/core/SkBitmapDevice.cpp:359-361
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Bitmap may wrap safe-installed raw pixels or share its PixelRef with clones.
    /// Caller guarantees valid writable initialized destination storage and exclusive
    /// pixel access for the whole draw, plus Draw's chooser/filter arena contracts.
    pub unsafe fn drawPaint(
        self: core::pin::Pin<&mut Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        BDDraw::new(self).draw.drawPaint(paint)?;
        Ok(())
    }
}

// Source: src/core/SkBitmapDevice.cpp:363-366
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the whole-device pixel/clip/context and native chooser lease.
    pub unsafe fn drawPoints(
        self: core::pin::Pin<&mut Self>,
        mode: crate::src::core::SkCanvas::PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        LOOP_TILER!(self, None, drawPoints(mode, points, paint, None))
    }
}
// Source: src/core/SkBitmapDevice.cpp:368-370
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the whole-device pixel lease and native Draw factory contracts.
    pub unsafe fn drawRect(
        self: core::pin::Pin<&mut Self>,
        r: &SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounder = Bounder::new(r, paint)?;
        LOOP_TILER!(self, bounder.bounds(), drawRectSimple(r, paint))
    }
}
// Source: src/core/SkBitmapDevice.cpp:372-374
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the whole-device pixel lease and native Draw factory contracts.
    pub unsafe fn drawOval(
        self: core::pin::Pin<&mut Self>,
        oval: &SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounder = Bounder::new(oval, paint)?;
        LOOP_TILER!(self, bounder.bounds(), drawOval(oval, paint))
    }
}
// Source: src/core/SkBitmapDevice.cpp:376-378
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the whole-device pixel lease and native Draw factory contracts.
    pub unsafe fn drawRRect(
        self: core::pin::Pin<&mut Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounder = Bounder::new(rrect.getBounds(), paint)?;
        LOOP_TILER!(self, bounder.bounds(), drawRRect(rrect, paint))
    }
}
// Source: src/core/SkBitmapDevice.cpp:380-389
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the whole-device pixel lease and native Draw factory contracts.
    pub unsafe fn drawPath(
        self: core::pin::Pin<&mut Self>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounder = if needs_tiling(&self.base) && !path.isInverseFillType() {
            Some(Bounder::new(path.getBounds(), paint)?)
        } else {
            None
        };
        let mut tiler = SkDrawTiler::new(self, bounder.as_ref().and_then(Bounder::bounds));
        while let Some(draw) = tiler.next() {
            unsafe {
                draw.drawPath(path, paint, None)?;
            }
        }
        Ok(())
    }
}

// Source: src/core/SkBitmapDevice.cpp:391-407
impl SkBitmapDevice<'_, '_> {
    /// Safety: native device/source pixel and shader leases hold for all tiles.
    pub unsafe fn drawBitmap(
        self: core::pin::Pin<&mut Self>,
        bitmap: &SkBitmap,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        dst: Option<&SkRect>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<(), crate::RasterError> {
        let mut storage = SkRect::default();
        let mut bounds = dst;
        if bounds.is_none() && needs_tiling(&self.as_ref().get_ref().base) {
            matrix.mapRect(
                &mut storage,
                &SkRect::MakeIWH(bitmap.width(), bitmap.height()),
            );
            let bounder = Bounder::new(&storage, paint)?;
            if let Some(b) = bounder.bounds() {
                storage = *b;
                bounds = Some(&storage);
            }
        }
        let mut tiler = SkDrawTiler::new(self, bounds);
        while let Some(draw) = tiler.next() {
            unsafe { draw.drawBitmap(bitmap, matrix, dst, sampling, paint, mips.clone()) }?;
        }
        Ok(())
    }
}
// Source: src/core/SkBitmapDevice.cpp:410-418
// Canonical native Paint mask-filter accessor preserves the CTM restriction.
fn CanApplyDstMatrixAsCTM(
    matrix: &crate::src::core::SkMatrix::SkMatrix,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> bool {
    if paint.getMaskFilter().is_none() {
        return true;
    }
    matrix.getType() <= crate::src::core::SkMatrix::SkMatrix::kTranslate_Mask
}

// Source: src/core/SkBitmapDevice.cpp:419-540
impl SkBitmapDevice<'_, '_> {
    /// Safety: the real source image retains live immutable backing; destination
    /// Canvas/Device and retained shaders uphold the native exclusive draw lease.
    pub unsafe fn drawImageRect(
        mut self: core::pin::Pin<&mut Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: Option<&SkRect>,
        dst: &SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkMatrix::{ScaleToFit, SkMatrix};
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 423; @condition b"dst.isFinite()\0".as_ptr(); dst.isFinite());
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 424; @condition b"dst.isSorted()\0".as_ptr(); dst.isSorted());
        let mut bitmap = SkBitmap::default();
        let base = image.asImageBase();
        if !base.getROPixelsCPU(
            &mut bitmap,
            crate::src::image::SkImage::CachingHint::kAllow_CachingHint,
        ) {
            return Ok(());
        }
        let mut mips = base.refMips();
        let bounds = SkRect::MakeIWH(bitmap.width(), bitmap.height());
        let mut tmp_src = src.copied().unwrap_or(bounds);
        let mut tmp_dst = *dst;
        let mut matrix =
            SkMatrix::RectToRectOrIdentity(&tmp_src, dst, ScaleToFit::kFill_ScaleToFit);
        let mut subset = false;
        if let Some(src) = src {
            if !bounds.contains(src) {
                if !tmp_src.intersect(&bounds) {
                    return Ok(());
                }
                matrix.mapRect(&mut tmp_dst, &tmp_src);
                if !tmp_dst.isFinite() {
                    return Ok(());
                }
            }
            subset = !tmp_src.contains(&bounds);
        }
        let force_shader = subset
            && constraint == crate::src::core::SkCanvas::SrcRectConstraint::kFast_SrcRectConstraint
            && *sampling != crate::include::core::SkSamplingOptions::SkSamplingOptions::new();
        let mut tmp_bitmap = SkBitmap::default();
        let mut bitmap_ptr = &bitmap;
        let mut use_draw_bitmap = !subset;
        if !force_shader && subset {
            let src_ir = tmp_src.roundOut();
            if !bitmap.extractSubset(Some(&mut tmp_bitmap), &src_ir) {
                return Ok(());
            }
            bitmap_ptr = &tmp_bitmap;
            mips = None;
            let dx = if src_ir.fLeft > 0 {
                src_ir.fLeft as f32
            } else {
                0.0
            };
            let dy = if src_ir.fTop > 0 {
                src_ir.fTop as f32
            } else {
                0.0
            };
            if dx != 0.0 || dy != 0.0 {
                matrix.preTranslate(dx, dy);
            }
            // Selected native configuration leaves SK_DRAWBITMAPRECT_FAST_OFFSET undefined.
            use_draw_bitmap = SkRect::MakeIWH(bitmap_ptr.width(), bitmap_ptr.height()) == tmp_src;
        }
        if !force_shader && use_draw_bitmap && CanApplyDstMatrixAsCTM(&matrix, paint) {
            return unsafe {
                self.as_mut()
                    .drawBitmap(bitmap_ptr, &matrix, Some(&tmp_dst), sampling, paint, mips)
            };
        }
        let raster = unsafe {
            crate::src::image::SkImage_Raster::SkImage_Raster::MakeFromBitmap(
                bitmap_ptr,
                crate::src::image::SkImage_Raster::SkCopyPixelsMode::kNever,
                mips,
            )
        }
        .expect("native valid raster bitmap");
        let Some(shader) = raster.makeShaderForPaint(
            paint,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            Some(&matrix),
        ) else {
            return Ok(());
        };
        let mut shader_paint = paint.clone();
        shader_paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
        shader_paint.setShader(Some(shader));
        unsafe { self.drawRect(&tmp_dst, &shader_paint) }
    }
}

// Source: src/core/SkBitmapDevice.cpp:542-547
unsafe fn draw_glyph_run_list_bitmap<'device, 'canvas>(
    device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    list: &crate::src::text::GlyphRun::GlyphRunList,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    debug_assert!(!unsafe { list.hasRSXForm() });
    // This virtual function is installed only by the complete BitmapDevice.
    // Strip its trait metadata, retaining the original writable allocation token.
    let concrete = device.cast::<SkBitmapDevice<'_, '_>>();
    let painter = unsafe { (*core::ptr::addr_of!((*concrete.as_ptr()).fGlyphPainter)) }
        .expect("completed BitmapDevice owns its painter");
    let mut tiler = unsafe { SkDrawTiler::new_raw(concrete, None) };
    while let Some(draw) = tiler.next() {
        unsafe { draw.drawGlyphRunList(canvas, &painter, list, paint) }?;
    }
    Ok(())
}

// Source: src/core/SkBitmapDevice.cpp:549-559
impl SkBitmapDevice<'_, '_> {
    /// Safety: retain the complete Device/Draw destination and shader leases.
    pub unsafe fn drawVertices(
        self: core::pin::Pin<&mut Self>,
        vertices: &crate::src::core::SkVertices::SkVertices,
        mut blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
        skip_color_xform: bool,
    ) -> Result<(), crate::RasterError> {
        #[cfg(sk_legacy_ignore_draw_vertices_blend_with_no_shader)]
        if paint.getShader().is_none() {
            blender = <dyn crate::include::core::SkBlender::SkBlender>::Mode(
                crate::src::core::SkBlendMode::SkBlendMode::kDst,
            );
        }
        unsafe {
            BDDraw::new(self)
                .draw
                .drawVertices(vertices, blender, paint, skip_color_xform)
        }
    }
}

// Source: src/core/SkBitmapDevice.cpp:575-595
impl SkBitmapDevice<'_, '_> {
    /// Safety: complete native destination lease excludes all conflicting pixel
    /// access; source SpecialImage retains live immutable storage for the draw.
    pub unsafe fn drawSpecial(
        self: core::pin::Pin<&mut Self>,
        src: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        local_to_device: &crate::src::core::SkMatrix::SkMatrix,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        _constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 580; @condition b"!paint.getImageFilter()\0".as_ptr(); paint.getImageFilter().is_none());
        crate::SkASSERT!(@source b"src/core/SkBitmapDevice.cpp\0".as_ptr(), 581; @condition b"!paint.getMaskFilter()\0".as_ptr(); paint.getMaskFilter().is_none());
        // GPU backings are outside the selected CPU trait's sealed backing set.
        let mut bitmap = SkBitmap::default();
        if crate::src::core::SkSpecialImage::AsBitmap(Some(src), Some(&mut bitmap)) {
            let dev = unsafe { self.get_unchecked_mut() };
            let mut draw = crate::src::core::SkDraw::Draw::new();
            if !SkDeviceVirtual::accessPixels(dev, Some(&mut draw.fDst)) {
                return Ok(());
            }
            draw.fCTM = Some(local_to_device);
            draw.fRC = Some(dev.fRCStack.rc());
            unsafe {
                draw.drawBitmap(
                    &bitmap,
                    crate::src::core::SkMatrix::SkMatrix::I(),
                    None,
                    sampling,
                    paint,
                    None,
                )
            }?;
        }
        Ok(())
    }
}

// Source: src/core/SkBitmapDevice.cpp:597-616
impl SkBitmapDevice<'_, '_> {
    /// Safety: source and destination allocations uphold the complete native
    /// coverage draw lease, including shader evaluation and temporary mask use.
    pub unsafe fn drawCoverageMask(
        self: core::pin::Pin<&mut Self>,
        mask: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        mask_to_device: &crate::src::core::SkMatrix::SkMatrix,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let mut bitmap = SkBitmap::default();
        if !crate::src::core::SkSpecialImage::AsBitmap(Some(mask), Some(&mut bitmap)) {
            return Ok(());
        }
        let dev = unsafe { self.get_unchecked_mut() };
        let mut draw = crate::src::core::SkDraw::Draw::new();
        if !SkDeviceVirtual::accessPixels(dev, Some(&mut draw.fDst)) {
            return Ok(());
        }
        draw.fRC = Some(dev.fRCStack.rc());
        draw.fCTM = Some(mask_to_device);
        unsafe { draw.drawBitmapAsMask(&bitmap, sampling, paint, Some(dev.base.local_to_device())) }
    }
}

// Source: src/core/SkBitmapDevice.cpp:619-634
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Preserve the constructor's whole-device raster lease and native filter/
    /// arena contracts; all tiles retain their actual bounded raster clip.
    pub unsafe fn drawBlurredRRect(
        self: core::pin::Pin<&mut Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
        _device_sigma: f32,
    ) -> Result<bool, crate::RasterError> {
        debug_assert!(
            paint.getMaskFilter().is_some()
                && paint.getMaskFilter().unwrap().r#type()
                    == crate::src::core::SkMaskFilterBase::Type::kBlur
        );
        let bounds = Bounder::new(rrect.getBounds(), paint)?;
        let mut tiler = SkDrawTiler::new(self, bounds.bounds());
        if !tiler.needsTiling() {
            if let Some(draw) = tiler.next() {
                return unsafe { draw.drawRRectNinePatch(rrect, paint) };
            }
        }
        Ok(false)
    }
}
// Source: src/core/SkBitmapDevice.cpp:636-642
impl SkBitmapDevice<'_, '_> {
    /// # Safety
    /// Source backing must retain its device lease through any copy. A non-copy
    /// image keeps the actual bitmap allocation alive; subsequent pixel access
    /// must respect sharing and any concurrent device mutation.
    pub unsafe fn snapSpecial(
        &self,
        bounds: &SkIRect,
        force_copy: bool,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        if force_copy {
            unsafe {
                crate::src::core::SkSpecialImage::CopyFromRaster(
                    *bounds,
                    &self.fBitmap,
                    *self.base.surface_props(),
                )
            }
        } else {
            Ok(crate::src::core::SkSpecialImage::MakeFromRasterBitmap(
                *bounds,
                &self.fBitmap,
                *self.base.surface_props(),
            ))
        }
    }
}

// Source: src/core/SkBitmapDevice.cpp:646-648
impl SkBitmapDevice<'_, '_> {
    pub fn makeSurface(
        &self,
        info: &SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::image::SkSurface::SkSurface<'static> + 'static,
    > {
        crate::src::image::SkSurface_Raster::Raster(info, 0, Some(props)).unwrap_or_default()
    }
}

// Source: src/core/SkBitmapDevice.cpp:652-654
impl SkBitmapDevice<'_, '_> {
    pub fn pushClipStack(self: core::pin::Pin<&mut Self>) {
        unsafe { self.map_unchecked_mut(|d| &mut d.fRCStack) }.save();
    }
}
// Source: src/core/SkBitmapDevice.cpp:656-658
impl SkBitmapDevice<'_, '_> {
    pub fn popClipStack(self: core::pin::Pin<&mut Self>) {
        unsafe { self.map_unchecked_mut(|d| &mut d.fRCStack) }.restore();
    }
}
// Source: src/core/SkBitmapDevice.cpp:660-662
impl SkBitmapDevice<'_, '_> {
    pub fn clipRect(self: core::pin::Pin<&mut Self>, rect: &SkRect, op: SkClipOp, aa: bool) {
        let d = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut d.fRCStack) }.clip_rect(
            d.base.local_to_device(),
            rect,
            op,
            aa,
        );
    }
}
// Source: src/core/SkBitmapDevice.cpp:664-666
impl SkBitmapDevice<'_, '_> {
    pub fn clipRRect(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRRect::SkRRect,
        op: SkClipOp,
        aa: bool,
    ) {
        let d = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut d.fRCStack) }.clip_rrect(
            d.base.local_to_device(),
            rect,
            op,
            aa,
        );
    }
}
// Source: src/core/SkBitmapDevice.cpp:668-670
impl SkBitmapDevice<'_, '_> {
    pub fn clipPath(
        self: core::pin::Pin<&mut Self>,
        path: &crate::src::core::SkPath::SkPath,
        op: SkClipOp,
        aa: bool,
    ) {
        let d = unsafe { self.get_unchecked_mut() };
        unsafe { core::pin::Pin::new_unchecked(&mut d.fRCStack) }.clip_path(
            d.base.local_to_device(),
            path,
            op,
            aa,
        );
    }
}
// Source: src/core/SkBitmapDevice.cpp:672-674
// Canonical stack consumes this Arc owner without cloning it at this boundary.
impl SkBitmapDevice<'_, '_> {
    pub fn onClipShader(
        self: core::pin::Pin<&mut Self>,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) {
        unsafe { self.map_unchecked_mut(|d| &mut d.fRCStack) }.clip_shader(shader);
    }
}
// Source: src/core/SkBitmapDevice.cpp:676-686
impl SkBitmapDevice<'_, '_> {
    pub fn clipRegion(self: core::pin::Pin<&mut Self>, region: &SkRegion, op: SkClipOp) {
        let d = unsafe { self.get_unchecked_mut() };
        let origin = d.base.get_origin();
        let mut tmp = SkRegion::new();
        let mut r = region;
        if origin.fX | origin.fY != 0 {
            region.translate_to(-origin.fX, -origin.fY, Some(&mut tmp));
            r = &tmp;
        }
        unsafe { core::pin::Pin::new_unchecked(&mut d.fRCStack) }.clip_region(r, op);
    }
}
// Source: src/core/SkBitmapDevice.cpp:688-692
impl SkBitmapDevice<'_, '_> {
    pub fn replaceClip(self: core::pin::Pin<&mut Self>, rect: &SkIRect) {
        let device_rect = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
            self.base.global_to_device(),
            &SkRect::Make(rect),
        );
        unsafe { self.map_unchecked_mut(|d| &mut d.fRCStack) }.replace_clip(device_rect.round());
    }
}
// Source: src/core/SkBitmapDevice.cpp:694-699
impl SkBitmapDevice<'_, '_> {
    pub fn isClipWideOpen(&self) -> bool {
        let rc = self.fRCStack.rc();
        rc.is_bw()
            && rc.bw_rgn().is_rect()
            && *rc.bw_rgn().get_bounds()
                == SkIRect::make_ltrb(0, 0, self.base.width(), self.base.height())
    }
}
// Source: src/core/SkBitmapDevice.cpp:701-703
impl SkBitmapDevice<'_, '_> {
    pub fn isClipEmpty(&self) -> bool {
        self.fRCStack.rc().is_empty()
    }
}
// Source: src/core/SkBitmapDevice.cpp:705-708
// Adaptation: canonical clip_shader() returns a temporary Arc clone; it is dropped
// after the emptiness query. No second persistent owner is retained.
impl SkBitmapDevice<'_, '_> {
    pub fn isClipRect(&self) -> bool {
        let rc = self.fRCStack.rc();
        !rc.is_empty() && rc.is_rect() && rc.clip_shader().is_none()
    }
}
// Source: src/core/SkBitmapDevice.cpp:710-713
impl SkBitmapDevice<'_, '_> {
    pub fn isClipAntiAliased(&self) -> bool {
        let rc = self.fRCStack.rc();
        !rc.is_empty() && rc.is_aa()
    }
}
// Source: src/core/SkBitmapDevice.cpp:715-722
impl SkBitmapDevice<'_, '_> {
    pub fn android_utils_clipAsRgn(&self, region: &mut SkRegion) {
        let rc = self.fRCStack.rc();
        if rc.is_aa() {
            region.set_rect(*rc.get_bounds());
        } else {
            *region = rc.bw_rgn().clone();
        }
    }
}
// Source: src/core/SkBitmapDevice.cpp:724-726
impl SkBitmapDevice<'_, '_> {
    pub fn devClipBounds(&self) -> SkIRect {
        *self.fRCStack.rc().get_bounds()
    }
}

// Rust inheritance/ownership adaptation: the complete pinned allocation owns
// exactly the counter embedded in SkDevice. Native virtual dispatch preserves
// structurally pinned fields; safe drawing relies on constructor backing lease
// and the actual root-bounded clip. No Box/Pin handle is a second native object.
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for SkBitmapDevice<'_, '_> {
    // Source: src/core/SkBitmapDevice.h:47-47
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
}
// Canonical native virtual dispatch is attached to real pinned owner/borrow handles.
// Pixel draws rely on the invariant established by unsafe device constructors;
// no safe raw Bitmap can be installed without the same lifetime lease contract.
unsafe impl SkDeviceVirtual for SkBitmapDevice<'_, '_> {
    // Source: src/core/SkBitmapDevice.h:85-85
    unsafe fn drawMesh(
        &mut self,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawMesh(core::pin::Pin::new_unchecked(self), mesh, blender, paint)
        }
    }
    unsafe fn drawAtlas(
        &mut self,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawAtlas(
                core::pin::Pin::new_unchecked(self),
                xform,
                tex,
                colors,
                blender,
                paint,
            )
        }
    }
    unsafe fn drawVertices(
        &mut self,
        vertices: &crate::src::core::SkVertices::SkVertices,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
        skip_color_xform: bool,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawVertices(
                core::pin::Pin::new_unchecked(self),
                vertices,
                blender,
                paint,
                skip_color_xform,
            )
        }
    }
    // Source: src/core/SkBitmapDevice.h:142-142
    fn drawGlyphRunListProc(&self) -> crate::src::core::SkDevice::DrawGlyphRunListProc {
        draw_glyph_run_list_bitmap
    }

    fn makeSurface(
        &self,
        info: &SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::image::SkSurface::SkSurface<'static> + 'static,
    > {
        SkBitmapDevice::makeSurface(self, info, props)
    }

    fn createDevice(
        &mut self,
        cinfo: &mut crate::src::core::SkDevice::CreateInfo<'_>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn SkDeviceVirtual + 'static> {
        unsafe { SkBitmapDevice::createDevice(self, cinfo, paint) }
    }
    fn baseRecorder(&self) -> Option<&dyn crate::include::core::SkRecorder::SkRecorder> {
        self.fRecorder
            .map(|recorder| recorder as &dyn crate::include::core::SkRecorder::SkRecorder)
    }

    unsafe fn drawImageRect(
        &mut self,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: Option<&SkRect>,
        dst: &SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawImageRect(
                core::pin::Pin::new_unchecked(self),
                image,
                src,
                dst,
                sampling,
                paint,
                constraint,
            )
        }
    }

    unsafe fn drawCoverageMask(
        &mut self,
        mask: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawCoverageMask(
                core::pin::Pin::new_unchecked(self),
                mask,
                matrix,
                sampling,
                paint,
            )
        }
    }

    unsafe fn drawSpecialWithConstraint(
        &mut self,
        src: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawSpecial(
                core::pin::Pin::new_unchecked(self),
                src,
                matrix,
                sampling,
                paint,
                constraint,
            )
        }
    }

    fn onClipShader(
        &mut self,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) {
        SkBitmapDevice::onClipShader(unsafe { core::pin::Pin::new_unchecked(self) }, shader);
    }
    fn getRasterHandle(&self) -> *mut core::ffi::c_void {
        SkBitmapDevice::getRasterHandle(self)
    }
    fn setImmutable(&mut self) {
        SkBitmapDevice::setImmutable(self)
    }
    fn drawBlurredRRect(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
        sigma: f32,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            SkBitmapDevice::drawBlurredRRect(
                core::pin::Pin::new_unchecked(&mut *self),
                rrect,
                paint,
                sigma,
            )
        }
    }
    fn snapSpecial(
        &mut self,
        bounds: &SkIRect,
        force_copy: bool,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        unsafe { SkBitmapDevice::snapSpecial(self, bounds, force_copy) }
    }
    fn base(&self) -> &SkDevice {
        &self.base
    }
    unsafe fn base_mut(&mut self) -> &mut SkDevice {
        &mut self.base
    }
    unsafe fn onReadPixels(
        &mut self,
        p: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { SkBitmapDevice::onReadPixels(self, p, x, y) }
    }
    unsafe fn onWritePixels(
        &mut self,
        p: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { SkBitmapDevice::onWritePixels(core::pin::Pin::new_unchecked(&mut *self), p, x, y) }
    }
    fn onAccessPixels(&mut self, p: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        SkBitmapDevice::onAccessPixels(unsafe { core::pin::Pin::new_unchecked(&mut *self) }, p)
    }
    fn onPeekPixels(&self, p: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        SkBitmapDevice::onPeekPixels(self, p)
    }
    fn drawPaint(
        &mut self,
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawPaint(core::pin::Pin::new_unchecked(&mut *self), p) }
    }
    fn drawRect(
        &mut self,
        r: &SkRect,
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawRect(core::pin::Pin::new_unchecked(&mut *self), r, p) }
    }
    fn drawOval(
        &mut self,
        r: &SkRect,
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawOval(core::pin::Pin::new_unchecked(&mut *self), r, p) }
    }
    fn drawRRect(
        &mut self,
        r: &crate::src::core::SkRRect::SkRRect,
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawRRect(core::pin::Pin::new_unchecked(&mut *self), r, p) }
    }
    fn drawPoints(
        &mut self,
        m: crate::src::core::SkCanvas::PointMode,
        pts: &[crate::src::core::SkPoint::SkPoint],
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawPoints(core::pin::Pin::new_unchecked(&mut *self), m, pts, p) }
    }
    fn drawPath(
        &mut self,
        r: &crate::src::core::SkPath::SkPath,
        p: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkBitmapDevice::drawPath(core::pin::Pin::new_unchecked(&mut *self), r, p) }
    }
    fn devClipBounds(&self) -> SkIRect {
        SkBitmapDevice::devClipBounds(self)
    }
    fn pushClipStack(&mut self) {
        SkBitmapDevice::pushClipStack(unsafe { core::pin::Pin::new_unchecked(&mut *self) })
    }
    fn popClipStack(&mut self) {
        SkBitmapDevice::popClipStack(unsafe { core::pin::Pin::new_unchecked(&mut *self) })
    }
    fn clipRect(&mut self, r: &SkRect, op: SkClipOp, aa: bool) {
        SkBitmapDevice::clipRect(
            unsafe { core::pin::Pin::new_unchecked(&mut *self) },
            r,
            op,
            aa,
        )
    }
    fn clipRRect(&mut self, r: &crate::src::core::SkRRect::SkRRect, op: SkClipOp, aa: bool) {
        SkBitmapDevice::clipRRect(
            unsafe { core::pin::Pin::new_unchecked(&mut *self) },
            r,
            op,
            aa,
        )
    }
    fn clipPath(&mut self, p: &crate::src::core::SkPath::SkPath, op: SkClipOp, aa: bool) {
        SkBitmapDevice::clipPath(
            unsafe { core::pin::Pin::new_unchecked(&mut *self) },
            p,
            op,
            aa,
        )
    }
    fn clipRegion(&mut self, r: &SkRegion, op: SkClipOp) {
        SkBitmapDevice::clipRegion(unsafe { core::pin::Pin::new_unchecked(&mut *self) }, r, op)
    }
    fn replaceClip(&mut self, r: &SkIRect) {
        SkBitmapDevice::replaceClip(unsafe { core::pin::Pin::new_unchecked(&mut *self) }, r)
    }
    fn isClipAntiAliased(&self) -> bool {
        SkBitmapDevice::isClipAntiAliased(self)
    }
    fn isClipEmpty(&self) -> bool {
        SkBitmapDevice::isClipEmpty(self)
    }
    fn isClipRect(&self) -> bool {
        SkBitmapDevice::isClipRect(self)
    }
    fn isClipWideOpen(&self) -> bool {
        SkBitmapDevice::isClipWideOpen(self)
    }
    fn android_utils_clipAsRgn(&self, r: &mut SkRegion) {
        SkBitmapDevice::android_utils_clipAsRgn(self, r)
    }
}
impl SkBitmapDevice<'_, '_> {
    // Inherited native wrapper on the pinned complete object, not a pinned handle.
    fn accessPixels(
        self: core::pin::Pin<&mut Self>,
        pixels: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        SkDeviceVirtual::accessPixels(unsafe { self.get_unchecked_mut() }, pixels)
    }
}

// Source: src/core/SkBitmapDevice.cpp:561-563
impl SkBitmapDevice<'_, '_> {
    /// Native CPU BitmapDevice's exact empty Mesh override.
    pub unsafe fn drawMesh(
        self: core::pin::Pin<&mut Self>,
        _mesh: &crate::src::core::SkMesh::SkMesh,
        _blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkBitmapDevice.cpp:565-571
    pub unsafe fn drawAtlas(
        self: core::pin::Pin<&mut Self>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            BDDraw::new(self)
                .draw
                .drawAtlas(xform, tex, colors, blender, paint)
        }
    }
}
