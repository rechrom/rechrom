// Translation status: complete
// Source: cc/animation/animation_id_provider.h:1-26
// Source: cc/animation/animation_id_provider.cc:1-38
use std::sync::atomic::{AtomicI32, Ordering};
// Source: cc/animation/animation_id_provider.cc:10-14
static g_next_keyframe_model_id: AtomicI32 = AtomicI32::new(0);
static g_next_group_id: AtomicI32 = AtomicI32::new(0);
static g_next_timeline_id: AtomicI32 = AtomicI32::new(0);
static g_next_animation_id: AtomicI32 = AtomicI32::new(0);
static g_next_animation_trigger_id: AtomicI32 = AtomicI32::new(0);
pub struct AnimationIdProvider;
impl AnimationIdProvider {
    // Source: cc/animation/animation_id_provider.cc:16-19
    pub fn NextKeyframeModelId() -> i32 {
        g_next_keyframe_model_id
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
    // Source: cc/animation/animation_id_provider.cc:21-24
    pub fn NextGroupId() -> i32 {
        g_next_group_id
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
    // Source: cc/animation/animation_id_provider.cc:26-28
    pub fn NextTimelineId() -> i32 {
        g_next_timeline_id
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
    // Source: cc/animation/animation_id_provider.cc:30-32
    pub fn NextAnimationId() -> i32 {
        g_next_animation_id
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
    // Source: cc/animation/animation_id_provider.cc:34-36
    pub fn NextAnimationTriggerId() -> i32 {
        g_next_animation_trigger_id
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }
}
