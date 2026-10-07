//! Cubic root solver subset from src/core/SkCubics.{h,cpp}.
//! BinarySearchRootsInInterval is a Rust helper for the upstream binary-search stage.

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
pub(crate) fn RootsReal(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    if if zero(b) {
        zero(a)
    } else {
        (a / b).abs() < 1e-7
    } {
        return crate::src::core::SkQuads::RootsReal(b, c, d);
    }
    if zero(d) {
        let mut r = crate::src::core::SkQuads::RootsReal(a, b, c);
        if !r.iter().any(|&r| zero(r)) {
            r.push(0.0);
        }
        return r;
    }
    if zero(a + b + c + d) {
        let mut r = crate::src::core::SkQuads::RootsReal(a, a + b, -d);
        if !r.iter().any(|&r| ulps(r, 1.0)) {
            r.push(1.0);
        }
        return r;
    }
    let inv = 1.0 / a;
    let a = b * inv;
    let b = c * inv;
    let c = d * inv;
    let a2 = a * a;
    let q = (a2 - b * 3.0) / 9.0;
    let r = (2.0 * a2 * a - 9.0 * a * b + 27.0 * c) / 54.0;
    let r2 = r * r;
    let q3 = q * q * q;
    let delta = r2 - q3;
    if !delta.is_finite() {
        return vec![];
    }
    let adiv = a / 3.0;
    if delta < 0.0 {
        let theta = (r / q3.sqrt()).clamp(-1.0, 1.0).acos();
        let n = -2.0 * q.sqrt();
        let mut out = vec![n * (theta / 3.0).cos() - adiv];
        for angle in [
            theta + 2.0 * std::f64::consts::PI,
            theta - 2.0 * std::f64::consts::PI,
        ] {
            let v = n * (angle / 3.0).cos() - adiv;
            if !out.iter().any(|&r| near(r, v)) {
                out.push(v);
            }
        }
        out
    } else {
        let mut a = (r.abs() + delta.sqrt()).cbrt();
        if r > 0.0 {
            a = -a;
        }
        if !zero(a) {
            a += q / a;
        }
        let mut out = vec![a - adiv];
        if !zero(r2) && ulps(r2, q3) {
            let v = -a / 2.0 - adiv;
            if !near(out[0], v) {
                out.push(v);
            }
        }
        out
    }
}
pub(crate) fn RootsValidT(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
    let mut out = Vec::new();
    for t in RootsReal(a, b, c, d) {
        if t <= 1.00005 && (t >= 1.0 || ulps(t, 1.0)) {
            if !out.iter().any(|&r| ulps(r, 1.0)) {
                out.push(1.0);
            }
        } else if t >= -0.00005 && (t <= 0.0 || zero(t)) {
            if !out.iter().any(|&r| zero(r)) {
                out.push(0.0);
            }
        } else if t > 0.0 && t < 1.0 {
            out.push(t);
        }
    }
    out
}
pub(crate) fn EvalAt(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
    t.mul_add(t.mul_add(t.mul_add(a, b), c), d)
}
pub(crate) fn BinarySearchRootsInInterval(
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    mut start: f64,
    mut stop: f64,
) -> Option<f64> {
    let left = EvalAt(a, b, c, d, start);
    if left.abs() < 1e-8 {
        return Some(start);
    }
    let right = EvalAt(a, b, c, d, stop);
    if !left.is_finite()
        || !right.is_finite()
        || (left > 0.0 && right > 0.0)
        || (left < 0.0 && right < 0.0)
    {
        return None;
    }
    for _ in 0..1000 {
        let step = (start + stop) / 2.0;
        let curr = EvalAt(a, b, c, d, step);
        if curr.abs() < 1e-8 {
            return Some(step);
        }
        if (curr < 0.0 && left < 0.0) || (curr > 0.0 && left > 0.0) {
            start = step;
        } else {
            stop = step;
        }
    }
    None
}
