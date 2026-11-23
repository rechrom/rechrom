//! `LayerizeEngine`: the owner of the main-thread compositor state
//! (MAIN_IMPL_SPLIT.md S1a-3 B3).
//!
//! Chromium keeps the LayerTreeHost, PaintArtifactCompositor, the previously
//! pushed artifact, the cc::ScrollCallbacks client and the main-frame
//! deferral state on the main thread (local_frame_view.cc:3253,3348;
//! layer_tree_host.cc; proxy_main.cc:282-307,793-817). This engine owns
//! exactly those objects and the ProxyMain endpoint of the LayerTreeHost.
//! During S1a the compositor's EngineCore holds it and calls it at the points
//! where it ran the main half itself, on the same thread and in the same
//! order. In S1b Page owns this engine.
use crate::{
    adapter::main_layer::{LayerContext, MainLayerHostId},
    commit::{
        commit_state::CommitState,
        completion_event::CompletionEvent,
        compositor_commit_data::CompositorCommitData,
        input_types::ElementId,
        layerize_error::{LayerizeError, LayerizeErrorKind},
        layerize_family::LayerizeFamily,
        main_effects::{MainHostInit, PacScrollEffect},
        main_mutator::MainMutatorHost,
        main_paint_config::{CpuPaintFeatures, SkiaPaintDiagnostics},
        paint::canvas_child_record::CanvasChildRecordReplay,
        proxy::ProtectedCommitLease,
        property_tree_services::{PropertyTreeFeatures, PropertyTreesOwnerId},
        skia_region::SkRegion,
        CommitTypes,
    },
    runtime::{
        main_endpoint::EngineMainEndpoint,
        main_frame_lifecycle::MainFrameLifecycle,
        main_host::{
            CpuMainHostEndpoint, CpuMainLayerHost, CpuMainLayerHosts,
            ViewTransitionElementResourceRects,
        },
        pac_scroll::{PacScrollCallbacks, PacScrollEffects},
        paint::{
            CpuPaintRecord, CpuPaintValueServices, NewCpuMainPaintServices, PaintArtifact,
            PaintChunkSubset, PaintConfigurationError, PaintInput, PropertyTreeState,
        },
        paint_services::CpuMainPaintServices,
    },
    third_party::blink::renderer::platform::graphics::compositing::{
        paint_artifact_compositor::PaintArtifactCompositor, paint_chunks_to_cc_layer,
    },
};
use std::{ptr::NonNull, sync::Arc};

/// Owner of the main LayerTreeHost allocation. The protected commit lease
/// (`commit::proxy::ProtectedCommitLease`) and this owner derive their
/// accesses from the same pointer. Every owner-side access goes through the
/// single waiting accessor `Get` / `GetMut`, which first waits for the
/// protected sequence (LayerTreeHost::WaitForProtectedSequenceCompletion,
/// decision D2; S1B_PROXY_DESIGN.md R1).
struct MainHostOwner<F: LayerizeFamily>(NonNull<CpuMainLayerHost<F>>);
impl<F: LayerizeFamily> MainHostOwner<F> {
    fn new(host: Box<CpuMainLayerHost<F>>) -> Self {
        Self(NonNull::from(Box::leak(host)))
    }
    fn Wait(endpoint: &EngineMainEndpoint) {
        use crate::commit::protected_sequence_synchronizer::ProtectedSequenceSynchronizer;
        endpoint.WaitForProtectedSequenceCompletion();
        debug_assert!(endpoint.ImplOnOwnerSequence() || !endpoint.InProtectedSequence());
    }
    fn Get(&self, endpoint: &EngineMainEndpoint) -> &CpuMainLayerHost<F> {
        Self::Wait(endpoint);
        // SAFETY: owned allocation; the impl side holds no lease after the
        // wait above.
        unsafe { self.0.as_ref() }
    }
    fn GetMut(&mut self, endpoint: &EngineMainEndpoint) -> &mut CpuMainLayerHost<F> {
        Self::Wait(endpoint);
        // SAFETY: as in `Get`; `&mut self` excludes other owner-side borrows.
        unsafe { self.0.as_mut() }
    }
    fn Pointer(&self) -> NonNull<CpuMainLayerHost<F>> {
        self.0
    }
}
impl<F: LayerizeFamily> Drop for MainHostOwner<F> {
    fn drop(&mut self) {
        // SAFETY: allocated by `new`; LayerizeEngine::drop waited for any
        // outstanding commit lease first.
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }
}

