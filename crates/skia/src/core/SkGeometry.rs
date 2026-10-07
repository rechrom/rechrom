//! Bezier geometry translated from src/core/SkGeometry.{h,cpp}.
//! Rust argument/return wrappers preserve the migrated subset behavior.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// A collection of functions to work with Bezier paths.
//
// Mainly for internal use. Do not rely on it!

use crate::path::{Point, Transform};

use crate::path::f32x2_t::f32x2;
use crate::path::floating_point::FLOAT_PI;
use crate::path::scalar::{Scalar, SCALAR_NEARLY_ZERO, SCALAR_ROOT_2_OVER_2};

use crate::path::floating_point::{NormalizedF32, NormalizedF32Exclusive};
use crate::path::path_builder::PathDirection;

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

// use for : eval(t) == A * t^2 + B * t + C
#[derive(Clone, Copy, Default, Debug)]
pub struct SkQuadCoeff {
    pub a: f32x2,
    pub b: f32x2,
    pub c: f32x2,
}

impl SkQuadCoeff {
    pub fn from_points(points: &[Point; 3]) -> Self {
        let c = points[0].to_f32x2();
        let p1 = points[1].to_f32x2();
        let p2 = points[2].to_f32x2();
        let b = times_2(p1 - c);
        let a = p2 - times_2(p1) + c;

        SkQuadCoeff { a, b, c }
    }

    pub fn eval(&self, t: f32x2) -> f32x2 {
        (self.a * t + self.b) * t + self.c
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct SkCubicCoeff {
    pub a: f32x2,
    pub b: f32x2,
    pub c: f32x2,
    pub d: f32x2,
}

impl SkCubicCoeff {
    pub fn from_points(points: &[Point; 4]) -> Self {
        let p0 = points[0].to_f32x2();
        let p1 = points[1].to_f32x2();
        let p2 = points[2].to_f32x2();
        let p3 = points[3].to_f32x2();
        let three = f32x2::splat(3.0);

        SkCubicCoeff {
            a: p3 + three * (p1 - p2) - p0,
            b: three * (p2 - times_2(p1) + p0),
            c: three * (p1 - p0),
            d: p0,
        }
    }

    pub fn eval(&self, t: f32x2) -> f32x2 {
        ((self.a * t + self.b) * t + self.c) * t + self.d
    }
}

// TODO: to a custom type?
pub fn new_t_values() -> [NormalizedF32Exclusive; 3] {
    [NormalizedF32Exclusive::ANY; 3]
}

pub fn SkChopQuadAt(src: &[Point], t: NormalizedF32Exclusive, dst: &mut [Point; 5]) {
    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();
    let tt = f32x2::splat(t.get());

    let p01 = interp(p0, p1, tt);
    let p12 = interp(p1, p2, tt);

    dst[0] = Point::from_f32x2(p0);
    dst[1] = Point::from_f32x2(p01);
    dst[2] = Point::from_f32x2(interp(p01, p12, tt));
    dst[3] = Point::from_f32x2(p12);
    dst[4] = Point::from_f32x2(p2);
}

// From Numerical Recipes in C.
//
// Q = -1/2 (B + sign(B) sqrt[B*B - 4*A*C])
// x1 = Q / A
// x2 = C / Q
pub fn SkFindUnitQuadRoots(
    a: f32,
    b: f32,
    c: f32,
    roots: &mut [NormalizedF32Exclusive; 3],
) -> usize {
    if a == 0.0 {
        if let Some(r) = valid_unit_divide(-c, b) {
            roots[0] = r;
            return 1;
        } else {
            return 0;
        }
    }

    // use doubles so we don't overflow temporarily trying to compute R
    let mut dr = f64::from(b) * f64::from(b) - 4.0 * f64::from(a) * f64::from(c);
    if dr < 0.0 {
        return 0;
    }
    dr = dr.sqrt();
    let r = dr as f32;
    if !r.is_finite() {
        return 0;
    }

    let q = if b < 0.0 {
        -(b - r) / 2.0
    } else {
        -(b + r) / 2.0
    };

    let mut roots_offset = 0;
    if let Some(r) = valid_unit_divide(q, a) {
        roots[roots_offset] = r;
        roots_offset += 1;
    }

    if let Some(r) = valid_unit_divide(c, q) {
        roots[roots_offset] = r;
        roots_offset += 1;
    }

    if roots_offset == 2 {
        if roots[0].get() > roots[1].get() {
            roots.swap(0, 1);
        } else if roots[0] == roots[1] {
            // nearly-equal?
            roots_offset -= 1; // skip the double root
        }
    }

    roots_offset
}

pub fn SkChopCubicAt(src: &[Point; 4], t: NormalizedF32Exclusive, dst: &mut [Point]) {
    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();
    let p3 = src[3].to_f32x2();
    let tt = f32x2::splat(t.get());

    let ab = interp(p0, p1, tt);
    let bc = interp(p1, p2, tt);
    let cd = interp(p2, p3, tt);
    let abc = interp(ab, bc, tt);
    let bcd = interp(bc, cd, tt);
    let abcd = interp(abc, bcd, tt);

    dst[0] = Point::from_f32x2(p0);
    dst[1] = Point::from_f32x2(ab);
    dst[2] = Point::from_f32x2(abc);
    dst[3] = Point::from_f32x2(abcd);
    dst[4] = Point::from_f32x2(bcd);
    dst[5] = Point::from_f32x2(cd);
    dst[6] = Point::from_f32x2(p3);
}

// Quad'(t) = At + B, where
// A = 2(a - 2b + c)
// B = 2(b - a)
// Solve for t, only if it fits between 0 < t < 1
pub(crate) fn SkFindQuadExtrema(a: f32, b: f32, c: f32) -> Option<NormalizedF32Exclusive> {
    // At + B == 0
    // t = -B / A
    valid_unit_divide(a - b, a - b - b + c)
}

pub fn valid_unit_divide(mut numer: f32, mut denom: f32) -> Option<NormalizedF32Exclusive> {
    if numer < 0.0 {
        numer = -numer;
        denom = -denom;
    }

    if denom == 0.0 || numer == 0.0 || numer >= denom {
        return None;
    }

    let r = numer / denom;
    NormalizedF32Exclusive::new(r)
}

fn interp(v0: f32x2, v1: f32x2, t: f32x2) -> f32x2 {
    v0 + (v1 - v0) * t
}

fn times_2(value: f32x2) -> f32x2 {
    value + value
}

// F(t)    = a (1 - t) ^ 2 + 2 b t (1 - t) + c t ^ 2
// F'(t)   = 2 (b - a) + 2 (a - 2b + c) t
// F''(t)  = 2 (a - 2b + c)
//
// A = 2 (b - a)
// B = 2 (a - 2b + c)
//
// Maximum curvature for a quadratic means solving
// Fx' Fx'' + Fy' Fy'' = 0
//
// t = - (Ax Bx + Ay By) / (Bx ^ 2 + By ^ 2)
pub(crate) fn SkFindQuadMaxCurvature(src: &[Point; 3]) -> NormalizedF32 {
    let ax = src[1].x - src[0].x;
    let ay = src[1].y - src[0].y;
    let bx = src[0].x - src[1].x - src[1].x + src[2].x;
    let by = src[0].y - src[1].y - src[1].y + src[2].y;

    let mut numer = -(ax * bx + ay * by);
    let mut denom = bx * bx + by * by;
    if denom < 0.0 {
        numer = -numer;
        denom = -denom;
    }

    if numer <= 0.0 {
        return NormalizedF32::ZERO;
    }

    if numer >= denom {
        // Also catches denom=0
        return NormalizedF32::ONE;
    }

    let t = numer / denom;
    NormalizedF32::new(t).unwrap()
}

pub(crate) fn SkEvalQuadAt(src: &[Point; 3], t: NormalizedF32) -> Point {
    Point::from_f32x2(SkQuadCoeff::from_points(src).eval(f32x2::splat(t.get())))
}

pub(crate) fn SkEvalQuadTangentAt(src: &[Point; 3], tol: NormalizedF32) -> Point {
    // The derivative equation is 2(b - a +(a - 2b +c)t). This returns a
    // zero tangent vector when t is 0 or 1, and the control point is equal
    // to the end point. In this case, use the quad end points to compute the tangent.
    if (tol == NormalizedF32::ZERO && src[0] == src[1])
        || (tol == NormalizedF32::ONE && src[1] == src[2])
    {
        return src[2] - src[0];
    }

    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();

    let b = p1 - p0;
    let a = p2 - p1 - b;
    let t = a * f32x2::splat(tol.get()) + b;

    Point::from_f32x2(t + t)
}

// Looking for F' dot F'' == 0
//
// A = b - a
// B = c - 2b + a
// C = d - 3c + 3b - a
//
// F' = 3Ct^2 + 6Bt + 3A
// F'' = 6Ct + 6B
//
// F' dot F'' -> CCt^3 + 3BCt^2 + (2BB + CA)t + AB
pub fn SkFindCubicMaxCurvature<'a>(
    src: &[Point; 4],
    t_values: &'a mut [NormalizedF32; 3],
) -> &'a [NormalizedF32] {
    let mut coeff_x = formulate_f1_dot_f2(&[src[0].x, src[1].x, src[2].x, src[3].x]);
    let coeff_y = formulate_f1_dot_f2(&[src[0].y, src[1].y, src[2].y, src[3].y]);

