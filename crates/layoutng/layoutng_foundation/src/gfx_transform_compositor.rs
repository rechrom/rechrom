// Copyright 2026 The Chromium Authors. BSD-style license.
// Pure gfx values required by canonical cc. This extends the existing native
// AxisTransform2d/Matrix44 representation; no compositor owner or policy lives
// here. Relative Source paths refer to the pinned Chromium checkout.

// Source: ui/gfx/geometry/double4.h:55-70
#[derive(Clone, Copy)]
struct GfxDouble4([f64; 4]);
impl std::ops::Add for GfxDouble4 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] + b.0[i]))
    }
}
impl std::ops::Sub for GfxDouble4 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] - b.0[i]))
    }
}
impl std::ops::Mul for GfxDouble4 {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        Self(std::array::from_fn(|i| self.0[i] * b.0[i]))
    }
}
impl std::ops::Mul<f64> for GfxDouble4 {
    type Output = Self;
    fn mul(self, b: f64) -> Self {
        Self(self.0.map(|v| v * b))
    }
}
impl std::ops::AddAssign for GfxDouble4 {
    fn add_assign(&mut self, b: Self) {
        *self = *self + b;
    }
}
impl std::ops::SubAssign for GfxDouble4 {
    fn sub_assign(&mut self, b: Self) {
        *self = *self - b;
    }
}
impl GfxDouble4 {
    fn sum(self) -> f64 {
        self.0[0] + self.0[1] + self.0[2] + self.0[3]
    }
}
// Source: ui/gfx/geometry/matrix44.cc:21-27
fn gfx_swap_high_low(v: GfxDouble4) -> GfxDouble4 {
    GfxDouble4([v.0[2], v.0[3], v.0[0], v.0[1]])
}
fn gfx_swap_in_pairs(v: GfxDouble4) -> GfxDouble4 {
    GfxDouble4([v.0[1], v.0[0], v.0[3], v.0[2]])
}
// Source: ui/gfx/geometry/matrix44.cc:29-106
// Lane operations preserve the native Double4 arithmetic/control-flow sequence;
// this is not an alternative Gaussian-elimination/inversion algorithm.
fn gfx_inverse_double4_cols(matrix: [[f64; 4]; 4]) -> Option<[[f64; 4]; 4]> {
    let [a, b, c, d] = matrix;
    let r0 = GfxDouble4([a[0], b[0], c[0], d[0]]);
    let r1 = GfxDouble4([c[1], d[1], a[1], b[1]]);
    let mut r2 = GfxDouble4([a[2], b[2], c[2], d[2]]);
    let r3 = GfxDouble4([c[3], d[3], a[3], b[3]]);
    let mut t = gfx_swap_in_pairs(r2 * r3);
    let mut c0 = r1 * t;
    let mut c1 = r0 * t;
    t = gfx_swap_high_low(t);
    c0 = r1 * t - c0;
    c1 = gfx_swap_high_low(r0 * t - c1);
    t = gfx_swap_in_pairs(r1 * r2);
    c0 += r3 * t;
    let mut c3 = r0 * t;
    t = gfx_swap_high_low(t);
    c0 -= r3 * t;
    c3 = gfx_swap_high_low(r0 * t - c3);
    t = gfx_swap_in_pairs(gfx_swap_high_low(r1) * r3);
    r2 = gfx_swap_high_low(r2);
    c0 += r2 * t;
    let mut c2 = r0 * t;
    t = gfx_swap_high_low(t);
    c0 -= r2 * t;
    let det = (r0 * c0).sum();
    if !(det as f32).is_normal() {
        return None;
    }
    c2 = gfx_swap_high_low(r0 * t - c2);
    t = gfx_swap_in_pairs(r0 * r1);
    c2 = r3 * t + c2;
    c3 = r2 * t - c3;
    t = gfx_swap_high_low(t);
    c2 = r3 * t - c2;
    c3 -= r2 * t;
    t = gfx_swap_in_pairs(r0 * r3);
    c1 -= r2 * t;
    c2 = r1 * t + c2;
    t = gfx_swap_high_low(t);
    c1 = r2 * t + c1;
    c2 -= r1 * t;
    t = gfx_swap_in_pairs(r0 * r2);
    c1 = r3 * t + c1;
    c3 -= r1 * t;
    t = gfx_swap_high_low(t);
    c1 -= r3 * t;
    c3 = r1 * t + c3;
    let det = 1. / det;
    Some([(c0 * det).0, (c1 * det).0, (c2 * det).0, (c3 * det).0])
}
// Source: ui/gfx/geometry/matrix44.cc:370-403
fn gfx_matrix_determinant(matrix: [[f64; 4]; 4]) -> f64 {
    if gfx_matrix_is_flat(&matrix) && !gfx_matrix_has_perspective(&matrix) {
        return matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
    }
    let [a, b, c, d] = matrix;
    let r0 = GfxDouble4([a[0], b[0], c[0], d[0]]);
    let r1 = GfxDouble4([c[1], d[1], a[1], b[1]]);
    let mut r2 = GfxDouble4([a[2], b[2], c[2], d[2]]);
    let r3 = GfxDouble4([c[3], d[3], a[3], b[3]]);
    let mut t = gfx_swap_in_pairs(r2 * r3);
    let mut c0 = r1 * t;
    t = gfx_swap_high_low(t);
    c0 = r1 * t - c0;
    t = gfx_swap_in_pairs(r1 * r2);
    c0 += r3 * t;
    t = gfx_swap_high_low(t);
    c0 -= r3 * t;
    t = gfx_swap_in_pairs(gfx_swap_high_low(r1) * r3);
    r2 = gfx_swap_high_low(r2);
    c0 += r2 * t;
    t = gfx_swap_high_low(t);
    c0 -= r2 * t;
    (r0 * c0).sum()
}
// Source: ui/gfx/geometry/matrix44.h:99-110
fn gfx_matrix_is_flat(m: &[[f64; 4]; 4]) -> bool {
    m[2] == [0., 0., 1., 0.] && m[0][2] == 0. && m[1][2] == 0. && m[3][2] == 0.
}
fn gfx_matrix_has_perspective(m: &[[f64; 4]; 4]) -> bool {
    [m[0][3], m[1][3], m[2][3], m[3][3]] != [0., 0., 0., 1.]
}
// Source: ui/gfx/geometry/matrix44.cc:432-440
fn gfx_map_vector4(m: &[[f64; 4]; 4], v: [f64; 4]) -> [f64; 4] {
    std::array::from_fn(|r| {
        (GfxDouble4([m[0][r], m[1][r], m[2][r], m[3][r]]) * GfxDouble4(v)).sum()
    })
}

