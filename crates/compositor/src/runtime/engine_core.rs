//! Private owner assembly for the canonical main/impl commit path.
//!
//! No intermediate owner or raster work escapes through the Engine API. The
//! source Scheduler calls the action methods below with its current live client;
//! no independent policy chooses activation or draws an unfinished pending tree.
use super::{
    host_services::{CpuHostServices, HostInit},
    layer_tree_frame_sink_endpoint::FrameSinkEndpoint,
    execution::EngineMainHostEffects,
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
    CompositorFrameSubmission,
};
use paint::PaintArtifact;
use std::sync::{Arc, Mutex};
type Root = CpuCompositorTypes<SkRegion>;

// Compile-time thread-safety of the main-half values that cross to, or are
// read by, the impl thread during the commit (S1B_PROXY_DESIGN.md B4): the
// main LayerTreeHost (lent inside the protected sequence), its
// ThreadUnsafeCommitState (read by FinishCommit) and the moved CommitState.
#[allow(dead_code)]
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_send_ref<T: Send + ?Sized>(_: &T) {}
    fn assertions(engine: &layerize::LayerizeEngine<Root>) {
        assert_send_ref(engine.Main());
        assert_sync::<layerize::commit::commit_state::ThreadUnsafeCommitState<Root>>();
        assert_send::<CommitState<Root>>();
    }
};

/// All constructor inputs are actual owners/capabilities, never test stand-ins.
/// This is private to compositor: Page only provides executors/pixel primitives.
pub(crate) struct EngineCore {
    // The two Proxy halves (S1B_PROXY_DESIGN.md S1b-4): ProxyMain holds the
    // main side of the open main frame and its prepared or ready commit,
    // ProxyImpl the posted DataForCommit. Declared before `layerize` so that
    // the commit leases they may hold are released before the main host.
    pub(crate) proxy_main: layerize::runtime::proxy_main::ProxyMain<Root>,
    pub(crate) proxy_impl: super::proxy_impl::ProxyImpl,
    // The main half: PAC, previous artifact, scroll callbacks, main paint
    // services and the main-frame deferral state (MAIN_IMPL_SPLIT.md B3).
    pub(crate) layerize: layerize::LayerizeEngine<Root>,
    pub(crate) host: Option<ClientLayerTreeHostImpl<Root>>,
    pub(crate) services: CpuHostServices<Root>,
    // The impl side of the main host's Proxy endpoint (the main AnimationHost
    // is owned by `layerize` since S1b-5).
    pub(crate) main_effects: Arc<EngineMainHostEffects>,
    lifecycle: Arc<super::host_lifecycle::HostLifecycle>,
    pub(crate) scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
    pub(crate) frame_sink: Arc<Mutex<FrameSinkEndpoint>>,
    // Source frame identity of the last commit consumed by
    // ProxyImpl::ScheduledActionCommit.  The FrameSink separately records the
    // source frame used by DrawLayers, so the output boundary can distinguish
    // an old-active-tree frame drawn immediately before pending-tree
    // activation from the frame produced by the commit that Page submitted.
    pub(crate) last_committed_source_frame_number: Option<i32>,
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
impl EngineCore {
    pub(crate) fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<PaintArtifact>,
        state: &paint::paint_property_tree::PropertyTreeState,
    ) -> crate::engine::ImmutablePaintRecord {
        self.layerize
            .ConvertPaintArtifactSubsetToRecord(artifact, state)
    }

    pub(crate) fn CreateVideoSurfaceForeignLayer(
        &mut self,
        range: crate::runtime::types::SurfaceRange,
    ) -> Arc<dyn paint::ForeignLayer> {
        self.layerize.CreateVideoSurfaceForeignLayer(range)
    }

    pub(crate) fn SetViewportState(
        &mut self,
        viewport: CompositorViewportState,
    ) -> Result<(), CompositorError> {
        viewport.Validate()?;
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
        self.layerize.MainMut().SetViewportRectAndScale(
            Rect::new(0, 0, viewport.size.width(), viewport.size.height()),
            viewport.device_scale_factor,
            self.local_surface_id,
        );
        self.layerize.MainMut().SetPageScaleFactorAndLimits(
            viewport.page_scale_factor,
            viewport.min_page_scale_factor,
            viewport.max_page_scale_factor,
        );
        self.host.as_mut().unwrap().BaseMutable().SetVisible(
            viewport.visible,
            &mut HostImplContext {
                services: &mut self.services,
            },
        );
        Ok(())
    }

