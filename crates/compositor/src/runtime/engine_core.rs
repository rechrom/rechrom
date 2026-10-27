//! Private owner assembly for the canonical main/impl commit path.
//!
//! No intermediate owner or raster work escapes through the Engine API. The
//! source Scheduler calls the action methods below with its current live client;
//! no independent policy chooses activation or draws an unfinished pending tree.
use super::{
    host_services::{CpuHostServices, HostInit},
    main_endpoint::EngineMainEndpoint,
    main_host::{CpuMainLayerHost, CpuMainLayerHosts},
    native_frame_sink::NativeFrameSinkEndpoint,
    native_pac_scroll::{NativePacScrollCallbacks, NativePacScrollEffects},
    paint_services::CpuPaintServices,
    resources::RasterReturnedResource,
    skia::SkRegion,
    types::{CpuCompositorTypes, LocalSurfaceId},
};
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        client_layer_tree_host_impl::ClientHostContext,
        geometry::Rect,
        layer_tree_host_impl::HostImplContext,
        main_layer::LayerContext,
        property_tree::{PropertyTreeFeatures, PropertyTreesOwnerId},
    },
    cc::{
        scheduler::draw_result::DrawResult,
        trees::{
            client_layer_tree_host_impl::ClientLayerTreeHostImpl, commit_state::CommitState,
            frame_data::FrameData,
        },
    },
    engine::{CompositorError, CompositorMainThreadEffect, CompositorViewportState},
    performance::{CompositorPackage, Span},
    third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor,
    NativeCompositorFrame, NativeCompositorFrameSubmission,
};
use paint::PaintArtifact;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};
type Root = CpuCompositorTypes<SkRegion>;

/// All constructor inputs are actual owners/capabilities, never test stand-ins.
/// This is private to compositor: Page only provides executors/pixel primitives.
pub(crate) struct NativeEngineCore {
    pub(crate) host: Option<ClientLayerTreeHostImpl<Root>>,
    pub(crate) services: CpuHostServices<Root>,
    pub(crate) main: CpuMainLayerHost<SkRegion>,
    pub(crate) paint: Arc<CpuPaintServices<SkRegion>>,
    pac: Option<PaintArtifactCompositor<Root>>,
    // Blink classifies each producer replacement against the artifact that
    // populated the resident PAC. This is the immutable engine input itself,
    // not a copied snapshot.
    previous_artifact: Option<Arc<PaintArtifact>>,
    pub(crate) endpoint: Arc<EngineMainEndpoint>,
    lifecycle: Arc<super::native_host_lifecycle::NativeHostLifecycle>,
    pub(crate) scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
    // PAC stores only Weak. Retaining this exact strong owner is required for
    // source scroll/snap/scrollbar-hidden delivery throughout PAC's lifetime.
    _scroll_callbacks: Rc<NativePacScrollCallbacks>,
    scroll_effects: NativePacScrollEffects,
    pub(crate) frame_sink: Arc<Mutex<NativeFrameSinkEndpoint>>,
    ready_commit: Option<CommitState<Root>>,
    // `ready_commit` is built on the Page sequence before the Scheduler may
    // open the BeginMainFrame that owns it. Keep the source distinction
    // between a prepared commit and ProxyImpl's already-notified
    // `data_for_commit_`; otherwise a re-entrant SEND_BEGIN_MAIN_FRAME can
    // notify the same commit twice.
    ready_commit_notified: bool,
    pub(crate) awaiting_main: bool,
    pub(crate) effects: Vec<CompositorMainThreadEffect>,
    pub(crate) failure: Option<CompositorError>,
    sequence: i32,
    property_owner_sequence: u64,
    local_surface_id: LocalSurfaceId,
    // The Page is the parent embedding this compositor surface. Chromium's
    // ParentLocalSurfaceIdAllocator advances the parent sequence whenever the
    // surface parameters change; UnboundedFrameSinkHandler deliberately keeps
    // a frame's output size fixed while that id is unchanged.
    // Source: components/viz/common/surfaces/
    //         parent_local_surface_id_allocator.cc:74-96;
    //         cc/trees/unbounded_frame_sink_handler.cc:56-77.
    surface_parameters: Option<(crate::adapter::geometry::Size, u32)>,
}
impl NativeEngineCore {
    // Source: cc/trees/single_thread_proxy.cc:1175-1190. This CPU runtime has
    // one sequenced Page executor, so the source single-thread Proxy ordering
    // is exact: pull impl deltas, then apply them to the resident main host
    // before asking the embedder to run its BeginMainFrame work.
    pub(crate) fn ApplyCompositorDeltas(&mut self) {
        // Source: cc/trees/proxy_impl.cc:801-814. A resident ready commit is
        // ProxyImpl's `data_for_commit_`: deltas pulled for the pipelined BMF
        // belong in SyncedProperty's next reflected slot.
        let next_bmf = self.ready_commit.is_some();
        let mut data = self
            .host
            .as_mut()
            .expect("resident Host")
            .BaseMutable()
            .ProcessCompositorDeltasForPipeline(
                next_bmf,
                &mut HostImplContext {
                    services: &mut self.services,
                },
            );
        self.main.ApplyCompositorChanges(&mut data);
    }

