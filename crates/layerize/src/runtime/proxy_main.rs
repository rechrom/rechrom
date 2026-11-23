//! ProxyMain: the main-thread half of cc's threaded Proxy
//! (cc/trees/proxy_main.{h,cc} at 6c1d401fcca5e1b0030563a90c2f2bba168e0c15),
//! S1b-4 of S1B_PROXY_DESIGN.md.
//!
//! It owns the main-side state of one main frame and talks to ProxyImpl only
//! through posted `MainToImpl` tasks; ProxyImpl answers with `ImplToMain`
//! tasks. Both halves still share one sequence (decision D1): the
//! compositor's in-process transport drains the queues at the posting call
//! site, which keeps the order of the former single-thread engine.
//!
//! Rechrom split of ProxyMain::BeginMainFrame (proxy_main.cc:174-580): the
//! Page answers a BeginMainFrame asynchronously (it runs Style/Layout/Paint
//! after the BeginMainFrame effect and then commits an artifact or reports
//! no updates), so the part after the embedder lifecycle (UpdateLayers,
//! WillCommit, NotifyReadyToCommit or BeginMainFrameAborted) is the separate
//! `FinalizePreparedCommit` / `FinishMainFrameWithoutUpdates` /
//! `AbortDeferredMainFrame` entry points, with the open frame recorded in
//! `begin_main_frame`. S1b-7 makes it one synchronous call again.
use crate::{
    commit::{
        commit_state::{CommitState, CommitTimestamps},
        completion_event::CompletionEvent,
        compositor_commit_data::CompositorCommitData,
        paint_holding::{CommitDeferralReason, PaintHoldingReason},
        property_tree_services::{PropertyTreeFeatures, PropertyTreesOwnerId},
        proxy::{
            BeginMainFrameAndCommitState, CommitEarlyOutReason, ImplToMain, MainToImpl,
            ProtectedCommitLease, ProxyTaskQueue, ProxyTypes,
        },
    },
    layerize_engine::LayerizeEngine,
};
use compositor_scheduler::TimeTicks;

/// The LayerTreeHost / LayerTreeHostClient calls ProxyMain makes whose
/// owners are still outside layerize (the main AnimationHost until S1b-5,
/// and the Page, which receives them as compositor main-thread effects).
pub trait ProxyMainClient<F: ProxyTypes> {
    /// ProxyMain::StopDeferringCommits after the commit timeout
    /// (proxy_main.cc:331-335, 867-879).
    fn CommitDeferralChanged(&mut self, deferred: bool, reason: PaintHoldingReason);
    /// LayerTreeHostClient::ApplyViewportChanges after
    /// LayerTreeHost::ApplyCompositorChanges changed the page scale or the
    /// inner viewport scroll (layer_tree_host.cc:1207-1244).
    fn ViewportScaleChanged(&mut self, page_scale_factor: f32, data: &CompositorCommitData);
    /// LayerTreeHost::ApplyMutatorEvents (layer_tree_host.cc:1254) into the
    /// main AnimationHost.
    fn ApplyMutatorEvents(&mut self, events: Option<F::MutatorEvents>);
    /// LayerTreeHost::BeginMainFrame: the embedder lifecycle
    /// (proxy_main.cc:357-360).
    fn BeginMainFrame(&mut self);
    /// LayerTreeHost::CommitComplete -> LayerTreeHostClient::DidCommit
    /// (layer_tree_host.cc:552-575).
    fn DidCommit(&mut self, source_frame_number: i32, timestamps: CommitTimestamps);
}

/// The open BeginMainFrame between ProxyMain::BeginMainFrame and the Page's
/// answer (see the module comment).
struct OpenMainFrame {
    // proxy_main.cc:176 begin_main_frame_start_time.
    main_thread_start_time: TimeTicks,
    // proxy_main.cc:340-349.
    scroll_and_viewport_changes_synced: bool,
}

/// PaintArtifactCompositor::Update has run for the next commit; UpdateLayers
/// (when `update_layers`) and WillCommit follow in FinalizePreparedCommit.
/// Rechrom distinguishes Blink's property-only commit (direct
/// transform/opacity updates, which call only SetNeedsCommit) from a new
/// artifact: the former keeps the existing PictureLayer recording, so it
/// skips UpdateLayers instead of mapping onto final_pipeline_stage_.
struct PreparedCommit {
    update_layers: bool,
}

pub struct ProxyMain<F: ProxyTypes> {
    to_impl: ProxyTaskQueue<MainToImpl<F>>,
    begin_main_frame: Option<OpenMainFrame>,
    prepared_commit: Option<PreparedCommit>,
    // A commit finished outside a dispatched BeginMainFrame (blocker B1),
    // posted with NotifyReadyToCommit from the next BeginMainFrame.
    ready_commit: Option<(CommitState<F>, ProtectedCommitLease<F>)>,
}

