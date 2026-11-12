// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: ui/gfx/geometry/quad_f.h:51-56,97-98,109-129,144-146
// Source: ui/gfx/geometry/quad_f.cc:16-67,86-121,128-131,140-145,166-298
// Complete the source intersection methods on foundation's shared QuadF
// storage. This extension has no second quad representation.

pub fn CrossProduct(lhs: Vector2dF, rhs: Vector2dF) -> f64 {
    lhs.x() as f64 * rhs.y() as f64 - lhs.y() as f64 * rhs.x() as f64
}
pub use crate::gfx::QuadF;
use crate::gfx::{PointF, RectF, SizeF, Vector2dF};

// Source: ui/gfx/geometry/quad_f.cc:16-37
fn RightMostCornerToVector(rect: &RectF, vector: Vector2dF) -> PointF {
    PointF::new(
        if vector.y() >= 0.0 {
            rect.x()
        } else {
            rect.right()
        },
        if vector.x() >= 0.0 {
            rect.bottom()
        } else {
            rect.y()
        },
    )
}

// Source: ui/gfx/geometry/quad_f.cc:39-67
fn LineIntersectsCircle(center: PointF, radius: f32, p0: PointF, p1: PointF) -> bool {
    let x0 = p0.x() - center.x();
    let y0 = p0.y() - center.y();
    let x1 = p1.x() - center.x();
    let y1 = p1.y() - center.y();
    let radius2 = radius * radius;
    if x0 * x0 + y0 * y0 <= radius2 || x1 * x1 + y1 * y1 <= radius2 {
        return true;
    }
    if p0 == p1 {
        return false;
    }
    let a = y0 - y1;
    let b = x1 - x0;
    let c = x0 * y1 - x1 * y0;
    let distance2 = c * c / (a * a + b * b);
    if distance2 > radius2 {
        return false;
    }
    let x = -a * c / (a * a + b * b);
    let y = -b * c / (a * a + b * b);
    ((x0 <= x && x <= x1) || (x0 >= x && x >= x1)) && ((y0 <= y && y <= y1) || (y1 <= y && y <= y0))
}

