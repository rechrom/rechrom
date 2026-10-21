#![allow(non_snake_case)]

// Source: include/core/SkPoint3.h:15-16
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct SkPoint3 {
    pub fX: f32,
    pub fY: f32,
    pub fZ: f32,
}

// Source: include/core/SkPoint3.h:18-22
impl SkPoint3 {
    #[inline]
    pub fn Make(x: f32, y: f32, z: f32) -> Self {
        let mut p = Self::default();
        p.set(x, y, z);
        p
    }
}

// Source: include/core/SkPoint3.h:24-24
impl SkPoint3 {
    #[inline]
    pub fn x(&self) -> f32 {
        self.fX
    }
}

// Source: include/core/SkPoint3.h:25-25
impl SkPoint3 {
    #[inline]
    pub fn y(&self) -> f32 {
        self.fY
    }
}

// Source: include/core/SkPoint3.h:26-26
impl SkPoint3 {
    #[inline]
    pub fn z(&self) -> f32 {
        self.fZ
    }
}

// Source: include/core/SkPoint3.h:28-28
impl SkPoint3 {
    #[inline]
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.fX = x;
        self.fY = y;
        self.fZ = z;
    }
}

// Source: include/core/SkPoint3.h:30-32
impl PartialEq for SkPoint3 {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fX == b.fX && self.fY == b.fY && self.fZ == b.fZ
    }

    // Source: include/core/SkPoint3.h:34-36
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}

// Source: include/core/SkPoint3.h:44-44
impl SkPoint3 {
    #[inline]
    pub fn length(&self) -> f32 {
        Self::Length(self.fX, self.fY, self.fZ)
    }
}

// Source: include/core/SkPoint3.h:54-58
impl SkPoint3 {
    #[inline]
    pub fn makeScale(&self, scale: f32) -> Self {
        Self::Make(scale * self.fX, scale * self.fY, scale * self.fZ)
    }
}

// Source: include/core/SkPoint3.h:62-66
impl SkPoint3 {
    #[inline]
    pub fn scale(&mut self, value: f32) {
        self.fX *= value;
        self.fY *= value;
        self.fZ *= value;
    }
}

// Source: include/core/SkPoint3.h:71-77
impl core::ops::Neg for SkPoint3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::Make(-self.fX, -self.fY, -self.fZ)
    }
}

// Source: include/core/SkPoint3.h:82-84
impl core::ops::Sub for SkPoint3 {
    type Output = Self;
    #[inline]
    fn sub(self, b: Self) -> Self {
        Self::Make(self.fX - b.fX, self.fY - b.fY, self.fZ - b.fZ)
    }
}

// Source: include/core/SkPoint3.h:88-90
impl core::ops::Add for SkPoint3 {
    type Output = Self;
    #[inline]
    fn add(self, b: Self) -> Self {
        Self::Make(self.fX + b.fX, self.fY + b.fY, self.fZ + b.fZ)
    }
}

// Source: include/core/SkPoint3.h:94-98
impl core::ops::AddAssign for SkPoint3 {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        self.fX += v.fX;
        self.fY += v.fY;
        self.fZ += v.fZ;
    }
}

// Source: include/core/SkPoint3.h:102-106
impl core::ops::SubAssign for SkPoint3 {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        self.fX -= v.fX;
        self.fY -= v.fY;
        self.fZ -= v.fZ;
    }
}

// Source: include/core/SkPoint3.h:108-110
impl core::ops::Mul<SkPoint3> for f32 {
    type Output = SkPoint3;
    #[inline]
    fn mul(self, p: SkPoint3) -> SkPoint3 {
        SkPoint3::Make(self * p.fX, self * p.fY, self * p.fZ)
    }
}

// Source: include/core/SkPoint3.h:116-118
impl SkPoint3 {
    #[inline]
    pub fn isFinite(&self) -> bool {
        self.fX.is_finite() && self.fY.is_finite() && self.fZ.is_finite()
    }
}

// Source: include/core/SkPoint3.h:122-124
impl SkPoint3 {
    #[inline]
    pub fn DotProduct(a: &Self, b: &Self) -> f32 {
        a.fX * b.fX + a.fY * b.fY + a.fZ * b.fZ
    }
}

// Source: include/core/SkPoint3.h:126-128
impl SkPoint3 {
    #[inline]
    pub fn dot(&self, v: &Self) -> f32 {
        Self::DotProduct(self, v)
    }
}

// Source: include/core/SkPoint3.h:132-139
impl SkPoint3 {
    #[inline]
    pub fn CrossProduct(a: &Self, b: &Self) -> Self {
        Self::Make(
            a.fY * b.fZ - a.fZ * b.fY,
            a.fZ * b.fX - a.fX * b.fZ,
            a.fX * b.fY - a.fY * b.fX,
        )
    }
}

// Source: include/core/SkPoint3.h:141-143
impl SkPoint3 {
    #[inline]
    pub fn cross(&self, v: &Self) -> Self {
        Self::CrossProduct(self, v)
    }
}

// Source: include/core/SkPoint3.h:146-147
pub type SkVector3 = SkPoint3;
pub type SkColor3f = SkPoint3;

// Source: src/core/SkPoint3.cpp:14-16
fn get_length_squared(x: f32, y: f32, z: f32) -> f32 {
    x * x + y * y + z * z
}

// Source: src/core/SkPoint3.cpp:24-27
fn is_length_nearly_zero(x: f32, y: f32, z: f32, length: &mut f32) -> bool {
    *length = get_length_squared(x, y, z);
    *length
        <= crate::include::core::SkScalar::SK_ScalarNearlyZero
            * crate::include::core::SkScalar::SK_ScalarNearlyZero
}

// Source: src/core/SkPoint3.cpp:29-39
impl SkPoint3 {
    pub fn Length(x: f32, y: f32, z: f32) -> f32 {
        let mag = get_length_squared(x, y, z);
        if mag.is_finite() {
            mag.sqrt()
        } else {
            let xx = x as f64;
            let yy = y as f64;
            let zz = z as f64;
            (xx * xx + yy * yy + zz * zz).sqrt() as f32
        }
    }
}

// Source: src/core/SkPoint3.cpp:49-79
impl SkPoint3 {
    pub fn normalize(&mut self) -> bool {
        let mut mag = 0.0;
        if is_length_nearly_zero(self.fX, self.fY, self.fZ, &mut mag) {
            self.set(0.0, 0.0, 0.0);
            return false;
        }
        let inverse = if mag.is_finite() {
            mag as f64
        } else {
            let x = self.fX as f64;
            let y = self.fY as f64;
            let z = self.fZ as f64;
            x * x + y * y + z * z
        };
        let scale = 1.0 / inverse.sqrt();
        self.fX = (self.fX as f64 * scale) as f32;
        self.fY = (self.fY as f64 * scale) as f32;
        self.fZ = (self.fZ as f64 * scale) as f32;
        if !self.isFinite() {
            self.set(0.0, 0.0, 0.0);
            return false;
        }
        true
    }
}
