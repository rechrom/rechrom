// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.
#![allow(non_snake_case)]

// Source: include/core/SkPoint.h:23-23
pub type SkIVector = SkIPoint;

// Source: include/core/SkPoint.h:28-30
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct SkIPoint {
    pub fX: i32,
    pub fY: i32,
}

// Source: include/core/SkPoint.h:38-40
impl SkIPoint {
    #[inline]
    pub const fn make(x: i32, y: i32) -> Self {
        Self { fX: x, fY: y }
    }
}

// Source: include/core/SkPoint.h:46-46
impl SkIPoint {
    #[inline]
    pub const fn x(&self) -> i32 {
        self.fX
    }
}

// Source: include/core/SkPoint.h:52-52
impl SkIPoint {
    #[inline]
    pub const fn y(&self) -> i32 {
        self.fY
    }
}

// Source: include/core/SkPoint.h:58-58
impl SkIPoint {
    #[inline]
    pub fn is_zero(&self) -> bool {
        (self.fX | self.fY) == 0
    }
}

// Source: include/core/SkPoint.h:65-68
impl SkIPoint {
    #[inline]
    pub fn set(&mut self, x: i32, y: i32) {
        self.fX = x;
        self.fY = y;
    }
}

// Source: include/core/SkPoint.h:74-76
impl core::ops::Neg for SkIPoint {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::make(self.fX.wrapping_neg(), self.fY.wrapping_neg())
    }
}

// Source: include/core/SkPoint.h:82-85
impl core::ops::AddAssign for SkIPoint {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        self.fX = self.fX.saturating_add(v.fX);
        self.fY = self.fY.saturating_add(v.fY);
    }
}

// Source: include/core/SkPoint.h:91-94
impl core::ops::SubAssign for SkIPoint {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        self.fX = self.fX.saturating_sub(v.fX);
        self.fY = self.fY.saturating_sub(v.fY);
    }
}

// Source: include/core/SkPoint.h:102-104
impl SkIPoint {
    #[inline]
    pub fn equals(&self, x: i32, y: i32) -> bool {
        self.fX == x && self.fY == y
    }
}

// Source: include/core/SkPoint.h:112-124
impl PartialEq for SkIPoint {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fX == b.fX && self.fY == b.fY
    }
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        self.fX != b.fX || self.fY != b.fY
    }
}

// Source: include/core/SkPoint.h:134-136
impl core::ops::Sub for SkIPoint {
    type Output = Self;
    #[inline]
    fn sub(self, b: Self) -> Self {
        Self::make(self.fX.saturating_sub(b.fX), self.fY.saturating_sub(b.fY))
    }
}

// Source: include/core/SkPoint.h:148-150
impl core::ops::Add for SkIPoint {
    type Output = Self;
    #[inline]
    fn add(self, b: Self) -> Self {
        Self::make(self.fX.saturating_add(b.fX), self.fY.saturating_add(b.fY))
    }
}

// Source: include/core/SkPoint.h:158-158
pub type SkVector = SkPoint;

// Source: include/core/SkPoint.h:163-165
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkPoint {
    pub fX: f32,
    pub fY: f32,
}

// Source: include/core/SkPoint.h:173-175
impl SkPoint {
    #[inline]
    pub const fn make(x: f32, y: f32) -> Self {
        Self { fX: x, fY: y }
    }
}

// Source: include/core/SkPoint.h:181-181
impl SkPoint {
    #[inline]
    pub const fn x(&self) -> f32 {
        self.fX
    }
}

// Source: include/core/SkPoint.h:187-187
impl SkPoint {
    #[inline]
    pub const fn y(&self) -> f32 {
        self.fY
    }
}

// Source: include/core/SkPoint.h:193-193
impl SkPoint {
    #[inline]
    pub fn is_zero(&self) -> bool {
        (self.fX == 0.0) & (self.fY == 0.0)
    }
}

// Source: include/core/SkPoint.h:200-203
impl SkPoint {
    #[inline]
    pub fn set(&mut self, x: f32, y: f32) {
        self.fX = x;
        self.fY = y;
    }
}

// Source: include/core/SkPoint.h:214-217
impl SkPoint {
    #[inline]
    pub fn iset(&mut self, x: i32, y: i32) {
        self.fX = x as f32;
        self.fY = y as f32;
    }
}