// Compile-time thread-safety of the family-independent main-half owners the
// impl thread reads inside the protected sequence (S1B_PROXY_DESIGN.md B4).
// The family-bound assertions are in the compositor, which names the family.
#[allow(dead_code)]
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assertions() {
        assert_send_sync::<EngineMainEndpoint>();
        assert_send_sync::<CpuMainPaintServices<SkRegion>>();
    }
};

pub struct LayerizeEngine<F: LayerizeFamily> {
    main_frame_lifecycle: MainFrameLifecycle,
    paint: Arc<CpuMainPaintServices<F::Skia>>,
    pac: Option<PaintArtifactCompositor<F>>,
    // Blink classifies each producer replacement against the artifact that
    // populated the resident PAC. This is the immutable engine input itself,
    // not a copied snapshot.
    previous_artifact: Option<Arc<PaintArtifact>>,
    // PAC stores only Weak. Retaining this exact strong owner is required for
    // source scroll/snap/scrollbar-hidden delivery throughout PAC's lifetime.
    _scroll_callbacks: Arc<PacScrollCallbacks>,
    scroll_effects: PacScrollEffects,
    // The main LayerTreeHost (declared after PAC: its root is detached in
    // Shutdown after PAC destruction). A stable allocation: the protected
    // commit lease (S1B_PROXY_DESIGN.md B4) points at it.
    main: MainHostOwner<F>,
    endpoint: Arc<EngineMainEndpoint>,
    // The main cc::AnimationHost (AnimationHost::CreateMainInstance), owned
    // here as Blink's LayerTreeView owns animation_host_; the main host holds
    // the same instance as ThreadUnsafeCommitState::mutator_host, which the
    // impl side reads inside the protected sequence (MutatorHost::
    // PushPropertiesTo, layer_tree_impl.cc:814-817). S1b-5.
    mutator_host: F::MutatorHostHandle,
}

impl<F: LayerizeFamily> LayerizeEngine<F> {
    /// Creates the main LayerTreeHost and the canonical PAC with its scroll
    /// callbacks, then attaches `PAC::RootLayer()` to the host (native
    /// PAC::Update returns while the root has no host).
    pub fn WithPaintServices(
        paint: Arc<CpuMainPaintServices<F::Skia>>,
        init: MainHostInit<F::MutatorHostHandle>,
    ) -> Self {
        let endpoint = Arc::new(EngineMainEndpoint::new(init.sink, init.impl_on_owner_sequence));
        let mutator_host = init.mutator;
        let mut main = Box::new(CpuMainLayerHost::new(
            MainLayerHostId(init.id),
            init.owner,
            mutator_host.clone(),
            init.settings,
            init.threaded,
            endpoint.clone() as Arc<dyn CpuMainHostEndpoint>,
        ));
        if let Some(color) = init.background_color {
            main.pending_commit_state_mut().background_color = color;
        }
        let scroll_effects = PacScrollEffects::default();
        let callbacks = PacScrollCallbacks::new(scroll_effects.clone());
        let pac = {
            let mut hosts = CpuMainLayerHosts { host: &mut main };
            let mut context =
                LayerContext::<F>::new(&mut hosts, paint.as_ref()).WithDerivedServices(paint.as_ref());
            paint.CreateCompositor(PacScrollCallbacks::Downgrade(&callbacks), &mut context)
        };
        main.SetRootLayer(pac.RootLayer().cloned(), paint.as_ref(), paint.as_ref());
        Self {
            main_frame_lifecycle: Default::default(),
            paint,
            pac: Some(pac),
            previous_artifact: None,
            _scroll_callbacks: callbacks,
            scroll_effects,
            main: MainHostOwner::new(main),
            endpoint,
            mutator_host,
        }
    }
    pub fn Paint(&self) -> &Arc<CpuMainPaintServices<F::Skia>> {
        &self.paint
    }
    /// The shared cc/paint value capability the impl half's tile, raster and
    /// image-decode owners replay committed records with.
    pub fn PaintValues(&self) -> Arc<dyn CpuPaintValueServices<F::Skia>> {
        self.paint.PaintValues()
    }
    /// The main LayerTreeHost, for its plain LayerTreeHost setters/getters.
    pub fn Main(&self) -> &CpuMainLayerHost<F> {
        self.main.Get(&self.endpoint)
    }
    pub fn MainMut(&mut self) -> &mut CpuMainLayerHost<F> {
        self.main.GetMut(&self.endpoint)
    }
    pub fn MainFrameLifecycle(&self) -> &MainFrameLifecycle {
        &self.main_frame_lifecycle
    }
    pub fn MainFrameLifecycleMut(&mut self) -> &mut MainFrameLifecycle {
        &mut self.main_frame_lifecycle
    }
    pub fn HasResidentPaintArtifact(&self) -> bool {
        self.previous_artifact.is_some()
    }

