//! ProxyImpl: the impl-thread half of cc's threaded Proxy
//! (cc/trees/proxy_impl.{h,cc} at 6c1d401fcca5e1b0030563a90c2f2bba168e0c15),
//! S1b-4 of S1B_PROXY_DESIGN.md.
//!
//! As in Chromium, the impl owner of LayerTreeHostImpl and the Scheduler
//! client is ProxyImpl; here that is `EngineCore`, whose SchedulerClient
//! callbacks touch only impl state. Their main-thread work is posted to
//! `layerize::runtime::proxy_main::ProxyMain` as `ImplToMain` tasks, and
//! ProxyMain answers with `MainToImpl` tasks. Both halves still share one
//! sequence (decision D1): `EngineCore::RunProxyTasks` is the in-process
//! transport and drains both queues at the posting call site, so every task
//! runs where the single-thread engine ran the same work.
use super::{
    engine_core::EngineCore, execution::EngineMainHostEffects, skia::SkRegion,
    types::CpuCompositorTypes,
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs, client_layer_tree_host_impl::ClientHostContext,
        layer_tree_host_impl::HostImplContext,
    },
    cc::scheduler::{commit_earlyout_reason::CommitEarlyOutReason, scheduler::Scheduler},
    engine::CompositorMainThreadEffect,
    performance::{CompositorPackage, Span},
};
use layerize::{
    commit::{
        commit_state::{CommitState, CommitTimestamps},
        completion_event::CompletionEvent,
        compositor_commit_data::CompositorCommitData,
        paint_holding::PaintHoldingReason,
        proxy::{
            BeginMainFrameAndCommitState, ImplToMain, MainToImpl, ProtectedCommitLease,
            ProxyTaskQueue,
        },
    },
    runtime::proxy_main::ProxyMainClient,
};
type Root = CpuCompositorTypes<SkRegion>;

/// Ensures that the commit CompletionEvent is always signaled.
/// Source: cc/trees/proxy_impl.cc:81-120. Rust extension (S1b-2): a commit
/// dropped before FinishCommit ran signals the event as abandoned.
pub(crate) struct ScopedCommitCompletionEvent {
    source_frame_number: i32,
    event: Option<CompletionEvent>,
    commit_timestamps: CommitTimestamps,
    notify_main: bool,
    main_thread_task_runner: ProxyTaskQueue<ImplToMain<Root>>,
    finished: bool,
}
impl ScopedCommitCompletionEvent {
    fn new(
        source_frame_number: i32,
        event: CompletionEvent,
        start_time: crate::adapter::time::TimeTicks,
        main_thread_task_runner: ProxyTaskQueue<ImplToMain<Root>>,
        notify_main: bool,
    ) -> Self {
        Self {
            source_frame_number,
            event: Some(event),
            commit_timestamps: CommitTimestamps {
                start: start_time,
                finish: Default::default(),
            },
            notify_main,
            main_thread_task_runner,
            finished: false,
        }
    }
    fn SetFinishTime(&mut self, finish_time: crate::adapter::time::TimeTicks) {
        self.commit_timestamps.finish = finish_time;
        self.finished = true;
    }
}
impl Drop for ScopedCommitCompletionEvent {
    fn drop(&mut self) {
        if let Some(event) = self.event.take() {
            if self.finished {
                event.Signal();
            } else {
                event.SignalAbandoned();
            }
        }
        if self.notify_main {
            self.main_thread_task_runner
                .PostTask(ImplToMain::DidCompleteCommit {
                    source_frame_number: self.source_frame_number,
                    timestamps: self.commit_timestamps,
                });
        }
    }
}

/// Source: cc/trees/proxy_impl.h:243-261. The protected lease is the
/// `unsafe_state` pointer; `commit_timestamps` is not translated because
/// the main thread does not block on the commit (decision D5).
struct DataForCommit {
    commit_completion_event: ScopedCommitCompletionEvent,
    commit_state: Box<CommitState<Root>>,
    unsafe_state: ProtectedCommitLease<Root>,
}

