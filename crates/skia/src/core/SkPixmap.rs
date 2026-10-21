// Copyright 2015 Google Inc. BSD-style license.
use super::SkColorSpace::SkColorSpace;
use super::SkImageInfo::SkImageInfo;
use crate::include::core::{SkAlphaType::SkAlphaType, SkColorType::*, SkSize::SkISize};
use crate::src::core::SkRect::SkIRect;
use std::sync::Arc;
// Source: include/core/SkPixmap.h:50-52
impl Default for SkPixmap {
    fn default() -> Self {
        Self {
            fPixels: std::ptr::null(),
            fRowBytes: 0,
            fInfo: SkImageInfo::MakeUnknown(0, 0),
        }
    }
}
// Source: include/core/SkPixmap.h:72-74
impl SkPixmap {
    pub fn new(info: &SkImageInfo, addr: *const u8, rowBytes: usize) -> Self {
        Self {
            fPixels: addr,
            fRowBytes: rowBytes,
            fInfo: info.clone(),
        }
    }
}
// Source: include/core/SkPixmap.h:135-135
impl SkPixmap {
    pub fn info(&self) -> &SkImageInfo {
        &self.fInfo
    }
}
// Source: include/core/SkPixmap.h:145-145
// Source: include/core/SkPixmap.h:153-153
impl SkPixmap {
    pub fn rowBytes(&self) -> usize {
        self.fRowBytes
    }
    pub fn addr(&self) -> *const u8 {
        self.fPixels
    }
}
// Source: include/core/SkPixmap.h:160-160
// Source: include/core/SkPixmap.h:166-166
// Source: include/core/SkPixmap.h:171-171
// Source: include/core/SkPixmap.h:176-176
// Source: include/core/SkPixmap.h:178-178
// Source: include/core/SkPixmap.h:180-180
impl SkPixmap {
    pub fn width(&self) -> i32 {
        self.fInfo.width()
    }
    pub fn height(&self) -> i32 {
        self.fInfo.height()
    }
    pub fn isEmpty(&self) -> bool {
        self.fInfo.isEmpty()
    }
    pub fn dimensions(&self) -> SkISize {
        self.fInfo.dimensions()
    }
    pub fn colorType(&self) -> SkColorType {
        self.fInfo.colorType()
    }
    pub fn alphaType(&self) -> SkAlphaType {
        self.fInfo.alphaType()
    }
}
// Source: include/core/SkPixmap.h:206-206
// Source: include/core/SkPixmap.h:212-212
// Source: include/core/SkPixmap.h:219-219
// Source: include/core/SkPixmap.h:226-226
// Source: include/core/SkPixmap.h:236-236
impl SkPixmap {
    pub fn isOpaque(&self) -> bool {
        self.fInfo.isOpaque()
    }
    pub fn bounds(&self) -> SkIRect {
        SkIRect::make_wh(self.width(), self.height())
    }
    pub fn rowBytesAsPixels(&self) -> i32 {
        (self.fRowBytes >> self.shiftPerPixel()) as i32
    }
    pub fn shiftPerPixel(&self) -> i32 {
        self.fInfo.shiftPerPixel()
    }
    pub fn computeByteSize(&self) -> usize {
        self.fInfo.computeByteSize(self.fRowBytes)
    }
}
// Source: include/core/SkPixmap.h:319-321
impl SkPixmap {
    pub fn addr_xy(&self, x: i32, y: i32) -> *const u8 {
        self.fPixels
            .wrapping_add(self.fInfo.computeOffset(x, y, self.fRowBytes))
    }
}
// Source: include/core/SkPixmap.h:331-334
impl SkPixmap {
    pub fn addr8(&self) -> *const u8 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),332; @condition b"1 == fInfo.bytesPerPixel()\0".as_ptr(); self.fInfo.bytesPerPixel() == 1);
        self.fPixels.cast::<u8>()
    }
}
// Source: include/core/SkPixmap.h:344-347
impl SkPixmap {
    pub fn addr16(&self) -> *const u16 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),345; @condition b"2 == fInfo.bytesPerPixel()\0".as_ptr(); self.fInfo.bytesPerPixel() == 2);
        self.fPixels.cast::<u16>()
    }
}
// Source: include/core/SkPixmap.h:357-360
impl SkPixmap {
    pub fn addr32(&self) -> *const u32 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),358; @condition b"4 == fInfo.bytesPerPixel()\0".as_ptr(); self.fInfo.bytesPerPixel() == 4);
        self.fPixels.cast::<u32>()
    }
}
// Source: include/core/SkPixmap.h:370-373
impl SkPixmap {
    pub fn addr64(&self) -> *const u64 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),371; @condition b"8 == fInfo.bytesPerPixel()\0".as_ptr(); self.fInfo.bytesPerPixel() == 8);
        self.fPixels.cast::<u64>()
    }
}
// Source: include/core/SkPixmap.h:384-389
impl SkPixmap {
    pub fn addrF16(&self) -> *const u16 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),385; @condition b"8 == fInfo.bytesPerPixel()\0".as_ptr(); self.fInfo.bytesPerPixel() == 8);
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),386; @condition b"kRGBA_F16_SkColorType == fInfo.colorType() || kRGBA_F16Norm_SkColorType == fInfo.colorType()\0".as_ptr(); self.fInfo.colorType() == kRGBA_F16_SkColorType
                || self.fInfo.colorType() == kRGBA_F16Norm_SkColorType);
        self.fPixels.cast::<u16>()
    }
}
// Source: include/core/SkPixmap.h:403-407
impl SkPixmap {
    pub fn addr8_xy(&self, x: i32, y: i32) -> *const u8 {
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),404; @condition b"x >= 0 && x < this->width()\0".as_ptr(); x >= 0 && x < self.width(),"x=%d; width=%d\n",x as core::ffi::c_int,self.fInfo.width() as core::ffi::c_int);
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),405; @condition b"y >= 0 && y < this->height()\0".as_ptr(); y >= 0 && y < self.height(),"y=%d; height=%d\n",y as core::ffi::c_int,self.fInfo.height() as core::ffi::c_int);
        self.addr8()
            .cast::<u8>()
            .wrapping_add(
                (y as usize)
                    .wrapping_mul(self.fRowBytes)
                    .wrapping_add((x << 0) as usize),
            )
            .cast::<u8>()
    }
}
// Source: include/core/SkPixmap.h:421-425
impl SkPixmap {
    pub fn addr16_xy(&self, x: i32, y: i32) -> *const u16 {
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),422; @condition b"x >= 0 && x < this->width()\0".as_ptr(); x >= 0 && x < self.width(),"x=%d; width=%d\n",x as core::ffi::c_int,self.fInfo.width() as core::ffi::c_int);
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),423; @condition b"y >= 0 && y < this->height()\0".as_ptr(); y >= 0 && y < self.height(),"y=%d; height=%d\n",y as core::ffi::c_int,self.fInfo.height() as core::ffi::c_int);
        self.addr16()
            .cast::<u8>()
            .wrapping_add(
                (y as usize)
                    .wrapping_mul(self.fRowBytes)
                    .wrapping_add((x << 1) as usize),
            )
            .cast::<u16>()
    }
}
// Source: include/core/SkPixmap.h:439-443
impl SkPixmap {
    pub fn addr32_xy(&self, x: i32, y: i32) -> *const u32 {
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),440; @condition b"x >= 0 && x < this->width()\0".as_ptr(); x >= 0 && x < self.width(),"x=%d; width=%d\n",x as core::ffi::c_int,self.fInfo.width() as core::ffi::c_int);
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),441; @condition b"y >= 0 && y < this->height()\0".as_ptr(); y >= 0 && y < self.height(),"y=%d; height=%d\n",y as core::ffi::c_int,self.fInfo.height() as core::ffi::c_int);
        self.addr32()
            .cast::<u8>()
            .wrapping_add(
                (y as usize)
                    .wrapping_mul(self.fRowBytes)
                    .wrapping_add((x << 2) as usize),
            )
            .cast::<u32>()
    }
}
// Source: include/core/SkPixmap.h:457-461
impl SkPixmap {
    pub fn addr64_xy(&self, x: i32, y: i32) -> *const u64 {
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),458; @condition b"x >= 0 && x < this->width()\0".as_ptr(); x >= 0 && x < self.width(),"x=%d; width=%d\n",x as core::ffi::c_int,self.fInfo.width() as core::ffi::c_int);
        crate::SkASSERTF!(@source b"include/core/SkPixmap.h\0".as_ptr(),459; @condition b"y >= 0 && y < this->height()\0".as_ptr(); y >= 0 && y < self.height(),"y=%d; height=%d\n",y as core::ffi::c_int,self.fInfo.height() as core::ffi::c_int);
        self.addr64()
            .cast::<u8>()
            .wrapping_add(
                (y as usize)
                    .wrapping_mul(self.fRowBytes)
                    .wrapping_add((x << 3) as usize),
            )
            .cast::<u64>()
    }
}
// Source: include/core/SkPixmap.h:478-482
impl SkPixmap {
    pub fn addrF16_xy(&self, x: i32, y: i32) -> *const u16 {
        crate::SkASSERT!(@source b"include/core/SkPixmap.h\0".as_ptr(),479; @condition b"kRGBA_F16_SkColorType == fInfo.colorType() || kRGBA_F16Norm_SkColorType == fInfo.colorType()\0".as_ptr(); self.fInfo.colorType() == kRGBA_F16_SkColorType
                || self.fInfo.colorType() == kRGBA_F16Norm_SkColorType);
        self.addr64_xy(x, y).cast::<u16>()
    }
}
// Source: include/core/SkPixmap.h:488-488
impl SkPixmap {
    pub fn writable_addr(&self) -> *mut u8 {
        self.fPixels.cast_mut()
    }
}
// Source: include/core/SkPixmap.h:499-501
impl SkPixmap {
    pub fn writable_addr_xy(&self, x: i32, y: i32) -> *mut u8 {
        self.addr_xy(x, y).cast_mut()
    }
}
// Source: include/core/SkPixmap.h:513-515
impl SkPixmap {
    pub fn writable_addr8(&self, x: i32, y: i32) -> *mut u8 {
        self.addr8_xy(x, y).cast_mut()
    }
}
// Source: include/core/SkPixmap.h:527-529
impl SkPixmap {
    pub fn writable_addr16(&self, x: i32, y: i32) -> *mut u16 {
        self.addr16_xy(x, y).cast_mut()
    }
}
// Source: include/core/SkPixmap.h:542-544
impl SkPixmap {
    pub fn writable_addr32(&self, x: i32, y: i32) -> *mut u32 {
        self.addr32_xy(x, y).cast_mut()
    }
}
// Source: include/core/SkPixmap.h:556-558
impl SkPixmap {
    pub fn writable_addr64(&self, x: i32, y: i32) -> *mut u64 {
        self.addr64_xy(x, y).cast_mut()
    }
}
// Source: include/core/SkPixmap.h:571-573
impl SkPixmap {
    pub fn writable_addrF16(&self, x: i32, y: i32) -> *mut u16 {
        self.writable_addr64(x, y).cast::<u16>()
    }
}
// Source: include/core/SkPixmap.h:728-728
impl SkPixmap {
    /// Destination retains the same exclusive initialized/writable backing
    /// lease as erase; native null subset selects the original entire pixmap.
    pub unsafe fn erase_default(
        &self,
        color: &super::SkColor::SkColor4f,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.erase(color, None) }
    }
}

