#![allow(non_snake_case)]

use crate::src::core::SkFloatingPoint::{
    sk_double_nearly_zero, sk_doubles_nearly_equal_ulps, SK_DoublePI,
};
use crate::src::core::SkQuads::SkQuads;
// Source: src/core/SkCubics.h:16-16
pub struct SkCubics;
fn quad_roots(a: f64, b: f64, c: f64, out: &mut [f64; 3]) -> usize {
    let mut roots = [0.0; 2];
    let n = SkQuads::RootsReal(a, b, c, &mut roots);
    out[..n].copy_from_slice(&roots[..n]);
    n
}

// Source: src/core/SkCubics.h:51-53
impl SkCubics {
    #[inline]
    pub fn EvalAt(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
        t.mul_add(t.mul_add(t.mul_add(a, b), c), d)
    }
}

// Source: src/core/SkCubics.h:55-57
impl SkCubics {
    pub fn EvalAtCoefficients(coefficients: &[f64; 4], t: f64) -> f64 {
        Self::EvalAt(
            coefficients[0],
            coefficients[1],
            coefficients[2],
            coefficients[3],
            t,
        )
    }
}

// Source: src/core/SkCubics.cpp:18-23
fn nearly_equal(x: f64, y: f64) -> bool {
    if sk_double_nearly_zero(x) {
        return sk_double_nearly_zero(y);
    }
    sk_doubles_nearly_equal_ulps(x, y, 16)
}

// Source: src/core/SkCubics.cpp:29-34
fn close_to_a_quadratic(a: f64, b: f64) -> bool {
    if sk_double_nearly_zero(b) {
        return sk_double_nearly_zero(a);
    }
    (a / b).abs() < 1.0e-7
}

// Source: src/core/SkCubics.cpp:36-123
impl SkCubics {
    // Source: src/core/SkCubics.h:23-24
    pub fn RootsReal(mut aa: f64, bb: f64, cc: f64, dd: f64, out: &mut [f64; 3]) -> usize {
        if close_to_a_quadratic(aa, bb) {
            return quad_roots(bb, cc, dd, out);
        }
        if sk_double_nearly_zero(dd) {
            let n = quad_roots(aa, bb, cc, out);
            if out[..n].iter().any(|x| sk_double_nearly_zero(*x)) {
                return n;
            }
            out[n] = 0.0;
            return n + 1;
        }
        if sk_double_nearly_zero(aa + bb + cc + dd) {
            let n = quad_roots(aa, aa + bb, -dd, out);
            if out[..n]
                .iter()
                .any(|x| sk_doubles_nearly_equal_ulps(*x, 1.0, 16))
            {
                return n;
            }
            out[n] = 1.0;
            return n + 1;
        }
        let inv = 1.0 / aa;
        let a = bb * inv;
        let b = cc * inv;
        let c = dd * inv;
        let a2 = a * a;
        let q = (a2 - b * 3.0) / 9.0;
        let r = (2.0 * a2 * a - 9.0 * a * b + 27.0 * c) / 54.0;
        let r2 = r * r;
        let q3 = q * q * q;
        let delta = r2 - q3;
        if !delta.is_finite() {
            return 0;
        }
        let a_div3 = a / 3.0;
        let mut count = 0;
        if delta < 0.0 {
            let theta = crate::include::private::SkTPin::SkTPin((r / q3.sqrt()), -1.0, 1.0).acos();
            let neg2 = -2.0 * q.sqrt();
            let mut root = neg2 * (theta / 3.0).cos() - a_div3;
            out[count] = root;
            count += 1;
            root = neg2 * ((theta + 2.0 * SK_DoublePI) / 3.0).cos() - a_div3;
            if !nearly_equal(out[0], root) {
                out[count] = root;
                count += 1;
            }
            root = neg2 * ((theta - 2.0 * SK_DoublePI) / 3.0).cos() - a_div3;
            if !nearly_equal(out[0], root) && (count == 1 || !nearly_equal(out[1], root)) {
                out[count] = root;
                count += 1;
            }
        } else {
            aa = (r.abs() + delta.sqrt()).cbrt();
            if r > 0.0 {
                aa = -aa;
            }
            if !sk_double_nearly_zero(aa) {
                aa += q / aa;
            }
            let mut root = aa - a_div3;
            out[count] = root;
            count += 1;
            if !sk_double_nearly_zero(r2) && sk_doubles_nearly_equal_ulps(r2, q3, 16) {
                root = -aa / 2.0 - a_div3;
                if !nearly_equal(out[0], root) {
                    out[count] = root;
                    count += 1;
                }
            }
        }
        count
    }
}

