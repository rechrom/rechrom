// Chromium ui/events/gestures/fling_curve.h:18-47; .cc:14-93.
use foundation::gfx_geometry::Vector2dF;
const ALPHA: f32 = -5.70762e3;
const BETA: f32 = 1.72e2;
const GAMMA: f32 = 3.7;
fn position(t: f64) -> f64 {
    ALPHA as f64 * (-(GAMMA as f64) * t).exp() - BETA as f64 * t - ALPHA as f64
}
fn velocity(t: f64) -> f64 {
    (-ALPHA * GAMMA) as f64 * (-(GAMMA as f64) * t).exp() - BETA as f64
}
fn time_at_velocity(v: f64) -> f64 {
    -((v + BETA as f64) / (-ALPHA * GAMMA) as f64).ln() / GAMMA as f64
}
pub struct FlingCurve {
    duration: f32,
    time_offset: f32,
    position_offset: f32,
    displacement_ratio: Vector2dF,
}
impl FlingCurve {
    pub fn new(initial: Vector2dF) -> Self {
        let max = initial
            .x()
            .abs()
            .max(initial.y().abs())
            .min(velocity(0.) as f32);
        assert!(max > 0.);
        let time_offset = time_at_velocity(max as f64) as f32;
        Self {
            duration: time_at_velocity(0.) as f32,
            time_offset,
            position_offset: position(time_offset as f64) as f32,
            displacement_ratio: Vector2dF::new(initial.x() / max, initial.y() / max),
        }
    }
    pub fn ComputeScrollOffset(&self, elapsed: f64) -> (bool, Vector2dF, Vector2dF) {
        if elapsed < 0. {
            return (true, Vector2dF::default(), Vector2dF::default());
        }
        let time = elapsed + self.time_offset as f64;
        let active = time < self.duration as f64;
        let offset =
            (position(time.min(self.duration as f64)) - self.position_offset as f64) as f32;
        let velocity = if active { velocity(time) as f32 } else { 0. };
        (
            active,
            Vector2dF::new(
                self.displacement_ratio.x() * offset,
                self.displacement_ratio.y() * offset,
            ),
            Vector2dF::new(
                self.displacement_ratio.x() * velocity,
                self.displacement_ratio.y() * velocity,
            ),
        )
    }
}