#[allow(non_snake_case)]
impl Transform {
    // Source: ui/gfx/geometry/transform.h:179-186
    pub fn rc(&self, row: usize, column: usize) -> f64 {
        assert!(row < 4 && column < 4);
        match &self.representation {
            Representation::Matrix44(m) => m[column][row],
            Representation::Axis2d { scale, translation } => match (row, column) {
                (0, 0) => scale[0] as f64,
                (1, 1) => scale[1] as f64,
                (2, 2) | (3, 3) => 1.,
                (0, 3) => translation[0] as f64,
                (1, 3) => translation[1] as f64,
                _ => 0.,
            },
        }
    }
    // Source: ui/gfx/geometry/transform.h:254-260
    pub fn IsIdentity(&self) -> bool {
        *self == Self::default()
    }
    // Source: ui/gfx/geometry/transform.h:284-293,313-328
    pub fn IsScaleOrTranslation(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { .. } => true,
            Representation::Matrix44(m) => {
                [m[0][1], m[0][2], m[0][3], m[1][0]] == [0.; 4]
                    && [m[1][2], m[1][3], m[2][0], m[2][1]] == [0.; 4]
                    && m[2][3] == 0.
                    && m[3][3] == 1.
            }
        }
    }
    pub fn IsPositiveScaleOrTranslation(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { scale, .. } => scale[0] > 0. && scale[1] > 0.,
            Representation::Matrix44(m) => {
                self.IsScaleOrTranslation() && m[0][0] > 0. && m[1][1] > 0. && m[2][2] > 0.
            }
        }
    }
    pub fn IsScale2d(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { translation, .. } => *translation == [0.; 2],
            Representation::Matrix44(m) => {
                self.IsScaleOrTranslation() && m[3] == [0., 0., 0., 1.] && m[2][2] == 1.
            }
        }
    }
    // Source: ui/gfx/geometry/transform.cc:328-376
    pub fn IsApproximatelyIdentityOrTranslation(&self, tolerance: f64) -> bool {
        debug_assert!(tolerance >= 0.);
        if let Representation::Axis2d { scale, .. } = &self.representation {
            return ((scale[0] as f64) - 1.).abs() <= tolerance
                && ((scale[1] as f64) - 1.).abs() <= tolerance;
        }
        for column in 0..3 {
            for row in 0..3 {
                if !((self.rc(row, column) - if row == column { 1. } else { 0. }).abs()
                    <= tolerance)
                {
                    return false;
                }
            }
        }
        let tolerance = (f32::EPSILON as f64).min(tolerance);
        (0..3).all(|column| self.rc(3, column).abs() <= tolerance)
            && (self.rc(3, 3) - 1.).abs() <= tolerance
    }
    pub fn IsApproximatelyIdentityOrIntegerTranslation(&self, tolerance: f64) -> bool {
        if !self.IsApproximatelyIdentityOrTranslation(tolerance) {
            return false;
        }
        match &self.representation {
            Representation::Axis2d { translation, .. } => translation.iter().all(|&v| {
                (v as f64) >= i32::MIN as f64
                    && (v as f64) <= i32::MAX as f64
                    && ((v.round() - v) as f64).abs() <= tolerance
            }),
            Representation::Matrix44(m) => m[3][..3].iter().all(|&v| {
                v >= i32::MIN as f64 && v <= i32::MAX as f64 && (v.round() - v).abs() <= tolerance
            }),
        }
    }
    // Source: ui/gfx/geometry/transform.cc:391-412
    pub fn IsIdentityOrIntegerTranslation(&self) -> bool {
        if !self.IsIdentityOrTranslation() {
            return false;
        }
        match &self.representation {
            Representation::Axis2d { translation, .. } => translation.iter().all(|&v| {
                (v as f64) >= i32::MIN as f64
                    && (v as f64) <= i32::MAX as f64
                    && v as i32 as f32 == v
            }),
            Representation::Matrix44(m) => m[3][..3]
                .iter()
                .all(|&v| v >= i32::MIN as f64 && v <= i32::MAX as f64 && v as i32 as f64 == v),
        }
    }
    // Source: ui/gfx/geometry/transform.cc:479-515
    // Source: ui/gfx/geometry/axis_transform2d.h:73-86
    // Source: ui/gfx/geometry/matrix44.cc:329-368
    pub fn GetInverse(&self, result: &mut Self) -> bool {
        match &self.representation {
            Representation::Axis2d { scale, translation } => {
                if (scale[0] * scale[1]).is_normal() {
                    let inverse = [1. / scale[0], 1. / scale[1]];
                    result.representation = Representation::Axis2d {
                        scale: inverse,
                        translation: [translation[0] * -inverse[0], translation[1] * -inverse[1]],
                    };
                    return true;
                }
            }
            Representation::Matrix44(m) => {
                let inverse = if gfx_matrix_is_flat(m) && !gfx_matrix_has_perspective(m) {
                    let determinant = gfx_matrix_determinant(*m);
                    if !(determinant as f32).is_normal() {
                        None
                    } else {
                        let inv_det = 1. / determinant;
                        let a = m[0][0];
                        let b = m[0][1];
                        let c = m[1][0];
                        let d = m[1][1];
                        let e = m[3][0];
                        let f = m[3][1];
                        Some([
                            [d * inv_det, -b * inv_det, 0., 0.],
                            [-c * inv_det, a * inv_det, 0., 0.],
                            [0., 0., 1., 0.],
                            [(c * f - d * e) * inv_det, (b * e - a * f) * inv_det, 0., 1.],
                        ])
                    }
                } else {
                    gfx_inverse_double4_cols(*m)
                };
                if let Some(m) = inverse {
                    result.representation = Representation::Matrix44(m);
                    return true;
                }
            }
        }
        result.MakeIdentity();
        false
    }
    pub fn GetCheckedInverse(&self) -> Self {
        let mut result = Self::default();
        let invertible = self.GetInverse(&mut result);
        debug_assert!(invertible, "gfx::Transform is not invertible");
        result
    }
    pub fn InverseOrIdentity(&self) -> Self {
        let mut result = Self::default();
        let invertible = self.GetInverse(&mut result);
        debug_assert!(invertible || result.IsIdentity());
        result
    }
    // Source: ui/gfx/geometry/transform.cc:180-188
    pub fn IsInvertible(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { scale, .. } => (scale[0] * scale[1]).is_normal(),
            Representation::Matrix44(m) => (gfx_matrix_determinant(*m) as f32).is_normal(),
        }
    }
    // Source: ui/gfx/geometry/transform.cc:424-477
    pub fn IsBackFaceVisible(&self) -> bool {
        let Representation::Matrix44(m) = &self.representation else {
            return false;
        };
        let determinant = gfx_matrix_determinant(*m);
        if determinant == 0. {
            return false;
        }
        let p1 = m[0][0] * m[1][1] * m[3][3];
        let p2 = m[1][0] * m[3][1] * m[0][3];
        let p3 = m[3][0] * m[0][1] * m[1][3];
        let p4 = m[0][0] * m[3][1] * m[1][3];
        let p5 = m[1][0] * m[0][1] * m[3][3];
        let p6 = m[3][0] * m[1][1] * m[0][3];
        (p1 + p2 + p3 - p4 - p5 - p6) * determinant < -(f32::EPSILON as f64)
    }
    // Source: ui/gfx/geometry/transform.cc:604-624
    pub fn NonDegeneratePreserves2dAxisAlignment(&self) -> bool {
        let e = f32::EPSILON as f64;
        if let Representation::Axis2d { scale, .. } = &self.representation {
            return (scale[0] as f64) > e && (scale[1] as f64) > e;
        }
        let m = self.matrix();
        let perspective = m[0][3] != 0. || m[1][3] != 0.;
        let positive_w = m[3][3] > e;
        let a = m[0][0].abs() > e;
        let b = m[1][0].abs() > e;
        let c = m[0][1].abs() > e;
        let d = m[1][1].abs() > e;
        a == d && b == c && a != b && !perspective && positive_w
    }
    // Source: ui/gfx/geometry/transform.cc:646-665
    pub fn Flatten(&mut self) {
        if let Representation::Matrix44(m) = &mut self.representation {
            m[0][2] = 0.;
            m[1][2] = 0.;
            m[3][2] = 0.;
            m[2] = [0., 0., 1., 0.];
        }
        debug_assert!(self.IsFlat());
    }
    pub fn IsFlat(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { .. } => true,
            Representation::Matrix44(m) => gfx_matrix_is_flat(m),
        }
    }
    pub fn HasPerspective(&self) -> bool {
        match &self.representation {
            Representation::Axis2d { .. } => false,
            Representation::Matrix44(m) => gfx_matrix_has_perspective(m),
        }
    }
    // Source: ui/gfx/geometry/transform.cc:667-693
    pub fn To2dTranslation(&self) -> foundation_base::gfx_geometry::Vector2dF {
        foundation_base::gfx_geometry::Vector2dF::new(
            Self::clamp_float_geometry(self.rc(0, 3)),
            Self::clamp_float_geometry(self.rc(1, 3)),
        )
    }
    pub fn To2dScale(&self) -> foundation_base::gfx_geometry::Vector2dF {
        foundation_base::gfx_geometry::Vector2dF::new(
            Self::clamp_float_geometry(self.rc(0, 0)),
            Self::clamp_float_geometry(self.rc(1, 1)),
        )
    }
    // Source: ui/gfx/geometry/transform.cc:699-704
    pub fn MapPoint(
        &self,
        point: foundation_base::gfx_geometry::PointF,
    ) -> foundation_base::gfx_geometry::PointF {
        self.map_point(point)
    }
    // Source: ui/gfx/geometry/transform.cc:706-713,1059-1077
    // Three coordinates are the external gfx Point3F field boundary only.
    pub fn MapPoint3d(&self, point: [f32; 3]) -> [f32; 3] {
        if matches!(self.representation, Representation::Axis2d { .. }) {
            let p = self.MapPoint(foundation_base::gfx_geometry::PointF::new(
                point[0], point[1],
            ));
            return [p.x(), p.y(), point[2]];
        }
        let p = gfx_map_vector4(
            &self.matrix(),
            [point[0] as f64, point[1] as f64, point[2] as f64, 1.],
        );
        let inverse_w = if p[3] != 1. && p[3].is_normal() {
            1. / p[3]
        } else {
            1.
        };
        std::array::from_fn(|i| Self::clamp_float_geometry(p[i] * inverse_w))
    }
    // Source: ui/gfx/geometry/transform.cc:726-741
    pub fn TransformVector4(&self, vector: &mut [f32; 4]) {
        match &self.representation {
            Representation::Axis2d { scale, translation } => {
                vector[0] = vector[0] * scale[0] + vector[3] * translation[0];
                vector[1] = vector[1] * scale[1] + vector[3] * translation[1];
                for v in vector {
                    *v = Self::clamp_float_geometry(*v as f64);
                }
            }
            Representation::Matrix44(m) => {
                let v = gfx_map_vector4(m, vector.map(f64::from));
                for i in 0..4 {
                    vector[i] = Self::clamp_float_geometry(v[i]);
                }
            }
        }
    }
    // Source: ui/gfx/geometry/transform.cc:193-203
    pub fn Scale(&mut self, x: f32, y: f32) {
        self.Scale3d(x, y, 1.);
    }
    pub fn PostScale(&mut self, x: f32, y: f32) {
        match &mut self.representation {
            Representation::Axis2d { scale, translation } => {
                scale[0] *= x;
                scale[1] *= y;
                translation[0] *= x;
                translation[1] *= y;
            }
            Representation::Matrix44(m) => {
                if x != 1. {
                    for col in m.iter_mut() {
                        col[0] *= x as f64;
                    }
                }
                if y != 1. {
                    for col in m.iter_mut() {
                        col[1] *= y as f64;
                    }
                }
            }
        }
    }
    // Source: ui/gfx/geometry/transform.h:214; transform.cc:220-228
    pub fn Translate(&mut self, x: f32, y: f32) {
        self.Translate3d(x, y, 0.);
    }
    // Source: ui/gfx/geometry/transform.cc:290-300,319-325
    pub fn PostConcat(&mut self, other: &Self) {
        if let Representation::Axis2d { scale, translation } = &other.representation {
            self.PostScale(scale[0], scale[1]);
            self.PostTranslate(translation[0], translation[1]);
        } else if matches!(self.representation, Representation::Axis2d { .. }) {
            let old = self.clone();
            *self = other.clone();
            self.PreConcat(&old);
        } else {
            self.representation =
                Representation::Matrix44(Self::multiply_matrices(other.matrix(), self.matrix()));
        }
    }
    // Source: ui/gfx/geometry/transform.cc:1008-1017,1030-1041
    pub fn Round2dTranslationComponents(&mut self) {
        match &mut self.representation {
            Representation::Axis2d { translation, .. } => {
                translation[0] = translation[0].round();
                translation[1] = translation[1].round();
            }
            Representation::Matrix44(m) => {
                m[3][0] = m[3][0].round();
                m[3][1] = m[3][1].round();
            }
        }
    }
    pub fn RoundToIdentityOrIntegerTranslation(&mut self) {
        match &mut self.representation {
            Representation::Axis2d { scale, translation } => {
                *scale = [1.; 2];
                translation[0] = translation[0].round();
                translation[1] = translation[1].round();
            }
            Representation::Matrix44(m) => {
                *m = [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [m[3][0].round(), m[3][1].round(), m[3][2].round(), 1.],
                ];
            }
        }
    }
    // Source: ui/gfx/geometry/transform_util.cc:124-179
    pub fn TryComputeTransform2dScaleComponents(
        &self,
    ) -> Option<foundation_base::gfx_geometry::Vector2dF> {
        if self.rc(3, 0) != 0. || self.rc(3, 1) != 0. {
            return None;
        }
        let w = self.rc(3, 3) as f32;
        if !w.is_normal() {
            return None;
        }
        let w_scale = 1. / w;
        let scale_axis = |a: f64, b: f64, c: f64| {
            let zero = |v: f64| v.abs() < f64::EPSILON;
            if zero(b) && zero(c) {
                Self::clamp_float_geometry(a.abs())
            } else if zero(a) && zero(c) {
                Self::clamp_float_geometry(b.abs())
            } else if zero(a) && zero(b) {
                Self::clamp_float_geometry(c.abs())
            } else {
                Self::clamp_float_geometry((a * a + b * b + c * c).sqrt())
            }
        };
        let x = scale_axis(self.rc(0, 0), self.rc(1, 0), self.rc(2, 0));
        let y = scale_axis(self.rc(0, 1), self.rc(1, 1), self.rc(2, 1));
        Some(foundation_base::gfx_geometry::Vector2dF::new(
            Self::clamp_float_geometry((x * w_scale) as f64),
            Self::clamp_float_geometry((y * w_scale) as f64),
        ))
    }
    pub fn ComputeTransform2dScaleComponents(
        &self,
        fallback: f32,
    ) -> foundation_base::gfx_geometry::Vector2dF {
        self.TryComputeTransform2dScaleComponents()
            .unwrap_or_else(|| foundation_base::gfx_geometry::Vector2dF::new(fallback, fallback))
    }
}