// Source: src/core/SkCubics.cpp:125-149
impl SkCubics {
    // Source: src/core/SkCubics.h:32-33
    pub fn RootsValidT(a: f64, b: f64, c: f64, d: f64, out: &mut [f64; 3]) -> usize {
        let mut all = [0.0; 3];
        let n = Self::RootsReal(a, b, c, d, &mut all);
        let mut found = 0;
        for &t in &all[..n] {
            if t <= 1.00005 && (t >= 1.0 || sk_doubles_nearly_equal_ulps(t, 1.0, 16)) {
                if (found < 1 || !sk_doubles_nearly_equal_ulps(out[0], 1.0, 16))
                    && (found < 2 || !sk_doubles_nearly_equal_ulps(out[1], 1.0, 16))
                {
                    out[found] = 1.0;
                    found += 1;
                }
            } else if t >= -0.00005 && (t <= 0.0 || sk_double_nearly_zero(t)) {
                if (found < 1 || !sk_double_nearly_zero(out[0]))
                    && (found < 2 || !sk_double_nearly_zero(out[1]))
                {
                    out[found] = 0.0;
                    found += 1;
                }
            } else if t > 0.0 && t < 1.0 {
                out[found] = t;
                found += 1;
            }
        }
        found
    }
}

// Source: src/core/SkCubics.cpp:151-155
fn approximately_zero(x: f64) -> bool {
    x.abs() < 0.00000001
}

// Source: src/core/SkCubics.cpp:157-172
fn find_extrema_valid_t(a: f64, b: f64, c: f64, t: &mut [f64; 2]) -> usize {
    let mut roots = [0.0; 2];
    let n = SkQuads::RootsReal(3.0 * a, 2.0 * b, c, &mut roots);
    let mut valid = 0;
    for &r in &roots[..n] {
        if r >= 0.0 && r <= 1.0 {
            t[valid] = r;
            valid += 1;
        }
    }
    valid
}

// Source: src/core/SkCubics.cpp:174-204
fn binary_search(a: f64, b: f64, c: f64, d: f64, mut start: f64, mut stop: f64) -> f64 {
    crate::SkASSERT!(@source b"src/core/SkCubics.cpp\0".as_ptr(),175; @condition b"start <= stop\0".as_ptr(); start <= stop);
    let left = SkCubics::EvalAt(a, b, c, d, start);
    if approximately_zero(left) {
        return start;
    }
    let right = SkCubics::EvalAt(a, b, c, d, stop);
    if !left.is_finite() || !right.is_finite() {
        return -1.0;
    }
    if (left > 0.0 && right > 0.0) || (left < 0.0 && right < 0.0) {
        return -1.0;
    }
    for _ in 0..1000 {
        let step = (start + stop) / 2.0;
        let curr = SkCubics::EvalAt(a, b, c, d, step);
        if approximately_zero(curr) {
            return step;
        }
        if (curr < 0.0 && left < 0.0) || (curr > 0.0 && left > 0.0) {
            start = step;
        } else {
            stop = step;
        }
    }
    -1.0
}

// Source: src/core/SkCubics.cpp:206-239
impl SkCubics {
    // Source: src/core/SkCubics.h:44-45
    pub fn BinarySearchRootsValidT(a: f64, b: f64, c: f64, d: f64, out: &mut [f64; 3]) -> usize {
        if ![a, b, c, d].iter().all(|x| x.is_finite()) {
            return 0;
        }
        let mut regions = [0.0, 0.0, 0.0, 1.0];
        let mut min_max = [0.0; 2];
        let n = find_extrema_valid_t(a, b, c, &mut min_max);
        let mut start = 2 - n;
        if n == 1 {
            regions[start + 1] = min_max[0];
        }
        if n == 2 {
            regions[start + 1] = min_max[0].min(min_max[1]);
            regions[start + 2] = min_max[0].max(min_max[1]);
        }
        let mut found = 0;
        while start < 3 {
            let root = binary_search(a, b, c, d, regions[start], regions[start + 1]);
            if root >= 0.0 {
                if (found < 1 || !approximately_zero(out[0] - root))
                    && (found < 2 || !approximately_zero(out[1] - root))
                {
                    out[found] = root;
                    found += 1;
                }
            }
            start += 1;
        }
        found
    }
}
