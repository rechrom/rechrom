// Copyright 2007 The Android Open Source Project. BSD-style license.
use crate::src::core::SkRect::SkIRect;

// Source: src/core/SkMask.h:25-32
// Source: src/core/SkMask.h:35-36
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    kBW_Format,
    kA8_Format,
    k3D_Format,
    kARGB32_Format,
    kLCD16_Format,
    kSDF_Format,
}
pub const kCountMaskFormats: u8 = Format::kSDF_Format as u8 + 1;

// Source: src/core/SkMask.h:39-44
// Source: src/core/SkMask.h:46-46
// Source: src/core/SkMask.h:50-50
// A non-owning descriptor: allocation ownership stays with its caller, as upstream.
#[derive(Clone, Copy)]
pub struct SkMask {
    pub fImage: *const u8,
    pub fBounds: SkIRect,
    pub fRowBytes: u32,
    pub fFormat: Format,
}
impl SkMask {
    pub fn new(img: *const u8, bounds: SkIRect, rowBytes: u32, format: Format) -> Self {
        Self {
            fImage: img,
            fBounds: bounds,
            fRowBytes: rowBytes,
            fFormat: format,
        }
    }
    pub fn IsValidFormat(format: u8) -> bool {
        format < kCountMaskFormats
    }
    pub fn isEmpty(&self) -> bool {
        self.fBounds.is_empty()
    }

    // Source: src/core/SkMask.h:68-72
    pub fn getAddr1(&self, x: i32, y: i32) -> *const u8 {
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),69; @condition b"kBW_Format == fFormat\0".as_ptr(); self.fFormat==Format::kBW_Format);
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),70; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.fBounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),71; @condition b"fImage != nullptr\0".as_ptr(); !self.fImage.is_null());
        self.fImage.wrapping_offset(
            ((x - self.fBounds.fLeft) >> 3) as isize
                + (y - self.fBounds.fTop) as isize * self.fRowBytes as isize,
        )
    }
    // Source: src/core/SkMask.h:79-83
    pub fn getAddr8(&self, x: i32, y: i32) -> *const u8 {
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),80; @condition b"kA8_Format == fFormat || kSDF_Format == fFormat\0".as_ptr(); matches!(self.fFormat, Format::kA8_Format | Format::kSDF_Format));
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),81; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.fBounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),82; @condition b"fImage != nullptr\0".as_ptr(); !self.fImage.is_null());
        self.fImage.wrapping_offset(
            (x - self.fBounds.fLeft) as isize
                + (y - self.fBounds.fTop) as isize * self.fRowBytes as isize,
        )
    }
    // Source: src/core/SkMask.h:91-96
    pub fn getAddrLCD16(&self, x: i32, y: i32) -> *const u16 {
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),92; @condition b"kLCD16_Format == fFormat\0".as_ptr(); self.fFormat==Format::kLCD16_Format);
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),93; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.fBounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),94; @condition b"fImage != nullptr\0".as_ptr(); !self.fImage.is_null());
        let row = self
            .fImage
            .wrapping_offset((y - self.fBounds.fTop) as isize * self.fRowBytes as isize)
            as *const u16;
        row.wrapping_offset((x - self.fBounds.fLeft) as isize)
    }
    // Source: src/core/SkMask.h:104-109
    pub fn getAddr32(&self, x: i32, y: i32) -> *const u32 {
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),105; @condition b"kARGB32_Format == fFormat\0".as_ptr(); self.fFormat==Format::kARGB32_Format);
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),106; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.fBounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkMask.h\0".as_ptr(),107; @condition b"fImage != nullptr\0".as_ptr(); !self.fImage.is_null());
        let row = self
            .fImage
            .wrapping_offset((y - self.fBounds.fTop) as isize * self.fRowBytes as isize)
            as *const u32;
        row.wrapping_offset((x - self.fBounds.fLeft) as isize)
    }
}

// Source: src/core/SkMask.h:136-144
// Source: src/core/SkMask.h:146-146
// Source: src/core/SkMask.h:148-153
// Source: src/core/SkMask.h:155-155
// Source: src/core/SkMask.h:157-159
// Source: src/core/SkMask.h:161-163
// Source: src/core/SkMask.h:165-166
#[derive(Clone, Copy)]
pub struct BWAlphaIter {
    pub fPtr: *const u8,
    pub fOffset: i32,
}
impl BWAlphaIter {
    pub fn new(ptr: *const u8, offset: i32) -> Self {
        Self {
            fPtr: ptr,
            fOffset: 7 - offset,
        }
    }
    pub fn increment(&mut self) -> &mut Self {
        if 0 < self.fOffset {
            self.fOffset -= 1;
        } else {
            self.fPtr = self.fPtr.wrapping_add(1);
            self.fOffset = 7;
        }
        self
    }
    pub fn decrement(&mut self) -> &mut Self {
        if self.fOffset < 7 {
            self.fOffset += 1;
        } else {
            self.fPtr = self.fPtr.wrapping_sub(1);
            self.fOffset = 0;
        }
        self
    }
    pub fn advance_row(&mut self, rb: u32) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_add(rb as usize);
        self
    }
    /// Safety: the iterator must reference a live readable byte.
    pub unsafe fn value(&self) -> u8 {
        if ((unsafe { *self.fPtr } >> self.fOffset) & 1) != 0 {
            255
        } else {
            0
        }
    }
    pub fn less_than(&self, that: &Self) -> bool {
        self.fPtr < that.fPtr || (self.fPtr == that.fPtr && self.fOffset > that.fOffset)
    }
}