/// The ProxyImpl members that are not LayerTreeHostImpl or Scheduler
/// (proxy_impl.h:225-290).
pub(crate) struct ProxyImpl {
    main_thread_task_runner: ProxyTaskQueue<ImplToMain<Root>>,
    data_for_commit: Option<DataForCommit>,
    activation_completion_event: Option<ScopedCommitCompletionEvent>,
    main_wants_defer_begin_main_frame: bool,
    impl_wants_defer_begin_main_frame: bool,
    // MainToImpl::ReadyCommitAwaitsBeginMainFrame (B1 bridge, S1b-7 deletes
    // it): the commit ProxyMain holds for the next BeginMainFrame predates
    // the deltas that BeginMainFrame pulls, which therefore belong to the
    // pipelined slot exactly as with `data_for_commit_`.
    ready_commit_awaits_begin_main_frame: bool,
}
impl ProxyImpl {
    pub(crate) fn new() -> Self {
        Self {
            main_thread_task_runner: Default::default(),
            data_for_commit: None,
            activation_completion_event: None,
            main_wants_defer_begin_main_frame: false,
            impl_wants_defer_begin_main_frame: false,
            ready_commit_awaits_begin_main_frame: false,
        }
    }
    pub(crate) fn HasDataForCommit(&self) -> bool {
        self.data_for_commit.is_some()
    }
    // Source: cc/trees/proxy_impl.cc ShouldDeferBeginMainFrame.
    fn ShouldDeferBeginMainFrame(&self) -> bool {
        self.main_wants_defer_begin_main_frame || self.impl_wants_defer_begin_main_frame
    }
}

/// The in-process LayerTreeHost client side of ProxyMain: the Page-bound
/// compositor main-thread effects and the main AnimationHost, which stays a
/// compositor owner until S1b-5.
pub(crate) struct InProcessMainClient<'a> {
    pub(crate) effects: &'a mut Vec<CompositorMainThreadEffect>,
    pub(crate) main_effects: &'a EngineMainHostEffects,
}
impl ProxyMainClient<Root> for InProcessMainClient<'_> {
    fn CommitDeferralChanged(&mut self, deferred: bool, reason: PaintHoldingReason) {
        self.effects
            .push(CompositorMainThreadEffect::CommitDeferralChanged { deferred, reason });
    }
    fn ViewportScaleChanged(&mut self, scale: f32, data: &CompositorCommitData) {
        self.effects
            .push(CompositorMainThreadEffect::ViewportScaleChanged {
                scale,
                delta: data.page_scale_delta,
                pinch_active: data.is_pinch_gesture_active,
                inner_scroll_delta: data.inner_viewport_scroll.scroll_delta,
            });
    }
    // ProxyMain.cc:351-352 -> AnimationHost.cc:795-822.
    fn ApplyMutatorEvents(&mut self, events: Option<Box<super::host_owners::MutatorEvents>>) {
        let animations = self.main_effects.Animations();
        if let Some(events) = events {
            animations.ApplySourceAnimationEvents(events);
        }
        for completion in animations.TakeSourceScrollCompletions() {
            self.effects
                .push(CompositorMainThreadEffect::ScrollAnimationCompleted(
                    completion,
                ));
        }
    }
    fn BeginMainFrame(&mut self) {
        self.effects.push(CompositorMainThreadEffect::BeginMainFrame);
    }
    // The finish timestamp belongs to FinishCommit, before the potentially
    // expensive ScheduledActionPostCommit and the independently scheduled
    // draw (proxy_impl.cc ScheduledActionCommit).
    fn DidCommit(&mut self, _source_frame_number: i32, timestamps: CommitTimestamps) {
        self.effects
            .push(CompositorMainThreadEffect::MainFrameCommitCompleted {
                commit_finish_time: timestamps.finish,
            });
    }
}

impl EngineCore {
    /// The in-process transport (decision D1): runs every posted Proxy task
    /// of both directions, in posting order, on this sequence.
    pub(crate) fn RunProxyTasks(&mut self, scheduler: &mut Scheduler) {
        loop {
            if let Some(task) = self.proxy_impl.main_thread_task_runner.TakeTask() {
                let now = self.services.core.MonotonicTimeNow();
                let mut client = InProcessMainClient {
                    effects: &mut self.effects,
                    main_effects: &self.main_effects,
                };
                self.proxy_main
                    .RunImplTask(task, now, &mut self.layerize, &mut client);
            } else if let Some(task) = self.proxy_main.TakeImplTask() {
                self.RunMainThreadTask(task, scheduler);
            } else {
                break;
            }
        }
    }

