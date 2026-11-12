//! Source: ui/gfx/geometry/line_f.h:17-41.
use crate::gfx::{PointF, Vector2dF};
use crate::gfx_quad_geometry::CrossProduct;
#[derive(Clone, Copy)]
pub struct LineF {
    pub p1: PointF,
    pub p2: PointF,
}
impl LineF {
    pub fn new(p1: PointF, p2: PointF) -> Self {
        Self { p1, p2 }
    }
    pub fn Normal(&self) -> Vector2dF {
        Vector2dF::new(self.p1.y() - self.p2.y(), self.p2.x() - self.p1.x())
    }
    pub fn IntersectionWith(&self, other: &Self) -> Option<PointF> {
        let a = self.p2 - self.p1;
        let b = other.p2 - other.p1;
        let denominator = CrossProduct(a, b) as f32;
        if denominator == 0. {
            return None;
        }
        let parameter = (CrossProduct(other.p1 - self.p1, b) / denominator as f64) as f32;
        Some(self.p1 + Vector2dF::new(a.x() * parameter, a.y() * parameter))
    }
}