// Source: ui/gfx/geometry/quad_f.cc:86-88
fn WithinEpsilon(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

pub trait QuadFHitTestExt {
    fn ContainsQuad(&self, other: &QuadF) -> bool;
    fn IsRectilinear(&self) -> bool;
    fn IsCounterClockwise(&self) -> bool;
    fn Move(&mut self, vector: Vector2dF);
    fn IntersectsRect(&self, rect: &RectF) -> bool;
    fn IntersectsRectPartial(&self, rect: &RectF) -> bool;
    fn IsToTheLeftOfOrTouchingLine(&self, base: PointF, vector: Vector2dF) -> bool;
    fn FullyOutsideOneEdge(&self, quad: &QuadF) -> bool;
    fn IntersectsQuad(&self, quad: &QuadF) -> bool;
    fn IntersectsCircle(&self, center: PointF, radius: f32) -> bool;
    fn IntersectsEllipse(&self, center: PointF, radii: SizeF) -> bool;
}

impl QuadFHitTestExt for QuadF {
    // Source: ui/gfx/geometry/quad_f.cc:90-96
    fn IsRectilinear(&self) -> bool {
        (WithinEpsilon(self.p1().x(), self.p2().x())
            && WithinEpsilon(self.p2().y(), self.p3().y())
            && WithinEpsilon(self.p3().x(), self.p4().x())
            && WithinEpsilon(self.p4().y(), self.p1().y()))
            || (WithinEpsilon(self.p1().y(), self.p2().y())
                && WithinEpsilon(self.p2().x(), self.p3().x())
                && WithinEpsilon(self.p3().y(), self.p4().y())
                && WithinEpsilon(self.p4().x(), self.p1().x()))
    }

    // Source: ui/gfx/geometry/quad_f.cc:98-121
    fn IsCounterClockwise(&self) -> bool {
        let p24 = self.p2().y() - self.p4().y();
        let p31 = self.p3().y() - self.p1().y();
        let element1 = self.p1().x() as f64 * p24 as f64;
        let element2 = self.p2().x() as f64 * p31 as f64;
        let element3 = self.p3().x() as f64 * p24 as f64;
        let element4 = self.p4().x() as f64 * p31 as f64;
        element1 + element2 < element3 + element4
    }

    // Source: ui/gfx/geometry/quad_f.cc:128-131
    fn ContainsQuad(&self, other: &QuadF) -> bool {
        self.Contains(*other.p1())
            && self.Contains(*other.p2())
            && self.Contains(*other.p3())
            && self.Contains(*other.p4())
    }

    // Source: ui/gfx/geometry/quad_f.cc:140-145
    fn Move(&mut self, vector: Vector2dF) {
        self.set_p1(*self.p1() + vector);
        self.set_p2(*self.p2() + vector);
        self.set_p3(*self.p3() + vector);
        self.set_p4(*self.p4() + vector);
    }

    // Source: ui/gfx/geometry/quad_f.cc:166-181
    fn IntersectsRect(&self, rect: &RectF) -> bool {
        let (min, max) = self.Extents();
        if min.y() > rect.bottom() || rect.y() > max.y() {
            return false;
        }
        if min.x() > rect.right() || rect.x() > max.x() {
            return false;
        }
        self.IntersectsRectPartial(rect)
    }

    // Source: ui/gfx/geometry/quad_f.cc:183-221
    fn IntersectsRectPartial(&self, rect: &RectF) -> bool {
        let (v1, v2, v3, v4) = if self.IsCounterClockwise() {
            (
                *self.p4() - *self.p1(),
                *self.p1() - *self.p2(),
                *self.p2() - *self.p3(),
                *self.p3() - *self.p4(),
            )
        } else {
            (
                *self.p2() - *self.p1(),
                *self.p3() - *self.p2(),
                *self.p4() - *self.p3(),
                *self.p1() - *self.p4(),
            )
        };
        let mut p = RightMostCornerToVector(rect, v1);
        if CrossProduct(v1, p - *self.p1()) < 0.0 {
            return false;
        }
        p = RightMostCornerToVector(rect, v2);
        if CrossProduct(v2, p - *self.p2()) < 0.0 {
            return false;
        }
        p = RightMostCornerToVector(rect, v3);
        if CrossProduct(v3, p - *self.p3()) < 0.0 {
            return false;
        }
        p = RightMostCornerToVector(rect, v4);
        if CrossProduct(v4, p - *self.p4()) < 0.0 {
            return false;
        }
        true
    }

    // Source: ui/gfx/geometry/quad_f.cc:223-238
    fn IsToTheLeftOfOrTouchingLine(&self, base: PointF, vector: Vector2dF) -> bool {
        if CrossProduct(vector, *self.p1() - base) >= 0.0 {
            return false;
        }
        if CrossProduct(vector, *self.p2() - base) >= 0.0 {
            return false;
        }
        if CrossProduct(vector, *self.p3() - base) >= 0.0 {
            return false;
        }
        if CrossProduct(vector, *self.p4() - base) >= 0.0 {
            return false;
        }
        true
    }

    // Source: ui/gfx/geometry/quad_f.cc:240-272
    fn FullyOutsideOneEdge(&self, quad: &QuadF) -> bool {
        let (v1, v2, v3, v4) = if self.IsCounterClockwise() {
            (
                *self.p4() - *self.p1(),
                *self.p1() - *self.p2(),
                *self.p2() - *self.p3(),
                *self.p3() - *self.p4(),
            )
        } else {
            (
                *self.p2() - *self.p1(),
                *self.p3() - *self.p2(),
                *self.p4() - *self.p3(),
                *self.p1() - *self.p4(),
            )
        };
        if quad.IsToTheLeftOfOrTouchingLine(*self.p1(), v1) {
            return true;
        }
        if quad.IsToTheLeftOfOrTouchingLine(*self.p2(), v2) {
            return true;
        }
        if quad.IsToTheLeftOfOrTouchingLine(*self.p3(), v3) {
            return true;
        }
        if quad.IsToTheLeftOfOrTouchingLine(*self.p4(), v4) {
            return true;
        }
        false
    }

    // Source: ui/gfx/geometry/quad_f.cc:274-278
    fn IntersectsQuad(&self, quad: &QuadF) -> bool {
        !self.FullyOutsideOneEdge(quad) && !quad.FullyOutsideOneEdge(self)
    }

    // Source: ui/gfx/geometry/quad_f.cc:280-285
    fn IntersectsCircle(&self, center: PointF, radius: f32) -> bool {
        self.Contains(center)
            || LineIntersectsCircle(center, radius, *self.p1(), *self.p2())
            || LineIntersectsCircle(center, radius, *self.p2(), *self.p3())
            || LineIntersectsCircle(center, radius, *self.p3(), *self.p4())
            || LineIntersectsCircle(center, radius, *self.p4(), *self.p1())
    }

    // Source: ui/gfx/geometry/quad_f.cc:287-298
    fn IntersectsEllipse(&self, center: PointF, radii: SizeF) -> bool {
        let mut transformed_quad = *self;
        transformed_quad.Move(Vector2dF::new(-center.x(), -center.y()));
        transformed_quad.Scale(radii.height(), radii.width());
        transformed_quad.IntersectsCircle(PointF::default(), radii.height() * radii.width())
    }
}
