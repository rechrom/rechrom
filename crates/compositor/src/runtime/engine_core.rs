//! Private owner assembly for the impl half of the canonical commit path.
//!
//! No intermediate owner or raster work escapes through the Engine API. The
//! source Scheduler calls the action methods below with its current live client;
//! no independent policy chooses activation or draws an unfinished pending tree.
//! Since S1b-7 the main half (LayerizeEngine, ProxyMain) lives on the
//! document thread; this core receives its commits only through ProxyImpl.
use super::{
    host_services::{CpuHostServices, HostInit},
    layer_tree_frame_sink_endpoint::FrameSinkEndpoint,
    resources::RasterReturnedResource,
    skia::SkRegion,
    types::CpuCompositorTypes,
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs, client_layer_tree_host_impl::ClientHostContext,
        layer_tree_host_impl::HostImplContext,
    },
    cc::{
        scheduler::draw_result::DrawResult,
        trees::{client_layer_tree_host_impl::ClientLayerTreeHostImpl, frame_data::FrameData},
    },
    engine::{CompositorError, CompositorInputEffect},
    performance::{CompositorPackage, Span},
    CompositorFrameSubmission,
};
use std::sync::{Arc, Mutex};
type Root = CpuCompositorTypes<SkRegion>;

/// All constructor inputs are actual owners/capabilities, never test stand-ins.
/// This is private to compositor: Page only provides executors/pixel primitives.
pub(crate) struct EngineCore {
    // ProxyImpl state: the posted DataForCommit
    // and the ProxyImpl -> ProxyMain task queue.
    pub(crate) proxy_impl: super::proxy_impl::ProxyImpl,
    pub(crate) host: Option<ClientLayerTreeHostImpl<Root>>,
    pub(crate) services: CpuHostServices<Root>,
    lifecycle: Arc<super::host_lifecycle::HostLifecycle>,
    pub(crate) scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
    pub(crate) frame_sink: Arc<Mutex<FrameSinkEndpoint>>,
    // Source frame identity of the last commit consumed by
    // ProxyImpl::ScheduledActionCommit.  The FrameSink separately records the
    // source frame used by DrawLayers, so the output boundary can distinguish
    // an old-active-tree frame drawn immediately before pending-tree
    // activation from the frame produced by the latest commit.
    pub(crate) last_committed_source_frame_number: Option<i32>,
    // The synchronous InputHandlerProxy results of the current input call.
    pub(crate) input_effects: Vec<CompositorInputEffect>,
    pub(crate) failure: Option<CompositorError>,
}
impl EngineCore {
    pub(crate) fn new(
        mut services: CpuHostServices<Root>,
        init: HostInit<Root>,
        lifecycle: Arc<super::host_lifecycle::HostLifecycle>,
        scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
        frame_sink: Arc<Mutex<FrameSinkEndpoint>>,
        input_client: std::sync::Arc<super::input_client::QueuedInputHandlerClient>,
    ) -> Result<Self, CompositorError> {
        let mut host = services.CreateClientHost(init);
        host.BaseMutable().CreateInputHandler(&mut HostImplContext {
            services: &mut services,
        });
        host.BaseMutable().WithInputHandler(
            &mut HostImplContext {
                services: &mut services,
            },
            |input, runtime| {
                let input = input.expect("source bound InputHandler");
                input.BindToClient(runtime, input_client);
                // Source: blink::InputHandlerProxy construction calls
                // UpdateElasticOverscroll immediately after BindToClient.
                let _ = input.CreateScrollElasticityHelper(runtime);
            },
        );
        // Frame-sink initialization is performed by the canonical Scheduler
        // creation action, after the actual scheduler query scope exists.
        Ok(Self {
            proxy_impl: super::proxy_impl::ProxyImpl::new(Default::default()),
            host: Some(host),
            services,
            lifecycle,
            scheduler_queries,
            frame_sink,
            last_committed_source_frame_number: None,
            input_effects: Vec::new(),
            failure: None,
        })
    }

    /// ProxyImpl::SetVisibleOnImpl (proxy_impl.cc:325-329): the host half;
    /// the caller updates the Scheduler.
    pub(crate) fn SetHostVisible(&mut self, visible: bool) {
        self.host.as_mut().unwrap().BaseMutable().SetVisible(
            visible,
            &mut HostImplContext {
                services: &mut self.services,
            },
        );
    }