    // Rechrom-only bridge for blocker B1 (S1B_PROXY_DESIGN.md 3), deleted in
    // S1b-7: the Page finished an artifact before the Scheduler dispatched the
    // BeginMainFrame that will own it. The impl deltas are pulled
    // (proxy_impl.cc:801-814; a ready commit puts them in the pipelined
    // slot) and applied to the main host (LayerTreeHost::
    // ApplyCompositorChanges) before PAC snapshots the main tree, as
    // single_thread_proxy.cc:1175-1190 orders them.
    pub(crate) fn ApplyCompositorDeltasOutsideMainFrame(&mut self) {
        let next_bmf = self.HasReadyCommit();
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
        if let Some(scale) = self.layerize.ApplyCompositorChanges(&mut data) {
            use layerize::runtime::proxy_main::ProxyMainClient;
            super::proxy_impl::InProcessMainClient {
                effects: &mut self.effects,
            }
            .ViewportScaleChanged(scale, &data);
        }
    }

    /// Creates the main half: the CPU main paint services, the main
    /// LayerTreeHost with its Proxy endpoint and the PAC, whose root it
    /// attaches before any producer commit (native PAC::Update returns while
    /// the root has no host).
    pub(crate) fn CreateLayerize(
        features: layerize::commit::main_paint_config::CpuPaintFeatures,
        diagnostics: layerize::commit::main_paint_config::SkiaPaintDiagnostics,
        init: layerize::commit::main_effects::MainHostInit<
            <Root as layerize::commit::CommitStateTypes>::MutatorHostHandle,
        >,
    ) -> Result<layerize::LayerizeEngine<Root>, CompositorError> {
        layerize::LayerizeEngine::<Root>::new(features, diagnostics, init)
            .map_err(|error| CompositorError::Backend(std::io::Error::other(error.to_string())))
    }

