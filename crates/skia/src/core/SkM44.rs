use crate::src::core::SkRect::SkRect;

// Source: include/core/SkM44.h:19-54
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkV2 {
    pub x: f32,
    pub y: f32,
}
impl SkV2 {
    #[inline]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y
    }
    #[inline]
    pub fn Cross(a: Self, b: Self) -> f32 {
        a.x * b.y - a.y * b.x
    }
    #[inline]
    pub fn Normalize(v: Self) -> Self {
        v * (1.0 / v.length())
    }
    #[inline]
    pub fn lengthSquared(&self) -> f32 {
        Self::Dot(*self, *self)
    }
    #[inline]
    pub fn length(&self) -> f32 {
        self.lengthSquared().sqrt()
    }
    #[inline]
    pub fn dot(&self, v: Self) -> f32 {
        Self::Dot(*self, v)
    }
    #[inline]
    pub fn normalize(&self) -> Self {
        Self::Normalize(*self)
    }
    #[inline]
    pub fn cross(&self, v: Self) -> f32 {
        Self::Cross(*self, v)
    }
    #[inline]
    pub fn ptr(&self) -> *const f32 {
        core::ptr::addr_of!(self.x)
    }
    #[inline]
    pub fn ptr_mut(&mut self) -> *mut f32 {
        core::ptr::addr_of_mut!(self.x)
    }
}
impl core::ops::Neg for SkV2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl core::ops::Add for SkV2 {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}
impl core::ops::Sub for SkV2 {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}
impl core::ops::Mul for SkV2 {
    type Output = Self;
    #[inline]
    fn mul(self, v: Self) -> Self {
        Self {
            x: self.x * v.x,
            y: self.y * v.y,
        }
    }
}
impl core::ops::Mul<f32> for SkV2 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}
impl core::ops::Mul<SkV2> for f32 {
    type Output = SkV2;
    #[inline]
    fn mul(self, v: SkV2) -> SkV2 {
        v * self
    }
}
impl core::ops::AddAssign<Self> for SkV2 {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        *self = *self + v;
    }
}
impl core::ops::SubAssign<Self> for SkV2 {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        *self = *self - v;
    }
}
impl core::ops::MulAssign<Self> for SkV2 {
    #[inline]
    fn mul_assign(&mut self, v: Self) {
        *self = *self * v;
    }
}
impl core::ops::MulAssign<f32> for SkV2 {
    #[inline]
    fn mul_assign(&mut self, v: f32) {
        *self = *self * v;
    }
}
impl core::ops::Div<f32> for SkV2 {
    type Output = Self;
    #[inline]
    fn div(self, s: f32) -> Self {
        Self {
            x: self.x / s,
            y: self.y / s,
        }
    }
}
impl core::ops::Div<SkV2> for f32 {
    type Output = SkV2;
    #[inline]
    fn div(self, v: SkV2) -> SkV2 {
        SkV2 {
            x: self / v.x,
            y: self / v.y,
        }
    }
}
impl core::ops::DivAssign<f32> for SkV2 {
    #[inline]
    fn div_assign(&mut self, v: f32) {
        *self = *self / v;
    }
}

