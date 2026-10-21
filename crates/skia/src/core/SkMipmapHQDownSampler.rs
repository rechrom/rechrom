// Copyright 2023 Google LLC. BSD-style license.
use super::SkColorData::SK_G16_MASK_IN_PLACE;
// Internal Rust traits replace C++ template-associated types and vector operations.
// They preserve each scalar lane's integer width and wrapping arithmetic.
pub(crate) trait Expanded: Copy {
    fn add(self, rhs: Self) -> Self;
    fn shr(self, bits: u32) -> Self;
    fn shl(self, bits: u32) -> Self;
}
macro_rules! expanded_scalar {
    ($t:ty) => {
        impl Expanded for $t {
            fn add(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }
            fn shr(self, b: u32) -> Self {
                self >> b
            }
            fn shl(self, b: u32) -> Self {
                self << b
            }
        }
    };
}
expanded_scalar!(u32);
expanded_scalar!(u64);
macro_rules! expanded_vector {
    ($t:ty) => {
        impl Expanded for [$t; 4] {
            fn add(self, rhs: Self) -> Self {
                std::array::from_fn(|i| self[i].wrapping_add(rhs[i]))
            }
            fn shr(self, b: u32) -> Self {
                self.map(|v| v >> b)
            }
            fn shl(self, b: u32) -> Self {
                self.map(|v| v << b)
            }
        }
    };
}
expanded_vector!(u16);
expanded_vector!(u32);
pub(crate) trait ColorTypeFilter {
    type Type: Copy;
    type Expanded: Expanded;
    fn Expand(x: Self::Type) -> Self::Expanded;
    fn Compact(x: Self::Expanded) -> Self::Type;
}
// Source: src/core/SkMipmapHQDownSampler.cpp:19-29
pub(crate) struct ColorTypeFilter_8888;
impl ColorTypeFilter for ColorTypeFilter_8888 {
    type Type = u32;
    type Expanded = [u16; 4];
    fn Expand(x: u32) -> Self::Expanded {
        x.to_ne_bytes().map(|v| v as u16)
    }
    fn Compact(x: Self::Expanded) -> u32 {
        u32::from_ne_bytes(x.map(|v| v as u8))
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:31-39
pub(crate) struct ColorTypeFilter_565;
impl ColorTypeFilter for ColorTypeFilter_565 {
    type Type = u16;
    type Expanded = u32;
    fn Expand(x: u16) -> u32 {
        let x = x as u32;
        (x & !SK_G16_MASK_IN_PLACE) | ((x & SK_G16_MASK_IN_PLACE) << 16)
    }
    fn Compact(x: u32) -> u16 {
        (((x & !SK_G16_MASK_IN_PLACE) & 0xffff) | ((x >> 16) & SK_G16_MASK_IN_PLACE)) as u16
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:41-49
pub(crate) struct ColorTypeFilter_4444;
impl ColorTypeFilter for ColorTypeFilter_4444 {
    type Type = u16;
    type Expanded = u32;
    fn Expand(x: u16) -> u32 {
        let x = x as u32;
        (x & 0xf0f) | ((x & !0xf0f) << 12)
    }
    fn Compact(x: u32) -> u16 {
        ((x & 0xf0f) | ((x >> 12) & !0xf0f)) as u16
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:51-59
pub(crate) struct ColorTypeFilter_8;
impl ColorTypeFilter for ColorTypeFilter_8 {
    type Type = u8;
    type Expanded = u32;
    fn Expand(x: u8) -> u32 {
        x as u32
    }
    fn Compact(x: u32) -> u8 {
        x as u8
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:61-72
// Source: src/core/SkMipmapHQDownSampler.cpp:74-84
pub(crate) struct ColorTypeFilter_F16;
impl ColorTypeFilter for ColorTypeFilter_F16 {
    type Type = u16;
    type Expanded = [f32; 4];
    fn Expand(x: u16) -> Self::Expanded {
        super::SkVx::from_half(unpack_half_words(x as u64))
    }
    fn Compact(x: Self::Expanded) -> u16 {
        (pack_half_words(super::SkVx::to_half(x)) & 65535) as u16
    }
}
pub(crate) struct ColorTypeFilter_RGBA_F16;
impl ColorTypeFilter for ColorTypeFilter_RGBA_F16 {
    type Type = u64;
    type Expanded = [f32; 4];
    fn Expand(x: u64) -> Self::Expanded {
        super::SkVx::from_half(unpack_half_words(x))
    }
    fn Compact(x: Self::Expanded) -> u64 {
        pack_half_words(super::SkVx::to_half(x))
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:86-94
pub(crate) struct ColorTypeFilter_88;
impl ColorTypeFilter for ColorTypeFilter_88 {
    type Type = u16;
    type Expanded = u32;
    fn Expand(x: u16) -> u32 {
        let x = x as u32;
        (x & 255) | ((x & !255) << 8)
    }
    fn Compact(x: u32) -> u16 {
        ((x & 255) | ((x >> 8) & !255)) as u16
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:96-104
pub(crate) struct ColorTypeFilter_1616;
impl ColorTypeFilter for ColorTypeFilter_1616 {
    type Type = u32;
    type Expanded = u64;
    fn Expand(x: u32) -> u64 {
        // The original shift is performed in uint32_t before widening.
        ((x & 65535) | ((x & !65535).wrapping_shl(16))) as u64
    }
    fn Compact(x: u64) -> u32 {
        // Preserve the source's uint16_t return conversion before Type assignment.
        ((x & 65535) | ((x >> 16) & !65535)) as u16 as u32
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:106-117
pub(crate) struct ColorTypeFilter_F16F16;
impl ColorTypeFilter for ColorTypeFilter_F16F16 {
    type Type = u32;
    type Expanded = [f32; 4];
    fn Expand(x: u32) -> Self::Expanded {
        super::SkVx::from_half(unpack_half_words(x as u64))
    }
    fn Compact(x: Self::Expanded) -> u32 {
        (pack_half_words(super::SkVx::to_half(x)) & 0xffffffff) as u32
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:119-129
pub(crate) struct ColorTypeFilter_16161616;
impl ColorTypeFilter for ColorTypeFilter_16161616 {
    type Type = u64;
    type Expanded = [u32; 4];
    fn Expand(x: u64) -> Self::Expanded {
        let b = x.to_ne_bytes();
        std::array::from_fn(|i| u16::from_ne_bytes([b[2 * i], b[2 * i + 1]]) as u32)
    }
    fn Compact(x: Self::Expanded) -> u64 {
        let mut b = [0u8; 8];
        for i in 0..4 {
            b[2 * i..2 * i + 2].copy_from_slice(&(x[i] as u16).to_ne_bytes());
        }
        u64::from_ne_bytes(b)
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:131-139
pub(crate) struct ColorTypeFilter_16;
impl ColorTypeFilter for ColorTypeFilter_16 {
    type Type = u16;
    type Expanded = u32;
    fn Expand(x: u16) -> u32 {
        x as u32
    }
    fn Compact(x: u32) -> u16 {
        x as u16
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:141-155
pub(crate) struct ColorTypeFilter_1010102;
impl ColorTypeFilter for ColorTypeFilter_1010102 {
    type Type = u32;
    type Expanded = u64;
    fn Expand(x: u32) -> u64 {
        let x = x as u64;
        (x & 0x3ff)
            | (((x >> 10) & 0x3ff) << 20)
            | (((x >> 20) & 0x3ff) << 40)
            | (((x >> 30) & 3) << 60)
    }
    fn Compact(x: u64) -> u32 {
        ((x & 0x3ff)
            | (((x >> 20) & 0x3ff) << 10)
            | (((x >> 40) & 0x3ff) << 20)
            | (((x >> 60) & 3) << 30)) as u32
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:157-159
fn add_121<T: Expanded>(a: T, b: T, c: T) -> T {
    a.add(b).add(b).add(c)
}
// Source: src/core/SkMipmapHQDownSampler.cpp:161-163
fn shift_right<T: Expanded>(x: T, bits: u32) -> T {
    x.shr(bits)
}
// Source: src/core/SkMipmapHQDownSampler.cpp:165-167
fn shift_right_float(x: [f32; 4], bits: u32) -> [f32; 4] {
    x.map(|v| v * (1.0 / (1 << bits) as f32))
}
// Source: src/core/SkMipmapHQDownSampler.cpp:169-171
fn shift_left<T: Expanded>(x: T, bits: u32) -> T {
    x.shl(bits)
}
// Source: src/core/SkMipmapHQDownSampler.cpp:173-175
fn shift_left_float(x: [f32; 4], bits: u32) -> [f32; 4] {
    x.map(|v| v * (1 << bits) as f32)
}
// Portable floating vector operation dispatch for the native template overloads.
impl Expanded for [f32; 4] {
    fn add(self, rhs: Self) -> Self {
        std::array::from_fn(|i| self[i] + rhs[i])
    }
    fn shr(self, bits: u32) -> Self {
        shift_right_float(self, bits)
    }
    fn shl(self, bits: u32) -> Self {
        shift_left_float(self, bits)
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:190-194
// Source: src/core/SkMipmapHQDownSampler.cpp:196-198
// Source: src/core/SkMipmapHQDownSampler.cpp:200-205
pub(crate) unsafe fn downsample_1_2<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),191; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    for i in 0..count as usize {
        let mut c00 = F::Expand(*p0.add(0));
        let mut c10 = F::Expand(*p1.add(0));
        let mut c = c00.add(c10);
        *d.add(i) = F::Compact(shift_right(c, 1));
        p0 = p0.add(2);
        p1 = p1.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:207-212
// Source: src/core/SkMipmapHQDownSampler.cpp:214-217
// Source: src/core/SkMipmapHQDownSampler.cpp:219-225
pub(crate) unsafe fn downsample_1_3<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),208; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let mut p2 = p1.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    for i in 0..count as usize {
        let mut c00 = F::Expand(*p0.add(0));
        let mut c10 = F::Expand(*p1.add(0));
        let mut c20 = F::Expand(*p2.add(0));
        let mut c = add_121(c00, c10, c20);
        *d.add(i) = F::Compact(shift_right(c, 2));
        p0 = p0.add(2);
        p1 = p1.add(2);
        p2 = p2.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:227-230
// Source: src/core/SkMipmapHQDownSampler.cpp:232-234
// Source: src/core/SkMipmapHQDownSampler.cpp:236-240
pub(crate) unsafe fn downsample_2_1<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),228; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let d = dst;
    for i in 0..count as usize {
        let mut c00 = F::Expand(*p0.add(0));
        let mut c01 = F::Expand(*p0.add(1));
        let mut c = c00.add(c01);
        *d.add(i) = F::Compact(shift_right(c, 1));
        p0 = p0.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:242-246
// Source: src/core/SkMipmapHQDownSampler.cpp:248-252
// Source: src/core/SkMipmapHQDownSampler.cpp:254-259
pub(crate) unsafe fn downsample_2_2<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),243; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    for i in 0..count as usize {
        let mut c00 = F::Expand(*p0.add(0));
        let mut c01 = F::Expand(*p0.add(1));
        let mut c10 = F::Expand(*p1.add(0));
        let mut c11 = F::Expand(*p1.add(1));
        let mut c = c00.add(c10).add(c01).add(c11);
        *d.add(i) = F::Compact(shift_right(c, 2));
        p0 = p0.add(2);
        p1 = p1.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:261-266
// Source: src/core/SkMipmapHQDownSampler.cpp:268-274
// Source: src/core/SkMipmapHQDownSampler.cpp:276-282
pub(crate) unsafe fn downsample_2_3<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),262; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let mut p2 = p1.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    for i in 0..count as usize {
        let mut c00 = F::Expand(*p0.add(0));
        let mut c01 = F::Expand(*p0.add(1));
        let mut c10 = F::Expand(*p1.add(0));
        let mut c11 = F::Expand(*p1.add(1));
        let mut c20 = F::Expand(*p2.add(0));
        let mut c21 = F::Expand(*p2.add(1));
        let mut c = add_121(c00, c10, c20).add(add_121(c01, c11, c21));
        *d.add(i) = F::Compact(shift_right(c, 3));
        p0 = p0.add(2);
        p1 = p1.add(2);
        p2 = p2.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:284-287
// Source: src/core/SkMipmapHQDownSampler.cpp:289-293
// Source: src/core/SkMipmapHQDownSampler.cpp:295-299
pub(crate) unsafe fn downsample_3_1<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),285; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let d = dst;
    let mut c02 = F::Expand(*p0.add(0));
    for i in 0..count as usize {
        let mut c00 = c02;
        let mut c01 = F::Expand(*p0.add(1));
        c02 = F::Expand(*p0.add(2));
        let mut c = add_121(c00, c01, c02);
        *d.add(i) = F::Compact(shift_right(c, 2));
        p0 = p0.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:301-305
// Source: src/core/SkMipmapHQDownSampler.cpp:315-319
// Source: src/core/SkMipmapHQDownSampler.cpp:321-323
// Source: src/core/SkMipmapHQDownSampler.cpp:325-327
// Source: src/core/SkMipmapHQDownSampler.cpp:329-334
pub(crate) unsafe fn downsample_3_2<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),302; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    let mut c0 = F::Expand(*p0.add(0));
    let mut c1 = F::Expand(*p1.add(0));
    let mut c = c0.add(c1);
    for i in 0..count as usize {
        let mut a = c;
        let mut b0 = F::Expand(*p0.add(1));
        let mut b1 = F::Expand(*p1.add(1));
        let mut b = b0.add(b0).add(b1).add(b1);
        c0 = F::Expand(*p0.add(2));
        c1 = F::Expand(*p1.add(2));
        c = c0.add(c1);
        let mut sum = a.add(b).add(c);
        *d.add(i) = F::Compact(shift_right(sum, 3));
        p0 = p0.add(2);
        p1 = p1.add(2);
    }
}
// Source: src/core/SkMipmapHQDownSampler.cpp:336-341
// Source: src/core/SkMipmapHQDownSampler.cpp:352-357
// Source: src/core/SkMipmapHQDownSampler.cpp:359-362
// Source: src/core/SkMipmapHQDownSampler.cpp:364-367
// Source: src/core/SkMipmapHQDownSampler.cpp:369-375
pub(crate) unsafe fn downsample_3_3<F: ColorTypeFilter>(
    dst: *mut F::Type,
    src: *const F::Type,
    srcRB: usize,
    count: i32,
) {
    crate::SkASSERT!(@source b"src/core/SkMipmapHQDownSampler.cpp\0".as_ptr(),337; @condition b"count > 0\0".as_ptr(); count>0);
    let mut p0 = src;
    let mut p1 = p0.cast::<u8>().add(srcRB).cast::<F::Type>();
    let mut p2 = p1.cast::<u8>().add(srcRB).cast::<F::Type>();
    let d = dst;
    let mut c0 = F::Expand(*p0.add(0));
    let mut c1 = F::Expand(*p1.add(0));
    let mut c2 = F::Expand(*p2.add(0));
    let mut c = add_121(c0, c1, c2);
    for i in 0..count as usize {
        let mut a = c;
        let mut b0 = F::Expand(*p0.add(1));
        let mut b1 = F::Expand(*p1.add(1));
        let mut b2 = F::Expand(*p2.add(1));
        let mut b = shift_left(add_121(b0, b1, b2), 1);
        c0 = F::Expand(*p0.add(2));
        c1 = F::Expand(*p1.add(2));
        c2 = F::Expand(*p2.add(2));
        c = add_121(c0, c1, c2);
        let mut sum = a.add(b).add(c);
        *d.add(i) = F::Compact(shift_right(sum, 4));
        p0 = p0.add(2);
        p1 = p1.add(2);
        p2 = p2.add(2);
    }
}

// Native vector load/store language glue; no independent source-line accounting.
fn unpack_half_words(x: u64) -> [u16; 4] {
    let b = x.to_ne_bytes();
    std::array::from_fn(|i| u16::from_ne_bytes([b[2 * i], b[2 * i + 1]]))
}
fn pack_half_words(x: [u16; 4]) -> u64 {
    let mut b = [0u8; 8];
    for i in 0..4 {
        b[2 * i..2 * i + 2].copy_from_slice(&x[i].to_ne_bytes());
    }
    u64::from_ne_bytes(b)
}

use super::SkMipmap::SkMipmapDownSampler;
use super::SkPixmap::SkPixmap;
use crate::include::core::SkColorType::*;
// Source: src/core/SkMipmapHQDownSampler.cpp:378-378
// Source: src/core/SkMipmapHQDownSampler.cpp:380-388
// Source: src/core/SkMipmapHQDownSampler.cpp:390-391
type FilterProc = unsafe fn(*mut u8, *const u8, usize, i32);
struct HQDownSampler {
    proc_1_2: Option<FilterProc>,
    proc_1_3: Option<FilterProc>,
    proc_2_1: Option<FilterProc>,
    proc_2_2: Option<FilterProc>,
    proc_2_3: Option<FilterProc>,
    proc_3_1: Option<FilterProc>,
    proc_3_2: Option<FilterProc>,
    proc_3_3: Option<FilterProc>,
}
// Source: src/core/SkMipmapHQDownSampler.cpp:393-395
// Source: src/core/SkMipmapHQDownSampler.cpp:397-426
// Source: src/core/SkMipmapHQDownSampler.cpp:428-430
// Source: src/core/SkMipmapHQDownSampler.cpp:432-437
impl SkMipmapDownSampler for HQDownSampler {
    unsafe fn buildLevel(&self, dst: &SkPixmap, src: &SkPixmap) -> Result<(), crate::RasterError> {
        let width = src.width();
        let height = src.height();
        let proc;
        if height & 1 != 0 {
            if height == 1 {
                if width & 1 != 0 {
                    proc = self.proc_3_1;
                } else {
                    proc = self.proc_2_1;
                }
            } else {
                if width & 1 != 0 {
                    if width == 1 {
                        proc = self.proc_1_3;
                    } else {
                        proc = self.proc_3_3;
                    }
                } else {
                    proc = self.proc_2_3;
                }
            }
        } else {
            if width & 1 != 0 {
                if width == 1 {
                    proc = self.proc_1_2;
                } else {
                    proc = self.proc_3_2;
                }
            } else {
                proc = self.proc_2_2;
            }
        }
        let proc = proc.expect("native HQ factory must install every kernel");
        let mut srcBasePtr = src.addr();
        let srcRB = src.rowBytes();
        let mut dstBasePtr = dst.writable_addr();
        for _y in 0..dst.height() {
            proc(dstBasePtr, srcBasePtr, srcRB, dst.width());
            srcBasePtr = srcBasePtr.wrapping_add(srcRB * 2);
            dstBasePtr = dstBasePtr.wrapping_add(dst.rowBytes());
        }
        Ok(())
    }
}
unsafe fn erased_1_2<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_1_2::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_1_3<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_1_3::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_2_1<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_2_1::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_2_2<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_2_2::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_2_3<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_2_3::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_3_1<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_3_1::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_3_2<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_3_2::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
unsafe fn erased_3_3<F: ColorTypeFilter>(dst: *mut u8, src: *const u8, rb: usize, count: i32) {
    downsample_3_3::<F>(dst.cast::<F::Type>(), src.cast::<F::Type>(), rb, count)
}
// Source: src/core/SkMipmapHQDownSampler.cpp:441-449
// Source: src/core/SkMipmapHQDownSampler.cpp:451-578
// Source: src/core/SkMipmapHQDownSampler.cpp:580-589
// Source: src/core/SkMipmapHQDownSampler.cpp:591-593
// Source: src/core/SkMipmapHQDownSampler.cpp:595-605
// Rust keeps the static factory at its native implementation-file module until the SkMipmap owner is assembled.
pub fn MakeDownSampler(root: &SkPixmap) -> Option<Box<dyn SkMipmapDownSampler>> {
    let mut proc_1_2: Option<FilterProc> = None;
    let mut proc_1_3: Option<FilterProc> = None;
    let mut proc_2_1: Option<FilterProc> = None;
    let mut proc_2_2: Option<FilterProc> = None;
    let mut proc_2_3: Option<FilterProc> = None;
    let mut proc_3_1: Option<FilterProc> = None;
    let mut proc_3_2: Option<FilterProc> = None;
    let mut proc_3_3: Option<FilterProc> = None;
    match root.colorType() {
        kRGBA_8888_SkColorType | kBGRA_8888_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_8888>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_8888>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_8888>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_8888>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_8888>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_8888>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_8888>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_8888>);
        }
        kRGB_565_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_565>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_565>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_565>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_565>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_565>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_565>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_565>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_565>);
        }
        kARGB_4444_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_4444>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_4444>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_4444>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_4444>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_4444>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_4444>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_4444>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_4444>);
        }
        kAlpha_8_SkColorType | kGray_8_SkColorType | kR8_unorm_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_8>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_8>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_8>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_8>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_8>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_8>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_8>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_8>);
        }
        kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_RGBA_F16>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_RGBA_F16>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_RGBA_F16>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_RGBA_F16>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_RGBA_F16>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_RGBA_F16>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_RGBA_F16>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_RGBA_F16>);
        }
        kR8G8_unorm_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_88>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_88>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_88>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_88>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_88>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_88>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_88>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_88>);
        }
        kR16G16_unorm_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_1616>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_1616>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_1616>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_1616>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_1616>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_1616>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_1616>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_1616>);
        }
        kA16_unorm_SkColorType | kR16_unorm_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_16>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_16>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_16>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_16>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_16>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_16>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_16>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_16>);
        }
        kRGBA_1010102_SkColorType | kBGRA_1010102_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_1010102>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_1010102>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_1010102>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_1010102>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_1010102>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_1010102>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_1010102>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_1010102>);
        }
        kA16_float_SkColorType | kR16_float_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_F16>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_F16>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_F16>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_F16>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_F16>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_F16>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_F16>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_F16>);
        }
        kR16G16_float_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_F16F16>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_F16F16>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_F16F16>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_F16F16>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_F16F16>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_F16F16>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_F16F16>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_F16F16>);
        }
        kR16G16B16A16_unorm_SkColorType => {
            proc_1_2 = Some(erased_1_2::<ColorTypeFilter_16161616>);
            proc_1_3 = Some(erased_1_3::<ColorTypeFilter_16161616>);
            proc_2_1 = Some(erased_2_1::<ColorTypeFilter_16161616>);
            proc_2_2 = Some(erased_2_2::<ColorTypeFilter_16161616>);
            proc_2_3 = Some(erased_2_3::<ColorTypeFilter_16161616>);
            proc_3_1 = Some(erased_3_1::<ColorTypeFilter_16161616>);
            proc_3_2 = Some(erased_3_2::<ColorTypeFilter_16161616>);
            proc_3_3 = Some(erased_3_3::<ColorTypeFilter_16161616>);
        }
        kUnknown_SkColorType
        | kRGB_888x_SkColorType
        | kRGB_101010x_SkColorType
        | kBGR_101010x_SkColorType
        | kBGR_101010x_XR_SkColorType
        | kRGB_F16F16F16x_SkColorType
        | kBGRA_10101010_XR_SkColorType
        | kRGBA_10x6_SkColorType
        | kRGBA_F32_SkColorType => return None,
        kSRGBA_8888_SkColorType => return None,
    }
    let sampler = HQDownSampler {
        proc_1_2,
        proc_1_3,
        proc_2_1,
        proc_2_2,
        proc_2_3,
        proc_3_1,
        proc_3_2,
        proc_3_3,
    };
    Some(Box::new(sampler))
}