// Source: include/core/SkPoint.h:227-230
impl SkPoint {
    #[inline]
    pub fn iset_point(&mut self, p: SkIPoint) {
        self.fX = p.fX as f32;
        self.fY = p.fY as f32;
    }
}

// Source: include/core/SkPoint.h:236-239
impl SkPoint {
    #[inline]
    pub fn set_abs(&mut self, p: SkPoint) {
        self.fX = p.fX.abs();
        self.fY = p.fY.abs();
    }
}

// Source: include/core/SkPoint.h:247-249
impl SkPoint {
    #[inline]
    pub fn offset_points_by(points: &mut [Self], offset: SkVector) {
        Self::offset_points(points, offset.fX, offset.fY);
    }
}

// Source: include/core/SkPoint.h:258-262
impl SkPoint {
    #[inline]
    pub fn offset_points(points: &mut [Self], dx: f32, dy: f32) {
        for p in points {
            p.offset(dx, dy);
        }
    }
}

// Source: include/core/SkPoint.h:269-272
impl SkPoint {
    #[inline]
    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.fX += dx;
        self.fY += dy;
    }
}

// Source: include/core/SkPoint.h:282-282
impl SkPoint {
    #[inline]
    pub fn length(&self) -> f32 {
        Self::length_xy(self.fX, self.fY)
    }
}

// Source: include/core/SkPoint.h:292-292
impl SkPoint {
    #[inline]
    pub fn distance_to_origin(&self) -> f32 {
        self.length()
    }
}

// Source: include/core/SkPoint.h:353-353
impl SkPoint {
    #[inline]
    pub fn scale(&mut self, v: f32) {
        self.fX *= v;
        self.fY *= v;
    }
}

// Source: include/core/SkPoint.h:357-360
impl SkPoint {
    #[inline]
    pub fn negate(&mut self) {
        self.fX = -self.fX;
        self.fY = -self.fY;
    }
}

// Source: include/core/SkPoint.h:366-368
impl core::ops::Neg for SkPoint {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::make(-self.fX, -self.fY)
    }
}

// Source: include/core/SkPoint.h:374-377
impl core::ops::AddAssign for SkPoint {
    #[inline]
    fn add_assign(&mut self, v: Self) {
        self.fX += v.fX;
        self.fY += v.fY;
    }
}

// Source: include/core/SkPoint.h:383-386
impl core::ops::SubAssign for SkPoint {
    #[inline]
    fn sub_assign(&mut self, v: Self) {
        self.fX -= v.fX;
        self.fY -= v.fY;
    }
}

// Source: include/core/SkPoint.h:393-395
impl core::ops::Mul<f32> for SkPoint {
    type Output = Self;
    #[inline]
    fn mul(self, v: f32) -> Self {
        Self::make(self.fX * v, self.fY * v)
    }
}

// Source: include/core/SkPoint.h:402-406
impl core::ops::MulAssign<f32> for SkPoint {
    #[inline]
    fn mul_assign(&mut self, v: f32) {
        self.fX *= v;
        self.fY *= v;
    }
}

// Source: include/core/SkPoint.h:412-414
impl SkPoint {
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.fX.is_finite() && self.fY.is_finite()
    }
}

// Source: include/core/SkPoint.h:422-424
impl SkPoint {
    #[inline]
    pub fn equals(&self, x: f32, y: f32) -> bool {
        self.fX == x && self.fY == y
    }
}

// Source: include/core/SkPoint.h:432-444
impl PartialEq for SkPoint {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fX == b.fX && self.fY == b.fY
    }
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        self.fX != b.fX || self.fY != b.fY
    }
}

// Source: include/core/SkPoint.h:455-457
impl core::ops::Sub for SkPoint {
    type Output = Self;
    #[inline]
    fn sub(self, b: Self) -> Self {
        Self::make(self.fX - b.fX, self.fY - b.fY)
    }
}

// Source: include/core/SkPoint.h:469-471
impl core::ops::Add for SkPoint {
    type Output = Self;
    #[inline]
    fn add(self, b: Self) -> Self {
        Self::make(self.fX + b.fX, self.fY + b.fY)
    }
}

// Source: include/core/SkPoint.h:508-510
impl SkPoint {
    #[inline]
    pub fn distance(a: Self, b: Self) -> f32 {
        Self::length_xy(a.fX - b.fX, a.fY - b.fY)
    }
}

// Source: include/core/SkPoint.h:518-520
impl SkPoint {
    #[inline]
    pub fn dot_product(a: Self, b: Self) -> f32 {
        a.fX * b.fX + a.fY * b.fY
    }
}

