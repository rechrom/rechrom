//! Scalar quadratic roots from src/core/SkQuads.{h,cpp}.

// SkCubics / SkQuads root solvers and SkGeometry::first_axis_intersection.
// Copyright 2023 Google LLC (BSD-3-Clause). License: ../glyphs/SKIA_LICENSE.
pub(crate) fn zero(a: f64) -> bool {
    a.abs() < f32::EPSILON as f64
}
pub(crate) fn ulps(a: f64, b: f64) -> bool {
    let magnitude = |a: f64| f64::from_bits(a.to_bits() & 0x7ff0_0000_0000_0000);
    let m = magnitude(a).max(f64::MIN_POSITIVE).max(magnitude(b));
    a == b || (b - a).abs() < m * (f64::EPSILON * 17.0)
}
pub(crate) fn near(a: f64, b: f64) -> bool {
    if zero(a) {
        zero(b)
    } else {
        ulps(a, b)
    }
}
pub(crate) fn RootsReal(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a == 0.0 || (b / a).abs() >= 1e16 {
        return if zero(b) {
            if zero(c) {
                vec![0.0]
            } else {
                vec![]
            }
        } else {
            let r = -c / b;
            if r.is_finite() {
                vec![r]
            } else {
                vec![]
            }
        };
    }
    let b = -0.5 * b;
    let b2 = b * b;
    let ac = a * c;
    let rough = b2 - ac;
    let disc = if 3.0 * rough.abs() >= b2 + ac {
        rough
    } else {
        rough + (b.mul_add(b, -b2) - a.mul_add(c, -ac))
    };
    if !disc.is_finite() || disc < 0.0 {
        return vec![];
    }
    let (r0, r1) = if disc == 0.0 {
        (b / a, b / a)
    } else {
        let d = disc.sqrt();
        let r = if b > 0.0 { b + d } else { b - d };
        (r / a, c / r)
    };
    let mut roots = Vec::new();
    for r in [r0, r1] {
        if r.is_finite() {
            roots.push(if zero(r) { 0.0 } else { r });
        }
    }
    if roots.len() == 2 && ulps(roots[0], roots[1]) {
        roots.pop();
    }
    roots
}
