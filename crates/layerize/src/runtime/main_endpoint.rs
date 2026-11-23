//! Engine-local Proxy effect endpoint for the canonical main LayerTreeHost.
//! No reference to a main/impl Host or Page can be recovered from this object.
//! Moved from compositor/src/runtime/main_endpoint.rs (MAIN_IMPL_SPLIT.md
//! S1a-3 B3). The impl half supplies the sequence, effect queue, task runner
//! and AnimationHost query through `MainHostEffectSink`; it keeps the
//! AnimationHost mutations itself.
use super::main_host::CpuMainHostEndpoint;
use crate::commit::{
    input_types::ElementId,
    main_callbacks::TaskRunner,
    main_effects::{LocalSurfaceId, MainHostEffect, MainHostEffectSink},
    protected_sequence_synchronizer::ProtectedSequenceSynchronizer,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct EngineMainEndpoint {
    sink: Arc<dyn MainHostEffectSink>,
    tasks: TaskRunner,
    commit_requested: AtomicBool,
    rendering_paused: AtomicBool,
    protected_sequence: AtomicBool,
}
impl EngineMainEndpoint {
    pub fn new(sink: Arc<dyn MainHostEffectSink>) -> Self {
        Self {
            tasks: sink.MainThreadTaskRunner(),
            sink,
            commit_requested: AtomicBool::new(false),
            rendering_paused: AtomicBool::new(false),
            protected_sequence: AtomicBool::new(false),
        }
    }
    fn Post(&self, effect: MainHostEffect) {
        self.sink.PostEffect(effect);
    }
    pub fn DidBeginCommit(&self) {
        self.commit_requested.store(false, Ordering::Release);
    }
    pub fn SetRenderingPaused(&self, paused: bool) {
        self.rendering_paused.store(paused, Ordering::Release);
    }
    /// The Engine's &mut owner guarantees that main mutation and its protected
    /// commit walk cannot overlap. The guard marks that actual logical phase;
    /// it is not a synthetic successful wait or a new worker implementation.
    pub fn EnterProtectedSequence(&self) -> EngineProtectedSequence<'_> {
        assert!(self.sink.RunsTasksInCurrentSequence());
        assert!(!self.protected_sequence.swap(true, Ordering::AcqRel));
        EngineProtectedSequence(self)
    }
}
pub struct EngineProtectedSequence<'a>(&'a EngineMainEndpoint);
impl Drop for EngineProtectedSequence<'_> {
    fn drop(&mut self) {
        assert!(self.0.protected_sequence.swap(false, Ordering::AcqRel));
    }
}
impl ProtectedSequenceSynchronizer for EngineMainEndpoint {
    fn IsOwnerThread(&self) -> bool {
        self.sink.RunsTasksInCurrentSequence() && !self.protected_sequence.load(Ordering::Acquire)
    }
    fn InProtectedSequence(&self) -> bool {
        self.sink.RunsTasksInCurrentSequence() && self.protected_sequence.load(Ordering::Acquire)
    }
    fn WaitForProtectedSequenceCompletion(&self) {
        // Both phases belong to this sequenced Engine. A main-owner access is
        // admitted only after the synchronous commit scope has ended; worker
        // raster tasks never borrow main Layer state or enter this endpoint.
        assert!(
            self.IsOwnerThread(),
            "main owner cannot mutate during protected commit"
        );
    }
}
impl CpuMainHostEndpoint for EngineMainEndpoint {
    fn IsMainThread(&self) -> bool {
        self.IsOwnerThread()
    }
    fn CommitRequested(&self) -> bool {
        self.commit_requested.load(Ordering::Acquire)
    }
    fn SetNeedsUpdateLayers(&self) {
        self.Post(MainHostEffect::NeedsUpdateLayers)
    }
    fn SetNeedsCommit(&self, urgent: bool) {
        self.commit_requested.store(true, Ordering::Release);
        self.Post(MainHostEffect::NeedsCommit { urgent });
    }
    fn NotifyLatencyInfoSwapPromiseMonitors(&self) {
        self.Post(MainHostEffect::NotifyLatencyInfoSwapPromiseMonitors)
    }
    fn SaveActiveEventMetrics(&self) {
        self.Post(MainHostEffect::SaveActiveEventMetrics)
    }
    fn TraceCommitRequest(&self, urgent: bool) {
        self.sink.TraceCommitRequest(urgent)
    }
    fn SetTargetLocalSurfaceId(&self, id: LocalSurfaceId) {
        self.Post(MainHostEffect::SetTargetLocalSurfaceId(id))
    }
    fn IsRenderingPaused(&self) -> bool {
        self.rendering_paused.load(Ordering::Acquire)
    }
    fn NotifyNewLocalSurfaceIdExpectedWhilePaused(&self) {
        self.Post(MainHostEffect::NewLocalSurfaceIdExpectedWhilePaused)
    }
    fn UpdateDeferMainFrameUpdate(&self) {
        self.Post(MainHostEffect::UpdateDeferMainFrameUpdate)
    }
    fn IsElementAnimating(&self, id: ElementId) -> bool {
        self.sink.IsElementAnimating(id)
    }
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.tasks.clone()
    }
}