    fn RunMainThreadTask(&mut self, task: MainToImpl<Root>, scheduler: &mut Scheduler) {
        match task {
            MainToImpl::NotifyReadyToCommit {
                lease,
                commit_state,
                main_thread_start_time,
                scroll_and_viewport_changes_synced,
                commit_timeout,
            } => self.NotifyReadyToCommitOnImpl(
                lease,
                commit_state,
                main_thread_start_time,
                scroll_and_viewport_changes_synced,
                commit_timeout,
                scheduler,
            ),
            MainToImpl::BeginMainFrameAborted {
                reason,
                main_thread_start_time,
                swap_promises,
                scroll_and_viewport_changes_synced,
            } => self.BeginMainFrameAbortedOnImpl(
                reason,
                main_thread_start_time,
                swap_promises,
                scroll_and_viewport_changes_synced,
                scheduler,
            ),
            // Source: proxy_impl.cc:294-298 -> SetNeedsCommitOnImplThread.
            MainToImpl::SetNeedsCommit {
                urgent,
                unthrottle_next,
            } => scheduler.SetNeedsBeginMainFrame(urgent, unthrottle_next, self),
            MainToImpl::SetDeferBeginMainFrame(defer) => {
                self.SetDeferBeginMainFrameFromMain(defer, scheduler)
            }
            MainToImpl::ReadyCommitAwaitsBeginMainFrame => {
                self.proxy_impl.ready_commit_awaits_begin_main_frame = true;
                scheduler.SetNeedsBeginMainFrame(false, false, self);
            }
            _ => unreachable!("ProxyMain posts this task from S1b-7 on"),
        }
    }

    /// Source: cc/trees/proxy_impl.cc:801-848. NotifyBeginMainFrameStarted
    /// is still called by the Scheduler callback before this (as the
    /// single-thread engine did); it moves to NotifyReadyToCommitOnImpl /
    /// BeginMainFrameAbortedOnImpl with the main start time in S1b-7.
    pub(crate) fn SendBeginMainFrame(&mut self, args: &BeginFrameArgs) {
        let next_bmf = self.proxy_impl.data_for_commit.is_some()
            || std::mem::take(&mut self.proxy_impl.ready_commit_awaits_begin_main_frame);
        let host = self.host.as_mut().expect("resident Host").BaseMutable();
        let mut context = HostImplContext {
            services: &mut self.services,
        };
        let commit_data = host.ProcessCompositorDeltasForPipeline(next_bmf, &mut context);
        let mutator_events = host.TakeMutatorEvents(&mut context);
        let state = BeginMainFrameAndCommitState {
            begin_frame_args: args.clone(),
            commit_data,
            completed_image_decode_requests: Vec::new(),
            mutator_events,
            active_sequence_trackers: 0,
            evicted_ui_resources: false,
            trace_id: args.trace_id,
            consecutive_no_damage_main_frames: 0,
        };
        self.proxy_impl
            .main_thread_task_runner
            .PostTask(ImplToMain::BeginMainFrame(Box::new(state)));
    }

    /// Source: cc/trees/proxy_impl.cc:425-493.
    fn NotifyReadyToCommitOnImpl(
        &mut self,
        mut unsafe_state: ProtectedCommitLease<Root>,
        commit_state: Box<CommitState<Root>>,
        _main_thread_start_time: crate::adapter::time::TimeTicks,
        scroll_and_viewport_changes_synced: bool,
        commit_timeout: bool,
        scheduler: &mut Scheduler,
    ) {
        assert!(self.proxy_impl.data_for_commit.is_none());
        debug_assert!(scheduler.CommitPending());
        let start_time = self.services.core.MonotonicTimeNow();
        let query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
        self.host.as_mut().unwrap().ReadyToCommit(
            scroll_and_viewport_changes_synced,
            None,
            commit_timeout,
            &mut ClientHostContext {
                services: &mut self.services,
            },
        );
        drop(query);
        let notify_main = !unsafe_state.IsBlocking();
        self.proxy_impl.data_for_commit = Some(DataForCommit {
            commit_completion_event: ScopedCommitCompletionEvent::new(
                commit_state.source_frame_number,
                unsafe_state.TakeCompletion(),
                start_time,
                self.proxy_impl.main_thread_task_runner.clone(),
                notify_main,
            ),
            commit_state,
            unsafe_state,
        });
        scheduler.NotifyReadyToCommit(None, self);
        // proxy_impl.cc:488-492 requests another BeginMainFrame when the
        // scroll offsets were not synchronized. The only unsynchronized
        // commit before S1b-7 is the B1 commit finished outside a
        // BeginMainFrame, which the single-thread engine never followed with
        // an extra main frame; the request is added with B1's removal.
    }

