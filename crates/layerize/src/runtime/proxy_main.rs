//! ProxyMain: the main-thread half of cc's threaded Proxy
//! (cc/trees/proxy_main.{h,cc} at 6c1d401fcca5e1b0030563a90c2f2bba168e0c15),
//! S1b-7 of S1B_PROXY_DESIGN.md.
//!
//! It owns the main-side state of the main-frame pipeline. Requests reach
//! ProxyImpl as posted `MainToImpl` tasks (`ProxyTaskQueue`: a cross-thread
//! poster when the impl half runs on the compositor thread, or the
//! in-process queue of a single-sequence embedding, decision D1).
//! ProxyMain::BeginMainFrame (proxy_main.cc:174-580) runs as one call of the
//! embedder's main-frame task and returns its answer to ProxyImpl
//! (NotifyReadyToCommitOnImpl or BeginMainFrameAbortedOnImpl) as a
//! `MainFrameResult` value, which the embedder hands to the impl half's
//! `CompositorEngine::ReadyToCommit` / `BeginMainFrameAborted`.
//!
//! The main LayerizeEngine is passed as a `RefCell`: the embedder lifecycle
//! that ProxyMain calls (`MainFrameClient::UpdateRendering`, Blink's
//! LayerTreeHost::BeginMainFrame + RequestMainFrameUpdate,
//! proxy_main.cc:357-367) itself borrows it for
//! PaintArtifactCompositor::Update, video ForeignLayers and direct animation
//! updates; ProxyMain borrows it only before and after that call.
use crate::{
    commit::{
        commit_state::{CommitState, CommitTimestamps},
        completion_event::CompletionEvent,
        compositor_commit_data::CompositorCommitData,
        main_callbacks::TaskRunner,
        main_effects::{MainHostEffect, MainHostEffectSink},
        paint_holding::PaintHoldingReason,
        property_tree_services::{PropertyTreeFeatures, PropertyTreesOwnerId},
        proxy::{
            BeginMainFrameAndCommitState, CommitEarlyOutReason, CommitPipelineStage,
            CompletionSignal, ImplToMain, MainToImpl, ProtectedCommitLease, ProxyTaskQueue,
            ProxyTypes,
        },
    },
    layerize_engine::LayerizeEngine,
};
use compositor_values::begin_frame::BeginFrameArgs;
use scroll_animation::TimeTicks;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, MutexGuard},
};

/// The LayerTreeHostClient (and AnimationDelegate) calls ProxyMain makes
/// into the embedder.
pub trait ProxyMainClient<F: ProxyTypes> {
    /// ProxyMain::StopDeferringCommits after the commit timeout
    /// (proxy_main.cc:331-335, 867-879).
    fn CommitDeferralChanged(&mut self, deferred: bool, reason: PaintHoldingReason);
    /// LayerTreeHostClient::ApplyViewportChanges after
    /// LayerTreeHost::ApplyCompositorChanges changed the page scale or the
    /// inner viewport scroll (layer_tree_host.cc:1207-1244).
    fn ApplyViewportChanges(&mut self, page_scale_factor: f32, data: &CompositorCommitData);
    /// The cc::AnimationDelegate notifications of the source scroll
    /// animations that LayerTreeHost::ApplyMutatorEvents finished
    /// (layer_tree_host.cc:1254; animation_host.cc:795-822).
    fn ScrollAnimationsFinished(
        &mut self,
        completions: Vec<scroll_animation::compositor_mutation::ScrollAnimationCompletion>,
    );
    /// LayerTreeHost::CommitComplete -> LayerTreeHostClient::DidCommit
    /// (layer_tree_host.cc:552-575).
    fn DidCommit(&mut self, source_frame_number: i32, timestamps: CommitTimestamps);
    /// The ProxyImpl -> ProxyMain notifications that ProxyMain forwards to
    /// LayerTreeHost(Client) unchanged.
    fn ImplNotification(&mut self, task: ImplToMain<F>);
    /// cc::ScrollCallbacks (PAC's DidCompositorScroll,
    /// DidChangeScrollbarsHidden) and LayerTreeHostClient::
    /// UpdateCompositorScrollState from ApplyCompositorChanges.
    fn DidCompositorScroll(&mut self, effect: crate::commit::main_effects::PacScrollEffect);
}