    for i in 0..4 {
        coeff_x[i] += coeff_y[i];
    }

    let len = solve_cubic_poly(&coeff_x, t_values);
    &t_values[0..len]
}

// Looking for F' dot F'' == 0
//
// A = b - a
// B = c - 2b + a
// C = d - 3c + 3b - a
//
// F' = 3Ct^2 + 6Bt + 3A
// F'' = 6Ct + 6B
//
// F' dot F'' -> CCt^3 + 3BCt^2 + (2BB + CA)t + AB
fn formulate_f1_dot_f2(src: &[f32; 4]) -> [f32; 4] {
    let a = src[1] - src[0];
    let b = src[2] - 2.0 * src[1] + src[0];
    let c = src[3] + 3.0 * (src[1] - src[2]) - src[0];

    [c * c, 3.0 * b * c, 2.0 * b * b + c * a, a * b]
}

/// Solve coeff(t) == 0, returning the number of roots that lie within 0 < t < 1.
/// coeff[0]t^3 + coeff[1]t^2 + coeff[2]t + coeff[3]
///
/// Eliminates repeated roots (so that all t_values are distinct, and are always
/// in increasing order.
fn solve_cubic_poly(coeff: &[f32; 4], t_values: &mut [NormalizedF32; 3]) -> usize {
    if coeff[0].is_nearly_zero() {
        // we're just a quadratic
        let mut tmp_t = new_t_values();
        let count = SkFindUnitQuadRoots(coeff[1], coeff[2], coeff[3], &mut tmp_t);
        for i in 0..count {
            t_values[i] = tmp_t[i].to_normalized();
        }

        return count;
    }

    debug_assert!(coeff[0] != 0.0);

    let inva = coeff[0].invert();
    let a = coeff[1] * inva;
    let b = coeff[2] * inva;
    let c = coeff[3] * inva;

    let q = (a * a - b * 3.0) / 9.0;
    let r = (2.0 * a * a * a - 9.0 * a * b + 27.0 * c) / 54.0;

    let q3 = q * q * q;
    let r2_minus_q3 = r * r - q3;
    let adiv3 = a / 3.0;

    if r2_minus_q3 < 0.0 {
        // we have 3 real roots
        // the divide/root can, due to finite precisions, be slightly outside of -1...1
        let theta = (r / q3.sqrt()).bound(-1.0, 1.0).acos();
        let neg2_root_q = -2.0 * q.sqrt();

        t_values[0] = NormalizedF32::new_clamped(neg2_root_q * (theta / 3.0).cos() - adiv3);
        t_values[1] = NormalizedF32::new_clamped(
            neg2_root_q * ((theta + 2.0 * FLOAT_PI) / 3.0).cos() - adiv3,
        );
        t_values[2] = NormalizedF32::new_clamped(
            neg2_root_q * ((theta - 2.0 * FLOAT_PI) / 3.0).cos() - adiv3,
        );

        // now sort the roots
        sort_array3(t_values);
        collapse_duplicates3(t_values)
    } else {
        // we have 1 real root
        let mut a = r.abs() + r2_minus_q3.sqrt();
        a = scalar_cube_root(a);
        if r > 0.0 {
            a = -a;
        }

        if a != 0.0 {
            a += q / a;
        }

        t_values[0] = NormalizedF32::new_clamped(a - adiv3);
        1
    }
}

fn sort_array3(array: &mut [NormalizedF32; 3]) {
    if array[0] > array[1] {
        array.swap(0, 1);
    }

    if array[1] > array[2] {
        array.swap(1, 2);
    }

    if array[0] > array[1] {
        array.swap(0, 1);
    }
}

