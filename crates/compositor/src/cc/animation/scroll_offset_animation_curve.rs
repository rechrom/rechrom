// Translation status: complete
// Source: cc/animation/scroll_offset_animation_curve.h:1-179
// Source: cc/animation/scroll_offset_animation_curve.cc:1-425
use crate::adapter::{
    browser_controls::Tween,
    cubic_bezier::CubicBezier,
    geometry::{PointF, Vector2dF},
    time::TimeDelta,
};
use std::{cell::Cell, sync::Arc};
// Source: cc/animation/scroll_offset_animation_curve.h:44-55,125
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationBehavior {
    kDeltaBased,
    kConstant,
    kInverseDelta,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollType {
    kProgrammatic,
    kKeyboard,
    kMouseWheel,
    kAutoScroll,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationType {
    kLinear,
    kEaseInOut,
}
// Source dependency: cc/base/features.cc:218-242; actual process feature values.
#[derive(Clone, Debug)]
pub struct ScrollCurveConfig {
    pub programmatic_control_points: [f64; 4],
    pub maximum_animation_duration: TimeDelta,
}
impl Default for ScrollCurveConfig {
    fn default() -> Self {
        #[cfg(target_os = "android")]
        let (programmatic_control_points, maximum_animation_duration) =
            ([0.42, 0.0, 0.58, 1.0], TimeDelta::Milliseconds(700));
        #[cfg(not(target_os = "android"))]
        let (programmatic_control_points, maximum_animation_duration) =
            ([0.4, 0.0, 0.0, 1.0], TimeDelta::Milliseconds(1500));
        Self {
            programmatic_control_points,
            maximum_animation_duration,
        }
    }
}
// Source: cc/animation/scroll_offset_animation_curve.cc:16-57,65
const kConstantDuration: f64 = 9.0;
const kDurationDivisor: f64 = 60.0;
const kEpsilon: f64 = 0.01f32 as f64;
const kEaseInOutControlPoints: [f64; 4] = [0.42, 0.0, 0.58, 1.0];
const kInverseDeltaRampStartPx: f64 = 120.0;
const kInverseDeltaRampEndPx: f64 = 480.0;
const kInverseDeltaMinDuration: f64 = 6.0;
const kInverseDeltaMaxDuration: f64 = 12.0;
const kInverseDeltaSlope: f64 = (kInverseDeltaMinDuration - kInverseDeltaMaxDuration)
    / (kInverseDeltaRampEndPx - kInverseDeltaRampStartPx);
const kInverseDeltaOffset: f64 =
    kInverseDeltaMaxDuration - kInverseDeltaRampStartPx * kInverseDeltaSlope;
// Source: cc/animation/scroll_offset_animation_curve.cc:105-106; source single animation sequence.
thread_local! {static animation_duration_for_testing_:Cell<Option<f64>>=const{Cell::new(None)};}
fn seconds(value: f64) -> TimeDelta {
    TimeDelta::Microseconds((value * 1_000_000.0) as i64)
}
// Source: cc/animation/scroll_offset_animation_curve.cc:67-69
fn MaximumDimension(delta: Vector2dF) -> f32 {
    if delta.x().abs() > delta.y().abs() {
        delta.x()
    } else {
        delta.y()
    }
}
// Source: cc/animation/scroll_offset_animation_curve.cc:70-79
fn EaseInOutWithInitialSlope(points: [f64; 4], slope: f64) -> TimingFunction {
    let slope = slope.clamp(-1000.0, 1000.0);
    TimingFunction::Cubic(CubicBezier::new(
        points[0],
        points[0] * slope,
        points[2],
        points[3],
    ))
}
// Source: cc/animation/scroll_offset_animation_curve.cc:81-101
fn VelocityBasedDurationBound(
    _old_delta: Vector2dF,
    velocity: f64,
    new_delta: Vector2dF,
) -> TimeDelta {
    let new_delta_max_dimension = MaximumDimension(new_delta) as f64;
    if new_delta_max_dimension.abs() < kEpsilon {
        return TimeDelta::default();
    }
    if velocity.abs() < kEpsilon {
        return TimeDelta::Max();
    }
    let bound = (new_delta_max_dimension / velocity) * 2.5f32 as f64;
    if bound < 0.0 {
        TimeDelta::Max()
    } else {
        seconds(bound)
    }
}
// Source dependency: ui/gfx/animation/keyframe/timing_function.cc:61-69,183-190.
#[derive(Clone)]
enum TimingFunction {
    Linear,
    Cubic(CubicBezier),
}
impl TimingFunction {
    fn GetValue(&self, x: f64) -> f64 {
        match self {
            Self::Linear => x,
            Self::Cubic(b) => b.Solve(x),
        }
    }
    fn Velocity(&self, x: f64) -> f64 {
        match self {
            Self::Linear => 0.0,
            Self::Cubic(b) => b.Slope(x),
        }
    }
}
// Source: cc/animation/scroll_offset_animation_curve.h:32-41,151-174
// Source raw Target* is supplied per Tick call instead of an owning backpointer.
pub trait ScrollCurveTarget<K> {
    fn OnScrollOffsetAnimated(
        &mut self,
        value: PointF,
        target_property_id: i32,
        keyframe_model: &K,
    );
}
impl crate::ui::gfx::animation::keyframe::keyframe_model::AnimationCurve
    for ScrollOffsetAnimationCurve
{
    fn Duration(&self) -> TimeDelta {
        self.Duration()
    }
}
pub struct ScrollOffsetAnimationCurve {
    initial_value_: PointF,
    target_value_: PointF,
    total_animation_duration_: TimeDelta,
    last_retarget_: TimeDelta,
    timing_function_: TimingFunction,
    animation_type_: AnimationType,
    scroll_type_: ScrollType,
    duration_behavior_: Option<DurationBehavior>,
    has_set_initial_value_: bool,
    config: Arc<ScrollCurveConfig>,
}
impl ScrollOffsetAnimationCurve {
    // Source: cc/animation/scroll_offset_animation_curve.cc:108-144
    pub(crate) fn new(
        target: PointF,
        animation_type: AnimationType,
        scroll_type: ScrollType,
        duration_behavior: Option<DurationBehavior>,
        config: Arc<ScrollCurveConfig>,
    ) -> Self {
        debug_assert_eq!(
            animation_type == AnimationType::kEaseInOut,
            duration_behavior.is_some()
        );
        let mut result = Self {
            initial_value_: PointF::default(),
            target_value_: target,
            total_animation_duration_: TimeDelta::default(),
            last_retarget_: TimeDelta::default(),
            timing_function_: TimingFunction::Linear,
            animation_type_: animation_type,
            scroll_type_: scroll_type,
            duration_behavior_: duration_behavior,
            has_set_initial_value_: false,
            config,
        };
        result.timing_function_ = match animation_type {
            AnimationType::kEaseInOut => result.GetEasingFunction(None),
            AnimationType::kLinear => TimingFunction::Linear,
        };
        result
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:148-183
    fn EaseInOutSegmentDuration(
        &self,
        delta: Vector2dF,
        behavior: DurationBehavior,
        delayed_by: TimeDelta,
    ) -> TimeDelta {
        let mut duration = kConstantDuration;
        if let Some(testing) = animation_duration_for_testing_.get() {
            duration = testing;
        } else {
            match behavior {
                DurationBehavior::kConstant => {
                    duration = kConstantDuration;
                }
                DurationBehavior::kDeltaBased => {
                    assert_eq!(self.scroll_type_, ScrollType::kProgrammatic);
                    duration = (MaximumDimension(delta).abs().sqrt() as f64).min(
                        self.config.maximum_animation_duration.InSecondsF() * kDurationDivisor,
                    );
                }
                DurationBehavior::kInverseDelta => {
                    duration = kInverseDeltaOffset
                        + MaximumDimension(delta).abs() as f64 * kInverseDeltaSlope;
                    duration = duration.clamp(kInverseDeltaMinDuration, kInverseDeltaMaxDuration);
                }
            }
            duration /= kDurationDivisor;
        }
        let adjusted = seconds(duration) - delayed_by;
        if adjusted >= TimeDelta::default() {
            adjusted
        } else {
            TimeDelta::default()
        }
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:185-198
    fn EaseInOutBoundedSegmentDuration(
        &self,
        new_delta: Vector2dF,
        t: TimeDelta,
        delayed_by: TimeDelta,
    ) -> TimeDelta {
        let old_delta = self.target_value_ - self.initial_value_;
        let velocity = self.CalculateVelocity(t);
        self.EaseInOutSegmentDuration(new_delta, self.duration_behavior_.unwrap(), delayed_by)
            .min(VelocityBasedDurationBound(old_delta, velocity, new_delta))
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:200-213
    fn SegmentDuration(
        &self,
        delta: Vector2dF,
        delayed_by: TimeDelta,
        velocity: Option<f64>,
    ) -> TimeDelta {
        match self.animation_type_ {
            AnimationType::kEaseInOut => {
                self.EaseInOutSegmentDuration(delta, self.duration_behavior_.unwrap(), delayed_by)
            }
            AnimationType::kLinear => {
                Self::LinearSegmentDuration(delta, delayed_by, velocity.unwrap() as f32)
            }
        }
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:215-229
    pub fn LinearSegmentDuration(
        delta: Vector2dF,
        delayed_by: TimeDelta,
        velocity: f32,
    ) -> TimeDelta {
        let duration = animation_duration_for_testing_
            .get()
            .unwrap_or_else(|| (MaximumDimension(delta) / velocity).abs() as f64);
        let adjusted = seconds(duration) - delayed_by;
        if adjusted >= TimeDelta::default() {
            adjusted
        } else {
            TimeDelta::default()
        }
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:231-240
    pub fn SetInitialValue(&mut self, initial: PointF, delayed_by: TimeDelta, velocity: f32) {
        self.initial_value_ = initial;
        self.has_set_initial_value_ = true;
        let delta = self.target_value_ - initial;
        self.total_animation_duration_ =
            self.SegmentDuration(delta, delayed_by, Some(velocity as f64));
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:242-244
    pub fn HasSetInitialValue(&self) -> bool {
        self.has_set_initial_value_
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:246-250
    pub fn ApplyAdjustment(&mut self, adjustment: Vector2dF) {
        self.initial_value_ = self.initial_value_ + adjustment;
        self.target_value_ = self.target_value_ + adjustment;
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:252-267
    pub fn GetValue(&self, mut t: TimeDelta) -> PointF {
        let duration = self.total_animation_duration_ - self.last_retarget_;
        t -= self.last_retarget_;
        if duration.is_zero() || t >= duration {
            return self.target_value_;
        }
        if t <= TimeDelta::default() {
            return self.initial_value_;
        }
        let progress = self.timing_function_.GetValue(t / duration);
        PointF::new(
            Tween::FloatValueBetween(progress, self.initial_value_.x(), self.target_value_.x()),
            Tween::FloatValueBetween(progress, self.initial_value_.y(), self.target_value_.y()),
        )
    }
    // Source: cc/animation/scroll_offset_animation_curve.h:86
    pub fn target_value(&self) -> PointF {
        self.target_value_
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:269-279
    pub fn Duration(&self) -> TimeDelta {
        self.total_animation_duration_
    }
    pub fn Type(&self) -> i32 {
        6
    }
    pub fn TypeName(&self) -> &'static str {
        "ScrollOffset"
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:281-308
    pub fn Clone(&self) -> Self {
        self.CloneToScrollOffsetAnimationCurve()
    }
    pub fn Tick<K>(
        &self,
        t: TimeDelta,
        property_id: i32,
        keyframe: &K,
        target: Option<&mut dyn ScrollCurveTarget<K>>,
    ) {
        if let Some(target) = target {
            target.OnScrollOffsetAnimated(self.GetValue(t), property_id, keyframe);
        }
    }
    pub fn CloneToScrollOffsetAnimationCurve(&self) -> Self {
        Self {
            initial_value_: self.initial_value_,
            target_value_: self.target_value_,
            total_animation_duration_: self.total_animation_duration_,
            last_retarget_: self.last_retarget_,
            timing_function_: self.timing_function_.clone(),
            animation_type_: self.animation_type_,
            scroll_type_: self.scroll_type_,
            duration_behavior_: self.duration_behavior_,
            has_set_initial_value_: self.has_set_initial_value_,
            config: self.config.clone(),
        }
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:310-313
    pub fn SetAnimationDurationForTesting(duration: TimeDelta) {
        animation_duration_for_testing_.set(Some(duration.InSecondsF()));
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:315-325
    fn CalculateVelocity(&self, t: TimeDelta) -> f64 {
        let duration = self.total_animation_duration_ - self.last_retarget_;
        let slope = self
            .timing_function_
            .Velocity((t - self.last_retarget_) / duration);
        let delta = self.target_value_ - self.initial_value_;
        slope * (MaximumDimension(delta) as f64 / duration.InSecondsF())
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:327-339
    fn GetEasingFunction(&self, slope: Option<f64>) -> TimingFunction {
        let points = if self.scroll_type_ == ScrollType::kProgrammatic {
            self.config.programmatic_control_points
        } else {
            kEaseInOutControlPoints
        };
        if let Some(slope) = slope {
            return EaseInOutWithInitialSlope(points, slope);
        }
        TimingFunction::Cubic(CubicBezier::new(points[0], points[1], points[2], points[3]))
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:341-410
    pub fn UpdateTarget(&mut self, mut t: TimeDelta, new_target: PointF) {
        debug_assert_ne!(
            self.animation_type_,
            AnimationType::kLinear,
            "UpdateTarget is not supported on linear scroll animations."
        );
        if self.animation_type_ == AnimationType::kLinear {
            return;
        }
        let delayed_by = TimeDelta::default().max(self.last_retarget_ - t);
        t = t.max(self.last_retarget_);
        if self.animation_type_ == AnimationType::kEaseInOut
            && (MaximumDimension(self.target_value_ - new_target).abs() as f64) < kEpsilon
        {
            self.target_value_ = new_target;
            return;
        }
        let current = self.GetValue(t);
        let new_delta = new_target - current;
        if (MaximumDimension(new_delta).abs() as f64) < kEpsilon {
            self.last_retarget_ = t;
            self.total_animation_duration_ = t;
            self.target_value_ = new_target;
            return;
        }
        let old_duration = self.total_animation_duration_ - self.last_retarget_;
        if old_duration.is_zero() {
            debug_assert_eq!(t, self.last_retarget_);
            self.total_animation_duration_ = self.SegmentDuration(new_delta, delayed_by, None);
            self.target_value_ = new_target;
            return;
        }
        let new_duration = self.EaseInOutBoundedSegmentDuration(new_delta, t, delayed_by);
        if new_duration.InSecondsF() < kEpsilon {
            self.target_value_ = new_target;
            self.total_animation_duration_ = t;
            return;
        }
        let velocity = self.CalculateVelocity(t);
        let new_slope = velocity * (new_duration.InSecondsF() / MaximumDimension(new_delta) as f64);
        self.timing_function_ = self.GetEasingFunction(Some(new_slope));
        self.initial_value_ = current;
        self.target_value_ = new_target;
        self.total_animation_duration_ = t + new_duration;
        self.last_retarget_ = t;
    }
    // Source: cc/animation/scroll_offset_animation_curve.cc:412-423
    // Rust's concrete type eliminates both source checked static downcasts.
    pub fn ToScrollOffsetAnimationCurve(&self) -> &Self {
        self
    }
    pub fn ToScrollOffsetAnimationCurveMut(&mut self) -> &mut Self {
        self
    }
}