/// The embedder of a main frame (LayerTreeHostClient).
pub trait MainFrameClient<F: ProxyTypes>: ProxyMainClient<F> {
    /// base::TimeTicks::Now on the main thread.
    fn Now(&self) -> TimeTicks;
    /// LayerTreeHost::BeginMainFrame + RequestMainFrameUpdate
    /// (proxy_main.cc:357-367): the embedder lifecycle. Its Paint pushes the
    /// artifact with LayerizeEngine::UpdatePaintArtifact
    /// (LocalFrameView::PushPaintArtifactToCompositor).
    fn UpdateRendering(&mut self, args: &BeginFrameArgs);
}

/// How ProxyMain::BeginMainFrame ended (its answer was posted to ProxyImpl).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainFrameResult {
    /// NotifyReadyToCommitOnImpl was posted for this source frame.
    ReadyToCommit { source_frame_number: i32 },
    /// BeginMainFrameAbortedOnImpl was posted with this reason.
    Aborted(CommitEarlyOutReason),
}

/// The answer of ProxyMain::BeginMainFrame to ProxyImpl.
enum MainFrameAnswer<F: ProxyTypes> {
    /// ProxyImpl::NotifyReadyToCommitOnImpl (proxy_main.cc:550-560).
    ReadyToCommit {
        commit_state: Box<CommitState<F>>,
        lease: ProtectedCommitLease<F>,
        main_thread_start_time: TimeTicks,
        scroll_and_viewport_changes_synced: bool,
        commit_timeout: bool,
    },
    /// ProxyImpl::BeginMainFrameAbortedOnImpl.
    Aborted {
        reason: CommitEarlyOutReason,
        main_thread_start_time: TimeTicks,
        swap_promises: Vec<F::ProxySwapPromise>,
        scroll_and_viewport_changes_synced: bool,
    },
}
impl<F: ProxyTypes> MainFrameAnswer<F> {
    fn Aborted(reason: CommitEarlyOutReason, start: TimeTicks, synced: bool) -> Self {
        Self::Aborted {
            reason,
            main_thread_start_time: start,
            swap_promises: Vec::new(),
            scroll_and_viewport_changes_synced: synced,
        }
    }
}

/// Source: proxy_main.h max_requested_pipeline_stage_,
/// current_pipeline_stage_, final_pipeline_stage_,
/// deferred_final_pipeline_stage_. Shared with the LayerTreeHost's Proxy
/// endpoint, which calls SetNeedsCommit / SetNeedsUpdateLayers from inside
/// the main-frame lifecycle (PaintArtifactCompositor::Update, Layer setters).
struct PipelineState {
    max_requested: CommitPipelineStage,
    current: CommitPipelineStage,
    final_stage: CommitPipelineStage,
    deferred_final: CommitPipelineStage,
    has_sent_urgent_commit_request: bool,
}

/// The LayerTreeHost -> Proxy side of ProxyMain (`MainHostEffectSink`):
/// the request bits of the pipeline and the posted impl tasks. It is the
/// owner-thread identity of the main LayerTreeHost
/// (LayerTreeHost::IsOwnerThread, layer_tree_host.cc:329-332).
pub struct ProxyMainEndpoint<F: ProxyTypes> {
    to_impl: ProxyTaskQueue<MainToImpl<F>>,
    pipeline: Mutex<PipelineState>,
    owner: std::thread::ThreadId,
    task_runner: TaskRunner,
    trace: Arc<dyn Fn(bool) + Send + Sync>,
    // LayerTreeHostClient notifications the embedder drains after the call
    // that produced them (SaveActiveEventMetrics,
    // UpdateDeferMainFrameUpdate).
    embedder_effects: Mutex<Vec<MainHostEffect>>,
}

