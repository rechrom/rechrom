//! Values that cross between ProxyMain (the document thread, layerize) and
//! ProxyImpl (the compositor thread) for one commit.
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

    /// The main paint services lent with the unsafe state; FinishCommit's
    /// TreeSynchronizer traces through them (SynchronizationTrace).
    pub fn PaintServices(&self) -> &Arc<CpuMainPaintServices<F::Skia>> {
        &self.paint
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

// ---------------------------------------------------------------------------
// Proxy messages (S1b-3). The enums are the posted tasks of
// cc/trees/proxy_main.cc (ProxyMain -> ProxyImpl) and proxy_impl.cc
// (ProxyImpl -> ProxyMain). Values that only the impl half can construct are
// named through `ProxyTypes`, which the compositor implements for its family.

pub use compositor_values::commit_earlyout_reason::CommitEarlyOutReason;
use crate::commit::{
    commit_state::{CommitState, CommitTimestamps},
    compositor_commit_data::CompositorCommitData,
    geometry::Rect,
    main_effects::LocalSurfaceId,
};
use compositor_values::begin_frame::BeginFrameArgs;
use scroll_animation::TimeTicks;

/// The impl-provided payload types of the Proxy messages. Main code treats
/// them as opaque values (it moves them, never inspects them), except where
/// a later step translates the main consumer. cc::MutatorEvents moved to the
/// main AnimationHost it is applied to (`MainMutatorHost::Events`, S1b-5).
pub trait ProxyTypes: LayerizeFamily {
    /// std::unique_ptr<cc::SwapPromise> returned by an aborted main frame.
    type ProxySwapPromise: Send;
    /// std::unique_ptr<cc::LayerTreeMutator> (ProxyMain::SetMutator).
    type LayerTreeMutator: Send;
    /// The impl construction values ProxyMain::InitializeOnImplThread
    /// passes to the ProxyImpl constructor (proxy_main.cc:80-89).
    type ImplInit: Send;
    /// The presentation callbacks and FrameTimingDetails of
    /// ProxyMain::DidPresentCompositorFrame (proxy_main.h:71-78).
    type PresentationFeedback: Send;
    /// The std::unique_ptr<DrawImage> of ProxyImpl::QueueImageDecodeOnImpl.
    type ProxyDrawImage: Send;
    /// Rechrom: the impl notifications Chromium delivers to the main thread
    /// outside ProxyMain (ReportTimeSwapPromise's posted callbacks,
    /// cc/trees/swap_promise.h; LayerTreeFrameSink DidNotProduceFrame; the
    /// video frame provider's BeginFrames for VideoFrameSubmitter; custom
    /// tracker metrics). They share the ProxyImpl -> ProxyMain queue as
    /// `ImplToMain::EmbedderNotification`, so CompositorEngine's outputs
    /// carry no main-thread effects.
    type EmbedderNotification: Send;
}

/// cc::MutatorEvents: the animation events of the impl AnimationHost for the
/// main instance (LayerTreeHost::ApplyMutatorEvents).
pub type MutatorEvents<F> = <<F as crate::commit::CommitStateTypes>::MutatorHostHandle as crate::commit::main_mutator::MainMutatorHost>::Events;

/// Source: cc/trees/proxy_common.h:20-39 (BeginMainFrameAndCommitState),
/// filled by ProxyImpl::ScheduledActionSendBeginMainFrame
/// (proxy_impl.cc:808-830). `reason` (BeginMainFrameReasons, metrics only)
/// is not translated.
pub struct BeginMainFrameAndCommitState<F: ProxyTypes> {
    pub begin_frame_args: BeginFrameArgs,
    pub commit_data: Box<CompositorCommitData>,
    pub completed_image_decode_requests: Vec<(i32, bool)>,
    pub mutator_events: Option<MutatorEvents<F>>,
    /// Bit encoding of the FrameSequenceTrackerType for active trackers.
    pub active_sequence_trackers: u64,
    pub evicted_ui_resources: bool,
    pub trace_id: i64,
    pub consecutive_no_damage_main_frames: i32,
}

/// Source: cc/trees/proxy_main.h:50-55.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommitPipelineStage {
    NO_PIPELINE_STAGE,
    ANIMATE_PIPELINE_STAGE,
    UPDATE_LAYERS_PIPELINE_STAGE,
    COMMIT_PIPELINE_STAGE,
}

