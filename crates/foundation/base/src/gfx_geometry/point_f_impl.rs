// C++: src/foundation/gfx_geometry/point_f.cc

use super::point_f::PointF;

impl PointF {
    pub fn SetToMin(&mut self, other: &Self) {
        self.set_x(if other.x() < self.x() {
            other.x()
        } else {
            self.x()
        });
        self.set_y(if other.y() < self.y() {
            other.y()
        } else {
            self.y()
        });
    }

    pub fn SetToMax(&mut self, other: &Self) {
        self.set_x(if self.x() < other.x() {
            other.x()
        } else {
            self.x()
        });
        self.set_y(if self.y() < other.y() {
            other.y()
        } else {
            self.y()
        });
    }

    pub fn IsWithinDistance(&self, rhs: &Self, allowed_distance: f32) -> bool {
        debug_assert!(allowed_distance > 0.0);
        let diff_x = self.x() - rhs.x();
        let diff_y = self.y() - rhs.y();
        let distance = (diff_x * diff_x + diff_y * diff_y).sqrt();
        distance < allowed_distance
    }

    pub fn ToString(&self) -> String {
        format!("{},{}", self.x(), self.y())
    }
}

pub fn ScalePoint(point: PointF, x_scale: f32, y_scale: f32) -> PointF {
    let mut scaled = point;
    scaled.Scale(x_scale, y_scale);
    scaled
}

pub fn ScalePointUniform(point: PointF, scale: f32) -> PointF {
    ScalePoint(point, scale, scale)
}