fn collapse_duplicates3(array: &mut [NormalizedF32; 3]) -> usize {
    let mut len = 3;

    if array[1] == array[2] {
        len = 2;
    }

    if array[0] == array[1] {
        len = 1;
    }

    len
}

fn scalar_cube_root(x: f32) -> f32 {
    x.powf(0.3333333)
}

// This is SkEvalCubicAt split into three functions.
pub(crate) fn SkEvalCubicAt(src: &[Point; 4], t: NormalizedF32) -> Point {
    Point::from_f32x2(SkCubicCoeff::from_points(src).eval(f32x2::splat(t.get())))
}

// This is SkEvalCubicAt split into three functions.
pub(crate) fn SkEvalCubicTangentAt(src: &[Point; 4], t: NormalizedF32) -> Point {
    // The derivative equation returns a zero tangent vector when t is 0 or 1, and the
    // adjacent control point is equal to the end point. In this case, use the
    // next control point or the end points to compute the tangent.
    if (t.get() == 0.0 && src[0] == src[1]) || (t.get() == 1.0 && src[2] == src[3]) {
        let mut tangent = if t.get() == 0.0 {
            src[2] - src[0]
        } else {
            src[3] - src[1]
        };

        if tangent.x == 0.0 && tangent.y == 0.0 {
            tangent = src[3] - src[0];
        }

        tangent
    } else {
        eval_cubic_derivative(src, t)
    }
}

fn eval_cubic_derivative(src: &[Point; 4], t: NormalizedF32) -> Point {
    let p0 = src[0].to_f32x2();
    let p1 = src[1].to_f32x2();
    let p2 = src[2].to_f32x2();
    let p3 = src[3].to_f32x2();

    let coeff = SkQuadCoeff {
        a: p3 + f32x2::splat(3.0) * (p1 - p2) - p0,
        b: times_2(p2 - times_2(p1) + p0),
        c: p1 - p0,
    };

    Point::from_f32x2(coeff.eval(f32x2::splat(t.get())))
}

// Cubic'(t) = At^2 + Bt + C, where
// A = 3(-a + 3(b - c) + d)
// B = 6(a - 2b + c)
// C = 3(b - a)
// Solve for t, keeping only those that fit between 0 < t < 1
pub(crate) fn find_cubic_extrema(
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    t_values: &mut [NormalizedF32Exclusive; 3],
) -> usize {
    // we divide A,B,C by 3 to simplify
    let aa = d - a + 3.0 * (b - c);
    let bb = 2.0 * (a - b - b + c);
    let cc = b - a;

    SkFindUnitQuadRoots(aa, bb, cc, t_values)
}

// http://www.faculty.idc.ac.il/arik/quality/appendixA.html
//
// Inflection means that curvature is zero.
// Curvature is [F' x F''] / [F'^3]
// So we solve F'x X F''y - F'y X F''y == 0
// After some canceling of the cubic term, we get
// A = b - a
// B = c - 2b + a
// C = d - 3c + 3b - a
// (BxCy - ByCx)t^2 + (AxCy - AyCx)t + AxBy - AyBx == 0
pub(crate) fn SkFindCubicInflections<'a>(
    src: &[Point; 4],
    t_values: &'a mut [NormalizedF32Exclusive; 3],
) -> &'a [NormalizedF32Exclusive] {
    let ax = src[1].x - src[0].x;
    let ay = src[1].y - src[0].y;
    let bx = src[2].x - 2.0 * src[1].x + src[0].x;
    let by = src[2].y - 2.0 * src[1].y + src[0].y;
    let cx = src[3].x + 3.0 * (src[1].x - src[2].x) - src[0].x;
    let cy = src[3].y + 3.0 * (src[1].y - src[2].y) - src[0].y;

    let len = SkFindUnitQuadRoots(
        bx * cy - by * cx,
        ax * cy - ay * cx,
        ax * by - ay * bx,
        t_values,
    );

    &t_values[0..len]
}

// Return location (in t) of cubic cusp, if there is one.
// Note that classify cubic code does not reliably return all cusp'd cubics, so
// it is not called here.
pub(crate) fn SkFindCubicCusp(src: &[Point; 4]) -> Option<NormalizedF32Exclusive> {
    // When the adjacent control point matches the end point, it behaves as if
    // the cubic has a cusp: there's a point of max curvature where the derivative
    // goes to zero. Ideally, this would be where t is zero or one, but math
    // error makes not so. It is not uncommon to create cubics this way; skip them.
    if src[0] == src[1] {
        return None;
    }

    if src[2] == src[3] {
        return None;
    }

    // Cubics only have a cusp if the line segments formed by the control and end points cross.
    // Detect crossing if line ends are on opposite sides of plane formed by the other line.
    if on_same_side(src, 0, 2) || on_same_side(src, 2, 0) {
        return None;
    }

    // Cubics may have multiple points of maximum curvature, although at most only
    // one is a cusp.
    let mut t_values = [NormalizedF32::ZERO; 3];
    let max_curvature = SkFindCubicMaxCurvature(src, &mut t_values);
    for test_t in max_curvature {
        if 0.0 >= test_t.get() || test_t.get() >= 1.0 {
            // no need to consider max curvature on the end
            continue;
        }

        // A cusp is at the max curvature, and also has a derivative close to zero.
        // Choose the 'close to zero' meaning by comparing the derivative length
        // with the overall cubic size.
        let d_pt = eval_cubic_derivative(src, *test_t);
        let d_pt_magnitude = d_pt.length_sqd();
        let precision = calc_cubic_precision(src);
        if d_pt_magnitude < precision {
            // All three max curvature t values may be close to the cusp;
            // return the first one.
            return Some(NormalizedF32Exclusive::new_bounded(test_t.get()));
        }
    }

    None
}

// Returns true if both points src[testIndex], src[testIndex+1] are in the same half plane defined
// by the line segment src[lineIndex], src[lineIndex+1].
fn on_same_side(src: &[Point; 4], test_index: usize, line_index: usize) -> bool {
    let origin = src[line_index];
    let line = src[line_index + 1] - origin;
    let mut crosses = [0.0, 0.0];
    for index in 0..2 {
        let test_line = src[test_index + index] - origin;
        crosses[index] = line.cross(test_line);
    }

    crosses[0] * crosses[1] >= 0.0
}

