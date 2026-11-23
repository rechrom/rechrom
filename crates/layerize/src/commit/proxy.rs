//! Values that cross between ProxyMain (the document thread, layerize) and
//! ProxyImpl (the compositor thread) for one commit (S1B_PROXY_DESIGN.md 1.1).
//! Source: cc/trees/proxy_main.cc, cc/trees/proxy_impl.cc.

use crate::{
    adapter::main_layer::{LayerContext, MainLayerHostId},
    commit::{
        commit_state::ThreadUnsafeCommitState, completion_event::CompletionEvent,
        layerize_family::LayerizeFamily,
    },
    runtime::{
        main_endpoint::EngineMainEndpoint, main_host::CpuMainLayerHost,
        paint_services::CpuMainPaintServices,
    },
};
use std::{ptr::NonNull, sync::Arc};

/// The protected lease of the main thread's `ThreadUnsafeCommitState` that
/// ProxyMain hands to ProxyImpl with NotifyReadyToCommitOnImpl
/// (`&unsafe_state` and `commit_completion_event_`,
/// cc/trees/proxy_main.cc:474-477,555-560). LayerTreeHostImpl::FinishCommit
/// reads the main Layers through it while the main thread may not touch them
/// (cc/base/protected_sequence_synchronizer.h).
///
/// Soundness: the pointer targets the main host allocation owned by
/// `LayerizeEngine`. Every owner-side access goes through the engine's single
/// waiting accessor, which blocks until this lease's completion is signaled
/// (decision D2), so the impl side has exclusive access while it holds the
/// lease. Dropping the lease signals the completion (covering unwind and
/// channel teardown); a lease dropped without having run FinishCommit
/// signals it as abandoned.
pub struct ProtectedCommitLease<F: LayerizeFamily> {
    host: NonNull<CpuMainLayerHost<F>>,
    paint: Arc<CpuMainPaintServices<F::Skia>>,
    endpoint: Arc<EngineMainEndpoint>,
    completion: Option<CompletionEvent>,
    // Source: proxy_main.cc:552-566 (block_on_next_commit_): the main thread
    // waits for this commit right after posting it.
    blocking: bool,
    committed: bool,
}

// SAFETY: see the type comment. The host and its commit state are `Send`
// (compile-time assertions in compositor/src/runtime/engine_core.rs); the
// protected sequence serializes the two threads' access to them.
unsafe impl<F: LayerizeFamily> Send for ProtectedCommitLease<F> where CpuMainLayerHost<F>: Send {}

impl<F: LayerizeFamily> ProtectedCommitLease<F> {
    /// Created by `LayerizeEngine` after WillCommit started the protected
    /// sequence with `completion`.
    pub(crate) fn new(
        host: NonNull<CpuMainLayerHost<F>>,
        paint: Arc<CpuMainPaintServices<F::Skia>>,
        endpoint: Arc<EngineMainEndpoint>,
        completion: CompletionEvent,
        blocking: bool,
    ) -> Self {
        Self {
            host,
            paint,
            endpoint,
            completion: Some(completion),
            blocking,
            committed: false,
        }
    }

    pub fn IsBlocking(&self) -> bool {
        self.blocking
    }

    /// Lends the resident main state to LayerTreeHostImpl::FinishCommit
    /// (ProxyImpl::ScheduledActionCommit, proxy_impl.cc:868-899).
    pub fn WithCommitContext<T>(
        &mut self,
        f: impl FnOnce(&ThreadUnsafeCommitState<F>, &mut LayerContext<'_, F>, MainLayerHostId) -> T,
    ) -> T {
        let _scope = self.endpoint.EnterCommitLease();
        // SAFETY: the protected sequence is open (asserted by
        // EnterCommitLease) and the owner side only reaches the host through
        // the waiting accessor; see the type comment.
        let host = unsafe { &mut *self.host.as_ptr() };
        let result = host.WithCommitContext(self.paint.as_ref(), self.paint.as_ref(), f);
        self.committed = true;
        result
    }

    /// The completion event, for the impl owner that signals it after
    /// FinishCommit (ScopedCommitCompletionEvent, proxy_impl.cc:42-81). The
    /// lease no longer signals once this has been taken.
    pub fn TakeCompletion(&mut self) -> CompletionEvent {
        self.completion.take().expect("commit completion present")
    }
}

impl<F: LayerizeFamily> Drop for ProtectedCommitLease<F> {
    fn drop(&mut self) {
        if let Some(completion) = self.completion.take() {
            if self.committed {
                completion.Signal();
            } else {
                completion.SignalAbandoned();
            }
        }
    }
}