// Source: src/core/SkMask.h:169-176
// Source: src/core/SkMask.h:178-180
#[derive(Clone, Copy)]
pub struct A8AlphaIter {
    pub fPtr: *const u8,
}
impl A8AlphaIter {
    pub fn new(ptr: *const u8) -> Self {
        Self { fPtr: ptr }
    }
    pub fn increment(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_add(1);
        self
    }
    pub fn decrement(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_sub(1);
        self
    }
    pub fn advance_row(&mut self, rb: u32) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_add(rb as usize);
        self
    }
    /// Safety: the iterator must reference a live readable byte.
    pub unsafe fn value(&self) -> u8 {
        unsafe { *self.fPtr }
    }
    pub fn less_than(&self, that: &Self) -> bool {
        self.fPtr < that.fPtr
    }
}

// Source: src/core/SkMask.h:183-190
// Source: src/core/SkMask.h:192-194
#[derive(Clone, Copy)]
pub struct ARGB32AlphaIter {
    pub fPtr: *const u32,
}
impl ARGB32AlphaIter {
    pub fn new(ptr: *const u32) -> Self {
        Self { fPtr: ptr }
    }
    pub fn increment(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_add(1);
        self
    }
    pub fn decrement(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_sub(1);
        self
    }
    pub fn advance_row(&mut self, rb: u32) -> &mut Self {
        self.fPtr = self.fPtr.cast::<u8>().wrapping_add(rb as usize).cast();
        self
    }
    /// Safety: the iterator must reference a live aligned readable pixel.
    pub unsafe fn value(&self) -> u8 {
        crate::src::core::SkColorPriv::SkGetPackedA32(unsafe { *self.fPtr }) as u8
    }
    pub fn less_than(&self, that: &Self) -> bool {
        self.fPtr < that.fPtr
    }
}

// Source: src/core/SkMask.h:197-204
// Source: src/core/SkMask.h:206-211
// Source: src/core/SkMask.h:213-214
#[derive(Clone, Copy)]
pub struct LCD16AlphaIter {
    pub fPtr: *const u16,
}
impl LCD16AlphaIter {
    pub fn new(ptr: *const u16) -> Self {
        Self { fPtr: ptr }
    }
    pub fn increment(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_add(1);
        self
    }
    pub fn decrement(&mut self) -> &mut Self {
        self.fPtr = self.fPtr.wrapping_sub(1);
        self
    }
    pub fn advance_row(&mut self, rb: u32) -> &mut Self {
        self.fPtr = self.fPtr.cast::<u8>().wrapping_add(rb as usize).cast();
        self
    }
    /// Safety: the iterator must reference a live aligned readable pixel.
    pub unsafe fn value(&self) -> u8 {
        {
            let packed = unsafe { *self.fPtr } as u32;
            use crate::src::core::SkColorData::{
                SkPacked16ToB32, SkPacked16ToG32, SkPacked16ToR32,
            };
            let r = SkPacked16ToR32(packed);
            let g = SkPacked16ToG32(packed);
            let b = SkPacked16ToB32(packed);
            ((r + g + b) / 3) as u8
        }
    }
    pub fn less_than(&self, that: &Self) -> bool {
        self.fPtr < that.fPtr
    }
}

// Source: src/core/SkMask.h:219-230
// Source: src/core/SkMask.h:233-234
// Source: src/core/SkMask.h:236-239
// Composition replaces inheritance. This descriptor does not automatically free image storage.
pub struct SkMaskBuilder {
    pub mask: SkMask,
}
impl SkMaskBuilder {
    pub fn new() -> Self {
        Self {
            mask: SkMask::new(
                core::ptr::null(),
                SkIRect::make_empty(),
                0,
                Format::kBW_Format,
            ),
        }
    }
    pub fn from_image(img: *mut u8, bounds: SkIRect, row_bytes: u32, format: Format) -> Self {
        Self {
            mask: SkMask::new(img, bounds, row_bytes, format),
        }
    }
    pub fn move_assign(&mut self, that: &mut Self) {
        self.mask = that.mask;
        that.mask.fImage = core::ptr::null();
    }
    pub fn image(&mut self) -> &mut *mut u8 {
        // Pointer mutability does not change representation; only builders expose this projection.
        unsafe { &mut *(&mut self.mask.fImage as *mut *const u8 as *mut *mut u8) }
    }
    pub fn bounds(&mut self) -> &mut SkIRect {
        &mut self.mask.fBounds
    }
    pub fn rowBytes(&mut self) -> &mut u32 {
        &mut self.mask.fRowBytes
    }
    pub fn format(&mut self) -> &mut Format {
        &mut self.mask.fFormat
    }
}
// Rust Deref preserves source access to inherited const descriptor methods.
impl core::ops::Deref for SkMaskBuilder {
    type Target = SkMask;
    fn deref(&self) -> &SkMask {
        &self.mask
    }
}