    pub(crate) fn HasReadyCommit(&self) -> bool {
        self.proxy_impl.HasDataForCommit()
    }
    pub(crate) fn PostCommit(&mut self) {
        self.host
            .as_mut()
            .unwrap()
            .CommitComplete(&mut ClientHostContext {
                services: &mut self.services,
            });
    }
    pub(crate) fn Activate(&mut self) {
        let _trace = Span(CompositorPackage::Tree, "Compositor.Tree.Activate");
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .ActivateSyncTree(&mut HostImplContext {
                services: &mut self.services,
            });
    }
    pub(crate) fn PrepareTiles(&mut self) {
        let _trace = Span(CompositorPackage::Tile, "Compositor.Tile.Prepare");
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .PrepareTiles(&mut HostImplContext {
                services: &mut self.services,
            });
    }
    pub(crate) fn SetFullViewportDamage(&mut self) {
        self.host
            .as_mut()
            .expect("a redraw requires a resident compositor host")
            .BaseMutable()
            .SetFullViewportDamage(&mut HostImplContext {
                services: &mut self.services,
            });
    }
    pub(crate) fn WillBeginImplFrame(&mut self, args: &BeginFrameArgs) -> bool {
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .WillBeginImplFrame(
                args,
                &mut HostImplContext {
                    services: &mut self.services,
                },
            )
    }
    pub(crate) fn DidFinishImplFrame(&mut self, args: &BeginFrameArgs) {
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .DidFinishImplFrame(
                args,
                &mut HostImplContext {
                    services: &mut self.services,
                },
            );
    }
    /// Source Proxy draw ordering. Scheduler decides whether a forced draw is
    /// permitted; the same canonical draw path owns preparation/submission.
    // Source: cc/trees/proxy_impl.cc:951-1048. This is only a call-scoped
    // owner bridge: every draw/animation/tree algorithm remains canonical.
    pub(crate) fn Draw(
        &mut self,
        forced: bool,
        scheduler: &mut crate::cc::scheduler::scheduler::Scheduler,
    ) -> DrawResult {
        let _inside = self.lifecycle.EnterDraw();
        let host = self.host.as_mut().unwrap().BaseMutable();
        let mut context = HostImplContext {
            services: &mut self.services,
        };
        let mut frame = FrameData::<Root>::default();
        frame.begin_frame_ack = scheduler.CurrentBeginFrameAckForActiveTree();
        frame.origin_begin_main_frame_args = scheduler.last_activate_origin_frame_args().clone();
        let mut draw_frame = false;
        let mut result = DrawResult::kAbortedCantDraw;
        let can_draw = host.CanDraw(&context);
        if can_draw {
            let mut trace = Span(CompositorPackage::Draw, "Compositor.Draw.Prepare");
            trace.set("forced", forced as u8 as f64);
            result = host.PrepareToDraw(&mut frame, false, &mut context);
            trace.set("result", result as i32 as f64);
            draw_frame = forced || result == DrawResult::kSuccess;
        }
        if draw_frame {
            let mut trace = Span(CompositorPackage::Frame, "Compositor.Frame.Build");
            trace.set("forced", forced as u8 as f64);
            if let Some(mut submit) = host.DrawLayers(&mut frame, &mut context) {
                scheduler.DidSubmitCompositorFrame(&mut submit);
                trace.set("submitted", 1.0);
            } else {
                trace.set("submitted", 0.0);
            }
            result = DrawResult::kSuccess;
        }
        {
            let _trace = Span(CompositorPackage::Draw, "Compositor.Draw.Finalize");
            host.DidDrawAllLayers(&frame, &mut context);
        }
        {
            let mut trace = Span(
                CompositorPackage::Animation,
                "Compositor.Animation.UpdateState",
            );
            trace.set("drew_frame", draw_frame as u8 as f64);
            host.UpdateAnimationState(draw_frame, &mut context);
        }
        if host.pending_tree().is_some() {
            let _trace = Span(
                CompositorPackage::Draw,
                "Compositor.Draw.UpdatePendingProperties",
            );
            host.WithPendingTree(&mut context, |tree, c| {
                tree.UpdateDrawProperties(true, true, None, c);
            });
        }
        result
    }
    pub(crate) fn ReclaimResources(&mut self, resources: Vec<RasterReturnedResource>) {
        let mut trace = Span(CompositorPackage::Resource, "Compositor.Resource.Reclaim");
        trace.set("resources", resources.len() as f64);
        self.host.as_mut().unwrap().BaseMutable().ReclaimResources(
            resources,
            &mut HostImplContext {
                services: &mut self.services,
            },
        );
    }
    pub(crate) fn TakeFrame(&self) -> (Option<CompositorFrameSubmission>, bool) {
        let _trace = Span(CompositorPackage::Frame, "Compositor.Frame.TakeSubmission");
        let sink = self.frame_sink.lock().expect("native frame sink");
        let frame = sink.TakeFrame();
        let matches_latest_commit = frame.is_some()
            && self.last_committed_source_frame_number == Some(sink.source_frame_number());
        (frame, matches_latest_commit)
    }
}
impl EngineCore {
    pub(crate) fn Shutdown(&mut self) {
        // A commit ProxyImpl still holds is dropped with it: its lease
        // signals the main thread's protected sequence as abandoned
        // (ProxyMain::Stop destroys ProxyImpl before the main host).
        self.proxy_impl.DropCommits();
        if let Some(mut host) = self.host.take() {
            host.BaseMutable()
                .ReleaseLayerTreeFrameSink(&mut HostImplContext {
                    services: &mut self.services,
                });
            host.Destroy(&mut HostImplContext {
                services: &mut self.services,
            });
        }
    }
}


