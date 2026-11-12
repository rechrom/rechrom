// Chromium cc/animation/animation_id_provider.cc:10-40. One canonical counter owner.

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
