use super::SkImageInfo::{SkColorTypeValidateAlphaType, SkImageInfo};
use super::SkPixelRef::{SkMakePixelRefWithProc, SkPixelRef};
use super::SkPixmap::SkPixmap;
use crate::include::core::{
    SkAlphaType::SkAlphaType,
    SkColorType::{SkColorType, SkColorType::*},
};
use crate::src::core::SkPoint::SkIPoint;
use std::ffi::c_void;
use std::sync::Arc;
impl SkBitmap {
    // Source: include/core/SkBitmap.h:131-131
    /// Borrows this bitmap's actual descriptor; pixel memory is retained only by
    /// the bitmap's original PixelRef, and COW/backing changes invalidate views.
    pub fn pixmap(&self) -> &SkPixmap {
        &self.fPixmap
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:137-137
    pub fn info(&self) -> &SkImageInfo {
        self.fPixmap.info()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:147-147
    pub fn width(&self) -> i32 {
        self.fPixmap.width()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:156-156
    // Source: include/core/SkBitmap.h:158-158
    // Source: include/core/SkBitmap.h:160-160
    pub fn height(&self) -> i32 {
        self.fPixmap.height()
    }
}
impl SkBitmap {
    pub fn colorType(&self) -> SkColorType {
        self.fPixmap.colorType()
    }
}
impl SkBitmap {
    pub fn alphaType(&self) -> SkAlphaType {
        self.fPixmap.alphaType()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:185-185
    pub fn bytesPerPixel(&self) -> i32 {
        self.info().bytesPerPixel()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:199-199
    pub fn shiftPerPixel(&self) -> i32 {
        self.fPixmap.shiftPerPixel()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:208-208
    pub fn empty(&self) -> bool {
        self.fPixmap.isEmpty()
    }
}
// Source: include/core/SkBitmap.h:217-217
impl SkBitmap {
    pub fn isNull(&self) -> bool {
        self.fPixelRef.is_none()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:224-226
    pub fn drawsNothing(&self) -> bool {
        self.empty() || self.isNull()
    }
}
// Source: include/core/SkBitmap.h:236-236
impl SkBitmap {
    pub fn rowBytes(&self) -> usize {
        self.fPixmap.rowBytes()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:281-281
    pub fn getPixels(&self) -> *mut u8 {
        self.fPixmap.writable_addr()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:291-291
    pub fn computeByteSize(&self) -> usize {
        self.fPixmap.computeByteSize()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:322-324
    pub fn isOpaque(&self) -> bool {
        crate::include::core::SkAlphaType::SkAlphaTypeIsOpaque(self.alphaType())
    }
}
// Source: include/core/SkBitmap.h:356-358
impl SkBitmap {
    /// The bitmap retains readable initialized pixels for the complete scan.
    pub unsafe fn ComputeIsOpaque(bm: &Self) -> bool {
        unsafe { bm.pixmap().computeIsOpaque() }
    }
}
// Source: include/core/SkBitmap.h:380-380
impl SkBitmap {
    pub fn bounds(&self) -> crate::src::core::SkRect::SkIRect {
        self.fPixmap.info().bounds()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:386-386
    pub fn dimensions(&self) -> crate::include::core::SkSize::SkISize {
        self.fPixmap.info().dimensions()
    }
}
impl SkBitmap {
    // Source: include/core/SkBitmap.h:392-395
    pub fn getSubset(&self) -> super::SkRect::SkIRect {
        let origin = self.pixelRefOrigin();
        super::SkRect::SkIRect::make_xywh(origin.fX, origin.fY, self.width(), self.height())
    }
}
// Source: include/core/SkBitmap.h:427-427
impl SkBitmap {
    pub fn setInfo_default(&mut self, info: &SkImageInfo) -> bool {
        self.setInfo(info, 0)
    }
}
// Source: include/core/SkBitmap.h:432-434
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocFlags {
    kZeroPixels_AllocFlag = 1 << 0,
}
impl SkBitmap {
    pub const kZeroPixels_AllocFlag: u32 = AllocFlags::kZeroPixels_AllocFlag as u32;
}

// Source: include/core/SkBitmap.h:525-527
impl SkBitmap {
    pub fn tryAllocPixels_info(&mut self, info: &SkImageInfo) -> bool {
        self.tryAllocPixels(info, info.minRowBytes())
    }
}
// Source: include/core/SkBitmap.h:564-564
impl SkBitmap {
    pub fn tryAllocN32Pixels_default(&mut self, width: i32, height: i32) -> bool {
        self.tryAllocN32Pixels(width, height, false)
    }
}
// Source: include/core/SkBitmap.h:583-583
impl SkBitmap {
    pub fn allocN32Pixels_default(&mut self, width: i32, height: i32) {
        self.allocN32Pixels(width, height, false);
    }
}
// Source: include/core/SkBitmap.h:625-627
impl SkBitmap {
    /// Pixel storage is live and obeys the original exclusive writable lease;
    /// the bitmap does not acquire ownership of this external allocation.
    pub unsafe fn installPixels_no_release(
        &mut self,
        info: &SkImageInfo,
        pixels: *mut c_void,
        rowBytes: usize,
    ) -> bool {
        unsafe { self.installPixels(info, pixels, rowBytes, None, core::ptr::null_mut()) }
    }
}
// Source: include/core/SkBitmap.h:714-714
impl SkBitmap {
    pub fn pixelRef(&self) -> Option<&Arc<SkPixelRef>> {
        self.fPixelRef.as_ref()
    }
}
// Source: include/core/SkBitmap.h:752-754
impl SkBitmap {
    pub fn readyToDraw(&self) -> bool {
        !self.getPixels().is_null()
    }
}
// Source: include/core/SkBitmap.h:811-813
impl SkBitmap {
    /// Same exclusive destination-pixel lease as eraseColor.
    pub unsafe fn eraseARGB(
        &self,
        a: u32,
        r: u32,
        g: u32,
        b: u32,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.eraseColor(super::SkColor::SkColorSetARGB(a, r, g, b)) }
    }
}
// Source: include/core/SkBitmap.h:848-850
impl SkBitmap {
    /// Same exclusive destination-pixel lease as erase_area.
    pub unsafe fn eraseArea(
        &self,
        area: &super::SkRect::SkIRect,
        color: super::SkColor::SkColor,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.erase_area(color, area) }
    }
}
// Source: include/core/SkBitmap.h:868-870
impl SkBitmap {
    /// The backing pixels must be readable at this coordinate.
    pub unsafe fn getColor(&self, x: i32, y: i32) -> super::SkColor::SkColor {
        self.pixmap().getColor(x, y)
    }
}
// Source: include/core/SkBitmap.h:887-887
impl SkBitmap {
    /// The backing pixels must be readable at this coordinate.
    pub unsafe fn getColor4f(&self, x: i32, y: i32) -> super::SkColor::SkColor4f {
        self.pixmap().getColor4f(x, y)
    }
}
// Source: include/core/SkBitmap.h:897-899
impl SkBitmap {
    /// The backing pixels must be readable at this coordinate.
    pub unsafe fn getAlphaf(&self, x: i32, y: i32) -> f32 {
        self.pixmap().getAlphaf(x, y)
    }
}
// Source: include/core/SkBitmap.h:1062-1064
impl SkBitmap {
    /// Source pixels are readable and destination pixels exclusively writable
    /// through the real conversion path; backend errors remain Result errors.
    pub unsafe fn readPixels_default(&self, dst: &SkPixmap) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels(dst, 0, 0) }
    }
}
// Source: include/core/SkBitmap.h:1116-1118
impl SkBitmap {
    /// Same source/destination backing and alias obligations as writePixels.
    pub unsafe fn writePixels_default(
        &mut self,
        src: &SkPixmap,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.writePixels(src, 0, 0) }
    }
}
// Source: include/core/SkBitmap.h:1128-1130
impl SkBitmap {
    /// Source pixels remain readable until the real alpha extraction completes.
    pub unsafe fn extractAlpha_default(&self, dst: &mut Self) -> Result<bool, crate::RasterError> {
        unsafe { self.extractAlpha(dst, None, None, None) }
    }
}
// Source: include/core/SkBitmap.h:1145-1148
impl SkBitmap {
    /// Source and filter follow extractAlpha's bounds/allocation lease contract.
    pub unsafe fn extractAlpha_with_paint(
        &self,
        dst: &mut Self,
        paint: Option<&super::SkPaint::SkPaint>,
        offset: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.extractAlpha(dst, paint, None, offset) }
    }
}
// Source: include/core/SkBitmap.h:1184-1185
impl SkBitmap {
    /// Source pixels retain the same immutable backing lease for the returned shader.
    pub unsafe fn makeShader_default(
        &self,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        unsafe { self.makeShader(tmx, tmy, sampling, None) }
    }
}
// Source: include/core/SkBitmap.h:1190-1191
impl SkBitmap {
    /// Source pixels retain the same immutable backing lease for the returned shader.
    pub unsafe fn makeShaderClamp_default(
        &self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        unsafe { self.makeShaderClamp(sampling, None) }
    }
}
// Source: include/core/SkBitmap.h:1243-1244
// Source: include/core/SkBitmap.h:57-57
pub struct SkBitmap {
    // Rust declaration order deliberately reverses native members for C++ destruction order.
    // Source: include/core/SkBitmap.h:1244-1244
    fPixmap: SkPixmap,
    // Source: include/core/SkBitmap.h:1243-1243
    fPixelRef: Option<Arc<SkPixelRef>>,
}
// Source: include/core/SkBitmap.h:1249-1252
impl SkBitmap {
    pub fn getAddr32(&self, x: i32, y: i32) -> *mut u32 {
        debug_assert!(!self.fPixmap.addr().is_null());
        self.fPixmap.writable_addr32(x, y)
    }
}
// Source: include/core/SkBitmap.h:1254-1257
impl SkBitmap {
    pub fn getAddr16(&self, x: i32, y: i32) -> *mut u16 {
        debug_assert!(!self.fPixmap.addr().is_null());
        self.fPixmap.writable_addr16(x, y)
    }
}
// Source: include/core/SkBitmap.h:1259-1262
impl SkBitmap {
    pub fn getAddr8(&self, x: i32, y: i32) -> *mut u8 {
        debug_assert!(!self.fPixmap.addr().is_null());
        self.fPixmap.writable_addr8(x, y)
    }
}
// Source: src/core/SkBitmap.cpp:38-41
fn reset_return_false(bm: &mut SkBitmap) -> bool {
    bm.reset();
    false
}
// Source: src/core/SkBitmap.cpp:43-43
impl Default for SkBitmap {
    fn default() -> Self {
        Self {
            fPixelRef: None,
            fPixmap: SkPixmap::default(),
        }
    }
}
// Source: src/core/SkBitmap.cpp:45-48
impl SkBitmap {
    fn clone_native(&self) -> Self {
        // Initialize in native declaration order, before either constructor-body validation.
        let pixel_ref = self.fPixelRef.clone();
        let pixmap = self.fPixmap.clone();
        let result = Self {
            fPixmap: pixmap,
            fPixelRef: pixel_ref,
        };
        #[cfg(sk_debug)]
        self.validate();
        #[cfg(sk_debug)]
        result.validate();
        result
    }
}

// Source: src/core/SkBitmap.cpp:50-54
impl SkBitmap {
    /// Explicit native move-constructor operation keeps the source observable in
    /// its original moved-from empty state, without incrementing PixelRef refs.
    pub fn take_from(other: &mut Self) -> Self {
        let pixel_ref = other.fPixelRef.take();
        let pixmap = SkPixmap::move_from(&mut other.fPixmap);
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),52; @condition b"!other.fPixelRef\0".as_ptr(); other.fPixelRef.is_none());
        other.fPixmap.reset();
        Self {
            fPixmap: pixmap,
            fPixelRef: pixel_ref,
        }
    }
}

// Source: src/core/SkBitmap.cpp:56-56
impl Drop for SkBitmap {
    fn drop(&mut self) { /* Native empty destructor; fields then drop pixmap before pixelref. */
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:58-65
    fn assign_native(&mut self, src: &Self) {
        if !std::ptr::eq(self, src) {
            let retained = src.fPixelRef.clone();
            let old = core::mem::replace(&mut self.fPixelRef, retained);
            drop(old);
            self.fPixmap.clone_from(&src.fPixmap);
        }
        #[cfg(sk_debug)]
        self.validate();
    }
}
// Source: src/core/SkBitmap.cpp:67-75
impl SkBitmap {
    pub fn assign_move(&mut self, other: &mut Self) {
        if !core::ptr::eq(self, other) {
            let old = core::mem::replace(&mut self.fPixelRef, other.fPixelRef.take());
            drop(old);
            self.fPixmap.assign_move(&mut other.fPixmap);
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),71; @condition b"!other.fPixelRef\0".as_ptr(); other.fPixelRef.is_none());
            other.fPixmap.reset();
        }
    }
}

impl Clone for SkBitmap {
    fn clone(&self) -> Self {
        self.clone_native()
    }
    fn clone_from(&mut self, src: &Self) {
        self.assign_native(src);
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:77-81
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
        #[cfg(sk_debug)]
        self.validate();
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:83-86
    pub fn reset(&mut self) {
        drop(self.fPixelRef.take());
        self.fPixmap.reset();
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:88-91
    pub fn getBounds(&self, bounds: &mut super::SkRect::SkRect) {
        *bounds = super::SkRect::SkRect::MakeIWH(self.width(), self.height());
    }
    // Source: src/core/SkBitmap.cpp:93-96
    pub fn getBoundsIRect(&self, bounds: &mut super::SkRect::SkIRect) {
        *bounds = self.fPixmap.bounds();
    }
    // Source: src/core/SkBitmap.cpp:98-98
    pub fn colorSpace(&self) -> Option<&super::SkColorSpace::SkColorSpace> {
        self.fPixmap.colorSpace()
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:100-100
    pub fn refColorSpace(&self) -> Option<Arc<super::SkColorSpace::SkColorSpace>> {
        self.fPixmap.info().refColorSpace()
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:104-108
    // Source: src/core/SkBitmap.cpp:112-118
    // Source: src/core/SkBitmap.cpp:120-122
    // Source: src/core/SkBitmap.cpp:124-130
    // Source: src/core/SkBitmap.cpp:132-136
    pub fn setInfo(&mut self, info: &SkImageInfo, mut rowBytes: usize) -> bool {
        let mut newAT = info.alphaType();
        if !SkColorTypeValidateAlphaType(info.colorType(), info.alphaType(), Some(&mut newAT)) {
            return reset_return_false(self);
        }
        let mrb = info.minRowBytes64() as i64;
        if mrb < i32::MIN as i64 || mrb > i32::MAX as i64 {
            return reset_return_false(self);
        }
        if rowBytes > i32::MAX as usize {
            return reset_return_false(self);
        }
        if info.width() < 0 || info.height() < 0 {
            return reset_return_false(self);
        }
        if info.colorType() == kUnknown_SkColorType {
            rowBytes = 0;
        } else if rowBytes == 0 {
            rowBytes = mrb as usize;
        } else if !info.validRowBytes(rowBytes) {
            return reset_return_false(self);
        }
        drop(self.fPixelRef.take());
        self.fPixmap
            .reset_with(&info.makeAlphaType(newAT), std::ptr::null(), rowBytes);
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:138-148
    pub fn setAlphaType(&mut self, mut newAlphaType: SkAlphaType) -> bool {
        if !SkColorTypeValidateAlphaType(self.colorType(), newAlphaType, Some(&mut newAlphaType)) {
            return false;
        }
        if self.alphaType() != newAlphaType {
            let info = self.fPixmap.info().makeAlphaType(newAlphaType);
            self.fPixmap
                .reset_with(&info, self.fPixmap.addr(), self.fPixmap.rowBytes());
        }
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:150-156
    pub fn setColorSpace(&mut self, newColorSpace: Option<Arc<super::SkColorSpace::SkColorSpace>>) {
        let same = match (self.colorSpace(), newColorSpace.as_deref()) {
            (None, None) => true,
            (Some(a), Some(b)) => std::ptr::eq(a, b),
            _ => false,
        };
        if !same {
            let info = self.fPixmap.info().makeColorSpace(newColorSpace);
            self.fPixmap
                .reset_with(&info, self.fPixmap.addr(), self.fPixmap.rowBytes());
        }
        #[cfg(sk_debug)]
        self.validate();
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:158-170
    pub fn pixelRefOrigin(&self) -> SkIPoint {
        let addr = self.fPixmap.addr() as usize;
        let pix = self.fPixelRef.as_ref().map_or(0, |p| p.pixels() as usize);
        let rb = self.rowBytes();
        if pix == 0 || rb == 0 {
            return SkIPoint { fX: 0, fY: 0 };
        }
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),165; @condition b"this->bytesPerPixel() > 0\0".as_ptr(); self.bytesPerPixel()>0);
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),166; @condition b"this->bytesPerPixel() == (1 << this->shiftPerPixel())\0".as_ptr(); self.bytesPerPixel()==(1<<self.shiftPerPixel()));
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),167; @condition b"addr >= pix\0".as_ptr(); addr>=pix);
        let off = addr - pix;
        SkIPoint {
            fX: ((off % rb) >> self.shiftPerPixel()) as i32,
            fY: (off / rb) as i32,
        }
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:172-172
    // Source: src/core/SkBitmap.cpp:174-179
    // Source: src/core/SkBitmap.cpp:181-183
    // Source: src/core/SkBitmap.cpp:185-186
    // Source: src/core/SkBitmap.cpp:188-195
    pub fn setPixelRef(&mut self, pr: Option<Arc<SkPixelRef>>, dx: i32, dy: i32) {
        #[cfg(sk_debug)]
        if let Some(pr) = &pr {
            if self.colorType() != kUnknown_SkColorType {
                crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),176; @condition b"dx >= 0 && this->width() + dx <= pr->width()\0".as_ptr(); dx>=0&&self.width()+dx<=pr.width());
                crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),177; @condition b"dy >= 0 && this->height() + dy <= pr->height()\0".as_ptr(); dy>=0&&self.height()+dy<=pr.height());
            }
        }
        let mut pr = pr;
        let replacement = if self.colorType() != kUnknown_SkColorType {
            pr.take()
        } else {
            None
        };
        let old = core::mem::replace(&mut self.fPixelRef, replacement);
        drop(old);
        let mut p = std::ptr::null_mut();
        let mut rb = self.rowBytes();
        if let Some(pr) = &self.fPixelRef {
            rb = pr.rowBytes();
            p = pr.pixels() as *mut u8;
            if !p.is_null() {
                p = p.wrapping_add(
                    (dy as usize)
                        .wrapping_mul(rb)
                        .wrapping_add((dx as usize).wrapping_mul(self.bytesPerPixel() as usize)),
                );
            }
        }
        self.fPixmap.reset_address(p, rb);
        #[cfg(sk_debug)]
        self.validate();
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:197-205
    pub fn setPixels(&mut self, mut p: *mut u8) {
        if self.colorType() == kUnknown_SkColorType {
            p = std::ptr::null_mut();
        }
        let rb = self.rowBytes();
        self.fPixmap.reset_address(p, rb);
        let replacement = if !p.is_null() {
            Some(Arc::new(SkPixelRef::new(
                self.width(),
                self.height(),
                p.cast(),
                rb,
            )))
        } else {
            None
        };
        let old = core::mem::replace(&mut self.fPixelRef, replacement);
        drop(old);
        #[cfg(sk_debug)]
        self.validate();
    }
}
// Source: include/core/SkBitmap.h:1207-1220
/// Safety: success installs actual pixel backing valid for the bitmap's declared
/// format/stride/size and its retained PixelRef lifetime, with the native release
/// ownership. Allocation itself does not borrow the allocator into the bitmap.
// Source: include/core/SkBitmap.h:59-59
// Source: include/core/SkBitmap.h:1207-1207
pub unsafe trait Allocator: crate::include::core::SkRefCnt::SkRefCntObject {
    fn allocPixelRef(&mut self, bitmap: &mut SkBitmap) -> bool;
}
// The actual native HeapAllocator owns its inherited sole reference counter.
// Source: include/core/SkBitmap.h:1227-1227
pub struct HeapAllocator {
    counter: crate::include::core::SkRefCnt::SkRefCnt,
}
impl HeapAllocator {
    pub fn new() -> Self {
        Self {
            counter: crate::include::core::SkRefCnt::SkRefCnt::new(),
        }
    }
}
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for HeapAllocator {
    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        &self.counter
    }
    fn ref_(&self) {
        self.counter.ref_();
    }
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
}
// Source: src/core/SkBitmap.cpp:207-214
impl SkBitmap {
    pub fn tryAllocPixelsWithAllocator(&mut self, allocator: Option<&mut dyn Allocator>) -> bool {
        let mut standard = HeapAllocator::new();
        match allocator {
            Some(allocator) => allocator.allocPixelRef(self),
            None => standard.allocPixelRef(self),
        }
    }
    // Source: include/core/SkBitmap.h:668-670
    // Source: include/core/SkBitmap.h:668-670
    pub fn tryAllocPixels_current(&mut self) -> bool {
        self.tryAllocPixelsWithAllocator(None)
    }
}
// Source: src/core/SkBitmap.cpp:216-220
impl SkBitmap {
    pub fn tryAllocN32Pixels(&mut self, width: i32, height: i32, opaque: bool) -> bool {
        let info = SkImageInfo::MakeN32(
            width,
            height,
            if opaque {
                SkAlphaType::kOpaque_SkAlphaType
            } else {
                SkAlphaType::kPremul_SkAlphaType
            },
        );
        self.tryAllocPixels(&info, 0)
    }
    // Source: src/core/SkBitmap.cpp:222-227
    pub fn allocN32Pixels(&mut self, width: i32, height: i32, opaque: bool) {
        let info = SkImageInfo::MakeN32(
            width,
            height,
            if opaque {
                SkAlphaType::kOpaque_SkAlphaType
            } else {
                SkAlphaType::kPremul_SkAlphaType
            },
        );
        self.allocPixels(&info);
    }
    // Source: src/core/SkBitmap.cpp:228-230
    pub fn allocPixels_current(&mut self) {
        self.allocPixelsWithAllocator(None);
    }
    // Source: src/core/SkBitmap.cpp:232-239
    pub fn allocPixelsWithAllocator(&mut self, allocator: Option<&mut dyn Allocator>) {
        if !self.tryAllocPixelsWithAllocator(allocator) {
            let info = self.info();
            crate::SK_ABORT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),235;
                "SkBitmap::tryAllocPixels failed ColorType:%d AlphaType:%d [w:%d h:%d] rb:%zu",
                info.colorType() as core::ffi::c_int,info.alphaType() as core::ffi::c_int,
                info.width() as core::ffi::c_int,info.height() as core::ffi::c_int,self.rowBytes());
        }
    }
    // Source: src/core/SkBitmap.cpp:241-246
    pub fn allocPixelsFlags(&mut self, info: &SkImageInfo, flags: u32) {
        crate::SkASSERTF_RELEASE!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),242;
            @condition b"this->tryAllocPixelsFlags(info, flags)\0".as_ptr();
            self.tryAllocPixelsFlags(info,flags),"ColorType:%d AlphaType:%d [w:%d h:%d] rb:%zu flags: 0x%x",
            info.colorType() as core::ffi::c_int,info.alphaType() as core::ffi::c_int,
            info.width() as core::ffi::c_int,info.height() as core::ffi::c_int,self.rowBytes(),flags);
    }
}

impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:248-253
    // Native release assertion retains its actual source, expression and printf diagnostics.
    pub fn allocPixels_row_bytes(&mut self, info: &SkImageInfo, rowBytes: usize) {
        crate::SkASSERTF_RELEASE!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),249;
            @condition b"this->tryAllocPixels(info, rowBytes)\0".as_ptr();
            self.tryAllocPixels(info,rowBytes),"ColorType:%d AlphaType:%d [w:%d h:%d] rb:%zu",
            info.colorType() as core::ffi::c_int,info.alphaType() as core::ffi::c_int,
            info.width() as core::ffi::c_int,info.height() as core::ffi::c_int,self.rowBytes());
    }
    // Source: src/core/SkBitmap.cpp:255-257
    pub fn allocPixels(&mut self, info: &SkImageInfo) {
        self.allocPixels_row_bytes(info, info.minRowBytes());
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:261-264
    // Source: src/core/SkBitmap.cpp:267-270
    // Source: src/core/SkBitmap.cpp:272-272
    // Source: src/core/SkBitmap.cpp:274-284
    pub fn tryAllocPixels(&mut self, info: &SkImageInfo, mut rowBytes: usize) -> bool {
        if !self.setInfo(info, rowBytes) {
            return reset_return_false(self);
        }
        let corrected = self.info();
        if corrected.colorType() == kUnknown_SkColorType {
            return true;
        }
        rowBytes = self.rowBytes();
        let Some(pr) = super::SkMallocPixelRef::MakeAllocate(corrected, rowBytes) else {
            return reset_return_false(self);
        };
        self.setPixelRef(Some(pr), 0, 0);
        if self.getPixels().is_null() {
            return reset_return_false(self);
        }
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:286-289
    // Source: src/core/SkBitmap.cpp:292-292
    // Source: src/core/SkBitmap.cpp:294-305
    pub fn tryAllocPixelsFlags(&mut self, info: &SkImageInfo, _allocFlags: u32) -> bool {
        if !self.setInfo(info, 0) {
            return reset_return_false(self);
        }
        let corrected = self.info();
        let Some(pr) = super::SkMallocPixelRef::MakeAllocate(corrected, corrected.minRowBytes())
        else {
            return reset_return_false(self);
        };
        self.setPixelRef(Some(pr), 0, 0);
        if self.getPixels().is_null() {
            return reset_return_false(self);
        }
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
// Source: src/core/SkBitmap.cpp:307-311
fn invoke_release_proc(
    proc: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    pixels: *mut c_void,
    ctx: *mut c_void,
) {
    if let Some(proc) = proc {
        unsafe {
            proc(pixels, ctx);
        }
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:313-323
    // Source: src/core/SkBitmap.cpp:326-332
    /// Safety: release/context must satisfy their ownership, lifetime and final-thread contract.
    pub unsafe fn installPixels(
        &mut self,
        info: &SkImageInfo,
        pixels: *mut c_void,
        rb: usize,
        release: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
        ctx: *mut c_void,
    ) -> bool {
        if !self.setInfo(info, rb) {
            invoke_release_proc(release, pixels, ctx);
            self.reset();
            return false;
        }
        if pixels.is_null() {
            invoke_release_proc(release, pixels, ctx);
            return true;
        }
        let corrected = self.info();
        let pr = SkMakePixelRefWithProc(
            corrected.width(),
            corrected.height(),
            rb,
            pixels,
            release,
            ctx,
        );
        self.setPixelRef(Some(pr), 0, 0);
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:334-337
    pub fn installPixmap(&mut self, pixmap: &SkPixmap) -> bool {
        // No release callback is registered; this remains a non-owning pixel descriptor.
        unsafe {
            self.installPixels(
                pixmap.info(),
                pixmap.writable_addr().cast(),
                pixmap.rowBytes(),
                None,
                std::ptr::null_mut(),
            )
        }
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:341-343
    pub fn getGenerationID(&self) -> u32 {
        self.fPixelRef.as_ref().map_or(0, |p| p.getGenerationID())
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:345-350
    pub fn notifyPixelsChanged(&self) {
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),346; @condition b"!this->isImmutable()\0".as_ptr(); !self.isImmutable());
        if let Some(pr) = &self.fPixelRef {
            pr.notifyPixelsChanged();
        }
    }
}
// Source: src/core/SkBitmap.cpp:357-372
unsafe impl Allocator for HeapAllocator {
    fn allocPixelRef(&mut self, bitmap: &mut SkBitmap) -> bool {
        if bitmap.colorType()
            == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType
        {
            return false;
        }
        let Some(pixel_ref) =
            super::SkMallocPixelRef::MakeAllocate(bitmap.info(), bitmap.rowBytes())
        else {
            return false;
        };
        bitmap.setPixelRef(Some(pixel_ref), 0, 0);
        #[cfg(sk_debug)]
        bitmap.validate();
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:376-378
    pub fn isImmutable(&self) -> bool {
        self.fPixelRef.as_ref().is_some_and(|p| p.isImmutable())
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:380-384
    pub fn setImmutable(&self) {
        if let Some(pr) = &self.fPixelRef {
            pr.setImmutable();
        }
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:386-388
    // Source: src/core/SkBitmap.cpp:390-395
    pub fn getAddr(&self, x: i32, y: i32) -> *mut u8 {
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),387; @condition b"(unsigned)x < (unsigned)this->width()\0".as_ptr(); (x as u32)<self.width()as u32);
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),388; @condition b"(unsigned)y < (unsigned)this->height()\0".as_ptr(); (y as u32)<self.height()as u32);
        let mut base = self.getPixels();
        if !base.is_null() {
            let offset = (y as usize)
                .wrapping_mul(self.rowBytes())
                .wrapping_add(x.wrapping_shl(self.shiftPerPixel() as u32) as usize);
            base = base.wrapping_add(offset);
        }
        base
    }
}
// Source: src/core/SkBitmap.cpp:400-416
impl SkBitmap {
    /// Safety: live exclusively writable bitmap backing including every alias;
    /// uphold Pixmap.erase's native layout, alignment and conversion lease.
    pub unsafe fn erase(
        &self,
        color: &super::SkColor::SkColor4f,
        area: &super::SkRect::SkIRect,
    ) -> Result<(), crate::RasterError> {
        #[cfg(sk_debug)]
        self.validate();
        if self.colorType() == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType
        {
            return Ok(());
        }
        let mut result = SkPixmap::default();
        if !self.peekPixels(Some(&mut result)) {
            return Ok(());
        }
        if unsafe { result.erase(color, Some(area)) }? {
            self.notifyPixelsChanged();
        }
        Ok(())
    }
    // Source: src/core/SkBitmap.cpp:418-420
    pub unsafe fn erase_area(
        &self,
        color: super::SkColor::SkColor,
        area: &super::SkRect::SkIRect,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.erase(&super::SkColor::SkColor4f::FromColor(color), area) }
    }
    // Source: src/core/SkBitmap.cpp:422-424
    pub unsafe fn eraseColor4f(
        &self,
        color: &super::SkColor::SkColor4f,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.erase(
                color,
                &super::SkRect::SkIRect::make_wh(self.width(), self.height()),
            )
        }
    }
    // Source: src/core/SkBitmap.cpp:426-428
    pub unsafe fn eraseColor(
        &self,
        color: super::SkColor::SkColor,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.erase(
                &super::SkColor::SkColor4f::FromColor(color),
                &super::SkRect::SkIRect::make_wh(self.width(), self.height()),
            )
        }
    }
    // Source: include/core/SkBitmap.h:192-192
    // Source: include/core/SkBitmap.h:192-192
    pub fn rowBytesAsPixels(&self) -> i32 {
        self.fPixmap.rowBytesAsPixels()
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:433-434
    // Source: src/core/SkBitmap.cpp:436-438
    // Source: src/core/SkBitmap.cpp:440-444
    // Source: src/core/SkBitmap.cpp:448-449
    // Source: src/core/SkBitmap.cpp:451-452
    // Source: src/core/SkBitmap.cpp:454-455
    // Source: src/core/SkBitmap.cpp:457-459
    // Source: src/core/SkBitmap.cpp:462-464
    pub fn extractSubset(
        &self,
        result: Option<&mut Self>,
        subset: &crate::src::core::SkRect::SkIRect,
    ) -> bool {
        #[cfg(sk_debug)]
        self.validate();
        let Some(result) = result else {
            return false;
        };
        if self.fPixelRef.is_none() {
            return false;
        }
        let mut srcRect = crate::src::core::SkRect::SkIRect::default();
        let mut r = crate::src::core::SkRect::SkIRect::default();
        srcRect.set_wh(self.width(), self.height());
        if !r.intersect_ab(srcRect, *subset) {
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),448; @condition b"static_cast<unsigned>(r.fLeft) < static_cast<unsigned>(this->width())\0".as_ptr(); (r.fLeft as u32)<self.width()as u32);
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),449; @condition b"static_cast<unsigned>(r.fTop) < static_cast<unsigned>(this->height())\0".as_ptr(); (r.fTop as u32)<self.height()as u32);
        let mut dst = Self::default();
        dst.setInfo(&self.info().makeDimensions(r.size()), self.rowBytes());
        if let Some(pr) = &self.fPixelRef {
            let origin = self.pixelRefOrigin();
            dst.setPixelRef(Some(pr.clone()), origin.fX + r.fLeft, origin.fY + r.fTop);
        }
        #[cfg(sk_debug)]
        dst.validate();
        result.swap(&mut dst);
        true
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:468-475
    /// # Safety
    /// Source backing and destination must satisfy Pixmap readPixels_raw's live
    /// span, exclusivity and native overlap requirements throughout conversion.
    pub unsafe fn readPixels_info(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let mut source = SkPixmap::default();
        if !self.peekPixels(Some(&mut source)) {
            return Ok(false);
        }
        unsafe { source.readPixels_raw(info, pixels, row_bytes, x, y) }
    }
    // Source: src/core/SkBitmap.cpp:477-479
    pub unsafe fn readPixels(
        &self,
        dst: &SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels_info(dst.info(), dst.writable_addr(), dst.rowBytes(), x, y) }
    }
    // Source: src/core/SkBitmap.cpp:481-499
    /// # Safety
    /// Retain a live readable source and exclusive writable bitmap backing; all
    /// aliases must respect native overlap and installed pixel-reference leases.
    pub unsafe fn writePixels(
        &mut self,
        src: &SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        if !super::SkImageInfoPriv::SkImageInfoValidConversion(self.info(), src.info()) {
            return Ok(false);
        }
        let mut rec = super::SkWritePixelsRec::SkWritePixelsRec::from_pixmap(src, x, y);
        if !rec.trim(self.width(), self.height()) {
            return Ok(false);
        }
        let pixels = self.getAddr(rec.fX, rec.fY);
        let info = self.info().makeDimensions(rec.fInfo.dimensions());
        if !unsafe {
            super::SkConvertPixels::SkConvertPixels(
                &info,
                pixels,
                self.rowBytes(),
                &rec.fInfo,
                rec.fPixels,
                rec.fRowBytes,
            )
        }? {
            return Ok(false);
        }
        self.notifyPixelsChanged();
        Ok(true)
    }
    // Source: src/core/SkBitmap.cpp:589-590
    // Source: src/core/SkBitmap.cpp:592-592
    // Source: src/core/SkBitmap.cpp:594-598
    // Source: src/core/SkBitmap.cpp:600-610
    // Source: include/core/SkBitmap.h:1202-1202
    #[cfg(sk_debug)]
    pub fn validate(&self) {
        self.info().validate();
        crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),592; @condition b"this->info().validRowBytes(this->rowBytes())\0".as_ptr(); self.info().validRowBytes(self.rowBytes()));
        if self
            .fPixelRef
            .as_ref()
            .is_some_and(|p| !p.pixels().is_null())
        {
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),595; @condition b"this->getPixels()\0".as_ptr(); !self.getPixels().is_null());
        } else {
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),597; @condition b"!this->getPixels()\0".as_ptr(); self.getPixels().is_null());
        }
        if !self.getPixels().is_null() {
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),601; @condition b"fPixelRef\0".as_ptr(); self.fPixelRef.is_some());
            let pr = self.fPixelRef.as_ref().expect("pixels require pixelref");
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),602; @condition b"fPixelRef->rowBytes() == this->rowBytes()\0".as_ptr(); pr.rowBytes()==self.rowBytes());
            let origin = self.pixelRefOrigin();
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),604; @condition b"origin.fX >= 0\0".as_ptr(); origin.fX>=0);
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),605; @condition b"origin.fY >= 0\0".as_ptr(); origin.fY>=0);
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),606; @condition b"fPixelRef->width() >= (int)this->width() + origin.fX\0".as_ptr(); pr.width()>=self.width()+origin.fX);
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),607; @condition b"fPixelRef->height() >= (int)this->height() + origin.fY\0".as_ptr(); pr.height()>=self.height()+origin.fY);
            crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),608; @condition b"fPixelRef->rowBytes() >= this->info().minRowBytes()\0".as_ptr(); pr.rowBytes()>=self.info().minRowBytes());
        }
    }
}
impl SkBitmap {
    // Source: src/core/SkBitmap.cpp:615-623
    pub fn peekPixels(&self, pmap: Option<&mut SkPixmap>) -> bool {
        if !self.getPixels().is_null() {
            if let Some(pmap) = pmap {
                pmap.clone_from(&self.fPixmap);
            }
            return true;
        }
        false
    }
}