// Source: include/core/SkPoint.h:532-534
impl SkPoint {
    #[inline]
    pub fn cross_product(a: Self, b: Self) -> f32 {
        a.fX * b.fY - a.fY * b.fX
    }
}

// Source: include/core/SkPoint.h:545-547
impl SkPoint {
    #[inline]
    pub fn cross(&self, v: Self) -> f32 {
        Self::cross_product(*self, v)
    }
}

// Source: include/core/SkPoint.h:554-556
impl SkPoint {
    #[inline]
    pub fn dot(&self, v: Self) -> f32 {
        Self::dot_product(*self, v)
    }
}

// Native-name/interface projections reuse the bodies above/below. Rust-only
// overload spellings use OffsetVector and setLengthXY. No numerical rewrite.
// Source: include/core/SkPoint.h:38-40
impl SkIPoint {
    pub const fn Make(x: i32, y: i32) -> Self {
        Self::make(x, y)
    }
}
// Source: include/core/SkPoint.h:58-58
impl SkIPoint {
    pub fn isZero(&self) -> bool {
        self.is_zero()
    }
}
// Source: include/core/SkPoint.h:173-175
impl SkPoint {
    pub const fn Make(x: f32, y: f32) -> Self {
        Self::make(x, y)
    }
}
// Source: include/core/SkPoint.h:193-193
impl SkPoint {
    pub fn isZero(&self) -> bool {
        self.is_zero()
    }
}
// Source: include/core/SkPoint.h:236-239
impl SkPoint {
    pub fn setAbs(&mut self, p: SkPoint) {
        self.set_abs(p)
    }
}
// Source: include/core/SkPoint.h:247-249
impl SkPoint {
    pub fn OffsetVector(points: &mut [Self], count: i32, offset: SkVector) {
        Self::Offset(points, count, offset.fX, offset.fY)
    }
}
// Source: include/core/SkPoint.h:258-262
impl SkPoint {
    pub fn Offset(points: &mut [Self], count: i32, dx: f32, dy: f32) {
        if count > 0 {
            Self::offset_points(&mut points[..count as usize], dx, dy)
        }
    }
}
// Source: include/core/SkPoint.h:292-292
impl SkPoint {
    pub fn distanceToOrigin(&self) -> f32 {
        self.distance_to_origin()
    }
}
// Source: include/core/SkPoint.h:314-314
impl SkPoint {
    pub fn setNormalize(&mut self, x: f32, y: f32) -> bool {
        self.set_normalize(x, y)
    }
}
// Source: include/core/SkPoint.h:325-325
impl SkPoint {
    pub fn setLength(&mut self, length: f32) -> bool {
        self.set_length(length)
    }
}
// Source: include/core/SkPoint.h:338-338
impl SkPoint {
    pub fn setLengthXY(&mut self, x: f32, y: f32, length: f32) -> bool {
        self.set_length_from(x, y, length)
    }
}
// Source: include/core/SkPoint.h:412-414
impl SkPoint {
    pub fn isFinite(&self) -> bool {
        self.is_finite()
    }
}
// Source: include/core/SkPoint.h:485-485
impl SkPoint {
    pub fn Length(x: f32, y: f32) -> f32 {
        Self::length_xy(x, y)
    }
}
// Source: include/core/SkPoint.h:500-500
impl SkPoint {
    pub fn Normalize(vec: &mut SkVector) -> f32 {
        Self::normalize_length(vec)
    }
}
// Source: include/core/SkPoint.h:508-510
impl SkPoint {
    pub fn Distance(a: Self, b: Self) -> f32 {
        Self::distance(a, b)
    }
}
// Source: include/core/SkPoint.h:518-520
impl SkPoint {
    pub fn DotProduct(a: Self, b: Self) -> f32 {
        Self::dot_product(a, b)
    }
}
// Source: include/core/SkPoint.h:532-534
impl SkPoint {
    pub fn CrossProduct(a: Self, b: Self) -> f32 {
        Self::cross_product(a, b)
    }
}
// Source: src/core/SkPoint.cpp:17-20
impl SkPoint {
    pub fn scale_to(&self, scale: f32, dst: &mut Self) {
        dst.set(self.fX * scale, self.fY * scale);
    }
}

// Source: src/core/SkPoint.cpp:22-24
impl SkPoint {
    pub fn normalize(&mut self) -> bool {
        self.set_length_from(self.fX, self.fY, 1.0)
    }
}

