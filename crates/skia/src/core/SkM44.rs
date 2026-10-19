//! Selected exact SkM44/SkMatrixInvert value primitives for no-pixels analysis.
#![allow(non_snake_case)]
use crate::include::core::SkRRect::SkRRectBounds;
// Source: include/core/SkM44.h:217-240. Native storage is column-major f32.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkM44 {
    pub values: [f32; 16],
}
impl Default for SkM44 {
    fn default() -> Self {
        Self {
            values: [
                1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
            ],
        }
    }
}
impl SkM44 {
    pub fn ColMajor(values: [f32; 16]) -> Self {
        Self { values }
    }
    pub fn rc(&self, r: usize, c: usize) -> f32 {
        self.values[4 * c + r]
    }
    // Source: src/core/SkM44.cpp:48-69. Keep source nested add order.
    pub fn setConcat(&mut self, a: &Self, b: &Self) {
        let mut result = [0.; 16];
        for col in 0..4 {
            for row in 0..4 {
                result[4 * col + row] = a.rc(row, 0) * b.rc(0, col)
                    + (a.rc(row, 1) * b.rc(1, col)
                        + (a.rc(row, 2) * b.rc(2, col) + a.rc(row, 3) * b.rc(3, col)));
            }
        }
        self.values = result;
    }
    pub fn preConcat(&mut self, b: &Self) {
        let a = *self;
        self.setConcat(&a, b)
    }
    // Source: src/core/SkM44.cpp:91-102,115-127.
    pub fn preTranslate(&mut self, x: f32, y: f32, z: f32) {
        for row in 0..4 {
            self.values[12 + row] = self.values[row] * x
                + (self.values[4 + row] * y + (self.values[8 + row] * z + self.values[12 + row]));
        }
    }
    // Source: src/core/SkM44.cpp:104-111.
    pub fn postTranslate(&mut self, x: f32, y: f32, z: f32) {
        let t = [x, y, z, 0.];
        for col in 0..4 {
            let w = self.values[4 * col + 3];
            for row in 0..4 {
                self.values[4 * col + row] = t[row] * w + self.values[4 * col + row];
            }
        }
    }
    pub fn preScale(&mut self, x: f32, y: f32, z: f32) {
        for row in 0..4 {
            self.values[row] *= x;
            self.values[4 + row] *= y;
            self.values[8 + row] *= z;
        }
    }
    // Source: src/core/SkM44.cpp:129-139.
    pub fn map(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        std::array::from_fn(|r| {
            self.rc(r, 0) * x + (self.rc(r, 1) * y + (self.rc(r, 2) * z + self.rc(r, 3) * w))
        })
    }
    // Source: src/core/SkM44.cpp:242-255.
    pub fn invert(&self) -> Option<Self> {
        inverse4(&self.values).map(Self::ColMajor)
    }
    // Source: include/core/SkM44.h:440-446. asM33 keeps the XYW entries.
    pub fn asM33Inverse(&self) -> Option<Self> {
        let m = &self.values;
        inverse3(&[m[0], m[4], m[12], m[1], m[5], m[13], m[3], m[7], m[15]]).map(|m| {
            Self::ColMajor([
                m[0], m[3], 0., m[6], m[1], m[4], 0., m[7], 0., 0., 1., 0., m[2], m[5], 0., m[8],
            ])
        })
    }
    pub fn isScaleTranslateAsM33(&self) -> bool {
        let m = &self.values;
        m[1] == 0. && m[4] == 0. && m[3] == 0. && m[7] == 0. && m[15] == 1.
    }
    // Source: src/core/SkM44.cpp:141-163,226-235.
    pub fn mapRect(&self, src: SkRRectBounds) -> SkRRectBounds {
        let m = &self.values;
        if m[3] != 0. || m[7] != 0. || m[11] != 0. || m[15] != 1. {
            return self.mapRectPerspective(src);
        }
        let corners = [
            (src.left, src.top),
            (src.right, src.top),
            (src.left, src.bottom),
            (src.right, src.bottom),
        ];
        let flip = [1., 1., -1., -1.];
        let v: [f32; 4] = std::array::from_fn(|i| {
            let row = i % 2;
            let f = flip[i];
            let c0 = m[row] * f;
            let c1 = m[4 + row] * f;
            let values = corners.map(|(x, y)| c0 * x + c1 * y);
            m[12 + row] + f * cpp_min(cpp_min(values[0], values[1]), cpp_min(values[2], values[3]))
        });
        SkRRectBounds::MakeLTRB(v[0], v[1], v[2], v[3])
    }
    // Source: src/core/SkM44.cpp:165-224; src/core/SkPathPriv.h:73.
    fn mapRectPerspective(&self, src: SkRRectBounds) -> SkRRectBounds {
        let corner = |x: f32, y: f32| {
            std::array::from_fn::<_, 4, _>(|r| {
                (self.rc(r, 0) * x + self.rc(r, 1) * y) + self.rc(r, 3)
            })
        };
        let tl = corner(src.left, src.top);
        let tr = corner(src.right, src.top);
        let bl = corner(src.left, src.bottom);
        let br = corner(src.right, src.bottom);
        const W: f32 = 1. / 16384.;
        let flip = [1., 1., -1., -1.];
        let project = |p0: [f32; 4], p1: [f32; 4], p2: [f32; 4]| -> [f32; 4] {
            let w0 = p0[3];
            if w0 >= W {
                return std::array::from_fn(|i| flip[i] * p0[i % 2] / w0);
            }
            let clip = |p: [f32; 4]| -> [f32; 4] {
                let w = p[3];
                if w >= W {
                    let t = (W - w0) / (w - w0);
                    let c = [
                        (t * p[0] + (1. - t) * p0[0]) / W,
                        (t * p[1] + (1. - t) * p0[1]) / W,
                    ];
                    std::array::from_fn(|i| flip[i] * c[i % 2])
                } else {
                    [f32::INFINITY; 4]
                }
            };
            let a = clip(p1);
            let b = clip(p2);
            std::array::from_fn(|i| cpp_min(a[i], b[i]))
        };
        let a = project(tl, tr, bl);
        let b = project(tr, br, tl);
        let c = project(br, bl, tr);
        let d = project(bl, tl, br);
        let v: [f32; 4] =
            std::array::from_fn(|i| flip[i] * cpp_min(cpp_min(a[i], b[i]), cpp_min(c[i], d[i])));
        SkRRectBounds::MakeLTRB(v[0], v[1], v[2], v[3])
    }
}
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
// Source: src/core/SkMatrix.cpp:712-755,788-881 and
// src/core/SkMatrixInvert.cpp:72-150. The two native inversions have
// distinct precision, cutoff and scalar fast-path semantics.
// Source: src/core/SkMatrix.cpp:254-256,712-755,788-881.
// asM33().invert() deliberately differs from full SkM44::invert(): scalar
// scale/translation fast paths and the general determinant cutoff are native.
fn inverse3(m: &[f32; 9]) -> Option<[f32; 9]> {
    let persp = m[6] != 0. || m[7] != 0. || m[8] != 1.;
    let affine = m[1] != 0. || m[3] != 0.;
    if !persp && !affine {
        if m[0] != 1. || m[4] != 1. {
            let sx = 1. / m[0];
            let sy = 1. / m[4];
            if !sx.is_finite() || !sy.is_finite() {
                return None;
            }
            let tx = -m[2] * sx;
            let ty = -m[5] * sy;
            if !tx.is_finite() || !ty.is_finite() {
                return None;
            }
            return Some([sx, 0., tx, 0., sy, ty, 0., 0., 1.]);
        }
        if !m[2].is_finite() || !m[5].is_finite() {
            return None;
        }
        return Some([1., 0., -m[2], 0., 1., -m[5], 0., 0., 1.]);
    }
    let d: [f64; 9] = m.map(|v| v as f64);
    let cross = |a: usize, b: usize, c: usize, e: usize| d[a] * d[b] - d[c] * d[e];
    let det = if persp {
        d[0] * cross(4, 8, 5, 7) + d[1] * cross(5, 6, 3, 8) + d[2] * cross(3, 7, 4, 6)
    } else {
        cross(0, 4, 1, 3)
    };
    let nearly_zero = 1. / 4096.;
    if (det as f32).abs() <= nearly_zero * nearly_zero * nearly_zero {
        return None;
    }
    let inv = 1. / det;
    let result = if persp {
        let scross = |a: usize, b: usize, c: usize, e: usize| {
            ((m[a] * m[b] - m[c] * m[e]) as f64 * inv) as f32
        };
        [
            scross(4, 8, 5, 7),
            scross(2, 7, 1, 8),
            scross(1, 5, 2, 4),
            scross(5, 6, 3, 8),
            scross(0, 8, 2, 6),
            scross(2, 3, 0, 5),
            scross(3, 7, 4, 6),
            scross(1, 6, 0, 7),
            scross(0, 4, 1, 3),
        ]
    } else {
        [
            (d[4] * inv) as f32,
            (-d[1] * inv) as f32,
            (cross(1, 5, 4, 2) * inv) as f32,
            (-d[3] * inv) as f32,
            (d[0] * inv) as f32,
            (cross(3, 2, 0, 5) * inv) as f32,
            0.,
            0.,
            1.,
        ]
    };
    result.iter().all(|v| v.is_finite()).then_some(result)
}
// Source: src/core/SkMatrixInvert.cpp:72-150.
fn inverse4(input: &[f32; 16]) -> Option<[f32; 16]> {
    let mut output = [0.; 16];
    let a00 = input[0] as f64;
    let a01 = input[1] as f64;
    let a02 = input[2] as f64;
    let a03 = input[3] as f64;
    let a10 = input[4] as f64;
    let a11 = input[5] as f64;
    let a12 = input[6] as f64;
    let a13 = input[7] as f64;
    let a20 = input[8] as f64;
    let a21 = input[9] as f64;
    let a22 = input[10] as f64;
    let a23 = input[11] as f64;
    let a30 = input[12] as f64;
    let a31 = input[13] as f64;
    let a32 = input[14] as f64;
    let a33 = input[15] as f64;

    let mut b00 = a00 * a11 - a01 * a10;
    let mut b01 = a00 * a12 - a02 * a10;
    let mut b02 = a00 * a13 - a03 * a10;
    let mut b03 = a01 * a12 - a02 * a11;
    let mut b04 = a01 * a13 - a03 * a11;
    let mut b05 = a02 * a13 - a03 * a12;
    let mut b06 = a20 * a31 - a21 * a30;
    let mut b07 = a20 * a32 - a22 * a30;
    let mut b08 = a20 * a33 - a23 * a30;
    let mut b09 = a21 * a32 - a22 * a31;
    let mut b10 = a21 * a33 - a23 * a31;
    let mut b11 = a22 * a33 - a23 * a32;

    // Calculate the determinant
    let mut determinant = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    {
        let invdet = 1.0 / determinant;
        b00 *= invdet;
        b01 *= invdet;
        b02 *= invdet;
        b03 *= invdet;
        b04 *= invdet;
        b05 *= invdet;
        b06 *= invdet;
        b07 *= invdet;
        b08 *= invdet;
        b09 *= invdet;
        b10 *= invdet;
        b11 *= invdet;

        output[0] = (a11 * b11 - a12 * b10 + a13 * b09) as f32;
        output[1] = (a02 * b10 - a01 * b11 - a03 * b09) as f32;
        output[2] = (a31 * b05 - a32 * b04 + a33 * b03) as f32;
        output[3] = (a22 * b04 - a21 * b05 - a23 * b03) as f32;
        output[4] = (a12 * b08 - a10 * b11 - a13 * b07) as f32;
        output[5] = (a00 * b11 - a02 * b08 + a03 * b07) as f32;
        output[6] = (a32 * b02 - a30 * b05 - a33 * b01) as f32;
        output[7] = (a20 * b05 - a22 * b02 + a23 * b01) as f32;
        output[8] = (a10 * b10 - a11 * b08 + a13 * b06) as f32;
        output[9] = (a01 * b08 - a00 * b10 - a03 * b06) as f32;
        output[10] = (a30 * b04 - a31 * b02 + a33 * b00) as f32;
        output[11] = (a21 * b02 - a20 * b04 - a23 * b00) as f32;
        output[12] = (a11 * b07 - a10 * b09 - a12 * b06) as f32;
        output[13] = (a00 * b09 - a01 * b07 + a02 * b06) as f32;
        output[14] = (a31 * b01 - a30 * b03 - a32 * b00) as f32;
        output[15] = (a20 * b03 - a21 * b01 + a22 * b00) as f32;

        // If 1/det overflows to infinity (i.e. det is denormalized) or any of the inverted matrix
        // values is non-finite, return zero to indicate a non-invertible matrix.
        if !output.iter().all(|v| v.is_finite()) {
            determinant = 0.0;
        }
    }
    if determinant as f32 == 0. {
        None
    } else {
        Some(output)
    }
}