// Source: include/core/SkBitmap.h:1197-1197
// Source: src/core/SkBitmap.cpp:625-625
impl SkBitmap {
    /// Safety: retain the actual readable pixel lease during the native copy/share factory.
    pub unsafe fn asImage(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>> {
        unsafe { crate::src::image::SkImage_RasterFactories::RasterFromBitmap(self) }
    }
}

// Source: src/core/SkBitmap.cpp:503-517
/// Safety: alpha covers every source-sized row exclusively; initialized source
/// pixel backing (if present) remains live through native format conversion.
unsafe fn GetBitmapAlpha(
    source: &SkBitmap,
    mut alpha: *mut u8,
    row_bytes: i32,
) -> Result<bool, crate::RasterError> {
    crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),504; @condition b"alpha != nullptr\0".as_ptr(); !alpha.is_null());
    crate::SkASSERT!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),505; @condition b"alphaRowBytes >= src.width()\0".as_ptr(); row_bytes>=source.width());
    let mut pixmap = SkPixmap::default();
    if !source.peekPixels(Some(&mut pixmap)) {
        for _ in 0..source.height() {
            unsafe {
                core::ptr::write_bytes(alpha, 0, source.width() as usize);
                alpha = alpha.add(row_bytes as usize);
            }
        }
        return Ok(false);
    }
    Ok(unsafe {
        super::SkConvertPixels::SkConvertPixels(
            &SkImageInfo::MakeA8(pixmap.width(), pixmap.height()),
            alpha,
            row_bytes as usize,
            pixmap.info(),
            pixmap.addr(),
            pixmap.rowBytes(),
        )
    }?)
}
// Source: src/core/SkBitmap.cpp:519-584
impl SkBitmap {
    /// Safety: present source pixels are live/initialized, destination/allocator
    /// retain their native backing obligations, and filterMask follows its bounds-
    /// only and actual owned allocation contracts. Pixel conversion errors propagate.
    pub unsafe fn extractAlpha(
        &self,
        destination: &mut Self,
        paint: Option<&super::SkPaint::SkPaint>,
        mut allocator: Option<&mut dyn Allocator>,
        mut offset: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        use super::SkMask::{AllocType, Format, SkAutoMaskFreeImage, SkMaskBuilder};
        self.validate();
        let mut temporary = Self::default();
        let identity = super::SkMatrix::SkMatrix::default();
        let mut source_mask = SkMaskBuilder::new();
        let mut destination_mask = SkMaskBuilder::new();
        if self.width() == 0 || self.height() == 0 {
            return Ok(false);
        }
        source_mask.mask.fBounds = super::SkRect::SkIRect::make_wh(self.width(), self.height());
        source_mask.mask.fRowBytes = (self.width() as u32).wrapping_add(3) & !3;
        source_mask.mask.fFormat = Format::kA8_Format;
        let source_row_bytes = source_mask.fRowBytes;
        let filter = paint.and_then(|paint| paint.getMaskFilter());
        // The original two goto NO_FILTER_CASE edges share this one block. Its
        // captures are released after the last possible fallback, before the
        // filtered allocation path borrows temporary/allocator/offset again.
        let mut no_filter = || -> Result<bool, crate::RasterError> {
            temporary.setInfo(
                &SkImageInfo::MakeA8(self.width(), self.height()),
                source_row_bytes as usize,
            );
            let allocated = match allocator.as_mut() {
                Some(allocator) => temporary.tryAllocPixelsWithAllocator(Some(&mut **allocator)),
                None => temporary.tryAllocPixelsWithAllocator(None),
            };
            if !allocated {
                // Source: src/core/SkBitmap.cpp:547-548
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),547;
                    b"extractAlpha failed to allocate (%d,%d) alpha bitmap\n\0".as_ptr(),
                    temporary.width() as core::ffi::c_int,temporary.height() as core::ffi::c_int);
                }
                return Ok(false);
            }
            // Native ignores the bool (absent source pixels produce zero alpha),
            // while an actual dispatcher error must leave through Result.
            let _ = unsafe {
                GetBitmapAlpha(self, temporary.getPixels().cast(), source_row_bytes as i32)
            }?;
            if let Some(offset) = offset.as_mut() {
                offset.fX = 0;
                offset.fY = 0;
            }
            temporary.swap(destination);
            Ok(true)
        };
        let Some(filter) = filter else {
            return no_filter();
        };
        if !unsafe { filter.filterMask(&mut destination_mask, &source_mask.mask, &identity, None) }?
        {
            return no_filter();
        }
        destination_mask.mask.fRowBytes =
            (destination_mask.fBounds.width() as u32).wrapping_add(3) & !3;
        source_mask.mask.fImage =
            SkMaskBuilder::AllocImage(source_mask.computeImageSize(), AllocType::kUninit_Alloc);
        let _source_cleanup =
            unsafe { SkAutoMaskFreeImage::from_raw(source_mask.fImage as *mut u8) };
        let _ = unsafe {
            GetBitmapAlpha(self, source_mask.fImage as *mut u8, source_row_bytes as i32)
        }?;
        if !unsafe { filter.filterMask(&mut destination_mask, &source_mask.mask, &identity, None) }?
        {
            return no_filter();
        }
        let _destination_cleanup =
            unsafe { SkAutoMaskFreeImage::from_raw(destination_mask.fImage as *mut u8) };
        temporary.setInfo(
            &SkImageInfo::MakeA8(
                destination_mask.fBounds.width(),
                destination_mask.fBounds.height(),
            ),
            destination_mask.fRowBytes as usize,
        );
        let allocated = match allocator.as_mut() {
            Some(allocator) => temporary.tryAllocPixelsWithAllocator(Some(&mut **allocator)),
            None => temporary.tryAllocPixelsWithAllocator(None),
        };
        if !allocated {
            // Source: src/core/SkBitmap.cpp:572-573
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkBitmap.cpp\0".as_ptr(),572;
                b"extractAlpha failed to allocate (%d,%d) alpha bitmap\n\0".as_ptr(),
                temporary.width() as core::ffi::c_int,temporary.height() as core::ffi::c_int);
            }
            return Ok(false);
        }
        unsafe {
            core::ptr::copy_nonoverlapping(
                destination_mask.fImage,
                temporary.getPixels().cast::<u8>(),
                destination_mask.computeImageSize(),
            );
        }
        if let Some(offset) = offset {
            offset.fX = destination_mask.fBounds.fLeft;
            offset.fY = destination_mask.fBounds.fTop;
        }
        temporary.validate();
        temporary.swap(destination);
        Ok(true)
    }
}

