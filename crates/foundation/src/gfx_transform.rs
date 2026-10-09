// This is the portion of gfx::Transform used by the block-layout style
// boundary. The source chooses float AxisTransform2d storage for 2D scale and
// translation and promotes to a double Matrix44 for 3D transforms.

// cpp: foundation/gfx_geometry/transform.h:45-57,60-65
#[derive(Clone, Debug)]
pub struct Transform {
    representation: Representation,
}

// cpp: foundation/gfx_geometry/transform.h:137-145
impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        match (&self.representation, &other.representation) {
            (Representation::Axis2d { .. }, Representation::Axis2d { .. })
            | (Representation::Matrix44(_), Representation::Matrix44(_)) => {
                self.representation == other.representation
            }
            _ => self.matrix() == other.matrix(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Representation {
    Axis2d {
        scale: [f32; 2],
        translation: [f32; 2],
    },
    Matrix44([[f64; 4]; 4]), // column-major, then row
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            representation: Representation::Axis2d {
                scale: [1.0, 1.0],
                translation: [0.0, 0.0],
            },
        }
    }
}

#[allow(non_snake_case)]
impl Transform {
    // cpp: foundation/gfx_geometry/transform.h:263-268
    // cpp: foundation/gfx_geometry/matrix44.h:79-84
    pub fn IsIdentityOrTranslation(&self) -> bool {
        let m = self.matrix();
        m[0] == [1.0, 0.0, 0.0, 0.0]
            && m[1] == [0.0, 1.0, 0.0, 0.0]
            && m[2] == [0.0, 0.0, 1.0, 0.0]
            && m[3][3] == 1.0
    }

    // cpp: foundation/gfx_geometry/transform.cc:549-602
    pub fn Preserves2dAxisAlignment(&self) -> bool {
        if matches!(self.representation, Representation::Axis2d { .. }) {
            return true;
        }
        let m = self.matrix();
        let epsilon = f64::from(f32::EPSILON);
        let nonzero = |column: usize, row: usize| m[column][row].abs() > epsilon;
        (nonzero(0, 0) as u8 + nonzero(1, 0) as u8) <= 1
            && (nonzero(0, 1) as u8 + nonzero(1, 1) as u8) <= 1
            && (nonzero(0, 0) as u8 + nonzero(0, 1) as u8) <= 1
            && (nonzero(1, 0) as u8 + nonzero(1, 1) as u8) <= 1
            && m[0][3] == 0.0
            && m[1][3] == 0.0
    }

    fn clamp_float_geometry(value: f64) -> f32 {
        if value.is_nan() {
            return 0.0;
        }
        let limit = f64::from(f32::MAX / 1_000_000.0_f32);
        value.clamp(-limit, limit) as f32
    }

    // cpp: foundation/gfx_geometry/transform.h:133-136
    pub fn MakeIdentity(&mut self) {
        *self = Self::default();
    }

    fn multiply_matrices(left: [[f64; 4]; 4], right: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
        let mut result = [[0.0; 4]; 4];
        for column in 0..4 {
            for row in 0..4 {
                for component in 0..4 {
                    result[column][row] += left[component][row] * right[column][component];
                }
            }
        }
        result
    }

    // cpp: foundation/gfx_geometry/transform.cc:278-329
    pub fn PreConcat(&mut self, other: &Self) {
        if let (
            Representation::Axis2d { scale, translation },
            Representation::Axis2d {
                scale: other_scale,
                translation: other_translation,
            },
        ) = (&mut self.representation, &other.representation)
        {
            translation[0] += scale[0] * other_translation[0];
            translation[1] += scale[1] * other_translation[1];
            scale[0] *= other_scale[0];
            scale[1] *= other_scale[1];
        } else {
            self.representation =
                Representation::Matrix44(Self::multiply_matrices(self.matrix(), other.matrix()));
        }
    }

    // cpp: foundation/gfx_geometry/transform.cc:231-241
    pub fn PostTranslate(&mut self, x: f32, y: f32) {
        match &mut self.representation {
            Representation::Axis2d { translation, .. } => {
                translation[0] += x;
                translation[1] += y;
            }
            Representation::Matrix44(matrix) => {
                for column in matrix.iter_mut() {
                    column[0] += f64::from(x) * column[3];
                    column[1] += f64::from(y) * column[3];
                }
            }
        }
    }

    // cpp: foundation/gfx_geometry/transform.cc:243-250
    fn post_translate_3d(&mut self, x: f32, y: f32, z: f32) {
        if z == 0.0 {
            self.PostTranslate(x, y);
            return;
        }
        let mut matrix = self.matrix();
        for column in &mut matrix {
            column[0] += f64::from(x) * column[3];
            column[1] += f64::from(y) * column[3];
            column[2] += f64::from(z) * column[3];
        }
        self.representation = Representation::Matrix44(matrix);
    }

