// Copyright 2015 Google Inc. BSD-style license.
// Fixed arrays represent the native skvx lanes. Alignment and lane widths are
// retained; this is not a claim of SIMD instruction or C++ ABI identity.
// Source: src/core/Sk4px.h:18-18
impl Sk4px {
    pub fn new(v: [u8; 16]) -> Self {
        Self { fV: v }
    }
}
impl Sk4px {
    // Source: src/core/Sk4px.h:20-26
    pub fn DupPMColor(c: u32) -> Self {
        let b = c.to_ne_bytes();
        Self::new(core::array::from_fn(|i| b[i % 4]))
    }
    // Source: src/core/Sk4px.h:29-32
    pub fn alphas(&self) -> Self {
        Self::new(crate::src::core::SkVx::shuffle(
            self.fV,
            [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15],
        ))
    }
    // Source: src/core/Sk4px.h:33-33
    pub fn inv(&self) -> Self {
        Self::new(self.fV.map(|v| 255 - v))
    }
    // Source: src/core/Sk4px.h:36-40
    pub fn Load4(px: &[u32]) -> Self {
        let mut v = [0u8; 16];
        for i in 0..4 {
            v[4 * i..4 * i + 4].copy_from_slice(&px[i].to_ne_bytes());
        }
        Self::new(v)
    }
    // Source: src/core/Sk4px.h:41-45
    // The unused upper lanes are initialized for Rust; they are never stored
    // by the corresponding partial driver. This language adaptation is ledgered.
    pub fn Load2(px: &[u32]) -> Self {
        let mut v = [0u8; 16];
        for i in 0..2 {
            v[4 * i..4 * i + 4].copy_from_slice(&px[i].to_ne_bytes());
        }
        Self::new(v)
    }
    // Source: src/core/Sk4px.h:46-50
    pub fn Load1(px: &[u32]) -> Self {
        let mut v = [0u8; 16];
        v[..4].copy_from_slice(&px[0].to_ne_bytes());
        Self::new(v)
    }
    // Source: src/core/Sk4px.h:54-57
    pub fn Load4Alphas(a: &[u8]) -> Self {
        Self::new(core::array::from_fn(|i| a[i / 4]))
    }
    // Source: src/core/Sk4px.h:59-62
    pub fn Load2Alphas(a: &[u8]) -> Self {
        Self::new(core::array::from_fn(|i| if i < 8 { a[i / 4] } else { 0 }))
    }
    // Source: src/core/Sk4px.h:64-66
    pub fn store4(&self, px: &mut [u32]) {
        for i in 0..4 {
            px[i] = u32::from_ne_bytes(self.fV[4 * i..4 * i + 4].try_into().unwrap());
        }
    }
    pub fn store2(&self, px: &mut [u32]) {
        for i in 0..2 {
            px[i] = u32::from_ne_bytes(self.fV[4 * i..4 * i + 4].try_into().unwrap());
        }
    }
    pub fn store1(&self, px: &mut [u32]) {
        px[0] = u32::from_ne_bytes(self.fV[..4].try_into().unwrap());
    }
}
// Source: src/core/Sk4px.h:70-85
#[repr(C, align(32))]
#[derive(Clone, Copy)]
pub struct Wide {
    fV: [u16; 16],
}
impl Wide {
    pub fn new(v: [u16; 16]) -> Self {
        Self { fV: v }
    }
    pub fn div255(&self) -> Sk4px {
        Sk4px::new(crate::src::core::SkVx::div255(&self.fV))
    }
}
impl core::ops::Mul for Wide {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i].wrapping_mul(o.fV[i])))
    }
}
impl core::ops::Add for Wide {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i].wrapping_add(o.fV[i])))
    }
}
impl core::ops::Sub for Wide {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i].wrapping_sub(o.fV[i])))
    }
}
impl core::ops::Shr<u32> for Wide {
    type Output = Self;
    fn shr(self, bits: u32) -> Self {
        Self::new(self.fV.map(|v| v >> bits))
    }
}
impl core::ops::Shl<u32> for Wide {
    type Output = Self;
    fn shl(self, bits: u32) -> Self {
        Self::new(self.fV.map(|v| v << bits))
    }
}
impl Sk4px {
    // Source: src/core/Sk4px.h:88-90
    pub fn widen(&self) -> Wide {
        Wide::new(self.fV.map(u16::from))
    }
    pub fn mulWiden(&self, o: [u8; 16]) -> Wide {
        Wide::new(core::array::from_fn(|i| self.fV[i] as u16 * o[i] as u16))
    }
}
// Source: src/core/Sk4px.h:93-93
impl core::ops::Mul for Sk4px {
    type Output = Wide;
    fn mul(self, o: Self) -> Wide {
        self.mulWiden(o.fV)
    }
}
// Source: src/core/Sk4px.h:95-95
impl core::ops::Add for Sk4px {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i].wrapping_add(o.fV[i])))
    }
}
// Source: src/core/Sk4px.h:96-96
impl core::ops::Sub for Sk4px {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i].wrapping_sub(o.fV[i])))
    }
}
impl Sk4px {
    // Source: src/core/Sk4px.h:97-97
    // Rust comparison traits return bool, so the native lane-mask operator is
    // an explicit method rather than an unrelated scalar PartialOrd contract.
    pub fn less_than(&self, o: &Self) -> Self {
        Self::new(core::array::from_fn(|i| {
            if self.fV[i] < o.fV[i] {
                255
            } else {
                0
            }
        }))
    }
}
// Source: src/core/Sk4px.h:98-98
impl core::ops::BitAnd for Sk4px {
    type Output = Self;
    fn bitand(self, o: Self) -> Self {
        Self::new(core::array::from_fn(|i| self.fV[i] & o.fV[i]))
    }
}
impl Sk4px {
    // Source: src/core/Sk4px.h:99-101
    pub fn thenElse(&self, t: &Self, e: &Self) -> Self {
        Self::new(crate::src::core::SkVx::if_then_else(self.fV, t.fV, e.fV))
    }
    // Source: src/core/Sk4px.h:105-107
    pub fn approxMulDiv255(&self, o: &Self) -> Self {
        Self::new(crate::src::core::SkVx::approx_scale(self.fV, o.fV))
    }
    // Source: src/core/Sk4px.h:109-111
    pub fn saturatedAdd(&self, o: &Self) -> Self {
        Self::new(crate::src::core::SkVx::saturated_add(self.fV, o.fV))
    }
}
impl Sk4px {
    // Source: src/core/Sk4px.h:115-144
    /// Safety: all n pixels/alphas must be valid. Kernel returns values, retaining no storage borrow.
    pub unsafe fn MapSrc(
        mut n: i32,
        mut dst: *mut u32,
        mut src: *const u32,
        kernel: impl Fn(Self) -> Self,
    ) {
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 117; @condition b"dst\0".as_ptr(); !dst.is_null());
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 118; @condition b"src\0".as_ptr(); !src.is_null());
        while n > 0 {
            if n >= 8 {
                let dst0 = kernel(Self::Load4(core::slice::from_raw_parts(src.add(0), 4)));
                let dst4 = kernel(Self::Load4(core::slice::from_raw_parts(src.add(4), 4)));
                dst0.store4(core::slice::from_raw_parts_mut(dst, 4));
                dst4.store4(core::slice::from_raw_parts_mut(dst.add(4), 4));
                dst = dst.add(8);
                src = src.add(8);
                n -= 8;
                continue;
            }
            crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 130; @condition b"n <= 7\0".as_ptr(); n <= 7);
            if n >= 4 {
                kernel(Self::Load4(core::slice::from_raw_parts(src.add(0), 4)))
                    .store4(core::slice::from_raw_parts_mut(dst, 4));
                dst = dst.add(4);
                src = src.add(4);
                n -= 4;
            }
            if n >= 2 {
                kernel(Self::Load2(core::slice::from_raw_parts(src.add(0), 2)))
                    .store2(core::slice::from_raw_parts_mut(dst, 2));
                dst = dst.add(2);
                src = src.add(2);
                n -= 2;
            }
            if n >= 1 {
                kernel(Self::Load1(core::slice::from_raw_parts(src.add(0), 1)))
                    .store1(core::slice::from_raw_parts_mut(dst, 1));
            }
            break;
        }
    }
    // Source: src/core/Sk4px.h:147-175
    /// Safety: all n pixels/alphas must be valid. Kernel returns values, retaining no storage borrow.
    pub unsafe fn MapDstSrc(
        mut n: i32,
        mut dst: *mut u32,
        mut src: *const u32,
        kernel: impl Fn(Self, Self) -> Self,
    ) {
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 150; @condition b"dst\0".as_ptr(); !dst.is_null());
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 151; @condition b"src\0".as_ptr(); !src.is_null());
        while n > 0 {
            if n >= 8 {
                let dst0 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(0), 4)),
                );
                let dst4 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(4), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(4), 4)),
                );
                dst0.store4(core::slice::from_raw_parts_mut(dst, 4));
                dst4.store4(core::slice::from_raw_parts_mut(dst.add(4), 4));
                dst = dst.add(8);
                src = src.add(8);
                n -= 8;
                continue;
            }
            crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 161; @condition b"n <= 7\0".as_ptr(); n <= 7);
            if n >= 4 {
                kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(0), 4)),
                )
                .store4(core::slice::from_raw_parts_mut(dst, 4));
                dst = dst.add(4);
                src = src.add(4);
                n -= 4;
            }
            if n >= 2 {
                kernel(
                    Self::Load2(core::slice::from_raw_parts(dst.add(0), 2)),
                    Self::Load2(core::slice::from_raw_parts(src.add(0), 2)),
                )
                .store2(core::slice::from_raw_parts_mut(dst, 2));
                dst = dst.add(2);
                src = src.add(2);
                n -= 2;
            }
            if n >= 1 {
                kernel(
                    Self::Load1(core::slice::from_raw_parts(dst.add(0), 1)),
                    Self::Load1(core::slice::from_raw_parts(src.add(0), 1)),
                )
                .store1(core::slice::from_raw_parts_mut(dst, 1));
            }
            break;
        }
    }
    // Source: src/core/Sk4px.h:178-206
    /// Safety: all n pixels/alphas must be valid. Kernel returns values, retaining no storage borrow.
    pub unsafe fn MapDstAlpha(
        mut n: i32,
        mut dst: *mut u32,
        mut a: *const u8,
        kernel: impl Fn(Self, Self) -> Self,
    ) {
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 181; @condition b"dst\0".as_ptr(); !dst.is_null());
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 182; @condition b"a\0".as_ptr(); !a.is_null());
        while n > 0 {
            if n >= 8 {
                let dst0 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(0), 4)),
                );
                let dst4 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(4), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(4), 4)),
                );
                dst0.store4(core::slice::from_raw_parts_mut(dst, 4));
                dst4.store4(core::slice::from_raw_parts_mut(dst.add(4), 4));
                dst = dst.add(8);
                a = a.add(8);
                n -= 8;
                continue;
            }
            crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 192; @condition b"n <= 7\0".as_ptr(); n <= 7);
            if n >= 4 {
                kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(0), 4)),
                )
                .store4(core::slice::from_raw_parts_mut(dst, 4));
                dst = dst.add(4);
                a = a.add(4);
                n -= 4;
            }
            if n >= 2 {
                kernel(
                    Self::Load2(core::slice::from_raw_parts(dst.add(0), 2)),
                    Self::Load2Alphas(core::slice::from_raw_parts(a.add(0), 2)),
                )
                .store2(core::slice::from_raw_parts_mut(dst, 2));
                dst = dst.add(2);
                a = a.add(2);
                n -= 2;
            }
            if n >= 1 {
                kernel(
                    Self::Load1(core::slice::from_raw_parts(dst.add(0), 1)),
                    Self::new([*a.add(0); 16]),
                )
                .store1(core::slice::from_raw_parts_mut(dst, 1));
            }
            break;
        }
    }
    // Source: src/core/Sk4px.h:209-238
    /// Safety: all n pixels/alphas must be valid. Kernel returns values, retaining no storage borrow.
    pub unsafe fn MapDstSrcAlpha(
        mut n: i32,
        mut dst: *mut u32,
        mut src: *const u32,
        mut a: *const u8,
        kernel: impl Fn(Self, Self, Self) -> Self,
    ) {
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 212; @condition b"dst\0".as_ptr(); !dst.is_null());
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 213; @condition b"src\0".as_ptr(); !src.is_null());
        crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 214; @condition b"a\0".as_ptr(); !a.is_null());
        while n > 0 {
            if n >= 8 {
                let dst0 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(0), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(0), 4)),
                );
                let dst4 = kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(4), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(4), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(4), 4)),
                );
                dst0.store4(core::slice::from_raw_parts_mut(dst, 4));
                dst4.store4(core::slice::from_raw_parts_mut(dst.add(4), 4));
                dst = dst.add(8);
                src = src.add(8);
                a = a.add(8);
                n -= 8;
                continue;
            }
            crate::SkASSERT!(@source b"src/core/Sk4px.h\0".as_ptr(), 224; @condition b"n <= 7\0".as_ptr(); n <= 7);
            if n >= 4 {
                kernel(
                    Self::Load4(core::slice::from_raw_parts(dst.add(0), 4)),
                    Self::Load4(core::slice::from_raw_parts(src.add(0), 4)),
                    Self::Load4Alphas(core::slice::from_raw_parts(a.add(0), 4)),
                )
                .store4(core::slice::from_raw_parts_mut(dst, 4));
                dst = dst.add(4);
                src = src.add(4);
                a = a.add(4);
                n -= 4;
            }
            if n >= 2 {
                kernel(
                    Self::Load2(core::slice::from_raw_parts(dst.add(0), 2)),
                    Self::Load2(core::slice::from_raw_parts(src.add(0), 2)),
                    Self::Load2Alphas(core::slice::from_raw_parts(a.add(0), 2)),
                )
                .store2(core::slice::from_raw_parts_mut(dst, 2));
                dst = dst.add(2);
                src = src.add(2);
                a = a.add(2);
                n -= 2;
            }
            if n >= 1 {
                kernel(
                    Self::Load1(core::slice::from_raw_parts(dst.add(0), 1)),
                    Self::Load1(core::slice::from_raw_parts(src.add(0), 1)),
                    Self::new([*a.add(0); 16]),
                )
                .store1(core::slice::from_raw_parts_mut(dst, 1));
            }
            break;
        }
    }
}
// Source: src/core/Sk4px.h:243-243
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Sk4px {
    fV: [u8; 16],
}
