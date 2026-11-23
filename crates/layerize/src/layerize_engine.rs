//! `LayerizeEngine`: the owner of the main-thread compositor state
//! (MAIN_IMPL_SPLIT.md S1a-3 B3).
//!
//! Chromium keeps PaintArtifactCompositor, the previously pushed artifact,
//! the cc::ScrollCallbacks client and the main-frame deferral state on the
//! main thread (local_frame_view.cc:3253,3348; proxy_main.cc:282-307,
//! 793-817). This engine owns exactly those objects. During S1a the
//! compositor's EngineCore holds it and calls it at the points where it ran
//! the main half itself, on the same thread and in the same order; the main
//! LayerTreeHost is still a compositor owner, so each call borrows it as
//! `MainLayerHosts`. In S1b Page owns this engine.
use crate::{
    adapter::main_layer::{LayerContext, MainLayerHosts},
    commit::{input_types::ElementId, layerize_family::LayerizeFamily, main_effects::PacScrollEffect},
    runtime::{
        main_frame_lifecycle::MainFrameLifecycle,
        pac_scroll::{PacScrollCallbacks, PacScrollEffects},
        paint::{PaintArtifact, PaintInput, PaintInputError},
        paint_services::CpuMainPaintServices,
    },
    third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::PaintArtifactCompositor,
};
use std::{rc::Rc, sync::Arc};

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
    _scroll_callbacks: Rc<PacScrollCallbacks>,
    scroll_effects: PacScrollEffects,
}

impl<F: LayerizeFamily> LayerizeEngine<F> {
    /// Creates the canonical PAC with its scroll callbacks. The caller
    /// attaches `Pac().RootLayer()` to the main LayerTreeHost before
    /// admitting any producer commit (native PAC::Update returns while the
    /// root has no host).
    pub fn new(
        paint: Arc<CpuMainPaintServices<F::Skia>>,
        hosts: &mut dyn MainLayerHosts<F>,
    ) -> Self {
        let scroll_effects = PacScrollEffects::default();
        let callbacks = PacScrollCallbacks::new(scroll_effects.clone());
        let pac = {
            let mut context =
                LayerContext::new(hosts, paint.as_ref()).WithDerivedServices(paint.as_ref());
            paint.CreateCompositor(PacScrollCallbacks::Downgrade(&callbacks), &mut context)
        };
        Self {
            main_frame_lifecycle: Default::default(),
            paint,
            pac: Some(pac),
            previous_artifact: None,
            _scroll_callbacks: callbacks,
            scroll_effects,
        }
    }
    pub fn Paint(&self) -> &Arc<CpuMainPaintServices<F::Skia>> {
        &self.paint
    }
    pub fn Pac(&self) -> Option<&PaintArtifactCompositor<F>> {
        self.pac.as_ref()
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

    /// LocalFrameView::PushPaintArtifactToCompositor -> PAC::Update, against
    /// the previously pushed artifact.
    pub fn UpdateCompositor(
        &mut self,
        input: PaintInput,
        hosts: &mut dyn MainLayerHosts<F>,
        sequence: &mut i32,
    ) -> Result<(), PaintInputError> {
        let mut context =
            LayerContext::new(hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        self.paint.UpdateCompositor(
            self.pac.as_mut().unwrap(),
            input,
            self.previous_artifact.as_ref(),
            Vec::new(),
            &mut context,
            sequence,
        )
    }
    /// Records the artifact that now populates the resident PAC.
    pub fn SetPreviousArtifact(&mut self, artifact: Arc<PaintArtifact>) {
        self.previous_artifact = Some(artifact);
    }

    pub fn ApplyScrollbarPaintMutation(
        &self,
        element: ElementId,
        opacity: f32,
        thumb_width: i32,
        needs_repaint: bool,
        hosts: &mut dyn MainLayerHosts<F>,
    ) -> bool {
        let mut context =
            LayerContext::new(hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
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
        &self,
        update: &::paint::CompositedAnimationPropertyUpdate,
        hosts: &mut dyn MainLayerHosts<F>,
    ) -> bool {
        let mut context =
            LayerContext::new(hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        self.paint
            .ApplyCompositedAnimationProperties(self.pac.as_ref().unwrap(), update, &mut context)
    }

    /// PAC half of ApplySourceScrollAnimation.
    pub fn DropCompositorScrollDeltaNextCommit(
        &self,
        element: ElementId,
        hosts: &mut dyn MainLayerHosts<F>,
    ) {
        let mut context =
            LayerContext::new(hosts, self.paint.as_ref()).WithDerivedServices(self.paint.as_ref());
        if let Some(pac) = self.pac.as_ref() {
            pac.DropCompositorScrollDeltaNextCommit(element, &mut context);
        }
    }

    /// LayerTreeHostClient::UpdateCompositorScrollState joins the same FIFO
    /// as PAC's offset callbacks, so it cannot overtake the final delta.
    pub fn PushScrollEffect(&self, effect: PacScrollEffect) {
        self.scroll_effects.Push(effect)
    }
    pub fn PopScrollEffect(&self) -> Option<PacScrollEffect> {
        self.scroll_effects.Pop()
    }

    /// Native child client finalization needs the current main tree; the
    /// caller detaches the main root afterwards.
    pub fn Shutdown(&mut self, hosts: &mut dyn MainLayerHosts<F>) {
        if let Some(pac) = self.pac.take() {
            let mut context = LayerContext::new(hosts, self.paint.as_ref())
                .WithDerivedServices(self.paint.as_ref());
            pac.Destroy(&mut context);
        }
    }
}