// Source: include/core/SkM44.h:56-96
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkV3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl SkV3 {
    #[inline]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }
    #[inline]
    pub fn Cross(a: Self, b: Self) -> Self {
        Self {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }
    #[inline]
    pub fn Normalize(v: Self) -> Self {
        v * (1.0 / v.length())
    }
    #[inline]
    pub fn lengthSquared(&self) -> f32 {
        Self::Dot(*self, *self)
    }
    #[inline]
    pub fn length(&self) -> f32 {
        self.lengthSquared().sqrt()
    }
    #[inline]
    pub fn dot(&self, v: Self) -> f32 {
        Self::Dot(*self, v)
    }
    #[inline]
    pub fn normalize(&self) -> Self {
        Self::Normalize(*self)
    }
    #[inline]
    pub fn cross(&self, v: Self) -> Self {
        Self::Cross(*self, v)
    }
    #[inline]
    pub fn ptr(&self) -> *const f32 {
        core::ptr::addr_of!(self.x)
    }
    #[inline]
    pub fn ptr_mut(&mut self) -> *mut f32 {
        core::ptr::addr_of_mut!(self.x)
    }
}
impl core::ops::Neg for SkV3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
impl core::ops::Add for SkV3 {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}
impl core::ops::Sub for SkV3 {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }
}
impl core::ops::Mul for SkV3 {
    type Output = Self;
    #[inline]
    fn mul(self, v: Self) -> Self {
        Self {
            x: self.x * v.x,
            y: self.y * v.y,
            z: self.z * v.z,
        }
    }
}
impl core::ops::Mul<f32> for SkV3 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}
impl core::ops::Mul<SkV3> for f32 {
    type Output = SkV3;
    #[inline]
    fn mul(self, v: SkV3) -> SkV3 {
        v * self
    }
}
impl core::ops::AddAssign<Self> for SkV3 {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        *self = *self + v;
    }
}
impl core::ops::SubAssign<Self> for SkV3 {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        *self = *self - v;
    }
}
impl core::ops::MulAssign<Self> for SkV3 {
    #[inline]
    fn mul_assign(&mut self, v: Self) {
        *self = *self * v;
    }
}
impl core::ops::MulAssign<f32> for SkV3 {
    #[inline]
    fn mul_assign(&mut self, v: f32) {
        *self = *self * v;
    }
}

// Source: include/core/SkM44.h:98-140
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkV4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl SkV4 {
    #[inline]
    pub fn Dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }
    #[inline]
    pub fn Normalize(v: Self) -> Self {
        v * (1.0 / v.length())
    }
    #[inline]
    pub fn lengthSquared(&self) -> f32 {
        Self::Dot(*self, *self)
    }
    #[inline]
    pub fn length(&self) -> f32 {
        self.lengthSquared().sqrt()
    }
    #[inline]
    pub fn dot(&self, v: Self) -> f32 {
        Self::Dot(*self, v)
    }
    #[inline]
    pub fn normalize(&self) -> Self {
        Self::Normalize(*self)
    }
    #[inline]
    pub fn ptr(&self) -> *const f32 {
        core::ptr::addr_of!(self.x)
    }
    #[inline]
    pub fn ptr_mut(&mut self) -> *mut f32 {
        core::ptr::addr_of_mut!(self.x)
    }
}
impl core::ops::Neg for SkV4 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
impl core::ops::Add for SkV4 {
    type Output = Self;
    #[inline]
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
            w: self.w + v.w,
        }
    }
}
impl core::ops::Sub for SkV4 {
    type Output = Self;
    #[inline]
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
            w: self.w - v.w,
        }
    }
}
impl core::ops::Mul for SkV4 {
    type Output = Self;
    #[inline]
    fn mul(self, v: Self) -> Self {
        Self {
            x: self.x * v.x,
            y: self.y * v.y,
            z: self.z * v.z,
            w: self.w * v.w,
        }
    }
}
impl core::ops::Mul<f32> for SkV4 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
            w: self.w * s,
        }
    }
}
impl core::ops::Mul<SkV4> for f32 {
    type Output = SkV4;
    #[inline]
    fn mul(self, v: SkV4) -> SkV4 {
        v * self
    }
}
impl core::ops::Index<usize> for SkV4 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        assert!(i < 4);
        unsafe { &*self.ptr().add(i) }
    }
}
impl core::ops::IndexMut<usize> for SkV4 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        assert!(i < 4);
        unsafe { &mut *self.ptr_mut().add(i) }
    }
}

