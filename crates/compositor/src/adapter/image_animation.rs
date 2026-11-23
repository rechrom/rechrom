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
// cc/paint image values (commit vocabulary).
pub use layerize::commit::paint::image_metadata::*;
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