// Source: include/core/SkPixmap.h:731-734
// This descriptor never owns its pixel allocation. Callers retain the upstream lifetime obligation.
pub struct SkPixmap {
    // Source: include/core/SkPixmap.h:731-731
    fPixels: *const u8,
    // Source: include/core/SkPixmap.h:732-732
    fRowBytes: usize,
    // Source: include/core/SkPixmap.h:733-733
    fInfo: SkImageInfo,
}
// Source: include/core/SkPixmap.h:40-40
// Implicit native member copy/assignment: the descriptor copies its pointer and
// stride, and retains only the original ImageInfo color-space owner. Pixels are
// never retained, freed, or made into a slice by copying this descriptor.
impl Clone for SkPixmap {
    fn clone(&self) -> Self {
        Self {
            fPixels: self.fPixels,
            fRowBytes: self.fRowBytes,
            fInfo: self.fInfo.clone(),
        }
    }
    fn clone_from(&mut self, src: &Self) {
        self.fPixels = src.fPixels;
        self.fRowBytes = src.fRowBytes;
        self.fInfo.clone_from(&src.fInfo);
    }
}
// Source: src/core/SkPixmap.cpp:38-46
impl SkPixmap {
    // Field-domain projection for reset(info(), addr, rb). The native self-copy
    // leaves that same ImageInfo in place; no temporary color-space retain is
    // needed to borrow the immutable metadata and mutate the other two fields.
    pub(crate) fn reset_address(&mut self, addr: *const u8, rowBytes: usize) {
        if !addr.is_null() {
            #[cfg(sk_debug)]
            self.fInfo.validate();
            crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),41;
                @condition b"info.validRowBytes(rowBytes)\0".as_ptr();self.fInfo.validRowBytes(rowBytes));
        }
        self.fPixels = addr;
        self.fRowBytes = rowBytes;
    }
}
// Source: include/core/SkPixmap.h:597-599
impl SkPixmap {
    /// Safety: destination pixels are exclusively writable for their declared
    /// layout; initialized source backing remains live throughout conversion.
    pub unsafe fn readPixels_info(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels_raw(info, pixels, row_bytes, 0, 0) }
    }
    // Source: include/core/SkPixmap.h:652-654
    pub unsafe fn readPixels_at(
        &self,
        dst: &Self,
        src_x: i32,
        src_y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.readPixels_raw(
                dst.info(),
                dst.writable_addr(),
                dst.rowBytes(),
                src_x,
                src_y,
            )
        }
    }
    // Source: include/core/SkPixmap.h:673-675
    pub unsafe fn readPixels(&self, dst: &Self) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels_raw(dst.info(), dst.writable_addr(), dst.rowBytes(), 0, 0) }
    }
    // Source: include/core/SkPixmap.h:717-717
    /// Safety: all destination pixels retain an exclusive writable backing lease.
    pub unsafe fn eraseColorAll(
        &self,
        color: super::SkColor::SkColor,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.eraseColor(color, &self.bounds()) }
    }
}