// Source: include/core/SkM44.h:152-153
impl Copy for SkM44 {}
impl Clone for SkM44 {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

// Source: include/core/SkM44.h:155-160
impl Default for SkM44 {
    #[inline]
    fn default() -> Self {
        Self {
            fMat: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

// Source: include/core/SkM44.h:162-164
impl SkM44 {
    #[inline]
    pub fn concat(a: &Self, b: &Self) -> Self {
        let mut m = Self::default();
        m.setConcat(a, b);
        m
    }
}

// Source: include/core/SkM44.h:174-179
impl SkM44 {
    #[inline]
    pub const fn nan() -> Self {
        Self {
            fMat: [f32::NAN; 16],
        }
    }
}

// Source: include/core/SkM44.h:184-193
impl SkM44 {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub const fn new(
        m0: f32,
        m4: f32,
        m8: f32,
        m12: f32,
        m1: f32,
        m5: f32,
        m9: f32,
        m13: f32,
        m2: f32,
        m6: f32,
        m10: f32,
        m14: f32,
        m3: f32,
        m7: f32,
        m11: f32,
        m15: f32,
    ) -> Self {
        Self {
            fMat: [
                m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15,
            ],
        }
    }
}

// Source: include/core/SkM44.h:195-202
impl SkM44 {
    #[inline]
    pub fn Rows(r0: SkV4, r1: SkV4, r2: SkV4, r3: SkV4) -> Self {
        let mut m = Self::default();
        m.setRow(0, r0);
        m.setRow(1, r1);
        m.setRow(2, r2);
        m.setRow(3, r3);
        m
    }
}

// Source: include/core/SkM44.h:203-210
impl SkM44 {
    #[inline]
    pub fn Cols(c0: SkV4, c1: SkV4, c2: SkV4, c3: SkV4) -> Self {
        let mut m = Self::default();
        m.setCol(0, c0);
        m.setCol(1, c1);
        m.setCol(2, c2);
        m.setCol(3, c3);
        m
    }
}

// Source: include/core/SkM44.h:212-217
impl SkM44 {
    pub fn RowMajor(r: &[f32; 16]) -> Self {
        Self::new(
            r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13],
            r[14], r[15],
        )
    }
}

// Source: include/core/SkM44.h:218-223
impl SkM44 {
    pub fn ColMajor(c: &[f32; 16]) -> Self {
        Self::new(
            c[0], c[4], c[8], c[12], c[1], c[5], c[9], c[13], c[2], c[6], c[10], c[14], c[3], c[7],
            c[11], c[15],
        )
    }
}

// Source: include/core/SkM44.h:225-230
impl SkM44 {
    #[inline]
    pub fn Translate(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0,
        )
    }
}

// Source: include/core/SkM44.h:232-237
impl SkM44 {
    #[inline]
    pub fn Scale(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
}

// Source: include/core/SkM44.h:239-243
impl SkM44 {
    #[inline]
    pub fn Rotate(axis: SkV3, radians: f32) -> Self {
        let mut m = Self::default();
        m.setRotate(axis, radians);
        m
    }
}

// Source: include/core/SkM44.h:256-258
impl SkM44 {
    pub fn getColMajor(&self, v: &mut [f32; 16]) {
        *v = self.fMat;
    }
}

// Source: include/core/SkM44.h:261-265
impl SkM44 {
    #[inline]
    pub fn rc(&self, r: usize, c: usize) -> f32 {
        debug_assert!(r < 4 && c < 4);
        self.fMat[c * 4 + r]
    }
}

// Source: include/core/SkM44.h:266-270
impl SkM44 {
    #[inline]
    pub fn setRC(&mut self, r: usize, c: usize, value: f32) {
        debug_assert!(r < 4 && c < 4);
        self.fMat[c * 4 + r] = value;
    }
}

// Source: include/core/SkM44.h:272-275
impl SkM44 {
    #[inline]
    pub fn row(&self, i: usize) -> SkV4 {
        debug_assert!(i < 4);
        SkV4 {
            x: self.fMat[i],
            y: self.fMat[i + 4],
            z: self.fMat[i + 8],
            w: self.fMat[i + 12],
        }
    }
}

// Source: include/core/SkM44.h:276-279
impl SkM44 {
    #[inline]
    pub fn col(&self, i: usize) -> SkV4 {
        debug_assert!(i < 4);
        SkV4 {
            x: self.fMat[i * 4],
            y: self.fMat[i * 4 + 1],
            z: self.fMat[i * 4 + 2],
            w: self.fMat[i * 4 + 3],
        }
    }
}

// Source: include/core/SkM44.h:281-287
impl SkM44 {
    #[inline]
    pub fn setRow(&mut self, i: usize, v: SkV4) {
        debug_assert!(i < 4);
        self.fMat[i] = v.x;
        self.fMat[i + 4] = v.y;
        self.fMat[i + 8] = v.z;
        self.fMat[i + 12] = v.w;
    }
}

// Source: include/core/SkM44.h:288-291
impl SkM44 {
    #[inline]
    pub fn setCol(&mut self, i: usize, v: SkV4) {
        debug_assert!(i < 4);
        self.fMat[i * 4..i * 4 + 4].copy_from_slice(&[v.x, v.y, v.z, v.w]);
    }
}

// Source: include/core/SkM44.h:293-299
impl SkM44 {
    #[inline]
    pub fn setIdentity(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

// Source: include/core/SkM44.h:301-307
impl SkM44 {
    #[inline]
    pub fn setTranslate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        *self = Self::Translate(x, y, z);
        self
    }
}

// Source: include/core/SkM44.h:309-315
impl SkM44 {
    #[inline]
    pub fn setScale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        *self = Self::Scale(x, y, z);
        self
    }
}

// Source: include/core/SkM44.h:332-334
impl SkM44 {
    #[inline]
    pub fn setRotateUnit(&mut self, axis: SkV3, radians: f32) -> &mut Self {
        self.setRotateUnitSinCos(axis, radians.sin(), radians.cos())
    }
}

// Source: include/core/SkM44.h:347-349
impl core::ops::Mul for SkM44 {
    type Output = Self;
    #[inline]
    fn mul(self, b: Self) -> Self {
        Self::concat(&self, &b)
    }
}

// Source: include/core/SkM44.h:351-353
impl SkM44 {
    #[inline]
    pub fn preConcat(&mut self, m: &Self) -> &mut Self {
        let a = *self;
        self.setConcat(&a, m)
    }
}

// Source: include/core/SkM44.h:355-357
impl SkM44 {
    #[inline]
    pub fn postConcat(&mut self, m: &Self) -> &mut Self {
        let b = *self;
        self.setConcat(m, &b)
    }
}

// Source: include/core/SkM44.h:378-378
impl SkM44 {
    #[inline]
    pub fn isFinite(&self) -> bool {
        self.fMat.iter().all(|v| v.is_finite())
    }
}

// Source: include/core/SkM44.h:392-394
impl core::ops::Mul<SkV4> for SkM44 {
    type Output = SkV4;
    #[inline]
    fn mul(self, v: SkV4) -> SkV4 {
        self.map(v.x, v.y, v.z, v.w)
    }
}

// Source: include/core/SkM44.h:395-398
impl core::ops::Mul<SkV3> for SkM44 {
    type Output = SkV3;
    #[inline]
    fn mul(self, v: SkV3) -> SkV3 {
        let v4 = self.map(v.x, v.y, v.z, 0.0);
        SkV3 {
            x: v4.x,
            y: v4.y,
            z: v4.z,
        }
    }
}

// Source: include/core/SkM44.h:409-413
impl SkM44 {
    #[inline]
    pub fn asM33(&self) -> crate::src::core::SkMatrix::SkMatrix {
        let m = &self.fMat;
        crate::src::core::SkMatrix::SkMatrix::MakeAll(
            m[0], m[4], m[12], m[1], m[5], m[13], m[3], m[7], m[15],
        )
    }
}

// Source: include/core/SkM44.h:415-420
impl From<&crate::src::core::SkMatrix::SkMatrix> for SkM44 {
    #[inline]
    fn from(src: &crate::src::core::SkMatrix::SkMatrix) -> Self {
        Self::new(
            src[0], src[1], 0.0, src[2], src[3], src[4], 0.0, src[5], 0.0, 0.0, 1.0, 0.0, src[6],
            src[7], 0.0, src[8],
        )
    }
}

// Source: include/core/SkM44.h:437-437
#[derive(Debug)]
pub struct SkM44 {
    pub(crate) fMat: [f32; 16],
}

// Source: src/core/SkM44.cpp:18-35
impl PartialEq for SkM44 {
    fn eq(&self, other: &Self) -> bool {
        if core::ptr::eq(self, other) {
            return true;
        }
        let mut eq = [true; 4];
        for col in 0..4 {
            for lane in 0..4 {
                eq[lane] &= self.fMat[col * 4 + lane] == other.fMat[col * 4 + lane];
            }
        }
        eq.into_iter().all(|v| v)
    }