    pub(crate) fn new(
        mut services: CpuHostServices<Root>,
        init: HostInit<Root>,
        mut main: CpuMainLayerHost<SkRegion>,
        paint: Arc<CpuPaintServices<SkRegion>>,
        endpoint: Arc<EngineMainEndpoint>,
        lifecycle: Arc<super::native_host_lifecycle::NativeHostLifecycle>,
        scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
        frame_sink: Arc<Mutex<NativeFrameSinkEndpoint>>,
        local_surface_id: LocalSurfaceId,
        next_property_owner: u64,
        input_client: std::sync::Arc<super::native_input_client::NativeInputHandlerClient>,
    ) -> Result<Self, CompositorError> {
        let scroll_effects = NativePacScrollEffects::default();
        let callbacks = NativePacScrollCallbacks::new(scroll_effects.clone());
        let pac = {
            let mut hosts = CpuMainLayerHosts { host: &mut main };
            let mut context =
                LayerContext::new(&mut hosts, paint.as_ref()).WithDerivedServices(paint.as_ref());
            paint.CreateCompositor(
                NativePacScrollCallbacks::Downgrade(&callbacks),
                &mut context,
            )
        };
        // Native PAC::Update intentionally returns when the root has no host.
        // Attach the actual PAC root before admitting any producer commit.
        main.SetRootLayer(pac.RootLayer().cloned(), paint.as_ref(), paint.as_ref());
        let mut host = services.CreateClientHost(init);
        host.BaseMutable().CreateInputHandler(&mut HostImplContext {
            services: &mut services,
        });
        host.BaseMutable().WithInputHandler(
            &mut HostImplContext {
                services: &mut services,
            },
            |input, runtime| {
                input
                    .expect("source bound InputHandler")
                    .BindToClient(runtime, input_client)
            },
        );
        let result = Self {
            host: Some(host),
            services,
            main,
            paint,
            pac: Some(pac),
            previous_artifact: None,
            endpoint,
            lifecycle,
            scheduler_queries,
            _scroll_callbacks: callbacks,
            scroll_effects,
            frame_sink,
            ready_commit: None,
            ready_commit_notified: false,
            awaiting_main: false,
            effects: Vec::new(),
            failure: None,
            sequence: 0,
            property_owner_sequence: next_property_owner,
            local_surface_id,
            surface_parameters: None,
        };
        // Frame-sink initialization is performed by the canonical Scheduler
        // creation action, after the actual scheduler query scope exists.
        Ok(result)
    }

