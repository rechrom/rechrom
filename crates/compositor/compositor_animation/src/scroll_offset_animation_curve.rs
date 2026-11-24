//! Compatibility re-export of the shared source animation primitive owner.
pub use scroll_animation::scroll_offset_animation_curve::*;

impl crate::gfx_keyframe_model::AnimationCurve for ScrollOffsetAnimationCurve {
    fn Duration(&self) -> scroll_animation::TimeDelta {
        ScrollOffsetAnimationCurve::Duration(self)
    }
}
