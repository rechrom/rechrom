//! Paint-owned metadata and injected sequenced-task/clock capabilities. No
//! animation decision, clock source, OS task runner, or tree state lives here.
use super::{
    input_types::ElementId,
    time::{TimeDelta, TimeTicks},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Weak},
};
// Source dependency: cc/paint/paint_image.h:177-186; paint_image.cc:45-46.
pub use paint::paint_engine::NativePaintImageId as PaintImageId;
pub const kInvalidId: PaintImageId = PaintImageId(-2);
pub const kDefaultFrameIndex: usize = 0;
// Source dependency: cc/paint/paint_image.h:218-219.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionState {
    kDone,
    kPartiallyDone,
}
// Source dependency: cc/paint/frame_metadata.h:14-27.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMetadata {
    pub complete: bool,
    pub duration: TimeDelta,
}
impl Default for FrameMetadata {
    fn default() -> Self {
        Self {
            complete: true,
            duration: TimeDelta::default(),
        }
    }
}
// Source dependency: cc/paint/discardable_image_map.h:44-61.
#[derive(Clone, Debug)]
pub struct AnimatedImageMetadata {
    pub paint_image_id: PaintImageId,
    pub sync_animation_target_id: PaintImageId,
    pub sync_animation_sequence_id: u32,
    pub completion_state: CompletionState,
    pub frames: Vec<FrameMetadata>,
    pub repetition_count: i32,
    pub reset_animation_sequence_id: u32,
}
// Source dependency: cc/paint/image_id.h:20-28.
pub type PaintImageIdFlatSet = BTreeSet<PaintImageId>;
pub type AnimatedImageDriverState = (bool, Vec<ElementId>);
pub type AnimatedImageDriverMap = BTreeMap<PaintImageId, AnimatedImageDriverState>;
// Source dependency: cc/paint/paint_image.h:520-532 (scoped_refptr value owner).
pub type AnimatedImageFrameIndexMap = BTreeMap<PaintImageId, usize>;
// Source dependency: cc/paint/image_animation_count.h:28-31.
pub const kAnimationLoopOnce: i32 = 0;
pub const kAnimationLoopInfinite: i32 = -1;
pub const kAnimationNone: i32 = -2;
pub const kAnimationPaused: i32 = -3;
// Source: cc/tiles/tile_priority.h:22-27. One canonical native enum.
pub use crate::cc::tiles::tile_priority::WhichTree;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageAnimationControllerId(pub u64);
/// Delivery never owns the controller. Invalidating its weak factory makes
/// already posted tasks inert, including after destruction and owner-id reuse.
#[derive(Clone, Debug)]
pub struct ImageAnimationTask {
    pub owner: ImageAnimationControllerId,
    pub(crate) alive: Weak<()>,
}
impl ImageAnimationTask {
    pub(crate) fn new(owner: ImageAnimationControllerId, alive: &Arc<()>) -> Self {
        Self {
            owner,
            alive: Arc::downgrade(alive),
        }
    }
    pub fn IsAlive(&self) -> bool {
        self.alive.upgrade().is_some()
    }
    pub(crate) fn matches(&self, owner: ImageAnimationControllerId, alive: &Arc<()>) -> bool {
        self.owner == owner && self.alive.ptr_eq(&Arc::downgrade(alive)) && self.IsAlive()
    }
}
/// An injected runner capability, never its thread/platform implementation.
/// PostDelayedTask must enqueue the weak task, routing it back through
/// ImageAnimationController::RunDelayedTask on this sequence.
pub trait ImageAnimationTaskRunner {
    fn BelongsToCurrentThread(&self) -> bool;
    fn PostDelayedTask(&mut self, task: ImageAnimationTask, delay: TimeDelta);
}
pub trait ImageAnimationClient {
    fn MonotonicTimeNow(&self) -> TimeTicks;
    fn RequestBeginFrameForAnimatedImages(&mut self);
    fn RequestInvalidationForAnimatedImages(&mut self);
    fn TraceEnabled(&self, event: &'static str) -> bool;
    fn Trace(&mut self, event: &'static str, value: ImageAnimationTraceValue<'_>);
}
pub enum ImageAnimationTraceValue<'a> {
    None,
    Duration(TimeDelta),
    State(&'a str),
    Frame { index: usize, duration: TimeDelta },
}
// Source dependency: base/time/time.h:467-479, integer modulo is C++ remainder.
pub fn SnappedToNextTick(time: TimeTicks, phase: TimeTicks, interval: TimeDelta) -> TimeTicks {
    let mut offset =
        TimeDelta::Microseconds((phase - time).InMicroseconds() % interval.InMicroseconds());
    if !offset.is_zero() && phase < time {
        offset += interval;
    }
    time + offset
}