// Source: src/core/SkMask.h:245-246
// Source: src/core/SkMask.h:253-254
// Source: src/core/SkMask.h:262-263
// Source: src/core/SkMask.h:271-272
// Source: src/core/SkMask.h:287-288
impl SkMaskBuilder {
    pub fn getAddr1(&mut self, x: i32, y: i32) -> *mut u8 {
        self.mask.getAddr1(x, y).cast_mut()
    }
    pub fn getAddr8(&mut self, x: i32, y: i32) -> *mut u8 {
        self.mask.getAddr8(x, y).cast_mut()
    }
    pub fn getAddrLCD16(&mut self, x: i32, y: i32) -> *mut u16 {
        self.mask.getAddrLCD16(x, y).cast_mut()
    }
    pub fn getAddr32(&mut self, x: i32, y: i32) -> *mut u32 {
        self.mask.getAddr32(x, y).cast_mut()
    }
    pub fn getAddr(&mut self, x: i32, y: i32) -> *mut core::ffi::c_void {
        self.mask.getAddr(x, y).cast_mut()
    }
}

// Source: src/core/SkMask.h:291-293
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AllocType {
    kUninit_Alloc,
    kZeroInit_Alloc,
}
// Source: src/core/SkMask.h:298-301
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CreateMode {
    kJustComputeBounds_CreateMode,
    kJustRenderImage_CreateMode,
    kComputeBoundsAndRenderImage_CreateMode,
}

// Source: src/core/SkMask.h:316-316
pub struct SkAutoMaskFreeImage(*mut u8);
impl SkAutoMaskFreeImage {
    /// Safety: image must be an exclusively owned C allocation, released only by this guard.
    pub unsafe fn from_raw(image: *mut u8) -> Self {
        Self(image)
    }
}
impl Drop for SkAutoMaskFreeImage {
    fn drop(&mut self) {
        unsafe {
            SkMaskBuilder::FreeImage(self.0.cast());
        }
    }
}

// Source: src/core/SkMask.cpp:22-25
// Source: src/core/SkMask.cpp:27-27
fn safeMul32(a: i32, b: i32) -> i32 {
    let size = a as i64 * b as i64;
    if size > 0 && size <= i32::MAX as i64 {
        return size as i32;
    }
    0
}

// Source: src/core/SkMask.cpp:30-31
// Source: src/core/SkMask.cpp:34-37
// Source: src/core/SkMask.cpp:39-39
impl SkMask {
    pub fn computeImageSize(&self) -> usize {
        safeMul32(self.fBounds.height(), self.fRowBytes as i32) as usize
    }
    pub fn computeTotalImageSize(&self) -> usize {
        let mut size = self.computeImageSize();
        if self.fFormat == Format::k3D_Format {
            size = safeMul32(size as i32, 3) as usize;
        }
        size
    }
}

// Source: src/core/SkMask.cpp:45-49
// Source: src/core/SkMask.cpp:51-51
// Source: src/core/SkMask.cpp:57-58
// C allocator preserves transferable bitmap/mask allocation ownership; the caller must free once.
unsafe extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn calloc(count: usize, size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}
impl SkMaskBuilder {
    pub fn AllocImage(size: usize, at: AllocType) -> *mut u8 {
        let aligned_size = size
            .checked_add(3)
            .map(|s| s & !3)
            .expect("mask allocation overflow");
        let image = unsafe {
            if at == AllocType::kZeroInit_Alloc {
                calloc(1, aligned_size)
            } else {
                malloc(aligned_size)
            }
        };
        if image.is_null() && aligned_size != 0 {
            std::alloc::handle_alloc_error(
                std::alloc::Layout::from_size_align(aligned_size, 4).unwrap(),
            );
        }
        image.cast()
    }
    /// Safety: image must be a live C allocation with no outstanding users.
    pub unsafe fn FreeImage(image: *mut core::ffi::c_void) {
        unsafe {
            free(image);
        }
    }
}