// Source: src/core/SkBitmap.cpp:627-631
impl SkBitmap {
    /// Safety: immutable image backing must remain live for every retained shader.
    pub unsafe fn makeShaderClampWithMatrix(
        &self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        unsafe { self.makeShaderClamp(sampling, Some(lm)) }
    }
    // Source: src/core/SkBitmap.cpp:633-637
    pub unsafe fn makeShaderClamp(
        &self,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        use crate::include::core::SkTileMode::SkTileMode;
        unsafe { self.makeShader(SkTileMode::kClamp, SkTileMode::kClamp, sampling, lm) }
    }
    // Source: src/core/SkBitmap.cpp:639-650
    pub unsafe fn makeShaderWithMatrix(
        &self,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        if lm.invert().is_none() {
            return None;
        }
        let image = unsafe {
            crate::src::image::SkImage_Raster::SkImage_Raster::MakeFromBitmap(
                self,
                crate::src::image::SkImage_Raster::SkCopyPixelsMode::kIfMutable,
                None,
            )
        }
        .map(|image| image as std::sync::Arc<dyn crate::src::image::SkImage::SkImage>);
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            image,
            tmx,
            tmy,
            sampling,
            Some(lm),
            false,
        )
    }
    // Source: src/core/SkBitmap.cpp:652-663
    /// Safety: backing retains the native immutable/COW lease through shader use.
    pub unsafe fn makeShader(
        &self,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        if lm.is_some_and(|matrix| matrix.invert().is_none()) {
            return None;
        }
        let image = unsafe {
            crate::src::image::SkImage_Raster::SkImage_Raster::MakeFromBitmap(
                self,
                crate::src::image::SkImage_Raster::SkCopyPixelsMode::kIfMutable,
                None,
            )
        }
        .map(|image| image as std::sync::Arc<dyn crate::src::image::SkImage::SkImage>);
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            image, tmx, tmy, sampling, lm, false,
        )
    }
}
