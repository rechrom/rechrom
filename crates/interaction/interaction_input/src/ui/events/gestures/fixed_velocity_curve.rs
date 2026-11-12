// Chromium ui/events/gestures/fixed_velocity_curve.h:16-34; .cc:10-29.
use foundation::gfx_geometry::Vector2dF;
pub struct FixedVelocityCurve {
    velocity: Vector2dF,
}
impl FixedVelocityCurve {
    pub fn new(velocity: Vector2dF) -> Self {
        Self { velocity }
    }
    pub fn ComputeScrollOffset(&self, elapsed: f64) -> (bool, Vector2dF, Vector2dF) {
        let multiplier = (elapsed * 5000.) as f32;
        (
            true,
            Vector2dF::new(
                self.velocity.x() * multiplier,
                self.velocity.y() * multiplier,
            ),
            self.velocity,
        )
    }
}