// Source: src/core/SkPixmap.cpp:32-36
// Source: src/core/SkPixmap.cpp:38-46
impl SkPixmap {
    pub fn reset(&mut self) {
        self.fPixels = std::ptr::null();
        self.fRowBytes = 0;
        self.fInfo = SkImageInfo::MakeUnknown(0, 0);
    }
    pub fn reset_with(&mut self, info: &SkImageInfo, addr: *const u8, rowBytes: usize) {
        if !addr.is_null() {
            #[cfg(sk_debug)]
            info.validate();
            crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),41; @condition b"info.validRowBytes(rowBytes)\0".as_ptr(); info.validRowBytes(rowBytes));
        }
        self.fPixels = addr;
        self.fRowBytes = rowBytes;
        self.fInfo.clone_from(info);
    }
}
// Source: src/core/SkPixmap.cpp:48-56
impl SkPixmap {
    pub fn reset_mask(&mut self, src: &crate::src::core::SkMask::SkMask) -> bool {
        if src.fFormat == crate::src::core::SkMask::Format::kA8_Format {
            self.reset_with(
                &SkImageInfo::MakeA8(src.fBounds.width(), src.fBounds.height()),
                src.fImage,
                src.fRowBytes as usize,
            );
            return true;
        }
        self.reset();
        false
    }
}
// Source: src/core/SkPixmap.cpp:58-60
// Source: src/core/SkPixmap.cpp:62-62
// Source: src/core/SkPixmap.cpp:64-64
impl SkPixmap {
    pub fn setColorSpace(&mut self, cs: Option<Arc<SkColorSpace>>) {
        self.fInfo = self.fInfo.makeColorSpace(cs);
    }
    pub fn colorSpace(&self) -> Option<&SkColorSpace> {
        self.fInfo.colorSpace()
    }
    pub fn refColorSpace(&self) -> Option<Arc<SkColorSpace>> {
        self.fInfo.refColorSpace()
    }
}
// Source: src/core/SkPixmap.cpp:66-71
// Source: src/core/SkPixmap.cpp:75-76
// Source: src/core/SkPixmap.cpp:78-85
impl SkPixmap {
    pub fn extractSubset(&self, result: &mut SkPixmap, subset: &SkIRect) -> bool {
        let srcRect = SkIRect::make_wh(self.width(), self.height());
        let mut r = srcRect;
        if !r.intersect(subset) {
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),75; @condition b"static_cast<unsigned>(r.fLeft) < static_cast<unsigned>(this->width())\0".as_ptr(); (r.fLeft as u32) < self.width() as u32);
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),76; @condition b"static_cast<unsigned>(r.fTop) < static_cast<unsigned>(this->height())\0".as_ptr(); (r.fTop as u32) < self.height() as u32);
        let mut pixels = std::ptr::null();
        if !self.fPixels.is_null() {
            let bpp = self.fInfo.bytesPerPixel() as usize;
            pixels = self.fPixels.wrapping_add(
                (r.fTop as usize)
                    .wrapping_mul(self.fRowBytes)
                    .wrapping_add((r.fLeft as usize).wrapping_mul(bpp)),
            );
        }
        result.reset_with(&self.fInfo.makeDimensions(r.size()), pixels, self.fRowBytes);
        true
    }
}
// Source: src/core/SkPixmap.cpp:90-93
unsafe fn fast_getaddr(pm: &SkPixmap, mut x: i32, y: i32) -> *const u8 {
    x = x.wrapping_shl(super::SkImageInfoPriv::SkColorTypeShiftPerPixel(pm.colorType()) as u32);
    pm.addr().wrapping_offset(
        (y as usize)
            .wrapping_mul(pm.rowBytes())
            .wrapping_add(x as usize) as isize,
    )
}
// Source: src/core/SkPixmap.cpp:95-98
// Source: src/core/SkPixmap.cpp:100-101
// Source: src/core/SkPixmap.cpp:103-172
impl SkPixmap {
    /// Descriptor pixels must be readable, initialized, and cover the requested coordinate.
    pub unsafe fn getAlphaf(&self, x: i32, y: i32) -> f32 {
        use crate::include::core::SkColorType::SkColorType::*;
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),96; @condition b"this->addr()\0".as_ptr(); !self.addr().is_null());
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),97; @condition b"(unsigned)x < (unsigned)this->width()\0".as_ptr(); (x as u32) < self.width() as u32);
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),98; @condition b"(unsigned)y < (unsigned)this->height()\0".as_ptr(); (y as u32) < self.height() as u32);
        let mut value = 0.0;
        let p = fast_getaddr(self, x, y);
        match self.colorType() {
            kUnknown_SkColorType => return 0.0,
            kGray_8_SkColorType
            | kR8G8_unorm_SkColorType
            | kR16_unorm_SkColorType
            | kR16_float_SkColorType
            | kR16G16_unorm_SkColorType
            | kR16G16_float_SkColorType
            | kRGB_565_SkColorType
            | kRGB_888x_SkColorType
            | kRGB_101010x_SkColorType
            | kBGR_101010x_SkColorType
            | kBGR_101010x_XR_SkColorType
            | kRGB_F16F16F16x_SkColorType
            | kR8_unorm_SkColorType => return 1.0,
            kAlpha_8_SkColorType => value = *p as f32 * (1.0 / 255.0),
            kA16_unorm_SkColorType => {
                value = p.cast::<u16>().read_unaligned() as f32 * (1.0 / 65535.0)
            }
            kA16_float_SkColorType => {
                let half = p.cast::<u16>().read_unaligned();
                value = super::SkHalf::SkHalfToFloat(half);
            }
            kARGB_4444_SkColorType => {
                let u = p.cast::<u16>().read_unaligned();
                value = super::SkColorData::SkGetPackedA4444(u as u32) as f32 * (1.0 / 15.0);
            }
            kRGBA_8888_SkColorType | kBGRA_8888_SkColorType | kSRGBA_8888_SkColorType => {
                value = *p.add(3) as f32 * (1.0 / 255.0)
            }
            kRGBA_1010102_SkColorType | kBGRA_1010102_SkColorType => {
                let u = p.cast::<u32>().read_unaligned();
                value = (u >> 30) as f32 * (1.0 / 3.0);
            }
            kBGRA_10101010_XR_SkColorType => {
                let u = p.cast::<u64>().read_unaligned();
                value = (u >> 54).wrapping_sub(384) as f32 / 510.0;
            }
            kRGBA_10x6_SkColorType => {
                let u = p.cast::<u64>().read_unaligned();
                value = (u >> 54) as f32 * (1.0 / 1023.0);
            }
            kR16G16B16A16_unorm_SkColorType => {
                let u = p.cast::<u64>().read_unaligned();
                value = (u >> 48) as f32 * (1.0 / 65535.0);
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                let h: [u16; 4] = std::array::from_fn(|i| p.cast::<u16>().add(i).read_unaligned());
                value = super::SkVx::from_half(h)[3];
            }
            kRGBA_F32_SkColorType => value = p.cast::<f32>().add(3).read_unaligned(),
        }
        value
    }
}