    // Source: include/core/SkM44.h:252-254
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !b.eq(self)
    }
}

// Source: src/core/SkM44.cpp:37-42
fn transpose_arrays(dst: &mut [f32; 16], src: &[f32; 16]) {
    *dst = [
        src[0], src[4], src[8], src[12], src[1], src[5], src[9], src[13], src[2], src[6], src[10],
        src[14], src[3], src[7], src[11], src[15],
    ];
}

// Source: src/core/SkM44.cpp:44-46
impl SkM44 {
    pub fn getRowMajor(&self, v: &mut [f32; 16]) {
        transpose_arrays(v, &self.fMat);
    }
}

// Source: src/core/SkM44.cpp:48-68
impl SkM44 {
    pub fn setConcat(&mut self, a: &Self, b: &Self) -> &mut Self {
        let mut out = [0.0; 16];
        for col in 0..4 {
            for lane in 0..4 {
                out[col * 4 + lane] = a.fMat[lane] * b.fMat[col * 4]
                    + (a.fMat[4 + lane] * b.fMat[col * 4 + 1]
                        + (a.fMat[8 + lane] * b.fMat[col * 4 + 2]
                            + a.fMat[12 + lane] * b.fMat[col * 4 + 3]));
            }
        }
        self.fMat = out;
        self
    }
}

// Source: src/core/SkM44.cpp:70-87
impl SkM44 {
    pub fn preConcat33(&mut self, b: &crate::src::core::SkMatrix::SkMatrix) -> &mut Self {
        let a = self.fMat;
        let compute = |r0: f32, r1: f32, r3: f32| -> [f32; 4] {
            core::array::from_fn(|i| a[i] * r0 + (a[4 + i] * r1 + a[12 + i] * r3))
        };
        let m0 = compute(b[0], b[3], b[6]);
        let m1 = compute(b[1], b[4], b[7]);
        let m3 = compute(b[2], b[5], b[8]);
        self.fMat[..4].copy_from_slice(&m0);
        self.fMat[4..8].copy_from_slice(&m1);
        self.fMat[12..].copy_from_slice(&m3);
        self
    }
}

// Source: src/core/SkM44.cpp:89-98
impl SkM44 {
    pub fn preTranslate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let m = self.fMat;
        for i in 0..4 {
            self.fMat[12 + i] = m[i] * x + (m[4 + i] * y + (m[8 + i] * z + m[12 + i]));
        }
        self
    }
}

// Source: src/core/SkM44.cpp:100-107
impl SkM44 {
    pub fn postTranslate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let t = [x, y, z, 0.0];
        for col in 0..4 {
            let m: [f32; 4] = self.fMat[col * 4..col * 4 + 4].try_into().unwrap();
            for i in 0..4 {
                self.fMat[col * 4 + i] = t[i] * m[3] + m[i];
            }
        }
        self
    }
}