/// ProxyMain -> ProxyImpl posted tasks (cc/trees/proxy_impl.h:75-110).
pub enum MainToImpl<F: ProxyTypes> {
    /// ProxyMain::InitializeOnImplThread (proxy_main.cc:80-89).
    Initialize {
        completion: CompletionSignal,
        init: F::ImplInit,
    },
    /// ProxyImpl::SetNeedsCommitOnImpl (proxy_impl.cc:294-298); the
    /// metrics-only BeginMainFrameReason is not translated.
    SetNeedsCommit { urgent: bool, unthrottle_next: bool },
    /// ProxyImpl::SetNeedsRedrawOnImpl.
    SetNeedsRedraw(Rect),
    /// ProxyImpl::SetDeferBeginMainFrameFromMain (proxy_impl.cc:232-243).
    SetDeferBeginMainFrame(bool),
    /// ProxyImpl::SetPauseRendering.
    SetPauseRendering {
        pause_rendering: bool,
        delay_until_visibility_change: bool,
    },
    /// ProxyImpl::SetVisibleOnImpl (proxy_impl.cc:325).
    SetVisible(bool),
    /// ProxyImpl::SetTargetLocalSurfaceIdOnImpl.
    SetTargetLocalSurfaceId(LocalSurfaceId),
    /// ProxyImpl::NotifyNewLocalSurfaceIdExpectedWhilePaused.
    NotifyNewLocalSurfaceIdExpectedWhilePaused,
    /// ProxyImpl::BeginMainFrameAbortedOnImpl (proxy_impl.cc:306-323).
    BeginMainFrameAborted {
        reason: CommitEarlyOutReason,
        main_thread_start_time: TimeTicks,
        swap_promises: Vec<F::ProxySwapPromise>,
        scroll_and_viewport_changes_synced: bool,
    },
    /// ProxyImpl::NotifyReadyToCommitOnImpl (proxy_impl.cc:425-493). The
    /// completion event travels inside the lease.
    NotifyReadyToCommit {
        lease: ProtectedCommitLease<F>,
        commit_state: Box<CommitState<F>>,
        main_thread_start_time: TimeTicks,
        scroll_and_viewport_changes_synced: bool,
        commit_timeout: bool,
    },
    /// ProxyImpl::InitializeMutatorOnImpl (proxy_impl.cc:185-190).
    SetMutator(F::LayerTreeMutator),
    /// ProxyImpl::QueueImageDecodeOnImpl.
    QueueImageDecode {
        request_id: i32,
        image: F::ProxyDrawImage,
        speculative: bool,
    },
    /// ProxyMain::Stop -> DestroyProxyImplOnImplThread
    /// (proxy_main.cc:916-946).
    Stop { completion: CompletionSignal },
    /// Rechrom: the main LayerTreeHost's commit request notifies the
    /// LatencyInfoSwapPromiseMonitors, whose registry is owned by the impl
    /// half (cc/trees/latency_info_swap_promise_monitor.cc).
    NotifyLatencyInfoSwapPromiseMonitors,
}

