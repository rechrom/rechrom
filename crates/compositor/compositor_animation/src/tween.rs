use crate::cubic_bezier::CubicBezier;
use compositor_scheduler::TimeTicks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweenType {
    LINEAR,
    ACCEL_30_DECEL_LIN,
    ACCEL_LIN_DECEL_60,
}

pub struct Tween;

impl Tween {
    pub fn FloatValueBetween(value: f64, start: f32, target: f32) -> f32 {
        (f64::from(start) + f64::from(target - start) * value) as f32
    }
    pub fn IntValueBetween(value: f64, start: i32, target: i32) -> i32 {
        if start == target {
            return start;
        }
        let mut delta = f64::from(target - start);
        if delta < 0.0 {
            delta -= 1.0;
        } else {
            delta += 1.0;
        }
        let toward_zero = if delta > 0.0 {
            delta.next_down()
        } else {
            delta.next_up()
        };
        start + (value * toward_zero) as i32
    }
    pub fn CalculateValue(tween_type: TweenType, state: f64) -> f64 {
        use std::sync::OnceLock;
        static ACCEL: OnceLock<CubicBezier> = OnceLock::new();
        static DECEL: OnceLock<CubicBezier> = OnceLock::new();
        match tween_type {
            TweenType::LINEAR => state,
            TweenType::ACCEL_30_DECEL_LIN => ACCEL
                .get_or_init(|| CubicBezier::new(0.3, 0.0, 1.0, 1.0))
                .Solve(state),
            TweenType::ACCEL_LIN_DECEL_60 => DECEL
                .get_or_init(|| CubicBezier::new(0.0, 0.0, 0.4, 1.0))
                .Solve(state),
        }
    }
    pub fn ClampedFloatValueBetween(
        time: TimeTicks,
        start_time: TimeTicks,
        start: f32,
        target_time: TimeTicks,
        target: f32,
        tween_type: TweenType,
    ) -> f32 {
        if time <= start_time {
            return start;
        }
        if time >= target_time {
            return target;
        }
        let progress = (time - start_time) / (target_time - start_time);
        Self::FloatValueBetween(Self::CalculateValue(tween_type, progress), start, target)
    }
}
