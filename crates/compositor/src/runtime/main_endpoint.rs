//! Engine-local Proxy effect endpoint for the canonical main LayerTreeHost.
//! No reference to a main/impl Host or Page can be recovered from this object.
use super::{
    execution::EngineTaskQueue,
    main_host::CpuMainHostEndpoint,
    native_animation::NativeAnimationOwner,
    native_services::NativeEventSink,
    types::{LocalSurfaceId, ResultTaskRunner, TaskRunner},
};
use crate::{
    adapter::input_types::ElementId,
    cc::base::protected_sequence_synchronizer::ProtectedSequenceSynchronizer,
    engine::CompositorExecutor,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// These are Proxy calls from LayerTreeHost, not copied commit/tree state.
/// Delivery preserves their source order on the current Engine command queue.
pub(crate) enum MainHostEffect {
    NeedsUpdateLayers,
    NeedsCommit { urgent: bool },
    NotifyLatencyInfoSwapPromiseMonitors,
    SaveActiveEventMetrics,
    SetTargetLocalSurfaceId(LocalSurfaceId),
    NewLocalSurfaceIdExpectedWhilePaused,
    UpdateDeferMainFrameUpdate,
}
struct MainTaskRunner {
    executor: Arc<dyn CompositorExecutor>,
}
impl ResultTaskRunner for MainTaskRunner {
    fn PostTask(&self, task: Box<dyn FnOnce() + Send>) {
        self.executor.Execute(task)
    }
}
pub(crate) struct EngineMainEndpoint {
    executor: Arc<dyn CompositorExecutor>,
    tasks: TaskRunner,
    commands: EngineTaskQueue,
    animations: NativeAnimationOwner,
    events: Arc<dyn NativeEventSink>,
    commit_requested: AtomicBool,
    rendering_paused: AtomicBool,
    protected_sequence: AtomicBool,
}
impl EngineMainEndpoint {
    pub(crate) fn new(
        executor: Arc<dyn CompositorExecutor>,
        commands: EngineTaskQueue,
        animations: NativeAnimationOwner,
        events: Arc<dyn NativeEventSink>,
    ) -> Self {
        Self {
            tasks: Arc::new(MainTaskRunner {
                executor: executor.clone(),
            }),
            executor,
            commands,
            animations,
            events,
            commit_requested: AtomicBool::new(false),
            rendering_paused: AtomicBool::new(false),
            protected_sequence: AtomicBool::new(false),
        }
    }
    fn Post(&self, effect: MainHostEffect) {
        self.commands
            .Push(super::execution::EngineTask::MainHost(effect));
    }
    pub(crate) fn DidBeginCommit(&self) {
        self.commit_requested.store(false, Ordering::Release);
    }
    pub(crate) fn SetRenderingPaused(&self, paused: bool) {
        self.rendering_paused.store(paused, Ordering::Release);
    }
    /// The Engine's &mut owner guarantees that main mutation and its protected
    /// commit walk cannot overlap. The guard marks that actual logical phase;
    /// it is not a synthetic successful wait or a new worker implementation.
    pub(crate) fn EnterProtectedSequence(&self) -> EngineProtectedSequence<'_> {
        assert!(self.executor.RunsTasksInCurrentSequence());
        assert!(!self.protected_sequence.swap(true, Ordering::AcqRel));
        EngineProtectedSequence(self)
    }
}
pub(crate) struct EngineProtectedSequence<'a>(&'a EngineMainEndpoint);
impl Drop for EngineProtectedSequence<'_> {
    fn drop(&mut self) {
        assert!(self.0.protected_sequence.swap(false, Ordering::AcqRel));
    }
}
impl ProtectedSequenceSynchronizer for EngineMainEndpoint {
    fn IsOwnerThread(&self) -> bool {
        self.executor.RunsTasksInCurrentSequence()
            && !self.protected_sequence.load(Ordering::Acquire)
    }
    fn InProtectedSequence(&self) -> bool {
        self.executor.RunsTasksInCurrentSequence()
            && self.protected_sequence.load(Ordering::Acquire)
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
        self.events
            .Emit("compositor.main", format!("SetNeedsCommit urgent={urgent}"));
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
        self.animations.IsElementAnimating(id)
    }
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.tasks.clone()
    }
}