    // cpp: ui/gfx/geometry/transform.cc:265-269; matrix44.cc:268-273.
    pub fn Skew(&mut self, degrees_x: f64, degrees_y: f64) {
        if degrees_x == 0.0 && degrees_y == 0.0 {
            return;
        }
        let mut matrix = self.matrix();
        let c0 = matrix[0];
        let c1 = matrix[1];
        let x = degrees_x.to_radians().tan();
        let y = degrees_y.to_radians().tan();
        for row in 0..4 {
            matrix[0][row] = c0[row] + c1[row] * y;
            matrix[1][row] = c1[row] + c0[row] * x;
        }
        self.representation = Representation::Matrix44(matrix);
    }
    // cpp: ui/gfx/geometry/transform.cc:638-644; matrix44.cc:412-430.
    pub fn Zoom(&mut self, zoom: f32) {
        match &mut self.representation {
            Representation::Axis2d { translation, .. } => {
                translation[0] *= zoom;
                translation[1] *= zoom;
            }
            Representation::Matrix44(matrix) => {
                let zoom = f64::from(zoom);
                for index in 0..3 {
                    matrix[index][3] /= zoom;
                    matrix[3][index] *= zoom;
                }
            }
        }
    }

    // cpp: foundation/gfx_geometry/transform.cc:271-275
    pub fn ApplyPerspectiveDepth(&mut self, depth: f64) {
        if depth == 0.0 {
            return;
        }
        let mut matrix = self.matrix();
        for row in 0..4 {
            matrix[2][row] -= matrix[3][row] / depth;
        }
        self.representation = Representation::Matrix44(matrix);
    }

    // cpp: foundation/gfx_geometry/transform.cc:633-636
    pub fn ApplyTransformOrigin(&mut self, x: f32, y: f32, z: f32) {
        self.post_translate_3d(x, y, z);
        self.Translate3d(-x, -y, -z);
    }

    // cpp: foundation/gfx_geometry/transform.h:169
    // cpp: foundation/gfx_geometry/transform.cc:106-110
    pub fn ColMajor(values: &[f64; 16]) -> Self {
        let mut matrix = [[0.0; 4]; 4];
        for column in 0..4 {
            for row in 0..4 {
                matrix[column][row] = values[column * 4 + row];
            }
        }
        Self {
            representation: Representation::Matrix44(matrix),
        }
    }

    // cpp: foundation/gfx_geometry/transform.h:177
    // cpp: foundation/gfx_geometry/transform.cc:123-130
    pub fn GetColMajor(&self) -> [f64; 16] {
        let mut result = [0.0; 16];
        let matrix = self.matrix();
        for column in 0..4 {
            for row in 0..4 {
                result[column * 4 + row] = matrix[column][row];
            }
        }
        result
    }

    fn matrix(&self) -> [[f64; 4]; 4] {
        match &self.representation {
            Representation::Matrix44(matrix) => *matrix,
            Representation::Axis2d { scale, translation } => [
                [f64::from(scale[0]), 0.0, 0.0, 0.0],
                [0.0, f64::from(scale[1]), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    f64::from(translation[0]),
                    f64::from(translation[1]),
                    0.0,
                    1.0,
                ],
            ],
        }
    }

    // cpp: foundation/gfx_geometry/transform.cc:698-704,1043-1057
    fn map_point(
        &self,
        point: foundation_base::gfx_geometry::PointF,
    ) -> foundation_base::gfx_geometry::PointF {
        use foundation_base::gfx_geometry::PointF;
        match &self.representation {
            Representation::Axis2d { scale, translation } => PointF::new(
                Self::clamp_float_geometry(f64::from(point.x() * scale[0] + translation[0])),
                Self::clamp_float_geometry(f64::from(point.y() * scale[1] + translation[1])),
            ),
            Representation::Matrix44(matrix) => {
                let x = f64::from(point.x());
                let y = f64::from(point.y());
                let mapped_x = matrix[0][0] * x + matrix[1][0] * y + matrix[3][0];
                let mapped_y = matrix[0][1] * x + matrix[1][1] * y + matrix[3][1];
                let w = matrix[0][3] * x + matrix[1][3] * y + matrix[3][3];
                let divisor = if w != 1.0 && w.is_normal() { w } else { 1.0 };
                PointF::new(
                    Self::clamp_float_geometry(mapped_x / divisor),
                    Self::clamp_float_geometry(mapped_y / divisor),
                )
            }
        }
    }