    /// This is the main phase of the native Proxy commit. It retains precisely
    /// the producer Arc during PAC Update; the resulting source commit owner is
    /// moved once to the impl phase, not converted to an artifact/frame plan.
    pub(crate) fn PrepareCommit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
        features: &dyn PropertyTreeFeatures,
    ) -> Result<(), CompositorError> {
        viewport.Validate()?;
        if self.ready_commit.is_some() {
            return Err(CompositorError::NotReady {
                stage: "previous main commit awaits scheduler",
            });
        }
        let mut layerize_trace = Span(
            CompositorPackage::Layerize,
            "Compositor.Layerize.PaintArtifact",
        );
        layerize_trace.set("sequence", self.sequence as f64);
        layerize_trace.set("display_items", artifact.display_items.len() as f64);
        layerize_trace.set("chunks", artifact.chunks.len() as f64);
        layerize_trace.set(
            "image_dependencies",
            artifact.image_dependencies.image_ids().count() as f64,
        );
        // Validate all producer metadata before changing resident owner state.
        self.paint.PrepareInput(artifact.clone())?;
        let surface_parameters = (viewport.size, viewport.device_scale_factor.to_bits());
        if self
            .surface_parameters
            .is_some_and(|previous| previous != surface_parameters)
        {
            // ParentLocalSurfaceIdAllocator::GenerateId increments only the
            // parent sequence and preserves the child sequence/embed token.
            self.local_surface_id.parent_sequence_number = self
                .local_surface_id
                .parent_sequence_number
                .checked_add(1)
                .expect("parent LocalSurfaceId sequence exhausted");
        }
        self.surface_parameters = Some(surface_parameters);
        self.main.SetViewportRectAndScale(
            Rect::new(0, 0, viewport.size.width(), viewport.size.height()),
            viewport.device_scale_factor,
            self.local_surface_id,
        );
        self.main.SetPageScaleFactorAndLimits(
            viewport.page_scale_factor,
            viewport.min_page_scale_factor,
            viewport.max_page_scale_factor,
        );
        {
            let mut hosts = CpuMainLayerHosts {
                host: &mut self.main,
            };
            let mut context = LayerContext::new(&mut hosts, self.paint.as_ref())
                .WithDerivedServices(self.paint.as_ref());
            self.paint.UpdateCompositor(
                self.pac.as_mut().unwrap(),
                artifact.clone(),
                self.previous_artifact.as_ref(),
                Vec::new(),
                &mut context,
                &mut self.sequence,
            )?;
        }
        self.previous_artifact = Some(artifact.clone());
        drop(layerize_trace);

        let mut tree_trace = Span(CompositorPackage::Tree, "Compositor.Tree.BuildCommit");
        tree_trace.set(
            "property_owner_sequence",
            self.property_owner_sequence as f64,
        );
        self.main
            .UpdateLayers(self.paint.as_ref(), self.paint.as_ref(), features);
        self.main
            .WillCommit(self.paint.as_ref(), self.paint.as_ref());
        let property_owner = PropertyTreesOwnerId::FromOwnerValue(self.property_owner_sequence);
        self.property_owner_sequence = self
            .property_owner_sequence
            .checked_add(1)
            .expect("native property-tree identity exhausted");
        self.ready_commit = Some(self.main.ActivateCommitState(property_owner));
        self.ready_commit_notified = false;
        drop(tree_trace);
        // Source: cc/trees/layer_tree_host.cc:487. Persistent source frame
        // numbering advances after the activated commit has taken its value.
        self.main.pending_commit_state_mut().source_frame_number += 1;
        self.endpoint.DidBeginCommit();
        self.host.as_mut().unwrap().BaseMutable().SetVisible(
            viewport.visible,
            &mut HostImplContext {
                services: &mut self.services,
            },
        );
        Ok(())
    }

    /// Called only for Scheduler's COMMIT action. The live main synchronization
    /// context borrows the very same resident ThreadUnsafeCommitState.
    pub(crate) fn Commit(&mut self) {
        let _trace = Span(CompositorPackage::Tree, "Compositor.Tree.Commit");
        assert!(self.ready_commit_notified);
        let mut state = self
            .ready_commit
            .take()
            .expect("source ready-to-commit state");
        self.ready_commit_notified = false;
        let host = self.host.as_mut().unwrap();
        let mut context = ClientHostContext {
            services: &mut self.services,
        };
        host.BeginCommit(state.source_frame_number, state.trace_id, &mut context);
        {
            let _protected = self.endpoint.EnterProtectedSequence();
            self.main.WithCommitContext(
                self.paint.as_ref(),
                self.paint.as_ref(),
                self.paint.as_ref(),
                |unsafe_state, main_context| {
                    host.FinishCommit(&mut state, unsafe_state, main_context, &mut context);
                },
            );
        }
    }
    pub(crate) fn HasReadyCommit(&self) -> bool {
        self.ready_commit.is_some()
    }
    pub(crate) fn ReadyCommitWasNotified(&self) -> bool {
        self.ready_commit_notified
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
    pub(crate) fn TakeFrame(&self) -> Option<NativeCompositorFrameSubmission> {
        let _trace = Span(CompositorPackage::Frame, "Compositor.Frame.TakeSubmission");
        self.frame_sink
            .lock()
            .expect("native frame sink")
            .TakeFrame()
    }
    pub(crate) fn DrainScrollEffects(&self, output: &mut Vec<CompositorMainThreadEffect>) {
        while let Some(effect) = self.scroll_effects.Pop() {
            output.push(effect.into());
        }
    }
}
impl NativeEngineCore {
    pub(crate) fn Shutdown(&mut self) {
        // Native child client finalization needs the current actual main tree;
        // clear clients before detaching/dropping the main root allocation.
        if let Some(pac) = self.pac.take() {
            let mut hosts = CpuMainLayerHosts {
                host: &mut self.main,
            };
            let mut context = LayerContext::new(&mut hosts, self.paint.as_ref())
                .WithDerivedServices(self.paint.as_ref());
            pac.Destroy(&mut context);
        }
        self.main
            .SetRootLayer(None, self.paint.as_ref(), self.paint.as_ref());
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
impl crate::cc::scheduler::scheduler::SchedulerClient for NativeEngineCore {
    type Types = crate::adapter::scheduler_host::FacadeSchedulerTypes;
    fn WillBeginImplFrame(&mut self, args: &BeginFrameArgs, scheduler: &mut Scheduler) -> bool {
        let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
        let redraw_sequence = self.lifecycle.redraw_sequence();
        let has_damage = NativeEngineCore::WillBeginImplFrame(self, args);
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
        if self.lifecycle.redraw_sequence() != redraw_sequence {
            scheduler.SetNeedsRedraw(self);
        }
        has_damage
    }
    fn ScheduledActionSendBeginMainFrame(
        &mut self,
        _args: &BeginFrameArgs,
        scheduler: &mut Scheduler,
    ) {
        // A Page artifact may already be in ready-to-commit state when the
        // Scheduler opens the next BeginMainFrame. Chromium keeps both
        // DataForCommit owners: the ready commit consumes the primary slot,
        // while deltas pulled below belong to the pipelined next BMF.
        let had_unnotified_commit = self.HasReadyCommit() && !self.ReadyCommitWasNotified();
        self.awaiting_main = true;
        self.lifecycle.DidBeginMainFrame();
        scheduler.NotifyBeginMainFrameStarted(self.services.core.MonotonicTimeNow());
        self.ApplyCompositorDeltas();
        if had_unnotified_commit {
            self.NotifyReadyToCommit(scheduler, false, true);
        } else {
            self.effects
                .push(CompositorMainThreadEffect::RequestMainFrame);
        }
    }
    fn ScheduledActionDrawIfPossible(&mut self, scheduler: &mut Scheduler) -> DrawResult {
        {
            let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
            self.Draw(false, scheduler)
        }
    }
    fn ScheduledActionDrawForced(&mut self, scheduler: &mut Scheduler) -> DrawResult {
        {
            let _query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
            self.Draw(true, scheduler)
        }
    }
    fn ScheduledActionCommit(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.Commit();
    }
    fn ScheduledActionPostCommit(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.PostCommit();
    }
    fn ScheduledActionActivateSyncTree(&mut self, _scheduler: &mut Scheduler) {
        let _query = self.scheduler_queries.Enter(_scheduler.IsReadyToActivate());
        self.Activate();
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
        NativeEngineCore::DidFinishImplFrame(self, args)
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
        self.effects
            .push(CompositorMainThreadEffect::BeginFrameSourcePaused { paused });
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
impl NativeEngineCore {
    pub(crate) fn NotifyReadyToCommit(
        &mut self,
        scheduler: &mut Scheduler,
        scroll_and_viewport_changes_synced: bool,
        completes_awaiting_main: bool,
    ) {
        assert!(self.awaiting_main && self.HasReadyCommit() && !self.ready_commit_notified);
        if completes_awaiting_main {
            self.awaiting_main = false;
        }
        self.ready_commit_notified = true;
        let query = self.scheduler_queries.Enter(scheduler.IsReadyToActivate());
        self.host.as_mut().unwrap().ReadyToCommit(
            scroll_and_viewport_changes_synced,
            None,
            false,
            &mut ClientHostContext {
                services: &mut self.services,
            },
        );
        drop(query);
        scheduler.NotifyReadyToCommit(None, self);
    }

    // Source: cc/trees/proxy_impl.cc:309-326. A main frame which applied
    // compositor deltas but produced no new PaintArtifact still has to finish
    // its Proxy transaction so SyncedProperty can retire the reflected slot.
    pub(crate) fn FinishMainFrameWithoutUpdates(&mut self, scheduler: &mut Scheduler) {
        assert!(self.awaiting_main);
        let next_bmf = self.HasReadyCommit();
        let args = scheduler.last_dispatched_begin_main_frame_args().clone();
        self.host.as_mut().unwrap().BeginMainFrameAborted(
            crate::cc::scheduler::commit_earlyout_reason::CommitEarlyOutReason::kFinishedNoUpdates,
            Vec::new(),
            &args,
            next_bmf,
            true,
            &mut ClientHostContext {
                services: &mut self.services,
            },
        );
        self.awaiting_main = false;
        scheduler.BeginMainFrameAborted(
            crate::cc::scheduler::commit_earlyout_reason::CommitEarlyOutReason::kFinishedNoUpdates,
            self,
        );
    }
}

impl Drop for NativeEngineCore {
    fn drop(&mut self) {
        self.Shutdown();
    }
}
