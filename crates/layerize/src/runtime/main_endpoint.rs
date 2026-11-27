//! Engine-local Proxy effect endpoint for the canonical main LayerTreeHost.
//! No reference to a main/impl Host or Page can be recovered from this object.
//! Moved from compositor/src/runtime/main_endpoint.rs. The impl half supplies the sequence, effect queue, task runner
//! and AnimationHost query through `MainHostEffectSink`; it keeps the
//! AnimationHost mutations itself.
//!
//! The protected sequence (cc/base/protected_sequence_synchronizer.h,
//! LayerTreeHost::IsOwnerThread / InProtectedSequence /
//! WaitForProtectedSequenceCompletion, layer_tree_host.cc:329-336,520-570)
//! is started on the owner sequence by WillCommit storing the commit
//! completion event and concluded on the impl side by signaling it
//!. The event lives here rather than on the host so
//! the impl side can query it without borrowing the host.
use super::main_host::CpuMainHostEndpoint;
use crate::commit::{
    completion_event::CompletionEvent,
    main_callbacks::TaskRunner,
    main_effects::{LocalSurfaceId, MainHostEffect, MainHostEffectSink},
    protected_sequence_synchronizer::ProtectedSequenceSynchronizer,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

pub struct EngineMainEndpoint {
    sink: Arc<dyn MainHostEffectSink>,
    tasks: TaskRunner,
    commit_requested: AtomicBool,
    rendering_paused: AtomicBool,
    // Source: layer_tree_host.h commit_completion_event_.
    commit_completion: Mutex<Option<CompletionEvent>>,
    // Whether `commit_completion` holds an event: Chromium tests the raw
    // `commit_completion_event_` pointer before waiting
    // (layer_tree_host.cc:520-536). Every main Layer protected-field write
    // and Writable read asks, so the empty case must not take the mutex.
    // Written only while the mutex is held.
    commit_completion_present: AtomicBool,
    // The impl side is running FinishCommit through the commit lease. While
    // both halves share one sequence (S1b-2..S1b-6) that code runs on the
    // owner sequence but is impl code, so it is not the owner thread.
    impl_in_commit_lease: AtomicBool,
    // Both halves run on the owner sequence (in-process Proxy transport,
    // decision D1). A blocking wait could never be satisfied there; this is
    // the CompositorMode::SINGLE_THREADED early return of
    // layer_tree_host.cc:521-522.
    impl_on_owner_sequence: bool,
}
impl EngineMainEndpoint {
    pub fn new(sink: Arc<dyn MainHostEffectSink>, impl_on_owner_sequence: bool) -> Self {
        Self {
            tasks: sink.MainThreadTaskRunner(),
            sink,
            commit_requested: AtomicBool::new(false),
            rendering_paused: AtomicBool::new(false),
            commit_completion: Mutex::new(None),
            commit_completion_present: AtomicBool::new(false),
            impl_in_commit_lease: AtomicBool::new(false),
            impl_on_owner_sequence,
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
    fn completion(&self) -> MutexGuard<'_, Option<CompletionEvent>> {
        self.commit_completion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Both halves share the owner sequence (no blocking waits).
    pub fn ImplOnOwnerSequence(&self) -> bool {
        self.impl_on_owner_sequence
    }
    /// LayerTreeHost::WillCommit stores the commit completion
    /// (layer_tree_host.cc:428,483,501): the protected sequence starts on
    /// the owner sequence.
    pub fn BeginProtectedSequence(&self, completion: CompletionEvent) {
        debug_assert!(self.sink.RunsTasksInCurrentSequence());
        let mut current = self.completion();
        // Source: layer_tree_host.cc:428 DCHECK(!commit_completion_event_).
        // A signaled event not yet retired by CommitComplete is allowed; an
        // unsignaled one is a second overlapping commit.
        assert!(
            current.as_ref().is_none_or(CompletionEvent::IsSignaled),
            "previous commit still holds the protected sequence"
        );
        *current = Some(completion);
        self.commit_completion_present.store(true, Ordering::Release);
    }
    /// Source: layer_tree_host.cc:526-536 WaitForCommitCompletion. The event
    /// is cloned out so impl-side `InProtectedSequence` queries never wait on
    /// this mutex while the owner blocks.
    pub fn WaitForCommitCompletion(&self) {
        debug_assert!(self.IsOwnerThread());
        if !self.commit_completion_present.load(Ordering::Acquire) {
            return;
        }
        let Some(event) = self.completion().clone() else {
            return;
        };
        event.Wait();
        assert!(
            !event.WasAbandoned(),
            "the impl half terminated inside the protected sequence"
        );
        let mut current = self.completion();
        if current
            .as_ref()
            .is_some_and(|current| current.SameEvent(&event))
        {
            *current = None;
            self.commit_completion_present.store(false, Ordering::Release);
        }
    }
    /// Source: layer_tree_host.cc:552-570 (LayerTreeHost::CommitComplete): a
    /// signaled completion is retired without blocking.
    pub fn CommitComplete(&self) {
        let mut current = self.completion();
        if current.as_ref().is_some_and(CompletionEvent::IsSignaled) {
            let event = current.take().unwrap();
            self.commit_completion_present.store(false, Ordering::Release);
            assert!(
                !event.WasAbandoned(),
                "the impl half terminated inside the protected sequence"
            );
        }
    }
    /// Owner teardown: waits until no commit lease can still reach the main
    /// host and retires the completion without the abandonment assertion (a
    /// commit dropped by the destroyed impl half is abandoned by design;
    /// ProxyMain::Stop destroys ProxyImpl before the main host,
    /// proxy_main.cc:916-946).
    pub fn WaitForCommitRelease(&self) {
        let event = self.completion().clone();
        if let Some(event) = event {
            if !self.impl_on_owner_sequence && !event.IsSignaled() {
                event.Wait();
            }
            if event.IsSignaled() {
                let mut current = self.completion();
                if current.as_ref().is_some_and(|current| current.SameEvent(&event)) {
                    *current = None;
                    self.commit_completion_present.store(false, Ordering::Release);
                }
            }
        }
    }
    /// The impl side enters FinishCommit through the commit lease. The scope
    /// restores the flag on drop, including unwind.
    pub(crate) fn EnterCommitLease(&self) -> CommitLeaseScope<'_> {
        assert!(
            self.InProtectedSequence(),
            "commit lease used outside the protected sequence"
        );
        assert!(!self.impl_in_commit_lease.swap(true, Ordering::AcqRel));
        CommitLeaseScope(self)
    }
}
pub(crate) struct CommitLeaseScope<'a>(&'a EngineMainEndpoint);
impl Drop for CommitLeaseScope<'_> {
    fn drop(&mut self) {
        assert!(self.0.impl_in_commit_lease.swap(false, Ordering::AcqRel));
    }
}
impl ProtectedSequenceSynchronizer for EngineMainEndpoint {
    // Source: layer_tree_host.cc:329-332.
    fn IsOwnerThread(&self) -> bool {
        self.sink.RunsTasksInCurrentSequence() && !self.impl_in_commit_lease.load(Ordering::Acquire)
    }
    // Source: layer_tree_host.cc:334-336 (in_commit(): a stored completion
    // event that has not been signaled).
    fn InProtectedSequence(&self) -> bool {
        if !self.commit_completion_present.load(Ordering::Acquire) {
            return false;
        }
        self.completion()
            .as_ref()
            .is_some_and(|event| !event.IsSignaled())
    }
    // Source: layer_tree_host.cc:520-524.
    fn WaitForProtectedSequenceCompletion(&self) {
        if self.impl_on_owner_sequence {
            return;
        }
        self.WaitForCommitCompletion();
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
    // Source: cc/trees/layer_tree_host.cc:835-849 calls the main
    // LayerTreeHost's own `swap_promise_manager_`
    // (SwapPromiseManager::NotifyLatencyInfoSwapPromiseMonitors), which
    // notifies only monitors registered on the main thread through
    // LayerTreeHost::CreateLatencyInfoSwapPromiseMonitor. The main half has no
    // such registration site, so the notification visits an empty set. It is
    // a main-thread call, never a ProxyMain -> ProxyImpl task: the impl
    // LayerTreeHostImpl notifies its own monitors from its own redraw paths.
    fn NotifyLatencyInfoSwapPromiseMonitors(&self) {}
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
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.tasks.clone()
    }
}