impl<F: ProxyTypes> ProxyMain<F> {
    pub fn new(to_impl: ProxyTaskQueue<MainToImpl<F>>) -> Self {
        Self {
            to_impl,
            begin_main_frame: None,
            prepared_commit: None,
            ready_commit: None,
        }
    }

    /// The next task posted to ProxyImpl, for the in-process transport.
    pub fn TakeImplTask(&self) -> Option<MainToImpl<F>> {
        self.to_impl.TakeTask()
    }

    /// A BeginMainFrame was received and the Page has not answered it yet.
    pub fn IsInBeginMainFrame(&self) -> bool {
        self.begin_main_frame.is_some()
    }
    pub fn HasPreparedCommit(&self) -> bool {
        self.prepared_commit.is_some()
    }
    pub fn PreparedCommitUpdatesLayers(&self) -> bool {
        self.prepared_commit.as_ref().is_some_and(|prepared| prepared.update_layers)
    }
    /// A finished commit not yet posted to ProxyImpl.
    pub fn HasReadyCommit(&self) -> bool {
        self.ready_commit.is_some()
    }

    /// Runs one task ProxyImpl posted.
    pub fn RunImplTask(
        &mut self,
        task: ImplToMain<F>,
        now: TimeTicks,
        layerize: &mut LayerizeEngine<F>,
        client: &mut dyn ProxyMainClient<F>,
    ) {
        match task {
            ImplToMain::BeginMainFrame(state) => self.BeginMainFrame(state, now, layerize, client),
            ImplToMain::DidCompleteCommit {
                source_frame_number,
                timestamps,
            } => self.DidCompleteCommit(source_frame_number, timestamps, layerize, client),
            _ => unreachable!("ProxyImpl posts this task from S1b-7 on"),
        }
    }

    /// Source: cc/trees/proxy_main.cc:174-360 (up to the embedder lifecycle).
    /// The visibility and DeferMainFrameUpdate early-outs stay with the
    /// Page's commit entry (`AbortDeferredMainFrame`) until S1b-7.
    fn BeginMainFrame(
        &mut self,
        state: Box<BeginMainFrameAndCommitState<F>>,
        now: TimeTicks,
        layerize: &mut LayerizeEngine<F>,
        client: &mut dyn ProxyMainClient<F>,
    ) {
        let mut state = *state;
        // proxy_main.cc:328-335: stop deferring commits after the timeout.
        if let Some(reason) = layerize.MainFrameLifecycleMut().Expire(now) {
            client.CommitDeferralChanged(false, reason);
        }
        // proxy_main.cc:340-349.
        let mut scroll_and_viewport_changes_synced = false;
        if !layerize.MainFrameLifecycle().IsDeferringCommits() {
            if let Some(scale) = layerize.ApplyCompositorChanges(&mut state.commit_data) {
                client.ViewportScaleChanged(scale, &state.commit_data);
            }
            scroll_and_viewport_changes_synced = true;
        }
        // proxy_main.cc:351-352.
        client.ApplyMutatorEvents(state.mutator_events.take());
        let frame = OpenMainFrame {
            main_thread_start_time: now,
            scroll_and_viewport_changes_synced,
        };
        if self.prepared_commit.is_some() {
            // The Page is between PAC::Update and the scroll-animation
            // update of this very main frame; FinalizePreparedCommit answers.
            self.begin_main_frame = Some(frame);
        } else if let Some((commit_state, lease)) = self.ready_commit.take() {
            // B1: the commit finished before this BeginMainFrame; its
            // snapshot predates the deltas applied above.
            self.to_impl.PostTask(MainToImpl::NotifyReadyToCommit {
                lease,
                commit_state: Box::new(commit_state),
                main_thread_start_time: frame.main_thread_start_time,
                scroll_and_viewport_changes_synced: false,
                commit_timeout: false,
            });
        } else {
            self.begin_main_frame = Some(frame);
            client.BeginMainFrame();
        }
    }

    /// Paint ran PaintArtifactCompositor::Update for the next commit.
    pub fn PrepareCommit(&mut self, update_layers: bool) {
        assert!(
            self.prepared_commit.is_none() && self.ready_commit.is_none(),
            "previous main commit awaits scheduler"
        );
        self.prepared_commit = Some(PreparedCommit { update_layers });
    }

