#![allow(non_snake_case)]

use crate::gfx;

// The six components are the source's column-major 2D affine matrix.
// Mapping to gfx::Transform preserves the same column/row layout.
// cpp: foundation/blink_geometry/transforms/affine_transform.h:47-61,200-200
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineTransform {
    transform_: [f64; 6],
}

impl Default for AffineTransform {
    fn default() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }
}

impl AffineTransform {
    pub const fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self {
            transform_: [a, b, c, d, e, f],
        }
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.h:63-65
    pub fn SetMatrix(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        *self = Self::new(a, b, c, d, e, f);
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.h:76-106
    pub fn IsIdentity(&self) -> bool {
        self.transform_ == [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    }
    pub fn IsIdentityOrTranslation(&self) -> bool {
        self.transform_[..4] == [1.0, 0.0, 0.0, 1.0]
    }
    pub fn A(&self) -> f64 {
        self.transform_[0]
    }
    pub fn B(&self) -> f64 {
        self.transform_[1]
    }
    pub fn C(&self) -> f64 {
        self.transform_[2]
    }
    pub fn D(&self) -> f64 {
        self.transform_[3]
    }
    pub fn E(&self) -> f64 {
        self.transform_[4]
    }
    pub fn F(&self) -> f64 {
        self.transform_[5]
    }
    pub fn SetA(&mut self, a: f64) {
        self.transform_[0] = a;
    }
    pub fn SetB(&mut self, b: f64) {
        self.transform_[1] = b;
    }
    pub fn SetC(&mut self, c: f64) {
        self.transform_[2] = c;
    }
    pub fn SetD(&mut self, d: f64) {
        self.transform_[3] = d;
    }
    pub fn SetE(&mut self, e: f64) {
        self.transform_[4] = e;
    }
    pub fn SetF(&mut self, f: f64) {
        self.transform_[5] = f;
    }
    pub fn MakeIdentity(&mut self) {
        *self = Self::default();
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:86-101
    fn DoMultiply(left: &Self, right: &Self) -> Self {
        if left.IsIdentityOrTranslation() {
            return Self::new(
                right.A(),
                right.B(),
                right.C(),
                right.D(),
                left.E() + right.E(),
                left.F() + right.F(),
            );
        }
        Self::new(
            left.A() * right.A() + left.C() * right.B(),
            left.B() * right.A() + left.D() * right.B(),
            left.A() * right.C() + left.C() * right.D(),
            left.B() * right.C() + left.D() * right.D(),
            left.A() * right.E() + left.C() * right.F() + left.E(),
            left.B() * right.E() + left.D() * right.F() + left.F(),
        )
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:105-115
    pub fn PreConcat(&mut self, other: Self) -> &mut Self {
        *self = Self::DoMultiply(self, &other);
        self
    }
    pub fn PostConcat(&mut self, other: Self) -> &mut Self {
        *self = Self::DoMultiply(&other, self);
        self
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:117-132
    pub fn Rotate(&mut self, degrees: f64) -> &mut Self {
        self.RotateRadians(degrees.to_radians())
    }
    pub fn RotateRadians(&mut self, radians: f64) -> &mut Self {
        let cosine = radians.cos();
        let sine = radians.sin();
        self.PreConcat(Self::new(cosine, sine, -sine, cosine, 0.0, 0.0))
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:147-154
    pub fn Translate(&mut self, x: f64, y: f64) -> &mut Self {
        self.transform_[4] += x * self.transform_[0] + y * self.transform_[2];
        self.transform_[5] += x * self.transform_[1] + y * self.transform_[3];
        self
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:136-145
    pub fn Scale(&mut self, x: f64, y: f64) -> &mut Self {
        self.transform_[0] *= x;
        self.transform_[1] *= x;
        self.transform_[2] *= y;
        self.transform_[3] *= y;
        self
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:197-201
    pub fn MapPoint(&self, point: &gfx::PointF) -> gfx::PointF {
        fn clamp(value: f64) -> f32 {
            if value.is_nan() {
                0.0
            } else {
                value.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
            }
        }
        gfx::PointF::new(
            clamp(self.A() * point.x() as f64 + self.C() * point.y() as f64 + self.E()),
            clamp(self.B() * point.x() as f64 + self.D() * point.y() as f64 + self.F()),
        )
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:220-224
    pub fn MapQuad(&self, quad: gfx::QuadF) -> gfx::QuadF {
        gfx::QuadF::new(
            self.MapPoint(quad.p1()),
            self.MapPoint(quad.p2()),
            self.MapPoint(quad.p3()),
            self.MapPoint(quad.p4()),
        )
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:208-217
    pub fn MapRect(&self, rect: gfx::RectF) -> gfx::RectF {
        if self.IsIdentityOrTranslation() {
            gfx::RectF::new(self.MapPoint(&rect.origin()), rect.size())
        } else {
            self.MapQuad(gfx::QuadF::from(rect)).BoundingBox()
        }
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:62-83
    pub fn Det(&self) -> f64 {
        self.A() * self.D() - self.B() * self.C()
    }
    pub fn IsInvertible(&self) -> bool {
        self.Det().is_normal()
    }
    pub fn Inverse(&self) -> Self {
        let mut result = Self::default();
        if self.IsIdentityOrTranslation() {
            result.SetE(-self.E());
            result.SetF(-self.F());
            return result;
        }
        let determinant = self.Det();
        if !determinant.is_normal() {
            return result;
        }
        result.SetMatrix(
            self.D() / determinant,
            -self.B() / determinant,
            -self.C() / determinant,
            self.A() / determinant,
            (self.C() * self.F() - self.D() * self.E()) / determinant,
            (self.B() * self.E() - self.A() * self.F()) / determinant,
        );
        result
    }

    // cpp: foundation/blink_geometry/transforms/affine_transform.cc:227-236
    pub fn FromTransform(transform: &gfx::Transform) -> Self {
        let matrix = transform.GetColMajor();
        Self::new(
            matrix[0], matrix[1], matrix[4], matrix[5], matrix[12], matrix[13],
        )
    }
    pub fn ToTransform(&self) -> gfx::Transform {
        gfx::Transform::ColMajor(&[
            self.A(),
            self.B(),
            0.0,
            0.0,
            self.C(),
            self.D(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            self.E(),
            self.F(),
            0.0,
            1.0,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_and_post_concat_preserve_source_order() {
        let translation = SelfAffine::new(1.0, 0.0, 0.0, 1.0, 10.0, 0.0);
        let scale = SelfAffine::new(2.0, 0.0, 0.0, 2.0, 0.0, 0.0);
        let mut pre = translation;
        pre.PreConcat(scale);
        let mut post = translation;
        post.PostConcat(scale);
        let point = gfx::PointF::new(1.0, 0.0);
        assert_eq!(pre.MapPoint(&point).x(), 12.0);
        assert_eq!(post.MapPoint(&point).x(), 22.0);
    }

    type SelfAffine = AffineTransform;
}
