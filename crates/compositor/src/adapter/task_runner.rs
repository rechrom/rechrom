//! Task/clock dependency of translated notifier code. Posting is enqueue-only:
//! implementations must not run the supplied once closure within PostDelayedTask.
//! The owner supplies its existing executor and sequence validation; this module
//! creates neither an OS thread nor a timer.
use super::time::{TimeDelta, TimeTicks};
use std::sync::Arc;

pub type OnceClosure = Box<dyn FnOnce() + Send + 'static>;
pub type RepeatingClosure = Arc<dyn Fn() + Send + Sync + 'static>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskLocation {
    pub function: &'static str,
    pub file: &'static str,
    pub line: u32,
}
pub trait SequencedTaskRunner: Send + Sync {
    fn PostDelayedTask(&self, location: TaskLocation, closure: OnceClosure, delay: TimeDelta);
    fn Now(&self) -> TimeTicks;
    fn CalledOnValidThread(&self) -> bool;
}
