//! Content-frame interval metadata decisions.

use compositor_scheduler::TimeDelta;
use compositor_tree::damage_reason::{DamageReason, DamageReasonSet};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentIntervalKind {
    CompositorScroll,
    Video,
    AnimatingImage,
    ScrollbarFadeOutAnimation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentInterval {
    pub kind: ContentIntervalKind,
    pub duration: TimeDelta,
    pub duplicate_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimatedImageDuration {
    pub frame_duration: TimeDelta,
    pub num_images: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MetadataIntervalPlan {
    pub major_scroll_speed: Option<f32>,
    pub intervals: Vec<ContentInterval>,
    pub consumed_damage_reasons: usize,
}

pub fn PlanMetadataContentIntervals(
    damage_reasons: &mut DamageReasonSet,
    video_layer_preferred_intervals: &BTreeMap<TimeDelta, u32>,
    frame_max_scroll_delta: f32,
    begin_frame_time_delta: TimeDelta,
    trees_in_viz_in_viz_process: bool,
    animated_image_duration: impl FnOnce() -> Option<AnimatedImageDuration>,
) -> MetadataIntervalPlan {
    let mut trace = browser_tracing::span(
        "compositor.frame",
        "Compositor.Frame.PlanMetadataContentIntervals",
    );
    let mut plan = MetadataIntervalPlan::default();

    if damage_reasons.Has(DamageReason::kCompositorScroll) {
        let micros = begin_frame_time_delta.InMicroseconds();
        plan.major_scroll_speed = Some(if !(100..=1_000_000).contains(&micros) {
            0.0
        } else {
            (frame_max_scroll_delta as f64 / begin_frame_time_delta.InSecondsF()) as f32
        });
        plan.intervals.push(ContentInterval {
            kind: ContentIntervalKind::CompositorScroll,
            duration: TimeDelta::default(),
            duplicate_count: 1,
        });
        damage_reasons.Remove(DamageReason::kCompositorScroll);
        plan.consumed_damage_reasons += 1;
    }

    if !video_layer_preferred_intervals.is_empty() && damage_reasons.Has(DamageReason::kVideoLayer)
    {
        for (&duration, &count) in video_layer_preferred_intervals {
            plan.intervals.push(ContentInterval {
                kind: ContentIntervalKind::Video,
                duration,
                duplicate_count: count - 1,
            });
        }
        damage_reasons.Remove(DamageReason::kVideoLayer);
        plan.consumed_damage_reasons += 1;
    }

    if damage_reasons.Has(DamageReason::kAnimatedImage) && !trees_in_viz_in_viz_process {
        if let Some(duration) = animated_image_duration() {
            plan.intervals.push(ContentInterval {
                kind: ContentIntervalKind::AnimatingImage,
                duration: duration.frame_duration,
                duplicate_count: duration.num_images - 1,
            });
            damage_reasons.Remove(DamageReason::kAnimatedImage);
            plan.consumed_damage_reasons += 1;
        }
    }

    if damage_reasons.Has(DamageReason::kScrollbarFadeOutAnimation) {
        plan.intervals.push(ContentInterval {
            kind: ContentIntervalKind::ScrollbarFadeOutAnimation,
            duration: TimeDelta::Microseconds(50_000),
            duplicate_count: 0,
        });
        damage_reasons.Remove(DamageReason::kScrollbarFadeOutAnimation);
        plan.consumed_damage_reasons += 1;
    }

    trace.set("intervals", plan.intervals.len() as f64);
    trace.set(
        "consumed_damage_reasons",
        plan.consumed_damage_reasons as f64,
    );
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_intervals_and_consumes_only_represented_damage_reasons() {
        let mut reasons = DamageReasonSet::default();
        reasons.Put(DamageReason::kCompositorScroll);
        reasons.Put(DamageReason::kVideoLayer);
        reasons.Put(DamageReason::kAnimatedImage);
        reasons.Put(DamageReason::kScrollbarFadeOutAnimation);
        let video = BTreeMap::from([(TimeDelta::Microseconds(33_000), 2)]);
        let plan = PlanMetadataContentIntervals(
            &mut reasons,
            &video,
            12.0,
            TimeDelta::Microseconds(10_000),
            false,
            || {
                Some(AnimatedImageDuration {
                    frame_duration: TimeDelta::Microseconds(20_000),
                    num_images: 3,
                })
            },
        );
        assert_eq!(plan.intervals.len(), 4);
        assert_eq!(plan.intervals[1].kind, ContentIntervalKind::Video);
        assert_eq!(plan.intervals[1].duplicate_count, 1);
        assert_eq!(plan.intervals[2].kind, ContentIntervalKind::AnimatingImage);
        assert_eq!(plan.intervals[2].duplicate_count, 2);
        assert_eq!(plan.consumed_damage_reasons, 4);
        assert!(reasons.empty());
        assert_eq!(plan.major_scroll_speed, Some(1200.0));
    }

    #[test]
    fn viz_animation_path_keeps_animated_image_damage_and_is_lazy() {
        let mut reasons = DamageReasonSet::default();
        reasons.Put(DamageReason::kAnimatedImage);
        let plan = PlanMetadataContentIntervals(
            &mut reasons,
            &BTreeMap::new(),
            0.0,
            TimeDelta::default(),
            true,
            || unreachable!(),
        );
        assert!(plan.intervals.is_empty());
        assert!(reasons.Has(DamageReason::kAnimatedImage));
    }

    #[test]
    fn invalid_scroll_interval_reports_zero_speed() {
        let mut reasons = DamageReasonSet::default();
        reasons.Put(DamageReason::kCompositorScroll);
        let plan = PlanMetadataContentIntervals(
            &mut reasons,
            &BTreeMap::new(),
            50.0,
            TimeDelta::Microseconds(99),
            false,
            || None,
        );
        assert_eq!(plan.major_scroll_speed, Some(0.0));
    }
}