// Source: src/core/SkPixmap.cpp:174-189
impl SkPixmap {
    /// # Safety
    /// Source/destination must provide live, aligned pixel spans for the declared
    /// layouts, with exclusive destination access and native conversion overlap rules.
    pub unsafe fn readPixels_raw(
        &self,
        dst_info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        if !super::SkImageInfoPriv::SkImageInfoValidConversion(dst_info, self.info()) {
            return Ok(false);
        }
        let mut rec =
            super::SkReadPixelsRec::SkReadPixelsRec::new(dst_info, pixels, row_bytes, x, y);
        if !rec.trim(self.width(), self.height()) {
            return Ok(false);
        }
        let src_pixels = self.addr_xy(rec.fX, rec.fY);
        let src_info = self.info().makeDimensions(rec.fInfo.dimensions());
        Ok(unsafe {
            super::SkConvertPixels::SkConvertPixels(
                &rec.fInfo,
                rec.fPixels,
                rec.fRowBytes,
                &src_info,
                src_pixels,
                self.rowBytes(),
            )
        }?)
    }
}

// Source: src/core/SkPixmap.cpp:191-194
// Source: src/core/SkPixmap.cpp:196-200
// Source: src/core/SkPixmap.cpp:202-266
// Source: src/core/SkPixmap.cpp:268-287
// Source: src/core/SkPixmap.cpp:289-298
// Source: src/core/SkPixmap.cpp:300-357
// Source: src/core/SkPixmap.cpp:359-369
// Source: src/core/SkPixmap.cpp:371-376
// Source: src/core/SkPixmap.cpp:378-381
// Source: src/core/SkPixmap.cpp:383-390
impl SkPixmap {
    /// Pixels must be initialized and readable at the requested coordinate.
    pub unsafe fn getColor(&self, x: i32, y: i32) -> super::SkColor::SkColor {
        use super::SkColor::*;
        use super::SkColorData::*;
        use super::SkHalf::SkHalfToFloat;
        use super::SkSwizzlePriv::{swizzle_rb, Sk4f_toL32};
        use super::SkUnPreMultiply::SkUnPreMultiply;
        use crate::include::core::SkColorType::SkColorType::*;
        use crate::include::private::SkTPin::SkTPin;
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),192; @condition b"this->addr()\0".as_ptr(); !self.addr().is_null());
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),193; @condition b"(unsigned)x < (unsigned)this->width()\0".as_ptr(); (x as u32) < self.width() as u32);
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),194; @condition b"(unsigned)y < (unsigned)this->height()\0".as_ptr(); (y as u32) < self.height() as u32);
        let needsUnpremul = self.fInfo.alphaType() == SkAlphaType::kPremul_SkAlphaType;
        let toColor = |c| {
            if needsUnpremul {
                SkUnPreMultiply::PMColorToColor(c)
            } else {
                SkSwizzle_BGRA_to_PMColor(c)
            }
        };
        match self.colorType() {
            kGray_8_SkColorType => {
                let v = *self.addr8_xy(x, y) as u32;
                return SkColorSetRGB(v, v, v);
            }
            kR8_unorm_SkColorType => return SkColorSetRGB(*self.addr8_xy(x, y) as u32, 0, 0),
            kAlpha_8_SkColorType => return SkColorSetA(0, *self.addr8_xy(x, y) as u32),
            kA16_unorm_SkColorType => {
                return SkColorSetA(
                    0,
                    (self.addr16_xy(x, y).read_unaligned() as f32 * (255.0 / 65535.0)) as u32,
                )
            }
            kA16_float_SkColorType => {
                return SkColorSetA(
                    0,
                    (255.0 * SkHalfToFloat(self.addr16_xy(x, y).read_unaligned())) as u32,
                )
            }
            kR16_float_SkColorType => {
                return SkColorSetRGB(
                    (255.0 * SkHalfToFloat(self.addr16_xy(x, y).read_unaligned())) as u32,
                    0,
                    0,
                )
            }
            kRGB_565_SkColorType => {
                return SkPixel16ToColor(self.addr16_xy(x, y).read_unaligned() as u32)
            }
            kARGB_4444_SkColorType => {
                return toColor(SkPixel4444ToPixel32(
                    self.addr16_xy(x, y).read_unaligned() as u32
                ))
            }
            kR8G8_unorm_SkColorType => {
                let v = self.addr16_xy(x, y).read_unaligned();
                return SkColorSetRGB(v as u8 as u32, (v >> 8) as u8 as u32, 0);
            }
            kR16_unorm_SkColorType => {
                let v = self.addr16_xy(x, y).read_unaligned();
                return SkColorSetRGB((v as f32 * (255.0 / 65535.0)) as u32, 0, 0);
            }
            kR16G16_unorm_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                let r = ((v & 65535) as f32 * (255.0 / 65535.0)) as u8;
                let g = ((v >> 16) as f32 * (255.0 / 65535.0)) as u8;
                return SkColorSetRGB(r as u32, g as u32, 0);
            }
            kR16G16_float_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                let r = SkHalfToFloat(v as u16);
                let g = SkHalfToFloat((v >> 16) as u16);
                return SkColorSetRGB((255.0 * r) as u8 as u32, (255.0 * g) as u8 as u32, 0);
            }
            kRGB_888x_SkColorType => {
                return SkSwizzle_RB(self.addr32_xy(x, y).read_unaligned() | 0xff000000)
            }
            kBGRA_8888_SkColorType => {
                return toColor(SkSwizzle_BGRA_to_PMColor(
                    self.addr32_xy(x, y).read_unaligned(),
                ))
            }
            kRGBA_8888_SkColorType => {
                return toColor(SkSwizzle_RGBA_to_PMColor(
                    self.addr32_xy(x, y).read_unaligned(),
                ))
            }
            kSRGBA_8888_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                let mut r = (v & 255) as f32 * (1.0 / 255.0);
                let mut g = ((v >> 8) & 255) as f32 * (1.0 / 255.0);
                let mut b = ((v >> 16) & 255) as f32 * (1.0 / 255.0);
                let mut a = (v >> 24) as f32 * (1.0 / 255.0);
                let srgb_to_linear = |v: f32| {
                    if v <= 0.04045 {
                        v * (1.0 / 12.92)
                    } else {
                        (v * (1.0 / 1.055) + (0.055 / 1.055)).powf(2.4)
                    }
                };
                r = srgb_to_linear(r);
                g = srgb_to_linear(g);
                b = srgb_to_linear(b);
                if a != 0.0 && needsUnpremul {
                    r = SkTPin(r / a, 0.0, 1.0);
                    g = SkTPin(g / a, 0.0, 1.0);
                    b = SkTPin(b / a, 0.0, 1.0);
                }
                r *= 255.0;
                g *= 255.0;
                b *= 255.0;
                a *= 255.0;
                return SkColorSetARGB(a as u32, r as u32, g as u32, b as u32);
            }
            kRGB_101010x_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return SkColorSetRGB(
                    ((v & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 10) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 20) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                );
            }
            kBGR_101010x_XR_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),294; @condition b"false\0".as_ptr(); false);
                return 0;
            }
            kBGR_101010x_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return SkColorSetRGB(
                    (((v >> 20) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 10) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    ((v & 1023) as f32 * (255.0 / 1023.0)) as u32,
                );
            }
            kBGRA_1010102_SkColorType | kRGBA_1010102_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                let mut r = (v & 1023) as f32 * (1.0 / 1023.0);
                let mut g = ((v >> 10) & 1023) as f32 * (1.0 / 1023.0);
                let mut b = ((v >> 20) & 1023) as f32 * (1.0 / 1023.0);
                let mut a = (v >> 30) as f32 * (1.0 / 3.0);
                if self.colorType() == kBGRA_1010102_SkColorType {
                    std::mem::swap(&mut r, &mut b);
                }
                if a != 0.0 && needsUnpremul {
                    r = SkTPin(r / a, 0.0, 1.0);
                    g = SkTPin(g / a, 0.0, 1.0);
                    b = SkTPin(b / a, 0.0, 1.0);
                }
                b *= 255.0;
                g *= 255.0;
                r *= 255.0;
                a *= 255.0;
                return SkColorSetARGB(a as u32, r as u32, g as u32, b as u32);
            }
            kBGRA_10101010_XR_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),326; @condition b"false\0".as_ptr(); false);
                return 0;
            }
            kRGBA_10x6_SkColorType => {
                let v = self.addr64_xy(x, y).read_unaligned();
                return SkColorSetARGB(
                    (((v >> 54) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 6) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 22) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                    (((v >> 38) & 1023) as f32 * (255.0 / 1023.0)) as u32,
                );
            }
            kR16G16B16A16_unorm_SkColorType => {
                let v = self.addr64_xy(x, y).read_unaligned();
                let mut r = (v & 65535) as f32 * (1.0 / 65535.0);
                let mut g = ((v >> 16) & 65535) as f32 * (1.0 / 65535.0);
                let mut b = ((v >> 32) & 65535) as f32 * (1.0 / 65535.0);
                let mut a = (v >> 48) as f32 * (1.0 / 65535.0);
                if a != 0.0 && needsUnpremul {
                    r *= 1.0 / a;
                    g *= 1.0 / a;
                    b *= 1.0 / a;
                }
                r *= 255.0;
                g *= 255.0;
                b *= 255.0;
                a *= 255.0;
                return SkColorSetARGB(a as u32, r as u32, g as u32, b as u32);
            }
            kRGB_F16F16F16x_SkColorType => {
                let addr = self
                    .fPixels
                    .cast::<u64>()
                    .add(y as usize * (self.fRowBytes >> 3) + x as usize);
                let mut p4 = super::SkVx::from_half(addr.cast::<[u16; 4]>().read_unaligned());
                p4[3] = 1.0;
                return Sk4f_toL32(swizzle_rb(p4));
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                let addr = self
                    .fPixels
                    .cast::<u64>()
                    .add(y as usize * (self.fRowBytes >> 3) + x as usize);
                let mut p4 = super::SkVx::from_half(addr.cast::<[u16; 4]>().read_unaligned());
                if p4[3] != 0.0 && needsUnpremul {
                    let inva = 1.0 / p4[3];
                    p4 = [p4[0] * inva, p4[1] * inva, p4[2] * inva, p4[3] * 1.0];
                }
                return Sk4f_toL32(swizzle_rb(p4));
            }
            kRGBA_F32_SkColorType => {
                let rgba = self
                    .fPixels
                    .cast::<f32>()
                    .add(4 * y as usize * (self.fRowBytes >> 4) + 4 * x as usize);
                let mut p4 = rgba.cast::<[f32; 4]>().read_unaligned();
                if p4[3] != 0.0 && needsUnpremul {
                    let inva = 1.0 / p4[3];
                    p4 = [p4[0] * inva, p4[1] * inva, p4[2] * inva, p4[3] * 1.0];
                }
                return Sk4f_toL32(swizzle_rb(p4));
            }
            kUnknown_SkColorType => {}
        }
        crate::SkDEBUGFAIL!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),388; b"\0".as_ptr());
        SkColorSetARGB(0, 0, 0, 0)
    }
}
// Source: src/core/SkPixmap.cpp:394-397
// Source: src/core/SkPixmap.cpp:399-403
// Source: src/core/SkPixmap.cpp:405-467
// Source: src/core/SkPixmap.cpp:469-489
// Source: src/core/SkPixmap.cpp:491-497
// Source: src/core/SkPixmap.cpp:499-506
// Source: src/core/SkPixmap.cpp:508-531
// Source: src/core/SkPixmap.cpp:533-562
// Source: src/core/SkPixmap.cpp:564-575
impl SkPixmap {
    /// Pixels must be initialized and readable at the requested coordinate.
    pub unsafe fn getColor4f(&self, x: i32, y: i32) -> super::SkColor::SkColor4f {
        use super::SkColor::{SkColor4f, SkColorSetRGB};
        use super::SkColorData::*;
        use super::SkHalf::SkHalfToFloat;
        use super::SkUnPreMultiply::SkUnPreMultiply;
        use crate::include::core::SkColorType::SkColorType::*;
        use crate::include::private::SkTPin::SkTPin;
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),395; @condition b"this->addr()\0".as_ptr(); !self.addr().is_null());
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),396; @condition b"(unsigned)x < (unsigned)this->width()\0".as_ptr(); (x as u32) < self.width() as u32);
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),397; @condition b"(unsigned)y < (unsigned)this->height()\0".as_ptr(); (y as u32) < self.height() as u32);
        let needsUnpremul = self.fInfo.alphaType() == SkAlphaType::kPremul_SkAlphaType;
        let toColor = |c| {
            if needsUnpremul {
                SkUnPreMultiply::PMColorToColor(c)
            } else {
                SkSwizzle_BGRA_to_PMColor(c)
            }
        };
        let make = |fR, fG, fB, fA| SkColor4f { fR, fG, fB, fA };
        match self.colorType() {
            kGray_8_SkColorType => {
                let v = *self.addr8_xy(x, y) as f32 / 255.0;
                return make(v, v, v, 1.0);
            }
            kR8_unorm_SkColorType => {
                return make(*self.addr8_xy(x, y) as f32 / 255.0, 0.0, 0.0, 1.0)
            }
            kAlpha_8_SkColorType => {
                return make(0.0, 0.0, 0.0, *self.addr8_xy(x, y) as f32 / 255.0)
            }
            kR16_unorm_SkColorType => {
                return make(
                    self.addr16_xy(x, y).read_unaligned() as f32 / 65535.0,
                    0.0,
                    0.0,
                    1.0,
                )
            }
            kA16_unorm_SkColorType => {
                return make(
                    0.0,
                    0.0,
                    0.0,
                    self.addr16_xy(x, y).read_unaligned() as f32 / 65535.0,
                )
            }
            kR16_float_SkColorType => {
                return make(
                    SkHalfToFloat(self.addr16_xy(x, y).read_unaligned()),
                    0.0,
                    0.0,
                    1.0,
                )
            }
            kA16_float_SkColorType => {
                return make(
                    0.0,
                    0.0,
                    0.0,
                    SkHalfToFloat(self.addr16_xy(x, y).read_unaligned()),
                )
            }
            kRGB_565_SkColorType => {
                return SkColor4f::FromColor(SkPixel16ToColor(
                    self.addr16_xy(x, y).read_unaligned() as u32,
                ))
            }
            kARGB_4444_SkColorType => {
                return SkColor4f::FromColor(toColor(SkPixel4444ToPixel32(
                    self.addr16_xy(x, y).read_unaligned() as u32,
                )))
            }
            kR8G8_unorm_SkColorType => {
                let v = self.addr16_xy(x, y).read_unaligned();
                return SkColor4f::FromColor(SkColorSetRGB(
                    v as u8 as u32,
                    (v >> 8) as u8 as u32,
                    0,
                ));
            }
            kR16G16_unorm_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return make(
                    (v & 65535) as f32 * (1.0 / 65535.0),
                    (v >> 16) as f32 * (1.0 / 65535.0),
                    0.0,
                    1.0,
                );
            }
            kR16G16_float_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return make(
                    SkHalfToFloat(v as u16),
                    SkHalfToFloat((v >> 16) as u16),
                    0.0,
                    1.0,
                );
            }
            kRGB_888x_SkColorType => {
                return SkColor4f::FromColor(SkSwizzle_RB(
                    self.addr32_xy(x, y).read_unaligned() | 0xff000000,
                ))
            }
            kBGRA_8888_SkColorType => {
                return SkColor4f::FromColor(toColor(SkSwizzle_BGRA_to_PMColor(
                    self.addr32_xy(x, y).read_unaligned(),
                )))
            }
            kRGBA_8888_SkColorType => {
                return SkColor4f::FromColor(toColor(SkSwizzle_RGBA_to_PMColor(
                    self.addr32_xy(x, y).read_unaligned(),
                )))
            }
            kSRGBA_8888_SkColorType => {
                let srgb_to_linear = |v: f32| {
                    if v <= 0.04045 {
                        v * (1.0 / 12.92)
                    } else {
                        (v * (1.0 / 1.055) + (0.055 / 1.055)).powf(2.4)
                    }
                };
                let v = self.addr32_xy(x, y).read_unaligned();
                let mut r = srgb_to_linear((v & 255) as f32 * (1.0 / 255.0));
                let mut g = srgb_to_linear(((v >> 8) & 255) as f32 * (1.0 / 255.0));
                let mut b = srgb_to_linear(((v >> 16) & 255) as f32 * (1.0 / 255.0));
                let a = (v >> 24) as f32 * (1.0 / 255.0);
                if a != 0.0 && needsUnpremul {
                    r = SkTPin(r / a, 0.0, 1.0);
                    g = SkTPin(g / a, 0.0, 1.0);
                    b = SkTPin(b / a, 0.0, 1.0);
                }
                return make(r, g, b, a);
            }
            kBGR_101010x_XR_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),485; @condition b"false\0".as_ptr(); false);
                return make(0.0, 0.0, 0.0, 0.0);
            }
            kRGB_101010x_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return make(
                    (v & 1023) as f32 / 1023.0,
                    ((v >> 10) & 1023) as f32 / 1023.0,
                    ((v >> 20) & 1023) as f32 / 1023.0,
                    1.0,
                );
            }
            kBGR_101010x_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                return make(
                    ((v >> 20) & 1023) as f32 / 1023.0,
                    ((v >> 10) & 1023) as f32 / 1023.0,
                    (v & 1023) as f32 / 1023.0,
                    1.0,
                );
            }
            kRGBA_1010102_SkColorType | kBGRA_1010102_SkColorType => {
                let v = self.addr32_xy(x, y).read_unaligned();
                let mut b = (v & 1023) as f32 * (1.0 / 1023.0);
                let mut g = ((v >> 10) & 1023) as f32 * (1.0 / 1023.0);
                let mut r = ((v >> 20) & 1023) as f32 * (1.0 / 1023.0);
                let a = (v >> 30) as f32 * (1.0 / 3.0);
                if a != 0.0 && needsUnpremul {
                    r = SkTPin(r / a, 0.0, 1.0);
                    g = SkTPin(g / a, 0.0, 1.0);
                    b = SkTPin(b / a, 0.0, 1.0);
                }
                return make(r, g, b, a);
            }
            kBGRA_10101010_XR_SkColorType => {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),520; @condition b"false\0".as_ptr(); false);
                return make(0.0, 0.0, 0.0, 0.0);
            }
            kRGBA_10x6_SkColorType => {
                let v = self.addr64_xy(x, y).read_unaligned();
                return make(
                    ((v >> 6) & 1023) as f32 * (1.0 / 1023.0),
                    ((v >> 22) & 1023) as f32 * (1.0 / 1023.0),
                    ((v >> 38) & 1023) as f32 * (1.0 / 1023.0),
                    ((v >> 54) & 1023) as f32 * (1.0 / 1023.0),
                );
            }
            kR16G16B16A16_unorm_SkColorType => {
                let v = self.addr64_xy(x, y).read_unaligned();
                let mut r = (v & 65535) as f32 * (1.0 / 65535.0);
                let mut g = ((v >> 16) & 65535) as f32 * (1.0 / 65535.0);
                let mut b = ((v >> 32) & 65535) as f32 * (1.0 / 65535.0);
                let a = (v >> 48) as f32 * (1.0 / 65535.0);
                if a != 0.0 && needsUnpremul {
                    r *= 1.0 / a;
                    g *= 1.0 / a;
                    b *= 1.0 / a;
                }
                return make(r, g, b, a);
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                let addr = self
                    .fPixels
                    .cast::<u64>()
                    .add(y as usize * (self.fRowBytes >> 3) + x as usize);
                let mut p4 = super::SkVx::from_half(addr.cast::<[u16; 4]>().read_unaligned());
                if p4[3] != 0.0 && needsUnpremul {
                    let inva = 1.0 / p4[3];
                    p4 = [p4[0] * inva, p4[1] * inva, p4[2] * inva, p4[3] * 1.0];
                }
                return make(p4[0], p4[1], p4[2], p4[3]);
            }
            kRGB_F16F16F16x_SkColorType => {
                let addr = self
                    .fPixels
                    .cast::<u64>()
                    .add(y as usize * (self.fRowBytes >> 3) + x as usize);
                let mut p4 = super::SkVx::from_half(addr.cast::<[u16; 4]>().read_unaligned());
                p4[3] = 1.0;
                return make(p4[0], p4[1], p4[2], p4[3]);
            }
            kRGBA_F32_SkColorType => {
                let rgba = self
                    .fPixels
                    .cast::<f32>()
                    .add(4 * y as usize * (self.fRowBytes >> 4) + 4 * x as usize);
                let mut p4 = rgba.cast::<[f32; 4]>().read_unaligned();
                if p4[3] != 0.0 && needsUnpremul {
                    let inva = 1.0 / p4[3];
                    p4 = [p4[0] * inva, p4[1] * inva, p4[2] * inva, p4[3] * 1.0];
                }
                return make(p4[0], p4[1], p4[2], p4[3]);
            }
            kUnknown_SkColorType => {}
        }
        crate::SkDEBUGFAIL!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),573; b"\0".as_ptr());
        make(0.0, 0.0, 0.0, 0.0)
    }
}
// Source: src/core/SkPixmap.cpp:577-579
// Source: src/core/SkPixmap.cpp:581-743
impl SkPixmap {
    /// Caller guarantees that the referenced pixel storage is readable for this descriptor.
    #[allow(unused_parens, unused_variables, unused_imports)]
    pub unsafe fn computeIsOpaque(&self) -> bool {
        use super::SkColor::SkPMColor;
        use super::SkColorData::{SkGetPackedA4444, SkPMColor16};
        use super::SkColorPriv::SkGetPackedA32;
        use super::SkHalf::{SK_Half1, SkHalf};
        let height = self.height();
        let width = self.width();
        match self.colorType() {
            kAlpha_8_SkColorType => {
                let mut a: u32 = 0xFF;
                for y in 0..height {
                    let row = self.addr8_xy(0, y);
                    for x in 0..width {
                        a &= *row.add((x) as usize) as u32;
                    }
                    if (0xFF != a) {
                        return false;
                    }
                }
                return true;
            }
            kA16_unorm_SkColorType => {
                let mut a: u32 = 0xFFFF;
                for y in 0..height {
                    let row = self.addr16_xy(0, y);
                    for x in 0..width {
                        a &= *row.add((x) as usize) as u32;
                    }
                    if (0xFFFF != a) {
                        return false;
                    }
                }
                return true;
            }
            kA16_float_SkColorType => {
                for y in 0..height {
                    let row = self.addr16_xy(0, y);
                    for x in 0..width {
                        if (*row.add((x) as usize) < SK_Half1) {
                            return false;
                        }
                    }
                }
                return true;
            }
            kRGB_565_SkColorType
            | kGray_8_SkColorType
            | kR8G8_unorm_SkColorType
            | kR16_unorm_SkColorType
            | kR16_float_SkColorType
            | kR16G16_unorm_SkColorType
            | kR16G16_float_SkColorType
            | kRGB_888x_SkColorType
            | kRGB_101010x_SkColorType
            | kBGR_101010x_SkColorType
            | kRGB_F16F16F16x_SkColorType
            | kBGR_101010x_XR_SkColorType
            | kR8_unorm_SkColorType => {
                return true;
            }
            kARGB_4444_SkColorType => {
                let mut c: u32 = 0xFFFF;
                for y in 0..height {
                    let row = self.addr16_xy(0, y);
                    for x in 0..width {
                        c &= *row.add((x) as usize) as u32;
                    }
                    if (0xF != SkGetPackedA4444(c)) {
                        return false;
                    }
                }
                return true;
            }
            kBGRA_8888_SkColorType | kRGBA_8888_SkColorType | kSRGBA_8888_SkColorType => {
                let mut c: SkPMColor = !0;
                for y in 0..height {
                    let row = self.addr32_xy(0, y);
                    for x in 0..width {
                        c &= *row.add((x) as usize) as u32;
                    }
                    if (0xFF != SkGetPackedA32(c)) {
                        return false;
                    }
                }
                return true;
            }
            kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                let mut row = self.addr().cast::<SkHalf>();
                for y in 0..height {
                    for x in 0..width {
                        if (*row.add((4 * x + 3) as usize) < SK_Half1) {
                            return false;
                        }
                    }
                    row = row.add(self.rowBytes() >> 1);
                }
                return true;
            }
            kRGBA_F32_SkColorType => {
                let mut row = self.addr().cast::<f32>();
                for y in 0..height {
                    for x in 0..width {
                        if (*row.add((4 * x + 3) as usize) < 1.0) {
                            return false;
                        }
                    }
                    row = row.add(self.rowBytes() >> 2);
                }
                return true;
            }
            kRGBA_1010102_SkColorType | kBGRA_1010102_SkColorType => {
                let mut c: u32 = !0;
                for y in 0..height {
                    let row = self.addr32_xy(0, y);
                    for x in 0..width {
                        c &= *row.add((x) as usize) as u32;
                    }
                    if (0b11 != c >> 30) {
                        return false;
                    }
                }
                return true;
            }
            kBGRA_10101010_XR_SkColorType => {
                const kOne: u64 = 510 + 384;
                for y in 0..height {
                    let row = self.addr64_xy(0, y);
                    for x in 0..width {
                        if ((*row.add((x) as usize) >> 54) < kOne) {
                            return false;
                        }
                    }
                }
                return true;
            }
            kRGBA_10x6_SkColorType => {
                let mut acc: u16 = 0xFFC0;
                for y in 0..height {
                    let row = self.addr64_xy(0, y);
                    for x in 0..width {
                        acc &= (*row.add((x) as usize) >> 48) as u16;
                    }
                    if (0xFFC0 != acc) {
                        return false;
                    }
                }
                return true;
            }
            kR16G16B16A16_unorm_SkColorType => {
                let mut acc: u16 = 0xFFFF;
                for y in 0..height {
                    let row = self.addr64_xy(0, y);
                    for x in 0..width {
                        acc &= (*row.add((x) as usize) >> 48) as u16;
                    }
                    if (0xFFFF != acc) {
                        return false;
                    }
                }
                return true;
            }
            kUnknown_SkColorType => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),739; b"\0".as_ptr());
            }
        }
        return false;
    }
}