impl<F: ProxyTypes> ProxyMainEndpoint<F> {
    pub fn new(
        to_impl: ProxyTaskQueue<MainToImpl<F>>,
        task_runner: TaskRunner,
        trace: Arc<dyn Fn(bool) + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            to_impl,
            pipeline: Mutex::new(PipelineState {
                max_requested: CommitPipelineStage::NO_PIPELINE_STAGE,
                current: CommitPipelineStage::NO_PIPELINE_STAGE,
                final_stage: CommitPipelineStage::NO_PIPELINE_STAGE,
                deferred_final: CommitPipelineStage::NO_PIPELINE_STAGE,
                has_sent_urgent_commit_request: false,
            }),
            owner: std::thread::current().id(),
            task_runner,
            trace,
            embedder_effects: Mutex::new(Vec::new()),
        })
    }
    fn pipeline(&self) -> MutexGuard<'_, PipelineState> {
        self.pipeline
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub fn Post(&self, task: MainToImpl<F>) {
        // A stopped impl sequence drops the task; its completion guards and
        // commit lease signal as abandoned.
        let _ = self.to_impl.PostTask(task);
    }
    /// Source: cc/trees/proxy_main.cc:1112-1144. The throttling-related
    /// unthrottle bits are not translated (`unthrottle_next` stays false,
    /// as before S1b-7).
    fn SendCommitRequestToImplThreadIfNeeded(
        &self,
        required_stage: CommitPipelineStage,
        urgent: bool,
    ) -> bool {
        debug_assert_ne!(required_stage, CommitPipelineStage::NO_PIPELINE_STAGE);
        {
            let mut pipeline = self.pipeline();
            let already_posted = pipeline.max_requested != CommitPipelineStage::NO_PIPELINE_STAGE;
            pipeline.max_requested = pipeline.max_requested.max(required_stage);
            if already_posted && (!urgent || pipeline.has_sent_urgent_commit_request) {
                return false;
            }
            pipeline.has_sent_urgent_commit_request |= urgent;
        }
        self.Post(MainToImpl::SetNeedsCommit {
            urgent,
            unthrottle_next: false,
        });
        true
    }
    /// Source: cc/trees/proxy_main.cc:715-723 (ProxyMain::SetNeedsAnimate).
    pub fn SetNeedsAnimate(&self, urgent: bool) {
        self.SendCommitRequestToImplThreadIfNeeded(
            CommitPipelineStage::ANIMATE_PIPELINE_STAGE,
            urgent,
        );
    }
    /// Source: cc/trees/proxy_main.cc:725-739.
    pub fn SetNeedsUpdateLayers(&self) {
        {
            let mut pipeline = self.pipeline();
            if pipeline.current == CommitPipelineStage::ANIMATE_PIPELINE_STAGE {
                pipeline.final_stage = pipeline
                    .final_stage
                    .max(CommitPipelineStage::UPDATE_LAYERS_PIPELINE_STAGE);
                return;
            }
        }
        self.SendCommitRequestToImplThreadIfNeeded(
            CommitPipelineStage::UPDATE_LAYERS_PIPELINE_STAGE,
            false,
        );
    }
    /// Source: cc/trees/proxy_main.cc:741-756.
    pub fn SetNeedsCommit(&self, urgent: bool) {
        {
            let mut pipeline = self.pipeline();
            if pipeline.current == CommitPipelineStage::ANIMATE_PIPELINE_STAGE {
                pipeline.final_stage = pipeline
                    .final_stage
                    .max(CommitPipelineStage::COMMIT_PIPELINE_STAGE);
                return;
            }
        }
        self.SendCommitRequestToImplThreadIfNeeded(
            CommitPipelineStage::COMMIT_PIPELINE_STAGE,
            urgent,
        );
    }
    /// Source: cc/trees/proxy_main.cc:895-899 (ProxyMain::CommitRequested).
    pub fn CommitRequested(&self) -> bool {
        let pipeline = self.pipeline();
        pipeline.current != CommitPipelineStage::NO_PIPELINE_STAGE
            || pipeline.max_requested == CommitPipelineStage::COMMIT_PIPELINE_STAGE
    }
    /// The LayerTreeHostClient notifications produced since the last call.
    pub fn TakeEmbedderEffects(&self) -> Vec<MainHostEffect> {
        std::mem::take(
            &mut *self
                .embedder_effects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}

impl<F: ProxyTypes> MainHostEffectSink for ProxyMainEndpoint<F>
where
    MainToImpl<F>: Send,
{
    fn RunsTasksInCurrentSequence(&self) -> bool {
        std::thread::current().id() == self.owner
    }
    fn PostEffect(&self, effect: MainHostEffect) {
        match effect {
            MainHostEffect::NeedsUpdateLayers => self.SetNeedsUpdateLayers(),
            MainHostEffect::NeedsCommit { urgent } => self.SetNeedsCommit(urgent),
            MainHostEffect::NotifyLatencyInfoSwapPromiseMonitors => {
                self.Post(MainToImpl::NotifyLatencyInfoSwapPromiseMonitors)
            }
            // Source: cc/trees/proxy_main.cc SetTargetLocalSurfaceId,
            // NotifyNewLocalSurfaceIdExpectedWhilePaused.
            MainHostEffect::SetTargetLocalSurfaceId(id) => {
                self.Post(MainToImpl::SetTargetLocalSurfaceId(id))
            }
            MainHostEffect::NewLocalSurfaceIdExpectedWhilePaused => {
                self.Post(MainToImpl::NotifyNewLocalSurfaceIdExpectedWhilePaused)
            }
            effect @ (MainHostEffect::SaveActiveEventMetrics
            | MainHostEffect::UpdateDeferMainFrameUpdate) => self
                .embedder_effects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(effect),
        }
    }
    fn TraceCommitRequest(&self, urgent: bool) {
        (self.trace)(urgent)
    }
    fn MainThreadTaskRunner(&self) -> TaskRunner {
        self.task_runner.clone()
    }
}

pub struct ProxyMain<F: ProxyTypes> {
    endpoint: Arc<ProxyMainEndpoint<F>>,
    // LayerTreeHost::IsVisible (layer_tree_host.cc SetVisible).
    visible: bool,
    // proxy_main.h block_on_next_commit_ (decision D5: stays false).
    block_on_next_commit: bool,
    started: bool,
    // Identity of the property trees of the next commit's CommitState.
    next_property_owner: u64,
}

impl<F: ProxyTypes> ProxyMain<F>
where
    MainToImpl<F>: Send,
{
    pub fn new(endpoint: Arc<ProxyMainEndpoint<F>>, first_property_owner: u64) -> Self {
        Self {
            endpoint,
            // Source: cc/trees/layer_tree_host.h `bool visible_ = false;`.
            // The impl Scheduler also starts invisible; the widget's first
            // SetVisible(true) (WidgetBase::WasShown) posts SetVisibleOnImpl,
            // without which the Scheduler never sends a BeginMainFrame.
            visible: false,
            block_on_next_commit: false,
            started: false,
            next_property_owner: first_property_owner,
        }
    }
    pub fn Endpoint(&self) -> &Arc<ProxyMainEndpoint<F>> {
        &self.endpoint
    }

    /// Source: cc/trees/proxy_main.cc:898-914. The impl half is created on
    /// its own sequence from `init`; the main thread waits until it exists.
    /// A single-sequence embedding constructs its impl half itself and only
    /// marks the proxy started (`init == None`).
    pub fn Start(&mut self, init: Option<F::ImplInit>) {
        debug_assert!(!self.started);
        if let Some(init) = init {
            let completion = CompletionEvent::new();
            self.endpoint.Post(MainToImpl::Initialize {
                completion: CompletionSignal::new(completion.clone()),
                init,
            });
            completion.Wait();
        }
        self.started = true;
    }

    /// Source: cc/trees/proxy_main.cc:916-946. The impl half is destroyed on
    /// its sequence before the main LayerTreeHost; the main thread waits (an
    /// abandoned completion means the impl sequence had already stopped).
    pub fn Stop(&mut self) {
        if !std::mem::take(&mut self.started) || self.endpoint.to_impl.IsInProcess() {
            return;
        }
        let completion = CompletionEvent::new();
        self.endpoint.Post(MainToImpl::Stop {
            completion: CompletionSignal::new(completion.clone()),
        });
        completion.Wait();
    }

    /// Source: cc/trees/proxy_main.cc:701-706.
    pub fn SetVisible(&mut self, visible: bool) {
        if self.visible == visible {
            return;
        }
        self.visible = visible;
        self.endpoint.Post(MainToImpl::SetVisible(visible));
    }

    /// Source: cc/trees/proxy_main.cc:840-846.
    pub fn SetInputResponsePending(&mut self) {
        self.block_on_next_commit = true;
    }

    /// Source: cc/trees/proxy_main.cc:792-815. Returns whether the request
    /// changed.
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
        self.endpoint
            .Post(MainToImpl::SetDeferBeginMainFrame(defer_main_frame_update));
        true
    }

    /// Runs one ProxyImpl -> ProxyMain task other than BeginMainFrame, which
    /// is the embedder's main-frame task (`BeginMainFrame`).
    pub fn RunImplTask(
        &mut self,
        task: ImplToMain<F>,
        layerize: &mut LayerizeEngine<F>,
        client: &mut dyn ProxyMainClient<F>,
    ) {
        match task {
            ImplToMain::BeginMainFrame(_) => {
                unreachable!("BeginMainFrame is the embedder's main-frame task")
            }
            // Source: proxy_main.cc:597-604.
            ImplToMain::DidCompleteCommit {
                source_frame_number,
                timestamps,
            } => {
                layerize.CommitComplete();
                client.DidCommit(source_frame_number, timestamps);
            }
            // Source: proxy_main.cc NotifyTransitionRequestFinished ->
            // LayerTreeHost::NotifyTransitionRequestsFinished.
            ImplToMain::NotifyTransitionRequestFinished { sequence_id, rects } => {
                layerize.NotifyTransitionRequestsFinished(sequence_id, &rects)
            }
            task => client.ImplNotification(task),
        }
    }

    /// Source: cc/trees/proxy_main.cc:174-580. Translates `has_updates`
    /// (decision D3): UpdateLayers' result and the requested pipeline stage
    /// decide whether WillCommit produces a CommitState.
    pub fn BeginMainFrame(
        &mut self,
        state: Box<BeginMainFrameAndCommitState<F>>,
        layerize: &RefCell<LayerizeEngine<F>>,
        features: &dyn PropertyTreeFeatures,
        client: &mut dyn MainFrameClient<F>,
    ) -> MainFrameResult {
        let answer = self.RunBeginMainFrame(state, layerize, features, client);
        // PAC scroll callbacks fired by ApplyCompositorChanges reach the
        // embedder in the same task (cc::ScrollCallbacks).
        while let Some(effect) = layerize.borrow().PopScrollEffect() {
            client.DidCompositorScroll(effect);
        }
        match answer {
            MainFrameAnswer::ReadyToCommit {
                commit_state,
                lease,
                main_thread_start_time,
                scroll_and_viewport_changes_synced,
                commit_timeout,
            } => {
                let source_frame_number = commit_state.source_frame_number;
                // proxy_main.cc:550-560.
                self.endpoint.Post(MainToImpl::NotifyReadyToCommit {
                    lease,
                    commit_state,
                    main_thread_start_time,
                    scroll_and_viewport_changes_synced,
                    commit_timeout,
                });
                MainFrameResult::ReadyToCommit {
                    source_frame_number,
                }
            }
            MainFrameAnswer::Aborted {
                reason,
                main_thread_start_time,
                swap_promises,
                scroll_and_viewport_changes_synced,
            } => {
                self.endpoint.Post(MainToImpl::BeginMainFrameAborted {
                    reason,
                    main_thread_start_time,
                    swap_promises,
                    scroll_and_viewport_changes_synced,
                });
                MainFrameResult::Aborted(reason)
            }
        }
    }

    fn RunBeginMainFrame(
        &mut self,
        state: Box<BeginMainFrameAndCommitState<F>>,
        layerize: &RefCell<LayerizeEngine<F>>,
        features: &dyn PropertyTreeFeatures,
        client: &mut dyn MainFrameClient<F>,
    ) -> MainFrameAnswer<F> {
        let mut state = *state;
        assert_eq!(
            self.endpoint.pipeline().current,
            CommitPipelineStage::NO_PIPELINE_STAGE,
            "ProxyMain::BeginMainFrame is not reentrant"
        );
        let main_thread_start_time = client.Now();
        // proxy_main.cc:243-266.
        if !self.visible {
            return MainFrameAnswer::Aborted(
                CommitEarlyOutReason::kAbortedNotVisible,
                main_thread_start_time,
                false,
            );
        }
        // proxy_main.cc:268-271.
        {
            let mut pipeline = self.endpoint.pipeline();
            pipeline.final_stage = pipeline.max_requested;
            pipeline.max_requested = CommitPipelineStage::NO_PIPELINE_STAGE;
            pipeline.has_sent_urgent_commit_request = false;
        }
        // proxy_main.cc:273-303: deferred main frame updates skip the whole
        // pipeline and keep the requested stage.
        if layerize.borrow().MainFrameLifecycle().DeferMainFrameUpdates() {
            let mut pipeline = self.endpoint.pipeline();
            pipeline.deferred_final = pipeline.final_stage.max(pipeline.deferred_final);
            return MainFrameAnswer::Aborted(
                CommitEarlyOutReason::kAbortedDeferredMainFrameUpdate,
                main_thread_start_time,
                false,
            );
        }
        // proxy_main.cc:313-317.
        {
            let mut pipeline = self.endpoint.pipeline();
            pipeline.final_stage = pipeline.final_stage.max(pipeline.deferred_final);
            pipeline.deferred_final = CommitPipelineStage::NO_PIPELINE_STAGE;
            pipeline.current = CommitPipelineStage::ANIMATE_PIPELINE_STAGE;
        }
        let mut commit_timeout = false;
        let blocking;
        let mut scroll_and_viewport_changes_synced = false;
        {
            let mut main = layerize.borrow_mut();
            // proxy_main.cc:322-327.
            if let Some(reason) = main.MainFrameLifecycleMut().Expire(main_thread_start_time) {
                client.CommitDeferralChanged(false, reason);
                commit_timeout = true;
            }
            // proxy_main.cc:329-330.
            blocking = std::mem::take(&mut self.block_on_next_commit);
            // proxy_main.cc:332-343.
            if !main.MainFrameLifecycle().IsDeferringCommits() {
                if let Some(scale) = main.ApplyCompositorChanges(&mut state.commit_data) {
                    client.ApplyViewportChanges(scale, &state.commit_data);
                }
                scroll_and_viewport_changes_synced = true;
            }
            // proxy_main.cc:345-346.
            let finished = main.ApplyMutatorEvents(state.mutator_events.take());
            client.ScrollAnimationsFinished(finished);
            // The cc::ScrollCallbacks of ApplyCompositorChanges reach the
            // embedder before its lifecycle, in the same task.
            while let Some(effect) = main.PopScrollEffect() {
                client.DidCompositorScroll(effect);
            }
        }
        // proxy_main.cc:357-367: Blink's lifecycle, including Paint and
        // PaintArtifactCompositor::Update.
        client.UpdateRendering(&state.begin_frame_args);
        let mut main = layerize.borrow_mut();
        // proxy_main.cc:373-381.
        let skip_commit = main.MainFrameLifecycle().DeferMainFrameUpdates()
            || main.MainFrameLifecycle().IsDeferringCommits()
            || state.begin_frame_args.animate_only;
        if skip_commit {
            let mut pipeline = self.endpoint.pipeline();
            pipeline.current = CommitPipelineStage::NO_PIPELINE_STAGE;
            // proxy_main.cc:411-414.
            pipeline.deferred_final = pipeline.final_stage;
            return MainFrameAnswer::Aborted(
                CommitEarlyOutReason::kAbortedDeferredCommit,
                main_thread_start_time,
                scroll_and_viewport_changes_synced,
            );
        }
        // proxy_main.cc:434-460.
        let should_update_layers = {
            let mut pipeline = self.endpoint.pipeline();
            pipeline.current = CommitPipelineStage::UPDATE_LAYERS_PIPELINE_STAGE;
            pipeline.final_stage >= CommitPipelineStage::UPDATE_LAYERS_PIPELINE_STAGE
        };
        let updated = should_update_layers && main.UpdateLayers(features);
        let has_updates = {
            let mut pipeline = self.endpoint.pipeline();
            if updated {
                pipeline.final_stage = CommitPipelineStage::COMMIT_PIPELINE_STAGE;
            }
            // proxy_main.cc:466-470.
            pipeline.current = CommitPipelineStage::NO_PIPELINE_STAGE;
            pipeline.final_stage == CommitPipelineStage::COMMIT_PIPELINE_STAGE
        };
        let owner = PropertyTreesOwnerId::FromOwnerValue(self.next_property_owner);
        self.next_property_owner = self
            .next_property_owner
            .checked_add(1)
            .expect("native property-tree identity exhausted");
        // proxy_main.cc:472-478: MANUAL-reset completion; WillCommit stores
        // it and opens the protected sequence when it returns a CommitState.
        let completion = CompletionEvent::new();
        let Some((mut commit_state, lease)) =
            main.WillCommit(has_updates, owner, completion.clone(), blocking)
        else {
            // proxy_main.cc:482-521: kFinishedNoUpdates; from the embedder's
            // perspective the commit went through.
            completion.Signal();
            let source_frame_number = main.SourceFrameNumber();
            drop(main);
            client.DidCommit(
                source_frame_number,
                CommitTimestamps {
                    start: TimeTicks::default(),
                    finish: client.Now(),
                },
            );
            return MainFrameAnswer::Aborted(
                CommitEarlyOutReason::kFinishedNoUpdates,
                main_thread_start_time,
                true,
            );
        };
        commit_state.trace_id =
            crate::commit::commit_types::BeginMainFrameTraceId(state.trace_id as _);
        // block_on_next_commit_ (proxy_main.cc:563-564) is never set before
        // the input path wires SetInputResponsePending (decision D5); the
        // embedder would have to wait after posting the commit.
        debug_assert!(!blocking);
        MainFrameAnswer::ReadyToCommit {
            commit_state: Box::new(commit_state),
            lease,
            main_thread_start_time,
            scroll_and_viewport_changes_synced,
            commit_timeout,
        }
    }
}