    // cpp: foundation/gfx_geometry/transform.cc:775-802,858-862
    pub fn MapRect(&self, rect: crate::gfx_ext::RectF) -> crate::gfx_ext::RectF {
        use foundation_base::gfx_geometry::{PointF, SizeF};
        if self == &Self::default() {
            return rect;
        }
        if let Representation::Axis2d { scale, .. } = &self.representation {
            if scale[0] >= 0.0 && scale[1] >= 0.0 {
                let origin = self.map_point(PointF::new(rect.x(), rect.y()));
                return crate::gfx_ext::RectF::new(
                    origin,
                    SizeF::new(
                        Self::clamp_float_geometry(f64::from(rect.width() * scale[0])),
                        Self::clamp_float_geometry(f64::from(rect.height() * scale[1])),
                    ),
                );
            }
        }
        if let Representation::Matrix44(matrix) = &self.representation {
            let is_scale_or_translation = matrix[0][1] == 0.0
                && matrix[0][2] == 0.0
                && matrix[0][3] == 0.0
                && matrix[1][0] == 0.0
                && matrix[1][2] == 0.0
                && matrix[1][3] == 0.0
                && matrix[2][0] == 0.0
                && matrix[2][1] == 0.0
                && matrix[2][3] == 0.0
                && matrix[3][3] == 1.0;
            if is_scale_or_translation && matrix[0][0] >= 0.0 && matrix[1][1] >= 0.0 {
                let x =
                    Self::clamp_float_geometry(f64::from(rect.x()) * matrix[0][0] + matrix[3][0]);
                let y =
                    Self::clamp_float_geometry(f64::from(rect.y()) * matrix[1][1] + matrix[3][1]);
                return crate::gfx_ext::RectF::new(
                    PointF::new(x, y),
                    SizeF::new(
                        Self::clamp_float_geometry(
                            f64::from(rect.right()) * matrix[0][0] + matrix[3][0],
                        ) - x,
                        Self::clamp_float_geometry(
                            f64::from(rect.bottom()) * matrix[1][1] + matrix[3][1],
                        ) - y,
                    ),
                );
            }
        }
        let quad = crate::gfx_quad_f::QuadF::from(rect);
        crate::gfx_quad_f::QuadF::new(
            self.map_point(*quad.p1()),
            self.map_point(*quad.p2()),
            self.map_point(*quad.p3()),
            self.map_point(*quad.p4()),
        )
        .BoundingBox()
    }

    // cpp: foundation/gfx_geometry/transform.h:216-219
    // cpp: foundation/gfx_geometry/transform.cc:225-233,254-263
    pub fn Translate3d(&mut self, x: f32, y: f32, z: f32) {
        match &mut self.representation {
            Representation::Axis2d { scale, translation } if z == 0.0 => {
                translation[0] += x * scale[0];
                translation[1] += y * scale[1];
            }
            _ => {
                let mut matrix = self.matrix();
                for row in 0..4 {
                    matrix[3][row] += matrix[0][row] * f64::from(x)
                        + matrix[1][row] * f64::from(y)
                        + matrix[2][row] * f64::from(z);
                }
                self.representation = Representation::Matrix44(matrix);
            }
        }
    }
}

// cpp: foundation/gfx_geometry/transform.cc:302-316
impl std::ops::Mul for Transform {
    type Output = Self;

    fn mul(mut self, other: Self) -> Self::Output {
        self.PreConcat(&other);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Transform;

    #[test]
    fn pretranslation_uses_axis_scale_then_promotes_for_z() {
        let mut transform = Transform::default();
        transform.Translate3d(7.0, -2.0, 0.0);
        let values = transform.GetColMajor();
        assert_eq!(values[12], 7.0);
        assert_eq!(values[13], -2.0);
        transform.Translate3d(0.0, 0.0, 5.0);
        let values = transform.GetColMajor();
        assert_eq!(values[12], 7.0);
        assert_eq!(values[13], -2.0);
        assert_eq!(values[14], 5.0);
    }

    #[test]
    fn equivalent_axis_and_full_matrix_compare_equal() {
        let axis = Transform::default();
        let full = Transform::ColMajor(&axis.GetColMajor());
        assert_eq!(axis, full);
    }

    #[test]
    fn concat_order_and_perspective_origin_match_matrix_order() {
        let mut left = Transform::default();
        left.PostTranslate(10.0, 20.0);
        let mut right = Transform::default();
        right.Translate3d(3.0, 4.0, 5.0);
        let result = left.clone() * right;
        assert_eq!(result.GetColMajor()[12..15], [13.0, 24.0, 5.0]);

        let mut perspective = Transform::default();
        perspective.ApplyPerspectiveDepth(200.0);
        perspective.ApplyTransformOrigin(10.0, 20.0, 0.0);
        assert_eq!(perspective.GetColMajor()[11], -0.005);
    }

    #[test]
    fn rotated_rectangle_uses_all_four_mapped_corners() {
        use crate::gfx_ext::RectF;
        use foundation_base::gfx_geometry::{PointF, SizeF};

        let rotation = Transform::ColMajor(&[
            0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);
        let rect = RectF::new(PointF::new(2.0, 3.0), SizeF::new(4.0, 5.0));
        assert_eq!(
            rotation.MapRect(rect),
            RectF::new(PointF::new(-8.0, 2.0), SizeF::new(5.0, 4.0))
        );
    }
}

include!("gfx_transform_rotation_scale.rs");