// Returns a constant proportional to the dimensions of the cubic.
// Constant found through experimentation -- maybe there's a better way....
fn calc_cubic_precision(src: &[Point; 4]) -> f32 {
    (src[1].distance_to_sqd(src[0])
        + src[2].distance_to_sqd(src[1])
        + src[3].distance_to_sqd(src[2]))
        * 1e-8
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct SkConic {
    pub points: [Point; 3],
    pub weight: f32,
}

impl SkConic {
    pub fn new(pt0: Point, pt1: Point, pt2: Point, weight: f32) -> Self {
        SkConic {
            points: [pt0, pt1, pt2],
            weight,
        }
    }

    pub fn from_points(points: &[Point], weight: f32) -> Self {
        SkConic {
            points: [points[0], points[1], points[2]],
            weight,
        }
    }

    fn compute_quad_pow2(&self, tolerance: f32) -> Option<u8> {
        if tolerance < 0.0 || !tolerance.is_finite() {
            return None;
        }

        if !self.points[0].is_finite() || !self.points[1].is_finite() || !self.points[2].is_finite()
        {
            return None;
        }

        // Limit the number of suggested quads to approximate a conic
        const MAX_CONIC_TO_QUAD_POW2: usize = 4;

        // "High order approximation of conic sections by quadratic splines"
        // by Michael Floater, 1993
        let a = self.weight - 1.0;
        let k = a / (4.0 * (2.0 + a));
        let x = k * (self.points[0].x - 2.0 * self.points[1].x + self.points[2].x);
        let y = k * (self.points[0].y - 2.0 * self.points[1].y + self.points[2].y);

        let mut error = (x * x + y * y).sqrt();
        let mut pow2 = 0;
        for _ in 0..MAX_CONIC_TO_QUAD_POW2 {
            if error <= tolerance {
                break;
            }

            error *= 0.25;
            pow2 += 1;
        }

        // Unlike Skia, we always expect `pow2` to be at least 1.
        // Otherwise it produces ugly results.
        Some(pow2.max(1))
    }

    // Chop this conic into N quads, stored continuously in pts[], where
    // N = 1 << pow2. The amount of storage needed is (1 + 2 * N)
    pub fn chop_into_quads_pow2(&self, pow2: u8, points: &mut [Point]) -> u8 {
        debug_assert!(pow2 < 5);

        points[0] = self.points[0];
        subdivide(self, &mut points[1..], pow2);

        let quad_count = 1 << pow2;
        let pt_count = 2 * quad_count + 1;
        if points.iter().take(pt_count).any(|n| !n.is_finite()) {
            // if we generated a non-finite, pin ourselves to the middle of the hull,
            // as our first and last are already on the first/last pts of the hull.
            for p in points.iter_mut().take(pt_count - 1).skip(1) {
                *p = self.points[1];
            }
        }

        1 << pow2
    }

    fn chop(&self) -> (SkConic, SkConic) {
        let scale = f32x2::splat((1.0 + self.weight).invert());
        let new_w = subdivide_weight_value(self.weight);

        let p0 = self.points[0].to_f32x2();
        let p1 = self.points[1].to_f32x2();
        let p2 = self.points[2].to_f32x2();
        let ww = f32x2::splat(self.weight);

        let wp1 = ww * p1;
        let m = (p0 + times_2(wp1) + p2) * scale * f32x2::splat(0.5);
        let mut m_pt = Point::from_f32x2(m);
        if !m_pt.is_finite() {
            let w_d = self.weight as f64;
            let w_2 = w_d * 2.0;
            let scale_half = 1.0 / (1.0 + w_d) * 0.5;
            m_pt.x = ((self.points[0].x as f64
                + w_2 * self.points[1].x as f64
                + self.points[2].x as f64)
                * scale_half) as f32;

            m_pt.y = ((self.points[0].y as f64
                + w_2 * self.points[1].y as f64
                + self.points[2].y as f64)
                * scale_half) as f32;
        }

        (
            SkConic {
                points: [self.points[0], Point::from_f32x2((p0 + wp1) * scale), m_pt],
                weight: new_w,
            },
            SkConic {
                points: [m_pt, Point::from_f32x2((wp1 + p2) * scale), self.points[2]],
                weight: new_w,
            },
        )
    }

    pub fn build_unit_arc(
        u_start: Point,
        u_stop: Point,
        dir: PathDirection,
        user_transform: Transform,
        dst: &mut [SkConic; 5],
    ) -> Option<&[SkConic]> {
        // rotate by x,y so that u_start is (1.0)
        let x = u_start.dot(u_stop);
        let mut y = u_start.cross(u_stop);

        let abs_y = y.abs();

        // check for (effectively) coincident vectors
        // this can happen if our angle is nearly 0 or nearly 180 (y == 0)
        // ... we use the dot-prod to distinguish between 0 and 180 (x > 0)
        if abs_y <= SCALAR_NEARLY_ZERO
            && x > 0.0
            && ((y >= 0.0 && dir == PathDirection::CW) || (y <= 0.0 && dir == PathDirection::CCW))
        {
            return None;
        }

        if dir == PathDirection::CCW {
            y = -y;
        }

        // We decide to use 1-conic per quadrant of a circle. What quadrant does [xy] lie in?
        //      0 == [0  .. 90)
        //      1 == [90 ..180)
        //      2 == [180..270)
        //      3 == [270..360)
        //
        let mut quadrant = 0;
        if y == 0.0 {
            quadrant = 2; // 180
            debug_assert!((x + 1.0) <= SCALAR_NEARLY_ZERO);
        } else if x == 0.0 {
            debug_assert!(abs_y - 1.0 <= SCALAR_NEARLY_ZERO);
            quadrant = if y > 0.0 { 1 } else { 3 }; // 90 / 270
        } else {
            if y < 0.0 {
                quadrant += 2;
            }

            if (x < 0.0) != (y < 0.0) {
                quadrant += 1;
            }
        }

        let quadrant_points = [
            Point::from_xy(1.0, 0.0),
            Point::from_xy(1.0, 1.0),
            Point::from_xy(0.0, 1.0),
            Point::from_xy(-1.0, 1.0),
            Point::from_xy(-1.0, 0.0),
            Point::from_xy(-1.0, -1.0),
            Point::from_xy(0.0, -1.0),
            Point::from_xy(1.0, -1.0),
        ];

        const QUADRANT_WEIGHT: f32 = SCALAR_ROOT_2_OVER_2;

        let mut conic_count = quadrant;
        for i in 0..conic_count {
            dst[i] = SkConic::from_points(&quadrant_points[i * 2..], QUADRANT_WEIGHT);
        }

        // Now compute any remaining (sub-90-degree) arc for the last conic
        let final_pt = Point::from_xy(x, y);
        let last_q = quadrant_points[quadrant * 2]; // will already be a unit-vector
        let dot = last_q.dot(final_pt);
        debug_assert!(0.0 <= dot && dot <= 1.0 + SCALAR_NEARLY_ZERO);

        if dot < 1.0 {
            let mut off_curve = Point::from_xy(last_q.x + x, last_q.y + y);
            // compute the bisector vector, and then rescale to be the off-curve point.
            // we compute its length from cos(theta/2) = length / 1, using half-angle identity we get
            // length = sqrt(2 / (1 + cos(theta)). We already have cos() when to computed the dot.
            // This is nice, since our computed weight is cos(theta/2) as well!
            let cos_theta_over_2 = ((1.0 + dot) / 2.0).sqrt();
            off_curve.set_length(cos_theta_over_2.invert());
            if !last_q.almost_equal(off_curve) {
                dst[conic_count] = SkConic::new(last_q, off_curve, final_pt, cos_theta_over_2);
                conic_count += 1;
            }
        }

        // now handle counter-clockwise and the initial unitStart rotation
        let mut transform = Transform::from_sin_cos(u_start.y, u_start.x);
        if dir == PathDirection::CCW {
            transform = transform.pre_scale(1.0, -1.0);
        }

        transform = transform.post_concat(user_transform);

        for conic in dst.iter_mut().take(conic_count) {
            transform.map_points(&mut conic.points);
        }

        if conic_count == 0 {
            None
        } else {
            Some(&dst[0..conic_count])
        }
    }
}

fn subdivide_weight_value(w: f32) -> f32 {
    (0.5 + w * 0.5).sqrt()
}

fn subdivide<'a>(src: &SkConic, mut points: &'a mut [Point], mut level: u8) -> &'a mut [Point] {
    if level == 0 {
        points[0] = src.points[1];
        points[1] = src.points[2];
        &mut points[2..]
    } else {
        let mut dst = src.chop();

        let start_y = src.points[0].y;
        let end_y = src.points[2].y;
        if between(start_y, src.points[1].y, end_y) {
            // If the input is monotonic and the output is not, the scan converter hangs.
            // Ensure that the chopped conics maintain their y-order.
            let mid_y = dst.0.points[2].y;
            if !between(start_y, mid_y, end_y) {
                // If the computed midpoint is outside the ends, move it to the closer one.
                let closer_y = if (mid_y - start_y).abs() < (mid_y - end_y).abs() {
                    start_y
                } else {
                    end_y
                };
                dst.0.points[2].y = closer_y;
                dst.1.points[0].y = closer_y;
            }

            if !between(start_y, dst.0.points[1].y, dst.0.points[2].y) {
                // If the 1st control is not between the start and end, put it at the start.
                // This also reduces the quad to a line.
                dst.0.points[1].y = start_y;
            }

            if !between(dst.1.points[0].y, dst.1.points[1].y, end_y) {
                // If the 2nd control is not between the start and end, put it at the end.
                // This also reduces the quad to a line.
                dst.1.points[1].y = end_y;
            }

            // Verify that all five points are in order.
            debug_assert!(between(start_y, dst.0.points[1].y, dst.0.points[2].y));
            debug_assert!(between(
                dst.0.points[1].y,
                dst.0.points[2].y,
                dst.1.points[1].y
            ));
            debug_assert!(between(dst.0.points[2].y, dst.1.points[1].y, end_y));
        }

        level -= 1;
        points = subdivide(&dst.0, points, level);
        subdivide(&dst.1, points, level)
    }
}

