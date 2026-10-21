// C++: src/foundation/gfx_geometry/vector2d_f.cc

use super::vector2d_f::Vector2dF;

impl Vector2dF {
    pub fn ToString(&self) -> String {
        format!("[{} {}]", self.x(), self.y())
    }

    pub fn IsZero(&self) -> bool {
        self.x() == 0.0 && self.y() == 0.0
    }

    pub fn Add(&mut self, other: &Self) {
        self.set_x(self.x() + other.x());
        self.set_y(self.y() + other.y());
    }

    pub fn Subtract(&mut self, other: &Self) {
        self.set_x(self.x() - other.x());
        self.set_y(self.y() - other.y());
    }

    pub fn LengthSquared(&self) -> f64 {
        self.x() as f64 * self.x() as f64 + self.y() as f64 * self.y() as f64
    }

    pub fn Length(&self) -> f32 {
        self.x().hypot(self.y())
    }

    pub fn Scale(&mut self, x_scale: f32, y_scale: f32) {
        self.set_x(self.x() * x_scale);
        self.set_y(self.y() * y_scale);
    }

    pub fn InvScale(&mut self, inv_x_scale: f32, inv_y_scale: f32) {
        self.set_x(self.x() / inv_x_scale);
        self.set_y(self.y() / inv_y_scale);
    }

    pub fn SlopeAngleRadians(&self) -> f32 {
        (self.y() as f64).atan2(self.x() as f64) as f32
    }
}

pub fn CrossProduct(lhs: &Vector2dF, rhs: &Vector2dF) -> f64 {
    lhs.x() as f64 * rhs.y() as f64 - lhs.y() as f64 * rhs.x() as f64
}

pub fn DotProduct(lhs: &Vector2dF, rhs: &Vector2dF) -> f64 {
    lhs.x() as f64 * rhs.x() as f64 + lhs.y() as f64 * rhs.y() as f64
}

pub fn ScaleVector2d(vector: Vector2dF, x_scale: f32, y_scale: f32) -> Vector2dF {
    let mut scaled = vector;
    scaled.Scale(x_scale, y_scale);
    scaled
}