// Source: src/core/SkM44.cpp:109-116
impl SkM44 {
    pub fn preScale(&mut self, x: f32, y: f32) -> &mut Self {
        for i in 0..4 {
            self.fMat[i] *= x;
            self.fMat[4 + i] *= y;
        }
        self
    }
}

// Source: src/core/SkM44.cpp:118-127
impl SkM44 {
    pub fn preScale3(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        for i in 0..4 {
            self.fMat[i] *= x;
            self.fMat[4 + i] *= y;
            self.fMat[8 + i] *= z;
        }
        self
    }
}

// Source: src/core/SkM44.cpp:129-138
impl SkM44 {
    pub fn map(&self, x: f32, y: f32, z: f32, w: f32) -> SkV4 {
        let m = &self.fMat;
        let mut v = SkV4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        for i in 0..4 {
            v[i] = m[i] * x + (m[4 + i] * y + (m[8 + i] * z + m[12 + i] * w));
        }
        v
    }
}

// Source: src/core/SkM44.cpp:140-162
fn map_rect_affine(src: &SkRect, mat: &[f32; 16]) -> SkRect {
    let flip = [1.0, 1.0, -1.0, -1.0];
    let c0 = [mat[0], mat[1], mat[0], mat[1]];
    let c0 = core::array::from_fn::<_, 4, _>(|i| c0[i] * flip[i]);
    let c1 = [mat[4], mat[5], mat[4], mat[5]];
    let c1 = core::array::from_fn::<_, 4, _>(|i| c1[i] * flip[i]);
    let c3 = [mat[12], mat[13], mat[12], mat[13]];
    let point = |x: f32, y: f32| core::array::from_fn::<_, 4, _>(|i| c0[i] * x + c1[i] * y);
    let min4 = |a: [f32; 4], b: [f32; 4]| {
        core::array::from_fn::<_, 4, _>(|i| if b[i] < a[i] { b[i] } else { a[i] })
    };
    let min_max = min4(
        min4(point(src.fLeft, src.fTop), point(src.fRight, src.fTop)),
        min4(
            point(src.fLeft, src.fBottom),
            point(src.fRight, src.fBottom),
        ),
    );
    let min_max = core::array::from_fn::<_, 4, _>(|i| c3[i] + flip[i] * min_max[i]);
    SkRect::MakeLTRB(min_max[0], min_max[1], min_max[2], min_max[3])
}