    // ProxyMain endpoint state (cc/trees/proxy_main.cc
    // commit_requested_/SetNeedsCommit; layer_tree_host.cc:321-324).
    pub fn CommitRequested(&self) -> bool {
        self.endpoint.CommitRequested()
    }
    pub fn DidBeginCommit(&self) {
        self.endpoint.DidBeginCommit()
    }
    /// The main AnimationHost's SetNeedsCommit (MutatorHostClient ->
    /// LayerTreeHost -> ProxyMain::SetNeedsCommit).
    pub fn SetNeedsCommitFromMutatorHost(&self) {
        self.endpoint.SetNeedsCommit(false)
    }

    /// Validates all producer metadata once, before changing resident owner
    /// state; the same live input is carried through PAC update.
    pub fn PrepareInput(&self, artifact: Arc<PaintArtifact>) -> Result<PaintInput, LayerizeError> {
        Ok(self.paint.PrepareInput(artifact)?)
    }

    /// Source: third_party/blink/renderer/platform/graphics/compositing/
    /// paint_chunks_to_cc_layer.cc PaintChunksToCcLayer::Convert over a whole
    /// artifact (the document-thread video/drag-image recording).
    pub fn ConvertPaintArtifactSubsetToRecord(
        &self,
        artifact: Arc<PaintArtifact>,
        state: &::paint::paint_property_tree::PropertyTreeState,
    ) -> CpuPaintRecord<F::Skia> {
        let trees = artifact.property_trees.clone();
        let chunks = PaintChunkSubset::from_artifact(artifact);
        let state = PropertyTreeState::<F::Skia>::from_source_in(state, trees);
        paint_chunks_to_cc_layer::Convert::<F>(&chunks, &state, None, self.paint.as_ref())
    }

    /// Blink's video ForeignLayer, created through the resident main host.
    pub fn CreateVideoSurfaceForeignLayer(
        &mut self,
        range: <F as CommitTypes>::SurfaceRange,
    ) -> Arc<dyn ::paint::ForeignLayer> {
        self.main.GetMut(&self.endpoint)
            .CreateVideoSurfaceForeignLayer(self.paint.as_ref(), self.paint.as_ref(), range)
    }

    /// Main half of ProxyMain::BeginMainFrame's delta application
    /// (LayerTreeHost::ApplyCompositorChanges, layer_tree_host.cc:1207-1244).
    /// The client scroll state joins the same FIFO as PAC's offset callbacks,
    /// so it cannot overtake the final delta. Returns the main page scale
    /// factor when the viewport scale or inner viewport scroll changed.
    pub fn ApplyCompositorChanges(&mut self, data: &mut CompositorCommitData) -> Option<f32> {
        let scroll_state = self.main.GetMut(&self.endpoint).ApplyCompositorChanges(data);
        let viewport_changed =
            data.page_scale_delta != 1.0 || !data.inner_viewport_scroll.scroll_delta.IsZero();
        let scale = viewport_changed.then(|| self.main.GetMut(&self.endpoint).PageScaleFactor());
        self.scroll_effects
            .Push(PacScrollEffect::DidUpdateCompositorScrollState(scroll_state));
        scale
    }