/// The posted-task queue of one Proxy direction (the PostTask of
/// MainThreadTaskRunner / ImplThreadTaskRunner in proxy_main.cc and
/// proxy_impl.cc). Decision D1: the same type is the in-process queue of a
/// single-sequence embedding (drained by that embedding after each call) and
/// the cross-thread transport, whose poster hands every task to the
/// receiving sequence (S1b-7).
pub struct ProxyTaskQueue<T>(ProxyTaskQueueKind<T>);
enum ProxyTaskQueueKind<T> {
    InProcess(Arc<std::sync::Mutex<std::collections::VecDeque<T>>>),
    Posted(Arc<dyn Fn(T) -> Result<(), T> + Send + Sync>),
}
impl<T> Clone for ProxyTaskQueue<T> {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            ProxyTaskQueueKind::InProcess(queue) => ProxyTaskQueueKind::InProcess(queue.clone()),
            ProxyTaskQueueKind::Posted(post) => ProxyTaskQueueKind::Posted(post.clone()),
        })
    }
}
impl<T> Default for ProxyTaskQueue<T> {
    fn default() -> Self {
        Self(ProxyTaskQueueKind::InProcess(Default::default()))
    }
}
impl<T> ProxyTaskQueue<T> {
    /// The cross-thread form: `post` enqueues the task on the receiving
    /// sequence and hands it back when that sequence has stopped.
    pub fn Posting(post: Arc<dyn Fn(T) -> Result<(), T> + Send + Sync>) -> Self {
        Self(ProxyTaskQueueKind::Posted(post))
    }
    pub fn IsInProcess(&self) -> bool {
        matches!(self.0, ProxyTaskQueueKind::InProcess(_))
    }
    /// Returns the task when the receiving sequence has stopped; its
    /// completion guards then signal as abandoned when it is dropped.
    pub fn PostTask(&self, task: T) -> Result<(), T> {
        match &self.0 {
            ProxyTaskQueueKind::InProcess(queue) => {
                queue.lock().expect("proxy task queue").push_back(task);
                Ok(())
            }
            ProxyTaskQueueKind::Posted(post) => post(task),
        }
    }
    /// The next task of an in-process queue (always `None` when posting).
    pub fn TakeTask(&self) -> Option<T> {
        match &self.0 {
            ProxyTaskQueueKind::InProcess(queue) => {
                queue.lock().expect("proxy task queue").pop_front()
            }
            ProxyTaskQueueKind::Posted(_) => None,
        }
    }
}

/// The CompletionEvent of a posted Initialize/Stop task, which ProxyMain
/// waits for (proxy_main.cc:898-946). Rust extension: a task dropped without
/// having run (the impl sequence stopped) signals it as abandoned, so the
/// waiting main thread is released instead of deadlocking.
pub struct CompletionSignal(Option<CompletionEvent>);
impl CompletionSignal {
    pub fn new(event: CompletionEvent) -> Self {
        Self(Some(event))
    }
    pub fn Signal(mut self) {
        if let Some(event) = self.0.take() {
            event.Signal();
        }
    }
}
impl Drop for CompletionSignal {
    fn drop(&mut self) {
        if let Some(event) = self.0.take() {
            event.SignalAbandoned();
        }
    }
}

/// ProxyImpl -> ProxyMain posted tasks (cc/trees/proxy_main.h:57-86).
pub enum ImplToMain<F: ProxyTypes> {
    BeginMainFrame(Box<BeginMainFrameAndCommitState<F>>),
    DidCompleteCommit {
        source_frame_number: i32,
        timestamps: CommitTimestamps,
    },
    DidCommitAndDrawFrame(i32),
    DidPresentCompositorFrame {
        frame_token: u32,
        feedback: F::PresentationFeedback,
    },
    /// Source: cc/trees/proxy_main.h NotifyImageDecodeRequestFinished
    /// (request_id, speculative, decode_succeeded).
    NotifyImageDecodeRequestFinished {
        request_id: i32,
        speculative: bool,
        decode_succeeded: bool,
    },
    NotifyTransitionRequestFinished {
        sequence_id: u32,
        rects: crate::runtime::main_host::ViewTransitionElementResourceRects<F>,
    },
    DidCompletePageScaleAnimation,
    DidChangeBeginFrameSourcePaused(bool),
    BeginMainFrameNotExpectedSoon,
    BeginMainFrameNotExpectedUntil(TimeTicks),
    DidLoseLayerTreeFrameSink,
    /// Rechrom: see `ProxyTypes::EmbedderNotification`.
    EmbedderNotification(F::EmbedderNotification),
}