// Source: src/core/SkM44.cpp:164-214
fn map_rect_perspective(src: &SkRect, mat: &[f32; 16]) -> SkRect {
    let c0 = SkV4 {
        x: mat[0],
        y: mat[1],
        z: mat[2],
        w: mat[3],
    };
    let c1 = SkV4 {
        x: mat[4],
        y: mat[5],
        z: mat[6],
        w: mat[7],
    };
    let c3 = SkV4 {
        x: mat[12],
        y: mat[13],
        z: mat[14],
        w: mat[15],
    };
    let tl = c0 * src.fLeft + c1 * src.fTop + c3;
    let tr = c0 * src.fRight + c1 * src.fTop + c3;
    let bl = c0 * src.fLeft + c1 * src.fBottom + c3;
    let br = c0 * src.fRight + c1 * src.fBottom + c3;
    let flip = [1.0, 1.0, -1.0, -1.0];
    let min4 = |a: [f32; 4], b: [f32; 4]| {
        core::array::from_fn::<_, 4, _>(|i| if b[i] < a[i] { b[i] } else { a[i] })
    };
    let plane = crate::src::core::SkPathPriv::SkPathPriv::kW0PlaneDistance;
    let project = |p0: SkV4, p1: SkV4, p2: SkV4| -> [f32; 4] {
        let w0 = p0.w;
        if w0 >= plane {
            let xy = [p0.x, p0.y, p0.x, p0.y];
            core::array::from_fn(|i| flip[i] * xy[i] / w0)
        } else {
            let clip = |p: SkV4| -> [f32; 4] {
                let w = p.w;
                if w >= plane {
                    let t = (plane - w0) / (w - w0);
                    let cx = (t * p.x + (1.0 - t) * p0.x) / plane;
                    let cy = (t * p.y + (1.0 - t) * p0.y) / plane;
                    [cx, cy, -cx, -cy]
                } else {
                    [f32::INFINITY; 4]
                }
            };
            min4(clip(p1), clip(p2))
        }
    };
    let min_max = min4(
        min4(project(tl, tr, bl), project(tr, br, tl)),
        min4(project(br, bl, tr), project(bl, tl, br)),
    );
    let min_max = core::array::from_fn::<_, 4, _>(|i| flip[i] * min_max[i]);
    SkRect::MakeLTRB(min_max[0], min_max[1], min_max[2], min_max[3])
}

// Source: src/core/SkM44.cpp:216-224
impl crate::src::core::SkMatrixPriv::SkMatrixPriv {
    pub fn MapRect(m: &SkM44, src: &SkRect) -> SkRect {
        let has_perspective =
            m.fMat[3] != 0.0 || m.fMat[7] != 0.0 || m.fMat[11] != 0.0 || m.fMat[15] != 1.0;
        if has_perspective {
            map_rect_perspective(src, &m.fMat)
        } else {
            map_rect_affine(src, &m.fMat)
        }
    }
}

// Source: src/core/SkM44.cpp:226-239
impl SkM44 {
    pub fn normalizePerspective(&mut self) {
        if self.fMat[15] != 1.0
            && self.fMat[15] != 0.0
            && self.fMat[3] == 0.0
            && self.fMat[7] == 0.0
            && self.fMat[11] == 0.0
        {
            let inv = (1.0f64 / self.fMat[15] as f64) as f32;
            for v in &mut self.fMat {
                *v *= inv;
            }
            self.fMat[15] = 1.0;
        }
    }
}