    /// Source: cc/trees/proxy_impl.cc:306-323 (NotifyBeginMainFrameStarted:
    /// see SendBeginMainFrame).
    fn BeginMainFrameAbortedOnImpl(
        &mut self,
        reason: CommitEarlyOutReason,
        _main_thread_start_time: crate::adapter::time::TimeTicks,
        swap_promises: Vec<Box<super::host_owners::SwapPromise<SkRegion>>>,
        scroll_and_viewport_changes_synced: bool,
        scheduler: &mut Scheduler,
    ) {
        debug_assert!(scheduler.CommitPending());
        let args = scheduler.last_dispatched_begin_main_frame_args().clone();
        let next_bmf = self.proxy_impl.data_for_commit.is_some();
        self.host.as_mut().unwrap().BeginMainFrameAborted(
            reason,
            swap_promises,
            &args,
            next_bmf,
            scroll_and_viewport_changes_synced,
            &mut ClientHostContext {
                services: &mut self.services,
            },
        );
        scheduler.BeginMainFrameAborted(reason, self);
    }

    /// Source: cc/trees/proxy_impl.cc:868-899. The live main synchronization
    /// context borrows the very same resident ThreadUnsafeCommitState through
    /// the protected lease posted with ReadyToCommit.
    pub(crate) fn ScheduledActionCommitOnImpl(&mut self) {
        let _trace = Span(CompositorPackage::Tree, "Compositor.Tree.Commit");
        let DataForCommit {
            mut commit_completion_event,
            mut commit_state,
            mut unsafe_state,
        } = self
            .proxy_impl
            .data_for_commit
            .take()
            .expect("source ready-to-commit state");
        let committed_source_frame_number = commit_state.source_frame_number;
        let host = self.host.as_mut().unwrap();
        let mut context = ClientHostContext {
            services: &mut self.services,
        };
        host.BeginCommit(
            commit_state.source_frame_number,
            commit_state.trace_id,
            &mut context,
        );
        let trace = unsafe_state.PaintServices().clone();
        unsafe_state.WithCommitContext(|unsafe_state, layer, main_host| {
            let mut main_context = super::sync::MainSynchronizationContext {
                main: layer,
                host: main_host,
                trace: trace.as_ref(),
            };
            host.FinishCommit(
                &mut commit_state,
                unsafe_state,
                &mut main_context,
                &mut context,
            );
        });
        let finish_time = self.services.core.MonotonicTimeNow();
        commit_completion_event.SetFinishTime(finish_time);
        drop(unsafe_state);
        if commit_state.commit_waits_for_activation {
            // For some layer types in impl-side painting, the commit is held
            // until the sync tree is activated.
            self.proxy_impl.activation_completion_event = Some(commit_completion_event);
        } else {
            // Signals the completion and posts DidCompleteCommit.
            drop(commit_completion_event);
        }
        self.last_committed_source_frame_number = Some(committed_source_frame_number);
    }

    /// Source: cc/trees/proxy_impl.cc DidActivateSyncTree (:660-668).
    pub(crate) fn DidActivateSyncTreeOnImpl(&mut self) {
        self.proxy_impl.activation_completion_event = None;
    }

    /// Source: cc/trees/proxy_impl.cc SetDeferBeginMainFrameFromMain.
    fn SetDeferBeginMainFrameFromMain(&mut self, defer_begin_main_frame: bool, scheduler: &mut Scheduler) {
        let was_deferring = self.proxy_impl.ShouldDeferBeginMainFrame();
        self.proxy_impl.main_wants_defer_begin_main_frame = defer_begin_main_frame;
        let should_defer = self.proxy_impl.ShouldDeferBeginMainFrame();
        if was_deferring != should_defer {
            scheduler.SetDeferBeginMainFrame(should_defer, self);
        }
    }

    /// Source: cc/trees/proxy_impl.cc SetDeferBeginMainFrameFromImpl.
    pub(crate) fn SetDeferBeginMainFrameFromImpl(
        &mut self,
        defer_begin_main_frame: bool,
        scheduler: &mut Scheduler,
    ) {
        let was_deferring = self.proxy_impl.ShouldDeferBeginMainFrame();
        self.proxy_impl.impl_wants_defer_begin_main_frame = defer_begin_main_frame;
        let should_defer = self.proxy_impl.ShouldDeferBeginMainFrame();
        if was_deferring != should_defer {
            scheduler.SetDeferBeginMainFrame(should_defer, self);
        }
    }
}
