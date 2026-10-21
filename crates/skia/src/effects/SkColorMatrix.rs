// Copyright 2007 Google Inc. BSD-style license.
// Rust requires contiguous fields. The header's field group precedes its inline
// method group; each group retains native order. Native friend SkColorFilters
// access is a crate-visible field projection, not public mutable matrix storage.
// Source: include/effects/SkColorMatrix.h:51-54
#[derive(Clone, Copy)]
pub struct SkColorMatrix {
    pub(crate) fMat: [f32; 20],
}
// Source: include/effects/SkColorMatrix.h:20-23
impl SkColorMatrix {
    pub const fn new() -> Self {
        Self::from_values(
            1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0.,
        )
    }
}
// Rust Default delegates to the actual constexpr-equivalent initialized constructor.
impl Default for SkColorMatrix {
    fn default() -> Self {
        Self::new()
    }
}
impl SkColorMatrix {
    // Source: include/effects/SkColorMatrix.h:25-32
    pub const fn from_values(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m04: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m24: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
        m34: f32,
    ) -> Self {
        Self {
            fMat: [
                m00, m01, m02, m03, m04, m10, m11, m12, m13, m14, m20, m21, m22, m23, m24, m30,
                m31, m32, m33, m34,
            ],
        }
    }
    // Source: include/effects/SkColorMatrix.h:34-34
    // Source: src/effects/SkColorMatrix.cpp:13-17
    pub fn RGBtoYUV(cs: crate::src::core::SkImageInfo::SkYUVColorSpace) -> Self {
        let mut m = Self::new();
        crate::src::core::SkYUVMath::SkColorMatrix_RGB2YUV(cs, &mut m.fMat);
        m
    }
    // Source: include/effects/SkColorMatrix.h:35-35
    // Source: src/effects/SkColorMatrix.cpp:19-23
    pub fn YUVtoRGB(cs: crate::src::core::SkImageInfo::SkYUVColorSpace) -> Self {
        let mut m = Self::new();
        crate::src::core::SkYUVMath::SkColorMatrix_YUV2RGB(cs, &mut m.fMat);
        m
    }
    // Source: include/effects/SkColorMatrix.h:43-43
    pub fn preConcat(&mut self, mat: &Self) {
        unsafe {
            set_concat(
                self.fMat.as_mut_ptr(),
                self.fMat.as_ptr(),
                mat.fMat.as_ptr(),
            );
        }
    }
    // Source: include/effects/SkColorMatrix.h:44-44
    pub fn postConcat(&mut self, mat: &Self) {
        unsafe {
            set_concat(
                self.fMat.as_mut_ptr(),
                mat.fMat.as_ptr(),
                self.fMat.as_ptr(),
            );
        }
    }
    // Source: include/effects/SkColorMatrix.h:48-48
    pub fn setRowMajor(&mut self, src: &[f32; 20]) {
        self.fMat.copy_from_slice(src);
    }
    // Source: include/effects/SkColorMatrix.h:49-49
    pub fn getRowMajor(&self, dst: &mut [f32; 20]) {
        dst.copy_from_slice(&self.fMat);
    }
}
// Source: src/effects/SkColorMatrix.cpp:26-34
const kR_Scale: usize = 0;
const kG_Scale: usize = 6;
const kB_Scale: usize = 12;
const kA_Scale: usize = 18;
const kR_Trans: usize = 4;
const kG_Trans: usize = 9;
const kB_Trans: usize = 14;
const kA_Trans: usize = 19;
// Source: src/effects/SkColorMatrix.cpp:37-65
// The private raw projection permits the exact native self-alias cases used by
// pre/postConcat. Every input has twenty initialized floats; result is writable;
// allocation spans are disjoint or identical, with no live overlapping references.
// MaybeUninit represents the native indeterminate temporary until all20 writes.
unsafe fn set_concat(result: *mut f32, outer: *const f32, inner: *const f32) {
    let mut tmp = core::mem::MaybeUninit::<[f32; 20]>::uninit();
    let target = if outer == result.cast_const() || inner == result.cast_const() {
        tmp.as_mut_ptr().cast::<f32>()
    } else {
        result
    };
    let mut index = 0;
    for j in (0..20).step_by(5) {
        for i in 0..4 {
            target.add(index).write(
                *outer.add(j) * *inner.add(i)
                    + *outer.add(j + 1) * *inner.add(i + 5)
                    + *outer.add(j + 2) * *inner.add(i + 10)
                    + *outer.add(j + 3) * *inner.add(i + 15),
            );
            index += 1;
        }
        target.add(index).write(
            *outer.add(j) * *inner.add(4)
                + *outer.add(j + 1) * *inner.add(9)
                + *outer.add(j + 2) * *inner.add(14)
                + *outer.add(j + 3) * *inner.add(19)
                + *outer.add(j + 4),
        );
        index += 1;
    }
    if target != result {
        core::ptr::copy_nonoverlapping(target, result, 20);
    }
}
impl SkColorMatrix {
    // Source: src/effects/SkColorMatrix.cpp:67-70
    pub fn setIdentity(&mut self) {
        self.fMat.fill(0.);
        // Native chained assignments write right to left.
        self.fMat[kA_Scale] = 1.;
        self.fMat[kB_Scale] = 1.;
        self.fMat[kG_Scale] = 1.;
        self.fMat[kR_Scale] = 1.;
    }
    // Source: src/effects/SkColorMatrix.cpp:72-78
    pub fn setScale(&mut self, rScale: f32, gScale: f32, bScale: f32, aScale: f32) {
        self.fMat.fill(0.);
        self.fMat[kR_Scale] = rScale;
        self.fMat[kG_Scale] = gScale;
        self.fMat[kB_Scale] = bScale;
        self.fMat[kA_Scale] = aScale;
    }
    // Source: src/effects/SkColorMatrix.cpp:80-85
    pub fn postTranslate(&mut self, dr: f32, dg: f32, db: f32, da: f32) {
        self.fMat[kR_Trans] += dr;
        self.fMat[kG_Trans] += dg;
        self.fMat[kB_Trans] += db;
        self.fMat[kA_Trans] += da;
    }
    // Source: src/effects/SkColorMatrix.cpp:89-91
    pub fn setConcat(&mut self, a: &Self, b: &Self) {
        unsafe {
            set_concat(self.fMat.as_mut_ptr(), a.fMat.as_ptr(), b.fMat.as_ptr());
        }
    }
}
// Source: src/effects/SkColorMatrix.cpp:95-99
fn setrow(row: &mut [f32], r: f32, g: f32, b: f32) {
    row[0] = r;
    row[1] = g;
    row[2] = b;
}
// Source: src/effects/SkColorMatrix.cpp:101-103
const kHueR: f32 = 0.213;
const kHueG: f32 = 0.715;
const kHueB: f32 = 0.072;
impl SkColorMatrix {
    // Source: src/effects/SkColorMatrix.cpp:105-116
    pub fn setSaturation(&mut self, sat: f32) {
        self.fMat.fill(0.);
        let r = kHueR * (1. - sat);
        let g = kHueG * (1. - sat);
        let b = kHueB * (1. - sat);
        setrow(&mut self.fMat[0..], r + sat, g, b);
        setrow(&mut self.fMat[5..], r, g + sat, b);
        setrow(&mut self.fMat[10..], r, g, b + sat);
        self.fMat[kA_Scale] = 1.;
    }
}