// This was originally developed and tested for pathops: see SkOpTypes.h
// returns true if (a <= b <= c) || (a >= b >= c)
fn between(a: f32, b: f32, c: f32) -> bool {
    (a - b) * (c - b) <= 0.0
}

pub(crate) struct SkAutoConicToQuads {
    pub points: [Point; 64],
    pub len: u8, // the number of quads
}

impl SkAutoConicToQuads {
    pub fn compute(pt0: Point, pt1: Point, pt2: Point, weight: f32) -> Option<Self> {
        let conic = SkConic::new(pt0, pt1, pt2, weight);
        let pow2 = conic.compute_quad_pow2(0.25)?;
        let mut points = [Point::zero(); 64];
        let len = conic.chop_into_quads_pow2(pow2, &mut points);
        Some(SkAutoConicToQuads { points, len })
    }
}

#[cfg(test)]
mod path_geometry_tests {
    use super::*;

    #[test]
    fn eval_cubic_at_1() {
        let src = [
            Point::from_xy(30.0, 40.0),
            Point::from_xy(30.0, 40.0),
            Point::from_xy(171.0, 45.0),
            Point::from_xy(180.0, 155.0),
        ];

        assert_eq!(
            SkEvalCubicAt(&src, NormalizedF32::ZERO),
            Point::from_xy(30.0, 40.0)
        );
        assert_eq!(
            SkEvalCubicTangentAt(&src, NormalizedF32::ZERO),
            Point::from_xy(141.0, 5.0)
        );
    }

    #[test]
    fn find_cubic_max_curvature_1() {
        let src = [
            Point::from_xy(20.0, 160.0),
            Point::from_xy(20.0001, 160.0),
            Point::from_xy(160.0, 20.0),
            Point::from_xy(160.0001, 20.0),
        ];

        let mut t_values = [NormalizedF32::ZERO; 3];
        let t_values = SkFindCubicMaxCurvature(&src, &mut t_values);

        assert_eq!(
            &t_values,
            &[
                NormalizedF32::ZERO,
                NormalizedF32::new_clamped(0.5),
                NormalizedF32::ONE,
            ]
        );
    }
}

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// TODO: return custom type
/// Returns 0 for 1 quad, and 1 for two quads, either way the answer is stored in dst[].
///
/// Guarantees that the 1/2 quads will be monotonic.
pub fn SkChopQuadAtXExtrema(src: &[Point; 3], dst: &mut [Point; 5]) -> usize {
    let a = src[0].x;
    let mut b = src[1].x;
    let c = src[2].x;

    if is_not_monotonic(a, b, c) {
        if let Some(t_value) = valid_unit_divide(a - b, a - b - b + c) {
            SkChopQuadAt(src, t_value, dst);

            // flatten double quad extrema
            dst[1].x = dst[2].x;
            dst[3].x = dst[2].x;

            return 1;
        }

        // if we get here, we need to force dst to be monotonic, even though
        // we couldn't compute a unit_divide value (probably underflow).
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }

    dst[0] = Point::from_xy(a, src[0].y);
    dst[1] = Point::from_xy(b, src[1].y);
    dst[2] = Point::from_xy(c, src[2].y);
    0
}

