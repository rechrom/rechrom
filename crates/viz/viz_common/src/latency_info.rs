//! ui::LatencyInfo as carried in viz::CompositorFrameMetadata::latency_info
//! (ui/latency/latency_info.h). The actual latency event owns its component
//! timestamps behind `LatencyInfoBackend`; frames share that allocation.
use scroll_animation::TimeTicks;
use std::sync::{Arc, Mutex, PoisonError};

// Source dependency: ui/latency/latency_info.h:64-67,163.
// These named variants select upstream components, not replacement numeric IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LatencyComponentType {
    FirstScrollUpdateOriginal,
    ScrollUpdateOriginal,
}

/// The actual latency event supplies component timestamps. This trait is kept
/// with the event allocation so the host never fabricates a reduced event copy.
pub trait LatencyInfoBackend: Send {
    fn FindLatency(&self, component: LatencyComponentType, timestamp: &mut TimeTicks) -> bool;
    fn AddRendererSwapTimestamp(&mut self, timestamp: TimeTicks);
    fn AddRenderingScheduledImpl(&mut self) -> bool;
    fn CloneForSwapPromise(&self) -> Box<dyn LatencyInfoBackend>;
    fn Terminate(&mut self);
    fn TraceId(&self) -> i64;
    fn TraceMainCommit(&self);
}
#[derive(Clone)]
pub struct LatencyInfo(pub Arc<Mutex<Box<dyn LatencyInfoBackend>>>);
impl LatencyInfo {
    pub fn new(value: Box<dyn LatencyInfoBackend>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
    fn backend(&self) -> std::sync::MutexGuard<'_, Box<dyn LatencyInfoBackend>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
    pub fn CloneForSwapPromise(&self) -> Self {
        let value = self.backend().CloneForSwapPromise();
        Self::new(value)
    }
    pub fn AddRenderingScheduledImpl(&mut self) -> bool {
        self.backend().AddRenderingScheduledImpl()
    }
    pub fn Terminate(&mut self) {
        self.backend().Terminate()
    }
    pub fn TraceId(&self) -> i64 {
        self.backend().TraceId()
    }
    pub fn TraceMainCommit(&self) {
        self.backend().TraceMainCommit()
    }
    pub fn AddRendererSwapTimestamp(&mut self, timestamp: TimeTicks) {
        self.backend().AddRendererSwapTimestamp(timestamp)
    }
    pub fn FindLatency(&self, component: LatencyComponentType, timestamp: &mut TimeTicks) -> bool {
        self.backend().FindLatency(component, timestamp)
    }
}