// Source: src/core/SkPixmap.cpp:745-747
impl SkPixmap {
    /// Safety: subset destination has live exclusively writable pixels of the declared layout.
    pub unsafe fn eraseColor(
        &self,
        color: super::SkColor::SkColor,
        subset: &SkIRect,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.erase(&super::SkColor::SkColor4f::FromColor(color), Some(subset)) }
    }
    // Source: src/core/SkPixmap.cpp:749-810
    /// Safety: every clipped destination pixel is writable, correctly aligned and
    /// exclusively leased for the complete conversion/memset operation.
    pub unsafe fn erase(
        &self,
        color: &super::SkColor::SkColor4f,
        subset: Option<&SkIRect>,
    ) -> Result<bool, crate::RasterError> {
        if self.colorType() == kUnknown_SkColorType {
            return Ok(false);
        }
        let mut clip = self.bounds();
        if let Some(subset) = subset {
            if !clip.intersect(subset) {
                return Ok(false);
            }
        }
        let color = color.premul();
        let dst = SkImageInfo::MakeWithColorSpace(
            1,
            1,
            self.colorType(),
            self.alphaType(),
            self.refColorSpace(),
        );
        let src = SkImageInfo::MakeWithColorSpace(
            1,
            1,
            kRGBA_F32_SkColorType,
            crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
            None,
        );
        let mut pixel = [0u64; 2];
        crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),768; @condition b"(size_t)dst.bytesPerPixel() <= sizeof(dstPixel)\0".as_ptr(); dst.bytesPerPixel() as usize<=core::mem::size_of_val(&pixel));
        if !unsafe {
            super::SkConvertPixels::SkConvertPixels(
                &dst,
                pixel.as_mut_ptr().cast(),
                core::mem::size_of_val(&pixel),
                &src,
                core::ptr::from_ref(&color).cast(),
                core::mem::size_of_val(&color),
            )
        }? {
            return Ok(false);
        }
        if self.colorType() == kRGBA_F32_SkColorType {
            let value = unsafe { pixel.as_ptr().cast::<super::SkColor::SkColor4f>().read() };
            for y in clip.fTop..clip.fBottom {
                unsafe {
                    crate::src::opts::SkMemset_opts::memsetT_raw(
                        self.writable_addr_xy(clip.fLeft, y).cast(),
                        value,
                        clip.width() as usize,
                    );
                }
            }
        } else {
            type MemSet = unsafe fn(*mut u8, u64, i32);
            unsafe fn byte(dst: *mut u8, value: u64, count: i32) {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),785; @condition b"c == (uint8_t)c\0".as_ptr(); (value)==(value as u8 as u64));
                unsafe {
                    crate::src::opts::SkMemset_opts::memsetT_raw(dst, value as u8, count as usize);
                }
            }
            unsafe fn word(dst: *mut u8, value: u64, count: i32) {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),789; @condition b"c == (uint16_t)c\0".as_ptr(); (value)==(value as u16 as u64));
                unsafe {
                    crate::src::opts::SkMemset_opts::memsetT_raw(
                        dst.cast(),
                        value as u16,
                        count as usize,
                    );
                }
            }
            unsafe fn dword(dst: *mut u8, value: u64, count: i32) {
                crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),793; @condition b"c == (uint32_t)c\0".as_ptr(); (value)==(value as u32 as u64));
                unsafe {
                    crate::src::opts::SkMemset_opts::memsetT_raw(
                        dst.cast(),
                        value as u32,
                        count as usize,
                    );
                }
            }
            unsafe fn qword(dst: *mut u8, value: u64, count: i32) {
                unsafe {
                    crate::src::opts::SkMemset_opts::memsetT_raw(dst.cast(), value, count as usize);
                }
            }
            let procs: [MemSet; 4] = [byte, word, dword, qword];
            let shift = super::SkImageInfoPriv::SkColorTypeShiftPerPixel(self.colorType()) as usize;
            crate::SkASSERT!(@source b"src/core/SkPixmap.cpp\0".as_ptr(),802; @condition b"shift < std::size(procs)\0".as_ptr(); shift<procs.len());
            let proc = procs[shift];
            for y in clip.fTop..clip.fBottom {
                unsafe {
                    proc(self.writable_addr_xy(clip.fLeft, y), pixel[0], clip.width());
                }
            }
        }
        Ok(true)
    }
}

// Source: include/core/SkPixmap.h:731-733
// Implicit native move construction/assignment copies pointer and stride, then
// moves ImageInfo. Source pixels/stride/dimensions/encoding scalars stay intact;
// only its color-space smart pointer becomes empty.
impl SkPixmap {
    pub(crate) fn move_from(source: &mut Self) -> Self {
        let pixels = source.fPixels;
        let row_bytes = source.fRowBytes;
        let info = SkImageInfo::move_from(&mut source.fInfo);
        Self {
            fPixels: pixels,
            fRowBytes: row_bytes,
            fInfo: info,
        }
    }
    pub(crate) fn assign_move(&mut self, source: &mut Self) {
        self.fPixels = source.fPixels;
        self.fRowBytes = source.fRowBytes;
        self.fInfo.assign_move(&mut source.fInfo);
    }
}