// Source: cc/trees/proxy_impl.cc:784-799,801-949; proxy_impl.h:219.
// Scheduler owns every action decision. A fresh mutable Engine client is lent
// on each call, matching Proxy's synchronous source callbacks without retaining
// a Host/client pointer in Scheduler or external services.
impl crate::cc::scheduler::scheduler::SchedulerClient for EngineCore {
    type Types = crate::adapter::scheduler_host::FacadeSchedulerTypes;
    fn WillBeginImplFrame(&mut self, args: &BeginFrameArgs, scheduler: &mut Scheduler) -> bool {
        let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
        let redraw_sequence = self.lifecycle.redraw_sequence();
        let has_damage = EngineCore::WillBeginImplFrame(self, args);
        let requested_redraw = self.lifecycle.redraw_sequence() != redraw_sequence;
        // Source: cc/trees/proxy_impl.cc:394-396. LayerTreeHostImpl calls
        // ProxyImpl::SetWaitingForScrollEvent from WillBeginImplFrame and the
        // proxy updates Scheduler synchronously, before Scheduler chooses this
        // frame's deadline. The Rust host delegate also records a FIFO effect
        // because most callbacks must outlive the current Host borrow; mirror
        // this one state value here while the SchedulerClient callback still
        // owns Scheduler so WAIT_FOR_SCROLL is selected for the current frame.
        scheduler.SetWaitingForScrollEvent(self.lifecycle.waiting_for_scroll_event());
        // Source: cc/trees/proxy_impl.cc:550-554. Queued input is delivered by
        // LayerTreeHostImpl::WillBeginImplFrame. ScrollUpdate can call
        // ProxyImpl::SetNeedsRedrawOnImplThread from inside that same callback,
        // and Chromium updates Scheduler synchronously before it selects this
        // frame's deadline. The Rust delegate records the call while Host is
        // borrowed, so mirror the resident state here while this callback owns
        // Scheduler. The queued lifecycle effect remains useful to the outer
        // owner and is idempotent when it is drained later.
        if requested_redraw {
            scheduler.SetNeedsRedraw(self);
        }
        // ProxyImpl's SetNeedsRedrawOnImplThread callback is synchronous in
        // Chromium. Our typed delegate records it while Host is borrowed and
        // mirrors it here. Treat the same-turn redraw as damage as well;
        // otherwise Scheduler::BeginImplFrame calls AbortDraw after an impl
        // animation (notably overlay-scrollbar opacity) explicitly requested
        // a frame, and the mutated active tree is never submitted.
        has_damage || requested_redraw
    }
    fn ScheduledActionSendBeginMainFrame(
        &mut self,
        args: &BeginFrameArgs,
        scheduler: &mut Scheduler,
    ) {
        // Impl state only (proxy_impl.cc:801-848): ProxyImpl pulls the
        // compositor deltas and mutator events and posts ProxyMain::
        // BeginMainFrame to the main thread. NotifyBeginMainFrameStarted
        // follows with the main start time (proxy_impl.cc:318,440).
        let _ = scheduler;
        self.lifecycle.DidBeginMainFrame();
        self.SendBeginMainFrame(args);
    }
    fn ScheduledActionDrawIfPossible(&mut self, scheduler: &mut Scheduler) -> DrawResult {
        let result = {
            let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
            self.Draw(false, scheduler)
        };
        // Source: cc/trees/proxy_impl.cc SetNeedsPrepareTilesOnImplThread.
        // UpdateDrawProperties can discover newly exposed tiles while the
        // Scheduler is inside DRAW. Chromium records that request directly in
        // Scheduler, so its current deadline runs PREPARE_TILES after DRAW.
        // The Rust delegate also leaves the typed FIFO effect for the outer
        // owner; mirror its live state here before this Scheduler callback
        // returns, otherwise raster starts one complete BeginFrame late.
        if self.lifecycle.needs_prepare_tiles() {
            scheduler.SetNeedsPrepareTiles(self);
        }
        result
    }
    fn ScheduledActionDrawForced(&mut self, scheduler: &mut Scheduler) -> DrawResult {
        let result = {
            let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
            self.Draw(true, scheduler)
        };
        if self.lifecycle.needs_prepare_tiles() {
            scheduler.SetNeedsPrepareTiles(self);
        }
        result
    }
    fn ScheduledActionCommit(&mut self, scheduler: &mut Scheduler) {
        {
            let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
            self.ScheduledActionCommitOnImpl();
        }
    }
    fn ScheduledActionPostCommit(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.PostCommit();
    }
    fn ScheduledActionActivateSyncTree(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.Activate();
        // LayerTreeHostImpl -> ProxyImpl::DidActivateSyncTree releases a
        // commit held for activation.
        self.DidActivateSyncTreeOnImpl();
        // Source: cc/trees/proxy_impl.cc:521-525,909-917. The source
        // OnCanDrawStateChanged delegate updates Scheduler synchronously inside
        // activation. Refresh from the same resident Host before its current
        // ProcessScheduledActions loop may choose DRAW_ABORT; the queued outer
        // notification alone would arrive after that decision.
        let can_draw = self
            .host
            .as_ref()
            .unwrap()
            .Base()
            .CanDraw(&HostImplContext {
                services: &mut self.services,
            });
        drop(_query);
        _scheduler.SetCanDraw(can_draw, self);
    }
    fn ScheduledActionBeginLayerTreeFrameSinkCreation(&mut self, scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
        let sink = self.frame_sink.lock().expect("native frame sink").id();
        let initialized = self.host.as_ref().unwrap().Base().layer_tree_frame_sink() == Some(sink)
            || self
                .host
                .as_mut()
                .unwrap()
                .BaseMutable()
                .InitializeFrameSink(
                    sink,
                    &mut HostImplContext {
                        services: &mut self.services,
                    },
                );
        drop(_query);
        if initialized {
            scheduler.DidCreateAndInitializeLayerTreeFrameSink(self);
        } else {
            self.failure = Some(CompositorError::NotReady {
                stage: "native frame sink bind",
            });
        }
    }
    fn ScheduledActionPrepareTiles(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.PrepareTiles();
    }
    fn ScheduledActionInvalidateLayerTreeFrameSink(
        &mut self,
        needs_redraw: bool,
        _scheduler: &mut Scheduler,
    ) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.host.as_mut().unwrap().InvalidateLayerTreeFrameSink(
            needs_redraw,
            &mut ClientHostContext {
                services: &mut self.services,
            },
        );
    }
    fn ScheduledActionPerformImplSideInvalidation(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.host
            .as_mut()
            .unwrap()
            .InvalidateContentOnImplSide(&mut ClientHostContext {
                services: &mut self.services,
            });
    }
    fn DidFinishImplFrame(&mut self, args: &BeginFrameArgs, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        EngineCore::DidFinishImplFrame(self, args)
    }
    fn DidNotProduceFrame(
        &mut self,
        ack: crate::adapter::scheduler_host::BeginFrameAck,
        reason: crate::cc::scheduler::scheduler::FrameSkippedReason,
        _scheduler: &mut Scheduler,
    ) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .DidNotProduceFrame(
                &ack,
                reason,
                &mut HostImplContext {
                    services: &mut self.services,
                },
            );
    }
    fn WillNotReceiveBeginFrame(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .DidNotNeedBeginFrame(&mut HostImplContext {
                services: &mut self.services,
            });
    }
    fn DidChangeBeginFrameSourcePaused(&mut self, paused: bool, _scheduler: &mut Scheduler) {
        // ProxyImpl -> ProxyMain::DidChangeBeginFrameSourcePaused.
        self.proxy_impl.PostToMain(
            layerize::commit::proxy::ImplToMain::DidChangeBeginFrameSourcePaused(paused),
        );
    }
    // Source: cc/trees/proxy_impl.h:219 is an empty inline override.
    fn FrameIntervalUpdated(
        &mut self,
        _interval: crate::adapter::time::TimeDelta,
        _scheduler: &mut Scheduler,
    ) {
    }
    fn OnBeginImplFrameDeadline(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.host
            .as_mut()
            .unwrap()
            .BaseMutable()
            .OnBeginImplFrameDeadline(&mut HostImplContext {
                services: &mut self.services,
            });
    }
}
use crate::cc::scheduler::scheduler::Scheduler;

impl Drop for EngineCore {
    fn drop(&mut self) {
        self.Shutdown();
    }
}
