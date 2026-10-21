// Copyright 2012, 2022, 2023 The Chromium Authors. BSD-style license.
// Source: gfx_geometry/{transform.cc,matrix44.cc,sin_cos_degrees.h}.
// Included inside gfx_transform so the source's float Axis2d representation is
// preserved until a rotation or non-unit z scale promotes it to double Matrix44.
fn sin_cos_degrees(mut degrees: f64) -> (f64, f64) {
    // cpp: foundation/gfx_geometry/sin_cos_degrees.h:23-119
    if degrees > -90_000_000.0 && degrees < 90_000_000.0 {
        let n45degrees = degrees / 45.0;
        let mut octant = n45degrees as i32;
        if f64::from(octant) == n45degrees {
            const Q: f64 = core::f64::consts::SQRT_2 / 2.0;
            const TABLE: [(f64, f64); 8] = [
                (0.0, 1.0),
                (Q, Q),
                (1.0, 0.0),
                (Q, -Q),
                (0.0, -1.0),
                (-Q, -Q),
                (-1.0, 0.0),
                (-Q, Q),
            ];
            return TABLE[(octant & 7) as usize];
        }
        if degrees < 0.0 {
            octant -= 1;
        }
        degrees -= f64::from(octant) * 45.0;
        if octant & 1 != 0 {
            degrees = 45.0 - degrees;
        }
        // cpp: foundation/base/numerics/angle_conversions.h:15-17
        let rad = degrees * core::f64::consts::PI / 180.0;
        let (mut s, mut c) = rad.sin_cos();
        if (octant + 1) & 2 != 0 {
            core::mem::swap(&mut s, &mut c);
        }
        if octant & 4 != 0 {
            s = -s;
        }
        if (octant + 2) & 4 != 0 {
            c = -c;
        }
        return (s, c);
    }
    degrees %= 360.0;
    (degrees * core::f64::consts::PI / 180.0).sin_cos()
}
#[allow(non_snake_case)]
impl Transform {
    // cpp: foundation/gfx_geometry/transform.h:233
    pub fn Rotate(&mut self, degrees: f64) {
        self.RotateAboutZAxis(degrees);
    }
    // cpp: foundation/gfx_geometry/transform.cc:140-158
    pub fn RotateAboutXAxis(&mut self, degrees: f64) {
        let (s, c) = sin_cos_degrees(degrees);
        if s == 0.0 && c == 1.0 {
            return;
        }
        self.rotate_principal_axis(0, s, c);
    }
    pub fn RotateAboutYAxis(&mut self, degrees: f64) {
        let (s, c) = sin_cos_degrees(degrees);
        if s == 0.0 && c == 1.0 {
            return;
        }
        self.rotate_principal_axis(1, s, c);
    }
    pub fn RotateAboutZAxis(&mut self, degrees: f64) {
        let (s, c) = sin_cos_degrees(degrees);
        if s == 0.0 && c == 1.0 {
            return;
        }
        self.rotate_principal_axis(2, s, c);
    }
    // cpp: foundation/gfx_geometry/matrix44.cc:247-266
    // Explicit mul_add preserves the compiled C++ vector contraction order.
    fn rotate_principal_axis(&mut self, axis: usize, s: f64, c: f64) {
        let mut matrix = self.matrix();
        let (a, b, reverse) = match axis {
            0 => (1, 2, false),
            1 => (0, 2, true),
            _ => (0, 1, false),
        };
        let ca = matrix[a];
        let cb = matrix[b];
        for row in 0..4 {
            if reverse {
                matrix[a][row] = ca[row].mul_add(c, (-cb[row]) * s);
                matrix[b][row] = cb[row].mul_add(c, ca[row] * s);
            } else {
                matrix[a][row] = ca[row].mul_add(c, cb[row] * s);
                matrix[b][row] = cb[row].mul_add(c, (-ca[row]) * s);
            }
        }
        self.representation = Representation::Matrix44(matrix);
    }
    // cpp: foundation/gfx_geometry/transform.cc:161-179
    pub fn RotateAbout(&mut self, mut x: f64, mut y: f64, mut z: f64, degrees: f64) {
        let (s, c) = sin_cos_degrees(degrees);
        if s == 0.0 && c == 1.0 {
            return;
        }
        // Match the official arm64 C++ archive's contraction order.
        let square_length = z.mul_add(z, x.mul_add(x, y * y));
        if square_length == 0.0 {
            return;
        }
        if square_length != 1.0 {
            let scale = 1.0 / square_length.sqrt();
            x *= scale;
            y *= scale;
            z *= scale;
        }
        // cpp: foundation/gfx_geometry/matrix44.cc:207-244
        if z == 1.0 {
            self.rotate_principal_axis(2, s, c);
            return;
        }
        if y == 1.0 {
            self.rotate_principal_axis(1, s, c);
            return;
        }
        if x == 1.0 {
            self.rotate_principal_axis(0, s, c);
            return;
        }
        let cc = 1.0 - c;
        let xs = x * s;
        let ys = y * s;
        let zs = z * s;
        let xc = x * cc;
        let yc = y * cc;
        let zc = z * cc;
        let xyc = x * yc;
        let yzc = y * zc;
        let zxc = z * xc;
        let rotation = [
            [x.mul_add(xc, c), xyc + zs, zxc - ys, 0.0],
            [xyc - zs, y.mul_add(yc, c), yzc + xs, 0.0],
            [zxc + ys, yzc - xs, z.mul_add(zc, c), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let old = self.matrix();
        let mut result = [[0.0; 4]; 4];
        // cpp: foundation/gfx_geometry/matrix44.cc:292-329
        let is_2d = |m: [[f64; 4]; 4]| {
            m[2] == [0.0, 0.0, 1.0, 0.0]
                && m[0][2] == 0.0
                && m[1][2] == 0.0
                && m[3][2] == 0.0
                && m[0][3] == 0.0
                && m[1][3] == 0.0
                && m[3][3] == 1.0
        };
        if is_2d(old) && is_2d(rotation) {
            let a = old[0][0];
            let b = old[0][1];
            let c = old[1][0];
            let d = old[1][1];
            let e = old[3][0];
            let f = old[3][1];
            let ya = rotation[0][0];
            let yb = rotation[0][1];
            let yc = rotation[1][0];
            let yd = rotation[1][1];
            let ye = rotation[3][0];
            let yf = rotation[3][1];
            result = [
                [a.mul_add(ya, c * yb), b.mul_add(ya, d * yb), 0.0, 0.0],
                [a.mul_add(yc, c * yd), b.mul_add(yc, d * yd), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    a.mul_add(ye, c * yf) + e,
                    b.mul_add(ye, d * yf) + f,
                    0.0,
                    1.0,
                ],
            ];
        } else {
            for col in 0..4 {
                for row in 0..4 {
                    let first =
                        old[0][row].mul_add(rotation[col][0], old[1][row] * rotation[col][1]);
                    let second = old[2][row].mul_add(rotation[col][2], first);
                    result[col][row] = old[3][row].mul_add(rotation[col][3], second);
                }
            }
        }
        self.representation = Representation::Matrix44(result);
    }
    // cpp: foundation/gfx_geometry/transform.cc:184-209
    pub fn Scale3d(&mut self, x: f32, y: f32, z: f32) {
        if let Representation::Axis2d { scale, .. } = &mut self.representation {
            if z == 1.0 {
                scale[0] *= x;
                scale[1] *= y;
                return;
            }
        }
        let mut matrix = self.matrix();
        if z == 1.0 {
            // Already full Matrix44: Scale(x,y) touches only cols 0 and 1.
            for row in 0..4 {
                matrix[0][row] *= f64::from(x);
                matrix[1][row] *= f64::from(y);
            }
        } else {
            for row in 0..4 {
                matrix[0][row] *= f64::from(x);
                matrix[1][row] *= f64::from(y);
                matrix[2][row] *= f64::from(z);
            }
        }
        self.representation = Representation::Matrix44(matrix);
    }
}