    pub(crate) fn new(
        mut services: CpuHostServices<Root>,
        init: HostInit<Root>,
        layerize: layerize::LayerizeEngine<Root>,
        main_effects: Arc<EngineMainHostEffects>,
        lifecycle: Arc<super::host_lifecycle::HostLifecycle>,
        scheduler_queries: Arc<super::engine_queries::CallScopedSchedulerQueries>,
        frame_sink: Arc<Mutex<FrameSinkEndpoint>>,
        local_surface_id: LocalSurfaceId,
        next_property_owner: u64,
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
                // Creation remains inside compositor and borrows the same
                // resident HostImpl; no tree or Page state crosses the edge.
                let _ = input.CreateScrollElasticityHelper(runtime);
            },
        );
        let result = Self {
            proxy_main: layerize::runtime::proxy_main::ProxyMain::new(Default::default()),
            proxy_impl: super::proxy_impl::ProxyImpl::new(),
            host: Some(host),
            services,
            main_effects,
            lifecycle,
            scheduler_queries,
            layerize,
            frame_sink,
            last_committed_source_frame_number: None,
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
    pub(crate) fn PreparePaintCommit(
        &mut self,
        artifact: Arc<PaintArtifact>,
        viewport: CompositorViewportState,
    ) -> Result<(), CompositorError> {
        viewport.Validate()?;
        if self.HasReadyCommit() || self.proxy_main.HasPreparedCommit() {
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
        // Validate all producer metadata once, before changing resident owner
        // state, and carry that same live input through PAC update. Rebuilding
        // PaintInput below would repeat the complete artifact/property walk.
        let paint_input = self.layerize.PrepareInput(artifact.clone())?;
        self.SetViewportState(viewport)?;
        // PAC::Update plus the canvas-child recording publication of this
        // source frame (main half, LayerizeEngine).
        let publication =
            self.layerize
                .UpdatePaintArtifact(&artifact, paint_input, &mut self.sequence)?;
        if let Some(publication) = publication {
            self.effects
                .push(CompositorMainThreadEffect::CanvasChildPaintRecords(
                    publication,
                ));
        }
        drop(layerize_trace);

        // A new PaintArtifact requests UPDATE_LAYERS.
        self.proxy_main.PrepareCommit(true);
        Ok(())
    }
    pub(crate) fn HasPreparedPaintCommit(&self) -> bool {
        self.proxy_main.HasPreparedCommit()
    }

    pub(crate) fn CanPrepareCompositedAnimationCommit(&self) -> bool {
        self.layerize.HasResidentPaintArtifact()
            && !self.HasReadyCommit()
            && !self.proxy_main.HasPreparedCommit()
    }

    pub(crate) fn ApplyScrollbarPaintMutation(
        &mut self,
        mutation: &crate::engine::ScrollbarPaintMutation,
    ) {
        let applied = self.layerize.ApplyScrollbarPaintMutation(
            mutation.element,
            mutation.opacity,
            mutation.thumb_width,
            mutation.needs_repaint,
        );
        if !applied {
            self.effects
                .push(CompositorMainThreadEffect::ScrollbarPaintFallback {
                    element: mutation.element,
                });
        }
    }

    pub(crate) fn HasResidentPaintArtifact(&self) -> bool {
        self.layerize.HasResidentPaintArtifact()
    }

    /// Apply Blink's deferred animated property values to the resident PAC
    /// main tree. The current BeginMainFrame still owns final commit
    /// preparation, so FinishMainFrameWithoutUpdates completes this prepared
    /// property-only commit instead of aborting the transaction.
    pub(crate) fn PrepareCompositedAnimationCommit(
        &mut self,
        update: &paint::CompositedAnimationPropertyUpdate,
    ) -> Result<(), CompositorError> {
        if !self.layerize.HasResidentPaintArtifact() {
            return Err(CompositorError::NotReady {
                stage: "composited animation requires a resident paint artifact",
            });
        }
        if self.HasReadyCommit() || self.proxy_main.HasPreparedCommit() {
            return Err(CompositorError::NotReady {
                stage: "previous main commit awaits scheduler",
            });
        }
        if !self.layerize.ApplyCompositedAnimationProperties(update) {
            return Err(CompositorError::NotReady {
                stage: "resident compositor property mapping is unavailable",
            });
        }
        // Blink's direct transform/opacity path calls only SetNeedsCommit and
        // must retain the existing PictureLayer recording/raster source.
        self.proxy_main.PrepareCommit(false);
        // Direct PAC updates call LayerTreeHost::SetNeedsCommit. When this
        // mutation belongs to the currently open BMF, that request is already
        // being fulfilled by its property-only commit; suppress a redundant
        // BMF request while retaining the request for out-of-frame callers.
        if self.proxy_main.IsInBeginMainFrame() {
            self.layerize.DidBeginCommit();
        }
        Ok(())
    }

    pub(crate) fn ClassifyScrollAnimationAreas(
        &self,
        areas: &[crate::adapter::input_types::ElementId],
    ) -> Vec<crate::engine::ScrollAnimationAreaClassification> {
        use crate::adapter::main_layer::MainLayerHostRead;
        use crate::cc::input::main_thread_scrolling_reason::{
            MainThreadRepaintReason, MainThreadRepaintReasons,
        };
        let tree = self.layerize.Main().property_trees().scroll_tree();
        areas
            .iter()
            .map(|element| {
                let reasons = tree
                    .FindNodeFromElementId(*element)
                    .map(|node| node.main_thread_repaint_reasons)
                    .unwrap_or_else(|| {
                        let mut reasons = MainThreadRepaintReasons::new();
                        reasons.Put(MainThreadRepaintReason::kPreferNonCompositedScrolling);
                        reasons
                    });
                crate::engine::ScrollAnimationAreaClassification {
                    element: *element,
                    uses_composited_scrolling: tree.FindNodeFromElementId(*element).is_some(),
                    reasons,
                }
            })
            .collect()
    }
    pub(crate) fn FinalizePreparedCommit(
        &mut self,
        features: &dyn PropertyTreeFeatures,
    ) -> Result<(), CompositorError> {
        assert!(
            self.proxy_main.HasPreparedCommit(),
            "same PAC preparation before finalizing native animation commit"
        );
        let update_layers = self.proxy_main.PreparedCommitUpdatesLayers();
        let mut tree_trace = Span(CompositorPackage::Tree, "Compositor.Tree.BuildCommit");
        tree_trace.set(
            "property_owner_sequence",
            self.property_owner_sequence as f64,
        );
        tree_trace.set("update_layers", update_layers as u8 as f64);
        let property_owner = PropertyTreesOwnerId::FromOwnerValue(self.property_owner_sequence);
        self.property_owner_sequence = self
            .property_owner_sequence
            .checked_add(1)
            .expect("native property-tree identity exhausted");
        // UpdateLayers (when requested), WillCommit, ActivateCommitState and
        // the source frame advance run on the main half; ProxyMain posts the
        // commit (or holds it for the next BeginMainFrame). The caller runs
        // the posted tasks.
        self.proxy_main
            .FinalizePreparedCommit(&mut self.layerize, features, property_owner);
        drop(tree_trace);
        Ok(())
    }

    /// A finished main commit exists: held by ProxyMain for the next
    /// BeginMainFrame, or posted to ProxyImpl and not yet committed.
    pub(crate) fn HasReadyCommit(&self) -> bool {
        self.proxy_main.HasReadyCommit() || self.proxy_impl.HasDataForCommit()
    }

    /// ProxyMain::SetNeedsCommit (Page main-frame demand).
    pub(crate) fn RequestMainFrame(&mut self, scheduler: &mut Scheduler) {
        self.proxy_main.SetNeedsCommit();
        self.RunProxyTasks(scheduler);
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
    pub(crate) fn DrainScrollEffects(&self, output: &mut Vec<CompositorMainThreadEffect>) {
        while let Some(effect) = self.layerize.PopScrollEffect() {
            output.push(effect.into());
        }
    }
}
impl EngineCore {
    pub(crate) fn Shutdown(&mut self) {
        // Native child client finalization needs the current actual main tree;
        // clear clients before detaching/dropping the main root allocation.
        self.layerize.Shutdown();
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
        // Impl state only: ProxyImpl pulls the compositor deltas and mutator
        // events and posts ProxyMain::BeginMainFrame, which the in-process
        // transport runs right here (it applies them to the main host, then
        // answers with the held commit or asks the Page for its lifecycle).
        self.lifecycle.DidBeginMainFrame();
        scheduler.NotifyBeginMainFrameStarted(self.services.core.MonotonicTimeNow());
        self.SendBeginMainFrame(args);
        self.RunProxyTasks(scheduler);
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
        // ProxyMain::DidCompleteCommit (posted by the completion event).
        self.RunProxyTasks(scheduler);
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
        self.RunProxyTasks(_scheduler);
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
impl EngineCore {
    // Source: cc/trees/proxy_main.cc:482-521 -> proxy_impl.cc:306-323. A main
    // frame which applied compositor deltas but produced no new PaintArtifact
    // still has to finish its Proxy transaction so SyncedProperty can retire
    // the reflected slot.
    pub(crate) fn FinishMainFrameWithoutUpdates(&mut self, scheduler: &mut Scheduler) {
        self.proxy_main.FinishMainFrameWithoutUpdates();
        self.RunProxyTasks(scheduler);
    }
}

impl Drop for EngineCore {
    fn drop(&mut self) {
        self.Shutdown();
    }
}

impl EngineCore {
    pub(crate) fn ExpireCommitDeferral(&mut self) {
        if let Some(reason) = self
            .layerize
            .MainFrameLifecycleMut()
            .Expire(self.services.core.MonotonicTimeNow())
        {
            self.effects
                .push(CompositorMainThreadEffect::CommitDeferralChanged {
                    deferred: false,
                    reason,
                });
        }
    }
    /// ProxyMain's deferred-update early-outs (proxy_main.cc:281-310,
    /// 380-427), posted to ProxyImpl::BeginMainFrameAbortedOnImpl, or a
    /// retained commit request outside a BeginMainFrame.
    pub(crate) fn AbortDeferredMainFrame(
        &mut self,
        scheduler: &mut Scheduler,
        reason: crate::engine::CommitDeferralReason,
    ) {
        self.proxy_main.AbortDeferredMainFrame(reason);
        self.RunProxyTasks(scheduler);
        self.effects
            .push(CompositorMainThreadEffect::MainFrameCommitDeferred { reason });
    }
}