/// Returns 0 for 1 quad, and 1 for two quads, either way the answer is stored in dst[].
///
/// Guarantees that the 1/2 quads will be monotonic.
pub fn SkChopQuadAtYExtrema(src: &[Point; 3], dst: &mut [Point; 5]) -> usize {
    let a = src[0].y;
    let mut b = src[1].y;
    let c = src[2].y;

    if is_not_monotonic(a, b, c) {
        if let Some(t_value) = valid_unit_divide(a - b, a - b - b + c) {
            SkChopQuadAt(src, t_value, dst);

            // flatten double quad extrema
            dst[1].y = dst[2].y;
            dst[3].y = dst[2].y;

            return 1;
        }

        // if we get here, we need to force dst to be monotonic, even though
        // we couldn't compute a unit_divide value (probably underflow).
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }

    dst[0] = Point::from_xy(src[0].x, a);
    dst[1] = Point::from_xy(src[1].x, b);
    dst[2] = Point::from_xy(src[2].x, c);
    0
}

fn is_not_monotonic(a: f32, b: f32, c: f32) -> bool {
    let ab = a - b;
    let mut bc = b - c;
    if ab < 0.0 {
        bc = -bc;
    }

    ab == 0.0 || bc < 0.0
}

pub fn SkChopCubicAtXExtrema(src: &[Point; 4], dst: &mut [Point; 10]) -> usize {
    let mut t_values = new_t_values();
    let t_values = SkFindCubicExtrema(src[0].x, src[1].x, src[2].x, src[3].x, &mut t_values);

    SkChopCubicAtMultiple(src, t_values, dst);
    if !t_values.is_empty() {
        // we do some cleanup to ensure our X extrema are flat
        dst[2].x = dst[3].x;
        dst[4].x = dst[3].x;
        if t_values.len() == 2 {
            dst[5].x = dst[6].x;
            dst[7].x = dst[6].x;
        }
    }

    t_values.len()
}

/// Given 4 points on a cubic bezier, chop it into 1, 2, 3 beziers such that
/// the resulting beziers are monotonic in Y.
///
/// This is called by the scan converter.
///
/// Depending on what is returned, dst[] is treated as follows:
///
/// - 0: dst[0..3] is the original cubic
/// - 1: dst[0..3] and dst[3..6] are the two new cubics
/// - 2: dst[0..3], dst[3..6], dst[6..9] are the three new cubics
pub fn SkChopCubicAtYExtrema(src: &[Point; 4], dst: &mut [Point; 10]) -> usize {
    let mut t_values = new_t_values();
    let t_values = SkFindCubicExtrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut t_values);

    SkChopCubicAtMultiple(src, t_values, dst);
    if !t_values.is_empty() {
        // we do some cleanup to ensure our Y extrema are flat
        dst[2].y = dst[3].y;
        dst[4].y = dst[3].y;
        if t_values.len() == 2 {
            dst[5].y = dst[6].y;
            dst[7].y = dst[6].y;
        }
    }

    t_values.len()
}

// Cubic'(t) = At^2 + Bt + C, where
// A = 3(-a + 3(b - c) + d)
// B = 6(a - 2b + c)
// C = 3(b - a)
// Solve for t, keeping only those that fit between 0 < t < 1
fn SkFindCubicExtrema(
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    t_values: &mut [NormalizedF32Exclusive; 3],
) -> &[NormalizedF32Exclusive] {
    // we divide A,B,C by 3 to simplify
    let na = d - a + 3.0 * (b - c);
    let nb = 2.0 * (a - b - b + c);
    let nc = b - a;

    let roots = SkFindUnitQuadRoots(na, nb, nc, t_values);
    &t_values[0..roots]
}

// http://code.google.com/p/skia/issues/detail?id=32
//
// This test code would fail when we didn't check the return result of
// valid_unit_divide in SkChopCubicAt(... NormalizedF32Exclusives[], int roots). The reason is
// that after the first chop, the parameters to valid_unit_divide are equal
// (thanks to finite float precision and rounding in the subtracts). Thus
// even though the 2nd NormalizedF32Exclusive looks < 1.0, after we renormalize it, we end
// up with 1.0, hence the need to check and just return the last cubic as
// a degenerate clump of 4 points in the same place.
pub fn SkChopCubicAtMultiple(
    src: &[Point; 4],
    t_values: &[NormalizedF32Exclusive],
    dst: &mut [Point],
) {
    if t_values.is_empty() {
        // nothing to chop
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
        dst[3] = src[3];
    } else {
        let mut t = t_values[0];
        let mut tmp = [Point::zero(); 4];

        // Reduce the `src` lifetime, so we can use `src = &tmp` later.
        let mut src = src;

        let mut dst_offset = 0;
        for i in 0..t_values.len() {
            SkChopCubicAt(src, t, &mut dst[dst_offset..]);
            if i == t_values.len() - 1 {
                break;
            }

            dst_offset += 3;
            // have src point to the remaining cubic (after the chop)
            tmp[0] = dst[dst_offset + 0];
            tmp[1] = dst[dst_offset + 1];
            tmp[2] = dst[dst_offset + 2];
            tmp[3] = dst[dst_offset + 3];
            src = &tmp;

            // watch out in case the renormalized t isn't in range
            let n = valid_unit_divide(
                t_values[i + 1].get() - t_values[i].get(),
                1.0 - t_values[i].get(),
            );

            match n {
                Some(n) => t = n,
                None => {
                    // if we can't, just create a degenerate cubic
                    dst[dst_offset + 4] = src[3];
                    dst[dst_offset + 5] = src[3];
                    dst[dst_offset + 6] = src[3];
                    break;
                }
            }
        }
    }
}

pub fn SkChopCubicAtMaxCurvature(
    src: &[Point; 4],
    t_values: &mut [NormalizedF32Exclusive; 3],
    dst: &mut [Point],
) -> usize {
    let mut roots = [NormalizedF32::ZERO; 3];
    let roots = SkFindCubicMaxCurvature(src, &mut roots);

    // Throw out values not inside 0..1.
    let mut count = 0;
    for root in roots {
        if 0.0 < root.get() && root.get() < 1.0 {
            t_values[count] = NormalizedF32Exclusive::new_bounded(root.get());
            count += 1;
        }
    }

    if count == 0 {
        dst[0..4].copy_from_slice(src);
    } else {
        SkChopCubicAtMultiple(src, &t_values[0..count], dst);
    }

    count + 1
}