// Source: src/core/SkM44.cpp:247-254
impl SkM44 {
    pub fn invert(&self, inverse: &mut Self) -> bool {
        let mut tmp = [0.0; 16];
        if crate::src::core::SkMatrixInvert::SkInvert4x4Matrix(&self.fMat, Some(&mut tmp)) == 0.0 {
            return false;
        }
        inverse.fMat = tmp;
        true
    }
}

// Source: src/core/SkM44.cpp:256-260
impl SkM44 {
    pub fn transpose(&self) -> Self {
        let mut trans = Self::default();
        transpose_arrays(&mut trans.fMat, &self.fMat);
        trans
    }
}

// Source: src/core/SkM44.cpp:262-277
impl SkM44 {
    pub fn setRotateUnitSinCos(&mut self, axis: SkV3, s: f32, c: f32) -> &mut Self {
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;
        let t = 1.0 - c;
        *self = Self::new(
            t * x * x + c,
            t * x * y - s * z,
            t * x * z + s * y,
            0.0,
            t * x * y + s * z,
            t * y * y + c,
            t * y * z - s * x,
            0.0,
            t * x * z - s * y,
            t * y * z + s * x,
            t * z * z + c,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self
    }
}

// Source: src/core/SkM44.cpp:279-287
impl SkM44 {
    pub fn setRotate(&mut self, axis: SkV3, radians: f32) -> &mut Self {
        let len = axis.length();
        if len > 0.0 && len.is_finite() {
            self.setRotateUnit(axis * (1.0 / len), radians);
        } else {
            self.setIdentity();
        }
        self
    }
}

// Source: src/core/SkM44.cpp:304-321
impl SkM44 {
    pub fn RectToRect(src: &SkRect, dst: &SkRect) -> Self {
        if src.isEmpty() {
            return Self::default();
        } else if dst.isEmpty() {
            return Self::Scale(0.0, 0.0, 0.0);
        }
        let sx = dst.width() / src.width();
        let sy = dst.height() / src.height();
        let tx = dst.fLeft - sx * src.fLeft;
        let ty = dst.fTop - sy * src.fTop;
        Self::new(
            sx, 0.0, 0.0, tx, 0.0, sy, 0.0, ty, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        )
    }
}

// Source: src/core/SkM44.cpp:323-327
fn normalize(v: SkV3) -> SkV3 {
    let len = v.length();
    if crate::include::core::SkScalar::SkScalarNearlyZero(
        len,
        crate::include::core::SkScalar::SK_ScalarNearlyZero,
    ) {
        v
    } else {
        v * (1.0 / len)
    }
}

// Source: src/core/SkM44.cpp:329-329
fn v4(v: SkV3, w: f32) -> SkV4 {
    SkV4 {
        x: v.x,
        y: v.y,
        z: v.z,
        w,
    }
}

// Source: src/core/SkM44.cpp:331-341
impl SkM44 {
    pub fn LookAt(eye: SkV3, center: SkV3, up: SkV3) -> Self {
        let f = normalize(center - eye);
        let u = normalize(up);
        let s = normalize(f.cross(u));
        let mut m = Self::default();
        if !Self::Cols(v4(s, 0.0), v4(s.cross(f), 0.0), v4(-f, 0.0), v4(eye, 1.0)).invert(&mut m) {
            m.setIdentity();
        }
        m
    }
}

// Source: src/core/SkM44.cpp:343-358
impl SkM44 {
    pub fn Perspective(near: f32, far: f32, angle: f32) -> Self {
        crate::SkASSERT!(@source b"src/core/SkM44.cpp\0".as_ptr(),344; @condition b"far > near\0".as_ptr(); far > near);
        let denom_inv = 1.0 / (far - near);
        let half_angle = angle * 0.5;
        debug_assert!(half_angle != 0.0);
        let cot = 1.0 / half_angle.tan();
        let mut m = Self::default();
        m.setRC(0, 0, cot);
        m.setRC(1, 1, cot);
        m.setRC(2, 2, (far + near) * denom_inv);
        m.setRC(2, 3, 2.0 * far * near * denom_inv);
        m.setRC(3, 2, -1.0);
        m
    }
}

// Source: include/core/SkM44.h:169-169
impl SkM44 {
    /// Native uninitialized constructor. All fields must be initialized before assume_init().
    #[inline]
    pub fn Uninitialized() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
}
