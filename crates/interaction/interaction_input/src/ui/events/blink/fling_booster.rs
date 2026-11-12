// Chromium ui/events/blink/fling_booster.h:20-62; .cc:14-216.
use crate::{
    base::time::time::TimeTicks,
    public::common::input::{
        web_gesture_event::{GestureDevice, InertialPhaseState, WebGestureEvent},
        web_input_event::Type,
    },
};
use foundation::gfx_geometry::Vector2dF;

pub struct FlingBooster {
    cutoff_time_for_boost: TimeTicks,
    previous_fling_starting_velocity: Vector2dF,
    current_fling_velocity: Vector2dF,
    source_device: GestureDevice,
    modifiers: i32,
    previous_boosting_scroll_timestamp: TimeTicks,
}
impl Default for FlingBooster {
    fn default() -> Self {
        Self {
            cutoff_time_for_boost: TimeTicks::new(),
            previous_fling_starting_velocity: Vector2dF::default(),
            current_fling_velocity: Vector2dF::default(),
            source_device: GestureDevice::kUninitialized,
            modifiers: 0,
            previous_boosting_scroll_timestamp: TimeTicks::new(),
        }
    }
}
fn zero(v: Vector2dF) -> bool {
    v.x() == 0. && v.y() == 0.
}
fn square(v: Vector2dF) -> f64 {
    v.x() as f64 * v.x() as f64 + v.y() as f64 * v.y() as f64
}
fn dot(a: Vector2dF, b: Vector2dF) -> f32 {
    a.x() * b.x() + a.y() * b.y()
}
impl FlingBooster {
    pub fn GetVelocityForFlingStart(&mut self, event: &WebGestureEvent) -> Vector2dF {
        debug_assert_eq!(event.GetType(), Type::kGestureFlingStart);
        let mut velocity = Vector2dF::new(
            event.data.fling_start.velocity_x,
            event.data.fling_start.velocity_y,
        );
        if self.ShouldBoostFling(event) {
            velocity += self.previous_fling_starting_velocity;
        }
        self.Reset();
        self.previous_fling_starting_velocity = velocity;
        self.current_fling_velocity = velocity;
        self.source_device = event.SourceDevice();
        self.modifiers = event.GetModifiers();
        velocity
    }
    pub fn ObserveGestureEvent(&mut self, event: &WebGestureEvent) {
        if zero(self.previous_fling_starting_velocity) {
            return;
        }
        if (!self.cutoff_time_for_boost.is_null() && event.TimeStamp() > self.cutoff_time_for_boost)
            || event.SourceDevice() != self.source_device
        {
            self.Reset();
            return;
        }
        match event.GetType() {
            Type::kGestureScrollBegin => {
                self.cutoff_time_for_boost =
                    TimeTicks::FromInternalValue(event.TimeStamp().ToInternalValue() + 50_000)
            }
            Type::kGestureScrollUpdate => {
                if event.data.scroll_update.inertial_phase == InertialPhaseState::kMomentum
                    || self.cutoff_time_for_boost.is_null()
                {
                    return;
                }
                let delta = Vector2dF::new(
                    event.data.scroll_update.delta_x,
                    event.data.scroll_update.delta_y,
                );
                if dot(self.previous_fling_starting_velocity, delta) <= 0. {
                    self.Reset();
                    return;
                }
                if !self.previous_boosting_scroll_timestamp.is_null() {
                    let seconds = (event.TimeStamp().ToInternalValue()
                        - self.previous_boosting_scroll_timestamp.ToInternalValue())
                        as f64
                        / 1_000_000.;
                    if seconds >= 0.001 && square(delta) / (seconds * seconds) < 150. * 150. {
                        self.Reset();
                        return;
                    }
                }
                self.previous_boosting_scroll_timestamp = event.TimeStamp();
                self.cutoff_time_for_boost =
                    TimeTicks::FromInternalValue(event.TimeStamp().ToInternalValue() + 50_000);
            }
            Type::kGestureScrollEnd => self.previous_boosting_scroll_timestamp = TimeTicks::new(),
            Type::kGestureFlingCancel => {
                if event.data.fling_cancel.prevent_boosting {
                    self.Reset();
                    return;
                }
                self.previous_boosting_scroll_timestamp = TimeTicks::new();
                self.cutoff_time_for_boost =
                    TimeTicks::FromInternalValue(event.TimeStamp().ToInternalValue() + 50_000);
            }
            _ => {}
        }
    }
    pub fn ObserveProgressFling(&mut self, velocity: Vector2dF) {
        if !zero(self.previous_fling_starting_velocity) {
            self.current_fling_velocity = velocity;
        }
    }
    fn ShouldBoostFling(&self, event: &WebGestureEvent) -> bool {
        let velocity = Vector2dF::new(
            event.data.fling_start.velocity_x,
            event.data.fling_start.velocity_y,
        );
        !zero(self.previous_fling_starting_velocity)
            && self.source_device == event.SourceDevice()
            && self.modifiers == event.GetModifiers()
            && !self.cutoff_time_for_boost.is_null()
            && event.TimeStamp() <= self.cutoff_time_for_boost
            && dot(self.previous_fling_starting_velocity, velocity) > 0.
            && square(self.current_fling_velocity) >= 350. * 350.
            && square(velocity) >= 350. * 350.
    }
    pub fn Reset(&mut self) {
        *self = Self::default();
    }
}
