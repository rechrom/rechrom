pub struct SkBezierCubic;

// Source: src/core/SkBezierCurves.cpp:19-21
fn interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// Source: src/core/SkBezierCurves.cpp:23-55
impl SkBezierCubic {
    pub fn EvalAt(curve: &[f64; 8], t: f64) -> [f64; 2] {
        let in_x = |n: usize| curve[2 * n];
        let in_y = |n: usize| curve[2 * n + 1];
        if t == 0.0 {
            return [in_x(0), in_y(0)];
        }
        if t == 1.0 {
            return [in_x(3), in_y(3)];
        }
        let one_minus_t = 1.0 - t;
        let one_minus_t_squared = one_minus_t * one_minus_t;
        let a = one_minus_t_squared * one_minus_t;
        let b = 3.0 * one_minus_t_squared * t;
        let t_squared = t * t;
        let c = 3.0 * one_minus_t * t_squared;
        let d = t_squared * t;
        [
            a * in_x(0) + b * in_x(1) + c * in_x(2) + d * in_x(3),
            a * in_y(0) + b * in_y(1) + c * in_y(2) + d * in_y(3),
        ]
    }
}

// Source: src/core/SkBezierCurves.cpp:59-97
impl SkBezierCubic {
    pub fn Subdivide(curve: &[f64; 8], t: f64, two_curves: &mut [f64; 14]) {
        debug_assert!(t >= 0.0 && t <= 1.0);
        two_curves[0] = curve[0];
        two_curves[1] = curve[1];
        two_curves[12] = curve[6];
        two_curves[13] = curve[7];
        let x01 = interpolate(curve[0], curve[2], t);
        let y01 = interpolate(curve[1], curve[3], t);
        let x12 = interpolate(curve[2], curve[4], t);
        let y12 = interpolate(curve[3], curve[5], t);
        let x23 = interpolate(curve[4], curve[6], t);
        let y23 = interpolate(curve[5], curve[7], t);
        two_curves[2] = x01;
        two_curves[3] = y01;
        two_curves[10] = x23;
        two_curves[11] = y23;
        two_curves[4] = interpolate(x01, x12, t);
        two_curves[5] = interpolate(y01, y12, t);
        two_curves[8] = interpolate(x12, x23, t);
        two_curves[9] = interpolate(y12, y23, t);
        two_curves[6] = interpolate(two_curves[4], two_curves[8], t);
        two_curves[7] = interpolate(two_curves[5], two_curves[9], t);
    }
}

// Source: src/core/SkBezierCurves.cpp:99-115
impl SkBezierCubic {
    pub fn ConvertToPolynomial(curve: &[f64; 8], y_values: bool) -> [f64; 4] {
        let offset = usize::from(y_values);
        let p = |n: usize| curve[offset + 2 * n];
        [
            -p(0) + 3.0 * p(1) - 3.0 * p(2) + p(3),
            3.0 * p(0) - 6.0 * p(1) + 3.0 * p(2),
            -3.0 * p(0) + 3.0 * p(1),
            p(0),
        ]
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

use crate::src::core::SkCubics::SkCubics;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkQuads::SkQuads;
pub struct SkBezierQuad;

// Source: src/core/SkBezierCurves.cpp:118-122
#[derive(Clone, Copy)]
struct DPoint {
    x: f64,
    y: f64,
}
impl DPoint {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn from_point(p: SkPoint) -> Self {
        Self {
            x: p.fX as f64,
            y: p.fY as f64,
        }
    }
}

// Source: src/core/SkBezierCurves.cpp:124-126
impl core::ops::Neg for DPoint {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

// Source: src/core/SkBezierCurves.cpp:128-130
impl core::ops::Add for DPoint {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}

// Source: src/core/SkBezierCurves.cpp:132-134
impl core::ops::Sub for DPoint {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}

// Source: src/core/SkBezierCurves.cpp:136-138
impl core::ops::Mul<DPoint> for f64 {
    type Output = DPoint;
    fn mul(self, a: DPoint) -> DPoint {
        DPoint::new(self * a.x, self * a.y)
    }
}

// Source: src/core/SkBezierCurves.cpp:141-150
fn pinTRange(t: f64) -> f64 {
    if (t + 1.0) as f32 == 1.0 {
        0.0
    } else if t as f32 == 1.0 {
        1.0
    } else {
        t
    }
}

// Source: src/core/SkBezierCurves.cpp:153-168
impl SkBezierCubic {
    pub fn IntersectWithHorizontalLine<'a>(
        control: &[SkPoint],
        y: f32,
        storage: &'a mut [f32; 3],
    ) -> &'a [f32] {
        debug_assert!(control.len() >= 4);
        let p0 = DPoint::from_point(control[0]);
        let p1 = DPoint::from_point(control[1]);
        let p2 = DPoint::from_point(control[2]);
        let p3 = DPoint::from_point(control[3]);
        let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
        let b = 3.0 * p0 - 6.0 * p1 + 3.0 * p2;
        let c = -3.0 * p0 + 3.0 * p1;
        let d = p0;
        Self::Intersect(a.x, b.x, c.x, d.x, a.y, b.y, c.y, d.y, y, storage)
    }
}

// Source: src/core/SkBezierCurves.cpp:170-187
impl SkBezierCubic {
    pub fn Intersect(
        ax: f64,
        bx: f64,
        cx: f64,
        dx: f64,
        ay: f64,
        by: f64,
        cy: f64,
        dy: f64,
        y: f32,
        storage: &mut [f32; 3],
    ) -> &[f32] {
        let mut roots = [0.0; 3];
        let n = SkCubics::RootsReal(ay, by, cy, dy - y as f64, &mut roots);
        let mut count = 0;
        for &t in &roots[..n] {
            let pinned = pinTRange(t);
            if 0.0 <= pinned && pinned <= 1.0 {
                storage[count] = SkCubics::EvalAt(ax, bx, cx, dx, pinned) as f32;
                count += 1;
            }
        }
        &storage[..count]
    }
}

// Source: src/core/SkBezierCurves.cpp:189-206
impl SkBezierQuad {
    pub fn IntersectWithHorizontalLine<'a>(
        control: &[SkPoint],
        y: f32,
        storage: &'a mut [f32; 2],
    ) -> &'a [f32] {
        debug_assert!(control.len() >= 3);
        let p0 = DPoint::from_point(control[0]);
        let p1 = DPoint::from_point(control[1]);
        let p2 = DPoint::from_point(control[2]);
        let a = p0 - 2.0 * p1 + p2;
        let b = p0 - p1;
        let c = p0;
        Self::Intersect(a.x, b.x, c.x, a.y, b.y, c.y, y as f64, storage)
    }
}

// Source: src/core/SkBezierCurves.cpp:208-227
impl SkBezierQuad {
    pub fn Intersect(
        ax: f64,
        bx: f64,
        cx: f64,
        ay: f64,
        by: f64,
        cy: f64,
        y: f64,
        storage: &mut [f32; 2],
    ) -> &[f32] {
        let roots = SkQuads::Roots(ay, by, cy - y);
        let mut count = 0;
        let t0 = pinTRange(roots.root0);
        if 0.0 <= t0 && t0 <= 1.0 {
            storage[count] = SkQuads::EvalAt(ax, -2.0 * bx, cx, t0) as f32;
            count += 1;
        }
        let t1 = pinTRange(roots.root1);
        if 0.0 <= t1 && t1 <= 1.0 && t1 != t0 {
            storage[count] = SkQuads::EvalAt(ax, -2.0 * bx, cx, t1) as f32;
            count += 1;
        }
        &storage[..count]
    }
}