// Source: src/core/SkMask.cpp:61-62
// Source: src/core/SkMask.cpp:64-66
// Source: src/core/SkMask.cpp:69-69
// Source: src/core/SkMask.cpp:71-71
// Source: src/core/SkMask.cpp:73-73
// Source: src/core/SkMask.cpp:76-79
// Source: src/core/SkMask.cpp:82-85
// Source: src/core/SkMask.cpp:87-88
// Source: src/core/SkMask.cpp:91-91
impl SkMaskBuilder {
    pub fn PrepareDestination(radiusX: i32, radiusY: i32, src: &SkMask) -> Self {
        let mut dst = Self::new();
        dst.mask.fImage = core::ptr::null();
        dst.mask.fFormat = Format::kA8_Format;
        let dst_w = (radiusX as usize)
            .checked_add(radiusX as usize)
            .and_then(|v| (src.fBounds.width() as usize).checked_add(v));
        let dst_h = (radiusY as usize)
            .checked_add(radiusY as usize)
            .and_then(|v| (src.fBounds.height() as usize).checked_add(v));
        let size = dst_w.zip(dst_h).and_then(|(w, h)| w.checked_mul(h));
        if dst_w.is_none_or(|w| w > i32::MAX as usize)
            || dst_h.is_none_or(|h| h > i32::MAX as usize)
            || size.is_none_or(|n| n > i32::MAX as usize)
        {
            dst.mask.fBounds.set_empty();
            dst.mask.fRowBytes = 0;
            return dst;
        }
        let (w, h) = (dst_w.unwrap(), dst_h.unwrap());
        dst.mask.fBounds.set_wh(w as i32, h as i32);
        dst.mask.fBounds.offset(src.fBounds.fLeft, src.fBounds.fTop);
        dst.mask.fBounds.offset(-radiusX, -radiusY);
        dst.mask.fRowBytes = w as u32;
        if !src.fImage.is_null() {
            dst.mask.fImage = Self::AllocImage(size.unwrap(), AllocType::kUninit_Alloc);
        }
        dst
    }
}
// Source: src/core/SkMask.cpp:97-103
// Source: src/core/SkMask.cpp:106-109
const MASK_FORMAT_TO_SHIFT: [i32; 6] = [!0, 0, 0, 2, 1, 0];
fn maskFormatToShift(format: Format) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkMask.cpp\0".as_ptr(),107; @condition b"(unsigned)format < std::size(gMaskFormatToShift)\0".as_ptr(); (format as usize) < MASK_FORMAT_TO_SHIFT.len());
    crate::SkASSERT!(@source b"src/core/SkMask.cpp\0".as_ptr(),108; @condition b"SkMask::kBW_Format != format\0".as_ptr(); format!=Format::kBW_Format);
    MASK_FORMAT_TO_SHIFT[format as usize]
}

// Source: src/core/SkMask.cpp:112-115
// Source: src/core/SkMask.cpp:117-120
impl SkMask {
    pub fn getAddr(&self, x: i32, y: i32) -> *const core::ffi::c_void {
        crate::SkASSERT!(@source b"src/core/SkMask.cpp\0".as_ptr(),113; @condition b"kBW_Format != fFormat\0".as_ptr(); self.fFormat!=Format::kBW_Format);
        crate::SkASSERT!(@source b"src/core/SkMask.cpp\0".as_ptr(),114; @condition b"fBounds.contains(x, y)\0".as_ptr(); self.fBounds.contains_xy(x, y));
        crate::SkASSERT!(@source b"src/core/SkMask.cpp\0".as_ptr(),115; @condition b"fImage\0".as_ptr(); !self.fImage.is_null());
        let addr = self
            .fImage
            .wrapping_offset((y - self.fBounds.fTop) as isize * self.fRowBytes as isize);
        addr.wrapping_offset(((x - self.fBounds.fLeft) << maskFormatToShift(self.fFormat)) as isize)
            .cast()
    }
}

// Source: src/core/SkMask.h:133-133
// Only native-defined specializations provide an associated type; unsupported
// formats remain incomplete rather than getting an invented generic iterator.
pub trait AlphaIterSpecialization<const F: u8> {
    type Iter;
}
impl AlphaIterSpecialization<{ Format::kBW_Format as u8 }> for Format {
    type Iter = BWAlphaIter;
}
impl AlphaIterSpecialization<{ Format::kA8_Format as u8 }> for Format {
    type Iter = A8AlphaIter;
}
impl AlphaIterSpecialization<{ Format::kARGB32_Format as u8 }> for Format {
    type Iter = ARGB32AlphaIter;
}
impl AlphaIterSpecialization<{ Format::kLCD16_Format as u8 }> for Format {
    type Iter = LCD16AlphaIter;
}
pub type AlphaIter<const F: u8>
where
    Format: AlphaIterSpecialization<F>,
= <Format as AlphaIterSpecialization<F>>::Iter;