// Source: src/core/SkPoint.cpp:26-28
impl SkPoint {
    pub fn set_normalize(&mut self, x: f32, y: f32) -> bool {
        self.set_length_from(x, y, 1.0)
    }
}

// Source: src/core/SkPoint.cpp:30-32
impl SkPoint {
    pub fn set_length(&mut self, length: f32) -> bool {
        self.set_length_from(self.fX, self.fY, length)
    }
}

// Source: src/core/SkPoint.cpp:42-69
pub(crate) fn set_point_length(
    pt: &mut SkPoint,
    x: f32,
    y: f32,
    length: f32,
    orig_length: Option<&mut f32>,
) -> bool {
    let xx = x as f64;
    let yy = y as f64;
    let dmag = (xx * xx + yy * yy).sqrt();
    let dscale = length as f64 / dmag;
    // Compound assignment promotes to double before rounding back to float.
    let x = (x as f64 * dscale) as f32;
    let y = (y as f64 * dscale) as f32;
    if !x.is_finite() || !y.is_finite() || (x == 0.0 && y == 0.0) {
        pt.set(0.0, 0.0);
        return false;
    }
    pt.set(x, y);
    if let Some(mag) = orig_length {
        *mag = dmag as f32;
    }
    true
}

// Source: src/core/SkPoint.cpp:71-77
impl SkPoint {
    pub fn normalize_length(pt: &mut Self) -> f32 {
        let mut mag = 0.0;
        if set_point_length(pt, pt.fX, pt.fY, 1.0, Some(&mut mag)) {
            mag
        } else {
            0.0
        }
    }
}

// Source: src/core/SkPoint.cpp:79-88
impl SkPoint {
    pub fn length_xy(dx: f32, dy: f32) -> f32 {
        let mag2 = dx * dx + dy * dy;
        if mag2.is_finite() {
            mag2.sqrt()
        } else {
            let xx = dx as f64;
            let yy = dy as f64;
            (xx * xx + yy * yy).sqrt() as f32
        }
    }
}

// Source: src/core/SkPoint.cpp:90-92
impl SkPoint {
    pub fn set_length_from(&mut self, x: f32, y: f32, length: f32) -> bool {
        set_point_length(self, x, y, length, None)
    }
}

// Source: src/core/SkPoint.cpp:94-96
impl crate::src::core::SkPointPriv::SkPointPriv {
    pub fn SetLengthFast(pt: &mut SkPoint, length: f32) -> bool {
        set_point_length(pt, pt.fX, pt.fY, length, None)
    }
}

// Source: src/core/SkPoint.cpp:101-124
impl crate::src::core::SkPointPriv::SkPointPriv {
    pub fn DistanceToLineBetweenSqd(
        pt: SkPoint,
        a: SkPoint,
        b: SkPoint,
        side: Option<&mut crate::src::core::SkPointPriv::Side>,
    ) -> f32 {
        let u = b - a;
        let v = pt - a;
        let u_length_sqd = Self::LengthSqd(u);
        let det = u.cross(v);
        if let Some(side) = side {
            *side = if det < 0.0 {
                crate::src::core::SkPointPriv::Side::kLeft_Side
            } else if det > 0.0 {
                crate::src::core::SkPointPriv::Side::kRight_Side
            } else {
                crate::src::core::SkPointPriv::Side::kOn_Side
            };
        }
        let mut temp = det / u_length_sqd;
        temp *= det;
        if !temp.is_finite() {
            return Self::LengthSqd(v);
        }
        temp
    }
}

// Source: src/core/SkPoint.cpp:126-168
impl crate::src::core::SkPointPriv::SkPointPriv {
    pub fn DistanceToLineSegmentBetweenSqd(pt: SkPoint, a: SkPoint, b: SkPoint) -> f32 {
        let u = b - a;
        let v = pt - a;
        let u_length_sqd = Self::LengthSqd(u);
        let u_dot_v = SkPoint::dot_product(u, v);
        if u_dot_v <= 0.0 {
            Self::LengthSqd(v)
        } else if u_dot_v > u_length_sqd {
            Self::DistanceToSqd(b, pt)
        } else {
            let det = u.cross(v);
            let mut temp = det / u_length_sqd;
            temp *= det;
            if !temp.is_finite() {
                return Self::LengthSqd(v);
            }
            temp
        }
    }
}