    pub fn ApplyScrollbarPaintMutation(
        &mut self,
        element: ElementId,
        opacity: f32,
        thumb_width: i32,
        needs_repaint: bool,
    ) -> bool {
        let mut hosts = CpuMainLayerHosts {
            host: self.main.GetMut(&self.endpoint),
        };
        let mut context =
            LayerContext::<F>::new(&mut hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        self.pac.as_ref().is_some_and(|pac| {
            self.paint.ApplyScrollbarPaintMutation(
                pac,
                element,
                opacity,
                thumb_width,
                needs_repaint,
                &mut context,
            )
        })
    }

    pub fn ApplyCompositedAnimationProperties(
        &mut self,
        update: &::paint::CompositedAnimationPropertyUpdate,
    ) -> bool {
        let mut hosts = CpuMainLayerHosts {
            host: self.main.GetMut(&self.endpoint),
        };
        let mut context =
            LayerContext::<F>::new(&mut hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        self.paint
            .ApplyCompositedAnimationProperties(self.pac.as_ref().unwrap(), update, &mut context)
    }

    /// PAC half of ApplySourceScrollAnimation.
    pub fn DropCompositorScrollDeltaNextCommit(&mut self, element: ElementId) {
        let mut hosts = CpuMainLayerHosts {
            host: self.main.GetMut(&self.endpoint),
        };
        let mut context =
            LayerContext::<F>::new(&mut hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        if let Some(pac) = self.pac.as_ref() {
            pac.DropCompositorScrollDeltaNextCommit(element, &mut context);
        }
    }

    /// The main AnimationHost. It is read by the impl side's
    /// MutatorHost::PushPropertiesTo inside the protected sequence, so this
    /// is a waiting accessor like `Main()` (decision D2; cc::AnimationHost
    /// waits through MutatorHostClient::WaitForProtectedSequenceCompletion).
    pub fn MutatorHost(&self) -> &F::MutatorHostHandle {
        MainHostOwner::<F>::Wait(&self.endpoint);
        &self.mutator_host
    }

    /// Blink's scroll-offset animation update of the main AnimationHost,
    /// then MutatorHostClient::SetMutatorsNeedCommit -> LayerTreeHost::
    /// SetNeedsCommit and PAC's DropCompositorScrollDeltaNextCommit for an
    /// animation that replaces the compositor's delta.
    pub fn ApplySourceScrollAnimation(
        &mut self,
        mutation: scroll_animation::compositor_mutation::ScrollAnimationMutation,
    ) {
        let animations = self.MutatorHost();
        let drop_delta = animations.ApplySourceScrollAnimation(mutation);
        if animations.TakeNeedsCommit() {
            self.SetNeedsCommitFromMutatorHost();
        }
        if let Some(element) = drop_delta {
            self.DropCompositorScrollDeltaNextCommit(element);
        }
    }

    /// Blink CompositorAnimation updates of the main AnimationHost, then
    /// MutatorHostClient::SetMutatorsNeedCommit -> LayerTreeHost::
    /// SetNeedsCommit.
    pub fn ApplySourcePropertyAnimation(
        &mut self,
        mutation: compositor_animation::compositor_mutation::PropertyAnimationMutation,
    ) {
        let animations = self.MutatorHost();
        animations.ApplySourcePropertyAnimation(mutation);
        if animations.TakeNeedsCommit() {
            self.SetNeedsCommitFromMutatorHost();
        }
    }

    /// Source: cc/trees/layer_tree_host.cc:1254-1266
    /// (LayerTreeHost::ApplyMutatorEvents -> AnimationHost::
    /// SetAnimationEvents), called by ProxyMain::BeginMainFrame
    /// (proxy_main.cc:351-352). Returns the AnimationDelegate notifications
    /// of finished source scroll animations for the embedder.
    pub fn ApplyMutatorEvents(
        &mut self,
        events: Option<crate::commit::proxy::MutatorEvents<F>>,
    ) -> Vec<scroll_animation::compositor_mutation::ScrollAnimationCompletion> {
        let animations = self.MutatorHost();
        if let Some(events) = events {
            animations.SetAnimationEvents(events);
        }
        animations.TakeSourceScrollCompletions()
    }

    pub fn PopScrollEffect(&self) -> Option<PacScrollEffect> {
        self.scroll_effects.Pop()
    }

    pub fn NotifyTransitionRequestsFinished(
        &mut self,
        id: u32,
        rects: &ViewTransitionElementResourceRects<F>,
    ) {
        self.main.GetMut(&self.endpoint).NotifyTransitionRequestsFinished(id, rects)
    }

    /// LayerTreeHost::UpdateLayers (layer_tree_host.cc:958-972), run by
    /// ProxyMain::BeginMainFrame when the frame requested UPDATE_LAYERS
    /// (proxy_main.cc:441-455). Returns whether a layer's content updated.
    pub fn UpdateLayers(&mut self, features: &dyn PropertyTreeFeatures) -> bool {
        let main = self.main.GetMut(&self.endpoint);
        main.UpdateLayers(self.paint.as_ref(), self.paint.as_ref(), features)
    }

    /// Source: cc/trees/layer_tree_host.cc:424-503 (LayerTreeHost::
    /// WillCommit with ActivateCommitState). Without updates the activation
    /// is unwound and `None` returned (:470-483); otherwise persistent source
    /// frame numbering advances after the activated commit took its value
    /// (:487), `completion` is stored, which opens the protected sequence
    /// (:501), and the returned lease is the `&unsafe_state` ProxyMain posts
    /// with NotifyReadyToCommitOnImpl (proxy_main.cc:474-477).
    pub fn WillCommit(
        &mut self,
        has_updates: bool,
        owner: PropertyTreesOwnerId,
        completion: CompletionEvent,
        blocking: bool,
    ) -> Option<(CommitState<F>, ProtectedCommitLease<F>)> {
        let main = self.main.GetMut(&self.endpoint);
        main.WillCommit(self.paint.as_ref(), self.paint.as_ref());
        let state = main.ActivateCommitState(owner);
        if !has_updates {
            main.UnwindCommitState(state);
            return None;
        }
        main.pending_commit_state_mut().source_frame_number += 1;
        self.endpoint.BeginProtectedSequence(completion.clone());
        self.endpoint.DidBeginCommit();
        let lease = ProtectedCommitLease::new(
            self.main.Pointer(),
            self.paint.clone(),
            self.endpoint.clone(),
            completion,
            blocking,
        );
        Some((state, lease))
    }

    /// LayerTreeHost::SourceFrameNumber.
    pub fn SourceFrameNumber(&mut self) -> i32 {
        self.main.GetMut(&self.endpoint).pending_commit_state().source_frame_number
    }

    /// LayerTreeHost::WaitForCommitCompletion (layer_tree_host.cc:526-536),
    /// for ProxyMain's blocking commit (proxy_main.cc:563-564).
    pub fn WaitForCommitCompletion(&self) {
        MainHostOwner::<F>::Wait(&self.endpoint);
    }

    /// LayerTreeHost::CommitComplete (layer_tree_host.cc:552-570): retires
    /// the signaled commit completion on the owner sequence.
    pub fn CommitComplete(&self) {
        self.endpoint.CommitComplete();
    }

    /// Native child client finalization needs the current main tree; the
    /// main root is detached afterwards, before the host is dropped. Called
    /// after ProxyMain::Stop destroyed the impl half, so a commit it dropped
    /// is retired first instead of being reported as an impl failure.
    pub fn Shutdown(&mut self) {
        self.endpoint.WaitForCommitRelease();
        if let Some(pac) = self.pac.take() {
            let mut hosts = CpuMainLayerHosts {
                host: self.main.GetMut(&self.endpoint),
            };
            let mut context = LayerContext::<F>::new(&mut hosts, self.paint.as_ref())
                .WithDerivedServices(self.paint.as_ref());
            pac.Destroy(&mut context);
        }
        self.main.GetMut(&self.endpoint)
            .SetRootLayer(None, self.paint.as_ref(), self.paint.as_ref());
    }
}

impl<F: LayerizeFamily> Drop for LayerizeEngine<F> {
    /// The main host may not be freed while the impl side holds the commit
    /// lease (ProxyMain::Stop waits for the impl thread, proxy_main.cc:916-946).
    fn drop(&mut self) {
        self.endpoint.WaitForCommitRelease();
    }
}

/// The canvas-child raster input (`CanvasChildRecordReplay`) is the CPU
/// SkRegion recording the raster package downcasts to.
impl<F: LayerizeFamily<Skia = SkRegion>> LayerizeEngine<F> {
    /// The default CPU main half: the Skia paint platform services
    /// (`NewCpuMainPaintServices`) with the embedding's feature state, then
    /// the main host and PAC as in `WithPaintServices`.
    pub fn new(
        features: CpuPaintFeatures,
        diagnostics: SkiaPaintDiagnostics,
        init: MainHostInit<F::MutatorHostHandle>,
    ) -> Result<Self, PaintConfigurationError> {
        let paint = Arc::new(NewCpuMainPaintServices(features, diagnostics)?);
        Ok(Self::WithPaintServices(paint, init))
    }

    /// LocalFrameView::PushPaintArtifactToCompositor -> PAC::Update against
    /// the previously pushed artifact, then the publication of PAC's
    /// canvas-child recordings for this source frame. The artifact becomes
    /// the resident one.
    pub fn UpdatePaintArtifact(
        &mut self,
        artifact: &Arc<PaintArtifact>,
        input: PaintInput,
        sequence: &mut i32,
    ) -> Result<Option<Arc<canvas::CanvasChildRecordPublication>>, LayerizeError> {
        let had_canvas_children = self.pac.as_ref().unwrap().CanvasChildIds().next().is_some();
        let source_frame = u64::try_from(self.main.GetMut(&self.endpoint).pending_commit_state().source_frame_number)
            .expect("source main-frame number is nonnegative");
        let mut hosts = CpuMainLayerHosts {
            host: self.main.GetMut(&self.endpoint),
        };
        let mut context =
            LayerContext::<F>::new(&mut hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        self.paint.UpdateCompositor(
            self.pac.as_mut().unwrap(),
            input,
            self.previous_artifact.as_ref(),
            Vec::new(),
            &mut context,
            sequence,
        )?;
        let pac = self.pac.as_ref().unwrap();
        let publication = match artifact.canvas_document.clone() {
            Some(_) if !had_canvas_children && pac.CanvasChildIds().next().is_none() => None,
            Some(document) => {
                let mut records = Vec::new();
                for child in pac.CanvasChildIds() {
                    // Source getter may have no cached recording. It does not
                    // turn that case into an empty PaintRecord.
                    let Some(record) = pac.GetCanvasChildPaintRecordReadLease(child, &context)
                    else {
                        continue;
                    };
                    let canvas_id = u64::try_from(record.state.paint_state.canvas_node_id)
                        .map_err(|_| LayerizeError {
                            kind: LayerizeErrorKind::InvalidDomNodeId,
                            detail: "canvas child record has a negative canvas identity".into(),
                        })?;
                    let child_id = u64::try_from(child.0).map_err(|_| LayerizeError {
                        kind: LayerizeErrorKind::InvalidDomNodeId,
                        detail: "canvas child record has a negative element identity".into(),
                    })?;
                    records.push(canvas::draw_element_image::CanvasChildPaintRecord {
                        paint_state: record.state.paint_state.clone(),
                        record: Arc::new(CanvasChildRecordReplay {
                            list: record.list,
                            offset: record.offset,
                        }) as canvas::CanvasRecord,
                        canvas: document.Element(canvas_id),
                        element: document.Element(child_id),
                    });
                }
                Some(Arc::new(canvas::CanvasChildRecordPublication::new(
                    document,
                    source_frame,
                    records,
                )))
            }
            None => {
                if pac.CanvasChildIds().next().is_some() {
                    return Err(LayerizeError {
                        kind: LayerizeErrorKind::MissingCanvasDocument,
                        detail: "canvas child record publication requires its canonical document identity".into(),
                    });
                }
                None
            }
        };
        drop(context);
        self.previous_artifact = Some(artifact.clone());
        Ok(publication)
    }
}