#[allow(non_snake_case)]
impl Transform {
    // Source: ui/gfx/geometry/transform.cc:1077-1131;
    //         ui/gfx/geometry/transform.h:574-577.
    pub fn ApproximatelyEqual(
        &self,
        other: &Self,
        abs_translation_tolerance: f32,
        abs_other_tolerance: f32,
        rel_scale_tolerance: f32,
    ) -> bool {
        if self == other {
            return true;
        }
        if abs_translation_tolerance == 0. && abs_other_tolerance == 0. {
            return false;
        }
        let approximately_equal = |a: f32, b: f32| (a - b).abs() <= abs_other_tolerance;
        let translation_approximately_equal =
            |a: f32, b: f32| (a - b).abs() <= abs_translation_tolerance;
        let scale_approximately_equal = |a: f32, b: f32| {
            let diff = (a - b).abs();
            diff <= abs_other_tolerance
                && (rel_scale_tolerance == 0. || diff <= (a.abs() + b.abs()) * rel_scale_tolerance)
        };
        if let (
            Representation::Axis2d {
                scale: a,
                translation: at,
            },
            Representation::Axis2d {
                scale: b,
                translation: bt,
            },
        ) = (&self.representation, &other.representation)
        {
            return scale_approximately_equal(a[0], b[0])
                && scale_approximately_equal(a[1], b[1])
                && translation_approximately_equal(at[0], bt[0])
                && translation_approximately_equal(at[1], bt[1]);
        }
        for row in 0..4 {
            for col in 0..4 {
                let x = self.rc(row, col) as f32;
                let y = other.rc(row, col) as f32;
                if row < 3 && col == 3 {
                    if !translation_approximately_equal(x, y) {
                        return false;
                    }
                } else if row < 3 && col == row {
                    if !scale_approximately_equal(x, y) {
                        return false;
                    }
                } else if !approximately_equal(x, y) {
                    return false;
                }
            }
        }
        true
    }
}
