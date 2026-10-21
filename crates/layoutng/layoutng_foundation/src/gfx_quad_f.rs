#![allow(non_snake_case)]

use crate::gfx_ext::RectF;
use foundation_base::gfx_geometry::{PointF, SizeF};

// This is the value/constructor part of gfx::QuadF. Intersection and other
// out-of-line methods from quad_f.cc still need their source translations.
// cpp: foundation/gfx_geometry/quad_f.h:24-41
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuadF {
    p1_: PointF,
    p2_: PointF,
    p3_: PointF,
    p4_: PointF,
}

impl QuadF {
    // cpp: foundation/gfx_geometry/quad_f.h:29-33
    pub fn new(p1: PointF, p2: PointF, p3: PointF, p4: PointF) -> Self {
        Self {
            p1_: p1,
            p2_: p2,
            p3_: p3,
            p4_: p4,
        }
    }

    // cpp: foundation/gfx_geometry/quad_f.h:43-51
    pub fn set_p1(&mut self, p: PointF) {
        self.p1_ = p;
    }
    pub fn set_p2(&mut self, p: PointF) {
        self.p2_ = p;
    }
    pub fn set_p3(&mut self, p: PointF) {
        self.p3_ = p;
    }
    pub fn set_p4(&mut self, p: PointF) {
        self.p4_ = p;
    }
    pub fn p1(&self) -> &PointF {
        &self.p1_
    }
    pub fn p2(&self) -> &PointF {
        &self.p2_
    }
    pub fn p3(&self) -> &PointF {
        &self.p3_
    }
    pub fn p4(&self) -> &PointF {
        &self.p4_
    }

    // cpp: foundation/gfx_geometry/quad_f.h:67-84
    pub fn Extents(&self) -> (PointF, PointF) {
        let left = self
            .p1_
            .x()
            .min(self.p2_.x())
            .min(self.p3_.x())
            .min(self.p4_.x());
        let right = self
            .p1_
            .x()
            .max(self.p2_.x())
            .max(self.p3_.x())
            .max(self.p4_.x());
        let top = self
            .p1_
            .y()
            .min(self.p2_.y())
            .min(self.p3_.y())
            .min(self.p4_.y());
        let bottom = self
            .p1_
            .y()
            .max(self.p2_.y())
            .max(self.p3_.y())
            .max(self.p4_.y());
        (PointF::new(left, top), PointF::new(right, bottom))
    }

    pub fn BoundingBox(&self) -> RectF {
        let (min, max) = self.Extents();
        RectF::new(min, SizeF::new(max.x() - min.x(), max.y() - min.y()))
    }

    // cpp: foundation/gfx_geometry/quad_f.cc:123-126
    // cpp: foundation/gfx_geometry/triangle_f.cc:11-38
    pub fn Contains(&self, point: PointF) -> bool {
        fn point_is_in_triangle(point: PointF, r1: PointF, r2: PointF, r3: PointF) -> bool {
            let r31x = (r1.x() - r3.x()) as f64;
            let r31y = (r1.y() - r3.y()) as f64;
            let r32x = (r2.x() - r3.x()) as f64;
            let r32y = (r2.y() - r3.y()) as f64;
            let r3px = point.x() - r3.x();
            let r3py = point.y() - r3.y();
            let denom = r32y * r31x - r32x * r31y;
            let u = (r32y * r3px as f64 - r32x * r3py as f64) / denom;
            let v = (r31x * r3py as f64 - r31y * r3px as f64) / denom;
            let w = 1.0 - u - v;
            u >= 0.0 && v >= 0.0 && w >= 0.0
        }
        point_is_in_triangle(point, self.p1_, self.p2_, self.p3_)
            || point_is_in_triangle(point, self.p1_, self.p3_, self.p4_)
    }

    // cpp: foundation/gfx_geometry/quad_f.cc:133-138
    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        self.p1_ = PointF::new(self.p1_.x() * x_scale, self.p1_.y() * y_scale);
        self.p2_ = PointF::new(self.p2_.x() * x_scale, self.p2_.y() * y_scale);
        self.p3_ = PointF::new(self.p3_.x() * x_scale, self.p3_.y() * y_scale);
        self.p4_ = PointF::new(self.p4_.x() * x_scale, self.p4_.y() * y_scale);
    }
}

// cpp: foundation/gfx_geometry/quad_f.h:35-40
impl From<RectF> for QuadF {
    fn from(rect: RectF) -> Self {
        Self::new(
            PointF::new(rect.x(), rect.y()),
            PointF::new(rect.right(), rect.y()),
            PointF::new(rect.right(), rect.bottom()),
            PointF::new(rect.x(), rect.bottom()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation_base::gfx_geometry::{PointF, SizeF};

    #[test]
    fn rectangle_corners_follow_source_order() {
        let q = QuadF::from(RectF::new(PointF::new(2.0, 3.0), SizeF::new(4.0, 5.0)));
        assert_eq!(*q.p1(), PointF::new(2.0, 3.0));
        assert_eq!(*q.p2(), PointF::new(6.0, 3.0));
        assert_eq!(*q.p3(), PointF::new(6.0, 8.0));
        assert_eq!(*q.p4(), PointF::new(2.0, 8.0));
    }

    #[test]
    fn scaled_quad_contains_edge_and_excludes_outside_point() {
        let mut q = QuadF::from(RectF::new(PointF::new(1.0, 2.0), SizeF::new(2.0, 3.0)));
        q.Scale(2.0, 3.0);
        assert!(q.Contains(PointF::new(2.0, 6.0)));
        assert!(q.Contains(PointF::new(6.0, 15.0)));
        assert!(!q.Contains(PointF::new(6.1, 15.0)));
    }
}
