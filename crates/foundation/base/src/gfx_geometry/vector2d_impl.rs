// C++: src/foundation/gfx_geometry/vector2d.cc

use super::vector2d::Vector2d;

impl Vector2d {
    pub fn IsZero(&self) -> bool {
        self.x() == 0 && self.y() == 0
    }

    pub fn Add(&mut self, other: &Self) {
        self.set_x(other.x().saturating_add(self.x()));
        self.set_y(other.y().saturating_add(self.y()));
    }

    pub fn Subtract(&mut self, other: &Self) {
        self.set_x(self.x().saturating_sub(other.x()));
        self.set_y(self.y().saturating_sub(other.y()));
    }

    pub fn LengthSquared(&self) -> i64 {
        self.x() as i64 * self.x() as i64 + self.y() as i64 * self.y() as i64
    }

    pub fn Length(&self) -> f32 {
        (self.LengthSquared() as f64).sqrt() as f32
    }

    pub fn ToString(&self) -> String {
        format!("[{} {}]", self.x(), self.y())
    }
}