pub fn SkChopMonoCubicAtX(src: &[Point; 4], x: f32, dst: &mut [Point; 7]) -> bool {
    cubic_dchop_at_intercept(src, x, true, dst)
}

pub fn SkChopMonoCubicAtY(src: &[Point; 4], y: f32, dst: &mut [Point; 7]) -> bool {
    cubic_dchop_at_intercept(src, y, false, dst)
}

fn cubic_dchop_at_intercept(
    src: &[Point; 4],
    intercept: f32,
    is_vertical: bool,
    dst: &mut [Point; 7],
) -> bool {
    use crate::raster::path64::{cubic64::Cubic64, line_cubic_intersections, point64::Point64};

    let src = [
        Point64::from_point(src[0]),
        Point64::from_point(src[1]),
        Point64::from_point(src[2]),
        Point64::from_point(src[3]),
    ];

    let cubic = Cubic64::new(src);
    let mut roots = [0.0; 3];
    let count = if is_vertical {
        line_cubic_intersections::vertical_intersect(&cubic, f64::from(intercept), &mut roots)
    } else {
        line_cubic_intersections::horizontal_intersect(&cubic, f64::from(intercept), &mut roots)
    };

    if count > 0 {
        let pair = cubic.chop_at(roots[0]);
        for i in 0..7 {
            dst[i] = pair.points[i].to_point();
        }

        true
    } else {
        false
    }
}

#[cfg(test)]
mod raster_geometry_tests {
    use super::*;

    #[test]
    fn chop_cubic_at_y_extrema_1() {
        let src = [
            Point::from_xy(10.0, 20.0),
            Point::from_xy(67.0, 437.0),
            Point::from_xy(298.0, 213.0),
            Point::from_xy(401.0, 214.0),
        ];

        let mut dst = [Point::zero(); 10];
        let n = SkChopCubicAtYExtrema(&src, &mut dst);
        assert_eq!(n, 2);
        assert_eq!(dst[0], Point::from_xy(10.0, 20.0));
        assert_eq!(dst[1], Point::from_xy(37.508274, 221.24475));
        assert_eq!(dst[2], Point::from_xy(105.541855, 273.19803));
        assert_eq!(dst[3], Point::from_xy(180.15599, 273.19803));
        assert_eq!(dst[4], Point::from_xy(259.80502, 273.19803));
        assert_eq!(dst[5], Point::from_xy(346.9527, 213.99666));
        assert_eq!(dst[6], Point::from_xy(400.30844, 213.99666));
        assert_eq!(dst[7], Point::from_xy(400.53958, 213.99666));
        assert_eq!(dst[8], Point::from_xy(400.7701, 213.99777));
        assert_eq!(dst[9], Point::from_xy(401.0, 214.0));
    }
}

pub(crate) fn first_axis_intersection(p: [f64; 4], target: f64) -> Option<f64> {
    let [p0, p1, p2, p3] = p;
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 3.0 * p0 - 6.0 * p1 + 3.0 * p2;
    let c = -3.0 * p0 + 3.0 * p1;
    let d = p0 - target;
    let roots = crate::src::core::SkCubics::RootsValidT(a, b, c, d);
    if roots.is_empty() {
        return None;
    }
    if let Some(r) = roots
        .into_iter()
        .find(|&r| crate::src::core::SkCubics::EvalAt(a, b, c, d, r).abs() < 1e-5)
    {
        return Some(r);
    }
    let mut regions = vec![0.0];
    let mut extrema: Vec<_> = crate::src::core::SkQuads::RootsReal(3.0 * a, 2.0 * b, c)
        .into_iter()
        .filter(|&t| t >= 0.0 && t <= 1.0)
        .collect();
    extrema.sort_by(f64::total_cmp);
    regions.extend(extrema);
    regions.push(1.0);
    for pair in regions.windows(2) {
        if let Some(r) =
            crate::src::core::SkCubics::BinarySearchRootsInInterval(a, b, c, d, pair[0], pair[1])
        {
            if crate::src::core::SkCubics::EvalAt(a, b, c, d, r).abs() < 1e-5 {
                return Some(r);
            }
        }
    }
    None
}

// Legacy Rust spellings retained at the compatibility API boundary.
pub(crate) use SkAutoConicToQuads as AutoConicToQuads;
pub use SkChopCubicAt as chop_cubic_at2;
pub use SkChopCubicAtMaxCurvature as chop_cubic_at_max_curvature;
pub use SkChopCubicAtMultiple as chop_cubic_at;
pub use SkChopCubicAtXExtrema as chop_cubic_at_x_extrema;
pub use SkChopCubicAtYExtrema as chop_cubic_at_y_extrema;
pub use SkChopMonoCubicAtX as chop_mono_cubic_at_x;
pub use SkChopMonoCubicAtY as chop_mono_cubic_at_y;
pub use SkChopQuadAt as chop_quad_at;
pub use SkChopQuadAtXExtrema as chop_quad_at_x_extrema;
pub use SkChopQuadAtYExtrema as chop_quad_at_y_extrema;
pub(crate) use SkConic as Conic;
pub use SkCubicCoeff as CubicCoeff;
pub(crate) use SkEvalCubicAt as eval_cubic_pos_at;
pub(crate) use SkEvalCubicTangentAt as eval_cubic_tangent_at;
pub(crate) use SkEvalQuadAt as eval_quad_at;
pub(crate) use SkEvalQuadTangentAt as eval_quad_tangent_at;
pub(crate) use SkFindCubicCusp as find_cubic_cusp;
pub(crate) use SkFindCubicInflections as find_cubic_inflections;
pub use SkFindCubicMaxCurvature as find_cubic_max_curvature;
pub(crate) use SkFindQuadExtrema as find_quad_extrema;
pub(crate) use SkFindQuadMaxCurvature as find_quad_max_curvature;
pub use SkFindUnitQuadRoots as find_unit_quad_roots;
pub use SkQuadCoeff as QuadCoeff;

