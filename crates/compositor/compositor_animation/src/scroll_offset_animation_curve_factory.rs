// Translation status: complete
// Source: cc/animation/scroll_offset_animation_curve_factory.h:1-39
// Source: cc/animation/scroll_offset_animation_curve_factory.cc:1-78
use super::scroll_offset_animation_curve::{
    AnimationType, DurationBehavior, ScrollCurveConfig, ScrollOffsetAnimationCurve, ScrollType,
};
use compositor_tree::geometry::PointF;
use std::sync::Arc;
// Source: cc/animation/scroll_offset_animation_curve_factory.cc:12-24
fn GetDurationBehaviorFromScrollType(kind: ScrollType) -> DurationBehavior {
    match kind {
        ScrollType::kProgrammatic => DurationBehavior::kDeltaBased,
        ScrollType::kKeyboard => DurationBehavior::kConstant,
        ScrollType::kMouseWheel => DurationBehavior::kInverseDelta,
        ScrollType::kAutoScroll => unreachable!("autoscroll has a linear curve"),
    }
}
pub struct ScrollOffsetAnimationCurveFactory;
impl ScrollOffsetAnimationCurveFactory {
    // Source: cc/animation/scroll_offset_animation_curve_factory.cc:27-39
    pub fn CreateAnimation(
        target: PointF,
        kind: ScrollType,
        config: Arc<ScrollCurveConfig>,
    ) -> ScrollOffsetAnimationCurve {
        if kind == ScrollType::kAutoScroll {
            return Self::CreateLinearAnimation(target, config);
        }
        Self::CreateEaseInOutAnimation(
            target,
            kind,
            GetDurationBehaviorFromScrollType(kind),
            config,
        )
    }
    // Source: cc/animation/scroll_offset_animation_curve_factory.cc:41-50
    pub fn CreateEaseInOutAnimationForTesting(
        target: PointF,
        behavior: DurationBehavior,
        config: Arc<ScrollCurveConfig>,
    ) -> ScrollOffsetAnimationCurve {
        Self::CreateEaseInOutAnimation(target, ScrollType::kProgrammatic, behavior, config)
    }
    // Source: cc/animation/scroll_offset_animation_curve_factory.cc:52-58
    pub fn CreateLinearAnimationForTesting(
        target: PointF,
        config: Arc<ScrollCurveConfig>,
    ) -> ScrollOffsetAnimationCurve {
        Self::CreateLinearAnimation(target, config)
    }
    // Source: cc/animation/scroll_offset_animation_curve_factory.cc:60-68
    fn CreateEaseInOutAnimation(
        target: PointF,
        kind: ScrollType,
        behavior: DurationBehavior,
        config: Arc<ScrollCurveConfig>,
    ) -> ScrollOffsetAnimationCurve {
        ScrollOffsetAnimationCurve::new(
            target,
            AnimationType::kEaseInOut,
            kind,
            Some(behavior),
            config,
        )
    }
    // Source: cc/animation/scroll_offset_animation_curve_factory.cc:70-76
    fn CreateLinearAnimation(
        target: PointF,
        config: Arc<ScrollCurveConfig>,
    ) -> ScrollOffsetAnimationCurve {
        ScrollOffsetAnimationCurve::new(
            target,
            AnimationType::kLinear,
            ScrollType::kAutoScroll,
            None,
            config,
        )
    }
}
