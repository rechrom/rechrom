// Chromium ui/events/gestures/blink/web_gesture_curve_impl.h:21-70;
// .cc:24-34,58-79,90-128 (selected desktop/fixed curve branch).
// Desktop default curve branch; the host explicitly selects the current macOS
// source policy (mobile and ExperimentalFlingAnimation are disabled there).
use super::{fixed_velocity_curve::FixedVelocityCurve, fling_curve::FlingCurve};
use crate::public::common::input::web_gesture_event::GestureDevice;
use foundation::gfx_geometry::Vector2dF;
enum Curve {
    Fling(FlingCurve),
    Fixed(FixedVelocityCurve),
}
pub struct WebGestureCurveImpl {
    curve: Curve,
    last_offset: Vector2dF,
}
impl WebGestureCurveImpl {
    pub fn CreateFromDefaultPlatformCurve(device: GestureDevice, velocity: Vector2dF) -> Self {
        Self {
            curve: if device == GestureDevice::kSyntheticAutoscroll {
                Curve::Fixed(FixedVelocityCurve::new(velocity))
            } else {
                Curve::Fling(FlingCurve::new(velocity))
            },
            last_offset: Vector2dF::default(),
        }
    }
    pub fn Advance(&mut self, time: f64, velocity: &mut Vector2dF, delta: &mut Vector2dF) -> bool {
        if time <= 0. {
            return true;
        }
        let (active, offset, current_velocity) = match &self.curve {
            Curve::Fling(curve) => curve.ComputeScrollOffset(time),
            Curve::Fixed(curve) => curve.ComputeScrollOffset(time),
        };
        *velocity = current_velocity;
        *delta = offset - self.last_offset;
        self.last_offset = offset;
        active
    }
}