// SkChopCubicAtYExtrema and SkChopCubicAt (including parallel two-root chops).
pub(crate) fn chop_y_extrema(p: [Pt; 4]) -> Vec<[Pt; 4]> {
    let [a, b, c, d] = p.map(|p| p.1);
    let roots = unit_quad_roots(d - a + 3.0 * (b - c), 2.0 * (a - b - b + c), b - a);
    fn chop(p: [Pt; 4], t: f32) -> [Pt; 7] {
        let mix = |a: Pt, b: Pt| ((b.0 - a.0) * t + a.0, (b.1 - a.1) * t + a.1);
        let a = mix(p[0], p[1]);
        let b = mix(p[1], p[2]);
        let c = mix(p[2], p[3]);
        let ab = mix(a, b);
        let bc = mix(b, c);
        [p[0], a, ab, mix(ab, bc), bc, c, p[3]]
    }
    match roots.as_slice() {
        [] => vec![p],
        [t] => {
            let mut q = chop(p, *t);
            q[2].1 = q[3].1;
            q[4].1 = q[3].1;
            vec![[q[0], q[1], q[2], q[3]], [q[3], q[4], q[5], q[6]]]
        }
        [t0, t1] => {
            let mut a = chop(p, *t0);
            let mut b = chop(p, *t1);
            let mix = |a: Pt, b: Pt, t: f32| ((b.0 - a.0) * t + a.0, (b.1 - a.1) * t + a.1);
            let mut m0 = mix(a[2], a[4], *t1);
            let mut m1 = mix(b[2], b[4], *t0);
            a[2].1 = a[3].1;
            m0.1 = a[3].1;
            m1.1 = b[3].1;
            b[4].1 = b[3].1;
            vec![
                [a[0], a[1], a[2], a[3]],
                [a[3], m0, m1, b[3]],
                [b[3], b[4], b[5], b[6]],
            ]
        }
        _ => unreachable!(),
    }
}

use crate::src::core::SkAnalyticEdge::Pt;
pub(crate) fn unit_divide(mut n: f32, mut d: f32) -> Option<f32> {
    if n < 0.0 {
        n = -n;
        d = -d;
    }
    if d == 0.0 || n == 0.0 || n >= d {
        return None;
    }
    let r = n / d;
    (r.is_finite() && r != 0.0).then_some(r)
}

pub(crate) fn unit_quad_roots(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a == 0.0 {
        return unit_divide(-c, b).into_iter().collect();
    }
    let disc = f64::from(b) * f64::from(b) - 4.0 * f64::from(a) * f64::from(c);
    if disc < 0.0 {
        return Vec::new();
    }
    let r = disc.sqrt() as f32;
    if !r.is_finite() {
        return Vec::new();
    }
    let q = if b < 0.0 {
        -(b - r) * 0.5
    } else {
        -(b + r) * 0.5
    };
    let mut roots: Vec<_> = [unit_divide(q, a), unit_divide(c, q)]
        .into_iter()
        .flatten()
        .collect();
    roots.sort_by(|a, b| a.total_cmp(b));
    roots.dedup();
    roots
}

pub(crate) fn quad_root(c0: f32, c1: f32, c2: f32, target: f32) -> Option<f32> {
    unit_quad_roots(c0 - c1 - c1 + c2, 2.0 * (c1 - c0), c0 - target)
        .first()
        .copied()
}

pub(crate) fn chop_quad(p: [Pt; 3], t: f32) -> [Pt; 5] {
    let lerp = |a: Pt, b: Pt| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    let a = lerp(p[0], p[1]);
    let b = lerp(p[1], p[2]);
    [p[0], a, lerp(a, b), b, p[2]]
}

pub(crate) fn chop_quad_axis(mut p: [Pt; 3], axis: usize) -> Vec<[Pt; 3]> {
    let get = |p: Pt| if axis == 0 { p.0 } else { p.1 };
    let set = |p: &mut Pt, v| if axis == 0 { p.0 = v } else { p.1 = v };
    let [a, b, c] = p.map(get);
    let ab = a - b;
    let bc = if ab < 0.0 { c - b } else { b - c };
    if ab == 0.0 || bc < 0.0 {
        let mut n = ab;
        let mut d = a - b - b + c;
        if n < 0.0 {
            n = -n;
            d = -d;
        }
        if d != 0.0 && n != 0.0 && n < d {
            let t = n / d;
            if t > 0.0 && t < 1.0 {
                let mut q = chop_quad(p, t);
                let mid = get(q[2]);
                set(&mut q[1], mid);
                set(&mut q[3], mid);
                return vec![[q[0], q[1], q[2]], [q[2], q[3], q[4]]];
            }
        }
        set(&mut p[1], if (a - b).abs() < (b - c).abs() { a } else { c });
    }
    vec![p]
}

pub(crate) fn conic_quads(p: [Pt; 3], w: f32, depth: u32, out: &mut Vec<[Pt; 3]>) {
    if depth == 0 {
        out.push(p);
        return;
    }
    let scale = 1.0 / (1.0 + w);
    // The reference archive enables SK_SUPPORT_LEGACY_CONIC_CHOP.
    // Preserve its multiplication order, then SkGeometry::subdivide's
    // monotonic-Y correction before building quadratic edges.
    let wp = (p[1].0 * w, p[1].1 * w);
    let mut a = ((p[0].0 + wp.0) * scale, (p[0].1 + wp.1) * scale);
    let mut b = (
        (p[0].0 + 2.0 * wp.0 + p[2].0) * scale * 0.5,
        (p[0].1 + 2.0 * wp.1 + p[2].1) * scale * 0.5,
    );
    let mut c = ((wp.0 + p[2].0) * scale, (wp.1 + p[2].1) * scale);
    let between = |a: f32, b: f32, c: f32| (a - b) * (c - b) <= 0.0;
    if between(p[0].1, p[1].1, p[2].1) {
        if !between(p[0].1, b.1, p[2].1) {
            b.1 = if (b.1 - p[0].1).abs() < (b.1 - p[2].1).abs() {
                p[0].1
            } else {
                p[2].1
            };
        }
        if !between(p[0].1, a.1, b.1) {
            a.1 = p[0].1;
        }
        if !between(b.1, c.1, p[2].1) {
            c.1 = p[2].1;
        }
    }
    let nw = (0.5 + 0.5 * w).sqrt();
    conic_quads([p[0], a, b], nw, depth - 1, out);
    conic_quads([b, c, p[2]], nw, depth - 1, out);
}

pub(crate) fn conic_to_quads(p: [Pt; 3], w: f32) -> Vec<[Pt; 3]> {
    let a = w - 1.0;
    let k = a / (4.0 * (2.0 + a));
    let x = k * (p[0].0 - 2.0 * p[1].0 + p[2].0);
    let y = k * (p[0].1 - 2.0 * p[1].1 + p[2].1);
    let mut error = (x * x + y * y).sqrt();
    let mut depth = 0;
    while depth < 5 && error > 0.25 {
        depth += 1;
        error *= 0.25;
    }
    let mut out = Vec::new();
    conic_quads(p, w, depth, &mut out);
    out
}