    /// Source: cc/trees/proxy_main.cc:439-580 after the embedder lifecycle:
    /// UpdateLayers (when requested), WillCommit with the MANUAL-reset
    /// completion (:470-477), then NotifyReadyToCommitOnImpl (:550-560). A
    /// commit finished outside a BeginMainFrame waits for the next one (B1).
    /// `has_updates` stays true until S1b-7 (decision D3).
    pub fn FinalizePreparedCommit(
        &mut self,
        layerize: &mut LayerizeEngine<F>,
        features: &dyn PropertyTreeFeatures,
        owner: PropertyTreesOwnerId,
    ) {
        let prepared = self
            .prepared_commit
            .take()
            .expect("same PAC preparation before finalizing native animation commit");
        // block_on_next_commit_ stays false until the input path sets it
        // (decision D5).
        let (commit_state, lease) = layerize.ActivateCommitState(
            prepared.update_layers,
            features,
            owner,
            CompletionEvent::new(),
            false,
        );
        if let Some(frame) = self.begin_main_frame.take() {
            // The Page's commit answers this BeginMainFrame after its own
            // delta application, so the commit is reported as synced
            // (proxy_main.cc:556-560).
            self.to_impl.PostTask(MainToImpl::NotifyReadyToCommit {
                lease,
                commit_state: Box::new(commit_state),
                main_thread_start_time: frame.main_thread_start_time,
                scroll_and_viewport_changes_synced: true,
                commit_timeout: false,
            });
        } else {
            self.ready_commit = Some((commit_state, lease));
            self.to_impl
                .PostTask(MainToImpl::ReadyCommitAwaitsBeginMainFrame);
        }
    }

    /// Source: cc/trees/proxy_main.cc:482-521 (WillCommit returned no commit
    /// state): kFinishedNoUpdates with synchronized scroll and viewport.
    pub fn FinishMainFrameWithoutUpdates(&mut self) {
        let frame = self
            .begin_main_frame
            .take()
            .expect("a BeginMainFrame awaits a no-update response");
        self.to_impl.PostTask(MainToImpl::BeginMainFrameAborted {
            reason: CommitEarlyOutReason::kFinishedNoUpdates,
            main_thread_start_time: frame.main_thread_start_time,
            swap_promises: Vec::new(),
            scroll_and_viewport_changes_synced: true,
        });
    }

    /// Source: cc/trees/proxy_main.cc:281-310 (DeferMainFrameUpdate) and
    /// :380-427 (deferred commits). Outside a BeginMainFrame the deferred
    /// request is kept as a commit request (SendCommitRequestToImplThread).
    pub fn AbortDeferredMainFrame(&mut self, reason: CommitDeferralReason) {
        if let Some(frame) = self.begin_main_frame.take() {
            let early_out = match reason {
                CommitDeferralReason::MainFrameUpdates => {
                    CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate
                }
                CommitDeferralReason::PaintHolding(_) => CommitEarlyOutReason::kAbortedDeferredCommit,
            };
            self.to_impl.PostTask(MainToImpl::BeginMainFrameAborted {
                reason: early_out,
                main_thread_start_time: frame.main_thread_start_time,
                swap_promises: Vec::new(),
                scroll_and_viewport_changes_synced: frame.scroll_and_viewport_changes_synced,
            });
        } else {
            self.SetNeedsCommit();
        }
    }

    /// ProxyMain::SetNeedsCommit -> SendCommitRequestToImplThreadIfNeeded
    /// (proxy_main.cc:741-756, 1112-1144). Every request is posted: the
    /// max_requested_pipeline_stage_ de-duplication joins with the
    /// LayerTreeHost request bits in S1b-5.
    pub fn SetNeedsCommit(&mut self) {
        self.to_impl.PostTask(MainToImpl::SetNeedsCommit {
            urgent: false,
            unthrottle_next: false,
        });
    }

    /// Source: cc/trees/proxy_main.cc:792-815. Returns whether the request
    /// changed; ProxyImpl combines it with the impl-side request.
    pub fn SetDeferMainFrameUpdate(
        &mut self,
        defer_main_frame_update: bool,
        layerize: &mut LayerizeEngine<F>,
    ) -> bool {
        if !layerize
            .MainFrameLifecycleMut()
            .SetDeferMainFrameUpdates(defer_main_frame_update)
        {
            return false;
        }
        self.to_impl
            .PostTask(MainToImpl::SetDeferBeginMainFrame(defer_main_frame_update));
        true
    }

    /// Source: cc/trees/proxy_main.cc:597-604.
    fn DidCompleteCommit(
        &mut self,
        source_frame_number: i32,
        timestamps: CommitTimestamps,
        layerize: &mut LayerizeEngine<F>,
        client: &mut dyn ProxyMainClient<F>,
    ) {
        layerize.CommitComplete();
        client.DidCommit(source_frame_number, timestamps);
    }
}
