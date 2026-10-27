//! Compositor-owned animation identities and animation algorithms.
//!
//! This package is below the `compositor` facade. It must not depend on the
//! facade, Page, Paint, Viz, Renderer, or platform presentation.

#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementListType {
    ACTIVE,
    PENDING,
}

pub mod animation;
pub mod animation_events;
pub mod animation_timeline;
pub mod cubic_bezier;
pub mod gfx_keyframe_effect;
pub mod gfx_keyframe_model;
pub mod host_iteration;
pub mod keyframe_effect;
pub mod keyframe_model;
pub mod paint_worklet;
pub mod property_animation_state;
pub mod scroll_offset_animation_curve;
pub mod scroll_offset_animation_curve_factory;
pub mod scroll_offset_animations_impl;
pub mod tween;

pub mod animation_id_provider {
    use std::sync::atomic::{AtomicI32, Ordering};

    // Source: cc/animation/animation_id_provider.cc:10-14.
    static NEXT_KEYFRAME_MODEL_ID: AtomicI32 = AtomicI32::new(0);
    static NEXT_GROUP_ID: AtomicI32 = AtomicI32::new(0);
    static NEXT_TIMELINE_ID: AtomicI32 = AtomicI32::new(0);
    static NEXT_ANIMATION_ID: AtomicI32 = AtomicI32::new(0);
    static NEXT_ANIMATION_TRIGGER_ID: AtomicI32 = AtomicI32::new(0);

    pub struct AnimationIdProvider;

    impl AnimationIdProvider {
        pub fn NextKeyframeModelId() -> i32 {
            NEXT_KEYFRAME_MODEL_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        }

        pub fn NextGroupId() -> i32 {
            NEXT_GROUP_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        }

        pub fn NextTimelineId() -> i32 {
            NEXT_TIMELINE_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        }

        pub fn NextAnimationId() -> i32 {
            NEXT_ANIMATION_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        }

        pub fn NextAnimationTriggerId() -> i32 {
            NEXT_ANIMATION_TRIGGER_ID
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        }
    }
}
